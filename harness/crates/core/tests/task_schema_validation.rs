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

#[test]
fn test_task_schema_validates_complete_definition() {
    let task = Task::new(
        "P2-001",
        "Define Task Schema",
        TaskCategory::Schema,
        "fixtures/projects/example",
        "Create harness/tasks/schema.yaml defining Task struct with required fields",
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
    );

    let json = serde_json::to_string(&task).unwrap();
    let deserialized: Task = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, "P2-001");
    assert_eq!(deserialized.title, "Define Task Schema");
    assert_eq!(deserialized.category, TaskCategory::Schema);
}

#[test]
fn test_task_schema_yaml_format() {
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

    let task: Task = serde_yaml::from_str(yaml_content).unwrap();
    assert_eq!(task.id, "P2-001");
    assert_eq!(task.fixture_project, "fixtures/projects/example");
    assert_eq!(task.entry_mode, EntryMode::CLI);
}

#[test]
fn test_task_schema_json_format() {
    let json_content = r#"{
  "id": "P2-001",
  "title": "Define Task Schema",
  "category": "schema",
  "fixture_project": "fixtures/projects/example",
  "description": "Create harness/tasks/schema.yaml defining Task struct",
  "expected_outcome": "Schema validates Task definitions correctly",
  "preconditions": ["opencode binary exists"],
  "entry_mode": "CLI",
  "agent_mode": "OneShot",
  "provider_mode": "Both",
  "input": {
    "command": "opencode",
    "args": ["--help"],
    "cwd": "/project"
  },
  "expected_assertions": [
    {"type": "exit_code_equals", "value": 0}
  ],
  "severity": "High",
  "tags": [],
  "execution_policy": "ManualCheck",
  "timeout_seconds": 300,
  "on_missing_dependency": "Fail"
}"#;

    let task: Task = serde_json::from_str(json_content).unwrap();
    assert_eq!(task.id, "P2-001");
    assert_eq!(task.fixture_project, "fixtures/projects/example");
    assert_eq!(task.entry_mode, EntryMode::CLI);
}

#[test]
fn test_task_all_required_fields_present() {
    let task = Task::new(
        "P2-001",
        "Define Task Schema",
        TaskCategory::Schema,
        "fixtures/projects/example",
        "Create harness/tasks/schema.yaml defining Task struct with required fields",
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
    );

    assert!(!task.id.is_empty());
    assert!(!task.title.is_empty());
    assert!(!task.fixture_project.is_empty());
    assert!(!task.description.is_empty());
    assert!(!task.expected_outcome.is_empty());
}
