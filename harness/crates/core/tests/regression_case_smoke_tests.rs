use chrono::Utc;
use opencode_core::loaders::regression_loader::{DefaultRegressionLoader, RegressionLoader};
use opencode_core::types::execution_level::ExecutionLevel;
use opencode_core::types::regression_case::{RegressionCase, RegressionStatus};
use opencode_core::types::severity::Severity;
use tempfile::TempDir;

fn create_test_regression_case(
    id: &str,
    task_id: &str,
    status: RegressionStatus,
    execution_level: ExecutionLevel,
    severity: Severity,
) -> RegressionCase {
    let now = Utc::now();
    RegressionCase::new(
        id.to_string(),
        format!("https://github.com/example/repo/issues/{}", &id[4..]),
        format!("Background for {}", id),
        format!("Root cause for {}", id),
        format!("fixtures/regression/{}", id.to_lowercase()),
        task_id.to_string(),
        format!("Expected behavior for {}", id),
        severity,
        execution_level,
        status,
        now,
        now,
    )
}

#[test]
fn regression_case_smoke_tests_create_with_all_fields() {
    let now = Utc::now();
    let case = RegressionCase::new(
        "REG-TEST-001".to_string(),
        "https://github.com/example/repo/issues/999".to_string(),
        "Background description with full context".to_string(),
        "Root cause summary of the issue".to_string(),
        "fixtures/regression/test-001".to_string(),
        "TASK-012".to_string(),
        "Expected behavior description".to_string(),
        Severity::Critical,
        ExecutionLevel::AlwaysOn,
        RegressionStatus::Candidate,
        now,
        now,
    );

    assert_eq!(case.id, "REG-TEST-001");
    assert_eq!(
        case.issue_link,
        "https://github.com/example/repo/issues/999"
    );
    assert_eq!(case.background, "Background description with full context");
    assert_eq!(case.root_cause, "Root cause summary of the issue");
    assert_eq!(case.minimal_fixture, "fixtures/regression/test-001");
    assert_eq!(case.task_id, "TASK-012");
    assert_eq!(case.expected_result, "Expected behavior description");
    assert_eq!(case.severity, Severity::Critical);
    assert_eq!(case.execution_level, ExecutionLevel::AlwaysOn);
    assert_eq!(case.status, RegressionStatus::Candidate);
    assert_eq!(case.created_at, now);
    assert_eq!(case.updated_at, now);
}

#[test]
fn regression_case_smoke_tests_builder_pattern() {
    let now = Utc::now();
    let case = RegressionCase::default()
        .with_id("REG-TEST-002".to_string())
        .with_issue_link("https://github.com/example/repo/issues/888".to_string())
        .with_background("Builder background".to_string())
        .with_root_cause("Builder root cause".to_string())
        .with_minimal_fixture("fixtures/builder/test".to_string())
        .with_task_id("TASK-BUILD".to_string())
        .with_expected_result("Builder expected".to_string())
        .with_severity(Severity::High)
        .with_execution_level(ExecutionLevel::NightlyOnly)
        .with_status(RegressionStatus::Active)
        .with_created_at(now)
        .with_updated_at(now);

    assert_eq!(case.id, "REG-TEST-002");
    assert_eq!(case.severity, Severity::High);
    assert_eq!(case.execution_level, ExecutionLevel::NightlyOnly);
    assert_eq!(case.status, RegressionStatus::Active);
    assert_eq!(case.task_id, "TASK-BUILD");
}

#[test]
fn regression_case_smoke_tests_yaml_roundtrip() {
    let now = Utc::now();
    let original = RegressionCase::new(
        "REG-YAML-001".to_string(),
        "https://github.com/example/repo/issues/777".to_string(),
        "YAML background".to_string(),
        "YAML root cause".to_string(),
        "fixtures/regression/yaml-001".to_string(),
        "TASK-YAML".to_string(),
        "YAML expected".to_string(),
        Severity::Medium,
        ExecutionLevel::ReleaseOnly,
        RegressionStatus::Approved,
        now,
        now,
    );

    let yaml = serde_yaml::to_string(&original).expect("serialization should succeed");
    let deserialized: RegressionCase =
        serde_yaml::from_str(&yaml).expect("deserialization should succeed");

    assert_eq!(original.id, deserialized.id);
    assert_eq!(original.issue_link, deserialized.issue_link);
    assert_eq!(original.background, deserialized.background);
    assert_eq!(original.root_cause, deserialized.root_cause);
    assert_eq!(original.minimal_fixture, deserialized.minimal_fixture);
    assert_eq!(original.task_id, deserialized.task_id);
    assert_eq!(original.expected_result, deserialized.expected_result);
    assert_eq!(original.severity, deserialized.severity);
    assert_eq!(original.execution_level, deserialized.execution_level);
    assert_eq!(original.status, deserialized.status);
}

