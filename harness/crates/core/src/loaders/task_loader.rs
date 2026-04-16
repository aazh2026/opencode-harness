use crate::error::{ErrorType, Result};
use crate::types::task::Task;
use std::fs;
use std::path::Path;

pub trait TaskLoader: Send + Sync {
    fn load_from_dir(&self, path: &Path) -> Result<Vec<Task>>;
    fn load_single(&self, path: &Path) -> Result<Task>;
}

pub struct DefaultTaskLoader;

impl DefaultTaskLoader {
    pub fn new() -> Self {
        Self
    }

    fn load_yaml_file(&self, path: &Path) -> Result<Task> {
        let content = fs::read_to_string(path).map_err(ErrorType::Io)?;
        serde_yaml::from_str(&content)
            .map_err(|e| ErrorType::Config(format!("Failed to parse task YAML: {}", e)))
    }
}

impl Default for DefaultTaskLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskLoader for DefaultTaskLoader {
    fn load_from_dir(&self, path: &Path) -> Result<Vec<Task>> {
        if !path.is_dir() {
            return Err(ErrorType::Config(format!(
                "Path is not a directory: {}",
                path.display()
            )));
        }

        let mut tasks = Vec::new();
        for entry in fs::read_dir(path).map_err(ErrorType::Io)? {
            let entry = entry.map_err(ErrorType::Io)?;
            let file_path = entry.path();

            if file_path.is_file() {
                if let Some(ext) = file_path.extension() {
                    if ext == "yaml" || ext == "yml" {
                        match self.load_yaml_file(&file_path) {
                            Ok(task) => tasks.push(task),
                            Err(e) => {
                                return Err(ErrorType::Config(format!(
                                    "Failed to load task from {}: {}",
                                    file_path.display(),
                                    e
                                )));
                            }
                        }
                    }
                }
            }
        }

        Ok(tasks)
    }

    fn load_single(&self, path: &Path) -> Result<Task> {
        if !path.is_file() {
            return Err(ErrorType::Config(format!(
                "Path is not a file: {}",
                path.display()
            )));
        }

        self.load_yaml_file(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loaders::task_validator::{DefaultTaskSchemaValidator, TaskSchemaValidator};
    use tempfile::TempDir;

    fn create_valid_task_yaml() -> &'static str {
        r#"
id: TEST-TASK-001
title: Define Task Schema
category: schema
fixture_project: fixtures/projects/example
description: Create harness/tasks/schema.yaml defining Task struct
expected_outcome: Schema validates Task definitions correctly
preconditions:
  - opencode binary exists
entry_mode: CLI
agent_mode: OneShot
provider_mode: Both
input:
  command: opencode
  args: ["--help"]
  cwd: "/project"
expected_assertions:
  - type: exit_code_equals
    value: 0
severity: High
tags: []
execution_policy: ManualCheck
timeout_seconds: 300
on_missing_dependency: Fail
"#
    }

    fn create_second_task_yaml() -> &'static str {
        r#"
id: TEST-TASK-002
title: Another Task
category: smoke
fixture_project: fixtures/projects/cli-basic
description: Another test task
expected_outcome: Task runs successfully
preconditions:
  - opencode binary exists
entry_mode: CLI
agent_mode: OneShot
provider_mode: Both
input:
  command: opencode
  args: ["--version"]
  cwd: "/project"
expected_assertions:
  - type: exit_code_equals
    value: 0
severity: Medium
tags: []
execution_policy: ManualCheck
timeout_seconds: 60
on_missing_dependency: Fail
"#
    }

    #[test]
    fn test_default_loader_creation() {
        let loader = DefaultTaskLoader::new();
        drop(loader);
    }

