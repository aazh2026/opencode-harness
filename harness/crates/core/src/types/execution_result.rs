use crate::types::failure_classification::FailureClassification;
use crate::types::task_status::TaskStatus;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub task_id: String,
    pub status: TaskStatus,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub failure_classification: Option<FailureClassification>,
    pub error_message: Option<String>,
}

impl ExecutionResult {
    pub fn new(task_id: impl Into<String>) -> Self {
        Self {
            task_id: task_id.into(),
            status: TaskStatus::Todo,
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            duration_ms: 0,
            failure_classification: None,
            error_message: None,
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

    pub fn with_failure_classification(mut self, classification: FailureClassification) -> Self {
        self.failure_classification = Some(classification);
        self
    }

    pub fn with_error_message(mut self, error_message: String) -> Self {
        self.error_message = Some(error_message);
        self
    }

    pub fn is_success(&self) -> bool {
        self.status == TaskStatus::Done && self.exit_code == Some(0)
    }
}

impl From<crate::runners::LegacyRunnerResult> for ExecutionResult {
    fn from(result: crate::runners::LegacyRunnerResult) -> Self {
        Self {
            task_id: result.task_id,
            status: result.status,
            exit_code: result.exit_code,
            stdout: result.stdout,
            stderr: result.stderr,
            duration_ms: result.duration_ms,
            failure_classification: None,
            error_message: None,
        }
    }
}

impl From<crate::runners::RustRunnerResult> for ExecutionResult {
    fn from(result: crate::runners::RustRunnerResult) -> Self {
        Self {
            task_id: result.task_id,
            status: result.status,
            exit_code: result.exit_code,
            stdout: result.stdout,
            stderr: result.stderr,
            duration_ms: result.duration_ms,
            failure_classification: None,
            error_message: None,
        }
    }
}

impl From<crate::types::runner_output::RunnerOutput> for ExecutionResult {
    fn from(output: crate::types::runner_output::RunnerOutput) -> Self {
        let stderr = output.stderr.clone();
        Self {
            task_id: output.session_metadata.task_id,
            status: TaskStatus::Done,
            exit_code: output.exit_code,
            stdout: output.stdout,
            stderr,
            duration_ms: output.duration_ms,
            failure_classification: output.failure_kind,
            error_message: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_result_creation() {
        let result = ExecutionResult::new("P2-003");
        assert_eq!(result.task_id, "P2-003");
        assert_eq!(result.status, TaskStatus::Todo);
        assert!(result.exit_code.is_none());
        assert!(result.stdout.is_empty());
        assert!(result.stderr.is_empty());
        assert_eq!(result.duration_ms, 0);
        assert!(result.failure_classification.is_none());
        assert!(result.error_message.is_none());
    }

    #[test]
    fn test_execution_result_builder() {
        let result = ExecutionResult::new("P2-003")
            .with_status(TaskStatus::Done)
            .with_exit_code(0)
            .with_stdout("Test output".to_string())
            .with_stderr("".to_string())
            .with_duration_ms(100);

        assert_eq!(result.task_id, "P2-003");
        assert_eq!(result.status, TaskStatus::Done);
        assert_eq!(result.exit_code, Some(0));
        assert_eq!(result.stdout, "Test output");
        assert_eq!(result.duration_ms, 100);
    }

    #[test]
    fn test_execution_result_is_success() {
        let success_result = ExecutionResult::new("P2-003")
            .with_status(TaskStatus::Done)
            .with_exit_code(0);

        assert!(success_result.is_success());

        let failed_result = ExecutionResult::new("P2-003")
            .with_status(TaskStatus::Done)
            .with_exit_code(1);

        assert!(!failed_result.is_success());

        let in_progress_result = ExecutionResult::new("P2-003")
            .with_status(TaskStatus::InProgress)
            .with_exit_code(0);

        assert!(!in_progress_result.is_success());
    }

    #[test]
    fn test_execution_result_serde_json() {
        let result = ExecutionResult::new("P2-003")
            .with_status(TaskStatus::Done)
            .with_exit_code(0)
            .with_stdout("output".to_string())
            .with_duration_ms(50);

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"task_id\":\"P2-003\""));
        assert!(json.contains("\"status\":\"Done\""));

        let deserialized: ExecutionResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.task_id, "P2-003");
        assert_eq!(deserialized.status, TaskStatus::Done);
    }

    #[test]
    fn test_execution_result_from_legacy_runner_result() {
        let legacy = crate::runners::LegacyRunnerResult::new("TEST-001")
            .with_status(TaskStatus::Done)
            .with_exit_code(0)
            .with_stdout("legacy output".to_string())
            .with_stderr("".to_string())
            .with_duration_ms(100);

        let result: ExecutionResult = legacy.into();
        assert_eq!(result.task_id, "TEST-001");
        assert_eq!(result.status, TaskStatus::Done);
        assert_eq!(result.exit_code, Some(0));
        assert_eq!(result.stdout, "legacy output");
        assert_eq!(result.duration_ms, 100);
    }

    #[test]
    fn test_execution_result_from_rust_runner_result() {
        let rust = crate::runners::RustRunnerResult::new("TEST-002")
            .with_status(TaskStatus::Done)
            .with_exit_code(1)
            .with_stdout("rust output".to_string())
            .with_stderr("error".to_string())
            .with_duration_ms(200);

        let result: ExecutionResult = rust.into();
        assert_eq!(result.task_id, "TEST-002");
        assert_eq!(result.status, TaskStatus::Done);
        assert_eq!(result.exit_code, Some(1));
        assert_eq!(result.stdout, "rust output");
        assert_eq!(result.stderr, "error");
        assert_eq!(result.duration_ms, 200);
    }

    #[test]
    fn test_execution_result_from_runner_output() {
        let output = crate::types::runner_output::RunnerOutput::default()
            .with_exit_code(Some(0))
            .with_stdout("runner output".to_string())
            .with_stderr("".to_string())
            .with_duration_ms(300);

        let result: ExecutionResult = output.into();
        assert_eq!(result.task_id, "default");
        assert_eq!(result.status, TaskStatus::Done);
        assert_eq!(result.exit_code, Some(0));
        assert_eq!(result.stdout, "runner output");
        assert_eq!(result.duration_ms, 300);
    }
}
