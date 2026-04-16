use crate::error::{ErrorType, Result};
use crate::types::assertion::AssertionType;
use crate::types::execution_policy::ExecutionPolicy;
use crate::types::on_missing_dependency::OnMissingDependency;
use crate::types::task::{Task, TaskCategory};
use crate::types::task_input::TaskInput;
use regex::Regex;
use std::path::Path;

pub trait TaskSchemaValidator: Send + Sync {
    fn validate(&self, task: &Task) -> Result<()>;
    fn validate_file(&self, path: &Path) -> Result<()>;
}

pub struct DefaultTaskSchemaValidator {
    id_pattern: Regex,
}

impl DefaultTaskSchemaValidator {
    pub fn new() -> Self {
        Self {
            id_pattern: Regex::new(r"^[A-Z]+-[A-Z]+-[0-9]+$").unwrap(),
        }
    }

    fn validate_id(&self, id: &str) -> Result<()> {
        if !self.id_pattern.is_match(id) {
            return Err(ErrorType::Config(format!(
                "Task ID '{}' does not match pattern '^[A-Z]+-[A-Z]+-[0-9]+$'",
                id
            )));
        }
        Ok(())
    }

    fn validate_title(&self, title: &str) -> Result<()> {
        if title.is_empty() {
            return Err(ErrorType::Config("Task title cannot be empty".to_string()));
        }
        if title.len() > 200 {
            return Err(ErrorType::Config(format!(
                "Task title exceeds maximum length of 200 characters: {}",
                title.len()
            )));
        }
        Ok(())
    }

    fn validate_category(&self, category: &TaskCategory) -> Result<()> {
        match category {
            TaskCategory::Core
            | TaskCategory::Schema
            | TaskCategory::Integration
            | TaskCategory::Regression
            | TaskCategory::Smoke
            | TaskCategory::Performance
            | TaskCategory::Security => Ok(()),
        }
    }