    #[test]
    fn test_loader_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<DefaultTaskLoader>();
    }

    #[test]
    fn test_load_single_valid_yaml() {
        let loader = DefaultTaskLoader::new();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("task.yaml");

        std::fs::write(&file_path, create_valid_task_yaml()).unwrap();

        let task = loader.load_single(&file_path).unwrap();
        assert_eq!(task.id, "TEST-TASK-001");
        assert_eq!(task.title, "Define Task Schema");
    }

    #[test]
    fn test_load_single_invalid_yaml() {
        let loader = DefaultTaskLoader::new();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("invalid.yaml");

        std::fs::write(&file_path, "invalid: yaml: content: [").unwrap();

        assert!(loader.load_single(&file_path).is_err());
    }

    #[test]
    fn test_load_single_nonexistent_file() {
        let loader = DefaultTaskLoader::new();
        let result = loader.load_single(Path::new("/nonexistent/path/task.yaml"));
        assert!(result.is_err());
    }

    #[test]
    fn test_load_single_directory_error() {
        let loader = DefaultTaskLoader::new();
        let temp_dir = TempDir::new().unwrap();

        let result = loader.load_single(temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_load_from_dir_multiple_tasks() {
        let loader = DefaultTaskLoader::new();
        let temp_dir = TempDir::new().unwrap();

        std::fs::write(temp_dir.path().join("task1.yaml"), create_valid_task_yaml()).unwrap();
        std::fs::write(
            temp_dir.path().join("task2.yaml"),
            create_second_task_yaml(),
        )
        .unwrap();

        let tasks = loader.load_from_dir(temp_dir.path()).unwrap();
        assert_eq!(tasks.len(), 2);

        let ids: Vec<&str> = tasks.iter().map(|t| t.id.as_str()).collect();
        assert!(ids.contains(&"TEST-TASK-001"));
        assert!(ids.contains(&"TEST-TASK-002"));
    }

    #[test]
    fn test_load_from_dir_empty_directory() {
        let loader = DefaultTaskLoader::new();
        let temp_dir = TempDir::new().unwrap();

        let tasks = loader.load_from_dir(temp_dir.path()).unwrap();
        assert!(tasks.is_empty());
    }

    #[test]
    fn test_load_from_dir_with_subdirectories() {
        let loader = DefaultTaskLoader::new();
        let temp_dir = TempDir::new().unwrap();

        std::fs::create_dir(temp_dir.path().join("subdir")).unwrap();
        std::fs::write(temp_dir.path().join("task.yaml"), create_valid_task_yaml()).unwrap();
        std::fs::write(
            temp_dir.path().join("subdir").join("task2.yaml"),
            create_second_task_yaml(),
        )
        .unwrap();

        let tasks = loader.load_from_dir(temp_dir.path()).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, "TEST-TASK-001");
    }

    #[test]
    fn test_load_from_dir_nested_only_top_level() {
        let loader = DefaultTaskLoader::new();
        let temp_dir = TempDir::new().unwrap();

        std::fs::create_dir(temp_dir.path().join("subdir")).unwrap();
        std::fs::write(temp_dir.path().join("task.yaml"), create_valid_task_yaml()).unwrap();
        std::fs::write(
            temp_dir.path().join("subdir").join("task2.yaml"),
            create_second_task_yaml(),
        )
        .unwrap();

        let tasks = loader.load_from_dir(temp_dir.path()).unwrap();
        assert_eq!(tasks.len(), 1);
    }

    #[test]
    fn test_load_from_dir_nonexistent_directory() {
        let loader = DefaultTaskLoader::new();
        let result = loader.load_from_dir(Path::new("/nonexistent/directory"));
        assert!(result.is_err());
    }

    #[test]
    fn test_load_from_dir_file_instead_of_directory() {
        let loader = DefaultTaskLoader::new();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("task.yaml");

        std::fs::write(&file_path, create_valid_task_yaml()).unwrap();

        let result = loader.load_from_dir(&file_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_from_dir_skips_non_yaml_files() {
        let loader = DefaultTaskLoader::new();
        let temp_dir = TempDir::new().unwrap();

        std::fs::write(temp_dir.path().join("task.yaml"), create_valid_task_yaml()).unwrap();
        std::fs::write(temp_dir.path().join("readme.txt"), "This is not a task").unwrap();
        std::fs::write(temp_dir.path().join("data.json"), "{\"key\": \"value\"}").unwrap();

        let tasks = loader.load_from_dir(temp_dir.path()).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, "TEST-TASK-001");
    }

    #[test]
    fn test_task_loader_trait_object() {
        let loader: Box<dyn TaskLoader> = Box::new(DefaultTaskLoader::new());

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("task.yaml");
        std::fs::write(&file_path, create_valid_task_yaml()).unwrap();

        let task = loader.load_single(&file_path).unwrap();
        assert_eq!(task.id, "TEST-TASK-001");

        let tasks = loader.load_from_dir(temp_dir.path()).unwrap();
        assert_eq!(tasks.len(), 1);
    }

    #[test]
    fn test_task_loader_with_validator_integration() {
        let loader = DefaultTaskLoader::new();
        let validator = DefaultTaskSchemaValidator::new();

        let temp_dir = TempDir::new().unwrap();
        let task1_path = temp_dir.path().join("task1.yaml");
        let task2_path = temp_dir.path().join("task2.yaml");

        std::fs::write(&task1_path, create_valid_task_yaml()).unwrap();
        std::fs::write(&task2_path, create_second_task_yaml()).unwrap();

        let tasks = loader.load_from_dir(temp_dir.path()).unwrap();
        assert_eq!(tasks.len(), 2);

        for task in &tasks {
            assert!(
                validator.validate(task).is_ok(),
                "Task {} should pass validation",
                task.id
            );
        }

        let single_task = loader.load_single(&task1_path).unwrap();
        assert!(validator.validate(&single_task).is_ok());
    }
}
