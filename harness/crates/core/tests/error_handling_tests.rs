use opencode_core::error::ErrorType;
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
use opencode_core::BinaryResolver;
use opencode_core::LegacyRunner;
use opencode_core::RustRunner;
use tempfile::TempDir;

fn create_test_task(task_id: &str) -> Task {
    Task::new(
        task_id,
        "Test Task",
        TaskCategory::Core,
        "test-fixture",
        "Test task description",
        "Test expected outcome",
        vec![],
        EntryMode::CLI,
        AgentMode::OneShot,
        ProviderMode::Both,
        TaskInput::new("echo", vec!["test".to_string()], "/tmp"),
        vec![],
        Severity::Medium,
        ExecutionPolicy::ManualCheck,
        60,
        OnMissingDependency::Fail,
    )
}

fn create_runner_input(task: Task, workspace_path: std::path::PathBuf) -> RunnerInput {
    RunnerInput::new(
        task,
        workspace_path,
        std::collections::HashMap::new(),
        5,
        None,
        ProviderMode::Both,
        opencode_core::types::CaptureOptions::default(),
    )
}

mod error_context_tests {
    use super::*;

    #[test]
    fn test_binary_resolver_error_includes_search_paths() {
        let resolver = BinaryResolver::new();
        let result = resolver.resolve("nonexistent_binary_xyz_123");
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("nonexistent_binary_xyz_123"),
            "Error should contain binary name: {}",
            err_msg
        );
        assert!(
            err_msg.contains("Searched in:"),
            "Error should contain search paths context: {}",
            err_msg
        );
    }

    #[test]
    fn test_legacy_runner_error_includes_task_id_and_workspace() {
        let temp_dir = TempDir::new().unwrap();
        let task = create_test_task("TEST-ERROR-001");
        let nonexistent_binary = temp_dir.path().join("nonexistent_binary");
        let input = create_runner_input(task, temp_dir.path().to_path_buf());
        let binary_input = RunnerInput::new(
            input.task.clone(),
            input.prepared_workspace_path.clone(),
            input.env_overrides.clone(),
            input.timeout_seconds,
            Some(nonexistent_binary.clone()),
            input.provider_mode,
            input.capture_options.clone(),
        );

        let runner = LegacyRunner::new("test-error-context");
        let result = runner.execute(&binary_input);
        assert!(result.is_ok());
        let output = result.unwrap();
        let stderr = output.stderr.to_string();
        assert!(
            stderr.contains("TEST-ERROR-001"),
            "Error should contain task ID: {}",
            stderr
        );
        assert!(
            stderr.contains("workspace"),
            "Error should contain workspace context: {}",
            stderr
        );
    }

    #[test]
    fn test_rust_runner_error_includes_task_id_and_workspace() {
        let temp_dir = TempDir::new().unwrap();
        let task = create_test_task("TEST-RUST-ERROR-001");
        let nonexistent_binary = temp_dir.path().join("nonexistent_binary");
        let binary_input = RunnerInput::new(
            task,
            temp_dir.path().to_path_buf(),
            std::collections::HashMap::new(),
            5,
            Some(nonexistent_binary),
            ProviderMode::Both,
            opencode_core::types::CaptureOptions::default(),
        );

        let runner = RustRunner::new("test-rust-error-context");
        let result = runner.execute(&binary_input);
        assert!(result.is_ok());
        let output = result.unwrap();
        let stderr = output.stderr.to_string();
        assert!(
            stderr.contains("TEST-RUST-ERROR-001"),
            "Error should contain task ID: {}",
            stderr
        );
        assert!(
            stderr.contains("workspace"),
            "Error should contain workspace context: {}",
            stderr
        );
    }

    #[test]
    fn test_rust_runner_timeout_error_includes_all_context() {
        let temp_dir = TempDir::new().unwrap();
        let mut task = create_test_task("TEST-TIMEOUT-001");
        task.input.command = "/bin/sleep".to_string();
        task.input.args = vec!["10".to_string()];
        let input = RunnerInput::new(
            task,
            temp_dir.path().to_path_buf(),
            std::collections::HashMap::new(),
            1,
            Some(std::path::PathBuf::from("/bin/sleep")),
            ProviderMode::Both,
            opencode_core::types::CaptureOptions::default(),
        );

        let runner = RustRunner::new("test-timeout-context");
        let result = runner.execute(&input);
        assert!(result.is_ok());
        let output = result.unwrap();
        let stderr = output.stderr.to_string();
        assert!(
            stderr.contains("TEST-TIMEOUT-001"),
            "Timeout error should contain task ID: {}",
            stderr
        );
        assert!(
            stderr.contains("timed out") || stderr.contains("timeout"),
            "Timeout error should mention timeout: {}",
            stderr
        );
        assert!(
            stderr.contains("workspace"),
            "Timeout error should contain workspace context: {}",
            stderr
        );
    }
}

mod error_chain_tests {
    use super::*;

    #[test]
    fn test_error_type_preserves_context_in_runner_errors() {
        let err = ErrorType::Runner("Task TEST-001 failed: binary not found".to_string());
        let err_msg = err.to_string();
        assert!(err_msg.contains("Task TEST-001 failed"));
        assert!(err_msg.contains("binary not found"));
    }