#[test]
fn regression_case_smoke_tests_json_roundtrip() {
    let now = Utc::now();
    let original = RegressionCase::new(
        "REG-JSON-001".to_string(),
        "https://github.com/example/repo/issues/666".to_string(),
        "JSON background".to_string(),
        "JSON root cause".to_string(),
        "fixtures/regression/json-001".to_string(),
        "TASK-JSON".to_string(),
        "JSON expected".to_string(),
        Severity::Low,
        ExecutionLevel::AlwaysOn,
        RegressionStatus::Candidate,
        now,
        now,
    );

    let json = serde_json::to_string(&original).expect("JSON serialization should succeed");
    let deserialized: RegressionCase =
        serde_json::from_str(&json).expect("JSON deserialization should succeed");

    assert_eq!(original.id, deserialized.id);
    assert_eq!(original.status, deserialized.status);
    assert_eq!(original.severity, deserialized.severity);
    assert_eq!(original.execution_level, deserialized.execution_level);
}

#[test]
fn regression_case_smoke_tests_loader_save_load_roundtrip() {
    let temp_dir = TempDir::new().unwrap();
    let loader = DefaultRegressionLoader::new(temp_dir.path().to_path_buf());

    let original = create_test_regression_case(
        "REG-LOAD-001",
        "TASK-LOAD",
        RegressionStatus::Candidate,
        ExecutionLevel::AlwaysOn,
        Severity::High,
    );

    loader.save(&original).unwrap();

    let loaded = loader.load("candidates", "REG-LOAD-001").unwrap();
    assert!(loaded.is_some());

    let loaded = loaded.unwrap();
    assert_eq!(loaded.id, original.id);
    assert_eq!(loaded.issue_link, original.issue_link);
    assert_eq!(loaded.background, original.background);
    assert_eq!(loaded.root_cause, original.root_cause);
    assert_eq!(loaded.minimal_fixture, original.minimal_fixture);
    assert_eq!(loaded.task_id, original.task_id);
    assert_eq!(loaded.expected_result, original.expected_result);
    assert_eq!(loaded.severity, original.severity);
    assert_eq!(loaded.execution_level, original.execution_level);
    assert_eq!(loaded.status, original.status);
}

#[test]
fn regression_case_smoke_tests_status_transitions_candidate_to_approved() {
    let temp_dir = TempDir::new().unwrap();
    let loader = DefaultRegressionLoader::new(temp_dir.path().to_path_buf());

    let case = create_test_regression_case(
        "REG-STATUS-001",
        "TASK-STATUS",
        RegressionStatus::Candidate,
        ExecutionLevel::AlwaysOn,
        Severity::High,
    );
    loader.save(&case).unwrap();

    loader
        .update_status("REG-STATUS-001", RegressionStatus::Approved)
        .unwrap();

    let old_location = loader.load("candidates", "REG-STATUS-001").unwrap();
    assert!(old_location.is_none());

    let new_location = loader.load("approved", "REG-STATUS-001").unwrap();
    assert!(new_location.is_some());
    assert_eq!(new_location.unwrap().status, RegressionStatus::Approved);
}

#[test]
fn regression_case_smoke_tests_status_transitions_approved_to_active() {
    let temp_dir = TempDir::new().unwrap();
    let loader = DefaultRegressionLoader::new(temp_dir.path().to_path_buf());

    let case = create_test_regression_case(
        "REG-STATUS-002",
        "TASK-STATUS",
        RegressionStatus::Approved,
        ExecutionLevel::AlwaysOn,
        Severity::Critical,
    );
    loader.save(&case).unwrap();

    loader
        .update_status("REG-STATUS-002", RegressionStatus::Active)
        .unwrap();

    let approved_location = loader.load("approved", "REG-STATUS-002").unwrap();
    assert!(approved_location.is_none());

    let active_location = loader.load("bugs", "REG-STATUS-002").unwrap();
    assert!(active_location.is_some());
    assert_eq!(active_location.unwrap().status, RegressionStatus::Active);
}

