use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaselineMetadata {
    pub source_impl_version: Option<String>,
    pub target_impl_version: Option<String>,
    pub task_version: Option<String>,
    pub fixture_version: Option<String>,
    pub normalizer_version: Option<String>,
    pub approved_by: Option<String>,
    pub approved_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

impl BaselineMetadata {
    pub fn new() -> Self {
        Self {
            source_impl_version: None,
            target_impl_version: None,
            task_version: None,
            fixture_version: None,
            normalizer_version: None,
            approved_by: None,
            approved_at: None,
            notes: None,
        }
    }

    pub fn with_source_impl_version(mut self, version: String) -> Self {
        self.source_impl_version = Some(version);
        self
    }

    pub fn with_target_impl_version(mut self, version: String) -> Self {
        self.target_impl_version = Some(version);
        self
    }

    pub fn with_task_version(mut self, version: String) -> Self {
        self.task_version = Some(version);
        self
    }

    pub fn with_fixture_version(mut self, version: String) -> Self {
        self.fixture_version = Some(version);
        self
    }

    pub fn with_normalizer_version(mut self, version: String) -> Self {
        self.normalizer_version = Some(version);
        self
    }

    pub fn with_approved_by(mut self, approver: String) -> Self {
        self.approved_by = Some(approver);
        self
    }

    pub fn with_approved_at(mut self, timestamp: DateTime<Utc>) -> Self {
        self.approved_at = Some(timestamp);
        self
    }

    pub fn with_notes(mut self, notes: String) -> Self {
        self.notes = Some(notes);
        self
    }

    pub fn validate_version_fields(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if self.source_impl_version.is_none() {
            errors.push("source_impl_version is required".to_string());
        }
        if self.target_impl_version.is_none() {
            errors.push("target_impl_version is required".to_string());
        }
        if self.task_version.is_none() {
            errors.push("task_version is required".to_string());
        }
        if self.fixture_version.is_none() {
            errors.push("fixture_version is required".to_string());
        }
        if self.normalizer_version.is_none() {
            errors.push("normalizer_version is required".to_string());
        }

        errors
    }

    pub fn is_complete(&self) -> bool {
        self.source_impl_version.is_some()
            && self.target_impl_version.is_some()
            && self.task_version.is_some()
            && self.fixture_version.is_some()
            && self.normalizer_version.is_some()
    }
}

impl Default for BaselineMetadata {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_baseline_metadata_instantiation_with_all_fields() {
        let metadata = BaselineMetadata {
            source_impl_version: Some("1.0.0".to_string()),
            target_impl_version: Some("2.0.0".to_string()),
            task_version: Some("1.2.0".to_string()),
            fixture_version: Some("1.1.0".to_string()),
            normalizer_version: Some("1.0.5".to_string()),
            approved_by: Some("reviewer@example.com".to_string()),
            approved_at: Some(Utc::now()),
            notes: Some("Initial baseline".to_string()),
        };

        assert_eq!(metadata.source_impl_version, Some("1.0.0".to_string()));
        assert_eq!(metadata.target_impl_version, Some("2.0.0".to_string()));
        assert_eq!(metadata.task_version, Some("1.2.0".to_string()));
        assert_eq!(metadata.fixture_version, Some("1.1.0".to_string()));
        assert_eq!(metadata.normalizer_version, Some("1.0.5".to_string()));
        assert_eq!(
            metadata.approved_by,
            Some("reviewer@example.com".to_string())
        );
        assert!(metadata.approved_at.is_some());
        assert_eq!(metadata.notes, Some("Initial baseline".to_string()));
    }

    #[test]
    fn test_baseline_metadata_builder_pattern() {
        let now = Utc::now();
        let metadata = BaselineMetadata::new()
            .with_source_impl_version("1.0.0".to_string())
            .with_target_impl_version("2.0.0".to_string())
            .with_task_version("1.2.0".to_string())
            .with_fixture_version("1.1.0".to_string())
            .with_normalizer_version("1.0.5".to_string())
            .with_approved_by("reviewer@example.com".to_string())
            .with_approved_at(now)
            .with_notes("Test baseline".to_string());

        assert_eq!(metadata.source_impl_version, Some("1.0.0".to_string()));
        assert_eq!(metadata.target_impl_version, Some("2.0.0".to_string()));
        assert_eq!(metadata.task_version, Some("1.2.0".to_string()));
        assert_eq!(metadata.fixture_version, Some("1.1.0".to_string()));
        assert_eq!(metadata.normalizer_version, Some("1.0.5".to_string()));
        assert_eq!(
            metadata.approved_by,
            Some("reviewer@example.com".to_string())
        );
        assert_eq!(metadata.approved_at, Some(now));
        assert_eq!(metadata.notes, Some("Test baseline".to_string()));
    }

    #[test]
    fn test_baseline_metadata_yaml_serialization_roundtrip() {
        let now = Utc::now();
        let metadata = BaselineMetadata::new()
            .with_source_impl_version("1.0.0".to_string())
            .with_target_impl_version("2.0.0".to_string())
            .with_task_version("1.2.0".to_string())
            .with_fixture_version("1.1.0".to_string())
            .with_normalizer_version("1.0.5".to_string())
            .with_approved_by("reviewer@example.com".to_string())
            .with_approved_at(now)
            .with_notes("YAML test".to_string());

        let yaml = serde_yaml::to_string(&metadata).expect("serialization should succeed");
        let deserialized: BaselineMetadata =
            serde_yaml::from_str(&yaml).expect("deserialization should succeed");

        assert_eq!(
            metadata.source_impl_version,
            deserialized.source_impl_version
        );
        assert_eq!(
            metadata.target_impl_version,
            deserialized.target_impl_version
        );
        assert_eq!(metadata.task_version, deserialized.task_version);
        assert_eq!(metadata.fixture_version, deserialized.fixture_version);
        assert_eq!(metadata.normalizer_version, deserialized.normalizer_version);
        assert_eq!(metadata.approved_by, deserialized.approved_by);
        assert_eq!(metadata.notes, deserialized.notes);
    }

    #[test]
    fn test_baseline_metadata_yaml_format() {
        let metadata = BaselineMetadata::new()
            .with_source_impl_version("1.0.0".to_string())
            .with_target_impl_version("2.0.0".to_string())
            .with_task_version("1.2.0".to_string())
            .with_fixture_version("1.1.0".to_string())
            .with_normalizer_version("1.0.5".to_string());

        let yaml = serde_yaml::to_string(&metadata).unwrap();

        assert!(yaml.contains("source_impl_version: 1.0.0"));
        assert!(yaml.contains("target_impl_version: 2.0.0"));
        assert!(yaml.contains("task_version: 1.2.0"));
        assert!(yaml.contains("fixture_version: 1.1.0"));
        assert!(yaml.contains("normalizer_version: 1.0.5"));
    }

    #[test]
    fn test_baseline_metadata_version_fields_validated() {
        let incomplete_metadata = BaselineMetadata::new();
        let errors = incomplete_metadata.validate_version_fields();

        assert!(!errors.is_empty());
        assert!(errors.contains(&"source_impl_version is required".to_string()));
        assert!(errors.contains(&"target_impl_version is required".to_string()));
        assert!(errors.contains(&"task_version is required".to_string()));
        assert!(errors.contains(&"fixture_version is required".to_string()));
        assert!(errors.contains(&"normalizer_version is required".to_string()));
    }

    #[test]
    fn test_baseline_metadata_complete_validation() {
        let complete_metadata = BaselineMetadata::new()
            .with_source_impl_version("1.0.0".to_string())
            .with_target_impl_version("2.0.0".to_string())
            .with_task_version("1.2.0".to_string())
            .with_fixture_version("1.1.0".to_string())
            .with_normalizer_version("1.0.5".to_string());

        assert!(complete_metadata.is_complete());
        assert!(complete_metadata.validate_version_fields().is_empty());
    }

    #[test]
    fn test_baseline_metadata_optional_fields_default_to_none() {
        let metadata = BaselineMetadata::new();

        assert!(metadata.approved_by.is_none());
        assert!(metadata.approved_at.is_none());
        assert!(metadata.notes.is_none());
    }

    #[test]
    fn test_baseline_metadata_default() {
        let metadata = BaselineMetadata::default();

        assert!(metadata.source_impl_version.is_none());
        assert!(metadata.target_impl_version.is_none());
        assert!(metadata.task_version.is_none());
        assert!(metadata.fixture_version.is_none());
        assert!(metadata.normalizer_version.is_none());
        assert!(!metadata.is_complete());
    }

    #[test]
    fn test_baseline_metadata_git_commit_sha_versions() {
        let metadata = BaselineMetadata::new()
            .with_source_impl_version("abc123def456".to_string())
            .with_target_impl_version("789ghi012jkl".to_string())
            .with_task_version("task-v1".to_string())
            .with_fixture_version("fix-v2".to_string())
            .with_normalizer_version("norm-v1".to_string());

        assert!(metadata.is_complete());
        let yaml = serde_yaml::to_string(&metadata).unwrap();
        assert!(yaml.contains("abc123def456"));
    }
}
