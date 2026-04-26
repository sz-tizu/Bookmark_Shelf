use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub checker: CheckerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub bookmark_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckerConfig {
    pub concurrency: usize,
    pub timeout_secs: u64,
    pub follow_redirects: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            general: GeneralConfig {
                bookmark_dir: default_bookmark_dir(),
            },
            checker: CheckerConfig {
                concurrency: 20,
                timeout_secs: 10,
                follow_redirects: true,
            },
        }
    }
}

/// macOS: ~/Library/Application Support/bookmark-shelf/bookmarks
/// Windows: %APPDATA%\bookmark-shelf\bookmarks
/// Linux:   ~/.local/share/bookmark-shelf/bookmarks
pub fn default_bookmark_dir() -> String {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("bookmark-shelf")
        .join("bookmarks")
        .to_string_lossy()
        .to_string()
}

impl Config {
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("bookmark-shelf")
            .join("config.toml")
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if !path.exists() {
            return Ok(Config::default());
        }
        let content = std::fs::read_to_string(&path)?;
        Ok(toml::from_str(&content)?)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    #[cfg(test)]
    pub fn save_to(&self, path: &std::path::Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    #[cfg(test)]
    pub fn load_from(path: &std::path::Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_concurrency() {
        assert_eq!(Config::default().checker.concurrency, 20);
    }

    #[test]
    fn test_default_timeout() {
        assert_eq!(Config::default().checker.timeout_secs, 10);
    }

    #[test]
    fn test_default_follow_redirects() {
        assert!(Config::default().checker.follow_redirects);
    }

    #[test]
    fn test_default_bookmark_dir_is_in_app_data() {
        let dir = Config::default().general.bookmark_dir;
        // Must be inside the platform app-data directory, not just ~/bookmarks
        assert!(
            dir.contains("bookmark-shelf") && dir.ends_with("bookmarks"),
            "expected '.../bookmark-shelf/bookmarks', got '{dir}'"
        );
    }

    #[test]
    fn test_toml_roundtrip_preserves_all_fields() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");

        let mut original = Config::default();
        original.checker.concurrency = 42;
        original.checker.timeout_secs = 30;
        original.checker.follow_redirects = false;
        original.general.bookmark_dir = "/tmp/my-bookmarks".to_string();

        original.save_to(&path).unwrap();
        let loaded = Config::load_from(&path).unwrap();

        assert_eq!(loaded.checker.concurrency, 42);
        assert_eq!(loaded.checker.timeout_secs, 30);
        assert!(!loaded.checker.follow_redirects);
        assert_eq!(loaded.general.bookmark_dir, "/tmp/my-bookmarks");
    }

    #[test]
    fn test_toml_file_is_human_readable() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        Config::default().save_to(&path).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("[general]"), "missing [general] section");
        assert!(content.contains("[checker]"), "missing [checker] section");
        assert!(content.contains("concurrency"), "missing concurrency key");
    }

    #[test]
    fn test_load_from_missing_file_returns_default() {
        let result = Config::load();
        // load() falls back to default when file doesn't exist — should not error
        assert!(result.is_ok());
    }
}