#[test]
fn regression_case_smoke_tests_status_transitions_active_to_suppressed() {
    let temp_dir = TempDir::new().unwrap();
    let loader = DefaultRegressionLoader::new(temp_dir.path().to_path_buf());

    let case = create_test_regression_case(
        "REG-STATUS-003",
        "TASK-STATUS",
        RegressionStatus::Active,
        ExecutionLevel::NightlyOnly,
        Severity::Medium,
    );
    loader.save(&case).unwrap();

    loader
        .update_status("REG-STATUS-003", RegressionStatus::Suppressed)
        .unwrap();

    let bugs_location = loader.load("bugs", "REG-STATUS-003").unwrap();
    assert!(bugs_location.is_none());

    let suppressed_location = loader.load("suppressed", "REG-STATUS-003").unwrap();
    assert!(suppressed_location.is_some());
    assert_eq!(
        suppressed_location.unwrap().status,
        RegressionStatus::Suppressed
    );
}

#[test]
fn regression_case_smoke_tests_status_transitions_active_to_resolved() {
    let temp_dir = TempDir::new().unwrap();
    let loader = DefaultRegressionLoader::new(temp_dir.path().to_path_buf());

    let case = create_test_regression_case(
        "REG-STATUS-004",
        "TASK-STATUS",
        RegressionStatus::Active,
        ExecutionLevel::ReleaseOnly,
        Severity::Low,
    );
    loader.save(&case).unwrap();

    loader
        .update_status("REG-STATUS-004", RegressionStatus::Resolved)
        .unwrap();

    let bugs_location = loader.load("bugs", "REG-STATUS-004").unwrap();
    assert!(bugs_location.is_none());

    let resolved_location = loader.load("resolved", "REG-STATUS-004").unwrap();
    assert!(resolved_location.is_some());
    assert_eq!(
        resolved_location.unwrap().status,
        RegressionStatus::Resolved
    );
}

#[test]
fn regression_case_smoke_tests_status_update_error_for_nonexistent() {
    let temp_dir = TempDir::new().unwrap();
    let loader = DefaultRegressionLoader::new(temp_dir.path().to_path_buf());

    let result = loader.update_status("NONEXISTENT", RegressionStatus::Active);
    assert!(result.is_err());
}

#[test]
fn regression_case_smoke_tests_execution_level_filtering_always_on() {
    let temp_dir = TempDir::new().unwrap();
    let loader = DefaultRegressionLoader::new(temp_dir.path().to_path_buf());

    let case1 = create_test_regression_case(
        "REG-EXEC-001",
        "TASK-EXEC",
        RegressionStatus::Active,
        ExecutionLevel::AlwaysOn,
        Severity::High,
    );
    let case2 = create_test_regression_case(
        "REG-EXEC-002",
        "TASK-EXEC",
        RegressionStatus::Candidate,
        ExecutionLevel::NightlyOnly,
        Severity::Medium,
    );
    let case3 = create_test_regression_case(
        "REG-EXEC-003",
        "TASK-EXEC",
        RegressionStatus::Approved,
        ExecutionLevel::AlwaysOn,
        Severity::Low,
    );

    loader.save(&case1).unwrap();
    loader.save(&case2).unwrap();
    loader.save(&case3).unwrap();

    let always_on_cases = loader
        .load_by_execution_level(ExecutionLevel::AlwaysOn)
        .unwrap();
    assert_eq!(always_on_cases.len(), 2);
    assert!(always_on_cases
        .iter()
        .all(|c| c.execution_level == ExecutionLevel::AlwaysOn));
}

#[test]
fn regression_case_smoke_tests_execution_level_filtering_nightly_only() {
    let temp_dir = TempDir::new().unwrap();
    let loader = DefaultRegressionLoader::new(temp_dir.path().to_path_buf());

    let case1 = create_test_regression_case(
        "REG-NIGHT-001",
        "TASK-NIGHT",
        RegressionStatus::Active,
        ExecutionLevel::NightlyOnly,
        Severity::High,
    );
    let case2 = create_test_regression_case(
        "REG-NIGHT-002",
        "TASK-NIGHT",
        RegressionStatus::Candidate,
        ExecutionLevel::AlwaysOn,
        Severity::Medium,
    );
    let case3 = create_test_regression_case(
        "REG-NIGHT-003",
        "TASK-NIGHT",
        RegressionStatus::Approved,
        ExecutionLevel::NightlyOnly,
        Severity::Low,
    );

    loader.save(&case1).unwrap();
    loader.save(&case2).unwrap();
    loader.save(&case3).unwrap();

    let nightly_cases = loader
        .load_by_execution_level(ExecutionLevel::NightlyOnly)
        .unwrap();
    assert_eq!(nightly_cases.len(), 2);
    assert!(nightly_cases
        .iter()
        .all(|c| c.execution_level == ExecutionLevel::NightlyOnly));
}