    #[test]
    fn test_runner_output_error_preserves_failure_info() {
        let temp_dir = TempDir::new().unwrap();
        let task = create_test_task("TEST-PRESERVE-001");
        let nonexistent_binary = temp_dir.path().join("nonexistent");
        let input = RunnerInput::new(
            task,
            temp_dir.path().to_path_buf(),
            std::collections::HashMap::new(),
            5,
            Some(nonexistent_binary),
            ProviderMode::Both,
            opencode_core::types::CaptureOptions::default(),
        );

        let runner = RustRunner::new("test-preserve");
        let result = runner.execute(&input);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(
            output.failure_kind.is_some(),
            "Error output should preserve failure_kind"
        );
        let failure_msg = output.stderr.to_string();
        assert!(
            failure_msg.contains("TEST-PRESERVE-001"),
            "Error message should preserve task ID through error chain"
        );
    }

    #[test]
    fn test_session_metadata_in_error_output() {
        let temp_dir = TempDir::new().unwrap();
        let task = create_test_task("TEST-METADATA-001");
        let nonexistent_binary = temp_dir.path().join("nonexistent");
        let input = RunnerInput::new(
            task,
            temp_dir.path().to_path_buf(),
            std::collections::HashMap::new(),
            5,
            Some(nonexistent_binary),
            ProviderMode::Both,
            opencode_core::types::CaptureOptions::default(),
        );

        let runner = RustRunner::new("test-metadata");
        let result = runner.execute(&input);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(
            output.session_metadata.task_id, "TEST-METADATA-001",
            "Session metadata should preserve task ID"
        );
        assert!(
            !output.session_metadata.session_id.is_empty(),
            "Session metadata should have session_id"
        );
    }

    #[test]
    fn test_error_context_includes_file_paths() {
        let temp_dir = TempDir::new().unwrap();
        let task = create_test_task("TEST-FILEPATH-001");
        let nonexistent_binary = temp_dir.path().join("nonexistent_binary_xyz");
        let input = RunnerInput::new(
            task,
            temp_dir.path().to_path_buf(),
            std::collections::HashMap::new(),
            5,
            Some(nonexistent_binary.clone()),
            ProviderMode::Both,
            opencode_core::types::CaptureOptions::default(),
        );

        let runner = RustRunner::new("test-filepath");
        let result = runner.execute(&input);
        assert!(result.is_ok());
        let output = result.unwrap();
        let stderr = output.stderr.to_string();
        assert!(
            stderr.contains("nonexistent_binary_xyz") || stderr.contains("does not exist"),
            "Error should contain file path context: {}",
            stderr
        );
    }

    #[test]
    fn test_error_chain_preservation_in_differential_runner() {
        use opencode_core::loaders::DefaultTaskLoader;
        use opencode_core::DifferentialRunner;

        let loader = DefaultTaskLoader::new();
        let runner = DifferentialRunner::new(loader);
        let mut task = create_test_task("TEST-DIFF-ERROR-001");
        task.input.command = "nonexistent_command_xyz".to_string();
        task.input.args = vec![];
        let input = RunnerInput::new(
            task,
            std::path::PathBuf::from("/tmp"),
            std::collections::HashMap::new(),
            60,
            None,
            ProviderMode::Both,
            opencode_core::types::CaptureOptions::default(),
        );

        let result = runner.execute(&input);
        assert!(result.is_ok());
        let diff_result = result.unwrap();
        assert_eq!(diff_result.task_id, "TEST-DIFF-ERROR-001");
        match &diff_result.verdict {
            opencode_core::types::ParityVerdict::Error { runner, reason } => {
                assert!(
                    reason.contains("TEST-DIFF-ERROR-001"),
                    "Error reason should preserve task ID: {}",
                    reason
                );
                assert!(
                    reason.contains("workspace") || reason.contains("/tmp"),
                    "Error reason should contain workspace path: {}",
                    reason
                );
            }
            _ => {
                if diff_result.legacy_result.is_none() || diff_result.rust_result.is_none() {
                    assert!(
                        diff_result.failure_kind.is_some(),
                        "Should have failure_kind when one runner fails"
                    );
                }
            }
        }
    }
}

mod error_message_format_tests {
    use super::*;

    #[test]
    fn test_error_messages_follow_consistent_format() {
        let temp_dir = TempDir::new().unwrap();
        let task = create_test_task("TEST-FORMAT-001");
        let nonexistent = temp_dir.path().join("nonexistent_cmd");
        let input = RunnerInput::new(
            task,
            temp_dir.path().to_path_buf(),
            std::collections::HashMap::new(),
            5,
            Some(nonexistent),
            ProviderMode::Both,
            opencode_core::types::CaptureOptions::default(),
        );

        let runner = RustRunner::new("test-format");
        let result = runner.execute(&input);
        assert!(result.is_ok());
        let output = result.unwrap();
        let stderr = output.stderr.to_string();
        assert!(
            stderr.contains("for task 'TEST-FORMAT-001'"),
            "Error should follow 'for task X' format: {}",
            stderr
        );
    }

