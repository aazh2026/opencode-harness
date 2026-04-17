use crate::error::{ErrorType, Result};
use crate::loaders::baseline_loader::{BaselineLoader, DefaultBaselineLoader};
use crate::normalizers::normalizer::Normalizer;
use crate::runners::artifact_persister::ArtifactPersister;
use crate::types::baseline::{BaselineMetadata, BaselineRecord};
use crate::types::parity_verdict::ParityVerdict;
use crate::types::runner_output::RunnerOutput;
use chrono::Utc;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

pub trait BaselineRecorder: Send + Sync {
    fn record_baseline(
        &self,
        task_id: &str,
        legacy_result: &Result<RunnerOutput>,
        rust_result: &Result<RunnerOutput>,
        metadata: BaselineMetadata,
    ) -> Result<BaselineRecord>;

    fn update_metadata(
        &self,
        task_id: &str,
        baseline_id: &str,
        approved_by: &str,
        notes: Option<String>,
    ) -> Result<()>;
}

pub struct DefaultBaselineRecorder {
    baseline_loader: Arc<dyn BaselineLoader>,
    #[allow(dead_code)]
    persister: Arc<ArtifactPersister>,
    normalizer: Arc<dyn Normalizer>,
}

impl DefaultBaselineRecorder {
    pub fn new(
        baseline_loader: Arc<dyn BaselineLoader>,
        persister: Arc<ArtifactPersister>,
        normalizer: Arc<dyn Normalizer>,
    ) -> Self {
        Self {
            baseline_loader,
            persister,
            normalizer,
        }
    }

    pub fn with_default_loader(
        persister: Arc<ArtifactPersister>,
        normalizer: Arc<dyn Normalizer>,
    ) -> Self {
        Self::new(
            Arc::new(DefaultBaselineLoader::default()),
            persister,
            normalizer,
        )
    }

    fn compute_verdict(
        legacy_result: &Result<RunnerOutput>,
        rust_result: &Result<RunnerOutput>,
    ) -> ParityVerdict {
        match (legacy_result, rust_result) {
            (Ok(legacy), Ok(rust)) => {
                let normalized_legacy = legacy.stdout.clone();
                let normalized_rust = rust.stdout.clone();

                if normalized_legacy == normalized_rust {
                    ParityVerdict::Pass
                } else {
                    ParityVerdict::Fail {
                        category: crate::types::parity_verdict::DiffCategory::OutputText,
                        details: "Output mismatch between legacy and rust runners".to_string(),
                    }
                }
            }
            (Err(_), Ok(_)) => ParityVerdict::Fail {
                category: crate::types::parity_verdict::DiffCategory::OutputText,
                details: "Legacy runner failed".to_string(),
            },
            (Ok(_), Err(_)) => ParityVerdict::Fail {
                category: crate::types::parity_verdict::DiffCategory::OutputText,
                details: "Rust runner failed".to_string(),
            },
            (Err(e1), Err(e2)) => ParityVerdict::Error {
                runner: "Both".to_string(),
                reason: format!("Legacy: {:?}, Rust: {:?}", e1, e2),
            },
        }
    }

    fn extract_output_or_default(result: &Result<RunnerOutput>) -> RunnerOutput {
        result.as_ref().ok().cloned().unwrap_or_default()
    }
}

impl BaselineRecorder for DefaultBaselineRecorder {
    fn record_baseline(
        &self,
        task_id: &str,
        legacy_result: &Result<RunnerOutput>,
        rust_result: &Result<RunnerOutput>,
        metadata: BaselineMetadata,
    ) -> Result<BaselineRecord> {
        let baseline_id = Uuid::new_v4().to_string();
        let created_at = Utc::now();

        let legacy_output = Self::extract_output_or_default(legacy_result);
        let rust_output = Self::extract_output_or_default(rust_result);

        let normalized_legacy = self.normalizer.normalize(&legacy_output.stdout);
        let normalized_rust = self.normalizer.normalize(&rust_output.stdout);

        let verdict = Self::compute_verdict(legacy_result, rust_result);

        let raw_legacy_path = if legacy_output.stdout_path != PathBuf::from("/tmp/stdout.txt") {
            Some(legacy_output.stdout_path.clone())
        } else {
            None
        };

        let raw_rust_path = if rust_output.stdout_path != PathBuf::from("/tmp/stdout.txt") {
            Some(rust_output.stdout_path.clone())
        } else {
            None
        };

        let record = BaselineRecord::new(
            baseline_id.clone(),
            task_id.to_string(),
            metadata,
            legacy_output,
            rust_output,
            normalized_legacy,
            normalized_rust,
            verdict,
            created_at,
            raw_legacy_path,
            raw_rust_path,
        );

        self.baseline_loader.save(&record)?;

        Ok(record)
    }

