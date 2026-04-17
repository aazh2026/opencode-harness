use opencode_core::runners::RustRunner;
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
        "P2-010",
        "RustRunner Smoke Test",
        TaskCategory::Schema,
        "test-fixture",
        "Test RustRunner with actual binary invocation",
        "RustRunner executes binaries correctly",
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
fn test_rust_runner_smoke_execute_echo() {
    let runner = RustRunner::new("rust");
    let input = create_test_runner_input("/bin/echo", vec!["smoke_test".to_string()], "/tmp");

    let result = runner.execute(&input).unwrap();
    assert_eq!(result.session_metadata.task_id, "P2-010");
    assert_eq!(result.exit_code, Some(0));
    assert!(result.stdout.contains("smoke_test"));
}

#[test]
#[ignore]
fn test_rust_runner_smoke_captures_stderr() {
    let runner = RustRunner::new("rust");
    let input = create_test_runner_input(
        "/bin/sh",
        vec!["-c".to_string(), "echo smoke_error 1>&2".to_string()],
        "/tmp",
    );

    let result = runner.execute(&input).unwrap();
    assert!(result.stderr.contains("smoke_error"));
}

#[test]
fn test_rust_runner_smoke_dependency_missing() {
    let runner = RustRunner::new("rust");
    let task = Task::new(
        "P2-010",
        "RustRunner Smoke Test",
        TaskCategory::Schema,
        "test-fixture",
        "Test RustRunner with nonexistent binary",
        "RustRunner fails correctly",
        vec![],
        EntryMode::CLI,
        AgentMode::OneShot,
        ProviderMode::Both,
        TaskInput::new("/nonexistent/rust-runner-binary", vec![], "/tmp"),
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
        Some(PathBuf::from("/nonexistent/rust-runner-binary")),
        ProviderMode::Both,
        opencode_core::types::CaptureOptions::default(),
    );

    let result = runner.execute(&input).unwrap();
    assert_eq!(
        result.failure_kind,
        Some(opencode_core::types::FailureClassification::DependencyMissing)
    );
}
