use crate::error::{ErrorType, Result};
use crate::runners::binary_resolver::BinaryResolver;
use crate::types::artifact::Artifact;
use crate::types::task::Task;
use crate::types::task_status::TaskStatus;
use serde::{Deserialize, Serialize};
use std::process::{Command, Output};
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

pub struct RustRunner {
    name: String,
}

impl RustRunner {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    pub fn execute(&self, task: &Task) -> Result<RustRunnerResult> {
        let start = Instant::now();
        let resolver = BinaryResolver::new();
        let binary = resolver.resolve_opencode_rs()?;
        let task_input = &task.input;

        let output = self.run_command(&binary, &task_input.args, &task_input.cwd)?;

        let duration_ms = start.elapsed().as_millis() as u64;
        let exit_code = output.status.code().unwrap_or(-1);
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let status = TaskStatus::Done;

        Ok(RustRunnerResult::new(&task.id)
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
    use crate::types::TaskInput;

    fn create_test_task(command: &str, args: Vec<String>, cwd: &str) -> Task {
        Task::new(
            "P2-008",
            "Implement RustRunner",
            crate::types::task::TaskCategory::Schema,
            "test-fixture",
            "Implement RustRunner with actual binary invocation",
            "RustRunner executes binaries correctly",
            vec![],
            crate::types::entry_mode::EntryMode::CLI,
            crate::types::agent_mode::AgentMode::OneShot,
            crate::types::provider_mode::ProviderMode::Both,
            TaskInput::new(command, args, cwd),
            vec![],
            crate::types::severity::Severity::High,
            crate::types::execution_policy::ExecutionPolicy::ManualCheck,
            60,
            crate::types::on_missing_dependency::OnMissingDependency::Fail,
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
    #[ignore]
    fn test_rust_runner_execute_echo() {
        let runner = RustRunner::new("opencode");
        let task = create_test_task("echo", vec!["hello".to_string()], "/tmp");

        let result = runner.execute(&task);
        assert!(result.is_ok(), "execute failed: {:?}", result.err());
        let result = result.unwrap();
        assert_eq!(result.task_id, "P2-008");
        assert_eq!(result.status, TaskStatus::Done);
        assert_eq!(result.exit_code, Some(0));
        assert!(result.stdout.contains("hello"));
    }

    #[test]
    #[ignore]
    fn test_rust_runner_execute_with_stderr() {
        let runner = RustRunner::new("opencode");
        let task = create_test_task(
            "sh",
            vec!["-c".to_string(), "echo error 1>&2".to_string()],
            "/tmp",
        );

        let result = runner.execute(&task);
        assert!(result.is_ok(), "execute failed: {:?}", result.err());
        let result = result.unwrap();
        assert!(result.stderr.contains("error"));
    }

    #[test]
    #[ignore]
    fn test_rust_runner_execute_with_args() {
        let runner = RustRunner::new("opencode");
        let task = create_test_task(
            "printf",
            vec!["%s %d\n".to_string(), "test".to_string(), "42".to_string()],
            "/tmp",
        );

        let result = runner.execute(&task);
        assert!(result.is_ok(), "execute failed: {:?}", result.err());
        let result = result.unwrap();
        assert!(result.stdout.contains("test"));
        assert!(result.stdout.contains("42"));
    }
}
