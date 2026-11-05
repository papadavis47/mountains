//! `--resync-backups`: headless cleanup that makes `~/.mountains/mtslog-*.md`
//! mirror the database exactly (pulling from Turso first when configured).

use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};

use crate::config::{self, AppConfig};
use crate::db_manager::DbManager;
use crate::file_manager::{FileManager, ResyncReport};

/// Where the resync read its days from, for the summary line.
#[derive(Debug, PartialEq)]
pub enum Source {
    Turso,
    Local,
}

pub async fn run(config: &AppConfig) -> Result<(PathBuf, Source, ResyncReport)> {
    let data_dir = config::data_dir()?;
    std::fs::create_dir_all(&data_dir).context("Failed to create .mountains directory")?;

    let mut db = DbManager::new_local_first(&data_dir).await?;

    let source = if config.sync.is_configured() {
        let db_path = data_dir.join("mountains.db");
        let db_path_str = db_path
            .to_str()
            .context("Failed to convert database path to string")?;
        db.upgrade_to_remote_replica(
            db_path_str,
            config.sync.db_url.clone(),
            config.sync.auth_token.clone(),
        )
        .await
        .context("Couldn't connect to Turso Cloud; no files were changed")?;
        // `upgrade_to_remote_replica` tolerates a failed pull; a stale replica
        // must never decide which backups to delete, so require a fresh pull.
        db.sync_now()
            .await
            .context("Couldn't pull from Turso Cloud; no files were changed")?;
        Source::Turso
    } else {
        Source::Local
    };

    let logs = db.load_all_daily_logs().await?;
    // Same folder as the DB, never a different one: a mismatch would prune
    // real backups against an unrelated (e.g. dev) database.
    let file_manager = FileManager::in_dir(data_dir.clone());

    if should_abort_empty(logs.len(), file_manager.backup_file_count()?) {
        bail!(
            "Database has no days but backup files exist; refusing to delete them all. \
             No files were changed."
        );
    }

    let report = file_manager.resync(&logs)?;
    Ok((data_dir, source, report))
}

/// An empty database alongside existing backups almost certainly means a
/// fresh or broken DB, not "delete everything".
fn should_abort_empty(db_days: usize, backup_files: usize) -> bool {
    db_days == 0 && backup_files > 0
}

pub fn summary(dir: &Path, source: &Source, report: &ResyncReport) -> String {
    let source = match source {
        Source::Turso => "Turso Cloud",
        Source::Local => "local db",
    };
    let mut out = format!(
        "Resynced backups in {} (source: {source})\n  \
         created:   {}\n  updated:   {}\n  unchanged: {}\n  removed:   {}",
        dir.display(),
        report.created,
        report.updated,
        report.unchanged,
        report.removed.len(),
    );
    if !report.removed.is_empty() {
        out.push_str(&format!("  ({})", report.removed.join(", ")));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aborts_only_when_db_empty_and_backups_exist() {
        assert!(should_abort_empty(0, 3));
        assert!(!should_abort_empty(0, 0));
        assert!(!should_abort_empty(5, 0));
        assert!(!should_abort_empty(5, 7));
    }

    #[test]
    fn summary_lists_removed_filenames() {
        let report = ResyncReport {
            created: 3,
            updated: 5,
            unchanged: 412,
            removed: vec![
                "mtslog-10.02.2026.md".to_string(),
                "mtslog-10.09.2026.md".to_string(),
            ],
        };
        assert_eq!(
            summary(Path::new("/home/u/.mountains"), &Source::Turso, &report),
            "Resynced backups in /home/u/.mountains (source: Turso Cloud)\n  \
             created:   3\n  updated:   5\n  unchanged: 412\n  \
             removed:   2  (mtslog-10.02.2026.md, mtslog-10.09.2026.md)"
        );
    }

    #[test]
    fn summary_omits_list_when_nothing_removed() {
        let out = summary(Path::new("/d"), &Source::Local, &ResyncReport::default());
        assert!(out.starts_with("Resynced backups in /d (source: local db)"));
        assert!(out.ends_with("removed:   0"));
    }
}
