use crate::models::DailyLog;
use anyhow::{Context, Result};
use chrono::NaiveDate;
use std::fs;
use std::path::PathBuf;

#[derive(Clone)]
pub struct FileManager {
    mountains_dir: PathBuf,
}

impl FileManager {
    pub fn new() -> Result<Self> {
        let home_dir = dirs::home_dir().context("Could not find home directory")?;
        let mountains_dir = home_dir.join(".mountains");

        if !mountains_dir.exists() {
            fs::create_dir_all(&mountains_dir).context("Failed to create .mountains directory")?;
        }

        Ok(Self { mountains_dir })
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
}
