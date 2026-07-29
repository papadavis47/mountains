use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// `Default` is derived: bool defaults to false and String to "", which is
// exactly the "no cloud sync configured" state we want. Prefer deriving over a
// hand-written impl whenever the field defaults already produce the right value.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
    pub sync: SyncConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SyncConfig {
    pub enabled: bool,
    pub db_url: String,
    pub auth_token: String,
}

impl SyncConfig {
    pub fn is_configured(&self) -> bool {
        self.enabled && !self.db_url.is_empty() && !self.auth_token.is_empty()
    }
}

pub fn data_dir() -> Result<PathBuf> {
    if let Ok(dir) = std::env::var("MOUNTAINS_DATA_DIR") {
        return Ok(PathBuf::from(dir));
    }
    let home = dirs::home_dir().context("Could not find home directory")?;
    Ok(home.join(".mountains"))
}

impl AppConfig {
    pub fn load_from_path(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let contents = std::fs::read_to_string(path).context("Failed to read config file")?;
        if contents.trim().is_empty() {
            return Ok(Self::default());
        }
        toml::from_str(&contents).context("Failed to parse config TOML")
    }

    pub fn save_to_path(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create config directory")?;
        }
        let contents = toml::to_string_pretty(self).context("Failed to serialize config")?;
        std::fs::write(path, contents).context("Failed to write config file")?;
        Ok(())
    }

    pub fn load() -> Result<Self> {
        let path = data_dir()?.join("config.toml");
        Self::load_from_path(&path)
    }

    pub fn save(&self) -> Result<()> {
        let path = data_dir()?.join("config.toml");
        self.save_to_path(&path)
    }
}

/// One-time migration from .env to config.toml.
/// Parses TURSO_DATABASE_URL and TURSO_AUTH_TOKEN from .env,
/// writes config.toml, renames .env to .env.bak.
pub fn migrate_from_env(data_dir: &Path) -> Result<bool> {
    let env_path = data_dir.join(".env");
    if !env_path.exists() {
        return Ok(false);
    }

    let contents = std::fs::read_to_string(&env_path).context("Failed to read .env")?;

    let mut db_url = String::new();
    let mut auth_token = String::new();

    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim().trim_matches('"').trim_matches('\'');
            match key {
                "TURSO_DATABASE_URL" => db_url = value.to_string(),
                "TURSO_AUTH_TOKEN" => auth_token = value.to_string(),
                _ => {}
            }
        }
    }

    let has_credentials = !db_url.is_empty() && !auth_token.is_empty();

    let config = AppConfig {
        sync: SyncConfig {
            enabled: has_credentials,
            db_url,
            auth_token,
        },
    };

    let config_path = data_dir.join("config.toml");
    config.save_to_path(&config_path)?;

    let bak_path = data_dir.join(".env.bak");
    std::fs::rename(&env_path, &bak_path).context("Failed to rename .env to .env.bak")?;

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn default_config_has_sync_disabled() {
        let config = AppConfig::default();
        assert!(!config.sync.enabled);
        assert!(config.sync.db_url.is_empty());
        assert!(config.sync.auth_token.is_empty());
    }

    #[test]
    fn is_configured_requires_all_three() {
        let mut sync = SyncConfig::default();
        assert!(!sync.is_configured());

        sync.enabled = true;
        assert!(!sync.is_configured());

        sync.db_url = "libsql://test.turso.io".into();
        assert!(!sync.is_configured());

        sync.auth_token = "token123".into();
        assert!(sync.is_configured());
    }

    #[test]
    fn is_configured_false_when_disabled() {
        let sync = SyncConfig {
            enabled: false,
            db_url: "libsql://test.turso.io".into(),
            auth_token: "token123".into(),
        };
        assert!(!sync.is_configured());
    }

    #[test]
    fn roundtrip_save_load() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");

        let config = AppConfig {
            sync: SyncConfig {
                enabled: true,
                db_url: "libsql://mydb.turso.io".into(),
                auth_token: "secret".into(),
            },
        };

        config.save_to_path(&path).unwrap();
        let loaded = AppConfig::load_from_path(&path).unwrap();

        assert!(loaded.sync.enabled);
        assert_eq!(loaded.sync.db_url, "libsql://mydb.turso.io");
        assert_eq!(loaded.sync.auth_token, "secret");
    }

    #[test]
    fn load_missing_file_returns_default() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nonexistent.toml");
        let config = AppConfig::load_from_path(&path).unwrap();
        assert!(!config.sync.enabled);
    }

    #[test]
    fn load_empty_file_returns_default() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "").unwrap();
        let config = AppConfig::load_from_path(&path).unwrap();
        assert!(!config.sync.enabled);
    }

    #[test]
    fn load_invalid_toml_returns_error() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "not valid {{{{ toml").unwrap();
        assert!(AppConfig::load_from_path(&path).is_err());
    }

    #[test]
    fn partial_config_fills_defaults() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(
            &path,
            "[sync]\nenabled = true\ndb_url = \"\"\nauth_token = \"\"\n",
        )
        .unwrap();
        let config = AppConfig::load_from_path(&path).unwrap();
        assert!(config.sync.enabled);
        assert!(config.sync.db_url.is_empty());
    }

    #[test]
    fn save_creates_parent_dirs() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nested").join("deep").join("config.toml");
        let config = AppConfig::default();
        config.save_to_path(&path).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn migrate_from_env_works() {
        let dir = TempDir::new().unwrap();
        let env_path = dir.path().join(".env");
        std::fs::write(
            &env_path,
            "TURSO_DATABASE_URL=libsql://test.turso.io\nTURSO_AUTH_TOKEN=mytoken\n",
        )
        .unwrap();

        let migrated = migrate_from_env(dir.path()).unwrap();
        assert!(migrated);

        // .env renamed to .env.bak
        assert!(!env_path.exists());
        assert!(dir.path().join(".env.bak").exists());

        // config.toml created with correct values
        let config = AppConfig::load_from_path(&dir.path().join("config.toml")).unwrap();
        assert!(config.sync.enabled);
        assert_eq!(config.sync.db_url, "libsql://test.turso.io");
        assert_eq!(config.sync.auth_token, "mytoken");
    }

    #[test]
    fn migrate_no_env_returns_false() {
        let dir = TempDir::new().unwrap();
        let migrated = migrate_from_env(dir.path()).unwrap();
        assert!(!migrated);
    }
}