    fn update_metadata(
        &self,
        task_id: &str,
        baseline_id: &str,
        approved_by: &str,
        notes: Option<String>,
    ) -> Result<()> {
        let record = self
            .baseline_loader
            .load(task_id, baseline_id)?
            .ok_or_else(|| {
                ErrorType::Config(format!("Baseline not found: {}/{}", task_id, baseline_id))
            })?;

        let mut updated_metadata = record.metadata.clone();
        updated_metadata.approved_by = Some(approved_by.to_string());
        updated_metadata.approved_at = Some(Utc::now());
        if let Some(n) = notes {
            updated_metadata.notes = Some(n);
        }

        let updated_record = BaselineRecord::new(
            record.id.clone(),
            record.task_id.clone(),
            updated_metadata,
            record.legacy_output.clone(),
            record.rust_output.clone(),
            record.normalized_legacy.clone(),
            record.normalized_rust.clone(),
            record.verdict.clone(),
            record.created_at,
            record.raw_legacy_path.clone(),
            record.raw_rust_path.clone(),
        );

        self.baseline_loader.save(&updated_record)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loaders::baseline_loader::DefaultBaselineLoader;
    use crate::normalizers::normalizer::NoOpNormalizer;
    use crate::runners::artifact_persister::ArtifactPersister;
    use crate::types::baseline::BaselineMetadata;
    use crate::types::runner_output::RunnerOutput;
    use tempfile::TempDir;

    fn create_test_metadata() -> BaselineMetadata {
        BaselineMetadata::new()
            .with_source_impl_version("1.0.0".to_string())
            .with_target_impl_version("2.0.0".to_string())
            .with_task_version("1.2.0".to_string())
            .with_fixture_version("1.1.0".to_string())
            .with_normalizer_version("1.0.5".to_string())
    }

    fn create_test_recorder() -> (DefaultBaselineRecorder, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let loader = DefaultBaselineLoader::new(temp_dir.path().to_path_buf());
        let persister = ArtifactPersister::new("test-run", temp_dir.path().to_path_buf());
        let normalizer = NoOpNormalizer;

        (
            DefaultBaselineRecorder::new(
                Arc::new(loader),
                Arc::new(persister),
                Arc::new(normalizer),
            ),
            temp_dir,
        )
    }

    #[test]
    fn test_record_baseline_creates_new_baseline() {
        let (recorder, _temp_dir) = create_test_recorder();

        let metadata = create_test_metadata();
        let legacy_result = Ok(RunnerOutput::default().with_stdout("legacy output".to_string()));
        let rust_result = Ok(RunnerOutput::default().with_stdout("rust output".to_string()));

        let record = recorder
            .record_baseline("TASK-001", &legacy_result, &rust_result, metadata)
            .unwrap();

        assert!(!record.id.is_empty());
        assert_eq!(record.task_id, "TASK-001");
        assert!(record.metadata.is_complete());
        assert_eq!(record.legacy_output.stdout, "legacy output");
        assert_eq!(record.rust_output.stdout, "rust output");
        assert!(record.verdict.is_different());
    }

    #[test]
    fn test_record_baseline_with_identical_outputs() {
        let (recorder, _temp_dir) = create_test_recorder();

        let metadata = create_test_metadata();
        let legacy_result = Ok(RunnerOutput::default().with_stdout("same output".to_string()));
        let rust_result = Ok(RunnerOutput::default().with_stdout("same output".to_string()));

        let record = recorder
            .record_baseline("TASK-002", &legacy_result, &rust_result, metadata)
            .unwrap();

        assert!(record.verdict.is_pass());
        assert!(record.verdict.is_identical());
    }

    #[test]
    fn test_record_baseline_handles_errored_results() {
        let (recorder, _temp_dir) = create_test_recorder();

        let metadata = create_test_metadata();
        let legacy_result: Result<RunnerOutput> =
            Err(ErrorType::Runner("Legacy failed".to_string()));
        let rust_result = Ok(RunnerOutput::default().with_stdout("rust output".to_string()));

        let record = recorder
            .record_baseline("TASK-003", &legacy_result, &rust_result, metadata)
            .unwrap();

        assert!(record.verdict.is_different());
    }

    #[test]
    fn test_record_baseline_saves_to_loader() {
        let (recorder, _temp_dir) = create_test_recorder();
        let loader = recorder.baseline_loader.clone();

        let metadata = create_test_metadata();
        let legacy_result = Ok(RunnerOutput::default().with_stdout("legacy".to_string()));
        let rust_result = Ok(RunnerOutput::default().with_stdout("rust".to_string()));

        let record = recorder
            .record_baseline("TASK-004", &legacy_result, &rust_result, metadata)
            .unwrap();

        let loaded = loader.load("TASK-004", &record.id).unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().id, record.id);
    }

