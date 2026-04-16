use core::types::failure_classification::FailureClassification;
use core::types::task_status::TaskStatus;
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
}
