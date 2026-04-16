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

fn get_smoke_cli_002_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../../../harness/tasks/cli/SMOKE-CLI-002.yaml");
    path
}

fn get_smoke_ws_001_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../../../harness/tasks/workspace/SMOKE-WS-001.yaml");
    path
}

fn get_smoke_ws_002_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../../../harness/tasks/workspace/SMOKE-WS-002.yaml");
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
         NOTE: ID format follows '^[A-Z]+-[A-Z]+-[0-9]+$' pattern (e.g., SMOKE-CLI-001)",
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

#[test]
fn smoke_cli_002_yaml_parses_correctly() {
    let path = get_smoke_cli_002_path();
    let content = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "Failed to read SMOKE-CLI-002.yaml: {} (path: {:?})",
            e, path
        )
    });
    let task: Task =
        serde_yaml::from_str(&content).expect("SMOKE-CLI-002.yaml should parse as valid Task YAML");

    assert_eq!(task.id, "SMOKE-CLI-002");
    assert_eq!(task.title, "CLI version command displays version");
    assert_eq!(task.category, TaskCategory::Smoke);
    assert_eq!(task.fixture_project, "fixtures/projects/cli-basic");
    assert_eq!(task.preconditions, vec!["opencode binary exists"]);
    assert_eq!(task.entry_mode, EntryMode::CLI);
    assert_eq!(task.agent_mode, AgentMode::OneShot);
    assert_eq!(task.provider_mode, ProviderMode::Both);
    assert_eq!(task.input.command, "opencode");
    assert_eq!(task.input.args, vec!["--version"]);
    assert_eq!(task.severity, Severity::Medium);
    assert_eq!(task.tags, vec!["smoke", "cli", "version"]);
    assert_eq!(task.timeout_seconds, 30);
}

#[test]
fn smoke_cli_002_schema_validation_passes() {
    let validator = DefaultTaskSchemaValidator::new();
    let path = get_smoke_cli_002_path();
    let content = std::fs::read_to_string(&path).unwrap();
    let task: Task = serde_yaml::from_str(&content).unwrap();

    let result = validator.validate(&task);
    assert!(
        result.is_ok(),
        "SMOKE-CLI-002 should pass schema validation: {:?}\n\
         NOTE: ID format follows '^[A-Z]+-[A-Z]+-[0-9]+$' pattern (e.g., SMOKE-CLI-002)",
        result
    );
}

#[test]
fn smoke_cli_002_has_correct_assertions() {
    let path = get_smoke_cli_002_path();
    let content = std::fs::read_to_string(&path).unwrap();
    let task: Task = serde_yaml::from_str(&content).unwrap();

    assert_eq!(task.expected_assertions.len(), 2);
    assert!(task
        .expected_assertions
        .contains(&AssertionType::ExitCodeEquals(0)));
    assert!(task
        .expected_assertions
        .contains(&AssertionType::StdoutContains("opencode".to_string())));
}

#[test]
fn smoke_cli_002_task_loader_can_load_task() {
    let path = get_smoke_cli_002_path();
    let content = std::fs::read_to_string(&path).unwrap();
    let task: Task = serde_yaml::from_str(&content).unwrap();

    let validator = DefaultTaskSchemaValidator::new();
    assert!(validator.validate(&task).is_ok());
}

#[test]
fn smoke_ws_001_yaml_parses_correctly() {
    let path = get_smoke_ws_001_path();
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read SMOKE-WS-001.yaml: {} (path: {:?})", e, path));
    let task: Task =
        serde_yaml::from_str(&content).expect("SMOKE-WS-001.yaml should parse as valid Task YAML");

    assert_eq!(task.id, "SMOKE-WS-001");
    assert_eq!(task.title, "Workspace initialization creates clean state");
    assert_eq!(task.category, TaskCategory::Smoke);
    assert_eq!(task.fixture_project, "fixtures/projects/cli-basic");
    assert_eq!(
        task.preconditions,
        vec!["opencode binary exists", "git available"]
    );
    assert_eq!(task.entry_mode, EntryMode::Workspace);
    assert_eq!(task.agent_mode, AgentMode::OneShot);
    assert_eq!(task.provider_mode, ProviderMode::Both);
    assert_eq!(task.input.command, "opencode");
    assert_eq!(task.input.args, vec!["workspace", "init"]);
    assert_eq!(task.severity, Severity::High);
    assert_eq!(task.tags, vec!["smoke", "workspace", "init"]);
    assert_eq!(task.timeout_seconds, 60);
}

