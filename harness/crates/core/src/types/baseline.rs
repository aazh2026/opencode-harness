use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::types::parity_verdict::ParityVerdict;
use crate::types::runner_output::RunnerOutput;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineRecord {
    pub id: String,
    pub task_id: String,
    pub metadata: BaselineMetadata,
    pub legacy_output: RunnerOutput,
    pub rust_output: RunnerOutput,
    pub normalized_legacy: String,
    pub normalized_rust: String,
    pub verdict: ParityVerdict,
    pub created_at: DateTime<Utc>,
    pub raw_legacy_path: Option<PathBuf>,
    pub raw_rust_path: Option<PathBuf>,
}

impl BaselineRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        task_id: String,
        metadata: BaselineMetadata,
        legacy_output: RunnerOutput,
        rust_output: RunnerOutput,
        normalized_legacy: String,
        normalized_rust: String,
        verdict: ParityVerdict,
        created_at: DateTime<Utc>,
        raw_legacy_path: Option<PathBuf>,
        raw_rust_path: Option<PathBuf>,
    ) -> Self {
        Self {
            id,
            task_id,
            metadata,
            legacy_output,
            rust_output,
            normalized_legacy,
            normalized_rust,
            verdict,
            created_at,
            raw_legacy_path,
            raw_rust_path,
        }
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.id = id;
        self
    }

    pub fn with_task_id(mut self, task_id: String) -> Self {
        self.task_id = task_id;
        self
    }

    pub fn with_metadata(mut self, metadata: BaselineMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn with_legacy_output(mut self, legacy_output: RunnerOutput) -> Self {
        self.legacy_output = legacy_output;
        self
    }

    pub fn with_rust_output(mut self, rust_output: RunnerOutput) -> Self {
        self.rust_output = rust_output;
        self
    }

    pub fn with_normalized_legacy(mut self, normalized_legacy: String) -> Self {
        self.normalized_legacy = normalized_legacy;
        self
    }

    pub fn with_normalized_rust(mut self, normalized_rust: String) -> Self {
        self.normalized_rust = normalized_rust;
        self
    }

    pub fn with_verdict(mut self, verdict: ParityVerdict) -> Self {
        self.verdict = verdict;
        self
    }

    pub fn with_created_at(mut self, created_at: DateTime<Utc>) -> Self {
        self.created_at = created_at;
        self
    }

    pub fn with_raw_legacy_path(mut self, raw_legacy_path: Option<PathBuf>) -> Self {
        self.raw_legacy_path = raw_legacy_path;
        self
    }

    pub fn with_raw_rust_path(mut self, raw_rust_path: Option<PathBuf>) -> Self {
        self.raw_rust_path = raw_rust_path;
        self
    }

    pub fn is_pass(&self) -> bool {
        self.verdict.is_pass()
    }

    pub fn is_regression(&self) -> bool {
        self.verdict.is_different()
    }
}

