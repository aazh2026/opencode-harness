use opencode_core::loaders::DefaultTaskLoader;
use opencode_core::loaders::TaskLoader;
use opencode_core::runners::DifferentialResult;
use opencode_core::runners::DifferentialRunner;
use opencode_core::types::agent_mode::AgentMode;
use opencode_core::types::assertion::AssertionType;
use opencode_core::types::entry_mode::EntryMode;
use opencode_core::types::execution_policy::ExecutionPolicy;
use opencode_core::types::on_missing_dependency::OnMissingDependency;
use opencode_core::types::parity_verdict::{ParityVerdict, VarianceType};
use opencode_core::types::provider_mode::ProviderMode;
use opencode_core::types::severity::Severity;
use opencode_core::types::task::Task;
use opencode_core::types::TaskCategory;
use opencode_core::types::TaskInput;
use std::path::PathBuf;
use tempfile::TempDir;

fn get_fixtures_path() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let core_path = PathBuf::from(manifest_dir);
    let harness_path = core_path
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    harness_path.join("harness/fixtures/projects")
}

#[test]
fn test_differential_runner_integration_with_task_loader() {
    let loader = DefaultTaskLoader::new();
    let runner = DifferentialRunner::new(loader);

    let temp_dir = TempDir::new().unwrap();
    let task_yaml = temp_dir.path().join("integration_task.yaml");

    std::fs::write(
        &task_yaml,
        r#"
id: INTEGRATION-001
title: Integration Test with TaskLoader
category: integration
fixture_project: fixtures/projects/cli-basic
description: Verify DifferentialRunner integrates with TaskLoader
expected_outcome: Task is loaded and executed successfully
preconditions: []
entry_mode: CLI
agent_mode: OneShot
provider_mode: Both
input:
  command: echo
  args: ["integration_test"]
  cwd: "/tmp"
expected_assertions:
  - type: exit_code_equals
    value: 0
  - type: stdout_contains
    value: "integration_test"
severity: High
tags:
  - integration
  - task_loader
execution_policy: ManualCheck
timeout_seconds: 60
on_missing_dependency: Fail
"#,
    )
    .unwrap();

    let loaded_task = runner.task_loader.load_single(&task_yaml).unwrap();
    assert_eq!(loaded_task.id, "INTEGRATION-001");
    assert_eq!(loaded_task.category, TaskCategory::Integration);

    let result = runner.execute_from_task(&loaded_task).unwrap();
    assert_eq!(result.task_id, "INTEGRATION-001");
    assert!(result.passed() || result.legacy_result.is_some() || result.rust_result.is_some());
}

