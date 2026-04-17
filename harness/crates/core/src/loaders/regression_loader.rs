use crate::error::{ErrorType, Result};
use crate::types::execution_level::ExecutionLevel;
use crate::types::regression_case::RegressionCase;
use crate::types::regression_case::RegressionStatus;
use std::fs;
use std::path::{Path, PathBuf};

pub trait RegressionLoader: Send + Sync {
    fn load(&self, category: &str, regression_id: &str) -> Result<Option<RegressionCase>>;
    fn load_all(&self) -> Result<Vec<RegressionCase>>;
    fn load_by_status(&self, status: RegressionStatus) -> Result<Vec<RegressionCase>>;
    fn load_by_execution_level(&self, level: ExecutionLevel) -> Result<Vec<RegressionCase>>;
    fn save(&self, case: &RegressionCase) -> Result<()>;
    fn update_status(&self, regression_id: &str, status: RegressionStatus) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct DefaultRegressionLoader {
    base_path: PathBuf,
}

impl DefaultRegressionLoader {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    fn regression_path(&self, category: &str, regression_id: &str) -> PathBuf {
        self.base_path
            .join(category)
            .join(format!("{}.yaml", regression_id))
    }

    fn all_regression_dirs(&self) -> Result<Vec<PathBuf>> {
        if !self.base_path.is_dir() {
            return Ok(Vec::new());
        }

        let mut dirs = Vec::new();
        for entry in fs::read_dir(&self.base_path).map_err(ErrorType::Io)? {
            let entry = entry.map_err(ErrorType::Io)?;
            let path = entry.path();
            if path.is_dir() {
                dirs.push(path);
            }
        }
        Ok(dirs)
    }

    fn load_yaml_file(&self, path: &Path) -> Result<RegressionCase> {
        let content = fs::read_to_string(path).map_err(ErrorType::Io)?;
        serde_yaml::from_str(&content)
            .map_err(|e| ErrorType::Config(format!("Failed to parse regression YAML: {}", e)))
    }

    fn save_yaml_file(&self, path: &Path, case: &RegressionCase) -> Result<()> {
        let content = serde_yaml::to_string(case).map_err(|e| {
            ErrorType::Config(format!("Failed to serialize regression to YAML: {}", e))
        })?;
        fs::write(path, content).map_err(ErrorType::Io)?;
        Ok(())
    }
}

impl Default for DefaultRegressionLoader {
    fn default() -> Self {
        Self::new(PathBuf::from("harness/regression"))
    }
}

impl RegressionLoader for DefaultRegressionLoader {
    fn load(&self, category: &str, regression_id: &str) -> Result<Option<RegressionCase>> {
        let path = self.regression_path(category, regression_id);
        if !path.is_file() {
            return Ok(None);
        }
        match self.load_yaml_file(&path) {
            Ok(case) => Ok(Some(case)),
            Err(e) => Err(e),
        }
    }

    fn load_all(&self) -> Result<Vec<RegressionCase>> {
        let mut cases = Vec::new();

        for dir in self.all_regression_dirs()? {
            for entry in fs::read_dir(&dir).map_err(ErrorType::Io)? {
                let entry = entry.map_err(ErrorType::Io)?;
                let file_path = entry.path();
                if file_path.is_file() {
                    if let Some(ext) = file_path.extension() {
                        if ext == "yaml" || ext == "yml" {
                            match self.load_yaml_file(&file_path) {
                                Ok(case) => cases.push(case),
                                Err(e) => return Err(e),
                            }
                        }
                    }
                }
            }
        }

        cases.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(cases)
    }

    fn load_by_status(&self, status: RegressionStatus) -> Result<Vec<RegressionCase>> {
        let all_cases = self.load_all()?;
        Ok(all_cases
            .into_iter()
            .filter(|case| case.status == status)
            .collect())
    }

    fn load_by_execution_level(&self, level: ExecutionLevel) -> Result<Vec<RegressionCase>> {
        let all_cases = self.load_all()?;
        Ok(all_cases
            .into_iter()
            .filter(|case| case.execution_level == level)
            .collect())
    }