impl Default for BaselineRecord {
    fn default() -> Self {
        Self {
            id: String::new(),
            task_id: String::new(),
            metadata: BaselineMetadata::default(),
            legacy_output: RunnerOutput::default(),
            rust_output: RunnerOutput::default(),
            normalized_legacy: String::new(),
            normalized_rust: String::new(),
            verdict: ParityVerdict::Pass,
            created_at: Utc::now(),
            raw_legacy_path: None,
            raw_rust_path: None,
        }
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

    #[test]
    fn test_baseline_record_instantiation_with_all_fields() {
        use crate::types::parity_verdict::ParityVerdict;
        use crate::types::runner_output::RunnerOutput;
        use std::path::PathBuf;

        let now = Utc::now();
        let metadata = BaselineMetadata::new()
            .with_source_impl_version("1.0.0".to_string())
            .with_target_impl_version("2.0.0".to_string())
            .with_task_version("1.2.0".to_string())
            .with_fixture_version("1.1.0".to_string())
            .with_normalizer_version("1.0.5".to_string());

        let legacy_output = RunnerOutput::default()
            .with_exit_code(Some(0))
            .with_stdout("legacy output".to_string())
            .with_stderr("".to_string());

        let rust_output = RunnerOutput::default()
            .with_exit_code(Some(0))
            .with_stdout("rust output".to_string())
            .with_stderr("".to_string());

        let record = BaselineRecord::new(
            "baseline-001".to_string(),
            "TASK-001".to_string(),
            metadata.clone(),
            legacy_output.clone(),
            rust_output.clone(),
            "normalized legacy".to_string(),
            "normalized rust".to_string(),
            ParityVerdict::Pass,
            now,
            Some(PathBuf::from("/artifacts/legacy/baseline-001")),
            Some(PathBuf::from("/artifacts/rust/baseline-001")),
        );

        assert_eq!(record.id, "baseline-001");
        assert_eq!(record.task_id, "TASK-001");
        assert_eq!(
            record.metadata.source_impl_version,
            Some("1.0.0".to_string())
        );
        assert_eq!(record.legacy_output.stdout, "legacy output");
        assert_eq!(record.rust_output.stdout, "rust output");
        assert_eq!(record.normalized_legacy, "normalized legacy");
        assert_eq!(record.normalized_rust, "normalized rust");
        assert!(record.verdict.is_pass());
        assert_eq!(record.created_at, now);
        assert!(record.raw_legacy_path.is_some());
        assert!(record.raw_rust_path.is_some());
    }

    #[test]
    fn test_baseline_record_builder_pattern() {
        use crate::types::parity_verdict::ParityVerdict;
        use crate::types::runner_output::RunnerOutput;
        use std::path::PathBuf;

        let now = Utc::now();
        let metadata = BaselineMetadata::default()
            .with_source_impl_version("2.0.0".to_string())
            .with_target_impl_version("3.0.0".to_string())
            .with_task_version("2.0.0".to_string())
            .with_fixture_version("2.0.0".to_string())
            .with_normalizer_version("2.0.0".to_string());

        let record = BaselineRecord::default()
            .with_id("baseline-002".to_string())
            .with_task_id("TASK-002".to_string())
            .with_metadata(metadata)
            .with_legacy_output(RunnerOutput::default().with_stdout("legacy".to_string()))
            .with_rust_output(RunnerOutput::default().with_stdout("rust".to_string()))
            .with_normalized_legacy("norm legacy".to_string())
            .with_normalized_rust("norm rust".to_string())
            .with_verdict(ParityVerdict::Pass)
            .with_created_at(now)
            .with_raw_legacy_path(Some(PathBuf::from("/path/legacy")))
            .with_raw_rust_path(Some(PathBuf::from("/path/rust")));

        assert_eq!(record.id, "baseline-002");
        assert_eq!(record.task_id, "TASK-002");
        assert_eq!(record.legacy_output.stdout, "legacy");
        assert_eq!(record.rust_output.stdout, "rust");
        assert_eq!(record.normalized_legacy, "norm legacy");
        assert_eq!(record.normalized_rust, "norm rust");
        assert!(record.verdict.is_pass());
    }

    #[test]
    fn test_baseline_record_yaml_serialization_roundtrip() {
        use crate::types::parity_verdict::ParityVerdict;
        use crate::types::runner_output::RunnerOutput;
        use std::path::PathBuf;

        let now = Utc::now();
        let metadata = BaselineMetadata::new()
            .with_source_impl_version("1.0.0".to_string())
            .with_target_impl_version("2.0.0".to_string())
            .with_task_version("1.2.0".to_string())
            .with_fixture_version("1.1.0".to_string())
            .with_normalizer_version("1.0.5".to_string());

        let record = BaselineRecord::new(
            "baseline-003".to_string(),
            "TASK-003".to_string(),
            metadata,
            RunnerOutput::default()
                .with_exit_code(Some(0))
                .with_stdout("legacy output".to_string()),
            RunnerOutput::default()
                .with_exit_code(Some(0))
                .with_stdout("rust output".to_string()),
            "norm legacy".to_string(),
            "norm rust".to_string(),
            ParityVerdict::Pass,
            now,
            Some(PathBuf::from("/artifacts/legacy")),
            Some(PathBuf::from("/artifacts/rust")),
        );

        let yaml = serde_yaml::to_string(&record).expect("serialization should succeed");
        let deserialized: BaselineRecord =
            serde_yaml::from_str(&yaml).expect("deserialization should succeed");

        assert_eq!(record.id, deserialized.id);
        assert_eq!(record.task_id, deserialized.task_id);
        assert_eq!(
            record.metadata.source_impl_version,
            deserialized.metadata.source_impl_version
        );
        assert_eq!(
            record.legacy_output.stdout,
            deserialized.legacy_output.stdout
        );
        assert_eq!(record.rust_output.stdout, deserialized.rust_output.stdout);
        assert_eq!(record.normalized_legacy, deserialized.normalized_legacy);
        assert_eq!(record.normalized_rust, deserialized.normalized_rust);
        assert!(deserialized.verdict.is_pass());
    }

    #[test]
    fn test_baseline_record_verdict_pass() {
        use crate::types::parity_verdict::ParityVerdict;

        let record = BaselineRecord::default().with_verdict(ParityVerdict::Pass);

        assert!(record.verdict.is_pass());
        assert!(record.verdict.is_identical());
        assert!(!record.verdict.is_different());
        assert!(record.is_pass());
        assert!(!record.is_regression());
    }

    #[test]
    fn test_baseline_record_verdict_pass_with_allowed_variance() {
        use crate::types::parity_verdict::{ParityVerdict, VarianceType};

        let record =
            BaselineRecord::default().with_verdict(ParityVerdict::PassWithAllowedVariance {
                variance_type: VarianceType::Timing,
                details: "Timing diff within tolerance".to_string(),
            });

        assert!(record.verdict.is_pass());
        assert!(record.verdict.is_equivalent());
        assert!(!record.verdict.is_identical());
        assert!(record.is_pass());
        assert!(!record.is_regression());
    }

    #[test]
    fn test_baseline_record_verdict_fail() {
        use crate::types::parity_verdict::{DiffCategory, ParityVerdict};

        let record = BaselineRecord::default().with_verdict(ParityVerdict::Fail {
            category: DiffCategory::OutputText,
            details: "Output mismatch".to_string(),
        });

        assert!(!record.verdict.is_pass());
        assert!(record.verdict.is_different());
        assert!(!record.is_pass());
        assert!(record.is_regression());
    }

    #[test]
    fn test_baseline_record_verdict_warn() {
        use crate::types::parity_verdict::{DiffCategory, ParityVerdict};

        let record = BaselineRecord::default().with_verdict(ParityVerdict::Warn {
            category: DiffCategory::Timing,
            message: "Timing slightly off".to_string(),
        });

        assert!(!record.verdict.is_pass());
        assert!(!record.verdict.is_different());
        assert!(!record.is_pass());
        assert!(!record.is_regression());
    }

    #[test]
    fn test_baseline_record_default() {
        let record = BaselineRecord::default();

        assert!(record.id.is_empty());
        assert!(record.task_id.is_empty());
        assert!(!record.metadata.is_complete());
        assert!(record.legacy_output.stdout.is_empty());
        assert!(record.rust_output.stdout.is_empty());
        assert!(record.normalized_legacy.is_empty());
        assert!(record.normalized_rust.is_empty());
        assert!(record.verdict.is_pass());
        assert!(record.raw_legacy_path.is_none());
        assert!(record.raw_rust_path.is_none());
    }
}