    #[test]
    fn test_update_metadata_modifies_existing_baseline() {
        let (recorder, _temp_dir) = create_test_recorder();
        let loader = recorder.baseline_loader.clone();

        let metadata = create_test_metadata();
        let legacy_result = Ok(RunnerOutput::default().with_stdout("output".to_string()));
        let rust_result = Ok(RunnerOutput::default().with_stdout("output".to_string()));

        let record = recorder
            .record_baseline("TASK-005", &legacy_result, &rust_result, metadata)
            .unwrap();

        assert!(record.metadata.approved_by.is_none());

        recorder
            .update_metadata(
                "TASK-005",
                &record.id,
                "reviewer@example.com",
                Some("Approved".to_string()),
            )
            .unwrap();

        let updated = loader.load("TASK-005", &record.id).unwrap().unwrap();
        assert_eq!(
            updated.metadata.approved_by,
            Some("reviewer@example.com".to_string())
        );
        assert!(updated.metadata.approved_at.is_some());
        assert_eq!(updated.metadata.notes, Some("Approved".to_string()));
    }

    #[test]
    fn test_update_metadata_with_only_approver() {
        let (recorder, _temp_dir) = create_test_recorder();
        let loader = recorder.baseline_loader.clone();

        let metadata = create_test_metadata();
        let legacy_result = Ok(RunnerOutput::default().with_stdout("output".to_string()));
        let rust_result = Ok(RunnerOutput::default().with_stdout("output".to_string()));

        let record = recorder
            .record_baseline("TASK-006", &legacy_result, &rust_result, metadata)
            .unwrap();

        recorder
            .update_metadata("TASK-006", &record.id, "admin@example.com", None)
            .unwrap();

        let updated = loader.load("TASK-006", &record.id).unwrap().unwrap();
        assert_eq!(
            updated.metadata.approved_by,
            Some("admin@example.com".to_string())
        );
        assert!(updated.metadata.notes.is_none());
    }

