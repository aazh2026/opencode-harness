use opencode_core::runners::LegacyRunner;
use opencode_core::types::agent_mode::AgentMode;
use opencode_core::types::entry_mode::EntryMode;
use opencode_core::types::execution_policy::ExecutionPolicy;
use opencode_core::types::on_missing_dependency::OnMissingDependency;
use opencode_core::types::provider_mode::ProviderMode;
use opencode_core::types::runner_input::RunnerInput;
use opencode_core::types::severity::Severity;
use opencode_core::types::task::Task;
use opencode_core::types::TaskCategory;
use opencode_core::types::TaskInput;
use std::collections::HashMap;
use std::path::PathBuf;

fn create_test_runner_input(command: &str, args: Vec<String>, cwd: &str) -> RunnerInput {
    let task = Task::new(
        "P2-009",
        "LegacyRunner Smoke Test",
        TaskCategory::Schema,
        "test-fixture",
        "Test LegacyRunner with actual binary invocation",
        "LegacyRunner executes binaries correctly",
        vec![],
        EntryMode::CLI,
        AgentMode::OneShot,
        ProviderMode::Both,
        TaskInput::new(command, args, cwd),
        vec![],
        Severity::High,
        ExecutionPolicy::ManualCheck,
        60,
        OnMissingDependency::Fail,
    );
    RunnerInput::new(
        task,
        PathBuf::from(cwd),
        HashMap::new(),
        60,
        None,
        ProviderMode::Both,
        opencode_core::types::CaptureOptions::default(),
    )
}

#[test]
#[ignore]
fn test_legacy_runner_integration_execute_echo() {
    let runner = LegacyRunner::new("legacy");
    let input = create_test_runner_input("echo", vec!["smoke_test".to_string()], "/tmp");

    let result = runner.execute(&input).unwrap();
    assert_eq!(result.session_metadata.task_id, "P2-009");
    assert_eq!(result.exit_code, Some(0));
    assert!(result.stdout.contains("smoke_test"));
}

#[test]
#[ignore]
fn test_legacy_runner_integration_execute_with_binary_path() {
    let runner = LegacyRunner::new("legacy");
    let input = create_test_runner_input("/bin/echo", vec!["binary_path_test".to_string()], "/tmp");

    let result = runner.execute(&input).unwrap();
    assert_eq!(result.exit_code, Some(0));
}

#[test]
#[ignore]
fn test_legacy_runner_integration_nonexistent_binary_fails() {
    let runner = LegacyRunner::new("legacy");
    let task = Task::new(
        "P2-009",
        "LegacyRunner Smoke Test",
        TaskCategory::Schema,
        "test-fixture",
        "Test LegacyRunner with nonexistent binary",
        "LegacyRunner fails correctly",
        vec![],
        EntryMode::CLI,
        AgentMode::OneShot,
        ProviderMode::Both,
        TaskInput::new("/nonexistent/binary", vec![], "/tmp"),
        vec![],
        Severity::High,
        ExecutionPolicy::ManualCheck,
        60,
        OnMissingDependency::Fail,
    );

    let input = RunnerInput::new(
        task,
        PathBuf::from("/tmp"),
        HashMap::new(),
        60,
        Some(PathBuf::from("/nonexistent/binary")),
        ProviderMode::Both,
        opencode_core::types::CaptureOptions::default(),
    );

    let result = runner.execute(&input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(
        output.failure_kind,
        Some(opencode_core::types::FailureClassification::DependencyMissing)
    );
}

#[test]
fn test_legacy_runner_result_serde() {
    use opencode_core::types::task_status::TaskStatus;

    let result = opencode_core::runners::LegacyRunnerResult::new("P2-009")
        .with_status(TaskStatus::Done)
        .with_exit_code(0)
        .with_stdout("test output".to_string())
        .with_stderr(String::new())
        .with_duration_ms(100);

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"task_id\":\"P2-009\""));
    assert!(json.contains("\"status\":\"Done\""));

    let deserialized: opencode_core::runners::LegacyRunnerResult =
        serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.task_id, "P2-009");
    assert_eq!(deserialized.status, TaskStatus::Done);
}

#[test]
fn test_legacy_runner_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    let _runner = LegacyRunner::new("legacy");
    assert_send_sync::<LegacyRunner>();
}
