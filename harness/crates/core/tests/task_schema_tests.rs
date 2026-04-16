use opencode_core::error::ErrorType;
use opencode_core::loaders::task_validator::{DefaultTaskSchemaValidator, TaskSchemaValidator};
use opencode_core::types::agent_mode::AgentMode;
use opencode_core::types::assertion::AssertionType;
use opencode_core::types::entry_mode::EntryMode;
use opencode_core::types::execution_policy::ExecutionPolicy;
use opencode_core::types::on_missing_dependency::OnMissingDependency;
use opencode_core::types::provider_mode::ProviderMode;
use opencode_core::types::severity::Severity;
use opencode_core::types::task::TaskCategory;
use opencode_core::types::task_input::TaskInput;
use opencode_core::Task;
use tempfile::TempDir;

fn create_valid_task() -> Task {
    Task::new(
        "AB-CD-001",
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

mod required_field_tests {
    use super::*;

    #[test]
    fn test_task_schema_validation_passes_for_complete_task() {
        let validator = DefaultTaskSchemaValidator::new();
        let task = create_valid_task();
        assert!(validator.validate(&task).is_ok());
    }

    #[test]
    fn test_task_schema_validation_fails_on_missing_id() {
        let validator = DefaultTaskSchemaValidator::new();
        let mut task = create_valid_task();
        task.id = "".to_string();
        let result = validator.validate(&task);
        assert!(result.is_err());
        match result.unwrap_err() {
            ErrorType::Config(msg) => {
                assert!(msg.contains("Task ID"))
            }
            _ => panic!("Expected Config error"),
        }
    }

    #[test]
    fn test_task_schema_validation_fails_on_missing_title() {
        let validator = DefaultTaskSchemaValidator::new();
        let mut task = create_valid_task();
        task.title = "".to_string();
        let result = validator.validate(&task);
        assert!(result.is_err());
        match result.unwrap_err() {
            ErrorType::Config(msg) => {
                assert!(msg.contains("title"))
            }
            _ => panic!("Expected Config error"),
        }
    }

    #[test]
    fn test_task_schema_validation_fails_on_missing_fixture_project() {
        let validator = DefaultTaskSchemaValidator::new();
        let mut task = create_valid_task();
        task.fixture_project = "".to_string();
        let result = validator.validate(&task);
        assert!(result.is_err());
        match result.unwrap_err() {
            ErrorType::Config(msg) => {
                assert!(msg.contains("Fixture project"))
            }
            _ => panic!("Expected Config error"),
        }
    }

    #[test]
    fn test_task_schema_validation_fails_on_missing_preconditions() {
        let validator = DefaultTaskSchemaValidator::new();
        let mut task = create_valid_task();
        task.preconditions = vec![];
        let result = validator.validate(&task);
        assert!(result.is_err());
        match result.unwrap_err() {
            ErrorType::Config(msg) => {
                assert!(msg.contains("precondition"))
            }
            _ => panic!("Expected Config error"),
        }
    }

    #[test]
    fn test_task_schema_validation_fails_on_empty_command() {
        let validator = DefaultTaskSchemaValidator::new();
        let mut task = create_valid_task();
        task.input = TaskInput::new("", vec![], "/project");
        let result = validator.validate(&task);
        assert!(result.is_err());
        match result.unwrap_err() {
            ErrorType::Config(msg) => {
                assert!(msg.contains("command"))
            }
            _ => panic!("Expected Config error"),
        }
    }

    #[test]
    fn test_task_schema_validation_fails_on_missing_expected_assertions() {
        let validator = DefaultTaskSchemaValidator::new();
        let mut task = create_valid_task();
        task.expected_assertions = vec![];
        let result = validator.validate(&task);
        assert!(result.is_err());
        match result.unwrap_err() {
            ErrorType::Config(msg) => {
                assert!(msg.contains("assertion"))
            }
            _ => panic!("Expected Config error"),
        }
    }
}

mod enum_variant_tests {
    use super::*;

    #[test]
    fn test_task_category_all_valid_variants() {
        let validator = DefaultTaskSchemaValidator::new();
        let mut task = create_valid_task();

        let variants = vec![
            TaskCategory::Core,
            TaskCategory::Schema,
            TaskCategory::Integration,
            TaskCategory::Regression,
            TaskCategory::Smoke,
            TaskCategory::Performance,
            TaskCategory::Security,
        ];

        for variant in variants {
            task.category = variant;
            assert!(validator.validate(&task).is_ok());
        }
    }

    #[test]
    fn test_entry_mode_all_valid_variants() {
        let validator = DefaultTaskSchemaValidator::new();
        let mut task = create_valid_task();

        let variants = vec![
            EntryMode::CLI,
            EntryMode::API,
            EntryMode::Session,
            EntryMode::Permissions,
            EntryMode::Web,
            EntryMode::Workspace,
            EntryMode::Recovery,
        ];

        for variant in variants {
            task.entry_mode = variant;
            assert!(validator.validate(&task).is_ok());
        }
    }

    #[test]
    fn test_agent_mode_all_valid_variants() {
        let validator = DefaultTaskSchemaValidator::new();
        let mut task = create_valid_task();

        let variants = vec![
            AgentMode::Interactive,
            AgentMode::Batch,
            AgentMode::Daemon,
            AgentMode::OneShot,
        ];

        for variant in variants {
            task.agent_mode = variant;
            assert!(validator.validate(&task).is_ok());
        }
    }

    #[test]
    fn test_provider_mode_all_valid_variants() {
        let validator = DefaultTaskSchemaValidator::new();
        let mut task = create_valid_task();

        let variants = vec![
            ProviderMode::OpenCode,
            ProviderMode::OpenCodeRS,
            ProviderMode::Both,
            ProviderMode::Either,
        ];

        for variant in variants {
            task.provider_mode = variant;
            assert!(validator.validate(&task).is_ok());
        }
    }

    #[test]
    fn test_severity_all_valid_variants() {
        let validator = DefaultTaskSchemaValidator::new();
        let mut task = create_valid_task();

        let variants = vec![
            Severity::Critical,
            Severity::High,
            Severity::Medium,
            Severity::Low,
            Severity::Cosmetic,
        ];

        for variant in variants {
            task.severity = variant;
            assert!(validator.validate(&task).is_ok());
        }
    }

    #[test]
    fn test_execution_policy_all_valid_variants() {
        let validator = DefaultTaskSchemaValidator::new();
        let mut task = create_valid_task();

        let variants = vec![
            ExecutionPolicy::ManualCheck,
            ExecutionPolicy::Blocked,
            ExecutionPolicy::Skip,
        ];

        for variant in variants {
            task.execution_policy = variant;
            assert!(validator.validate(&task).is_ok());
        }
    }

    #[test]
    fn test_on_missing_dependency_all_valid_variants() {
        let validator = DefaultTaskSchemaValidator::new();
        let mut task = create_valid_task();

        let variants = vec![
            OnMissingDependency::Fail,
            OnMissingDependency::Skip,
            OnMissingDependency::Warn,
            OnMissingDependency::Blocked,
        ];

        for variant in variants {
            task.on_missing_dependency = variant;
            assert!(validator.validate(&task).is_ok());
        }
    }

    #[test]
    fn test_invalid_category_enum_variant_fails_deserialization() {
        let yaml_content = r#"
id: AB-CD-001
title: Define Task Schema
category: invalid_category
fixture_project: fixtures/projects/example
description: Create schema
expected_outcome: Outcome
preconditions:
  - opencode exists
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
        let result: Result<Task, _> = serde_yaml::from_str(yaml_content);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_entry_mode_enum_variant_fails_deserialization() {
        let yaml_content = r#"
id: AB-CD-001
title: Define Task Schema
category: schema
fixture_project: fixtures/projects/example
description: Create schema
expected_outcome: Outcome
preconditions:
  - opencode exists
entry_mode: InvalidMode
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
        let result: Result<Task, _> = serde_yaml::from_str(yaml_content);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_agent_mode_enum_variant_fails_deserialization() {
        let yaml_content = r#"
id: AB-CD-001
title: Define Task Schema
category: schema
fixture_project: fixtures/projects/example
description: Create schema
expected_outcome: Outcome
preconditions:
  - opencode exists
entry_mode: CLI
agent_mode: InvalidAgentMode
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
        let result: Result<Task, _> = serde_yaml::from_str(yaml_content);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_severity_enum_variant_fails_deserialization() {
        let yaml_content = r#"
id: AB-CD-001
title: Define Task Schema
category: schema
fixture_project: fixtures/projects/example
description: Create schema
expected_outcome: Outcome
preconditions:
  - opencode exists
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
severity: InvalidSeverity
tags: []
execution_policy: ManualCheck
timeout_seconds: 300
on_missing_dependency: Fail
"#;
        let result: Result<Task, _> = serde_yaml::from_str(yaml_content);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_execution_policy_enum_variant_fails_deserialization() {
        let yaml_content = r#"
id: AB-CD-001
title: Define Task Schema
category: schema
fixture_project: fixtures/projects/example
description: Create schema
expected_outcome: Outcome
preconditions:
  - opencode exists
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
execution_policy: InvalidPolicy
timeout_seconds: 300
on_missing_dependency: Fail
"#;
        let result: Result<Task, _> = serde_yaml::from_str(yaml_content);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_on_missing_dependency_enum_variant_fails_deserialization() {
        let yaml_content = r#"
id: AB-CD-001
title: Define Task Schema
category: schema
fixture_project: fixtures/projects/example
description: Create schema
expected_outcome: Outcome
preconditions:
  - opencode exists
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
on_missing_dependency: InvalidDependency
"#;
        let result: Result<Task, _> = serde_yaml::from_str(yaml_content);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_assertion_type_fails_deserialization() {
        let yaml_content = r#"
id: AB-CD-001
title: Define Task Schema
category: schema
fixture_project: fixtures/projects/example
description: Create schema
expected_outcome: Outcome
preconditions:
  - opencode exists
entry_mode: CLI
agent_mode: OneShot
provider_mode: Both
input:
  command: opencode
  args: ["--help"]
  cwd: "/project"
expected_assertions:
  - type: invalid_assertion_type
    value: 0
severity: High
tags: []
execution_policy: ManualCheck
timeout_seconds: 300
on_missing_dependency: Fail
"#;
        let result: Result<Task, _> = serde_yaml::from_str(yaml_content);
        assert!(result.is_err());
    }
}

mod edge_case_tests {
    use super::*;

    #[test]
    fn test_empty_preconditions_array_fails_validation() {
        let validator = DefaultTaskSchemaValidator::new();
        let mut task = create_valid_task();
        task.preconditions = vec![];
        let result = validator.validate(&task);
        assert!(result.is_err());
        match result.unwrap_err() {
            ErrorType::Config(msg) => {
                assert!(msg.contains("at least one precondition"))
            }
            _ => panic!("Expected Config error"),
        }
    }

    #[test]
    fn test_empty_string_in_preconditions_fails_validation() {
        let validator = DefaultTaskSchemaValidator::new();
        let mut task = create_valid_task();
        task.preconditions = vec!["opencode exists".to_string(), "".to_string()];
        let result = validator.validate(&task);
        assert!(result.is_err());
        match result.unwrap_err() {
            ErrorType::Config(msg) => {
                assert!(msg.contains("Precondition cannot be empty"))
            }
            _ => panic!("Expected Config error"),
        }
    }

    #[test]
    fn test_timeout_zero_fails_validation() {
        let validator = DefaultTaskSchemaValidator::new();
        let mut task = create_valid_task();
        task.timeout_seconds = 0;
        let result = validator.validate(&task);
        assert!(result.is_err());
        match result.unwrap_err() {
            ErrorType::Config(msg) => {
                assert!(msg.contains("Timeout"))
            }
            _ => panic!("Expected Config error"),
        }
    }

    #[test]
    fn test_timeout_negative_fails_deserialization() {
        let yaml_content = r#"
id: AB-CD-001
title: Define Task Schema
category: schema
fixture_project: fixtures/projects/example
description: Create schema
expected_outcome: Outcome
preconditions:
  - opencode exists
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
timeout_seconds: -1
on_missing_dependency: Fail
"#;
        let result: Result<Task, _> = serde_yaml::from_str(yaml_content);
        assert!(result.is_err());
    }

    #[test]
    fn test_timeout_too_large_fails_validation() {
        let validator = DefaultTaskSchemaValidator::new();
        let mut task = create_valid_task();
        task.timeout_seconds = 3601;
        let result = validator.validate(&task);
        assert!(result.is_err());
        match result.unwrap_err() {
            ErrorType::Config(msg) => {
                assert!(msg.contains("Timeout"))
            }
            _ => panic!("Expected Config error"),
        }
    }

    #[test]
    fn test_timeout_at_minimum_boundary_passes() {
        let validator = DefaultTaskSchemaValidator::new();
        let mut task = create_valid_task();
        task.timeout_seconds = 1;
        assert!(validator.validate(&task).is_ok());
    }

    #[test]
    fn test_timeout_at_maximum_boundary_passes() {
        let validator = DefaultTaskSchemaValidator::new();
        let mut task = create_valid_task();
        task.timeout_seconds = 3600;
        assert!(validator.validate(&task).is_ok());
    }

    #[test]
    fn test_title_exceeds_max_length_fails_validation() {
        let validator = DefaultTaskSchemaValidator::new();
        let mut task = create_valid_task();
        task.title = "a".repeat(201);
        let result = validator.validate(&task);
        assert!(result.is_err());
        match result.unwrap_err() {
            ErrorType::Config(msg) => {
                assert!(msg.contains("200 characters"))
            }
            _ => panic!("Expected Config error"),
        }
    }

    #[test]
    fn test_title_at_max_length_passes() {
        let validator = DefaultTaskSchemaValidator::new();
        let mut task = create_valid_task();
        task.title = "a".repeat(200);
        assert!(validator.validate(&task).is_ok());
    }

    #[test]
    fn test_invalid_task_id_pattern_fails_validation() {
        let validator = DefaultTaskSchemaValidator::new();
        let mut task = create_valid_task();
        task.id = "invalid-id".to_string();
        let result = validator.validate(&task);
        assert!(result.is_err());
        match result.unwrap_err() {
            ErrorType::Config(msg) => {
                assert!(msg.contains("does not match pattern"))
            }
            _ => panic!("Expected Config error"),
        }
    }

    #[test]
    fn test_lowercase_task_id_fails_validation() {
        let validator = DefaultTaskSchemaValidator::new();
        let mut task = create_valid_task();
        task.id = "ab-cd-001".to_string();
        let result = validator.validate(&task);
        assert!(result.is_err());
        match result.unwrap_err() {
            ErrorType::Config(msg) => {
                assert!(msg.contains("does not match pattern"))
            }
            _ => panic!("Expected Config error"),
        }
    }

    #[test]
    fn test_valid_task_id_pattern_passes() {
        let validator = DefaultTaskSchemaValidator::new();
        let mut task = create_valid_task();
        task.id = "XX-YY-123".to_string();
        assert!(validator.validate(&task).is_ok());
    }

    #[test]
    fn test_all_assertion_types_pass_validation() {
        let validator = DefaultTaskSchemaValidator::new();
        let mut task = create_valid_task();

        let assertion_types = vec![
            AssertionType::ExitCodeEquals(0),
            AssertionType::ExitCodeEquals(1),
            AssertionType::StdoutContains("hello".to_string()),
            AssertionType::StderrContains("error".to_string()),
            AssertionType::FileChanged("src/main.rs".to_string()),
            AssertionType::NoExtraFilesChanged,
            AssertionType::PermissionPromptSeen("Allow?".to_string()),
        ];

        for assertion in assertion_types {
            task.expected_assertions = vec![assertion];
            assert!(validator.validate(&task).is_ok());
        }
    }

    #[test]
    fn test_multiple_preconditions_all_valid() {
        let validator = DefaultTaskSchemaValidator::new();
        let mut task = create_valid_task();
        task.preconditions = vec![
            "opencode binary exists".to_string(),
            "git available".to_string(),
            "workspace is clean".to_string(),
        ];
        assert!(validator.validate(&task).is_ok());
    }
}

mod yaml_file_validation_tests {
    use super::*;

    #[test]
    fn test_validate_file_with_valid_yaml() {
        let validator = DefaultTaskSchemaValidator::new();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("valid_task.yaml");

        let yaml_content = r#"
id: AB-CD-001
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
    fn test_validate_file_with_missing_required_fields() {
        let validator = DefaultTaskSchemaValidator::new();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("incomplete_task.yaml");

        let yaml_content = r#"
id: AB-CD-001
title: Define Task Schema
category: schema
"#;

        std::fs::write(&file_path, yaml_content).unwrap();
        let result = validator.validate_file(&file_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_file_with_invalid_yaml_syntax() {
        let validator = DefaultTaskSchemaValidator::new();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("invalid_yaml.yaml");

        let yaml_content = "invalid: yaml: content: [";
        std::fs::write(&file_path, yaml_content).unwrap();

        let result = validator.validate_file(&file_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_nonexistent_file_fails() {
        let validator = DefaultTaskSchemaValidator::new();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("nonexistent.yaml");

        let result = validator.validate_file(&file_path);
        assert!(result.is_err());
    }
}