    #[test]
    fn test_update_metadata_returns_error_for_nonexistent_baseline() {
        let (recorder, _temp_dir) = create_test_recorder();

        let result =
            recorder.update_metadata("TASK-999", "nonexistent-id", "reviewer@example.com", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_recorder_trait_object() {
        let (recorder, _temp_dir) = create_test_recorder();
        let boxed: Box<dyn BaselineRecorder> = Box::new(recorder);

        let metadata = create_test_metadata();
        let legacy_result = Ok(RunnerOutput::default().with_stdout("legacy".to_string()));
        let rust_result = Ok(RunnerOutput::default().with_stdout("rust".to_string()));

        let record = boxed
            .record_baseline("TASK-007", &legacy_result, &rust_result, metadata)
            .unwrap();

        assert_eq!(record.task_id, "TASK-007");
    }

    #[test]
    fn test_recorder_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        let (_recorder, _temp_dir) = create_test_recorder();
        assert_send_sync::<DefaultBaselineRecorder>();
    }

    #[test]
    fn baseline_record_smoke_tests() {
        use crate::loaders::baseline_loader::{BaselineLoader, DefaultBaselineLoader};
        use crate::normalizers::normalizer::NoOpNormalizer;
        use crate::runners::artifact_persister::ArtifactPersister;
        use crate::types::baseline::{BaselineMetadata, BaselineRecord};
        use crate::types::runner_output::RunnerOutput;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let loader = DefaultBaselineLoader::new(temp_dir.path().to_path_buf());
        let persister = ArtifactPersister::new("test-run", temp_dir.path().to_path_buf());
        let normalizer = NoOpNormalizer;

        let recorder = DefaultBaselineRecorder::new(
            Arc::new(loader.clone()),
            Arc::new(persister),
            Arc::new(normalizer),
        );

        let metadata = BaselineMetadata::new()
            .with_source_impl_version("1.0.0".to_string())
            .with_target_impl_version("2.0.0".to_string())
            .with_task_version("1.2.0".to_string())
            .with_fixture_version("1.1.0".to_string())
            .with_normalizer_version("1.0.5".to_string())
            .with_approved_by("reviewer@example.com".to_string())
            .with_notes("Integration test baseline".to_string());

        assert!(
            metadata.is_complete(),
            "Baseline metadata should have all version fields"
        );
        assert_eq!(
            metadata.source_impl_version.as_deref(),
            Some("1.0.0"),
            "source_impl_version should be set"
        );
        assert_eq!(
            metadata.target_impl_version.as_deref(),
            Some("2.0.0"),
            "target_impl_version should be set"
        );
        assert_eq!(
            metadata.task_version.as_deref(),
            Some("1.2.0"),
            "task_version should be set"
        );
        assert_eq!(
            metadata.fixture_version.as_deref(),
            Some("1.1.0"),
            "fixture_version should be set"
        );
        assert_eq!(
            metadata.normalizer_version.as_deref(),
            Some("1.0.5"),
            "normalizer_version should be set"
        );

        let legacy_output = RunnerOutput::default()
            .with_exit_code(Some(0))
            .with_stdout("legacy integration output".to_string())
            .with_stderr("legacy error".to_string());

        let rust_output = RunnerOutput::default()
            .with_exit_code(Some(0))
            .with_stdout("rust integration output".to_string())
            .with_stderr("rust error".to_string());

        let legacy_result: Result<RunnerOutput> = Ok(legacy_output.clone());
        let rust_result: Result<RunnerOutput> = Ok(rust_output.clone());

        let record = recorder
            .record_baseline(
                "TASK-SMOKE-001",
                &legacy_result,
                &rust_result,
                metadata.clone(),
            )
            .expect("record_baseline should succeed");

        let yaml = serde_yaml::to_string(&record).expect("serialization should succeed");
        assert!(
            yaml.contains("TASK-SMOKE-001"),
            "YAML should contain task_id"
        );
        assert!(
            yaml.contains("source_impl_version: 1.0.0"),
            "YAML should contain source_impl_version"
        );
        assert!(
            yaml.contains("target_impl_version: 2.0.0"),
            "YAML should contain target_impl_version"
        );
        assert!(
            yaml.contains("legacy integration output"),
            "YAML should contain legacy output"
        );
        assert!(
            yaml.contains("rust integration output"),
            "YAML should contain rust output"
        );

        let deserialized: BaselineRecord =
            serde_yaml::from_str(&yaml).expect("deserialization should succeed");
        assert_eq!(record.id, deserialized.id, "Roundtrip should preserve id");
        assert_eq!(
            record.task_id, deserialized.task_id,
            "Roundtrip should preserve task_id"
        );
        assert_eq!(
            record.metadata.source_impl_version, deserialized.metadata.source_impl_version,
            "Roundtrip should preserve source_impl_version"
        );
        assert_eq!(
            record.metadata.target_impl_version, deserialized.metadata.target_impl_version,
            "Roundtrip should preserve target_impl_version"
        );
        assert_eq!(
            record.metadata.task_version, deserialized.metadata.task_version,
            "Roundtrip should preserve task_version"
        );
        assert_eq!(
            record.metadata.fixture_version, deserialized.metadata.fixture_version,
            "Roundtrip should preserve fixture_version"
        );
        assert_eq!(
            record.metadata.normalizer_version, deserialized.metadata.normalizer_version,
            "Roundtrip should preserve normalizer_version"
        );
        assert_eq!(
            record.legacy_output.stdout, deserialized.legacy_output.stdout,
            "Roundtrip should preserve legacy stdout"
        );
        assert_eq!(
            record.rust_output.stdout, deserialized.rust_output.stdout,
            "Roundtrip should preserve rust stdout"
        );
        assert_eq!(
            record.normalized_legacy, deserialized.normalized_legacy,
            "Roundtrip should preserve normalized_legacy"
        );
        assert_eq!(
            record.normalized_rust, deserialized.normalized_rust,
            "Roundtrip should preserve normalized_rust"
        );

        let loaded = loader
            .load("TASK-SMOKE-001", &record.id)
            .expect("loader.load should succeed")
            .expect("loader should return Some record");

        assert_eq!(
            loaded.id, record.id,
            "Loader should return correct baseline id"
        );
        assert_eq!(
            loaded.task_id, "TASK-SMOKE-001",
            "Loader should return correct task_id"
        );
        assert_eq!(
            loaded.metadata.source_impl_version,
            Some("1.0.0".to_string()),
            "Loader should preserve source_impl_version"
        );
        assert_eq!(
            loaded.metadata.target_impl_version,
            Some("2.0.0".to_string()),
            "Loader should preserve target_impl_version"
        );
        assert_eq!(
            loaded.metadata.task_version,
            Some("1.2.0".to_string()),
            "Loader should preserve task_version"
        );
        assert_eq!(
            loaded.metadata.fixture_version,
            Some("1.1.0".to_string()),
            "Loader should preserve fixture_version"
        );
        assert_eq!(
            loaded.metadata.normalizer_version,
            Some("1.0.5".to_string()),
            "Loader should preserve normalizer_version"
        );
        assert_eq!(
            loaded.legacy_output.stdout, "legacy integration output",
            "Loader should preserve legacy output"
        );
        assert_eq!(
            loaded.rust_output.stdout, "rust integration output",
            "Loader should preserve rust output"
        );

        loader
            .delete("TASK-SMOKE-001", &record.id)
            .expect("delete should succeed");
        let after_delete = loader
            .load("TASK-SMOKE-001", &record.id)
            .expect("load after delete should succeed");
        assert!(
            after_delete.is_none(),
            "Record should not exist after delete"
        );
    }
}