    fn save(&self, case: &RegressionCase) -> Result<()> {
        let category = determine_category(&case.status);
        let path = self.regression_path(&category, &case.id);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(ErrorType::Io)?;
        }
        self.save_yaml_file(&path, case)
    }

    fn update_status(&self, regression_id: &str, status: RegressionStatus) -> Result<()> {
        let all_cases = self.load_all()?;
        let mut found_case = None;
        let mut old_category = None;

        for case in all_cases {
            if case.id == regression_id {
                old_category = Some(determine_category(&case.status));
                found_case = Some(case);
                break;
            }
        }

        let case = found_case.ok_or_else(|| {
            ErrorType::Config(format!("Regression case not found: {}", regression_id))
        })?;

        let mut updated_case = case;
        updated_case.status = status;
        updated_case.updated_at = chrono::Utc::now();

        let new_category = determine_category(&updated_case.status);
        let new_path = self.regression_path(&new_category, &updated_case.id);
        if let Some(parent) = new_path.parent() {
            fs::create_dir_all(parent).map_err(ErrorType::Io)?;
        }
        self.save_yaml_file(&new_path, &updated_case)?;

        if let Some(old_cat) = old_category {
            if old_cat != new_category {
                let old_path = self.regression_path(&old_cat, regression_id);
                if old_path.is_file() {
                    fs::remove_file(old_path).map_err(ErrorType::Io)?;
                }
            }
        }

        Ok(())
    }
}

