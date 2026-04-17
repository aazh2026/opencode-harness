use crate::error::{ErrorType, Result};
use crate::types::baseline::BaselineRecord;
use std::fs;
use std::path::{Path, PathBuf};

pub trait BaselineLoader: Send + Sync {
    fn load(&self, task_id: &str, baseline_id: &str) -> Result<Option<BaselineRecord>>;
    fn load_latest(&self, task_id: &str) -> Result<Option<BaselineRecord>>;
    fn load_all_for_task(&self, task_id: &str) -> Result<Vec<BaselineRecord>>;
    fn save(&self, record: &BaselineRecord) -> Result<()>;
    fn list_baselines(&self, task_id: &str) -> Result<Vec<String>>;
    fn delete(&self, task_id: &str, baseline_id: &str) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct DefaultBaselineLoader {
    base_path: PathBuf,
}

impl DefaultBaselineLoader {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    fn baseline_path(&self, task_id: &str, baseline_id: &str) -> PathBuf {
        self.base_path
            .join(task_id)
            .join(format!("{}.yaml", baseline_id))
    }

    fn task_dir(&self, task_id: &str) -> PathBuf {
        self.base_path.join(task_id)
    }

    fn load_yaml_file(&self, path: &Path) -> Result<BaselineRecord> {
        let content = fs::read_to_string(path).map_err(ErrorType::Io)?;
        serde_yaml::from_str(&content)
            .map_err(|e| ErrorType::Config(format!("Failed to parse baseline YAML: {}", e)))
    }

    fn save_yaml_file(&self, path: &Path, record: &BaselineRecord) -> Result<()> {
        let content = serde_yaml::to_string(record).map_err(|e| {
            ErrorType::Config(format!("Failed to serialize baseline to YAML: {}", e))
        })?;
        fs::write(path, content).map_err(ErrorType::Io)?;
        Ok(())
    }
}

impl Default for DefaultBaselineLoader {
    fn default() -> Self {
        Self::new(PathBuf::from("harness/golden/baselines"))
    }
}

impl BaselineLoader for DefaultBaselineLoader {
    fn load(&self, task_id: &str, baseline_id: &str) -> Result<Option<BaselineRecord>> {
        let path = self.baseline_path(task_id, baseline_id);
        if !path.is_file() {
            return Ok(None);
        }
        match self.load_yaml_file(&path) {
            Ok(record) => Ok(Some(record)),
            Err(e) => Err(e),
        }
    }

    fn load_latest(&self, task_id: &str) -> Result<Option<BaselineRecord>> {
        let task_dir = self.task_dir(task_id);
        if !task_dir.is_dir() {
            return Ok(None);
        }

        let mut baselines = Vec::new();
        for entry in fs::read_dir(&task_dir).map_err(ErrorType::Io)? {
            let entry = entry.map_err(ErrorType::Io)?;
            let file_path = entry.path();
            if file_path.is_file() {
                if let Some(ext) = file_path.extension() {
                    if ext == "yaml" || ext == "yml" {
                        match self.load_yaml_file(&file_path) {
                            Ok(record) => baselines.push(record),
                            Err(e) => return Err(e),
                        }
                    }
                }
            }
        }

        baselines.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(baselines.into_iter().next())
    }

    fn load_all_for_task(&self, task_id: &str) -> Result<Vec<BaselineRecord>> {
        let task_dir = self.task_dir(task_id);
        if !task_dir.is_dir() {
            return Ok(Vec::new());
        }

        let mut baselines = Vec::new();
        for entry in fs::read_dir(&task_dir).map_err(ErrorType::Io)? {
            let entry = entry.map_err(ErrorType::Io)?;
            let file_path = entry.path();
            if file_path.is_file() {
                if let Some(ext) = file_path.extension() {
                    if ext == "yaml" || ext == "yml" {
                        match self.load_yaml_file(&file_path) {
                            Ok(record) => baselines.push(record),
                            Err(e) => return Err(e),
                        }
                    }
                }
            }
        }

        baselines.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(baselines)
    }