#[test]
fn regression_case_smoke_tests_execution_level_filtering_release_only() {
    let temp_dir = TempDir::new().unwrap();
    let loader = DefaultRegressionLoader::new(temp_dir.path().to_path_buf());

    let case1 = create_test_regression_case(
        "REG-RELEASE-001",
        "TASK-RELEASE",
        RegressionStatus::Active,
        ExecutionLevel::ReleaseOnly,
        Severity::Critical,
    );
    let case2 = create_test_regression_case(
        "REG-RELEASE-002",
        "TASK-RELEASE",
        RegressionStatus::Candidate,
        ExecutionLevel::AlwaysOn,
        Severity::High,
    );

    loader.save(&case1).unwrap();
    loader.save(&case2).unwrap();

    let release_cases = loader
        .load_by_execution_level(ExecutionLevel::ReleaseOnly)
        .unwrap();
    assert_eq!(release_cases.len(), 1);
    assert_eq!(release_cases[0].id, "REG-RELEASE-001");
    assert_eq!(
        release_cases[0].execution_level,
        ExecutionLevel::ReleaseOnly
    );
}

#[test]
fn regression_case_smoke_tests_status_filtering() {
    let temp_dir = TempDir::new().unwrap();
    let loader = DefaultRegressionLoader::new(temp_dir.path().to_path_buf());

    let case1 = create_test_regression_case(
        "REG-FILT-001",
        "TASK-FILT",
        RegressionStatus::Active,
        ExecutionLevel::AlwaysOn,
        Severity::High,
    );
    let case2 = create_test_regression_case(
        "REG-FILT-002",
        "TASK-FILT",
        RegressionStatus::Candidate,
        ExecutionLevel::AlwaysOn,
        Severity::Medium,
    );
    let case3 = create_test_regression_case(
        "REG-FILT-003",
        "TASK-FILT",
        RegressionStatus::Candidate,
        ExecutionLevel::AlwaysOn,
        Severity::Low,
    );
    let case4 = create_test_regression_case(
        "REG-FILT-004",
        "TASK-FILT",
        RegressionStatus::Approved,
        ExecutionLevel::NightlyOnly,
        Severity::Critical,
    );

    loader.save(&case1).unwrap();
    loader.save(&case2).unwrap();
    loader.save(&case3).unwrap();
    loader.save(&case4).unwrap();

    let candidate_cases = loader.load_by_status(RegressionStatus::Candidate).unwrap();
    assert_eq!(candidate_cases.len(), 2);

    let active_cases = loader.load_by_status(RegressionStatus::Active).unwrap();
    assert_eq!(active_cases.len(), 1);
    assert_eq!(active_cases[0].id, "REG-FILT-001");

    let approved_cases = loader.load_by_status(RegressionStatus::Approved).unwrap();
    assert_eq!(approved_cases.len(), 1);
    assert_eq!(approved_cases[0].id, "REG-FILT-004");

    let suppressed_cases = loader.load_by_status(RegressionStatus::Suppressed).unwrap();
    assert!(suppressed_cases.is_empty());
}

#[test]
fn regression_case_smoke_tests_load_all() {
    let temp_dir = TempDir::new().unwrap();
    let loader = DefaultRegressionLoader::new(temp_dir.path().to_path_buf());

    let case1 = create_test_regression_case(
        "REG-ALL-001",
        "TASK-ALL",
        RegressionStatus::Active,
        ExecutionLevel::AlwaysOn,
        Severity::High,
    );
    let case2 = create_test_regression_case(
        "REG-ALL-002",
        "TASK-ALL",
        RegressionStatus::Candidate,
        ExecutionLevel::NightlyOnly,
        Severity::Medium,
    );
    let case3 = create_test_regression_case(
        "REG-ALL-003",
        "TASK-ALL",
        RegressionStatus::Approved,
        ExecutionLevel::ReleaseOnly,
        Severity::Low,
    );

    loader.save(&case1).unwrap();
    loader.save(&case2).unwrap();
    loader.save(&case3).unwrap();

    let all_cases = loader.load_all().unwrap();
    assert_eq!(all_cases.len(), 3);
}

