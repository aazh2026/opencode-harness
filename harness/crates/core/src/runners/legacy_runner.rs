use crate::error::{ErrorType, Result};
use crate::runners::artifact_persister::{ArtifactPersister, RunnerType};
use crate::runners::binary_resolver::BinaryResolver;
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
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyRunnerResult {
    pub task_id: String,
    pub status: TaskStatus,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub artifacts: Vec<crate::types::artifact::Artifact>,
}

impl LegacyRunnerResult {
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

    pub fn with_artifacts(mut self, artifacts: Vec<crate::types::artifact::Artifact>) -> Self {
        self.artifacts = artifacts;
        self
    }

    pub fn is_success(&self) -> bool {
        self.status == TaskStatus::Done && self.exit_code == Some(0)
    }
}

pub struct LegacyRunner {
    name: String,
}

impl LegacyRunner {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    pub fn execute(&self, input: &RunnerInput) -> Result<RunnerOutput> {
        let start = Instant::now();
        let started_at = Utc::now();
        let resolver = BinaryResolver::new();

        let binary = match resolver.resolve_opencode_with_override(input.binary_path.as_ref()) {
            Ok(path) => path,
            Err(e) => {
                let err_msg = e.to_string();
                let failure_kind = if err_msg.contains("does not exist")
                    || err_msg.contains("Could not find binary")
                {
                    Some(FailureClassification::DependencyMissing)
                } else {
                    Some(FailureClassification::InfraFailure)
                };
                return self.build_error_output(
                    started_at,
                    Utc::now(),
                    input,
                    None,
                    format!(
                        "Binary resolution failed for task '{}' (workspace: {}): {}",
                        input.task.id,
                        input.prepared_workspace_path.display(),
                        err_msg
                    ),
                    failure_kind,
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
            Err(e) => {
                let err_msg = e.to_string();
                let failure = if err_msg.contains("timed out") {
                    FailureClassification::FlakySuspected
                } else {
                    FailureClassification::InfraFailure
                };
                return self.build_error_output(
                    started_at,
                    Utc::now(),
                    input,
                    Some(1),
                    format!(
                        "Command execution failed for task '{}' (binary: {}, workspace: {}): {}",
                        input.task.id,
                        binary.display(),
                        input.prepared_workspace_path.display(),
                        err_msg
                    ),
                    Some(failure),
                );
            }
        };

        let finished_at = Utc::now();
        let duration_ms = start.elapsed().as_millis() as u64;
        let exit_code = output.status.code();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        info!(
            "LegacyRunner executed task={} binary={} args={:?} cwd={} exit_code={:?} stdout_len={} stderr_len={}",
            input.task.id,
            binary.display(),
            task_input.args,
            input.prepared_workspace_path.display(),
            exit_code,
            stdout.len(),
            stderr.len()
        );

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
            RunnerType::Legacy,
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
            RunnerType::Legacy,
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
                Err(ErrorType::Runner(format!(
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
mod tests {
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
    fn test_legacy_runner_creation() {
        let runner = LegacyRunner::new("legacy");
        assert_eq!(runner.name(), "legacy");
    }

    #[test]
    fn test_legacy_runner_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<LegacyRunner>();
    }

    #[test]
    fn test_legacy_runner_result_creation() {
        let result = LegacyRunnerResult::new("task-1")
            .with_status(TaskStatus::Todo)
            .with_exit_code(0);
        assert_eq!(result.task_id, "task-1");
        assert_eq!(result.status, TaskStatus::Todo);
        assert_eq!(result.exit_code, Some(0));
        assert!(result.artifacts.is_empty());
    }

    #[test]
    fn test_legacy_runner_result_with_artifacts() {
        let artifact = crate::types::artifact::Artifact::stdout();
        let result = LegacyRunnerResult::new("task-1").with_artifacts(vec![artifact]);
        assert_eq!(result.artifacts.len(), 1);
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

        let runner = LegacyRunner::new("test-dependency");
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
    fn test_failure_classification_infra_failure_on_binary_spawn_failure() {
        let temp_dir = TempDir::new().unwrap();
        let task = create_test_task();
        let invalid_binary = temp_dir
            .path()
            .join("invalid_binary_that_cannot_be_executed");
        #[cfg(unix)]
        {
            std::fs::write(&invalid_binary, "not an executable").unwrap();
        }
        #[cfg(windows)]
        {
            std::fs::write(&invalid_binary, "not executable").unwrap();
        }
        let input = create_runner_input(
            task,
            temp_dir.path().to_path_buf(),
            HashMap::new(),
            5,
            Some(invalid_binary),
        );

        let runner = LegacyRunner::new("test-infra");
        let result = runner.execute(&input);

        assert!(result.is_ok(), "Expected Ok but got: {:?}", result);
        let output = result.unwrap();
        assert_eq!(
            output.failure_kind,
            Some(FailureClassification::InfraFailure),
            "Expected InfraFailure when binary cannot be executed"
        );
    }

    #[test]
    fn test_runner_output_captures_all_failure_kinds() {
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

        let runner = LegacyRunner::new("test-capture");
        let result = runner.execute(&input);

        assert!(result.is_ok());
        let output = result.unwrap();

        assert!(
            output.failure_kind.is_some(),
            "failure_kind should be Some for error case"
        );
        let failure = output.failure_kind.unwrap();
        match failure {
            FailureClassification::DependencyMissing => {}
            FailureClassification::EnvironmentNotSupported => {}
            FailureClassification::InfraFailure => {}
            FailureClassification::ImplementationFailure => {}
            FailureClassification::FlakySuspected => {}
        }
    }

    #[test]
    fn test_timeout_enforcement_kills_process_after_timeout() {
        let temp_dir = TempDir::new().unwrap();
        let mut task = create_test_task();
        task.input.command = "/bin/sleep".to_string();
        task.input.args = vec!["10".to_string()];

        let input = create_runner_input(
            task,
            temp_dir.path().to_path_buf(),
            HashMap::new(),
            1,
            Some(PathBuf::from("/bin/sleep")),
        );

        let runner = LegacyRunner::new("test-timeout");
        let start = std::time::Instant::now();
        let result = runner.execute(&input);
        let elapsed = start.elapsed();

        assert!(
            result.is_ok(),
            "Expected Ok with failure_kind on timeout but got: {:?}",
            result
        );
        let output = result.unwrap();
        assert!(
            output.stderr.contains("timed out") || output.stderr.contains("killed"),
            "Error message should mention timeout or killed: {}",
            output.stderr
        );
        assert_eq!(
            output.failure_kind,
            Some(FailureClassification::FlakySuspected),
            "Expected FlakySuspected on timeout"
        );
        assert!(
            elapsed.as_secs() < 5,
            "Process should be killed quickly after timeout, elapsed: {}s",
            elapsed.as_secs()
        );
    }

    #[test]
    fn test_timeout_failure_classification_is_flaky_on_timeout() {
        let temp_dir = TempDir::new().unwrap();
        let mut task = create_test_task();
        task.input.command = "/bin/sleep".to_string();
        task.input.args = vec!["10".to_string()];

        let input = create_runner_input(
            task,
            temp_dir.path().to_path_buf(),
            HashMap::new(),
            1,
            Some(PathBuf::from("/bin/sleep")),
        );

        let runner = LegacyRunner::new("test-timeout-classification");
        let result = runner.execute(&input);

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(
            output.failure_kind,
            Some(FailureClassification::FlakySuspected),
            "Expected FlakySuspected on timeout"
        );
    }

    #[test]
    fn test_timeout_with_various_timeout_values() {
        let temp_dir = TempDir::new().unwrap();
        let mut task = create_test_task();
        task.input.command = "/bin/sleep".to_string();
        task.input.args = vec!["3".to_string()];

        for timeout_value in [1u64, 5, 10] {
            let input = create_runner_input(
                task.clone(),
                temp_dir.path().to_path_buf(),
                HashMap::new(),
                timeout_value,
                Some(PathBuf::from("/bin/sleep")),
            );

            let runner = LegacyRunner::new(&format!("test-timeout-{}", timeout_value));
            let start = std::time::Instant::now();
            let result = runner.execute(&input);
            let elapsed = start.elapsed();

            if timeout_value <= 3 {
                assert!(
                    result.is_ok(),
                    "Expected Ok with failure_kind for timeout={} but got: {:?}",
                    timeout_value,
                    result
                );
                let output = result.unwrap();
                assert!(
                    output.stderr.contains("timed out") || output.stderr.contains("killed"),
                    "Error should mention timeout: {}",
                    output.stderr
                );
                assert_eq!(
                    output.failure_kind,
                    Some(FailureClassification::FlakySuspected),
                    "Expected FlakySuspected on timeout"
                );
            } else {
                assert!(
                    result.is_ok(),
                    "Should succeed with timeout={} but got: {:?}",
                    timeout_value,
                    result
                );
                let output = result.unwrap();
                assert!(
                    output.failure_kind.is_none()
                        || output.failure_kind == Some(FailureClassification::FlakySuspected),
                    "Should not have failure_kind for successful run"
                );
            }
            assert!(
                elapsed.as_secs() < timeout_value as u64 + 2,
                "Elapsed {}s should be less than timeout+2s for timeout={}",
                elapsed.as_secs(),
                timeout_value
            );
        }
    }

    #[test]
    fn test_env_overrides_applied_via_command_envs() {
        let temp_dir = TempDir::new().unwrap();
        let mut task = create_test_task();
        task.input.command = "/usr/bin/printenv".to_string();
        task.input.args = vec!["TEST_LEGACY_ENV_VAR".to_string()];

        let mut env_overrides = HashMap::new();
        env_overrides.insert(
            "TEST_LEGACY_ENV_VAR".to_string(),
            "legacy_test_value_456".to_string(),
        );

        let input = create_runner_input(
            task,
            temp_dir.path().to_path_buf(),
            env_overrides,
            5,
            Some(PathBuf::from("/usr/bin/printenv")),
        );

        let runner = LegacyRunner::new("test-env-overrides");
        let result = runner.execute(&input);

        assert!(result.is_ok(), "Expected Ok but got: {:?}", result);
        let output = result.unwrap();
        assert!(
            output.capability_summary.binary_available,
            "binary_available should be true"
        );
        assert!(
            output.stdout.contains("legacy_test_value_456"),
            "Expected stdout to contain 'legacy_test_value_456' but got: {}",
            output.stdout
        );
    }

    #[test]
    fn test_env_overrides_passed_to_subprocess() {
        let temp_dir = TempDir::new().unwrap();
        let mut task = create_test_task();
        task.input.command = "/usr/bin/env".to_string();
        task.input.args = vec![];

        let mut env_overrides = HashMap::new();
        env_overrides.insert("CUSTOM_VAR_ONE".to_string(), "value_one".to_string());
        env_overrides.insert("CUSTOM_VAR_TWO".to_string(), "value_two".to_string());

        let input = create_runner_input(
            task,
            temp_dir.path().to_path_buf(),
            env_overrides,
            5,
            Some(PathBuf::from("/usr/bin/env")),
        );

        let runner = LegacyRunner::new("test-env-passing");
        let result = runner.execute(&input);

        assert!(result.is_ok(), "Expected Ok but got: {:?}", result);
        let output = result.unwrap();
        assert!(
            output.stdout.contains("CUSTOM_VAR_ONE=value_one"),
            "Expected stdout to contain 'CUSTOM_VAR_ONE=value_one' but got: {}",
            output.stdout
        );
        assert!(
            output.stdout.contains("CUSTOM_VAR_TWO=value_two"),
            "Expected stdout to contain 'CUSTOM_VAR_TWO=value_two' but got: {}",
            output.stdout
        );
    }

    #[test]
    fn test_env_overrides_with_timeout() {
        let temp_dir = TempDir::new().unwrap();
        let mut task = create_test_task();
        task.input.command = "/usr/bin/printenv".to_string();
        task.input.args = vec!["TIMEOUT_TEST_VAR".to_string()];

        let mut env_overrides = HashMap::new();
        env_overrides.insert(
            "TIMEOUT_TEST_VAR".to_string(),
            "timeout_env_value".to_string(),
        );

        let input = create_runner_input(
            task,
            temp_dir.path().to_path_buf(),
            env_overrides,
            10,
            Some(PathBuf::from("/usr/bin/printenv")),
        );

        let runner = LegacyRunner::new("test-env-with-timeout");
        let result = runner.execute(&input);

        assert!(result.is_ok(), "Expected Ok but got: {:?}", result);
        let output = result.unwrap();
        assert!(
            output.capability_summary.timeout_enforced,
            "timeout_enforced should be true"
        );
        assert!(
            output.stdout.contains("timeout_env_value"),
            "Expected stdout to contain 'timeout_env_value' but got: {}",
            output.stdout
        );
    }

    #[test]
    fn test_env_overrides_multiple_variables() {
        let temp_dir = TempDir::new().unwrap();
        let mut task = create_test_task();
        task.input.command = "/usr/bin/env".to_string();
        task.input.args = vec![];

        let mut env_overrides = HashMap::new();
        env_overrides.insert("VAR_A".to_string(), "A".to_string());
        env_overrides.insert("VAR_B".to_string(), "B".to_string());
        env_overrides.insert("VAR_C".to_string(), "C".to_string());

        let input = create_runner_input(
            task,
            temp_dir.path().to_path_buf(),
            env_overrides,
            5,
            Some(PathBuf::from("/usr/bin/env")),
        );

        let runner = LegacyRunner::new("test-multiple-envs");
        let result = runner.execute(&input);

        assert!(result.is_ok(), "Expected Ok but got: {:?}", result);
        let output = result.unwrap();
        assert!(
            output.stdout.contains("VAR_A=A"),
            "Expected stdout to contain 'VAR_A=A' but got: {}",
            output.stdout
        );
        assert!(
            output.stdout.contains("VAR_B=B"),
            "Expected stdout to contain 'VAR_B=B' but got: {}",
            output.stdout
        );
        assert!(
            output.stdout.contains("VAR_C=C"),
            "Expected stdout to contain 'VAR_C=C' but got: {}",
            output.stdout
        );
    }
}
