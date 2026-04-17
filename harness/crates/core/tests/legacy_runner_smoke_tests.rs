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

#[test]
fn test_legacy_runner_basic_execution_returns_valid_runner_output() {
    let runner = LegacyRunner::new("legacy-smoke-basic");
    let temp_dir = tempfile::TempDir::new().unwrap();

    let task = Task::new(
        "P2-009-BASIC",
        "LegacyRunner Basic Smoke Test",
        TaskCategory::Schema,
        "test-fixture",
        "Test basic execution",
        "Basic execution returns valid RunnerOutput",
        vec![],
        EntryMode::CLI,
        AgentMode::OneShot,
        ProviderMode::Both,
        TaskInput::new(
            "/bin/echo",
            vec!["hello_world".to_string()],
            temp_dir.path().to_str().unwrap(),
        ),
        vec![],
        Severity::High,
        ExecutionPolicy::ManualCheck,
        60,
        OnMissingDependency::Fail,
    );

    let input = RunnerInput::new(
        task,
        temp_dir.path().to_path_buf(),
        HashMap::new(),
        60,
        Some(PathBuf::from("/bin/echo")),
        ProviderMode::Both,
        opencode_core::types::CaptureOptions::default(),
    );

    let result = runner.execute(&input);
    assert!(result.is_ok(), "Expected Ok but got: {:?}", result);
    let output = result.unwrap();

    assert_eq!(output.exit_code, Some(0), "Expected exit_code 0");
    assert!(
        output.stdout.contains("hello_world"),
        "Expected stdout to contain 'hello_world', got: {}",
        output.stdout
    );
    assert!(
        output.stdout_path.to_string_lossy().contains("legacy"),
        "Expected stdout_path to be set, got: {}",
        output.stdout_path.display()
    );
    assert!(
        output.stderr_path.to_string_lossy().contains("legacy"),
        "Expected stderr_path to be set, got: {}",
        output.stderr_path.display()
    );
    assert!(
        output.session_metadata.session_id.len() > 0,
        "Expected session_id to be set"
    );
    assert_eq!(
        output.session_metadata.runner_name, "legacy-smoke-basic",
        "Expected runner_name to be 'legacy-smoke-basic'"
    );
    assert!(
        output.capability_summary.binary_available,
        "Expected binary_available to be true"
    );
}

#[test]
fn test_legacy_runner_timeout_behavior_returns_appropriate_failure_classification() {
    let runner = LegacyRunner::new("legacy-smoke-timeout");
    let temp_dir = tempfile::TempDir::new().unwrap();

    let task = Task::new(
        "P2-009-TIMEOUT",
        "LegacyRunner Timeout Smoke Test",
        TaskCategory::Schema,
        "test-fixture",
        "Test timeout behavior",
        "Timeout correctly classified",
        vec![],
        EntryMode::CLI,
        AgentMode::OneShot,
        ProviderMode::Both,
        TaskInput::new(
            "/bin/sleep",
            vec!["10".to_string()],
            temp_dir.path().to_str().unwrap(),
        ),
        vec![],
        Severity::High,
        ExecutionPolicy::ManualCheck,
        60,
        OnMissingDependency::Fail,
    );

    let input = RunnerInput::new(
        task,
        temp_dir.path().to_path_buf(),
        HashMap::new(),
        1,
        Some(PathBuf::from("/bin/sleep")),
        ProviderMode::Both,
        opencode_core::types::CaptureOptions::default(),
    );

    let start = std::time::Instant::now();
    let result = runner.execute(&input);
    let elapsed = start.elapsed();

    assert!(
        result.is_ok(),
        "Expected Ok on timeout but got: {:?}",
        result
    );
    let output = result.unwrap();

    assert_eq!(
        output.failure_kind,
        Some(opencode_core::types::FailureClassification::FlakySuspected),
        "Expected FlakySuspected failure_kind on timeout, got: {:?}",
        output.failure_kind
    );
    assert!(
        output.stderr.contains("timed out") || output.stderr.contains("killed"),
        "Expected stderr to mention timeout or killed, got: {}",
        output.stderr
    );
    assert!(
        elapsed.as_secs() < 5,
        "Expected process to be killed quickly, elapsed: {}s",
        elapsed.as_secs()
    );
}

