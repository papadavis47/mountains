use crate::models::DailyLog;
use anyhow::{Context, Result};
use chrono::NaiveDate;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct FileManager {
    mountains_dir: PathBuf,
}

impl FileManager {
    /// Rooted at `config::data_dir()` (honors `MOUNTAINS_DATA_DIR`), so backups
    /// always live beside the database they mirror.
    pub fn new() -> Result<Self> {
        let mountains_dir = crate::config::data_dir()?;

        if !mountains_dir.exists() {
            fs::create_dir_all(&mountains_dir).context("Failed to create .mountains directory")?;
        }

        Ok(Self { mountains_dir })
    }

    /// Rooted at an explicit directory (e.g. `config::data_dir()`, which honors
    /// `MOUNTAINS_DATA_DIR`), so resync compares backups against the same folder
    /// the database lives in.
    pub fn in_dir(mountains_dir: PathBuf) -> Self {
        Self { mountains_dir }
    }

    fn get_file_path(&self, date: NaiveDate) -> PathBuf {
        let filename = format!("mtslog-{}.md", date.format("%m.%d.%Y"));
        self.mountains_dir.join(filename)
    }

    pub fn save_daily_log(&self, log: &DailyLog) -> Result<()> {
        let file_path = self.get_file_path(log.date);
        let content = self.daily_log_to_markdown(log);
        fs::write(&file_path, content)
            .context(format!("Failed to write to file: {:?}", file_path))?;
        Ok(())
    }

    fn daily_log_to_markdown(&self, log: &DailyLog) -> String {
        let mut content = String::new();

        content.push_str(&format!(
            "# Mountains Training Log - {}\n\n",
            log.date.format("%B %d, %Y")
        ));

        if log.weight.is_some() || log.waist.is_some() {
            content.push_str("## Measurements\n");
            if let Some(weight) = log.weight {
                content.push_str(&format!("- **Weight:** {} lbs\n", weight));
            }
            if let Some(waist) = log.waist {
                content.push_str(&format!("- **Waist:** {} inches\n", waist));
            }
            content.push('\n');
        }

        if !log.food_entries.is_empty() {
            content.push_str("## Food\n");
            for entry in &log.food_entries {
                content.push_str(&format!("- {}\n", entry.name));
            }
            content.push('\n');
        }

        if log.miles_covered.is_some() || log.elevation_gain.is_some() {
            content.push_str("## Running\n");
            if let Some(miles) = log.miles_covered {
                content.push_str(&format!("- **Miles:** {} mi\n", miles));
            }
            if let Some(elevation) = log.elevation_gain {
                content.push_str(&format!("- **Elevation:** {} ft\n", elevation));
            }
            content.push('\n');
        }

        if !log.sokay_entries.is_empty() {
            content.push_str("## Sokay\n");
            for entry in &log.sokay_entries {
                content.push_str(&format!("- {}\n", entry));
            }
            content.push('\n');
        }

        if let Some(strength_mobility) = &log.strength_mobility {
            content.push_str("## Strength & Mobility\n");
            content.push_str(strength_mobility);
            content.push('\n');
        }

        if let Some(notes) = &log.notes {
            content.push_str("## Notes\n");
            content.push_str(notes);
            content.push('\n');
        }

        content
    }

    pub fn delete_daily_log(&self, date: NaiveDate) -> Result<()> {
        let file_path = self.get_file_path(date);

        if file_path.exists() {
            fs::remove_file(&file_path)
                .context(format!("Failed to delete file: {:?}", file_path))?;
        }

        Ok(())
    }

    /// Makes the backup folder mirror `logs` exactly: writes missing files,
    /// rewrites stale ones, and removes `mtslog-*.md` files whose date isn't in
    /// `logs`. Files not named like a backup are never touched.
    pub fn resync(&self, logs: &[DailyLog]) -> Result<ResyncReport> {
        let mut report = ResyncReport::default();

        for log in logs {
            let path = self.get_file_path(log.date);
            let content = self.daily_log_to_markdown(log);
            match fs::read_to_string(&path) {
                Ok(existing) if existing == content => report.unchanged += 1,
                Ok(_) => {
                    fs::write(&path, content)
                        .with_context(|| format!("Failed to write to file: {:?}", path))?;
                    report.updated += 1;
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    fs::write(&path, content)
                        .with_context(|| format!("Failed to write to file: {:?}", path))?;
                    report.created += 1;
                }
                Err(e) => {
                    return Err(e).with_context(|| format!("Failed to read file: {:?}", path));
                }
            }
        }

        let dates: HashSet<NaiveDate> = logs.iter().map(|log| log.date).collect();
        for (date, path) in self.backup_files()? {
            if !dates.contains(&date) {
                fs::remove_file(&path)
                    .with_context(|| format!("Failed to delete file: {:?}", path))?;
                report.removed.push(file_name(&path));
            }
        }
        report.removed.sort();

        Ok(report)
    }