    fn save(&self, record: &BaselineRecord) -> Result<()> {
        let path = self.baseline_path(&record.task_id, &record.id);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(ErrorType::Io)?;
        }
        self.save_yaml_file(&path, record)
    }

    fn list_baselines(&self, task_id: &str) -> Result<Vec<String>> {
        let task_dir = self.task_dir(task_id);
        if !task_dir.is_dir() {
            return Ok(Vec::new());
        }

        let mut baseline_ids = Vec::new();
        for entry in fs::read_dir(&task_dir).map_err(ErrorType::Io)? {
            let entry = entry.map_err(ErrorType::Io)?;
            let file_path = entry.path();
            if file_path.is_file() {
                if let Some(ext) = file_path.extension() {
                    if ext == "yaml" || ext == "yml" {
                        if let Some(stem) = file_path.file_stem() {
                            baseline_ids.push(stem.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }

        Ok(baseline_ids)
    }

    fn delete(&self, task_id: &str, baseline_id: &str) -> Result<()> {
        let path = self.baseline_path(task_id, baseline_id);
        if !path.is_file() {
            return Err(ErrorType::Config(format!(
                "Baseline not found: {}/{}",
                task_id, baseline_id
            )));
        }
        fs::remove_file(path).map_err(ErrorType::Io)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::baseline::BaselineMetadata;
    use crate::types::parity_verdict::ParityVerdict;
    use crate::types::runner_output::RunnerOutput;
    use chrono::Utc;
    use tempfile::TempDir;

    fn create_test_baseline_record(
        task_id: &str,
        baseline_id: &str,
        created_at: chrono::DateTime<Utc>,
    ) -> BaselineRecord {
        let metadata = BaselineMetadata::new()
            .with_source_impl_version("1.0.0".to_string())
            .with_target_impl_version("2.0.0".to_string())
            .with_task_version("1.2.0".to_string())
            .with_fixture_version("1.1.0".to_string())
            .with_normalizer_version("1.0.5".to_string());

        BaselineRecord::new(
            baseline_id.to_string(),
            task_id.to_string(),
            metadata,
            RunnerOutput::default()
                .with_exit_code(Some(0))
                .with_stdout("legacy output".to_string()),
            RunnerOutput::default()
                .with_exit_code(Some(0))
                .with_stdout("rust output".to_string()),
            "normalized legacy".to_string(),
            "normalized rust".to_string(),
            ParityVerdict::Pass,
            created_at,
            Some(PathBuf::from("/artifacts/legacy")),
            Some(PathBuf::from("/artifacts/rust")),
        )
    }

    #[test]
    fn test_default_loader_creation() {
        let loader = DefaultBaselineLoader::default();
        assert_eq!(loader.base_path, PathBuf::from("harness/golden/baselines"));
    }

    #[test]
    fn test_loader_with_custom_base_path() {
        let loader = DefaultBaselineLoader::new(PathBuf::from("/custom/path"));
        assert_eq!(loader.base_path, PathBuf::from("/custom/path"));
    }

    #[test]
    fn test_loader_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<DefaultBaselineLoader>();
    }

    #[test]
    fn test_load_retrieves_baseline_by_id() {
        let temp_dir = TempDir::new().unwrap();
        let loader = DefaultBaselineLoader::new(temp_dir.path().to_path_buf());

        let record = create_test_baseline_record("TASK-001", "baseline-001", Utc::now());
        loader.save(&record).unwrap();

        let loaded = loader.load("TASK-001", "baseline-001").unwrap();
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.id, "baseline-001");
        assert_eq!(loaded.task_id, "TASK-001");
        assert_eq!(
            loaded.metadata.source_impl_version,
            Some("1.0.0".to_string())
        );
        assert_eq!(loaded.legacy_output.stdout, "legacy output");
        assert_eq!(loaded.rust_output.stdout, "rust output");
    }

    #[test]
    fn test_load_returns_none_for_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let loader = DefaultBaselineLoader::new(temp_dir.path().to_path_buf());

        let loaded = loader.load("TASK-001", "nonexistent").unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn test_load_latest_returns_most_recent() {
        let temp_dir = TempDir::new().unwrap();
        let loader = DefaultBaselineLoader::new(temp_dir.path().to_path_buf());

        let older = create_test_baseline_record("TASK-001", "baseline-old", Utc::now());
        let newer = create_test_baseline_record("TASK-001", "baseline-new", Utc::now());

        std::thread::sleep(std::time::Duration::from_millis(10));
        let newer_created = Utc::now();
        let newer_record = create_test_baseline_record("TASK-001", "baseline-new", newer_created);

        loader.save(&older).unwrap();
        loader.save(&newer_record).unwrap();

        let latest = loader.load_latest("TASK-001").unwrap();
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().id, "baseline-new");
    }

    #[test]
    fn test_load_latest_returns_none_for_empty_task() {
        let temp_dir = TempDir::new().unwrap();
        let loader = DefaultBaselineLoader::new(temp_dir.path().to_path_buf());

        let latest = loader.load_latest("TASK-999").unwrap();
        assert!(latest.is_none());
    }

    #[test]
    fn test_save_persists_to_correct_yaml_path() {
        let temp_dir = TempDir::new().unwrap();
        let loader = DefaultBaselineLoader::new(temp_dir.path().to_path_buf());

        let record = create_test_baseline_record("TASK-001", "baseline-001", Utc::now());
        loader.save(&record).unwrap();

        let expected_path = temp_dir.path().join("TASK-001").join("baseline-001.yaml");
        assert!(expected_path.is_file());

        let content = fs::read_to_string(&expected_path).unwrap();
        assert!(content.contains("baseline-001"));
        assert!(content.contains("TASK-001"));
        assert!(content.contains("source_impl_version: 1.0.0"));
    }

    #[test]
    fn test_list_baselines_returns_all_for_task() {
        let temp_dir = TempDir::new().unwrap();
        let loader = DefaultBaselineLoader::new(temp_dir.path().to_path_buf());

        let record1 = create_test_baseline_record("TASK-001", "baseline-a", Utc::now());
        let record2 = create_test_baseline_record("TASK-001", "baseline-b", Utc::now());
        let record3 = create_test_baseline_record("TASK-001", "baseline-c", Utc::now());

        loader.save(&record1).unwrap();
        loader.save(&record2).unwrap();
        loader.save(&record3).unwrap();

        let baselines = loader.list_baselines("TASK-001").unwrap();
        assert_eq!(baselines.len(), 3);
        assert!(baselines.contains(&"baseline-a".to_string()));
        assert!(baselines.contains(&"baseline-b".to_string()));
        assert!(baselines.contains(&"baseline-c".to_string()));
    }

    #[test]
    fn test_list_baselines_returns_empty_for_nonexistent_task() {
        let temp_dir = TempDir::new().unwrap();
        let loader = DefaultBaselineLoader::new(temp_dir.path().to_path_buf());

        let baselines = loader.list_baselines("TASK-999").unwrap();
        assert!(baselines.is_empty());
    }

    #[test]
    fn test_delete_removes_baseline() {
        let temp_dir = TempDir::new().unwrap();
        let loader = DefaultBaselineLoader::new(temp_dir.path().to_path_buf());

        let record = create_test_baseline_record("TASK-001", "baseline-001", Utc::now());
        loader.save(&record).unwrap();

        let path = temp_dir.path().join("TASK-001").join("baseline-001.yaml");
        assert!(path.is_file());

        loader.delete("TASK-001", "baseline-001").unwrap();
        assert!(!path.is_file());
    }

    #[test]
    fn test_delete_returns_error_for_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let loader = DefaultBaselineLoader::new(temp_dir.path().to_path_buf());

        let result = loader.delete("TASK-001", "nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_load_all_for_task_returns_all_baselines() {
        let temp_dir = TempDir::new().unwrap();
        let loader = DefaultBaselineLoader::new(temp_dir.path().to_path_buf());

        let record1 = create_test_baseline_record("TASK-001", "baseline-a", Utc::now());
        let record2 = create_test_baseline_record("TASK-001", "baseline-b", Utc::now());

        loader.save(&record1).unwrap();
        loader.save(&record2).unwrap();

        let baselines = loader.load_all_for_task("TASK-001").unwrap();
        assert_eq!(baselines.len(), 2);
    }

    #[test]
    fn test_load_all_for_task_returns_empty_for_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let loader = DefaultBaselineLoader::new(temp_dir.path().to_path_buf());

        let baselines = loader.load_all_for_task("TASK-999").unwrap();
        assert!(baselines.is_empty());
    }

    #[test]
    fn test_save_load_roundtrip_preserves_all_data() {
        let temp_dir = TempDir::new().unwrap();
        let loader = DefaultBaselineLoader::new(temp_dir.path().to_path_buf());

        let metadata = BaselineMetadata::new()
            .with_source_impl_version("1.0.0".to_string())
            .with_target_impl_version("2.0.0".to_string())
            .with_task_version("1.2.0".to_string())
            .with_fixture_version("1.1.0".to_string())
            .with_normalizer_version("1.0.5".to_string())
            .with_approved_by("reviewer@example.com".to_string())
            .with_notes("Test notes".to_string());

        let original = BaselineRecord::new(
            "baseline-001".to_string(),
            "TASK-001".to_string(),
            metadata,
            RunnerOutput::default()
                .with_exit_code(Some(0))
                .with_stdout("legacy output".to_string())
                .with_stderr("legacy stderr".to_string()),
            RunnerOutput::default()
                .with_exit_code(Some(0))
                .with_stdout("rust output".to_string())
                .with_stderr("rust stderr".to_string()),
            "normalized legacy".to_string(),
            "normalized rust".to_string(),
            ParityVerdict::Pass,
            Utc::now(),
            Some(PathBuf::from("/artifacts/legacy")),
            Some(PathBuf::from("/artifacts/rust")),
        );

        loader.save(&original).unwrap();

        let loaded = loader.load("TASK-001", "baseline-001").unwrap();
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();

        assert_eq!(loaded.id, original.id);
        assert_eq!(loaded.task_id, original.task_id);
        assert_eq!(
            loaded.metadata.source_impl_version,
            original.metadata.source_impl_version
        );
        assert_eq!(
            loaded.metadata.target_impl_version,
            original.metadata.target_impl_version
        );
        assert_eq!(loaded.metadata.task_version, original.metadata.task_version);
        assert_eq!(
            loaded.metadata.fixture_version,
            original.metadata.fixture_version
        );
        assert_eq!(
            loaded.metadata.normalizer_version,
            original.metadata.normalizer_version
        );
        assert_eq!(loaded.metadata.approved_by, original.metadata.approved_by);
        assert_eq!(loaded.metadata.notes, original.metadata.notes);
        assert_eq!(loaded.legacy_output.stdout, original.legacy_output.stdout);
        assert_eq!(loaded.legacy_output.stderr, original.legacy_output.stderr);
        assert_eq!(
            loaded.legacy_output.exit_code,
            original.legacy_output.exit_code
        );
        assert_eq!(loaded.rust_output.stdout, original.rust_output.stdout);
        assert_eq!(loaded.rust_output.stderr, original.rust_output.stderr);
        assert_eq!(loaded.rust_output.exit_code, original.rust_output.exit_code);
        assert_eq!(loaded.normalized_legacy, original.normalized_legacy);
        assert_eq!(loaded.normalized_rust, original.normalized_rust);
    }

    #[test]
    fn test_baseline_loader_trait_object() {
        let loader: Box<dyn BaselineLoader> = Box::new(DefaultBaselineLoader::default());

        let temp_dir = TempDir::new().unwrap();
        let record = create_test_baseline_record("TASK-001", "baseline-001", Utc::now());
        loader.save(&record).unwrap();

        let loaded = loader.load("TASK-001", "baseline-001").unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().id, "baseline-001");
    }
}
