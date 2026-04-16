use opencode_core::loaders::task_validator::DefaultTaskSchemaValidator;
use opencode_core::types::agent_mode::AgentMode;
use opencode_core::types::assertion::AssertionType;
use opencode_core::types::entry_mode::EntryMode;
use opencode_core::types::provider_mode::ProviderMode;
use opencode_core::types::severity::Severity;
use opencode_core::types::task::TaskCategory;
use opencode_core::Task;
use opencode_core::TaskSchemaValidator;
use std::path::PathBuf;

fn get_smoke_cli_001_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../../../harness/tasks/cli/SMOKE-CLI-001.yaml");
    path
}

#[test]
fn smoke_cli_001_yaml_parses_correctly() {
    let path = get_smoke_cli_001_path();
    let content = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "Failed to read SMOKE-CLI-001.yaml: {} (path: {:?})",
            e, path
        )
    });
    let task: Task =
        serde_yaml::from_str(&content).expect("SMOKE-CLI-001.yaml should parse as valid Task YAML");

    assert_eq!(task.id, "SMOKE-CLI-001");
    assert_eq!(task.title, "CLI help command displays usage");
    assert_eq!(task.category, TaskCategory::Smoke);
    assert_eq!(task.fixture_project, "fixtures/projects/cli-basic");
    assert_eq!(task.preconditions, vec!["opencode binary exists"]);
    assert_eq!(task.entry_mode, EntryMode::CLI);
    assert_eq!(task.agent_mode, AgentMode::OneShot);
    assert_eq!(task.provider_mode, ProviderMode::Both);
    assert_eq!(task.input.command, "opencode");
    assert_eq!(task.input.args, vec!["--help"]);
    assert_eq!(task.severity, Severity::High);
    assert_eq!(task.tags, vec!["smoke", "cli", "help"]);
    assert_eq!(task.timeout_seconds, 30);
}

#[test]
fn smoke_cli_001_schema_validation_passes() {
    let validator = DefaultTaskSchemaValidator::new();
    let path = get_smoke_cli_001_path();
    let content = std::fs::read_to_string(&path).unwrap();
    let task: Task = serde_yaml::from_str(&content).unwrap();

    let result = validator.validate(&task);
    assert!(
        result.is_ok(),
        "SMOKE-CLI-001 should pass schema validation: {:?}\n\
         NOTE: If this fails due to ID pattern mismatch, this is a documented contract gap.\n\
         The spec shows ID format 'SMOKE-CLI-001' but validator expects '^[A-Z]+[0-9]+-[0-9]+$'",
        result
    );
}

#[test]
fn smoke_cli_001_has_correct_assertions() {
    let path = get_smoke_cli_001_path();
    let content = std::fs::read_to_string(&path).unwrap();
    let task: Task = serde_yaml::from_str(&content).unwrap();

    assert_eq!(task.expected_assertions.len(), 2);
    assert!(task
        .expected_assertions
        .contains(&AssertionType::ExitCodeEquals(0)));
    assert!(task
        .expected_assertions
        .contains(&AssertionType::StdoutContains("Usage:".to_string())));
}

#[test]
fn smoke_cli_001_task_loader_can_load_task() {
    let path = get_smoke_cli_001_path();
    let content = std::fs::read_to_string(&path).unwrap();
    let task: Task = serde_yaml::from_str(&content).unwrap();

    let validator = DefaultTaskSchemaValidator::new();
    assert!(validator.validate(&task).is_ok());
}
