use opencode_core::reporting::gate::GateLevel;
use opencode_core::reporting::suite::{
    ArtifactPolicy, DefaultSuiteSelector, SuiteDefinition, SuiteName, SuiteSelector,
};
use opencode_core::types::task::TaskCategory;
use opencode_core::types::{
    AgentMode, EntryMode, ExecutionPolicy, OnMissingDependency, ProviderMode, Severity,
    Task, TaskInput,
};

fn create_test_task(
    id: &str,
    category: TaskCategory,
    execution_policy: ExecutionPolicy,
) -> Task {
    Task::new(
        id,
        format!("Test Task {}", id),
        category,
        "fixtures/test",
        "Test description",
        "Expected outcome",
        vec![],
        EntryMode::CLI,
        AgentMode::OneShot,
        ProviderMode::Both,
        TaskInput::new("opencode", vec![], "/tmp"),
        vec![],
        Severity::Medium,
        execution_policy,
        300,
        OnMissingDependency::Fail,
    )
}

#[test]
fn test_pr_smoke_suite_configuration() {
    let suite = SuiteDefinition::pr_smoke();

    assert_eq!(suite.name, SuiteName::PrSmoke, "pr_smoke should have PrSmoke name");
    assert_eq!(suite.name_str(), "pr-smoke", "pr_smoke should have string name 'pr-smoke'");
    assert!(
        suite.description.contains("PR smoke"),
        "pr_smoke description should mention PR smoke"
    );

    assert_eq!(
        suite.gate_level, GateLevel::PR,
        "pr_smoke should have GateLevel::PR"
    );

    assert!(
        suite.included_task_categories.contains(&TaskCategory::Smoke),
        "pr_smoke should include Smoke category"
    );

    assert!(
        suite.allowed_whitelists,
        "pr_smoke should allow whitelists (allowed_whitelists=true)"
    );
    assert!(
        !suite.allow_skipped,
        "pr_smoke should NOT allow skipped (allow_skipped=false)"
    );
    assert!(
        suite.allow_manual_check,
        "pr_smoke should allow manual check (allow_manual_check=true)"
    );

    assert_eq!(
        suite.artifact_retention_policy, ArtifactPolicy::OnFailure,
        "pr_smoke should have OnFailure artifact retention"
    );
}

#[test]
fn test_nightly_full_suite_configuration() {
    let suite = SuiteDefinition::nightly_full();

    assert_eq!(
        suite.name, SuiteName::NightlyFull,
        "nightly_full should have NightlyFull name"
    );
    assert_eq!(
        suite.name_str(), "nightly-full",
        "nightly_full should have string name 'nightly-full'"
    );
    assert!(
        suite.description.contains("Nightly"),
        "nightly_full description should mention Nightly"
    );

    assert_eq!(
        suite.gate_level, GateLevel::Nightly,
        "nightly_full should have GateLevel::Nightly"
    );

    assert!(
        suite.included_task_categories.contains(&TaskCategory::Smoke),
        "nightly_full should include Smoke category"
    );
    assert!(
        suite.included_task_categories.contains(&TaskCategory::Regression),
        "nightly_full should include Regression category"
    );

    assert!(
        suite.allowed_whitelists,
        "nightly_full should allow whitelists (allowed_whitelists=true)"
    );
    assert!(
        suite.allow_skipped,
        "nightly_full should allow skipped (allow_skipped=true)"
    );
    assert!(
        suite.allow_manual_check,
        "nightly_full should allow manual check (allow_manual_check=true)"
    );

    assert_eq!(
        suite.artifact_retention_policy, ArtifactPolicy::Always,
        "nightly_full should have Always artifact retention"
    );
}

#[test]
fn test_release_qualification_suite_configuration() {
    let suite = SuiteDefinition::release_qualification();

    assert_eq!(
        suite.name, SuiteName::ReleaseQualification,
        "release_qualification should have ReleaseQualification name"
    );
    assert_eq!(
        suite.name_str(), "release-qualification",
        "release_qualification should have string name 'release-qualification'"
    );
    assert!(
        suite.description.contains("Release qualification"),
        "release_qualification description should mention Release qualification"
    );

    assert_eq!(
        suite.gate_level, GateLevel::Release,
        "release_qualification should have GateLevel::Release"
    );

    assert!(
        suite.included_task_categories.contains(&TaskCategory::Regression),
        "release_qualification should include Regression category"
    );

    assert!(
        !suite.allowed_whitelists,
        "release_qualification should NOT allow whitelists (allowed_whitelists=false)"
    );
    assert!(
        !suite.allow_skipped,
        "release_qualification should NOT allow skipped (allow_skipped=false)"
    );
    assert!(
        !suite.allow_manual_check,
        "release_qualification should NOT allow manual check (allow_manual_check=false)"
    );

    assert_eq!(
        suite.artifact_retention_policy, ArtifactPolicy::Always,
        "release_qualification should have Always artifact retention"
    );
}

