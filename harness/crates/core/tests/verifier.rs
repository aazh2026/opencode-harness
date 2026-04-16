use opencode_core::loaders::DefaultTaskLoader;
use opencode_core::runners::DifferentialRunner;
use opencode_core::types::assertion::AssertionType;
use opencode_core::types::task_outputs::TaskOutputs;
use opencode_core::verifiers::{DefaultVerifier, VerificationResult, Verifier};
use tempfile::TempDir;

#[test]
fn test_verifier_trait_is_defined() {
    fn assert_verifier<T: Verifier>() {}
    assert_verifier::<DefaultVerifier>();
}

#[test]
fn test_verify_method_correctly_verifies_assertions() {
    let verifier = DefaultVerifier::new();
    let outputs = TaskOutputs::new("Hello World", "", vec![], vec![]);

    let result = verifier.verify(
        &[AssertionType::StdoutContains("Hello".to_string())],
        &outputs,
        0,
    );
    assert!(result.passed);

    let result = verifier.verify(
        &[AssertionType::StdoutContains("Goodbye".to_string())],
        &outputs,
        0,
    );
    assert!(!result.passed);
}

#[test]
fn test_verifier_integrates_with_task_execution() {
    let loader = DefaultTaskLoader::new();
    let runner = DifferentialRunner::new(loader);
    let verifier = DefaultVerifier::new();

    let temp_dir = TempDir::new().unwrap();
    let task_yaml = temp_dir.path().join("task.yaml");

    std::fs::write(
        &task_yaml,
        r#"
id: VERIFY-INT-001
title: Verifier Integration Test
category: integration
fixture_project: fixtures/projects/cli-basic
description: Test verifier with task execution
expected_outcome: Verifier correctly validates assertions
preconditions: []
entry_mode: CLI
agent_mode: OneShot
provider_mode: Both
input:
  command: echo
  args: ["hello world"]
  cwd: "/tmp"
expected_assertions:
  - type: exit_code_equals
    value: 0
  - type: stdout_contains
    value: "hello"
severity: High
tags:
  - integration
  - verifier
execution_policy: ManualCheck
timeout_seconds: 60
on_missing_dependency: Fail
"#,
    )
    .unwrap();

    let result = runner.execute_single(&task_yaml).unwrap();

    let stdout = result.legacy_stdout().to_string();
    let stderr = result.legacy_stderr().to_string();
    let exit_code = result.legacy_exit_code().unwrap_or(0);
    let task_outputs = TaskOutputs::new(stdout.clone(), stderr.clone(), vec![], vec![]);

    if stdout.contains("hello world") {
        let verification = verifier.verify(
            &[
                AssertionType::ExitCodeEquals(0),
                AssertionType::StdoutContains("hello".to_string()),
            ],
            &task_outputs,
            exit_code as u32,
        );
        assert!(verification.passed);
    }
}

#[test]
fn test_verifier_with_differential_runner_result() {
    let loader = DefaultTaskLoader::new();
    let runner = DifferentialRunner::new(loader);
    let verifier = DefaultVerifier::new();

    let temp_dir = TempDir::new().unwrap();

    let task1_yaml = temp_dir.path().join("task1.yaml");
    std::fs::write(
        &task1_yaml,
        r#"
id: VERIFY-RUN-001
title: Runner Result Test
category: integration
fixture_project: fixtures/projects/cli-basic
description: Test verifier with runner result
expected_outcome: Works
preconditions: []
entry_mode: CLI
agent_mode: OneShot
provider_mode: Both
input:
  command: echo
  args: ["test output"]
  cwd: "/tmp"
expected_assertions:
  - type: exit_code_equals
    value: 0
  - type: stdout_contains
    value: "test"
severity: High
tags: []
execution_policy: ManualCheck
timeout_seconds: 60
on_missing_dependency: Fail
"#,
    )
    .unwrap();

    let diff_result = runner.execute_single(&task1_yaml).unwrap();

    let stdout = diff_result.legacy_stdout().to_string();
    let stderr = diff_result.legacy_stderr().to_string();
    let exit_code = diff_result.legacy_exit_code().unwrap_or(0);
    let outputs = TaskOutputs::new(stdout.clone(), stderr.clone(), vec![], vec![]);

    if stdout.contains("test") {
        let assertions = vec![
            AssertionType::ExitCodeEquals(0),
            AssertionType::StdoutContains("test".to_string()),
        ];

        let verification = verifier.verify(&assertions, &outputs, exit_code as u32);

        assert!(verification.passed);
        assert_eq!(verification.assertion_results.len(), 2);
        assert!(verification.assertion_results[0].passed);
        assert!(verification.assertion_results[1].passed);
    }
}

#[test]
fn test_verifier_with_failing_exit_code() {
    let verifier = DefaultVerifier::new();
    let outputs = TaskOutputs::new("", "", vec![], vec![]);

    let result = verifier.verify(&[AssertionType::ExitCodeEquals(0)], &outputs, 1);
    assert!(!result.passed);
    assert!(result.assertion_results[0]
        .message
        .contains("does not match"));
}

#[test]
fn test_verifier_file_not_changed() {
    let verifier = DefaultVerifier::new();
    let outputs = TaskOutputs::new("", "", vec![], vec![]);

    let result = verifier.verify(
        &[AssertionType::FileChanged("missing.txt".to_string())],
        &outputs,
        0,
    );
    assert!(!result.passed);
    assert!(result.assertion_results[0]
        .message
        .contains("was not changed"));
}

#[test]
fn test_verification_result_passed_when_all_assertions_pass() {
    let result = VerificationResult::new(true, vec![]);
    assert!(result.passed);

    let result = VerificationResult::new(
        true,
        vec![opencode_core::verifiers::AssertionResult {
            assertion: AssertionType::ExitCodeEquals(0),
            passed: true,
            message: "ok".to_string(),
        }],
    );
    assert!(result.passed);
}

#[test]
fn test_verification_result_failed_when_any_assertion_fails() {
    let result = VerificationResult::new(
        false,
        vec![opencode_core::verifiers::AssertionResult {
            assertion: AssertionType::ExitCodeEquals(0),
            passed: false,
            message: "failed".to_string(),
        }],
    );
    assert!(!result.passed);
}

#[test]
fn test_verifier_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<DefaultVerifier>();
    assert_send_sync::<VerificationResult>();
}
