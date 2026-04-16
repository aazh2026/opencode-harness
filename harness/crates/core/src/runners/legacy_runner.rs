use crate::error::{ErrorType, Result};
use crate::runners::artifact_persister::{ArtifactPersister, RunnerType};
use crate::runners::binary_resolver::BinaryResolver;
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
        let binary = resolver.resolve_opencode_with_override(input.binary_path.as_ref())?;
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
            RunnerType::Legacy,
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
}
