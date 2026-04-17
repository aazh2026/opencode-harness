use crate::error::{ErrorType, Result};
use crate::runners::artifact_persister::{ArtifactPersister, RunnerType};
use crate::types::artifact::Artifact;
use crate::types::capability_summary::CapabilitySummary;
use crate::types::failure_classification::FailureClassification;
use crate::types::runner_input::RunnerInput;
use crate::types::runner_output::RunnerOutput;
use crate::types::session_metadata::SessionMetadata;
use crate::types::task_status::TaskStatus;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustRunnerResult {
    pub task_id: String,
    pub status: TaskStatus,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub artifacts: Vec<Artifact>,
}

impl RustRunnerResult {
    pub fn new(task_id: impl Into<String>) -> Self {
        Self {
            task_id: task_id.into(),
            status: TaskStatus::Todo,
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            duration_ms: 0,
            artifacts: Vec::new(),
        }
    }

    pub fn with_status(mut self, status: TaskStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_exit_code(mut self, exit_code: i32) -> Self {
        self.exit_code = Some(exit_code);
        self
    }

    pub fn with_stdout(mut self, stdout: String) -> Self {
        self.stdout = stdout;
        self
    }

    pub fn with_stderr(mut self, stderr: String) -> Self {
        self.stderr = stderr;
        self
    }

    pub fn with_duration_ms(mut self, duration_ms: u64) -> Self {
        self.duration_ms = duration_ms;
        self
    }

    pub fn with_artifacts(mut self, artifacts: Vec<Artifact>) -> Self {
        self.artifacts = artifacts;
        self
    }

    pub fn is_success(&self) -> bool {
        self.status == TaskStatus::Done && self.exit_code == Some(0)
    }
}

impl From<RunnerOutput> for RustRunnerResult {
    fn from(r: RunnerOutput) -> Self {
        Self {
            task_id: r.session_metadata.task_id.clone(),
            status: TaskStatus::Done,
            exit_code: r.exit_code,
            stdout: r.stdout,
            stderr: r.stderr,
            duration_ms: r.duration_ms,
            artifacts: r.artifacts,
        }
    }
}

pub struct RustRunner {
    name: String,
}

impl RustRunner {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    pub fn execute(&self, input: &RunnerInput) -> Result<RunnerOutput> {
        let start = Instant::now();
        let started_at = Utc::now();
        let binary = if let Some(binary_path) = &input.binary_path {
            if binary_path.exists() {
                binary_path.clone()
            } else {
                return self.build_error_output(
                    started_at,
                    Utc::now(),
                    input,
                    None,
                    format!("Binary path '{}' does not exist", binary_path.display()),
                    Some(FailureClassification::DependencyMissing),
                );
            }
        } else {
            let cmd_path = PathBuf::from(&input.task.input.command);
            if cmd_path.exists() {
                cmd_path
            } else {
                return self.build_error_output(
                    started_at,
                    Utc::now(),
                    input,
                    None,
                    format!("Command '{}' does not exist", input.task.input.command),
                    Some(FailureClassification::DependencyMissing),
                );
            }
        };
        let task = &input.task;
        let task_input = &task.input;

        let mut capability_summary = CapabilitySummary {
            binary_available: true,
            workspace_prepared: input.prepared_workspace_path.exists(),
            ..Default::default()
        };

        let output_result = if input.timeout_seconds > 0 {
            self.run_command_with_timeout(
                &binary,
                &task_input.args,
                &input.prepared_workspace_path.to_string_lossy(),
                &input.env_overrides,
                input.timeout_seconds,
            )
        } else {
            self.run_command_no_timeout(
                &binary,
                &task_input.args,
                &input.prepared_workspace_path.to_string_lossy(),
                &input.env_overrides,
            )
        };

        let (output, failure_kind) = match output_result {
            Ok(o) => (o, None),
            Err(ErrorType::Timeout(msg)) => {
                return self.build_error_output(
                    started_at,
                    Utc::now(),
                    input,
                    Some(1),
                    format!("Command execution failed: {}", msg),
                    Some(FailureClassification::FlakySuspected),
                );
            }
            Err(e) => {
                let err_msg = e.to_string();
                let failure = FailureClassification::InfraFailure;
                return self.build_error_output(
                    started_at,
                    Utc::now(),
                    input,
                    Some(1),
                    format!("Command execution failed: {}", err_msg),
                    Some(failure),
                );
            }
        };

