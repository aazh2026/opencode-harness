//! Suite configuration loader supporting YAML and JSON formats.
//!
//! Implements FR-308: SuiteConfigLoader for loading suite definitions
//! from external configuration files.

use crate::reporting::gate::GateLevel;
use crate::reporting::suite::{ArtifactPolicy, SuiteDefinition, SuiteName};
use crate::types::TaskCategory;
use serde::Deserialize;
use std::path::Path;
use thiserror::Error;

// =============================================================================
// Error Types
// =============================================================================

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to parse YAML: {0}")]
    YamlParseError(#[from] serde_yaml::Error),

    #[error("Failed to parse JSON: {0}")]
    JsonParseError(#[from] serde_json::Error),

    #[error(
        "Invalid suite name '{0}': must be one of pr-smoke, nightly-full, release-qualification"
    )]
    InvalidSuiteName(String),

    #[error("Invalid task category '{0}': must be one of core, schema, integration, regression, smoke, performance, security")]
    InvalidTaskCategory(String),

    #[error("Invalid gate level '{0}': must be one of pr, nightly, release")]
    InvalidGateLevel(String),

    #[error("Invalid artifact policy '{0}': must be one of always, on_failure, never")]
    InvalidArtifactPolicy(String),
}

// =============================================================================
// Config Structures
// =============================================================================

/// Suite configuration as loaded from YAML/JSON file.
#[derive(Debug, Clone, Deserialize)]
pub struct SuiteConfigFile {
    /// List of suite definitions.
    pub suites: Vec<SuiteDefinitionConfig>,
}

/// Individual suite configuration entry.
#[derive(Debug, Clone, Deserialize)]
pub struct SuiteDefinitionConfig {
    /// Suite identifier (pr-smoke, nightly-full, release-qualification).
    pub name: String,

    /// Human-readable description.
    #[serde(default)]
    pub description: Option<String>,

    /// Task categories to include in this suite.
    pub task_categories: Vec<String>,

    /// Whether whitelists are allowed for this suite.
    #[serde(default)]
    pub allowed_whitelists: bool,

    /// Whether skipped tasks are allowed for this suite.
    #[serde(default)]
    pub allow_skipped: bool,

    /// Whether manual check tasks are allowed for this suite.
    #[serde(default)]
    pub allow_manual_check: bool,

    /// Artifact retention policy.
    #[serde(default)]
    pub artifact_policy: Option<String>,
}

// =============================================================================
// SuiteConfigLoader
// =============================================================================

/// Loads suite configurations from YAML or JSON files.
pub struct SuiteConfigLoader;

impl SuiteConfigLoader {
    /// Load suite configurations from a file (auto-detects YAML vs JSON by extension).
    ///
    /// # Errors
    /// Returns `ConfigError` if the file cannot be read or parsed.
    ///
    /// # FR-308-10
    pub fn load_from_file(path: &Path) -> Result<Vec<SuiteDefinition>, ConfigError> {
        let content = std::fs::read_to_string(path)?;

        // Detect format by extension
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        match ext.to_lowercase().as_str() {
            "yaml" | "yml" => Self::load_from_str(&content),
            "json" => Self::load_from_json_str(&content),
            _ => {
                // Try YAML first, fall back to JSON
                match Self::load_from_str(&content) {
                    Ok(suites) => Ok(suites),
                    Err(_) => Self::load_from_json_str(&content),
                }
            }
        }
    }

    /// Parse suite configurations from a YAML string.
    ///
    /// # Errors
    /// Returns `ConfigError` if the YAML cannot be parsed.
    ///
    /// # FR-308-11
    pub fn load_from_str(yaml: &str) -> Result<Vec<SuiteDefinition>, ConfigError> {
        let config_file: SuiteConfigFile = serde_yaml::from_str(yaml)?;
        Self::convert_suites(config_file.suites)
    }

    /// Parse suite configurations from a JSON string.
    ///
    /// # Errors
    /// Returns `ConfigError` if the JSON cannot be parsed.
    pub fn load_from_json_str(json: &str) -> Result<Vec<SuiteDefinition>, ConfigError> {
        let config_file: SuiteConfigFile = serde_json::from_str(json)?;
        Self::convert_suites(config_file.suites)
    }

