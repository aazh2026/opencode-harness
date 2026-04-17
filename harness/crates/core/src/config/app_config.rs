use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parse error: {0}")]
    YamlParse(#[from] serde_yaml::Error),

    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("Config file not found at {path}")]
    NotFound { path: PathBuf },

    #[error("Invalid config format")]
    InvalidFormat,

    #[error("Home directory not found")]
    HomeNotFound,
}

pub type Result<T> = std::result::Result<T, AppConfigError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateThresholds {
    #[serde(default = "default_pr_pass_rate")]
    pub pr_pass_rate: f64,
    #[serde(default = "default_nightly_pass_rate")]
    pub nightly_pass_rate: f64,
    #[serde(default = "default_release_pass_rate")]
    pub release_pass_rate: f64,
    #[serde(default = "default_pr_max_warnings")]
    pub pr_max_warnings: u32,
    #[serde(default = "default_nightly_max_warnings")]
    pub nightly_max_warnings: u32,
    #[serde(default = "default_release_max_warnings")]
    pub release_max_warnings: u32,
    #[serde(default = "default_error_rate_threshold")]
    pub error_rate_threshold: f64,
}

fn default_pr_pass_rate() -> f64 {
    0.90
}
fn default_nightly_pass_rate() -> f64 {
    0.80
}
fn default_release_pass_rate() -> f64 {
    1.0
}
fn default_pr_max_warnings() -> u32 {
    5
}
fn default_nightly_max_warnings() -> u32 {
    10
}
fn default_release_max_warnings() -> u32 {
    0
}
fn default_error_rate_threshold() -> f64 {
    0.1
}