    fn validate_fixture_project(&self, fixture_project: &str) -> Result<()> {
        if fixture_project.is_empty() {
            return Err(ErrorType::Config(
                "Fixture project cannot be empty".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_preconditions(&self, preconditions: &[String]) -> Result<()> {
        if preconditions.is_empty() {
            return Err(ErrorType::Config(
                "Task must have at least one precondition".to_string(),
            ));
        }
        for precondition in preconditions {
            if precondition.is_empty() {
                return Err(ErrorType::Config(
                    "Precondition cannot be empty string".to_string(),
                ));
            }
        }
        Ok(())
    }

    fn validate_input(&self, input: &TaskInput) -> Result<()> {
        if input.command.is_empty() {
            return Err(ErrorType::Config(
                "Task input command cannot be empty".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_assertions(&self, assertions: &[AssertionType]) -> Result<()> {
        if assertions.is_empty() {
            return Err(ErrorType::Config(
                "Task must have at least one expected assertion".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_timeout(&self, timeout_seconds: u64) -> Result<()> {
        if !(1..=3600).contains(&timeout_seconds) {
            return Err(ErrorType::Config(format!(
                "Timeout seconds must be between 1 and 3600, got {}",
                timeout_seconds
            )));
        }
        Ok(())
    }

    fn validate_execution_policy(&self, policy: &ExecutionPolicy) -> Result<()> {
        match policy {
            ExecutionPolicy::ManualCheck | ExecutionPolicy::Blocked | ExecutionPolicy::Skip => {
                Ok(())
            }
        }
    }

    fn validate_on_missing_dependency(&self, on_missing: &OnMissingDependency) -> Result<()> {
        match on_missing {
            OnMissingDependency::Fail
            | OnMissingDependency::Skip
            | OnMissingDependency::Warn
            | OnMissingDependency::Blocked => Ok(()),
        }
    }
}

impl Default for DefaultTaskSchemaValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskSchemaValidator for DefaultTaskSchemaValidator {
    fn validate(&self, task: &Task) -> Result<()> {
        self.validate_id(&task.id)?;
        self.validate_title(&task.title)?;
        self.validate_category(&task.category)?;
        self.validate_fixture_project(&task.fixture_project)?;
        self.validate_preconditions(&task.preconditions)?;
        self.validate_input(&task.input)?;
        self.validate_assertions(&task.expected_assertions)?;
        self.validate_timeout(task.timeout_seconds)?;
        self.validate_execution_policy(&task.execution_policy)?;
        self.validate_on_missing_dependency(&task.on_missing_dependency)?;
        Ok(())
    }

    fn validate_file(&self, path: &Path) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        let task: Task = serde_yaml::from_str(&content)
            .map_err(|e| ErrorType::Config(format!("Failed to parse task YAML: {}", e)))?;
        self.validate(&task)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::agent_mode::AgentMode;
    use crate::types::entry_mode::EntryMode;
    use crate::types::provider_mode::ProviderMode;
    use crate::types::severity::Severity;
    use tempfile::TempDir;

    fn create_valid_task() -> Task {
        Task::new(
            "P2-001",
            "Define Task Schema",
            TaskCategory::Schema,
            "fixtures/projects/example",
            "Create harness/tasks/schema.yaml defining Task struct",
            "Schema validates Task definitions correctly",
            vec!["opencode binary exists".to_string()],
            EntryMode::CLI,
            AgentMode::OneShot,
            ProviderMode::Both,
            TaskInput::new("opencode", vec!["--help".to_string()], "/project"),
            vec![AssertionType::ExitCodeEquals(0)],
            Severity::High,
            ExecutionPolicy::ManualCheck,
            300,
            OnMissingDependency::Fail,
        )
    }

    #[test]
    fn test_default_validator_creation() {
        let validator = DefaultTaskSchemaValidator::new();
        assert!(std::mem::size_of_val(&validator) > 0);
    }

    #[test]
    fn test_validator_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<DefaultTaskSchemaValidator>();
    }

    #[test]
    fn test_validate_valid_task() {
        let validator = DefaultTaskSchemaValidator::new();
        let task = create_valid_task();
        assert!(validator.validate(&task).is_ok());
    }

    #[test]
    fn test_validate_invalid_id_pattern() {
        let validator = DefaultTaskSchemaValidator::new();
        let mut task = create_valid_task();
        task.id = "invalid-id".to_string();
        assert!(validator.validate(&task).is_err());
    }

    #[test]
    fn test_validate_lowercase_id() {
        let validator = DefaultTaskSchemaValidator::new();
        let mut task = create_valid_task();
        task.id = "p2-001".to_string();
        assert!(validator.validate(&task).is_err());
    }

    #[test]
    fn test_validate_empty_title() {
        let validator = DefaultTaskSchemaValidator::new();
        let mut task = create_valid_task();
        task.title = "".to_string();
        assert!(validator.validate(&task).is_err());
    }

    #[test]
    fn test_validate_title_too_long() {
        let validator = DefaultTaskSchemaValidator::new();
        let mut task = create_valid_task();
        task.title = "a".repeat(201);
        assert!(validator.validate(&task).is_err());
    }

    #[test]
    fn test_validate_empty_fixture_project() {
        let validator = DefaultTaskSchemaValidator::new();
        let mut task = create_valid_task();
        task.fixture_project = "".to_string();
        assert!(validator.validate(&task).is_err());
    }

    #[test]
    fn test_validate_empty_preconditions() {
        let validator = DefaultTaskSchemaValidator::new();
        let mut task = create_valid_task();
        task.preconditions = vec![];
        assert!(validator.validate(&task).is_err());
    }

    #[test]
    fn test_validate_empty_command() {
        let validator = DefaultTaskSchemaValidator::new();
        let mut task = create_valid_task();
        task.input = TaskInput::new("", vec![], "/project");
        assert!(validator.validate(&task).is_err());
    }

    #[test]
    fn test_validate_empty_assertions() {
        let validator = DefaultTaskSchemaValidator::new();
        let mut task = create_valid_task();
        task.expected_assertions = vec![];
        assert!(validator.validate(&task).is_err());
    }

    #[test]
    fn test_validate_timeout_too_low() {
        let validator = DefaultTaskSchemaValidator::new();
        let mut task = create_valid_task();
        task.timeout_seconds = 0;
        assert!(validator.validate(&task).is_err());
    }

    #[test]
    fn test_validate_timeout_too_high() {
        let validator = DefaultTaskSchemaValidator::new();
        let mut task = create_valid_task();
        task.timeout_seconds = 3601;
        assert!(validator.validate(&task).is_err());
    }

    #[test]
    fn test_validate_valid_timeout_boundaries() {
        let validator = DefaultTaskSchemaValidator::new();
        let mut task = create_valid_task();

        task.timeout_seconds = 1;
        assert!(validator.validate(&task).is_ok());

        task.timeout_seconds = 3600;
        assert!(validator.validate(&task).is_ok());
    }

    #[test]
    fn test_validate_execution_policy() {
        let validator = DefaultTaskSchemaValidator::new();
        let task = create_valid_task();
        assert!(validator.validate(&task).is_ok());
    }

    #[test]
    fn test_validate_on_missing_dependency() {
        let validator = DefaultTaskSchemaValidator::new();
        let task = create_valid_task();
        assert!(validator.validate(&task).is_ok());
    }

    #[test]
    fn test_validate_file_valid_yaml() {
        let validator = DefaultTaskSchemaValidator::new();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("task.yaml");

        let yaml_content = r#"
id: P2-001
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
"#;

        std::fs::write(&file_path, yaml_content).unwrap();
        assert!(validator.validate_file(&file_path).is_ok());
    }

    #[test]
    fn test_validate_file_invalid_yaml() {
        let validator = DefaultTaskSchemaValidator::new();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("invalid.yaml");

        std::fs::write(&file_path, "invalid: yaml: content: [").unwrap();
        assert!(validator.validate_file(&file_path).is_err());
    }

    #[test]
    fn test_validate_file_missing_fields() {
        let validator = DefaultTaskSchemaValidator::new();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("incomplete.yaml");

        let yaml_content = r#"
id: P2-001
title: Define Task Schema
"#;

        std::fs::write(&file_path, yaml_content).unwrap();
        assert!(validator.validate_file(&file_path).is_err());
    }

    #[test]
    fn test_assertion_types_all_valid() {
        let validator = DefaultTaskSchemaValidator::new();
        let mut task = create_valid_task();

        let assertion_types = vec![
            AssertionType::ExitCodeEquals(0),
            AssertionType::StdoutContains("hello".to_string()),
            AssertionType::StderrContains("error".to_string()),
            AssertionType::FileChanged("src/main.rs".to_string()),
            AssertionType::NoExtraFilesChanged,
            AssertionType::PermissionPromptSeen("Allow?".to_string()),
        ];

        for assertion in assertion_types {
            task.expected_assertions = vec![assertion.clone()];
            assert!(
                validator.validate(&task).is_ok(),
                "Failed for {:?}",
                assertion
            );
        }
    }

    #[test]
    fn test_task_category_all_variants() {
        let validator = DefaultTaskSchemaValidator::new();
        let mut task = create_valid_task();

        let categories = vec![
            TaskCategory::Core,
            TaskCategory::Schema,
            TaskCategory::Integration,
            TaskCategory::Regression,
            TaskCategory::Smoke,
            TaskCategory::Performance,
            TaskCategory::Security,
        ];

        for category in categories {
            task.category = category;
            assert!(validator.validate(&task).is_ok());
        }
    }

    #[test]
    fn test_execution_policy_all_variants() {
        let validator = DefaultTaskSchemaValidator::new();
        let mut task = create_valid_task();

        task.execution_policy = ExecutionPolicy::ManualCheck;
        assert!(validator.validate(&task).is_ok());

        task.execution_policy = ExecutionPolicy::Blocked;
        assert!(validator.validate(&task).is_ok());

        task.execution_policy = ExecutionPolicy::Skip;
        assert!(validator.validate(&task).is_ok());
    }

    #[test]
    fn test_on_missing_dependency_all_variants() {
        let validator = DefaultTaskSchemaValidator::new();
        let mut task = create_valid_task();

        task.on_missing_dependency = OnMissingDependency::Fail;
        assert!(validator.validate(&task).is_ok());

        task.on_missing_dependency = OnMissingDependency::Skip;
        assert!(validator.validate(&task).is_ok());

        task.on_missing_dependency = OnMissingDependency::Warn;
        assert!(validator.validate(&task).is_ok());

        task.on_missing_dependency = OnMissingDependency::Blocked;
        assert!(validator.validate(&task).is_ok());
    }
}
