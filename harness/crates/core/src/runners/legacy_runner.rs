use crate::error::{ErrorType, Result};
use crate::runners::binary_resolver::BinaryResolver;
use crate::types::task::Task;
use crate::types::task_status::TaskStatus;
use serde::{Deserialize, Serialize};
use std::process::{Command, Output};
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

    pub fn execute(&self, task: &Task) -> Result<LegacyRunnerResult> {
        let start = Instant::now();
        let resolver = BinaryResolver::new();
        let binary = resolver.resolve_opencode()?;
        let task_input = &task.input;

        let output = self.run_command(&binary, &task_input.args, &task_input.cwd)?;

        let duration_ms = start.elapsed().as_millis() as u64;
        let exit_code = output.status.code().unwrap_or(-1);
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let status = TaskStatus::Done;

        Ok(LegacyRunnerResult::new(&task.id)
            .with_status(status)
            .with_exit_code(exit_code)
            .with_stdout(stdout)
            .with_stderr(stderr)
            .with_duration_ms(duration_ms))
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    fn run_command(&self, binary: &std::path::Path, args: &[String], cwd: &str) -> Result<Output> {
        Command::new(binary)
            .args(args)
            .current_dir(cwd)
            .output()
            .map_err(|e| {
                ErrorType::Runner(format!("Failed to execute '{}': {}", binary.display(), e))
            })
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