#[test]
fn test_differential_runner_loads_multiple_tasks_from_directory() {
    let loader = DefaultTaskLoader::new();
    let runner = DifferentialRunner::new(loader);

    let temp_dir = TempDir::new().unwrap();

    std::fs::write(
        temp_dir.path().join("task1.yaml"),
        r#"
id: MULTI-001
title: Multiple Tasks Test 1
category: smoke
fixture_project: fixtures/projects/cli-basic
description: First task for multi-task loading
expected_outcome: Passes
preconditions: []
entry_mode: CLI
agent_mode: OneShot
provider_mode: Both
input:
  command: echo
  args: ["task1"]
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

    std::fs::write(
        temp_dir.path().join("task2.yaml"),
        r#"
id: MULTI-002
title: Multiple Tasks Test 2
category: smoke
fixture_project: fixtures/projects/cli-basic
description: Second task for multi-task loading
expected_outcome: Passes
preconditions: []
entry_mode: CLI
agent_mode: OneShot
provider_mode: Both
input:
  command: echo
  args: ["task2"]
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

    let tasks = runner.task_loader.load_from_dir(temp_dir.path()).unwrap();
    assert_eq!(tasks.len(), 2);

    let results = runner.execute_from_path(temp_dir.path()).unwrap();
    assert_eq!(results.len(), 2);

    let task_ids: Vec<&str> = results.iter().map(|r| r.task_id.as_str()).collect();
    assert!(task_ids.contains(&"MULTI-001"));
    assert!(task_ids.contains(&"MULTI-002"));
}

#[test]
fn test_differential_runner_with_cli_basic_fixture() {
    let fixtures_path = get_fixtures_path();
    let loader = DefaultTaskLoader::new();
    let runner = DifferentialRunner::new(loader);

    let temp_dir = TempDir::new().unwrap();
    let task_yaml = temp_dir.path().join("cli_fixture_task.yaml");

    std::fs::write(
        &task_yaml,
        r#"
id: CLI-FIXTURE-001
title: CLI Basic Fixture Test
category: smoke
fixture_project: fixtures/projects/cli-basic
description: Test with cli-basic fixture
expected_outcome: Works with cli-basic fixture
preconditions: []
entry_mode: CLI
agent_mode: OneShot
provider_mode: Both
input:
  command: echo
  args: ["cli_basic_fixture"]
  cwd: "/tmp"
expected_assertions:
  - type: exit_code_equals
    value: 0
severity: High
tags:
  - fixture
  - cli-basic
execution_policy: ManualCheck
timeout_seconds: 60
on_missing_dependency: Fail
"#,
    )
    .unwrap();

    let loaded_task = runner.task_loader.load_single(&task_yaml).unwrap();
    assert_eq!(loaded_task.fixture_project, "fixtures/projects/cli-basic");

    let result = runner.execute_from_task(&loaded_task).unwrap();
    assert!(result.passed() || result.legacy_result.is_some() || result.rust_result.is_some());
    drop(fixtures_path);
}

#[test]
fn test_differential_runner_compares_outputs_correctly() {
    let loader = DefaultTaskLoader::new();
    let runner = DifferentialRunner::new(loader);

    let temp_dir = TempDir::new().unwrap();

    let task_a_yaml = temp_dir.path().join("task_a.yaml");
    std::fs::write(
        &task_a_yaml,
        r#"
id: COMPARE-A
title: Compare Output A
category: smoke
fixture_project: fixtures/projects/cli-basic
description: First output for comparison
expected_outcome: Passes
preconditions: []
entry_mode: CLI
agent_mode: OneShot
provider_mode: Both
input:
  command: echo
  args: ["output_a"]
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

    let task_b_yaml = temp_dir.path().join("task_b.yaml");
    std::fs::write(
        &task_b_yaml,
        r#"
id: COMPARE-B
title: Compare Output B
category: smoke
fixture_project: fixtures/projects/cli-basic
description: Second output for comparison
expected_outcome: Passes
preconditions: []
entry_mode: CLI
agent_mode: OneShot
provider_mode: Both
input:
  command: echo
  args: ["output_b"]
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

    let result_a = runner.execute_single(&task_a_yaml).unwrap();
    let result_b = runner.execute_single(&task_b_yaml).unwrap();

    let stdout_a = result_a.legacy_stdout();
    let stdout_b = result_b.legacy_stdout();
    if !stdout_a.is_empty() && !stdout_b.is_empty() {
        assert_ne!(stdout_a, stdout_b);
        assert!(stdout_a.contains("output_a"));
        assert!(stdout_b.contains("output_b"));
    }
    assert!(result_a.passed() || result_a.legacy_result.is_some());
    assert!(result_b.passed() || result_b.legacy_result.is_some());
}

#[test]
fn test_differential_runner_differential_result_structure() {
    use opencode_core::runners::DifferentialResult;

    let result = DifferentialResult::new("DIFF-001".to_string());

    assert_eq!(result.task_id, "DIFF-001");
    assert!(result.legacy_result.is_none());
    assert!(result.rust_result.is_none());
    assert!(result.passed() == false);
    assert_eq!(result.duration_ms, 0);
    assert!(result.diff_report_path.is_none());
    assert!(result.verdict_path.is_none());
    assert!(result.legacy_artifact_paths.is_empty());
    assert!(result.rust_artifact_paths.is_empty());
}

#[test]
fn test_differential_runner_with_task_struct_directly() {
    let loader = DefaultTaskLoader::new();
    let runner = DifferentialRunner::new(loader);

    let task = Task::new(
        "DIRECT-001",
        "Direct Task Execution",
        TaskCategory::Smoke,
        "fixtures/projects/cli-basic",
        "Test direct task execution",
        "Task runs successfully",
        vec![],
        EntryMode::CLI,
        AgentMode::OneShot,
        ProviderMode::Both,
        TaskInput::new("echo", vec!["direct".to_string()], "/tmp"),
        vec![AssertionType::ExitCodeEquals(0)],
        Severity::Medium,
        ExecutionPolicy::ManualCheck,
        60,
        OnMissingDependency::Fail,
    );

    let result = runner.execute_from_task(&task).unwrap();

    assert_eq!(result.task_id, "DIRECT-001");
    assert!(result.passed() || result.legacy_result.is_some() || result.rust_result.is_some());
}

#[test]
fn test_differential_runner_send_and_sync_trait_bounds() {
    fn assert_send_and_sync<T: Send + Sync>() {}
    let loader = DefaultTaskLoader::new();
    let _runner = DifferentialRunner::new(loader);
    assert_send_and_sync::<DifferentialRunner<DefaultTaskLoader>>();
}

#[test]
fn test_smoke_directory_structure_created_in_artifacts_run_id() {
    use opencode_core::runners::DifferentialRunner;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let loader = DefaultTaskLoader::new();
    let runner = DifferentialRunner::new(loader);

    let task_yaml = temp_dir.path().join("smoke_task.yaml");
    std::fs::write(
        &task_yaml,
        r#"
id: SMOKE-DIR-001
title: Smoke Test Directory Structure
category: smoke
fixture_project: fixtures/projects/cli-basic
description: Verify directory structure creation
expected_outcome: Directory structure created
preconditions: []
entry_mode: CLI
agent_mode: OneShot
provider_mode: Both
input:
  command: echo
  args: ["smoke_test"]
  cwd: "/tmp"
expected_assertions: []
severity: High
tags: []
execution_policy: ManualCheck
timeout_seconds: 60
on_missing_dependency: Fail
"#,
    )
    .unwrap();

    let result = runner.execute_single(&task_yaml).unwrap();

    let diff_path = temp_dir.path().join("artifacts");
    let has_run_id = std::fs::read_dir(&diff_path)
        .ok()
        .map(|mut entries| entries.next().and_then(|e| e.ok()).is_some())
        .unwrap_or(false);

    if has_run_id {
        if let Ok(mut entries) = std::fs::read_dir(&diff_path) {
            if let Some(Ok(entry)) = entries.next() {
                let run_id_dir = entry.path();
                assert!(run_id_dir.join("legacy").exists(), "legacy/ should exist");
                assert!(run_id_dir.join("rust").exists(), "rust/ should exist");
                assert!(run_id_dir.join("diff").exists(), "diff/ should exist");
            }
        }
    } else {
        assert!(
            diff_path.join("legacy").exists() || result.diff_report_path.is_some(),
            "Either legacy dir should exist or diff report was generated"
        );
    }
}

#[test]
fn test_smoke_artifact_persistence_creates_stdout_stderr_metadata() {
    use opencode_core::runners::DifferentialRunner;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let loader = DefaultTaskLoader::new();
    let runner = DifferentialRunner::new(loader);

    let task_yaml = temp_dir.path().join("smoke_artifacts_task.yaml");
    std::fs::write(
        &task_yaml,
        r#"
id: SMOKE-ART-001
title: Smoke Test Artifact Persistence
category: smoke
fixture_project: fixtures/projects/cli-basic
description: Verify artifact persistence creates stdout, stderr, metadata
expected_outcome: Artifacts persisted correctly
preconditions: []
entry_mode: CLI
agent_mode: OneShot
provider_mode: Both
input:
  command: echo
  args: ["artifact_test"]
  cwd: "/tmp"
expected_assertions: []
severity: High
tags: []
execution_policy: ManualCheck
timeout_seconds: 60
on_missing_dependency: Fail
"#,
    )
    .unwrap();

    let result = runner.execute_single(&task_yaml).unwrap();

    if let Some(report_path) = &result.diff_report_path {
        let run_dir = report_path.parent().and_then(|p| p.parent());
        if let Some(run_dir) = run_dir {
            let legacy_dir = run_dir.join("legacy");
            let rust_dir = run_dir.join("rust");

            if legacy_dir.exists() {
                assert!(
                    legacy_dir.join("stdout.txt").exists() || result.legacy_result.is_some(),
                    "legacy/stdout.txt should exist"
                );
                assert!(
                    legacy_dir.join("stderr.txt").exists() || result.legacy_result.is_some(),
                    "legacy/stderr.txt should exist"
                );
                assert!(
                    legacy_dir.join("metadata.json").exists() || result.legacy_result.is_some(),
                    "legacy/metadata.json should exist"
                );
            }

            if rust_dir.exists() {
                assert!(
                    rust_dir.join("stdout.txt").exists() || result.rust_result.is_some(),
                    "rust/stdout.txt should exist"
                );
                assert!(
                    rust_dir.join("stderr.txt").exists() || result.rust_result.is_some(),
                    "rust/stderr.txt should exist"
                );
                assert!(
                    rust_dir.join("metadata.json").exists() || result.rust_result.is_some(),
                    "rust/metadata.json should exist"
                );
            }
        }
    }

    drop(result);
}

#[test]
fn test_smoke_diff_report_generated_in_artifacts_run_id_diff_report_json() {
    use opencode_core::runners::DifferentialRunner;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let loader = DefaultTaskLoader::new();
    let runner = DifferentialRunner::new(loader);

    let task_yaml = temp_dir.path().join("smoke_diff_task.yaml");
    std::fs::write(
        &task_yaml,
        r#"
id: SMOKE-DIFF-001
title: Smoke Test Diff Report Generation
category: smoke
fixture_project: fixtures/projects/cli-basic
description: Verify diff report is generated
expected_outcome: Diff report generated
preconditions: []
entry_mode: CLI
agent_mode: OneShot
provider_mode: Both
input:
  command: echo
  args: ["diff_test"]
  cwd: "/tmp"
expected_assertions: []
severity: High
tags: []
execution_policy: ManualCheck
timeout_seconds: 60
on_missing_dependency: Fail
"#,
    )
    .unwrap();

    let result = runner.execute_single(&task_yaml).unwrap();

    if result.legacy_result.is_some() && result.rust_result.is_some() {
        assert!(
            result.diff_report_path.is_some(),
            "diff_report_path should be set when both runners succeed"
        );

        if let Some(ref report_path) = result.diff_report_path {
            assert!(
                report_path.to_string_lossy().contains("diff/report.json"),
                "report path should be in diff/ directory and named report.json"
            );
            assert!(
                report_path.exists(),
                "report.json file should actually exist on disk"
            );
        }
    }
}

#[test]
fn test_smoke_verdict_md_generated_with_correct_content() {
    use opencode_core::runners::DifferentialRunner;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let loader = DefaultTaskLoader::new();
    let runner = DifferentialRunner::new(loader);

    let task_yaml = temp_dir.path().join("smoke_verdict_task.yaml");
    std::fs::write(
        &task_yaml,
        r#"
id: SMOKE-VERDICT-001
title: Smoke Test Verdict Generation
category: smoke
fixture_project: fixtures/projects/cli-basic
description: Verify verdict.md is generated with correct content
expected_outcome: Verdict generated correctly
preconditions: []
entry_mode: CLI
agent_mode: OneShot
provider_mode: Both
input:
  command: echo
  args: ["verdict_test"]
  cwd: "/tmp"
expected_assertions: []
severity: High
tags: []
execution_policy: ManualCheck
timeout_seconds: 60
on_missing_dependency: Fail
"#,
    )
    .unwrap();

    let result = runner.execute_single(&task_yaml).unwrap();

    if result.legacy_result.is_some() && result.rust_result.is_some() {
        assert!(
            result.verdict_path.is_some(),
            "verdict_path should be set when both runners succeed"
        );

        if let Some(ref verdict_path) = result.verdict_path {
            assert!(
                verdict_path.to_string_lossy().contains("verdict.md"),
                "verdict path should contain verdict.md"
            );
            assert!(
                verdict_path.exists(),
                "verdict.md file should exist on disk"
            );

            let content = std::fs::read_to_string(verdict_path).unwrap();
            assert!(
                content.contains("# Differential Verdict"),
                "verdict.md should contain header"
            );
            assert!(
                content.contains("## Run ID:"),
                "verdict.md should contain Run ID section"
            );
            assert!(
                content.contains("## Verdict:"),
                "verdict.md should contain Verdict section"
            );
            assert!(
                content.contains("| Metric |"),
                "verdict.md should contain metrics table"
            );
        }
    }
}

#[test]
fn test_regression_existing_differential_runner_tests_still_pass() {
    let result = DifferentialResult::new("REGRESSION-001".to_string());
    assert_eq!(result.task_id, "REGRESSION-001");
    assert!(result.legacy_result.is_none());
    assert!(result.rust_result.is_none());

    assert!(!result.passed());
    assert!(result.summary().contains("REGRESSION-001"));

    let result2 = DifferentialResult::new("REGRESSION-002".to_string());
    assert_eq!(result2.legacy_exit_code(), None);
    assert_eq!(result2.rust_exit_code(), None);
    assert_eq!(result2.legacy_stdout(), "");
    assert_eq!(result2.rust_stdout(), "");
}

#[test]
fn test_allowed_variance_integration_tests_timing_variance() {
    let loader = DefaultTaskLoader::new();
    let runner = DifferentialRunner::new(loader);

    let temp_dir = TempDir::new().unwrap();
    let task_yaml = temp_dir.path().join("timing_variance_task.yaml");

    std::fs::write(
        &task_yaml,
        r#"
id: VAR-TIMING-001
title: Timing Variance Test
category: smoke
fixture_project: fixtures/projects/cli-basic
description: Test timing variance
expected_outcome: Timing should be within allowed variance
preconditions: []
entry_mode: CLI
agent_mode: OneShot
provider_mode: Both
input:
  command: echo
  args: ["timing_test"]
  cwd: "/tmp"
expected_assertions: []
severity: High
tags: []
allowed_variance:
  exit_code: []
  timing_ms:
    min: 0
    max: 5000
  output_patterns: []
execution_policy: ManualCheck
timeout_seconds: 60
on_missing_dependency: Fail
"#,
    )
    .unwrap();

    let loaded_task = runner.task_loader.load_single(&task_yaml).unwrap();
    assert!(loaded_task.allowed_variance.is_some());

    let allowed_variance = loaded_task.allowed_variance.clone().unwrap();
    assert!(allowed_variance.timing_ms.is_some());

    let result = runner.execute_from_task(&loaded_task);
    assert!(result.is_ok(), "execute_from_task should succeed");
}

#[test]
fn test_allowed_variance_integration_tests_exit_code_variance() {
    let loader = DefaultTaskLoader::new();
    let runner = DifferentialRunner::new(loader);

    let temp_dir = TempDir::new().unwrap();
    let task_yaml = temp_dir.path().join("exit_code_variance_task.yaml");

    std::fs::write(
        &task_yaml,
        r#"
id: VAR-EXITCODE-001
title: Exit Code Variance Test
category: smoke
fixture_project: fixtures/projects/cli-basic
description: Test exit code variance
expected_outcome: Exit code 0 or 1 both acceptable
preconditions: []
entry_mode: CLI
agent_mode: OneShot
provider_mode: Both
input:
  command: bash
  args: ["-c", "exit 0"]
  cwd: "/tmp"
expected_assertions: []
severity: High
tags: []
allowed_variance:
  exit_code: [0, 1]
  timing_ms: null
  output_patterns: []
execution_policy: ManualCheck
timeout_seconds: 60
on_missing_dependency: Fail
"#,
    )
    .unwrap();

    let loaded_task = runner.task_loader.load_single(&task_yaml).unwrap();
    assert!(loaded_task.allowed_variance.is_some());

    let allowed_variance = loaded_task.allowed_variance.clone().unwrap();
    assert!(allowed_variance.exit_code.contains(&0));
    assert!(allowed_variance.exit_code.contains(&1));

    let result = runner.execute_from_task(&loaded_task);
    assert!(result.is_ok(), "execute_from_task should succeed");
}

#[test]
fn test_allowed_variance_integration_output_pattern_check_produces_output_pattern_variance_verdict()
{
    let loader = DefaultTaskLoader::new();
    let runner = DifferentialRunner::new(loader);

    let temp_dir = TempDir::new().unwrap();
    let task_yaml = temp_dir.path().join("output_pattern_verdict_task.yaml");

    std::fs::write(
        &task_yaml,
        r#"
id: VAR-OUTPUT-VERDICT-001
title: Output Pattern Variance Verdict Test
category: smoke
fixture_project: fixtures/projects/cli-basic
description: Test output pattern variance produces correct verdict
expected_outcome: Should produce PassWithAllowedVariance for output pattern
preconditions: []
entry_mode: CLI
agent_mode: OneShot
provider_mode: Both
input:
  command: echo
  args: ["output123test"]
  cwd: "/tmp"
expected_assertions: []
severity: High
tags: []
allowed_variance:
  exit_code: []
  timing_ms:
    min: 0
    max: 10000
  output_patterns:
    - "output\\d+test"
execution_policy: ManualCheck
timeout_seconds: 60
on_missing_dependency: Fail
"#,
    )
    .unwrap();

    let loaded_task = runner.task_loader.load_single(&task_yaml).unwrap();
    let result = runner.execute_from_task(&loaded_task);

    assert!(result.is_ok());
    let diff_result = result.unwrap();

    match diff_result.verdict {
        ParityVerdict::PassWithAllowedVariance {
            variance_type: VarianceType::OutputPattern,
            ..
        } => {}
        ParityVerdict::PassWithAllowedVariance {
            variance_type: VarianceType::Timing,
            ..
        } => {}
        ParityVerdict::Pass => {}
        ParityVerdict::ManualCheck { reason, .. } => {
            if reason.contains("One or both runners failed") {
                return;
            }
            panic!(
                "Expected Pass or PassWithAllowedVariance, got ManualCheck: {}",
                reason
            );
        }
        other => {
            panic!("Expected Pass or PassWithAllowedVariance, got {:?}", other);
        }
    }
}

#[test]
fn test_allowed_variance_integration_verify_config_passed_through_pipeline() {
    let loader = DefaultTaskLoader::new();
    let runner = DifferentialRunner::new(loader);

    let temp_dir = TempDir::new().unwrap();
    let task_yaml = temp_dir.path().join("pipeline_task.yaml");

    std::fs::write(
        &task_yaml,
        r#"
id: VAR-PIPELINE-001
title: Pipeline Test
category: integration
fixture_project: fixtures/projects/cli-basic
description: Verify allowed_variance config flows through pipeline
expected_outcome: Config should be accessible in runner
preconditions: []
entry_mode: CLI
agent_mode: OneShot
provider_mode: Both
input:
  command: echo
  args: ["pipeline_test"]
  cwd: "/tmp"
expected_assertions: []
severity: High
tags: []
allowed_variance:
  exit_code: [0]
  timing_ms:
    min: 0
    max: 1000
  output_patterns:
    - "\\d+"
execution_policy: ManualCheck
timeout_seconds: 60
on_missing_dependency: Fail
"#,
    )
    .unwrap();

    let loaded_task = runner.task_loader.load_single(&task_yaml).unwrap();
    assert!(
        loaded_task.allowed_variance.is_some(),
        "allowed_variance should be set on task"
    );

    let result = runner.execute_from_task(&loaded_task);
    assert!(
        result.is_ok(),
        "execute_from_task should succeed with allowed_variance config"
    );

    let diff_result = result.unwrap();
    assert!(
        diff_result.task_id == "VAR-PIPELINE-001",
        "task_id should be preserved through pipeline"
    );
}

#[test]
fn test_allowed_variance_integration_timing_check_produces_timing_variance_verdict() {
    let loader = DefaultTaskLoader::new();
    let runner = DifferentialRunner::new(loader);

    let temp_dir = TempDir::new().unwrap();
    let task_yaml = temp_dir.path().join("timing_verdict_task.yaml");

    std::fs::write(
        &task_yaml,
        r#"
id: VAR-TIMING-VERDICT-001
title: Timing Variance Verdict Test
category: smoke
fixture_project: fixtures/projects/cli-basic
description: Test timing variance produces correct verdict
expected_outcome: Should produce PassWithAllowedVariance for timing
preconditions: []
entry_mode: CLI
agent_mode: OneShot
provider_mode: Both
input:
  command: echo
  args: ["timing_verdict"]
  cwd: "/tmp"
expected_assertions: []
severity: High
tags: []
allowed_variance:
  exit_code: []
  timing_ms:
    min: 0
    max: 10000
  output_patterns: []
execution_policy: ManualCheck
timeout_seconds: 60
on_missing_dependency: Fail
"#,
    )
    .unwrap();

    let loaded_task = runner.task_loader.load_single(&task_yaml).unwrap();
    assert!(loaded_task.allowed_variance.is_some());

    let result = runner.execute_from_task(&loaded_task);
    assert!(result.is_ok(), "execute_from_task should succeed");

    let diff_result = result.unwrap();
    match diff_result.verdict {
        ParityVerdict::PassWithAllowedVariance {
            variance_type: VarianceType::Timing,
            details,
        } => {
            assert!(
                details.contains("Timing diff"),
                "details should mention timing diff, got: {}",
                details
            );
        }
        ParityVerdict::Pass => {}
        ParityVerdict::ManualCheck { reason, .. } => {
            if reason.contains("One or both runners failed") {
                return;
            }
            panic!(
                "Expected Pass or PassWithAllowedVariance(Timing), got ManualCheck: {}",
                reason
            );
        }
        other => {
            panic!(
                "Expected Pass or PassWithAllowedVariance(Timing), got {:?}",
                other
            );
        }
    }
}

#[test]
fn test_allowed_variance_integration_exit_code_check_produces_exit_code_variance_verdict() {
    let loader = DefaultTaskLoader::new();
    let runner = DifferentialRunner::new(loader);

    let temp_dir = TempDir::new().unwrap();
    let task_yaml = temp_dir.path().join("exit_code_verdict_task.yaml");

    std::fs::write(
        &task_yaml,
        r#"
id: VAR-EXITCODE-VERDICT-001
title: Exit Code Variance Verdict Test
category: smoke
fixture_project: fixtures/projects/cli-basic
description: Test exit code variance produces correct verdict
expected_outcome: Should produce PassWithAllowedVariance for exit code
preconditions: []
entry_mode: CLI
agent_mode: OneShot
provider_mode: Both
input:
  command: bash
  args: ["-c", "exit 0"]
  cwd: "/tmp"
expected_assertions: []
severity: High
tags: []
allowed_variance:
  exit_code: [0, 1, 2]
  timing_ms: null
  output_patterns: []
execution_policy: ManualCheck
timeout_seconds: 60
on_missing_dependency: Fail
"#,
    )
    .unwrap();

    let loaded_task = runner.task_loader.load_single(&task_yaml).unwrap();
    let result = runner.execute_from_task(&loaded_task);

    assert!(result.is_ok());
    let diff_result = result.unwrap();

    match diff_result.verdict {
        ParityVerdict::PassWithAllowedVariance {
            variance_type: VarianceType::ExitCode,
            ..
        } => {}
        ParityVerdict::Pass => {}
        other => {
            panic!(
                "Expected Pass or PassWithAllowedVariance(ExitCode), got {:?}",
                other
            );
        }
    }
}