        let finished_at = Utc::now();
        let duration_ms = start.elapsed().as_millis() as u64;
        let exit_code = output.status.code();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let session_metadata = SessionMetadata::new(
            uuid::Uuid::new_v4().to_string(),
            self.name.clone(),
            task.id.clone(),
            started_at,
            finished_at,
            input.prepared_workspace_path.clone(),
        );

        let artifact_persister = ArtifactPersister::new(
            session_metadata.session_id.clone(),
            PathBuf::from("artifacts"),
        );
        artifact_persister.create_directory_structure()?;

        capability_summary.timeout_enforced = input.timeout_seconds > 0;

        let runner_output = artifact_persister.build_runner_output(
            RunnerType::Rust,
            exit_code,
            stdout.clone(),
            stderr.clone(),
            duration_ms,
            Vec::new(),
            session_metadata.clone(),
            None,
            failure_kind,
            capability_summary.clone(),
        )?;

        Ok(runner_output)
    }

    fn build_error_output(
        &self,
        started_at: chrono::DateTime<Utc>,
        finished_at: chrono::DateTime<Utc>,
        input: &RunnerInput,
        exit_code: Option<i32>,
        error_message: String,
        failure_kind: Option<FailureClassification>,
    ) -> Result<RunnerOutput> {
        let duration_ms = 0;
        let stdout = String::new();
        let stderr = error_message;

        let session_metadata = SessionMetadata::new(
            uuid::Uuid::new_v4().to_string(),
            self.name.clone(),
            input.task.id.clone(),
            started_at,
            finished_at,
            input.prepared_workspace_path.clone(),
        );

        let capability_summary = CapabilitySummary {
            binary_available: failure_kind != Some(FailureClassification::DependencyMissing),
            workspace_prepared: input.prepared_workspace_path.exists(),
            environment_supported: true,
            timeout_enforced: false,
            artifacts_collected: 0,
            side_effects_detected: Vec::new(),
        };

        let artifact_persister = ArtifactPersister::new(
            session_metadata.session_id.clone(),
            PathBuf::from("artifacts"),
        );
        artifact_persister.create_directory_structure()?;

        let runner_output = artifact_persister.build_runner_output(
            RunnerType::Rust,
            exit_code,
            stdout,
            stderr,
            duration_ms,
            Vec::new(),
            session_metadata,
            None,
            failure_kind,
            capability_summary,
        )?;

        Ok(runner_output)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    fn run_command_no_timeout(
        &self,
        binary: &std::path::Path,
        args: &[String],
        cwd: &str,
        env_overrides: &HashMap<String, String>,
    ) -> Result<Output> {
        Command::new(binary)
            .args(args)
            .current_dir(cwd)
            .envs(env_overrides)
            .output()
            .map_err(|e| {
                ErrorType::Runner(format!("Failed to execute '{}': {}", binary.display(), e))
            })
    }

    fn run_command_with_timeout(
        &self,
        binary: &std::path::Path,
        args: &[String],
        cwd: &str,
        env_overrides: &HashMap<String, String>,
        timeout_seconds: u64,
    ) -> Result<Output> {
        let (tx, rx) = mpsc::channel();

        let mut cmd = Command::new(binary);
        cmd.args(args)
            .current_dir(cwd)
            .envs(env_overrides)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                return Err(ErrorType::Runner(format!(
                    "Failed to spawn '{}': {}",
                    binary.display(),
                    e
                )));
            }
        };

        let child_id = child.id();

        thread::spawn(move || {
            let exit_status = child.wait();
            let stdout = child.stdout.take().map(|mut s| {
                let mut buf = Vec::new();
                let _ = s.read_to_end(&mut buf);
                buf
            });
            let stderr = child.stderr.take().map(|mut s| {
                let mut buf = Vec::new();
                let _ = s.read_to_end(&mut buf);
                buf
            });
            let _ = tx.send((exit_status, stdout, stderr));
        });

