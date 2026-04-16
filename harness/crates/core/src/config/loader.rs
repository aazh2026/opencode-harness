use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML parse error: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Config file not found")]
    NotFound,
    #[error("Invalid config format")]
    InvalidFormat,
}

pub type Result<T> = std::result::Result<T, ConfigError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessConfig {
    pub timeout: u64,
    pub retries: u32,
    pub workspace_root: PathBuf,
    pub artifacts_dir: PathBuf,
    pub sessions_dir: PathBuf,
}

impl Default for HarnessConfig {
    fn default() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self {
            timeout: 300,
            retries: 3,
            workspace_root: cwd.clone(),
            artifacts_dir: cwd.join("artifacts"),
            sessions_dir: cwd.join("sessions"),
        }
    }
}

impl HarnessConfig {
    pub fn load() -> Result<HarnessConfig> {
        Self::load_from(PathBuf::from("."))
    }

    pub fn load_from(start: PathBuf) -> Result<HarnessConfig> {
        let dir = start.canonicalize().unwrap_or(start);

        let config_file = Self::find_config_file(&dir).ok_or(ConfigError::NotFound)?;

        Self::load_from_file(&config_file)
    }

    fn find_config_file(dir: &Path) -> Option<PathBuf> {
        let mut current = Some(dir.to_path_buf());

        while let Some(cwd) = current {
            let toml_path = cwd.join("harness.toml");
            let json_path = cwd.join("harness.json");

            if toml_path.exists() {
                return Some(toml_path);
            }
            if json_path.exists() {
                return Some(json_path);
            }

            current = cwd.parent().map(|p| p.to_path_buf());
        }
        None
    }

    fn load_from_file(path: &Path) -> Result<HarnessConfig> {
        let content = std::fs::read_to_string(path)?;

        let ext = path.extension().and_then(|e| e.to_str());
        match ext {
            Some("toml") => {
                let raw: toml::Value = toml::from_str(&content)?;
                Self::from_toml_value(raw)
            }
            Some("json") => {
                let config: HarnessConfig = serde_json::from_str(&content)?;
                Ok(config)
            }
            _ => Err(ConfigError::InvalidFormat),
        }
    }

    fn from_toml_value(value: toml::Value) -> Result<HarnessConfig> {
        let table = value.as_table().ok_or(ConfigError::InvalidFormat)?;

        let timeout = table
            .get("timeout")
            .and_then(|v| v.as_integer())
            .map(|v| v as u64)
            .unwrap_or(300);

        let retries = table
            .get("retries")
            .and_then(|v| v.as_integer())
            .map(|v| v as u32)
            .unwrap_or(3);

        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        let workspace_root = table
            .get("workspace_root")
            .and_then(|v| v.as_str())
            .map(PathBuf::from)
            .unwrap_or_else(|| cwd.clone());

        let artifacts_dir = table
            .get("artifacts_dir")
            .and_then(|v| v.as_str())
            .map(PathBuf::from)
            .unwrap_or_else(|| cwd.join("artifacts"));

        let sessions_dir = table
            .get("sessions_dir")
            .and_then(|v| v.as_str())
            .map(PathBuf::from)
            .unwrap_or_else(|| cwd.join("sessions"));

        Ok(HarnessConfig {
            timeout,
            retries,
            workspace_root,
            artifacts_dir,
            sessions_dir,
        })
    }

    #[cfg(test)]
    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = timeout;
        self
    }

    #[cfg(test)]
    pub fn with_retries(mut self, retries: u32) -> Self {
        self.retries = retries;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_config(content: &str, name: &str) -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(name);
        fs::write(&config_path, content).unwrap();
        temp_dir
    }

    #[test]
    fn test_config_has_all_required_fields() {
        let config = HarnessConfig::default();
        assert!(config.timeout == 300);
        assert!(config.retries == 3);
        assert!(config.workspace_root != PathBuf::new());
        assert!(config.artifacts_dir != PathBuf::new());
        assert!(config.sessions_dir != PathBuf::new());
    }

    #[test]
    fn test_default_returns_valid_config() {
        let config = HarnessConfig::default();
        assert_eq!(config.timeout, 300);
        assert_eq!(config.retries, 3);
        assert!(config.workspace_root.exists() || config.workspace_root.to_str() == Some("."));
        assert!(
            config.artifacts_dir.ends_with("artifacts")
                || config.artifacts_dir.to_str() == Some("artifacts")
        );
        assert!(
            config.sessions_dir.ends_with("sessions")
                || config.sessions_dir.to_str() == Some("sessions")
        );
    }

    #[test]
    fn test_load_toml_config() {
        let temp_dir = create_test_config(
            r#"
            timeout = 600
            retries = 5
            workspace_root = "/tmp/workspace"
            artifacts_dir = "/tmp/artifacts"
            sessions_dir = "/tmp/sessions"
            "#,
            "harness.toml",
        );

        let config = HarnessConfig::load_from(temp_dir.path().to_path_buf()).unwrap();
        assert_eq!(config.timeout, 600);
        assert_eq!(config.retries, 5);
        assert_eq!(config.workspace_root, PathBuf::from("/tmp/workspace"));
        assert_eq!(config.artifacts_dir, PathBuf::from("/tmp/artifacts"));
        assert_eq!(config.sessions_dir, PathBuf::from("/tmp/sessions"));
    }

    #[test]
    fn test_load_json_config() {
        let temp_dir = create_test_config(
            r#"{
                "timeout": 450,
                "retries": 2,
                "workspace_root": "/custom/workspace",
                "artifacts_dir": "/custom/artifacts",
                "sessions_dir": "/custom/sessions"
            }"#,
            "harness.json",
        );

        let config = HarnessConfig::load_from(temp_dir.path().to_path_buf()).unwrap();
        assert_eq!(config.timeout, 450);
        assert_eq!(config.retries, 2);
        assert_eq!(config.workspace_root, PathBuf::from("/custom/workspace"));
    }

    #[test]
    fn test_load_searches_current_directory() {
        let temp_dir = create_test_config(
            r#"
            timeout = 100
            retries = 1
            "#,
            "harness.toml",
        );

        let config = HarnessConfig::load_from(temp_dir.path().to_path_buf()).unwrap();
        assert_eq!(config.timeout, 100);
        assert_eq!(config.retries, 1);
    }

    #[test]
    fn test_load_walks_up_ancestor_directories() {
        let temp_dir = create_test_config(
            r#"
            timeout = 200
            retries = 4
            "#,
            "harness.toml",
        );

        let nested_dir = temp_dir.path().join("nested").join("deep");
        fs::create_dir_all(&nested_dir).unwrap();

        let config = HarnessConfig::load_from(nested_dir).unwrap();
        assert_eq!(config.timeout, 200);
        assert_eq!(config.retries, 4);
    }

    #[test]
    fn test_load_returns_error_if_no_config_found() {
        let temp_dir = TempDir::new().unwrap();
        let result = HarnessConfig::load_from(temp_dir.path().to_path_buf());
        assert!(matches!(result, Err(ConfigError::NotFound)));
    }

    #[test]
    fn test_default_values_when_fields_missing() {
        let temp_dir = create_test_config(
            r#"
            timeout = 500
            "#,
            "harness.toml",
        );

        let config = HarnessConfig::load_from(temp_dir.path().to_path_buf()).unwrap();
        assert_eq!(config.timeout, 500);
        assert_eq!(config.retries, 3);
    }
}
