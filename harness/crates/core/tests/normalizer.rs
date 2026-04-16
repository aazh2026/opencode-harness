use opencode_core::loaders::DefaultTaskLoader;
use opencode_core::normalizers::{
    normalize_for_comparison, normalize_output, NormalizedOutput, Normalizer, VarianceNormalizer,
    WhitespaceNormalizer,
};
use opencode_core::runners::DifferentialRunner;
use tempfile::TempDir;

#[test]
fn test_normalizer_trait_is_defined() {
    fn assert_normalizer<T: Normalizer>() {}
    assert_normalizer::<WhitespaceNormalizer>();
    assert_normalizer::<VarianceNormalizer>();
}

#[test]
fn test_normalize_method_correctly_normalizes_outputs() {
    let normalizer = WhitespaceNormalizer;

    let input = "  hello   world  \n";
    let result = normalizer.normalize(input);
    assert_eq!(result, "hello world");
}

#[test]
fn test_whitespace_normalizer_trims_leading_trailing() {
    let normalizer = WhitespaceNormalizer;
    assert_eq!(normalizer.normalize("  hello  "), "hello");
    assert_eq!(normalizer.normalize("\t\thello\t\t"), "hello");
}

#[test]
fn test_whitespace_normalizer_collapse_tabs() {
    let normalizer = WhitespaceNormalizer;
    assert_eq!(normalizer.normalize("hello\t\tworld"), "hello world");
}

#[test]
fn test_variance_normalizer_handles_timestamps() {
    let mut normalizer = VarianceNormalizer::new();
    normalizer.add_timestamp_pattern();

    let input = "Event at 2024-01-15T10:30:00 completed";
    let result = normalizer.normalize(input);

    assert!(result.contains("<TIMESTAMP>"));
    assert!(!result.contains("2024-01-15T10:30:00"));
}

#[test]
fn test_normalized_output_apply() {
    let output = NormalizedOutput::new("  hello \t world  ", "  error  ");
    let result = output.apply(&WhitespaceNormalizer);

    assert_eq!(result.stdout, "hello world");
    assert_eq!(result.stderr, "error");
    assert!(result.normalized);
}

#[test]
fn test_normalized_output_from_process_output() {
    let process_output = std::process::Output {
        stdout: b"  output  ".to_vec(),
        stderr: b"  error  ".to_vec(),
        status: std::process::ExitStatus::default(),
    };

    let normalized = NormalizedOutput::from_output(&process_output);
    assert_eq!(normalized.stdout, "  output  ");
    assert_eq!(normalized.stderr, "  error  ");
}

#[test]
fn test_normalize_output_function() {
    let input = "  line1  \n  line2  \n";
    let result = normalize_output(input);
    assert_eq!(result, "line1 line2");
}

#[test]
fn test_normalize_for_comparison_equal() {
    let left = "  hello   world  ";
    let right = "hello world";
    assert!(normalize_for_comparison(left, right));
}

#[test]
fn test_normalize_for_comparison_not_equal() {
    let left = "hello world";
    let right = "hello world!";
    assert!(!normalize_for_comparison(left, right));
}

#[test]
fn test_normalizer_integrates_with_differential_runner() {
    let loader = DefaultTaskLoader::new();
    let runner = DifferentialRunner::new(loader);

    let temp_dir = TempDir::new().unwrap();
    let task_yaml = temp_dir.path().join("normalize_task.yaml");

    std::fs::write(
        &task_yaml,
        r#"
id: NORM-INT-001
title: Normalizer Integration Test
category: integration
fixture_project: fixtures/projects/cli-basic
description: Test normalizer with DifferentialRunner
expected_outcome: Normalizer correctly processes runner output
preconditions: []
entry_mode: CLI
agent_mode: OneShot
provider_mode: Both
input:
  command: echo
  args: ["  hello   world  "]
  cwd: "/tmp"
expected_assertions:
  - type: exit_code_equals
    value: 0
  - type: stdout_contains
    value: "hello"
severity: High
tags:
  - integration
  - normalizer
execution_policy: ManualCheck
timeout_seconds: 60
on_missing_dependency: Fail
"#,
    )
    .unwrap();

    let result = runner.execute_single(&task_yaml).unwrap();
    assert!(result.assertions_passed);

    let normalized = normalize_output(&result.stdout);
    assert!(normalized.contains("hello world"));
}

#[test]
fn test_normalizer_with_multiple_outputs_from_runner() {
    let loader = DefaultTaskLoader::new();
    let runner = DifferentialRunner::new(loader);

    let temp_dir = TempDir::new().unwrap();

    let task1_yaml = temp_dir.path().join("task1.yaml");
    std::fs::write(
        &task1_yaml,
        r#"
id: NORM-MULTI-001
title: Multi Normalize 1
category: smoke
fixture_project: fixtures/projects/cli-basic
description: First output
expected_outcome: Passes
preconditions: []
entry_mode: CLI
agent_mode: OneShot
provider_mode: Both
input:
  command: echo
  args: ["  line1  "]
  cwd: "/tmp"
expected_assertions:
  - type: exit_code_equals
    value: 0
severity: High
tags: []
execution_policy: ManualCheck
timeout_seconds: 60
on_missing_dependency: Fail
"#,
    )
    .unwrap();

    let task2_yaml = temp_dir.path().join("task2.yaml");
    std::fs::write(
        &task2_yaml,
        r#"
id: NORM-MULTI-002
title: Multi Normalize 2
category: smoke
fixture_project: fixtures/projects/cli-basic
description: Second output
expected_outcome: Passes
preconditions: []
entry_mode: CLI
agent_mode: OneShot
provider_mode: Both
input:
  command: echo
  args: ["  line2  "]
  cwd: "/tmp"
expected_assertions:
  - type: exit_code_equals
    value: 0
severity: High
tags: []
execution_policy: ManualCheck
timeout_seconds: 60
on_missing_dependency: Fail
"#,
    )
    .unwrap();

    let result1 = runner.execute_single(&task1_yaml).unwrap();
    let result2 = runner.execute_single(&task2_yaml).unwrap();

    let norm1 = normalize_output(&result1.stdout);
    let norm2 = normalize_output(&result2.stdout);

    assert!(norm1.contains("line1"));
    assert!(norm2.contains("line2"));
    assert!(normalize_for_comparison(&result1.stdout, "  line1  "));
}

#[test]
fn test_normalizer_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<WhitespaceNormalizer>();
    assert_send_sync::<VarianceNormalizer>();
    assert_send_sync::<NormalizedOutput>();
}

#[test]
fn test_normalizer_name() {
    let normalizer = WhitespaceNormalizer;
    assert_eq!(normalizer.name(), "whitespace");

    let normalizer = VarianceNormalizer::new();
    assert_eq!(normalizer.name(), "variance");
}
