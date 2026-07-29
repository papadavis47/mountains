use anyhow::{Context, Result};
use chrono::NaiveDate;
use libsql::{Builder, Connection, Database};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::models::{DailyLog, FoodEntry};

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connected,
    Error(String),
}

pub struct DbManager {
    db: Database,
    conn: Connection,
    connection_state: Arc<RwLock<ConnectionState>>,
}

impl DbManager {
    pub async fn new_local_first(data_dir: &Path) -> Result<Self> {
        // Get local database path
        let db_path = data_dir.join("mountains.db");
        let db_path_str = db_path
            .to_str()
            .context("Failed to convert database path to string")?
            .to_string();

        // Always start with local connection for instant startup
        let db = Builder::new_local(&db_path_str).build().await?;
        let conn = db.connect()?;

        // Start disconnected - will upgrade to cloud replica in background if credentials available
        let state = ConnectionState::Disconnected;

        let mut manager = Self {
            db,
            conn,
            connection_state: Arc::new(RwLock::new(state)),
        };

        // Always initialize schema (needed even for in-memory placeholder)
        manager.init_schema().await?;

        Ok(manager)
    }

    /// Upgrades local database to remote replica (recreates as libsql can't convert in-place).
    /// Local data is stashed aside and imported into the replica after the first successful
    /// pull, so enabling cloud sync never loses locally logged days.
    pub async fn upgrade_to_remote_replica(
        &mut self,
        db_path_str: &str,
        url: String,
        token: String,
    ) -> Result<()> {
        *self.connection_state.write().await = ConnectionState::Disconnected;

        // Check if metadata file exists (indicating this is already a replica)
        let metadata_path = format!("{}-info", db_path_str);
        let is_already_replica = Path::new(&metadata_path).exists();

        if !is_already_replica {
            // libsql cannot convert a local database to a remote replica, so the
            // local files must be moved out of the way; stash instead of delete
            // so their rows can be imported after the first pull
            self.stash_local_db(db_path_str).await;
        }

        // Create or connect to remote replica
        match Builder::new_remote_replica(db_path_str, url, token)
            .build()
            .await
        {
            Ok(new_db) => {
                match new_db.connect() {
                    Ok(new_conn) => {
                        // Replace the database connection
                        self.db = new_db;
                        self.conn = new_conn;

                        // Only reinitialize schema if we replaced the database
                        if !is_already_replica {
                            self.init_schema().await?;
                        }

                        // Pull anything written to the primary by other clients
                        // (e.g. the web app) while we were away. Stashed local
                        // data is only imported after a successful pull: the
                        // "date already exists remotely" check is meaningless
                        // against a replica that hasn't seen the primary yet.
                        // On failure the stash stays for retry on next connect.
                        if self.db.sync().await.is_ok() {
                            let _ = self.import_stashed_dbs(db_path_str).await;
                        }

                        *self.connection_state.write().await = ConnectionState::Connected;
                        Ok(())
                    }
                    Err(e) => {
                        *self.connection_state.write().await =
                            ConnectionState::Error(format!("Failed to connect: {}", e));
                        Err(e.into())
                    }
                }
            }
            Err(e) => {
                *self.connection_state.write().await =
                    ConnectionState::Error(format!("Failed to create replica: {}", e));
                Err(e.into())
            }
        }
    }

    pub async fn get_connection_state(&self) -> ConnectionState {
        self.connection_state.read().await.clone()
    }

    /// Moves the local database files aside before replica creation. The stash name
    /// is unique per attempt: if a previous upgrade attempt failed and left a stash,
    /// overwriting it with the current (recreated, near-empty) database would lose
    /// the original data.
    async fn stash_local_db(&self, db_path_str: &str) {
        let db_path = Path::new(db_path_str);
        if !db_path.exists() {
            return;
        }

        // Fold the WAL into the main file so the stash is self-contained
        let _ = self.conn.query("PRAGMA wal_checkpoint(TRUNCATE)", ()).await;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let stash_path = format!("{}.pre-sync.{}", db_path_str, timestamp);

        if std::fs::rename(db_path, &stash_path).is_ok() {
            std::fs::rename(
                format!("{}-wal", db_path_str),
                format!("{}-wal", stash_path),
            )
            .ok();
        } else {
            // Rename failed; fall back to removal so replica creation can proceed
            std::fs::remove_file(db_path).ok();
            std::fs::remove_file(format!("{}-wal", db_path_str)).ok();
        }
        std::fs::remove_file(format!("{}-shm", db_path_str)).ok();
    }