    /// Number of backup files currently in the folder.
    pub fn backup_file_count(&self) -> Result<usize> {
        Ok(self.backup_files()?.len())
    }

    /// Every file in the folder whose name is a backup filename, with its date.
    fn backup_files(&self) -> Result<Vec<(NaiveDate, PathBuf)>> {
        let entries = fs::read_dir(&self.mountains_dir)
            .with_context(|| format!("Failed to read directory: {:?}", self.mountains_dir))?;
        let mut files = Vec::new();
        for entry in entries {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                continue;
            }
            if let Some(date) = entry.file_name().to_str().and_then(parse_backup_filename) {
                files.push((date, entry.path()));
            }
        }
        Ok(files)
    }
}

/// Outcome of `FileManager::resync`.
#[derive(Debug, Default, PartialEq)]
pub struct ResyncReport {
    pub created: usize,
    pub updated: usize,
    pub unchanged: usize,
    /// Filenames of removed backups, sorted.
    pub removed: Vec<String>,
}

/// Inverse of `get_file_path`: only exact `mtslog-MM.DD.YYYY.md` names with a
/// real date qualify, so resync can never delete an unrelated file.
fn parse_backup_filename(name: &str) -> Option<NaiveDate> {
    let stem = name.strip_prefix("mtslog-")?.strip_suffix(".md")?;
    // Reject non-zero-padded variants chrono would otherwise accept.
    if stem.len() != 10 {
        return None;
    }
    let date = NaiveDate::parse_from_str(stem, "%m.%d.%Y").ok()?;
    (date.format("%m.%d.%Y").to_string() == stem).then_some(date)
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::FileManager;
    use std::path::Path;

    pub fn manager(path: &Path) -> FileManager {
        FileManager {
            mountains_dir: path.to_path_buf(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::FoodEntry;
    use tempfile::TempDir;

    // Build a FileManager rooted at a scratch dir instead of `~/.mountains`, so
    // tests never touch the real home directory. The struct field is private but
    // reachable here because tests live in the same module.
    fn manager(dir: &TempDir) -> FileManager {
        FileManager {
            mountains_dir: dir.path().to_path_buf(),
        }
    }

    fn full_log() -> DailyLog {
        let mut log = DailyLog::new(NaiveDate::from_ymd_opt(2025, 1, 9).unwrap());
        log.weight = Some(175.5);
        log.waist = Some(34.2);
        log.miles_covered = Some(3.2);
        log.elevation_gain = Some(450);
        log.add_food_entry(FoodEntry::new("Oatmeal".to_string()));
        log.add_food_entry(FoodEntry::new("Chicken Salad".to_string()));
        log.add_sokay_entry("Coca Cola".to_string());
        log.strength_mobility = Some("Pull-ups: 3x8".to_string());
        log.notes = Some("Feeling strong.".to_string());
        log
    }

    #[test]
    fn markdown_matches_documented_format() {
        let md = manager(&TempDir::new().unwrap()).daily_log_to_markdown(&full_log());
        assert!(md.starts_with("# Mountains Training Log - January 09, 2025\n\n"));
        assert!(
            md.contains("## Measurements\n- **Weight:** 175.5 lbs\n- **Waist:** 34.2 inches\n")
        );
        assert!(md.contains("## Food\n- Oatmeal\n- Chicken Salad\n"));
        assert!(md.contains("## Running\n- **Miles:** 3.2 mi\n- **Elevation:** 450 ft\n"));
        assert!(md.contains("## Sokay\n- Coca Cola\n"));
        assert!(md.contains("## Strength & Mobility\nPull-ups: 3x8\n"));
        assert!(md.contains("## Notes\nFeeling strong.\n"));
    }

    #[test]
    fn markdown_omits_empty_sections_but_keeps_header() {
        let log = DailyLog::new(NaiveDate::from_ymd_opt(2025, 1, 9).unwrap());
        let md = manager(&TempDir::new().unwrap()).daily_log_to_markdown(&log);
        assert!(md.starts_with("# Mountains Training Log - January 09, 2025"));
        for section in [
            "## Measurements",
            "## Food",
            "## Running",
            "## Sokay",
            "## Strength & Mobility",
            "## Notes",
        ] {
            assert!(!md.contains(section), "unexpected section: {section}");
        }
    }

    #[test]
    fn measurements_section_shown_when_only_one_field_set() {
        let mut log = DailyLog::new(NaiveDate::from_ymd_opt(2025, 1, 9).unwrap());
        log.weight = Some(180.0);
        let md = manager(&TempDir::new().unwrap()).daily_log_to_markdown(&log);
        assert!(md.contains("## Measurements\n- **Weight:** 180 lbs\n"));
        assert!(!md.contains("Waist"));
    }

    #[test]
    fn save_writes_dated_file_then_delete_removes_it() {
        let dir = TempDir::new().unwrap();
        let fm = manager(&dir);
        let log = full_log();
        fm.save_daily_log(&log).unwrap();

        let path = dir.path().join("mtslog-01.09.2025.md");
        assert!(path.exists());
        // File content is exactly the serialized markdown.
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            fm.daily_log_to_markdown(&log)
        );

        fm.delete_daily_log(log.date).unwrap();
        assert!(!path.exists());
        // Deleting a missing file is a no-op, not an error.
        assert!(fm.delete_daily_log(log.date).is_ok());
    }

    fn day(m: u32, d: u32) -> DailyLog {
        DailyLog::new(NaiveDate::from_ymd_opt(2026, m, d).unwrap())
    }

    #[test]
    fn parse_backup_filename_accepts_only_exact_backup_names() {
        assert_eq!(
            parse_backup_filename("mtslog-01.09.2025.md"),
            NaiveDate::from_ymd_opt(2025, 1, 9)
        );
        for bad in [
            "mtslog-13.40.2026.md",
            "mtslog-02.30.2026.md",
            "mtslog-1.9.2025.md",
            "mtslog-bogus.md",
            "mtslog-01.09.2025.md.bak",
            "notes.md",
            "config.toml",
            "mountains.db",
        ] {
            assert_eq!(parse_backup_filename(bad), None, "{bad}");
        }
    }

    #[test]
    fn resync_creates_updates_keeps_and_removes() {
        let dir = TempDir::new().unwrap();
        let fm = manager(&dir);

        let unchanged = day(1, 1);
        fm.save_daily_log(&unchanged).unwrap();

        let mut stale = day(1, 2);
        fm.save_daily_log(&stale).unwrap();
        stale.notes = Some("edited remotely".to_string());

        let missing = day(1, 3);
        fm.save_daily_log(&day(1, 4)).unwrap(); // orphan: not in DB

        let report = fm
            .resync(&[unchanged.clone(), stale.clone(), missing.clone()])
            .unwrap();

        assert_eq!(
            report,
            ResyncReport {
                created: 1,
                updated: 1,
                unchanged: 1,
                removed: vec!["mtslog-01.04.2026.md".to_string()],
            }
        );
        for log in [&unchanged, &stale, &missing] {
            let path = dir
                .path()
                .join(format!("mtslog-{}.md", log.date.format("%m.%d.%Y")));
            assert_eq!(
                std::fs::read_to_string(path).unwrap(),
                fm.daily_log_to_markdown(log)
            );
        }
        assert!(!dir.path().join("mtslog-01.04.2026.md").exists());
    }

    #[test]
    fn resync_never_touches_non_backup_files() {
        let dir = TempDir::new().unwrap();
        let fm = manager(&dir);
        let others = [
            "config.toml",
            "mountains.db",
            "notes.md",
            "mtslog-bogus.md",
            "mtslog-13.40.2026.md",
        ];
        for name in others {
            std::fs::write(dir.path().join(name), "keep").unwrap();
        }
        std::fs::create_dir(dir.path().join("mtslog-01.05.2026.md")).unwrap();

        let report = fm.resync(&[]).unwrap();

        assert!(report.removed.is_empty());
        for name in others {
            assert_eq!(
                std::fs::read_to_string(dir.path().join(name)).unwrap(),
                "keep"
            );
        }
        assert!(dir.path().join("mtslog-01.05.2026.md").is_dir());
        assert_eq!(fm.backup_file_count().unwrap(), 0);
    }
}
