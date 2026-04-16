use runners::ExecutionResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    pub id: String,
    pub description: String,
    pub expected_outcome: String,
    pub constraints: Vec<ContractConstraint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractConstraint {
    pub field: String,
    pub operator: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationOutcome {
    Passed,
    Failed,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub outcome: VerificationOutcome,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

impl VerificationResult {
    pub fn passed(msg: impl Into<String>) -> Self {
        Self {
            outcome: VerificationOutcome::Passed,
            message: msg.into(),
            details: None,
        }
    }

    pub fn failed(msg: impl Into<String>) -> Self {
        Self {
            outcome: VerificationOutcome::Failed,
            message: msg.into(),
            details: None,
        }
    }

    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            outcome: VerificationOutcome::Error,
            message: msg.into(),
            details: None,
        }
    }
}

pub trait Verifier: Send + Sync {
    fn verify(&self, execution: &ExecutionResult, contract: &Contract) -> VerificationResult;

    fn name(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::types::task_status::TaskStatus;

    struct TestVerifier;

    impl Verifier for TestVerifier {
        fn verify(&self, execution: &ExecutionResult, _contract: &Contract) -> VerificationResult {
            if execution.is_success() {
                VerificationResult::passed(format!(
                    "Task {} completed successfully",
                    execution.task_id
                ))
            } else {
                VerificationResult::failed(format!(
                    "Task {} did not complete successfully",
                    execution.task_id
                ))
            }
        }

        fn name(&self) -> &str {
            "test-verifier"
        }
    }

    #[test]
    fn test_contract_creation() {
        let contract = Contract {
            id: "test-contract".to_string(),
            description: "Test contract".to_string(),
            expected_outcome: "success".to_string(),
            constraints: vec![ContractConstraint {
                field: "exit_code".to_string(),
                operator: "eq".to_string(),
                value: serde_json::json!(0),
            }],
        };
        assert_eq!(contract.id, "test-contract");
        assert_eq!(contract.constraints.len(), 1);
    }

    #[test]
    fn test_verification_result_passed() {
        let result = VerificationResult::passed("All good");
        assert_eq!(result.outcome, VerificationOutcome::Passed);
        assert_eq!(result.message, "All good");
        assert!(result.details.is_none());
    }

    #[test]
    fn test_verification_result_failed() {
        let result = VerificationResult::failed("Something went wrong");
        assert_eq!(result.outcome, VerificationOutcome::Failed);
        assert_eq!(result.message, "Something went wrong");
    }

    #[test]
    fn test_verification_result_error() {
        let result = VerificationResult::error("Unexpected error");
        assert_eq!(result.outcome, VerificationOutcome::Error);
        assert_eq!(result.message, "Unexpected error");
    }

    #[test]
    fn test_verifier_trait_defined() {
        fn assert_verifier<T: Verifier>() {}
        assert_verifier::<TestVerifier>();
    }

    #[test]
    fn test_verifier_verify_method_signature() {
        fn takes_verifier(v: &dyn Verifier) {
            let execution = ExecutionResult::new("test-task")
                .with_status(TaskStatus::Done)
                .with_exit_code(0);
            let contract = Contract {
                id: "test".to_string(),
                description: "test".to_string(),
                expected_outcome: "success".to_string(),
                constraints: vec![],
            };
            let _ = v.verify(&execution, &contract);
        }
        takes_verifier(&TestVerifier);
    }

    #[test]
    fn test_verifier_accepts_execution_result_and_contract() {
        let verifier = TestVerifier;
        let execution = ExecutionResult::new("P2-004")
            .with_status(TaskStatus::Done)
            .with_exit_code(0);
        let contract = Contract {
            id: "test-contract".to_string(),
            description: "Test contract".to_string(),
            expected_outcome: "success".to_string(),
            constraints: vec![],
        };

        let result = verifier.verify(&execution, &contract);
        assert_eq!(result.outcome, VerificationOutcome::Passed);
    }

    #[test]
    fn test_verifier_name() {
        let verifier = TestVerifier;
        assert_eq!(verifier.name(), "test-verifier");
    }
}
