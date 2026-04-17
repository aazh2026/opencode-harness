use crate::reporting::gate::GateLevel;
use crate::types::TaskCategory;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuiteName {
    PrSmoke,
    NightlyFull,
    ReleaseQualification,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactPolicy {
    Always,
    OnFailure,
    Never,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiteDefinition {
    pub name: SuiteName,
    pub description: String,
    pub included_task_categories: Vec<TaskCategory>,
    pub allowed_whitelists: bool,
    pub allow_skipped: bool,
    pub allow_manual_check: bool,
    pub artifact_retention_policy: ArtifactPolicy,
    pub gate_level: GateLevel,
}

impl SuiteDefinition {
    pub fn pr_smoke() -> Self {
        SuiteDefinition {
            name: SuiteName::PrSmoke,
            description: "PR smoke test suite - fast feedback for pull requests".to_string(),
            included_task_categories: vec![TaskCategory::Smoke],
            allowed_whitelists: true,
            allow_skipped: false,
            allow_manual_check: true,
            artifact_retention_policy: ArtifactPolicy::OnFailure,
            gate_level: GateLevel::PR,
        }
    }

    pub fn nightly_full() -> Self {
        SuiteDefinition {
            name: SuiteName::NightlyFull,
            description: "Nightly full suite - comprehensive testing".to_string(),
            included_task_categories: vec![TaskCategory::Smoke, TaskCategory::Regression],
            allowed_whitelists: true,
            allow_skipped: true,
            allow_manual_check: true,
            artifact_retention_policy: ArtifactPolicy::Always,
            gate_level: GateLevel::Nightly,
        }
    }

    pub fn release_qualification() -> Self {
        SuiteDefinition {
            name: SuiteName::ReleaseQualification,
            description: "Release qualification suite - strict pass required".to_string(),
            included_task_categories: vec![TaskCategory::Regression],
            allowed_whitelists: false,
            allow_skipped: false,
            allow_manual_check: false,
            artifact_retention_policy: ArtifactPolicy::Always,
            gate_level: GateLevel::Release,
        }
    }

    pub fn name_str(&self) -> &'static str {
        match self.name {
            SuiteName::PrSmoke => "pr-smoke",
            SuiteName::NightlyFull => "nightly-full",
            SuiteName::ReleaseQualification => "release-qualification",
        }
    }
}

pub trait SuiteSelector: Send + Sync {
    fn select_suite(&self, name: &str) -> Option<SuiteDefinition>;
    fn list_suites(&self) -> Vec<SuiteName>;
    fn filter_tasks(
        &self,
        suite: &SuiteDefinition,
        tasks: &[crate::types::Task],
    ) -> Vec<crate::types::Task>;
}

pub struct DefaultSuiteSelector {
    suites: Vec<SuiteDefinition>,
}

impl DefaultSuiteSelector {
    pub fn new() -> Self {
        Self {
            suites: vec![
                SuiteDefinition::pr_smoke(),
                SuiteDefinition::nightly_full(),
                SuiteDefinition::release_qualification(),
            ],
        }
    }
}

impl Default for DefaultSuiteSelector {
    fn default() -> Self {
        Self::new()
    }
}

impl SuiteSelector for DefaultSuiteSelector {
    fn select_suite(&self, name: &str) -> Option<SuiteDefinition> {
        self.suites.iter().find(|s| s.name_str() == name).cloned()
    }

    fn list_suites(&self) -> Vec<SuiteName> {
        self.suites.iter().map(|s| s.name.clone()).collect()
    }

    fn filter_tasks(
        &self,
        suite: &SuiteDefinition,
        tasks: &[crate::types::Task],
    ) -> Vec<crate::types::Task> {
        tasks
            .iter()
            .filter(|t| suite.included_task_categories.contains(&t.category))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suite_name_has_all_three_variants() {
        let variants = vec![
            SuiteName::PrSmoke,
            SuiteName::NightlyFull,
            SuiteName::ReleaseQualification,
        ];
        assert_eq!(variants.len(), 3);
    }

    #[test]
    fn test_suite_definition_pr_smoke() {
        let suite = SuiteDefinition::pr_smoke();
        assert_eq!(suite.name, SuiteName::PrSmoke);
        assert_eq!(suite.name_str(), "pr-smoke");
        assert!(suite.description.contains("PR smoke"));
        assert!(suite
            .included_task_categories
            .contains(&TaskCategory::Smoke));
        assert!(suite.allowed_whitelists);
        assert!(!suite.allow_skipped);
        assert!(suite.allow_manual_check);
        assert_eq!(suite.artifact_retention_policy, ArtifactPolicy::OnFailure);
        assert_eq!(suite.gate_level, GateLevel::PR);
    }

    #[test]
    fn test_suite_definition_nightly_full() {
        let suite = SuiteDefinition::nightly_full();
        assert_eq!(suite.name, SuiteName::NightlyFull);
        assert_eq!(suite.name_str(), "nightly-full");
        assert!(suite.description.contains("Nightly"));
        assert!(suite
            .included_task_categories
            .contains(&TaskCategory::Smoke));
        assert!(suite
            .included_task_categories
            .contains(&TaskCategory::Regression));
        assert!(suite.allowed_whitelists);
        assert!(suite.allow_skipped);
        assert!(suite.allow_manual_check);
        assert_eq!(suite.artifact_retention_policy, ArtifactPolicy::Always);
        assert_eq!(suite.gate_level, GateLevel::Nightly);
    }

    #[test]
    fn test_suite_definition_release_qualification() {
        let suite = SuiteDefinition::release_qualification();
        assert_eq!(suite.name, SuiteName::ReleaseQualification);
        assert_eq!(suite.name_str(), "release-qualification");
        assert!(suite.description.contains("Release qualification"));
        assert!(suite
            .included_task_categories
            .contains(&TaskCategory::Regression));
        assert!(!suite.allowed_whitelists);
        assert!(!suite.allow_skipped);
        assert!(!suite.allow_manual_check);
        assert_eq!(suite.artifact_retention_policy, ArtifactPolicy::Always);
        assert_eq!(suite.gate_level, GateLevel::Release);
    }

    #[test]
    fn test_default_suite_selector_select_suite() {
        let selector = DefaultSuiteSelector::new();
        let suite = selector.select_suite("pr-smoke");
        assert!(suite.is_some());
        assert_eq!(suite.unwrap().name, SuiteName::PrSmoke);

        let suite = selector.select_suite("nightly-full");
        assert!(suite.is_some());
        assert_eq!(suite.unwrap().name, SuiteName::NightlyFull);

        let suite = selector.select_suite("release-qualification");
        assert!(suite.is_some());
        assert_eq!(suite.unwrap().name, SuiteName::ReleaseQualification);

        let suite = selector.select_suite("nonexistent");
        assert!(suite.is_none());
    }

    #[test]
    fn test_default_suite_selector_list_suites() {
        let selector = DefaultSuiteSelector::new();
        let suites = selector.list_suites();
        assert_eq!(suites.len(), 3);
        assert!(suites.contains(&SuiteName::PrSmoke));
        assert!(suites.contains(&SuiteName::NightlyFull));
        assert!(suites.contains(&SuiteName::ReleaseQualification));
    }

    #[test]
    fn test_default_suite_selector_filter_tasks() {
        use crate::types::{
            AgentMode, EntryMode, ExecutionPolicy, OnMissingDependency, ProviderMode, Severity,
        };
        use crate::types::{Task, TaskInput};

        let selector = DefaultSuiteSelector::new();
        let suite = SuiteDefinition::pr_smoke();

        let tasks = vec![
            Task::new(
                "SMOKE-001",
                "Smoke Test 1",
                TaskCategory::Smoke,
                "fixtures/smoke",
                "Test description",
                "Expected outcome",
                vec![],
                EntryMode::CLI,
                AgentMode::OneShot,
                ProviderMode::Both,
                TaskInput::new("opencode", vec![], "/tmp"),
                vec![],
                Severity::High,
                ExecutionPolicy::ManualCheck,
                300,
                OnMissingDependency::Fail,
            ),
            Task::new(
                "SMOKE-002",
                "Smoke Test 2",
                TaskCategory::Smoke,
                "fixtures/smoke",
                "Test description",
                "Expected outcome",
                vec![],
                EntryMode::CLI,
                AgentMode::OneShot,
                ProviderMode::Both,
                TaskInput::new("opencode", vec![], "/tmp"),
                vec![],
                Severity::Medium,
                ExecutionPolicy::ManualCheck,
                300,
                OnMissingDependency::Fail,
            ),
            Task::new(
                "REGR-001",
                "Regression Test",
                TaskCategory::Regression,
                "fixtures/regression",
                "Test description",
                "Expected outcome",
                vec![],
                EntryMode::CLI,
                AgentMode::OneShot,
                ProviderMode::Both,
                TaskInput::new("opencode", vec![], "/tmp"),
                vec![],
                Severity::High,
                ExecutionPolicy::ManualCheck,
                300,
                OnMissingDependency::Fail,
            ),
        ];

        let filtered = selector.filter_tasks(&suite, &tasks);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|t| t.category == TaskCategory::Smoke));
    }

    #[test]
    fn test_suite_selector_trait_object() {
        let selector: Box<dyn SuiteSelector> = Box::new(DefaultSuiteSelector::new());
        let suite = selector.select_suite("pr-smoke");
        assert!(suite.is_some());
        let suites = selector.list_suites();
        assert_eq!(suites.len(), 3);
    }

    #[test]
    fn test_suite_name_serde() {
        let names = vec![
            (SuiteName::PrSmoke, "\"pr_smoke\""),
            (SuiteName::NightlyFull, "\"nightly_full\""),
            (SuiteName::ReleaseQualification, "\"release_qualification\""),
        ];
        for (name, expected) in names {
            let json = serde_json::to_string(&name).unwrap();
            assert_eq!(json, expected);
            let parsed: SuiteName = serde_json::from_str(expected).unwrap();
            assert_eq!(parsed, name);
        }
    }

    #[test]
    fn test_artifact_policy_serde() {
        let policies = vec![
            (ArtifactPolicy::Always, "\"always\""),
            (ArtifactPolicy::OnFailure, "\"on_failure\""),
            (ArtifactPolicy::Never, "\"never\""),
        ];
        for (policy, expected) in policies {
            let json = serde_json::to_string(&policy).unwrap();
            assert_eq!(json, expected);
            let parsed: ArtifactPolicy = serde_json::from_str(expected).unwrap();
            assert_eq!(parsed, policy);
        }
    }

    #[test]
    fn test_suite_definition_clone() {
        let suite = SuiteDefinition::pr_smoke();
        let cloned = suite.clone();
        assert_eq!(suite.name, cloned.name);
        assert_eq!(suite.description, cloned.description);
    }

    #[test]
    fn test_suite_definition_debug() {
        let suite = SuiteDefinition::pr_smoke();
        let debug = format!("{:?}", suite);
        assert!(debug.contains("PrSmoke"));
        assert!(debug.contains("OnFailure"));
    }

    #[test]
    fn test_default_suite_selector_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<DefaultSuiteSelector>();
    }
}
