use opencode_core::runners::RustRunner;
use opencode_core::types::agent_mode::AgentMode;
use opencode_core::types::entry_mode::EntryMode;
use opencode_core::types::execution_policy::ExecutionPolicy;
use opencode_core::types::on_missing_dependency::OnMissingDependency;
use opencode_core::types::provider_mode::ProviderMode;
use opencode_core::types::severity::Severity;
use opencode_core::types::task::Task;
use opencode_core::types::TaskCategory;
use opencode_core::types::TaskInput;

fn create_test_task(command: &str, args: Vec<String>, cwd: &str) -> Task {
    Task::new(
        "P2-008",
        "RustRunner Integration Test",
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
    )
}

#[test]
fn test_rust_runner_integration_execute_echo() {
    let runner = RustRunner::new("opencode");
    let task = create_test_task("echo", vec!["integration_test".to_string()], "/tmp");

    let result = runner.execute(&task).unwrap();
    assert_eq!(result.task_id, "P2-008");
    assert!(result.is_success());
    assert!(result.stdout.contains("integration_test"));
}

#[test]
fn test_rust_runner_integration_execute_with_args() {
    let runner = RustRunner::new("opencode");
    let task = create_test_task(
        "printf",
        vec![
            "%s %d\n".to_string(),
            "hello".to_string(),
            "123".to_string(),
        ],
        "/tmp",
    );

    let result = runner.execute(&task).unwrap();
    assert_eq!(result.task_id, "P2-008");
    assert!(result.is_success());
    assert!(result.stdout.contains("hello"));
    assert!(result.stdout.contains("123"));
}

#[test]
fn test_rust_runner_integration_captures_stderr() {
    let runner = RustRunner::new("opencode");
    let task = create_test_task(
        "sh",
        vec!["-c".to_string(), "echo error message 1>&2".to_string()],
        "/tmp",
    );

    let result = runner.execute(&task).unwrap();
    assert!(result.stderr.contains("error message"));
}

#[test]
fn test_rust_runner_integration_nonexistent_command_fails() {
    let runner = RustRunner::new("opencode");
    let task = create_test_task("this_command_does_not_exist_xyz", vec![], "/tmp");

    let result = runner.execute(&task);
    assert!(result.is_err());
}

#[test]
fn test_rust_runner_integration_with_task_execution_flow() {
    let runner = RustRunner::new("opencode");

    let task = create_test_task("echo", vec!["flow_test".to_string()], "/tmp");

    let result = runner.execute(&task).unwrap();
    assert_eq!(result.task_id, "P2-008");
    assert!(result.is_success());
    assert!(result.stdout.contains("flow_test"));
}

#[test]
fn test_rust_runner_result_serde() {
    use opencode_core::types::task_status::TaskStatus;

    let result = opencode_core::runners::RustRunnerResult::new("P2-008")
        .with_status(TaskStatus::Done)
        .with_exit_code(0)
        .with_stdout("test output".to_string())
        .with_stderr(String::new())
        .with_duration_ms(100);

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"task_id\":\"P2-008\""));
    assert!(json.contains("\"status\":\"Done\""));

    let deserialized: opencode_core::runners::RustRunnerResult =
        serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.task_id, "P2-008");
    assert_eq!(deserialized.status, TaskStatus::Done);
}

#[test]
fn test_rust_runner_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    let _runner = RustRunner::new("opencode");
    assert_send_sync::<RustRunner>();
}