#[test]
fn smoke_ws_001_schema_validation_passes() {
    let validator = DefaultTaskSchemaValidator::new();
    let path = get_smoke_ws_001_path();
    let content = std::fs::read_to_string(&path).unwrap();
    let task: Task = serde_yaml::from_str(&content).unwrap();

    let result = validator.validate(&task);
    assert!(
        result.is_ok(),
        "SMOKE-WS-001 should pass schema validation: {:?}\n\
         NOTE: ID format follows '^[A-Z]+-[A-Z]+-[0-9]+$' pattern (e.g., SMOKE-WS-001)",
        result
    );
}

#[test]
fn smoke_ws_001_has_correct_assertions() {
    let path = get_smoke_ws_001_path();
    let content = std::fs::read_to_string(&path).unwrap();
    let task: Task = serde_yaml::from_str(&content).unwrap();

    assert_eq!(task.expected_assertions.len(), 2);
    assert!(task
        .expected_assertions
        .contains(&AssertionType::ExitCodeEquals(0)));
    assert!(task
        .expected_assertions
        .contains(&AssertionType::FileChanged(
            ".opencode/workspace.json".to_string()
        )));
}

#[test]
fn smoke_ws_001_task_loader_can_load_task() {
    let path = get_smoke_ws_001_path();
    let content = std::fs::read_to_string(&path).unwrap();
    let task: Task = serde_yaml::from_str(&content).unwrap();

    let validator = DefaultTaskSchemaValidator::new();
    assert!(validator.validate(&task).is_ok());
}

#[test]
fn smoke_ws_002_yaml_parses_correctly() {
    let path = get_smoke_ws_002_path();
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read SMOKE-WS-002.yaml: {} (path: {:?})", e, path));
    let task: Task =
        serde_yaml::from_str(&content).expect("SMOKE-WS-002.yaml should parse as valid Task YAML");

    assert_eq!(task.id, "SMOKE-WS-002");
    assert_eq!(task.title, "Workspace cleanup removes runtime artifacts");
    assert_eq!(task.category, TaskCategory::Smoke);
    assert_eq!(task.fixture_project, "fixtures/projects/cli-basic");
    assert_eq!(task.preconditions, vec!["opencode binary exists"]);
    assert_eq!(task.entry_mode, EntryMode::Workspace);
    assert_eq!(task.agent_mode, AgentMode::OneShot);
    assert_eq!(task.provider_mode, ProviderMode::Both);
    assert_eq!(task.input.command, "opencode");
    assert_eq!(task.input.args, vec!["workspace", "cleanup"]);
    assert_eq!(task.severity, Severity::Medium);
    assert_eq!(task.tags, vec!["smoke", "workspace", "cleanup"]);
    assert_eq!(task.timeout_seconds, 60);
}

#[test]
fn smoke_ws_002_schema_validation_passes() {
    let validator = DefaultTaskSchemaValidator::new();
    let path = get_smoke_ws_002_path();
    let content = std::fs::read_to_string(&path).unwrap();
    let task: Task = serde_yaml::from_str(&content).unwrap();

    let result = validator.validate(&task);
    assert!(
        result.is_ok(),
        "SMOKE-WS-002 should pass schema validation: {:?}\n\
         NOTE: ID format follows '^[A-Z]+-[A-Z]+-[0-9]+$' pattern (e.g., SMOKE-WS-002)",
        result
    );
}

#[test]
fn smoke_ws_002_has_correct_assertions() {
    let path = get_smoke_ws_002_path();
    let content = std::fs::read_to_string(&path).unwrap();
    let task: Task = serde_yaml::from_str(&content).unwrap();

    assert_eq!(task.expected_assertions.len(), 2);
    assert!(task
        .expected_assertions
        .contains(&AssertionType::ExitCodeEquals(0)));
    assert!(task
        .expected_assertions
        .contains(&AssertionType::NoExtraFilesChanged));
}

#[test]
fn smoke_ws_002_task_loader_can_load_task() {
    let path = get_smoke_ws_002_path();
    let content = std::fs::read_to_string(&path).unwrap();
    let task: Task = serde_yaml::from_str(&content).unwrap();

    let validator = DefaultTaskSchemaValidator::new();
    assert!(validator.validate(&task).is_ok());
}