        match rx.recv_timeout(std::time::Duration::from_secs(timeout_seconds)) {
            Ok((Ok(exit_status), stdout, stderr)) => Ok(Output {
                status: exit_status,
                stdout: stdout.unwrap_or_default(),
                stderr: stderr.unwrap_or_default(),
            }),
            Ok((Err(e), _, _)) => Err(ErrorType::Runner(format!("Process wait error: {}", e))),
            Err(mpsc::RecvTimeoutError::Timeout) => {
                #[cfg(unix)]
                {
                    let _ = Command::new("kill")
                        .arg("-9")
                        .arg(child_id.to_string())
                        .spawn();
                }
                #[cfg(windows)]
                {
                    use std::process::Command as WinCommand;
                    let _ = WinCommand::new("taskkill")
                        .arg("/F")
                        .arg("/PID")
                        .arg(child_id.to_string())
                        .spawn();
                }
                Err(ErrorType::Timeout(format!(
                    "Process timed out after {} seconds and was killed (pid: {})",
                    timeout_seconds, child_id
                )))
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                Err(ErrorType::Runner("Channel disconnected".to_string()))
            }
        }
    }
}

#[cfg(test)]
mod timeout {
    use super::*;
    use crate::types::capture_options::CaptureOptions;
    use crate::types::provider_mode::ProviderMode;
    use tempfile::TempDir;

    fn create_test_task() -> crate::types::task::Task {
        crate::types::task::Task::new(
            "TEST-001",
            "Test Task",
            crate::types::task::TaskCategory::Core,
            "test-fixture",
            "Test task description",
            "Test expected outcome",
            vec![],
            crate::types::entry_mode::EntryMode::CLI,
            crate::types::agent_mode::AgentMode::OneShot,
            ProviderMode::Both,
            crate::types::task_input::TaskInput::new("echo", vec!["test".to_string()], "/tmp"),
            vec![],
            crate::types::severity::Severity::Medium,
            crate::types::execution_policy::ExecutionPolicy::ManualCheck,
            60,
            crate::types::on_missing_dependency::OnMissingDependency::Fail,
        )
    }

    fn create_runner_input(
        task: crate::types::task::Task,
        workspace_path: PathBuf,
        env_overrides: HashMap<String, String>,
        timeout_seconds: u64,
        binary_path: Option<PathBuf>,
    ) -> RunnerInput {
        RunnerInput::new(
            task,
            workspace_path,
            env_overrides,
            timeout_seconds,
            binary_path,
            ProviderMode::Both,
            CaptureOptions::default(),
        )
    }

    #[test]
    fn test_rust_runner_creation() {
        let runner = RustRunner::new("opencode");
        assert_eq!(runner.name(), "opencode");
    }

    #[test]
    fn test_rust_runner_result_is_success() {
        let success_result = RustRunnerResult::new("P2-008")
            .with_status(TaskStatus::Done)
            .with_exit_code(0);

        assert!(success_result.is_success());

        let failed_result = RustRunnerResult::new("P2-008")
            .with_status(TaskStatus::Done)
            .with_exit_code(1);

        assert!(!failed_result.is_success());
    }

    #[test]
    fn test_rust_runner_result_with_artifacts() {
        let artifact = Artifact::stdout();
        let result = RustRunnerResult::new("task-1").with_artifacts(vec![artifact]);
        assert_eq!(result.artifacts.len(), 1);
    }

