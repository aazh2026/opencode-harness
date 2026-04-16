use opencode_core::loaders::DefaultTaskLoader;
use opencode_core::loaders::TaskLoader;
use opencode_core::runners::DifferentialRunner;
use opencode_core::types::agent_mode::AgentMode;
use opencode_core::types::assertion::AssertionType;
use opencode_core::types::entry_mode::EntryMode;
use opencode_core::types::execution_policy::ExecutionPolicy;
use opencode_core::types::on_missing_dependency::OnMissingDependency;
use opencode_core::types::provider_mode::ProviderMode;
use opencode_core::types::severity::Severity;
use opencode_core::types::task::Task;
use opencode_core::types::TaskCategory;
use opencode_core::types::TaskInput;
use std::path::PathBuf;
use tempfile::TempDir;

fn get_fixtures_path() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let core_path = PathBuf::from(manifest_dir);
    let harness_path = core_path
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    harness_path.join("harness/fixtures/projects")
}

#[test]
fn test_differential_runner_integration_with_task_loader() {
    let loader = DefaultTaskLoader::new();
    let runner = DifferentialRunner::new(loader);

    let temp_dir = TempDir::new().unwrap();
    let task_yaml = temp_dir.path().join("integration_task.yaml");

    std::fs::write(
        &task_yaml,
        r#"
id: INTEGRATION-001
title: Integration Test with TaskLoader
category: integration
fixture_project: fixtures/projects/cli-basic
description: Verify DifferentialRunner integrates with TaskLoader
expected_outcome: Task is loaded and executed successfully
preconditions: []
entry_mode: CLI
agent_mode: OneShot
provider_mode: Both
input:
  command: echo
  args: ["integration_test"]
  cwd: "/tmp"
expected_assertions:
  - type: exit_code_equals
    value: 0
  - type: stdout_contains
    value: "integration_test"
severity: High
tags:
  - integration
  - task_loader
execution_policy: ManualCheck
timeout_seconds: 60
on_missing_dependency: Fail
"#,
    )
    .unwrap();

    let loaded_task = runner.task_loader.load_single(&task_yaml).unwrap();
    assert_eq!(loaded_task.id, "INTEGRATION-001");
    assert_eq!(loaded_task.category, TaskCategory::Integration);

    let result = runner.execute(&loaded_task).unwrap();
    assert_eq!(result.task_id, "INTEGRATION-001");
    assert!(result.assertions_passed);
    assert_eq!(result.exit_code, 0);
}

#[test]
fn test_differential_runner_loads_multiple_tasks_from_directory() {
    let loader = DefaultTaskLoader::new();
    let runner = DifferentialRunner::new(loader);

    let temp_dir = TempDir::new().unwrap();

    std::fs::write(
        temp_dir.path().join("task1.yaml"),
        r#"
id: MULTI-001
title: Multiple Tasks Test 1
category: smoke
fixture_project: fixtures/projects/cli-basic
description: First task for multi-task loading
expected_outcome: Passes
preconditions: []
entry_mode: CLI
agent_mode: OneShot
provider_mode: Both
input:
  command: echo
  args: ["task1"]
  cwd: "/tmp"
expected_assertions:
  - type: exit_code_equals
    value: 0
severity: High
tags: []
execution_policy: ManualCheck
timeout_seconds: 60
on_missing_dependency: Fail
"#,
    )
    .unwrap();

    std::fs::write(
        temp_dir.path().join("task2.yaml"),
        r#"
id: MULTI-002
title: Multiple Tasks Test 2
category: smoke
fixture_project: fixtures/projects/cli-basic
description: Second task for multi-task loading
expected_outcome: Passes
preconditions: []
entry_mode: CLI
agent_mode: OneShot
provider_mode: Both
input:
  command: echo
  args: ["task2"]
  cwd: "/tmp"
expected_assertions:
  - type: exit_code_equals
    value: 0
severity: High
tags: []
execution_policy: ManualCheck
timeout_seconds: 60
on_missing_dependency: Fail
"#,
    )
    .unwrap();

    let tasks = runner.task_loader.load_from_dir(temp_dir.path()).unwrap();
    assert_eq!(tasks.len(), 2);

    let results = runner.execute_from_path(temp_dir.path()).unwrap();
    assert_eq!(results.len(), 2);

    let task_ids: Vec<&str> = results.iter().map(|r| r.task_id.as_str()).collect();
    assert!(task_ids.contains(&"MULTI-001"));
    assert!(task_ids.contains(&"MULTI-002"));
}

