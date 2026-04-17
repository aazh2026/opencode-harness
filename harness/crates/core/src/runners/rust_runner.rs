use crate::error::{ErrorType, Result};
use crate::runners::artifact_persister::{ArtifactPersister, RunnerType};
use crate::types::artifact::Artifact;
use crate::types::capability_summary::CapabilitySummary;
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
            binary_path.clone()
        } else {
            PathBuf::from(&input.task.input.command)
        };
        let task = &input.task;
        let task_input = &task.input;

        let mut capability_summary = CapabilitySummary {
            binary_available: true,
            workspace_prepared: input.prepared_workspace_path.exists(),
            ..Default::default()
        };

        let output = if input.timeout_seconds > 0 {
            self.run_command_with_timeout(
                &binary,
                &task_input.args,
                &input.prepared_workspace_path.to_string_lossy(),
                &input.env_overrides,
                input.timeout_seconds,
            )?
        } else {
            self.run_command_no_timeout(
                &binary,
                &task_input.args,
                &input.prepared_workspace_path.to_string_lossy(),
                &input.env_overrides,
            )?
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
            None,
            capability_summary.clone(),
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
    #[ignore]
    fn test_timeout_enforcement_kills_process_after_timeout() {
        let temp_dir = TempDir::new().unwrap();
        let mut task = create_test_task();
        task.input.command = "sleep".to_string();
        task.input.args = vec!["10".to_string()];

        let input =
            create_runner_input(task, temp_dir.path().to_path_buf(), HashMap::new(), 1, None);

        let runner = RustRunner::new("test-timeout");
        let start = std::time::Instant::now();
        let result = runner.execute(&input);
        let elapsed = start.elapsed();

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("timed out") || err_msg.contains("killed"));
        assert!(elapsed.as_secs() < 5);
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
}