    /// Stashed pre-sync databases waiting to be imported, newest first.
    fn find_stashed_dbs(db_path_str: &str) -> Vec<PathBuf> {
        let db_path = Path::new(db_path_str);
        let (Some(dir), Some(name)) = (
            db_path.parent(),
            db_path.file_name().and_then(|n| n.to_str()),
        ) else {
            return Vec::new();
        };
        let prefix = format!("{}.pre-sync.", name);

        let mut stashes: Vec<PathBuf> = std::fs::read_dir(dir)
            .map(|entries| {
                entries
                    .flatten()
                    .filter_map(|entry| {
                        let file_name = entry.file_name();
                        let file_name = file_name.to_str()?;
                        (file_name.starts_with(&prefix)
                            && !file_name.ends_with("-wal")
                            && !file_name.ends_with("-shm"))
                        .then(|| entry.path())
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Newest first so the most recent session wins when stashes share a date
        stashes.sort();
        stashes.reverse();
        stashes
    }

    /// Imports daily logs from stashed pre-sync databases into the replica.
    /// Only dates the replica doesn't already have are inserted (remote wins on
    /// conflict), then each stash is removed. A failure leaves the remaining
    /// stashes in place for retry on the next connect.
    async fn import_stashed_dbs(&mut self, db_path_str: &str) -> Result<()> {
        let stashes = Self::find_stashed_dbs(db_path_str);
        if stashes.is_empty() {
            return Ok(());
        }

        let mut existing_dates = std::collections::HashSet::new();
        let mut rows = self.conn.query("SELECT date FROM daily_logs", ()).await?;
        while let Some(row) = rows.next().await? {
            existing_dates.insert(row.get::<String>(0)?);
        }

        for stash in stashes {
            let stash_str = stash.to_str().context("Invalid stash path")?;
            let stash_db = Builder::new_local(stash_str).build().await?;
            let stash_conn = stash_db.connect()?;
            let logs = Self::load_daily_logs_from(&stash_conn).await?;
            drop(stash_conn);
            drop(stash_db);

            for log in logs {
                let date_str = log.date.format("%Y-%m-%d").to_string();
                if existing_dates.contains(&date_str) {
                    continue;
                }
                self.save_daily_log(&log).await?;
                existing_dates.insert(date_str);
            }

            std::fs::remove_file(&stash).ok();
            std::fs::remove_file(format!("{}-wal", stash_str)).ok();
            std::fs::remove_file(format!("{}-shm", stash_str)).ok();
        }

        Ok(())
    }

    async fn init_schema(&mut self) -> Result<()> {
        // Create daily_logs table with all columns
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS daily_logs (
                    date TEXT PRIMARY KEY,
                    weight REAL,
                    waist REAL,
                    miles_covered REAL,
                    elevation_gain INTEGER,
                    strength_mobility TEXT,
                    notes TEXT
                )",
                (),
            )
            .await
            .context("Failed to create daily_logs table")?;

        // Create food_entries table
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS food_entries (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    date TEXT NOT NULL,
                    name TEXT NOT NULL,
                    FOREIGN KEY (date) REFERENCES daily_logs(date) ON DELETE CASCADE
                )",
                (),
            )
            .await
            .context("Failed to create food_entries table")?;

        // Create index on date for faster queries
        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_food_entries_date ON food_entries(date)",
                (),
            )
            .await
            .context("Failed to create index on food_entries")?;

        // Create sokay_entries table
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS sokay_entries (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    date TEXT NOT NULL,
                    entry_text TEXT NOT NULL,
                    FOREIGN KEY (date) REFERENCES daily_logs(date) ON DELETE CASCADE
                )",
                (),
            )
            .await
            .context("Failed to create sokay_entries table")?;

        // Create index on date for faster queries
        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_sokay_entries_date ON sokay_entries(date)",
                (),
            )
            .await
            .context("Failed to create index on sokay_entries")?;