fn determine_category(status: &RegressionStatus) -> String {
    match status {
        RegressionStatus::Candidate => "candidates".to_string(),
        RegressionStatus::Approved => "approved".to_string(),
        RegressionStatus::Active => "bugs".to_string(),
        RegressionStatus::Suppressed => "suppressed".to_string(),
        RegressionStatus::Resolved => "resolved".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::severity::Severity;
    use chrono::{DateTime, Utc};
    use tempfile::TempDir;

    fn create_test_regression_case(
        id: &str,
        task_id: &str,
        status: RegressionStatus,
        execution_level: ExecutionLevel,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> RegressionCase {
        RegressionCase::new(
            id.to_string(),
            "https://github.com/example/repo/issues/123".to_string(),
            "Background description".to_string(),
            "Root cause summary".to_string(),
            "fixtures/regression/test".to_string(),
            task_id.to_string(),
            "Expected behavior".to_string(),
            Severity::High,
            execution_level,
            status,
            created_at,
            updated_at,
        )
    }

    #[test]
    fn test_default_loader_creation() {
        let loader = DefaultRegressionLoader::default();
        assert_eq!(loader.base_path, PathBuf::from("harness/regression"));
    }

    #[test]
    fn test_loader_with_custom_base_path() {
        let loader = DefaultRegressionLoader::new(PathBuf::from("/custom/path"));
        assert_eq!(loader.base_path, PathBuf::from("/custom/path"));
    }

    #[test]
    fn test_loader_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<DefaultRegressionLoader>();
    }

    #[test]
    fn test_load_retrieves_regression_by_id() {
        let temp_dir = TempDir::new().unwrap();
        let loader = DefaultRegressionLoader::new(temp_dir.path().to_path_buf());

        let case = create_test_regression_case(
            "REG-001",
            "TASK-001",
            RegressionStatus::Candidate,
            ExecutionLevel::AlwaysOn,
            Utc::now(),
            Utc::now(),
        );
        loader.save(&case).unwrap();

        let loaded = loader.load("candidates", "REG-001").unwrap();
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.id, "REG-001");
        assert_eq!(loaded.task_id, "TASK-001");
        assert_eq!(loaded.status, RegressionStatus::Candidate);
        assert_eq!(loaded.execution_level, ExecutionLevel::AlwaysOn);
    }

    #[test]
    fn test_load_returns_none_for_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let loader = DefaultRegressionLoader::new(temp_dir.path().to_path_buf());

        let loaded = loader.load("candidates", "nonexistent").unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn test_load_all_returns_all_regression_cases() {
        let temp_dir = TempDir::new().unwrap();
        let loader = DefaultRegressionLoader::new(temp_dir.path().to_path_buf());

        let case1 = create_test_regression_case(
            "REG-001",
            "TASK-001",
            RegressionStatus::Active,
            ExecutionLevel::AlwaysOn,
            Utc::now(),
            Utc::now(),
        );
        let case2 = create_test_regression_case(
            "REG-002",
            "TASK-002",
            RegressionStatus::Candidate,
            ExecutionLevel::NightlyOnly,
            Utc::now(),
            Utc::now(),
        );
        let case3 = create_test_regression_case(
            "REG-003",
            "TASK-003",
            RegressionStatus::Approved,
            ExecutionLevel::ReleaseOnly,
            Utc::now(),
            Utc::now(),
        );

        loader.save(&case1).unwrap();
        loader.save(&case2).unwrap();
        loader.save(&case3).unwrap();

        let all_cases = loader.load_all().unwrap();
        assert_eq!(all_cases.len(), 3);
    }

    #[test]
    fn test_load_all_returns_empty_for_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let loader = DefaultRegressionLoader::new(temp_dir.path().to_path_buf());

        let all_cases = loader.load_all().unwrap();
        assert!(all_cases.is_empty());
    }

    #[test]
    fn test_load_by_status_filters_by_status() {
        let temp_dir = TempDir::new().unwrap();
        let loader = DefaultRegressionLoader::new(temp_dir.path().to_path_buf());

        let case1 = create_test_regression_case(
            "REG-001",
            "TASK-001",
            RegressionStatus::Active,
            ExecutionLevel::AlwaysOn,
            Utc::now(),
            Utc::now(),
        );
        let case2 = create_test_regression_case(
            "REG-002",
            "TASK-002",
            RegressionStatus::Candidate,
            ExecutionLevel::AlwaysOn,
            Utc::now(),
            Utc::now(),
        );
        let case3 = create_test_regression_case(
            "REG-003",
            "TASK-003",
            RegressionStatus::Candidate,
            ExecutionLevel::AlwaysOn,
            Utc::now(),
            Utc::now(),
        );

        loader.save(&case1).unwrap();
        loader.save(&case2).unwrap();
        loader.save(&case3).unwrap();

        let candidate_cases = loader.load_by_status(RegressionStatus::Candidate).unwrap();
        assert_eq!(candidate_cases.len(), 2);

        let active_cases = loader.load_by_status(RegressionStatus::Active).unwrap();
        assert_eq!(active_cases.len(), 1);
        assert_eq!(active_cases[0].id, "REG-001");
    }

    #[test]
    fn test_load_by_execution_level_filters_by_level() {
        let temp_dir = TempDir::new().unwrap();
        let loader = DefaultRegressionLoader::new(temp_dir.path().to_path_buf());

        let case1 = create_test_regression_case(
            "REG-001",
            "TASK-001",
            RegressionStatus::Active,
            ExecutionLevel::AlwaysOn,
            Utc::now(),
            Utc::now(),
        );
        let case2 = create_test_regression_case(
            "REG-002",
            "TASK-002",
            RegressionStatus::Candidate,
            ExecutionLevel::NightlyOnly,
            Utc::now(),
            Utc::now(),
        );
        let case3 = create_test_regression_case(
            "REG-003",
            "TASK-003",
            RegressionStatus::Approved,
            ExecutionLevel::NightlyOnly,
            Utc::now(),
            Utc::now(),
        );

        loader.save(&case1).unwrap();
        loader.save(&case2).unwrap();
        loader.save(&case3).unwrap();

        let nightly_cases = loader
            .load_by_execution_level(ExecutionLevel::NightlyOnly)
            .unwrap();
        assert_eq!(nightly_cases.len(), 2);

        let always_on_cases = loader
            .load_by_execution_level(ExecutionLevel::AlwaysOn)
            .unwrap();
        assert_eq!(always_on_cases.len(), 1);
        assert_eq!(always_on_cases[0].id, "REG-001");

        let release_cases = loader
            .load_by_execution_level(ExecutionLevel::ReleaseOnly)
            .unwrap();
        assert!(release_cases.is_empty());
    }

    #[test]
    fn test_save_persists_to_correct_yaml_path() {
        let temp_dir = TempDir::new().unwrap();
        let loader = DefaultRegressionLoader::new(temp_dir.path().to_path_buf());

        let case = create_test_regression_case(
            "REG-001",
            "TASK-001",
            RegressionStatus::Candidate,
            ExecutionLevel::AlwaysOn,
            Utc::now(),
            Utc::now(),
        );
        loader.save(&case).unwrap();

        let expected_path = temp_dir.path().join("candidates").join("REG-001.yaml");
        assert!(expected_path.is_file());

        let content = fs::read_to_string(&expected_path).unwrap();
        assert!(content.contains("REG-001"));
        assert!(content.contains("TASK-001"));
    }

    #[test]
    fn test_save_load_roundtrip_preserves_all_data() {
        let temp_dir = TempDir::new().unwrap();
        let loader = DefaultRegressionLoader::new(temp_dir.path().to_path_buf());

        let original = create_test_regression_case(
            "REG-001",
            "TASK-001",
            RegressionStatus::Active,
            ExecutionLevel::ReleaseOnly,
            Utc::now(),
            Utc::now(),
        );

        loader.save(&original).unwrap();

        let loaded = loader.load("bugs", "REG-001").unwrap();
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();

        assert_eq!(loaded.id, original.id);
        assert_eq!(loaded.issue_link, original.issue_link);
        assert_eq!(loaded.background, original.background);
        assert_eq!(loaded.root_cause, original.root_cause);
        assert_eq!(loaded.minimal_fixture, original.minimal_fixture);
        assert_eq!(loaded.task_id, original.task_id);
        assert_eq!(loaded.expected_result, original.expected_result);
        assert_eq!(loaded.severity, original.severity);
        assert_eq!(loaded.execution_level, original.execution_level);
        assert_eq!(loaded.status, original.status);
    }

    #[test]
    fn test_update_status_changes_status() {
        let temp_dir = TempDir::new().unwrap();
        let loader = DefaultRegressionLoader::new(temp_dir.path().to_path_buf());

        let original = create_test_regression_case(
            "REG-001",
            "TASK-001",
            RegressionStatus::Candidate,
            ExecutionLevel::AlwaysOn,
            Utc::now(),
            Utc::now(),
        );
        loader.save(&original).unwrap();

        loader
            .update_status("REG-001", RegressionStatus::Approved)
            .unwrap();

        let old_loaded = loader.load("candidates", "REG-001").unwrap();
        assert!(old_loaded.is_none());

        let new_loaded = loader.load("approved", "REG-001").unwrap();
        assert!(new_loaded.is_some());
        assert_eq!(new_loaded.unwrap().status, RegressionStatus::Approved);
    }

    #[test]
    fn test_update_status_returns_error_for_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let loader = DefaultRegressionLoader::new(temp_dir.path().to_path_buf());

        let result = loader.update_status("nonexistent", RegressionStatus::Active);
        assert!(result.is_err());
    }

    #[test]
    fn test_regression_loader_trait_object() {
        let temp_dir = TempDir::new().unwrap();
        let loader: Box<dyn RegressionLoader> =
            Box::new(DefaultRegressionLoader::new(temp_dir.path().to_path_buf()));

        let case = create_test_regression_case(
            "REG-001",
            "TASK-001",
            RegressionStatus::Candidate,
            ExecutionLevel::AlwaysOn,
            Utc::now(),
            Utc::now(),
        );
        loader.save(&case).unwrap();

        let loaded = loader.load("candidates", "REG-001").unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().id, "REG-001");
    }

    #[test]
    fn test_determine_category() {
        assert_eq!(
            determine_category(&RegressionStatus::Candidate),
            "candidates"
        );
        assert_eq!(determine_category(&RegressionStatus::Approved), "approved");
        assert_eq!(determine_category(&RegressionStatus::Active), "bugs");
        assert_eq!(
            determine_category(&RegressionStatus::Suppressed),
            "suppressed"
        );
        assert_eq!(determine_category(&RegressionStatus::Resolved), "resolved");
    }
}