#[test]
fn test_differential_runner_with_cli_basic_fixture() {
    let fixtures_path = get_fixtures_path();
    let loader = DefaultTaskLoader::new();
    let runner = DifferentialRunner::new(loader);

    let temp_dir = TempDir::new().unwrap();
    let task_yaml = temp_dir.path().join("cli_fixture_task.yaml");

    std::fs::write(
        &task_yaml,
        r#"
id: CLI-FIXTURE-001
title: CLI Basic Fixture Test
category: smoke
fixture_project: fixtures/projects/cli-basic
description: Test with cli-basic fixture
expected_outcome: Works with cli-basic fixture
preconditions: []
entry_mode: CLI
agent_mode: OneShot
provider_mode: Both
input:
  command: echo
  args: ["cli_basic_fixture"]
  cwd: "/tmp"
expected_assertions:
  - type: exit_code_equals
    value: 0
severity: High
tags:
  - fixture
  - cli-basic
execution_policy: ManualCheck
timeout_seconds: 60
on_missing_dependency: Fail
"#,
    )
    .unwrap();

    let loaded_task = runner.task_loader.load_single(&task_yaml).unwrap();
    assert_eq!(loaded_task.fixture_project, "fixtures/projects/cli-basic");

    let result = runner.execute(&loaded_task).unwrap();
    assert!(result.assertions_passed);
    drop(fixtures_path);
}

#[test]
fn test_differential_runner_compares_outputs_correctly() {
    let loader = DefaultTaskLoader::new();
    let runner = DifferentialRunner::new(loader);

    let temp_dir = TempDir::new().unwrap();

    let task_a_yaml = temp_dir.path().join("task_a.yaml");
    std::fs::write(
        &task_a_yaml,
        r#"
id: COMPARE-A
title: Compare Output A
category: smoke
fixture_project: fixtures/projects/cli-basic
description: First output for comparison
expected_outcome: Passes
preconditions: []
entry_mode: CLI
agent_mode: OneShot
provider_mode: Both
input:
  command: echo
  args: ["output_a"]
  cwd: "/tmp"
expected_assertions:
  - type: exit_code_equals
    value: 0
severity: High
tags: []
execution_policy: ManualCheck
timeout_seconds: 60
on_missing_dependency: Fail
"#,
    )
    .unwrap();

    let task_b_yaml = temp_dir.path().join("task_b.yaml");
    std::fs::write(
        &task_b_yaml,
        r#"
id: COMPARE-B
title: Compare Output B
category: smoke
fixture_project: fixtures/projects/cli-basic
description: Second output for comparison
expected_outcome: Passes
preconditions: []
entry_mode: CLI
agent_mode: OneShot
provider_mode: Both
input:
  command: echo
  args: ["output_b"]
  cwd: "/tmp"
expected_assertions:
  - type: exit_code_equals
    value: 0
severity: High
tags: []
execution_policy: ManualCheck
timeout_seconds: 60
on_missing_dependency: Fail
"#,
    )
    .unwrap();

    let result_a = runner.execute_single(&task_a_yaml).unwrap();
    let result_b = runner.execute_single(&task_b_yaml).unwrap();

    assert_ne!(result_a.stdout, result_b.stdout);
    assert!(result_a.stdout.contains("output_a"));
    assert!(result_b.stdout.contains("output_b"));
    assert!(result_a.passed());
    assert!(result_b.passed());
}

#[test]
fn test_differential_runner_differential_result_structure() {
    use opencode_core::runners::DifferentialResult;

    let result = DifferentialResult::new("DIFF-001".to_string());

    assert_eq!(result.task_id, "DIFF-001");
    assert_eq!(result.exit_code, 0);
    assert!(result.stdout.is_empty());
    assert!(result.stderr.is_empty());
    assert!(!result.assertions_passed);
    assert!(!result.output_changed);
}

#[test]
fn test_differential_runner_with_task_struct_directly() {
    let loader = DefaultTaskLoader::new();
    let runner = DifferentialRunner::new(loader);

    let task = Task::new(
        "DIRECT-001",
        "Direct Task Execution",
        TaskCategory::Smoke,
        "fixtures/projects/cli-basic",
        "Test direct task execution",
        "Task runs successfully",
        vec![],
        EntryMode::CLI,
        AgentMode::OneShot,
        ProviderMode::Both,
        TaskInput::new("echo", vec!["direct".to_string()], "/tmp"),
        vec![AssertionType::ExitCodeEquals(0)],
        Severity::Medium,
        ExecutionPolicy::ManualCheck,
        60,
        OnMissingDependency::Fail,
    );

    let result = runner.execute(&task).unwrap();

    assert_eq!(result.task_id, "DIRECT-001");
    assert!(result.passed());
    assert_eq!(result.exit_code, 0);
}

#[test]
fn test_differential_runner_send_and_sync_trait_bounds() {
    fn assert_send_and_sync<T: Send + Sync>() {}
    let loader = DefaultTaskLoader::new();
    let _runner = DifferentialRunner::new(loader);
    assert_send_and_sync::<DifferentialRunner<DefaultTaskLoader>>();
}