        Ok(())
    }

    pub async fn save_daily_log(&mut self, log: &DailyLog) -> Result<()> {
        let date_str = log.date.format("%Y-%m-%d").to_string();

        // Start a transaction for atomic operations
        let tx = self.conn.transaction().await?;

        // Upsert daily_logs record
        tx.execute(
            "INSERT OR REPLACE INTO daily_logs (date, weight, waist, miles_covered, elevation_gain, strength_mobility, notes) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            libsql::params![
                date_str.clone(),
                log.weight,
                log.waist,
                log.miles_covered,
                log.elevation_gain,
                log.strength_mobility.as_deref(),
                log.notes.as_deref(),
            ],
        )
        .await
        .context("Failed to save daily log")?;

        // Delete existing food entries for this date
        tx.execute(
            "DELETE FROM food_entries WHERE date = ?1",
            [date_str.as_str()],
        )
        .await
        .context("Failed to delete old food entries")?;

        // Insert all food entries
        for entry in &log.food_entries {
            tx.execute(
                "INSERT INTO food_entries (date, name) VALUES (?1, ?2)",
                libsql::params![date_str.clone(), entry.name.clone(),],
            )
            .await
            .context("Failed to insert food entry")?;
        }

        // Delete existing sokay entries for this date
        tx.execute(
            "DELETE FROM sokay_entries WHERE date = ?1",
            [date_str.as_str()],
        )
        .await
        .context("Failed to delete old sokay entries")?;

        // Insert all sokay entries
        for entry in &log.sokay_entries {
            tx.execute(
                "INSERT INTO sokay_entries (date, entry_text) VALUES (?1, ?2)",
                libsql::params![date_str.clone(), entry.clone(),],
            )
            .await
            .context("Failed to insert sokay entry")?;
        }

        // Commit the transaction
        tx.commit().await.context("Failed to commit transaction")?;

        // Trigger manual sync after save
        self.sync().await;

        Ok(())
    }

    pub async fn load_all_daily_logs(&self) -> Result<Vec<DailyLog>> {
        Self::load_daily_logs_from(&self.conn).await
    }

    async fn load_daily_logs_from(conn: &Connection) -> Result<Vec<DailyLog>> {
        // Query all dates from daily_logs
        let mut rows = conn
            .query(
                "SELECT date, weight, waist, miles_covered, elevation_gain, strength_mobility, notes FROM daily_logs ORDER BY date DESC",
                (),
            )
            .await
            .context("Failed to query all daily logs")?;

        let mut daily_logs = Vec::new();

        while let Some(row) = rows.next().await? {
            let date_str: String = row.get(0)?;
            let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                .context("Failed to parse date from database")?;

            let weight: Option<f32> = row.get::<Option<f64>>(1)?.map(|v| v as f32);
            let waist: Option<f32> = row.get::<Option<f64>>(2)?.map(|v| v as f32);
            let miles_covered: Option<f32> = row.get::<Option<f64>>(3)?.map(|v| v as f32);
            let elevation_gain: Option<i32> = row.get::<Option<i64>>(4)?.map(|v| v as i32);
            let strength_mobility: Option<String> = row.get(5)?;
            let notes: Option<String> = row.get(6)?;

            // Query food entries for this date
            let mut food_rows = conn
                .query(
                    "SELECT name FROM food_entries WHERE date = ?1 ORDER BY id",
                    [date_str.as_str()],
                )
                .await
                .context("Failed to query food entries")?;

            let mut food_entries = Vec::new();
            while let Some(food_row) = food_rows.next().await? {
                let name: String = food_row.get(0)?;
                food_entries.push(FoodEntry::new(name));
            }

            // Query sokay entries for this date
            let mut sokay_rows = conn
                .query(
                    "SELECT entry_text FROM sokay_entries WHERE date = ?1 ORDER BY id",
                    [date_str.as_str()],
                )
                .await
                .context("Failed to query sokay entries")?;

            let mut sokay_entries = Vec::new();
            while let Some(sokay_row) = sokay_rows.next().await? {
                let entry_text: String = sokay_row.get(0)?;
                sokay_entries.push(entry_text);
            }

            daily_logs.push(DailyLog {
                date,
                food_entries,
                weight,
                waist,
                miles_covered,
                elevation_gain,
                sokay_entries,
                strength_mobility,
                notes,
            });
        }

        Ok(daily_logs)
    }

    /// Best-effort sync after save/delete operations
    async fn sync(&self) {
        // Only sync if we're connected to Turso
        let state = self.connection_state.read().await;
        if *state != ConnectionState::Connected {
            return; // Skip sync if not connected
        }
        drop(state); // Release lock before sync

        let _ = self.db.sync().await; // Ignore sync errors - best effort
    }

    /// Explicit sync with Turso Cloud (called on shutdown)
    pub async fn sync_now(&self) -> Result<()> {
        // Only sync if we're connected to Turso
        let state = self.connection_state.read().await;
        if *state != ConnectionState::Connected {
            return Ok(()); // Skip sync if not connected, but don't error
        }
        drop(state); // Release lock before sync

        self.db
            .sync()
            .await
            .context("Failed to sync with Turso Cloud")?;
        Ok(())
    }

    pub async fn delete_daily_log(&mut self, date: NaiveDate) -> Result<()> {
        let date_str = date.format("%Y-%m-%d").to_string();

        // Start a transaction for atomic deletion
        let tx = self.conn.transaction().await?;

        // Delete the daily_logs record (this will cascade to food_entries and sokay_entries)
        tx.execute(
            "DELETE FROM daily_logs WHERE date = ?1",
            [date_str.as_str()],
        )
        .await
        .context("Failed to delete daily log")?;

        // Commit the transaction
        tx.commit().await.context("Failed to commit transaction")?;

        // Trigger manual sync after deletion
        self.sync().await;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::FoodEntry;
    use tempfile::TempDir;

    fn log(date: &str, notes: &str) -> DailyLog {
        let mut l = DailyLog::new(NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap());
        l.notes = Some(notes.to_string());
        l.add_food_entry(FoodEntry::new(format!("food-{}", notes)));
        l
    }

    #[tokio::test]
    async fn stash_then_import_preserves_local_data_and_keeps_existing_dates() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("mountains.db");
        let db_path_str = db_path.to_str().unwrap().to_string();

        // Session 1: local-only db with two logged days, then stashed (as
        // upgrade_to_remote_replica would before replica creation)
        let mut db = DbManager::new_local_first(dir.path()).await.unwrap();
        db.save_daily_log(&log("2026-07-01", "local-day1"))
            .await
            .unwrap();
        db.save_daily_log(&log("2026-07-02", "local-day2"))
            .await
            .unwrap();
        db.stash_local_db(&db_path_str).await;
        drop(db);
        assert!(!db_path.exists());
        assert_eq!(DbManager::find_stashed_dbs(&db_path_str).len(), 1);

        // Session 2: fresh db standing in for the pulled replica, already
        // holding 07-02 (as if another client wrote it)
        let mut db = DbManager::new_local_first(dir.path()).await.unwrap();
        db.save_daily_log(&log("2026-07-02", "remote-day2"))
            .await
            .unwrap();
        db.import_stashed_dbs(&db_path_str).await.unwrap();

        let logs = db.load_all_daily_logs().await.unwrap();
        assert_eq!(logs.len(), 2);
        // Stashed day absent from the db is imported with entries intact
        let day1 = logs
            .iter()
            .find(|l| l.notes.as_deref() == Some("local-day1"))
            .unwrap();
        assert_eq!(day1.food_entries[0].name, "food-local-day1");
        // Existing date wins over the stash
        assert!(
            logs.iter()
                .any(|l| l.notes.as_deref() == Some("remote-day2"))
        );
        assert!(
            !logs
                .iter()
                .any(|l| l.notes.as_deref() == Some("local-day2-overwritten"))
        );

        // Stash consumed after successful import
        assert!(DbManager::find_stashed_dbs(&db_path_str).is_empty());
    }

    /// Builds a db with the given logs in a scratch dir, stashes it, and moves the
    /// stash into `main_dir` under `stash_name` (bypasses the unix-seconds stash
    /// naming, which would collide for two stashes created within the same second).
    async fn make_stash(main_dir: &std::path::Path, stash_name: &str, logs: &[DailyLog]) {
        let scratch = TempDir::new().unwrap();
        let mut db = DbManager::new_local_first(scratch.path()).await.unwrap();
        for l in logs {
            db.save_daily_log(l).await.unwrap();
        }
        let scratch_db = scratch.path().join("mountains.db");
        db.stash_local_db(scratch_db.to_str().unwrap()).await;
        drop(db);
        let stash = DbManager::find_stashed_dbs(scratch_db.to_str().unwrap())
            .pop()
            .unwrap();
        std::fs::rename(stash, main_dir.join(stash_name)).unwrap();
    }

    #[tokio::test]
    async fn import_across_multiple_stashes_newest_wins_collisions() {
        let dir = TempDir::new().unwrap();
        let db_path_str = dir
            .path()
            .join("mountains.db")
            .to_str()
            .unwrap()
            .to_string();

        make_stash(
            dir.path(),
            "mountains.db.pre-sync.100",
            &[
                log("2026-07-01", "older-day1"),
                log("2026-07-02", "only-in-older"),
            ],
        )
        .await;
        make_stash(
            dir.path(),
            "mountains.db.pre-sync.200",
            &[log("2026-07-01", "newer-day1")],
        )
        .await;

        let mut db = DbManager::new_local_first(dir.path()).await.unwrap();
        db.import_stashed_dbs(&db_path_str).await.unwrap();

        let logs = db.load_all_daily_logs().await.unwrap();
        assert_eq!(logs.len(), 2);
        // Colliding date comes from the newest stash
        assert!(
            logs.iter()
                .any(|l| l.notes.as_deref() == Some("newer-day1"))
        );
        assert!(
            !logs
                .iter()
                .any(|l| l.notes.as_deref() == Some("older-day1"))
        );
        // Date unique to the older stash still imported
        assert!(
            logs.iter()
                .any(|l| l.notes.as_deref() == Some("only-in-older"))
        );
        assert!(DbManager::find_stashed_dbs(&db_path_str).is_empty());
    }

    #[tokio::test]
    async fn failed_import_leaves_stash_for_retry() {
        let dir = TempDir::new().unwrap();
        let db_path_str = dir
            .path()
            .join("mountains.db")
            .to_str()
            .unwrap()
            .to_string();
        let mut db = DbManager::new_local_first(dir.path()).await.unwrap();

        // Valid sqlite file lacking the schema: import errors reading it
        let bad_stash = dir.path().join("mountains.db.pre-sync.100");
        {
            let stash_db = Builder::new_local(bad_stash.to_str().unwrap())
                .build()
                .await
                .unwrap();
            let conn = stash_db.connect().unwrap();
            conn.execute("CREATE TABLE unrelated (x INTEGER)", ())
                .await
                .unwrap();
        }

        assert!(db.import_stashed_dbs(&db_path_str).await.is_err());
        assert!(bad_stash.exists());
        assert_eq!(DbManager::find_stashed_dbs(&db_path_str).len(), 1);
    }

    #[test]
    fn find_stashed_dbs_newest_first_and_skips_sidecar_files() {
        let dir = TempDir::new().unwrap();
        let db_path_str = dir
            .path()
            .join("mountains.db")
            .to_str()
            .unwrap()
            .to_string();

        for name in [
            "mountains.db.pre-sync.100",
            "mountains.db.pre-sync.200",
            "mountains.db.pre-sync.200-wal",
            "mountains.db.pre-sync.200-shm",
            "mountains.db",
        ] {
            std::fs::write(dir.path().join(name), b"").unwrap();
        }

        let stashes = DbManager::find_stashed_dbs(&db_path_str);
        let names: Vec<_> = stashes
            .iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap().to_string())
            .collect();
        assert_eq!(
            names,
            ["mountains.db.pre-sync.200", "mountains.db.pre-sync.100"]
        );
    }
}