#[test]
fn test_legacy_runner_env_override_behavior_correctly_passes_environment_variables() {
    let runner = LegacyRunner::new("legacy-smoke-env");
    let temp_dir = tempfile::TempDir::new().unwrap();

    let task = Task::new(
        "P2-009-ENV",
        "LegacyRunner Env Override Smoke Test",
        TaskCategory::Schema,
        "test-fixture",
        "Test env override behavior",
        "Env overrides correctly passed",
        vec![],
        EntryMode::CLI,
        AgentMode::OneShot,
        ProviderMode::Both,
        TaskInput::new("/usr/bin/env", vec![], temp_dir.path().to_str().unwrap()),
        vec![],
        Severity::High,
        ExecutionPolicy::ManualCheck,
        60,
        OnMissingDependency::Fail,
    );

    let mut env_overrides = HashMap::new();
    env_overrides.insert("TEST_SMOKE_VAR_1".to_string(), "value_one".to_string());
    env_overrides.insert("TEST_SMOKE_VAR_2".to_string(), "value_two".to_string());

    let input = RunnerInput::new(
        task,
        temp_dir.path().to_path_buf(),
        env_overrides,
        10,
        Some(PathBuf::from("/usr/bin/env")),
        ProviderMode::Both,
        opencode_core::types::CaptureOptions::default(),
    );

    let result = runner.execute(&input);
    assert!(result.is_ok(), "Expected Ok but got: {:?}", result);
    let output = result.unwrap();

    assert!(
        output.capability_summary.binary_available,
        "Expected binary_available to be true"
    );
    assert!(
        output.stdout.contains("TEST_SMOKE_VAR_1=value_one"),
        "Expected stdout to contain 'TEST_SMOKE_VAR_1=value_one', got: {}",
        output.stdout
    );
    assert!(
        output.stdout.contains("TEST_SMOKE_VAR_2=value_two"),
        "Expected stdout to contain 'TEST_SMOKE_VAR_2=value_two', got: {}",
        output.stdout
    );
}

#[test]
fn test_legacy_runner_artifact_persistence_creates_expected_files_in_artifacts_run_id_legacy() {
    let runner = LegacyRunner::new("legacy-smoke-artifact");
    let temp_dir = tempfile::TempDir::new().unwrap();

    let task = Task::new(
        "P2-009-ARTIFACT",
        "LegacyRunner Artifact Smoke Test",
        TaskCategory::Schema,
        "test-fixture",
        "Test artifact persistence",
        "Artifacts persisted correctly",
        vec![],
        EntryMode::CLI,
        AgentMode::OneShot,
        ProviderMode::Both,
        TaskInput::new(
            "/bin/echo",
            vec!["artifact_test".to_string()],
            temp_dir.path().to_str().unwrap(),
        ),
        vec![],
        Severity::High,
        ExecutionPolicy::ManualCheck,
        60,
        OnMissingDependency::Fail,
    );

    let input = RunnerInput::new(
        task,
        temp_dir.path().to_path_buf(),
        HashMap::new(),
        60,
        Some(PathBuf::from("/bin/echo")),
        ProviderMode::Both,
        opencode_core::types::CaptureOptions::default(),
    );

    let result = runner.execute(&input);
    assert!(result.is_ok(), "Expected Ok but got: {:?}", result);
    let output = result.unwrap();

    let session_id = &output.session_metadata.session_id;
    let base_artifact_path = PathBuf::from("artifacts").join(session_id);

    let legacy_dir = base_artifact_path.join("legacy");
    assert!(
        legacy_dir.exists(),
        "Expected legacy directory to exist at {}, got: {:?}",
        legacy_dir.display(),
        std::fs::read_dir(&base_artifact_path).map(|i| i
            .collect::<Result<Vec<_>, _>>()
            .map(|v| v.len())
            .unwrap_or(0))
    );

    let stdout_file = legacy_dir.join("stdout.txt");
    let stderr_file = legacy_dir.join("stderr.txt");
    let metadata_file = legacy_dir.join("metadata.json");
    let artifacts_dir = legacy_dir.join("artifacts");
    let side_effects_dir = legacy_dir.join("side-effects");

    assert!(
        stdout_file.exists(),
        "Expected stdout.txt to exist at {}",
        stdout_file.display()
    );
    assert!(
        stderr_file.exists(),
        "Expected stderr.txt to exist at {}",
        stderr_file.display()
    );
    assert!(
        metadata_file.exists(),
        "Expected metadata.json to exist at {}",
        metadata_file.display()
    );
    assert!(
        artifacts_dir.exists(),
        "Expected artifacts/ directory to exist at {}",
        artifacts_dir.display()
    );
    assert!(
        side_effects_dir.exists(),
        "Expected side-effects/ directory to exist at {}",
        side_effects_dir.display()
    );

    let stdout_content = std::fs::read_to_string(&stdout_file).unwrap();
    assert!(
        stdout_content.contains("artifact_test"),
        "Expected stdout.txt to contain 'artifact_test', got: {}",
        stdout_content
    );

    let metadata_content = std::fs::read_to_string(&metadata_file).unwrap();
    assert!(
        metadata_content.contains("\"session_id\""),
        "Expected metadata.json to contain session_id"
    );
    assert!(
        metadata_content.contains("\"runner_name\""),
        "Expected metadata.json to contain runner_name"
    );
    assert!(
        metadata_content.contains("legacy-smoke-artifact"),
        "Expected metadata.json to contain runner name value, got: {}",
        metadata_content
    );
}