impl Default for GateThresholds {
    fn default() -> Self {
        Self {
            pr_pass_rate: default_pr_pass_rate(),
            nightly_pass_rate: default_nightly_pass_rate(),
            release_pass_rate: default_release_pass_rate(),
            pr_max_warnings: default_pr_max_warnings(),
            nightly_max_warnings: default_nightly_max_warnings(),
            release_max_warnings: default_release_max_warnings(),
            error_rate_threshold: default_error_rate_threshold(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    #[serde(default = "default_default_timeout_seconds")]
    pub default_timeout_seconds: u64,
    #[serde(default = "default_suite_timeout_seconds")]
    pub suite_timeout_seconds: u64,
    #[serde(default = "default_max_timeout_seconds")]
    pub max_timeout_seconds: u64,
}

fn default_default_timeout_seconds() -> u64 {
    300
}
fn default_suite_timeout_seconds() -> u64 {
    60
}
fn default_max_timeout_seconds() -> u64 {
    3600
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            default_timeout_seconds: default_default_timeout_seconds(),
            suite_timeout_seconds: default_suite_timeout_seconds(),
            max_timeout_seconds: default_max_timeout_seconds(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub gate_thresholds: GateThresholds,
    #[serde(default)]
    pub timeout: TimeoutConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            gate_thresholds: GateThresholds {
                pr_pass_rate: default_pr_pass_rate(),
                nightly_pass_rate: default_nightly_pass_rate(),
                release_pass_rate: default_release_pass_rate(),
                pr_max_warnings: default_pr_max_warnings(),
                nightly_max_warnings: default_nightly_max_warnings(),
                release_max_warnings: default_release_max_warnings(),
                error_rate_threshold: default_error_rate_threshold(),
            },
            timeout: TimeoutConfig {
                default_timeout_seconds: default_default_timeout_seconds(),
                suite_timeout_seconds: default_suite_timeout_seconds(),
                max_timeout_seconds: default_max_timeout_seconds(),
            },
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<AppConfig> {
        Self::load_from_default_location()
    }

    pub fn load_from_default_location() -> Result<AppConfig> {
        let config_path = Self::default_config_path()?;
        Self::load_from_file(&config_path)
    }

    pub fn load_from_file(path: &PathBuf) -> Result<AppConfig> {
        if !path.exists() {
            return Err(AppConfigError::NotFound { path: path.clone() });
        }

        let content = std::fs::read_to_string(path)?;

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        match ext.to_lowercase().as_str() {
            "yaml" | "yml" => Self::load_from_yaml_str(&content),
            "json" => Self::load_from_json_str(&content),
            _ => Self::load_from_yaml_str(&content).or_else(|_| Self::load_from_json_str(&content)),
        }
    }

    pub fn load_from_yaml_str(yaml: &str) -> Result<AppConfig> {
        let config: AppConfig = serde_yaml::from_str(yaml)?;
        Ok(config)
    }

    pub fn load_from_json_str(json: &str) -> Result<AppConfig> {
        let config: AppConfig = serde_json::from_str(json)?;
        Ok(config)
    }

    fn default_config_path() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or(AppConfigError::HomeNotFound)?;
        Ok(home.join(".opencode").join("config.yaml"))
    }

    #[cfg(test)]
    pub fn with_pr_pass_rate(mut self, rate: f64) -> Self {
        self.gate_thresholds.pr_pass_rate = rate;
        self
    }

    #[cfg(test)]
    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout.default_timeout_seconds = timeout;
        self
    }
}

pub struct AppConfigLoader;

impl AppConfigLoader {
    pub fn load() -> Result<AppConfig> {
        AppConfig::load()
    }

    pub fn load_from(path: PathBuf) -> Result<AppConfig> {
        AppConfig::load_from_file(&path)
    }

    pub fn load_with_defaults() -> AppConfig {
        AppConfig::default()
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
    fn test_default_config_has_all_values() {
        let config = AppConfig::default();
        assert!((config.gate_thresholds.pr_pass_rate - 0.90).abs() < 0.01);
        assert!((config.gate_thresholds.nightly_pass_rate - 0.80).abs() < 0.01);
        assert!((config.gate_thresholds.release_pass_rate - 1.0).abs() < 0.01);
        assert_eq!(config.gate_thresholds.pr_max_warnings, 5);
        assert_eq!(config.gate_thresholds.nightly_max_warnings, 10);
        assert_eq!(config.gate_thresholds.release_max_warnings, 0);
        assert!((config.gate_thresholds.error_rate_threshold - 0.1).abs() < 0.01);
        assert_eq!(config.timeout.default_timeout_seconds, 300);
        assert_eq!(config.timeout.suite_timeout_seconds, 60);
        assert_eq!(config.timeout.max_timeout_seconds, 3600);
    }

    #[test]
    fn test_load_yaml_config() {
        let temp_dir = create_test_config(
            r#"
gate_thresholds:
  pr_pass_rate: 0.85
  nightly_pass_rate: 0.75
  release_pass_rate: 0.95
  pr_max_warnings: 3
  nightly_max_warnings: 8
  release_max_warnings: 1
  error_rate_threshold: 0.15
timeout:
  default_timeout_seconds: 500
  suite_timeout_seconds: 120
  max_timeout_seconds: 1800
"#,
            "config.yaml",
        );

        let config = AppConfig::load_from_file(&temp_dir.path().join("config.yaml")).unwrap();
        assert!((config.gate_thresholds.pr_pass_rate - 0.85).abs() < 0.01);
        assert!((config.gate_thresholds.nightly_pass_rate - 0.75).abs() < 0.01);
        assert!((config.gate_thresholds.release_pass_rate - 0.95).abs() < 0.01);
        assert_eq!(config.gate_thresholds.pr_max_warnings, 3);
        assert_eq!(config.gate_thresholds.nightly_max_warnings, 8);
        assert_eq!(config.gate_thresholds.release_max_warnings, 1);
        assert!((config.gate_thresholds.error_rate_threshold - 0.15).abs() < 0.01);
        assert_eq!(config.timeout.default_timeout_seconds, 500);
        assert_eq!(config.timeout.suite_timeout_seconds, 120);
        assert_eq!(config.timeout.max_timeout_seconds, 1800);
    }

    #[test]
    fn test_load_json_config() {
        let temp_dir = create_test_config(
            r#"{
                "gate_thresholds": {
                    "pr_pass_rate": 0.88,
                    "nightly_pass_rate": 0.70,
                    "release_pass_rate": 0.98,
                    "pr_max_warnings": 4,
                    "nightly_max_warnings": 12,
                    "release_max_warnings": 2,
                    "error_rate_threshold": 0.20
                },
                "timeout": {
                    "default_timeout_seconds": 400,
                    "suite_timeout_seconds": 90,
                    "max_timeout_seconds": 2400
                }
            }"#,
            "config.json",
        );

        let config = AppConfig::load_from_file(&temp_dir.path().join("config.json")).unwrap();
        assert!((config.gate_thresholds.pr_pass_rate - 0.88).abs() < 0.01);
        assert_eq!(config.timeout.default_timeout_seconds, 400);
    }

    #[test]
    fn test_partial_config_uses_defaults() {
        let temp_dir = create_test_config(
            r#"
gate_thresholds:
  pr_pass_rate: 0.92
"#,
            "partial.yaml",
        );

        let config = AppConfig::load_from_file(&temp_dir.path().join("partial.yaml")).unwrap();
        assert!((config.gate_thresholds.pr_pass_rate - 0.92).abs() < 0.01);
        assert!((config.gate_thresholds.nightly_pass_rate - 0.80).abs() < 0.01);
        assert_eq!(config.gate_thresholds.pr_max_warnings, 5);
    }

    #[test]
    fn test_not_found_error() {
        let temp_dir = TempDir::new().unwrap();
        let result = AppConfig::load_from_file(&temp_dir.path().join("nonexistent.yaml"));
        assert!(matches!(result, Err(AppConfigError::NotFound { .. })));
    }

    #[test]
    fn test_malformed_yaml_error() {
        let temp_dir = create_test_config("invalid: yaml: content: [", "malformed.yaml");
        let result = AppConfig::load_from_file(&temp_dir.path().join("malformed.yaml"));
        assert!(matches!(result, Err(AppConfigError::YamlParse(_))));
    }

    #[test]
    fn test_malformed_json_error() {
        let temp_dir = create_test_config("{ invalid json }", "malformed.json");
        let result = AppConfig::load_from_file(&temp_dir.path().join("malformed.json"));
        assert!(matches!(result, Err(AppConfigError::JsonParse(_))));
    }
}