#[test]
fn test_suite_selector_select_suite() {
    let selector = DefaultSuiteSelector::new();

    let pr_suite = selector.select_suite("pr-smoke");
    assert!(
        pr_suite.is_some(),
        "select_suite should return Some for 'pr-smoke'"
    );
    assert_eq!(
        pr_suite.unwrap().name, SuiteName::PrSmoke,
        "select_suite('pr-smoke') should return PrSmoke suite"
    );

    let nightly_suite = selector.select_suite("nightly-full");
    assert!(
        nightly_suite.is_some(),
        "select_suite should return Some for 'nightly-full'"
    );
    assert_eq!(
        nightly_suite.unwrap().name, SuiteName::NightlyFull,
        "select_suite('nightly-full') should return NightlyFull suite"
    );

    let release_suite = selector.select_suite("release-qualification");
    assert!(
        release_suite.is_some(),
        "select_suite should return Some for 'release-qualification'"
    );
    assert_eq!(
        release_suite.unwrap().name, SuiteName::ReleaseQualification,
        "select_suite('release-qualification') should return ReleaseQualification suite"
    );

    let nonexistent = selector.select_suite("nonexistent");
    assert!(
        nonexistent.is_none(),
        "select_suite should return None for unknown suite name"
    );
}

#[test]
fn test_suite_selector_list_suites() {
    let selector = DefaultSuiteSelector::new();
    let suites = selector.list_suites();

    assert_eq!(suites.len(), 3, "list_suites should return exactly 3 suites");
    assert!(
        suites.contains(&SuiteName::PrSmoke),
        "list_suites should contain PrSmoke"
    );
    assert!(
        suites.contains(&SuiteName::NightlyFull),
        "list_suites should contain NightlyFull"
    );
    assert!(
        suites.contains(&SuiteName::ReleaseQualification),
        "list_suites should contain ReleaseQualification"
    );
}

#[test]
fn test_filter_tasks_by_suite() {
    let selector = DefaultSuiteSelector::new();

    let smoke_task = create_test_task("SMOKE-001", TaskCategory::Smoke, ExecutionPolicy::Blocked);
    let regression_task = create_test_task(
        "REGR-001",
        TaskCategory::Regression,
        ExecutionPolicy::Blocked,
    );
    let core_task = create_test_task("CORE-001", TaskCategory::Core, ExecutionPolicy::Blocked);

    let all_tasks = vec![smoke_task.clone(), regression_task.clone(), core_task.clone()];

    let pr_suite = SuiteDefinition::pr_smoke();
    let filtered = selector.filter_tasks(&pr_suite, &all_tasks);
    assert_eq!(
        filtered.len(), 1,
        "pr_smoke filter should return only Smoke tasks"
    );
    assert_eq!(
        filtered[0].id, "SMOKE-001",
        "pr_smoke filter should return SMOKE-001"
    );

    let nightly_suite = SuiteDefinition::nightly_full();
    let filtered = selector.filter_tasks(&nightly_suite, &all_tasks);
    assert_eq!(
        filtered.len(), 2,
        "nightly_full filter should return Smoke and Regression tasks"
    );
    assert!(
        filtered.iter().any(|t| t.id == "SMOKE-001"),
        "nightly_full filter should include SMOKE-001"
    );
    assert!(
        filtered.iter().any(|t| t.id == "REGR-001"),
        "nightly_full filter should include REGR-001"
    );

    let release_suite = SuiteDefinition::release_qualification();
    let filtered = selector.filter_tasks(&release_suite, &all_tasks);
    assert_eq!(
        filtered.len(), 1,
        "release_qualification filter should return only Regression tasks"
    );
    assert_eq!(
        filtered[0].id, "REGR-001",
        "release_qualification filter should return REGR-001"
    );
}

#[test]
fn test_whitelist_permission_enforcement() {
    let pr_suite = SuiteDefinition::pr_smoke();
    let release_suite = SuiteDefinition::release_qualification();

    assert!(
        pr_suite.allowed_whitelists,
        "pr_smoke suite should allow whitelisted tasks"
    );

    assert!(
        !release_suite.allowed_whitelists,
        "release_qualification suite should NOT allow whitelisted tasks"
    );

    let nightly_suite = SuiteDefinition::nightly_full();
    assert!(
        nightly_suite.allowed_whitelists,
        "nightly_full suite should allow whitelisted tasks"
    );
}

#[test]
fn test_skipped_permission_enforcement() {
    let pr_suite = SuiteDefinition::pr_smoke();
    let nightly_suite = SuiteDefinition::nightly_full();
    let release_suite = SuiteDefinition::release_qualification();

    assert!(
        !pr_suite.allow_skipped,
        "pr_smoke suite should NOT allow skipped tasks"
    );

    assert!(
        nightly_suite.allow_skipped,
        "nightly_full suite should allow skipped tasks"
    );

    assert!(
        !release_suite.allow_skipped,
        "release_qualification suite should NOT allow skipped tasks"
    );
}

#[test]
fn test_manual_check_permission_enforcement() {
    let pr_suite = SuiteDefinition::pr_smoke();
    let nightly_suite = SuiteDefinition::nightly_full();
    let release_suite = SuiteDefinition::release_qualification();

    assert!(
        pr_suite.allow_manual_check,
        "pr_smoke suite should allow manual check tasks"
    );

    assert!(
        nightly_suite.allow_manual_check,
        "nightly_full suite should allow manual check tasks"
    );

    assert!(
        !release_suite.allow_manual_check,
        "release_qualification suite should NOT allow manual check tasks"
    );
}