    /// Convert loaded configs to SuiteDefinition objects.
    fn convert_suites(
        configs: Vec<SuiteDefinitionConfig>,
    ) -> Result<Vec<SuiteDefinition>, ConfigError> {
        configs
            .into_iter()
            .map(|cfg| {
                let name = Self::parse_suite_name(&cfg.name)?;
                let categories = Self::parse_task_categories(&cfg.task_categories)?;
                let artifact_policy = Self::parse_artifact_policy(
                    cfg.artifact_policy.as_deref().unwrap_or("on_failure"),
                )?;

                // Determine gate level from suite name
                let gate_level = match name {
                    SuiteName::PrSmoke => GateLevel::PR,
                    SuiteName::NightlyFull => GateLevel::Nightly,
                    SuiteName::ReleaseQualification => GateLevel::Release,
                };

                Ok(SuiteDefinition {
                    name: name.clone(),
                    description: cfg
                        .description
                        .unwrap_or_else(|| format!("{:?} suite", name)),
                    included_task_categories: categories,
                    allowed_whitelists: cfg.allowed_whitelists,
                    allow_skipped: cfg.allow_skipped,
                    allow_manual_check: cfg.allow_manual_check,
                    artifact_retention_policy: artifact_policy,
                    gate_level,
                })
            })
            .collect()
    }

    fn parse_suite_name(name: &str) -> Result<SuiteName, ConfigError> {
        match name {
            "pr-smoke" => Ok(SuiteName::PrSmoke),
            "nightly-full" => Ok(SuiteName::NightlyFull),
            "release-qualification" => Ok(SuiteName::ReleaseQualification),
            _ => Err(ConfigError::InvalidSuiteName(name.to_string())),
        }
    }

    fn parse_task_categories(categories: &[String]) -> Result<Vec<TaskCategory>, ConfigError> {
        categories
            .iter()
            .map(|c| match c.to_lowercase().as_str() {
                "core" => Ok(TaskCategory::Core),
                "schema" => Ok(TaskCategory::Schema),
                "integration" => Ok(TaskCategory::Integration),
                "regression" => Ok(TaskCategory::Regression),
                "smoke" => Ok(TaskCategory::Smoke),
                "performance" => Ok(TaskCategory::Performance),
                "security" => Ok(TaskCategory::Security),
                _ => Err(ConfigError::InvalidTaskCategory(c.clone())),
            })
            .collect()
    }

    fn parse_artifact_policy(policy: &str) -> Result<ArtifactPolicy, ConfigError> {
        match policy.to_lowercase().as_str() {
            "always" => Ok(ArtifactPolicy::Always),
            "on_failure" | "onfailure" => Ok(ArtifactPolicy::OnFailure),
            "never" => Ok(ArtifactPolicy::Never),
            _ => Err(ConfigError::InvalidArtifactPolicy(policy.to_string())),
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_pr_smoke_from_yaml() {
        let yaml = r#"
suites:
  - name: pr-smoke
    description: PR smoke suite
    task_categories:
      - smoke
      - regression
    allowed_whitelists: true
    allow_skipped: false
    allow_manual_check: true
    artifact_policy: on_failure
"#;
        let suites = SuiteConfigLoader::load_from_str(yaml).unwrap();
        assert_eq!(suites.len(), 1);

        let suite = &suites[0];
        assert!(matches!(suite.name, SuiteName::PrSmoke));
        assert_eq!(suite.gate_level, GateLevel::PR);
        assert!(suite.allowed_whitelists);
        assert!(!suite.allow_skipped);
        assert!(suite.allow_manual_check);
        assert!(matches!(
            suite.artifact_retention_policy,
            ArtifactPolicy::OnFailure
        ));
    }

    #[test]
    fn test_load_multiple_suites_from_json() {
        let json = r#"{
  "suites": [
    {
      "name": "pr-smoke",
      "description": "PR smoke",
      "task_categories": ["smoke"],
      "allowed_whitelists": true,
      "allow_skipped": false,
      "allow_manual_check": true
    },
    {
      "name": "nightly-full",
      "description": "Nightly full",
      "task_categories": ["smoke", "regression"],
      "allowed_whitelists": true,
      "allow_skipped": true,
      "allow_manual_check": true,
      "artifact_policy": "always"
    }
  ]
}"#;
        let suites = SuiteConfigLoader::load_from_json_str(json).unwrap();
        assert_eq!(suites.len(), 2);
        assert!(matches!(suites[0].name, SuiteName::PrSmoke));
        assert!(matches!(suites[1].name, SuiteName::NightlyFull));
    }

    #[test]
    fn test_invalid_suite_name() {
        let yaml = r#"
suites:
  - name: invalid-suite
    task_categories: []
"#;
        let result = SuiteConfigLoader::load_from_str(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_task_category() {
        let yaml = r#"
suites:
  - name: pr-smoke
    task_categories:
      - invalid-category
"#;
        let result = SuiteConfigLoader::load_from_str(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_default_artifact_policy() {
        let yaml = r#"
suites:
  - name: pr-smoke
    task_categories: []
"#;
        let suites = SuiteConfigLoader::load_from_str(yaml).unwrap();
        // Default should be OnFailure when not specified
        assert!(matches!(
            suites[0].artifact_retention_policy,
            ArtifactPolicy::OnFailure
        ));
    }
}