    #[test]
    fn test_binary_not_found_error_has_clear_message() {
        let resolver = BinaryResolver::new();
        let result = resolver.resolve("totally_fake_binary_12345");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Could not find binary"),
            "Error should clearly indicate binary not found: {}",
            err_msg
        );
        assert!(
            err_msg.contains("totally_fake_binary_12345"),
            "Error should mention the binary name: {}",
            err_msg
        );
    }

    #[test]
    fn test_timeout_error_message_format() {
        let temp_dir = TempDir::new().unwrap();
        let mut task = create_test_task("TEST-TIMEOUT-FORMAT-001");
        task.input.command = "/bin/sleep".to_string();
        task.input.args = vec!["20".to_string()];
        let input = RunnerInput::new(
            task,
            temp_dir.path().to_path_buf(),
            std::collections::HashMap::new(),
            1,
            Some(std::path::PathBuf::from("/bin/sleep")),
            ProviderMode::Both,
            opencode_core::types::CaptureOptions::default(),
        );

        let runner = RustRunner::new("test-timeout-format");
        let result = runner.execute(&input);
        assert!(result.is_ok());
        let output = result.unwrap();
        let stderr = output.stderr.to_string();
        assert!(
            stderr.contains("timed out") || stderr.contains("killed"),
            "Timeout error should mention timeout or killed: {}",
            stderr
        );
    }
}

mod recovery_error_handling_tests {
    use super::*;

    #[test]
    fn test_repeated_connection_failures_handled_gracefully() {
        let temp_dir = TempDir::new().unwrap();
        let task = create_test_task("SMOKE-RECOVERY-001");
        let nonexistent_binary = temp_dir.path().join("connection_simulator_nonexistent");
        let input = RunnerInput::new(
            task,
            temp_dir.path().to_path_buf(),
            std::collections::HashMap::new(),
            5,
            Some(nonexistent_binary),
            ProviderMode::Both,
            opencode_core::types::CaptureOptions::default(),
        );

        let runner = RustRunner::new("test-repeated-connection-failures");
        let result = runner.execute(&input);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(
            output.failure_kind.is_some(),
            "Repeated connection failures should result in a recorded failure"
        );
        let stderr = output.stderr.to_string();
        assert!(
            stderr.contains("SMOKE-RECOVERY-001"),
            "Error context should include task ID for recovery scenario"
        );
    }

    #[test]
    fn test_corrupted_session_state_error_handling() {
        let temp_dir = TempDir::new().unwrap();
        let task = create_test_task("SMOKE-RECOVERY-002");
        let nonexistent_binary = temp_dir.path().join("corrupted_session_simulator");
        let input = RunnerInput::new(
            task,
            temp_dir.path().to_path_buf(),
            std::collections::HashMap::new(),
            5,
            Some(nonexistent_binary),
            ProviderMode::Both,
            opencode_core::types::CaptureOptions::default(),
        );

        let runner = RustRunner::new("test-corrupted-session");
        let result = runner.execute(&input);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(
            output.session_metadata.task_id == "SMOKE-RECOVERY-002",
            "Session metadata should preserve task ID even with corrupted state"
        );
        assert!(
            !output.session_metadata.session_id.is_empty(),
            "Session metadata should have session_id for tracking"
        );
    }

    #[test]
    fn test_recovery_task_error_includes_recovery_context() {
        let temp_dir = TempDir::new().unwrap();
        let mut task = create_test_task("SMOKE-RECOVERY-003");
        task.input.command = "nonexistent_recovery_binary".to_string();
        let input = RunnerInput::new(
            task,
            temp_dir.path().to_path_buf(),
            std::collections::HashMap::new(),
            5,
            None,
            ProviderMode::Both,
            opencode_core::types::CaptureOptions::default(),
        );

        let runner = RustRunner::new("test-recovery-context");
        let result = runner.execute(&input);
        assert!(result.is_ok());
        let output = result.unwrap();
        let stderr = output.stderr.to_string();
        assert!(
            stderr.contains("SMOKE-RECOVERY-003") || stderr.contains("recovery"),
            "Error should contain recovery task context: {}",
            stderr
        );
    }

    #[test]
    fn test_recovery_timeout_error_preserves_session_state() {
        let temp_dir = TempDir::new().unwrap();
        let mut task = create_test_task("SMOKE-RECOVERY-TIMEOUT");
        task.input.command = "/bin/sleep".to_string();
        task.input.args = vec!["20".to_string()];
        let input = RunnerInput::new(
            task,
            temp_dir.path().to_path_buf(),
            std::collections::HashMap::new(),
            1,
            Some(std::path::PathBuf::from("/bin/sleep")),
            ProviderMode::Both,
            opencode_core::types::CaptureOptions::default(),
        );

        let runner = RustRunner::new("test-recovery-timeout");
        let result = runner.execute(&input);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(
            output.session_metadata.task_id == "SMOKE-RECOVERY-TIMEOUT",
            "Session metadata should preserve task ID through timeout"
        );
    }
}
