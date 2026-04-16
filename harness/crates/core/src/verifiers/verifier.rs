use crate::types::assertion::AssertionType;
use crate::types::task_outputs::TaskOutputs;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationResult {
    pub passed: bool,
    pub assertion_results: Vec<AssertionResult>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssertionResult {
    pub assertion: AssertionType,
    pub passed: bool,
    pub message: String,
}

impl VerificationResult {
    pub fn new(passed: bool, assertion_results: Vec<AssertionResult>) -> Self {
        Self {
            passed,
            assertion_results,
        }
    }

    pub fn all_passed() -> Self {
        Self {
            passed: true,
            assertion_results: Vec::new(),
        }
    }
}

pub trait Verifier: Send + Sync {
    fn verify(
        &self,
        assertions: &[AssertionType],
        outputs: &TaskOutputs,
        exit_code: u32,
    ) -> VerificationResult;
}

pub struct DefaultVerifier;

impl DefaultVerifier {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultVerifier {
    fn default() -> Self {
        Self::new()
    }
}

impl Verifier for DefaultVerifier {
    fn verify(
        &self,
        assertions: &[AssertionType],
        outputs: &TaskOutputs,
        exit_code: u32,
    ) -> VerificationResult {
        let assertion_results: Vec<AssertionResult> = assertions
            .iter()
            .map(|assertion| verify_single_assertion(assertion, outputs, exit_code))
            .collect();

        let passed = assertion_results.iter().all(|r| r.passed);

        VerificationResult::new(passed, assertion_results)
    }
}

fn verify_single_assertion(
    assertion: &AssertionType,
    outputs: &TaskOutputs,
    exit_code: u32,
) -> AssertionResult {
    match assertion {
        AssertionType::ExitCodeEquals(expected_code) => {
            let passed = exit_code == *expected_code;
            AssertionResult {
                assertion: assertion.clone(),
                passed,
                message: if passed {
                    format!("Exit code {} matches expected {}", exit_code, expected_code)
                } else {
                    format!(
                        "Exit code {} does not match expected {}",
                        exit_code, expected_code
                    )
                },
            }
        }
        AssertionType::StdoutContains(expected) => {
            let passed = outputs.stdout.contains(expected);
            AssertionResult {
                assertion: assertion.clone(),
                passed,
                message: if passed {
                    format!("stdout contains '{}'", expected)
                } else {
                    format!("stdout does not contain '{}'", expected)
                },
            }
        }
        AssertionType::StderrContains(expected) => {
            let passed = outputs.stderr.contains(expected);
            AssertionResult {
                assertion: assertion.clone(),
                passed,
                message: if passed {
                    format!("stderr contains '{}'", expected)
                } else {
                    format!("stderr does not contain '{}'", expected)
                },
            }
        }
        AssertionType::FileChanged(expected_path) => {
            let passed = outputs.files_modified.iter().any(|p| p == expected_path)
                || outputs.files_created.iter().any(|p| p == expected_path);
            AssertionResult {
                assertion: assertion.clone(),
                passed,
                message: if passed {
                    format!("file '{}' was changed", expected_path)
                } else {
                    format!("file '{}' was not changed", expected_path)
                },
            }
        }
        AssertionType::NoExtraFilesChanged => {
            let passed = outputs.files_created.is_empty() && outputs.files_modified.is_empty();
            AssertionResult {
                assertion: assertion.clone(),
                passed,
                message: if passed {
                    "no extra files changed".to_string()
                } else {
                    format!(
                        "extra files changed: created={:?}, modified={:?}",
                        outputs.files_created, outputs.files_modified
                    )
                },
            }
        }
        AssertionType::PermissionPromptSeen(expected_prompt) => {
            let passed = outputs.stdout.contains(expected_prompt)
                || outputs.stderr.contains(expected_prompt);
            AssertionResult {
                assertion: assertion.clone(),
                passed,
                message: if passed {
                    format!("permission prompt '{}' was seen", expected_prompt)
                } else {
                    format!("permission prompt '{}' was not seen", expected_prompt)
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verifier_trait_is_defined() {
        fn assert_verifier<T: Verifier>() {}
        assert_verifier::<DefaultVerifier>();
    }

    #[test]
    fn test_verify_exit_code_equals_pass() {
        let verifier = DefaultVerifier::new();
        let outputs = TaskOutputs::new("output", "", vec![], vec![]);
        let assertions = vec![AssertionType::ExitCodeEquals(0)];

        let result = verifier.verify(&assertions, &outputs, 0);
        assert!(result.passed);
        assert_eq!(result.assertion_results.len(), 1);
        assert!(result.assertion_results[0].passed);
    }

    #[test]
    fn test_verify_exit_code_equals_fail() {
        let verifier = DefaultVerifier::new();
        let outputs = TaskOutputs::new("output", "", vec![], vec![]);
        let assertions = vec![AssertionType::ExitCodeEquals(1)];

        let result = verifier.verify(&assertions, &outputs, 0);
        assert!(!result.passed);
        assert!(!result.assertion_results[0].passed);
    }

    #[test]
    fn test_verify_stdout_contains_pass() {
        let verifier = DefaultVerifier::new();
        let outputs = TaskOutputs::new("Hello World", "", vec![], vec![]);
        let assertions = vec![AssertionType::StdoutContains("World".to_string())];

        let result = verifier.verify(&assertions, &outputs, 0);
        assert!(result.passed);
    }

    #[test]
    fn test_verify_stdout_contains_fail() {
        let verifier = DefaultVerifier::new();
        let outputs = TaskOutputs::new("Hello World", "", vec![], vec![]);
        let assertions = vec![AssertionType::StdoutContains("Goodbye".to_string())];

        let result = verifier.verify(&assertions, &outputs, 0);
        assert!(!result.passed);
    }

    #[test]
    fn test_verify_stderr_contains_pass() {
        let verifier = DefaultVerifier::new();
        let outputs = TaskOutputs::new("", "Error: permission denied", vec![], vec![]);
        let assertions = vec![AssertionType::StderrContains(
            "permission denied".to_string(),
        )];

        let result = verifier.verify(&assertions, &outputs, 0);
        assert!(result.passed);
    }

    #[test]
    fn test_verify_file_changed_in_modified() {
        let verifier = DefaultVerifier::new();
        let outputs = TaskOutputs::new("", "", vec![], vec!["src/main.rs".to_string()]);
        let assertions = vec![AssertionType::FileChanged("src/main.rs".to_string())];

        let result = verifier.verify(&assertions, &outputs, 0);
        assert!(result.passed);
    }

    #[test]
    fn test_verify_file_changed_in_created() {
        let verifier = DefaultVerifier::new();
        let outputs = TaskOutputs::new("", "", vec!["new_file.txt".to_string()], vec![]);
        let assertions = vec![AssertionType::FileChanged("new_file.txt".to_string())];

        let result = verifier.verify(&assertions, &outputs, 0);
        assert!(result.passed);
    }

    #[test]
    fn test_verify_file_changed_fail() {
        let verifier = DefaultVerifier::new();
        let outputs = TaskOutputs::new("", "", vec![], vec!["other_file.rs".to_string()]);
        let assertions = vec![AssertionType::FileChanged("src/main.rs".to_string())];

        let result = verifier.verify(&assertions, &outputs, 0);
        assert!(!result.passed);
    }

    #[test]
    fn test_verify_no_extra_files_changed_pass() {
        let verifier = DefaultVerifier::new();
        let outputs = TaskOutputs::new("output", "", vec![], vec![]);
        let assertions = vec![AssertionType::NoExtraFilesChanged];

        let result = verifier.verify(&assertions, &outputs, 0);
        assert!(result.passed);
    }

    #[test]
    fn test_verify_no_extra_files_changed_fail() {
        let verifier = DefaultVerifier::new();
        let outputs = TaskOutputs::new("output", "", vec!["extra.txt".to_string()], vec![]);
        let assertions = vec![AssertionType::NoExtraFilesChanged];

        let result = verifier.verify(&assertions, &outputs, 0);
        assert!(!result.passed);
    }

    #[test]
    fn test_verify_permission_prompt_seen_pass() {
        let verifier = DefaultVerifier::new();
        let outputs = TaskOutputs::new("Allow access?", "", vec![], vec![]);
        let assertions = vec![AssertionType::PermissionPromptSeen(
            "Allow access?".to_string(),
        )];

        let result = verifier.verify(&assertions, &outputs, 0);
        assert!(result.passed);
    }

    #[test]
    fn test_verify_permission_prompt_seen_in_stderr() {
        let verifier = DefaultVerifier::new();
        let outputs = TaskOutputs::new("", "Allow access?", vec![], vec![]);
        let assertions = vec![AssertionType::PermissionPromptSeen(
            "Allow access?".to_string(),
        )];

        let result = verifier.verify(&assertions, &outputs, 0);
        assert!(result.passed);
    }

    #[test]
    fn test_verify_multiple_assertions_all_pass() {
        let verifier = DefaultVerifier::new();
        let outputs = TaskOutputs::new(
            "Usage: opencode",
            "",
            vec![],
            vec!["src/main.rs".to_string()],
        );
        let assertions = vec![
            AssertionType::ExitCodeEquals(0),
            AssertionType::StdoutContains("Usage: opencode".to_string()),
            AssertionType::FileChanged("src/main.rs".to_string()),
        ];

        let result = verifier.verify(&assertions, &outputs, 0);
        assert!(result.passed);
        assert_eq!(result.assertion_results.len(), 3);
    }

    #[test]
    fn test_verify_multiple_assertions_one_fails() {
        let verifier = DefaultVerifier::new();
        let outputs = TaskOutputs::new("Usage: opencode", "", vec![], vec![]);
        let assertions = vec![
            AssertionType::ExitCodeEquals(0),
            AssertionType::StdoutContains("Usage: opencode".to_string()),
            AssertionType::FileChanged("src/main.rs".to_string()),
        ];

        let result = verifier.verify(&assertions, &outputs, 0);
        assert!(!result.passed);
        assert_eq!(result.assertion_results.len(), 3);
        assert!(result.assertion_results[0].passed);
        assert!(result.assertion_results[1].passed);
        assert!(!result.assertion_results[2].passed);
    }

    #[test]
    fn test_verify_empty_assertions() {
        let verifier = DefaultVerifier::new();
        let outputs = TaskOutputs::new("output", "error", vec![], vec![]);
        let assertions: Vec<AssertionType> = vec![];

        let result = verifier.verify(&assertions, &outputs, 0);
        assert!(result.passed);
        assert!(result.assertion_results.is_empty());
    }

    #[test]
    fn test_verification_result_all_passed() {
        let result = VerificationResult::all_passed();
        assert!(result.passed);
        assert!(result.assertion_results.is_empty());
    }

    #[test]
    fn test_default_verifier_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<DefaultVerifier>();
    }
}