#[test]
fn regression_case_smoke_tests_load_nonexistent() {
    let temp_dir = TempDir::new().unwrap();
    let loader = DefaultRegressionLoader::new(temp_dir.path().to_path_buf());

    let loaded = loader.load("candidates", "DOES-NOT-EXIST").unwrap();
    assert!(loaded.is_none());
}

#[test]
fn regression_case_smoke_tests_trait_object() {
    let temp_dir = TempDir::new().unwrap();
    let loader: Box<dyn RegressionLoader> =
        Box::new(DefaultRegressionLoader::new(temp_dir.path().to_path_buf()));

    let case = create_test_regression_case(
        "REG-TRAIT-001",
        "TASK-TRAIT",
        RegressionStatus::Candidate,
        ExecutionLevel::AlwaysOn,
        Severity::High,
    );
    loader.save(&case).unwrap();

    let loaded = loader.load("candidates", "REG-TRAIT-001").unwrap();
    assert!(loaded.is_some());
    assert_eq!(loaded.unwrap().id, "REG-TRAIT-001");
}

#[test]
fn regression_case_smoke_tests_is_active_helper() {
    let now = Utc::now();
    let active_case = RegressionCase::new(
        "REG-HELPER-001".to_string(),
        "https://example.com/1".to_string(),
        "bg".to_string(),
        "rc".to_string(),
        "fix".to_string(),
        "TASK".to_string(),
        "exp".to_string(),
        Severity::High,
        ExecutionLevel::AlwaysOn,
        RegressionStatus::Active,
        now,
        now,
    );
    assert!(active_case.is_active());
    assert!(!active_case.is_candidate());
    assert!(!active_case.is_suppressed());
    assert!(!active_case.is_resolved());
}

#[test]
fn regression_case_smoke_tests_is_candidate_helper() {
    let now = Utc::now();
    let candidate_case = RegressionCase::new(
        "REG-HELPER-002".to_string(),
        "https://example.com/2".to_string(),
        "bg".to_string(),
        "rc".to_string(),
        "fix".to_string(),
        "TASK".to_string(),
        "exp".to_string(),
        Severity::Medium,
        ExecutionLevel::NightlyOnly,
        RegressionStatus::Candidate,
        now,
        now,
    );
    assert!(!candidate_case.is_active());
    assert!(candidate_case.is_candidate());
    assert!(!candidate_case.is_suppressed());
    assert!(!candidate_case.is_resolved());
}

#[test]
fn regression_case_smoke_tests_is_suppressed_helper() {
    let now = Utc::now();
    let suppressed_case = RegressionCase::new(
        "REG-HELPER-003".to_string(),
        "https://example.com/3".to_string(),
        "bg".to_string(),
        "rc".to_string(),
        "fix".to_string(),
        "TASK".to_string(),
        "exp".to_string(),
        Severity::Low,
        ExecutionLevel::ReleaseOnly,
        RegressionStatus::Suppressed,
        now,
        now,
    );
    assert!(!suppressed_case.is_active());
    assert!(!suppressed_case.is_candidate());
    assert!(suppressed_case.is_suppressed());
    assert!(!suppressed_case.is_resolved());
}

#[test]
fn regression_case_smoke_tests_is_resolved_helper() {
    let now = Utc::now();
    let resolved_case = RegressionCase::new(
        "REG-HELPER-004".to_string(),
        "https://example.com/4".to_string(),
        "bg".to_string(),
        "rc".to_string(),
        "fix".to_string(),
        "TASK".to_string(),
        "exp".to_string(),
        Severity::Cosmetic,
        ExecutionLevel::AlwaysOn,
        RegressionStatus::Resolved,
        now,
        now,
    );
    assert!(!resolved_case.is_active());
    assert!(!resolved_case.is_candidate());
    assert!(!resolved_case.is_suppressed());
    assert!(resolved_case.is_resolved());
}

#[test]
fn regression_case_smoke_tests_default_values() {
    let case = RegressionCase::default();

    assert!(case.id.is_empty());
    assert!(case.issue_link.is_empty());
    assert!(case.background.is_empty());
    assert!(case.root_cause.is_empty());
    assert!(case.minimal_fixture.is_empty());
    assert!(case.task_id.is_empty());
    assert!(case.expected_result.is_empty());
    assert_eq!(case.severity, Severity::Medium);
    assert_eq!(case.execution_level, ExecutionLevel::AlwaysOn);
    assert_eq!(case.status, RegressionStatus::Candidate);
}