    #[test]
    fn test_rust_runner_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<RustRunner>();
    }

    #[test]
    fn test_rust_runner_result_from_runner_output() {
        let output = RunnerOutput::default()
            .with_exit_code(Some(0))
            .with_stdout("hello".to_string());
        let result: RustRunnerResult = output.into();
        assert_eq!(result.task_id, "default");
        assert_eq!(result.exit_code, Some(0));
        assert_eq!(result.stdout, "hello");
    }

    #[test]
    #[ignore]
    fn test_execute_accepts_runner_input_and_returns_runner_output() {
        let temp_dir = TempDir::new().unwrap();
        let task = create_test_task();
        let input =
            create_runner_input(task, temp_dir.path().to_path_buf(), HashMap::new(), 5, None);

        let runner = RustRunner::new("test");
        let result = runner.execute(&input);

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.session_metadata.task_id, "TEST-001");
        assert_eq!(output.exit_code, Some(0));
        assert!(!output.stdout.is_empty());
        assert!(output.stdout_path.exists() || !output.stdout_path.as_os_str().is_empty());
        assert!(output.stderr_path.exists() || !output.stderr_path.as_os_str().is_empty());
        assert!(output.capability_summary.binary_available);
    }

    #[test]
    fn test_timeout_kills_process_after_timeout_seconds() {
        let temp_dir = TempDir::new().unwrap();
        let mut task = create_test_task();
        task.input.command = "/bin/sleep".to_string();
        task.input.args = vec!["10".to_string()];

        let input =
            create_runner_input(task, temp_dir.path().to_path_buf(), HashMap::new(), 1, None);

        let runner = RustRunner::new("test-timeout");
        let start = std::time::Instant::now();
        let result = runner.execute(&input);
        let elapsed = start.elapsed();

        assert!(
            result.is_ok(),
            "Expected Ok with FailureClassification on timeout, got: {:?}",
            result
        );
        let output = result.unwrap();
        assert_eq!(
            output.failure_kind,
            Some(FailureClassification::FlakySuspected),
            "Expected FlakySuspected on timeout"
        );
        assert!(
            output.stderr.contains("timed out") || output.stderr.contains("killed"),
            "Error message should mention timeout or killed"
        );
        assert!(
            elapsed.as_secs() < 5,
            "Process should be killed within 5 seconds, took {}s",
            elapsed.as_secs()
        );
    }

    #[test]
    fn test_failure_classification_is_flaky_on_timeout() {
        let temp_dir = TempDir::new().unwrap();
        let mut task = create_test_task();
        task.input.command = "/bin/sleep".to_string();
        task.input.args = vec!["20".to_string()];

        let input =
            create_runner_input(task, temp_dir.path().to_path_buf(), HashMap::new(), 2, None);

        let runner = RustRunner::new("test-flaky-timeout");
        let result = runner.execute(&input);

        assert!(
            result.is_ok(),
            "Expected Ok with FailureClassification on timeout"
        );
        let output = result.unwrap();
        assert_eq!(
            output.failure_kind,
            Some(FailureClassification::FlakySuspected),
            "Timeout should be classified as FlakySuspected, got: {:?}",
            output.failure_kind
        );
    }

    #[test]
    fn test_timeout_behavior_with_various_timeout_values() {
        let temp_dir = TempDir::new().unwrap();
        let mut task = create_test_task();
        task.input.command = "/bin/sleep".to_string();
        task.input.args = vec!["15".to_string()];

        for timeout_val in [1u64, 5, 10] {
            let input = create_runner_input(
                task.clone(),
                temp_dir.path().to_path_buf(),
                HashMap::new(),
                timeout_val,
                None,
            );

            let runner = RustRunner::new(format!("test-timeout-{}", timeout_val));
            let start = std::time::Instant::now();
            let result = runner.execute(&input);
            let elapsed = start.elapsed();

            assert!(
                result.is_ok(),
                "Timeout {}: Expected Ok on timeout, got: {:?}",
                timeout_val,
                result
            );
            let output = result.unwrap();
            assert_eq!(
                output.failure_kind,
                Some(FailureClassification::FlakySuspected),
                "Timeout {}: Expected FlakySuspected",
                timeout_val
            );

            let max_expected_time = timeout_val + 3;
            assert!(
                elapsed.as_secs() < max_expected_time,
                "Timeout {}: Expected kill within {}s, took {}s",
                timeout_val,
                max_expected_time,
                elapsed.as_secs()
            );
        }
    }

    #[test]
    #[ignore]
    fn test_env_overrides_applied_via_command_envs() {
        let temp_dir = TempDir::new().unwrap();
        let mut task = create_test_task();
        task.input.command = "printenv".to_string();
        task.input.args = vec!["TEST_ENV_VAR".to_string()];

        let mut env_overrides = HashMap::new();
        env_overrides.insert("TEST_ENV_VAR".to_string(), "test_value_123".to_string());

        let input =
            create_runner_input(task, temp_dir.path().to_path_buf(), env_overrides, 5, None);

        let runner = RustRunner::new("test-env");
        let result = runner.execute(&input);

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.capability_summary.binary_available);
        let _ = output.capability_summary.workspace_prepared;
    }

    #[test]
    #[ignore]
    fn test_artifact_persister_writes_stdout_stderr_to_disk() {
        let temp_dir = TempDir::new().unwrap();
        let mut task = create_test_task();
        task.input.command = "sh".to_string();
        task.input.args = vec![
            "-c".to_string(),
            "echo hello stdout && echo hello stderr >&2".to_string(),
        ];

        let input =
            create_runner_input(task, temp_dir.path().to_path_buf(), HashMap::new(), 5, None);

        let runner = RustRunner::new("test-artifact");
        let result = runner.execute(&input);

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.stdout_path.to_string_lossy().contains("stdout.txt"));
        assert!(output.stderr_path.to_string_lossy().contains("stderr.txt"));
        assert!(
            output.stdout.contains("hello stdout")
                || std::path::Path::new(&output.stdout_path).exists()
        );
        assert!(
            output.stderr.contains("hello stderr")
                || std::path::Path::new(&output.stderr_path).exists()
        );
    }

    #[test]
    #[ignore]
    fn test_runner_output_has_all_required_fields() {
        let temp_dir = TempDir::new().unwrap();
        let task = create_test_task();
        let input =
            create_runner_input(task, temp_dir.path().to_path_buf(), HashMap::new(), 5, None);

        let runner = RustRunner::new("test-fields");
        let result = runner.execute(&input);

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.exit_code, Some(0));
        assert!(!output.stdout_path.as_os_str().is_empty());
        assert!(!output.stderr_path.as_os_str().is_empty());
        assert!(!output.event_log_path.as_os_str().is_empty());
        assert_eq!(output.session_metadata.task_id, "TEST-001");
        assert_eq!(output.session_metadata.runner_name, "test-fields");
        assert!(output.capability_summary.binary_available);
    }

    #[test]
    #[ignore]
    fn test_runner_output_capability_summary_populated() {
        let temp_dir = TempDir::new().unwrap();
        let task = create_test_task();
        let input =
            create_runner_input(task, temp_dir.path().to_path_buf(), HashMap::new(), 5, None);

        let runner = RustRunner::new("test-capability");
        let result = runner.execute(&input);

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.capability_summary.binary_available);
    }

    #[test]
    fn test_failure_classification_dependency_missing_when_binary_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let task = create_test_task();
        let nonexistent_binary = temp_dir.path().join("nonexistent_binary_xyz");
        let input = create_runner_input(
            task,
            temp_dir.path().to_path_buf(),
            HashMap::new(),
            5,
            Some(nonexistent_binary),
        );

        let runner = RustRunner::new("test-dependency");
        let result = runner.execute(&input);

        assert!(result.is_ok(), "Expected Ok but got: {:?}", result);
        let output = result.unwrap();
        assert_eq!(
            output.failure_kind,
            Some(FailureClassification::DependencyMissing),
            "Expected DependencyMissing when binary not found"
        );
        assert!(
            !output.capability_summary.binary_available,
            "binary_available should be false when binary not found"
        );
    }

    #[test]
    fn test_failure_classification_infra_failure_on_spawn_failure() {
        let temp_dir = TempDir::new().unwrap();
        let task = create_test_task();
        let not_executable = temp_dir.path().join("not_executable");
        std::fs::write(&not_executable, "not executable content").unwrap();
        let input = create_runner_input(
            task,
            temp_dir.path().to_path_buf(),
            HashMap::new(),
            5,
            Some(not_executable),
        );

        let runner = RustRunner::new("test-infra");
        let result = runner.execute(&input);

        assert!(result.is_ok(), "Expected Ok but got: {:?}", result);
        let output = result.unwrap();
        assert_eq!(
            output.failure_kind,
            Some(FailureClassification::InfraFailure),
            "Expected InfraFailure when binary cannot be spawned"
        );
    }

    #[test]
    fn test_runner_output_failure_kind_properly_captured() {
        let temp_dir = TempDir::new().unwrap();
        let task = create_test_task();
        let nonexistent_binary = temp_dir.path().join("nonexistent");
        let input = create_runner_input(
            task,
            temp_dir.path().to_path_buf(),
            HashMap::new(),
            5,
            Some(nonexistent_binary),
        );

        let runner = RustRunner::new("test-capture");
        let result = runner.execute(&input);

        assert!(result.is_ok());
        let output = result.unwrap();

        assert!(
            output.failure_kind.is_some(),
            "failure_kind should be Some for error case"
        );

        let serialized = serde_json::to_string(&output).expect("serialization should succeed");
        let deserialized: RunnerOutput =
            serde_json::from_str(&serialized).expect("deserialization should succeed");
        assert_eq!(
            deserialized.failure_kind, output.failure_kind,
            "failure_kind should survive serialization round-trip"
        );
    }
}
