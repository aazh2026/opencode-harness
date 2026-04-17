use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::execution_level::ExecutionLevel;
use crate::types::severity::Severity;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegressionStatus {
    Candidate,
    Approved,
    Active,
    Suppressed,
    Resolved,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionCase {
    pub id: String,
    pub issue_link: String,
    pub background: String,
    pub root_cause: String,
    pub minimal_fixture: String,
    pub task_id: String,
    pub expected_result: String,
    pub severity: Severity,
    pub execution_level: ExecutionLevel,
    pub status: RegressionStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RegressionCase {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        issue_link: String,
        background: String,
        root_cause: String,
        minimal_fixture: String,
        task_id: String,
        expected_result: String,
        severity: Severity,
        execution_level: ExecutionLevel,
        status: RegressionStatus,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            issue_link,
            background,
            root_cause,
            minimal_fixture,
            task_id,
            expected_result,
            severity,
            execution_level,
            status,
            created_at,
            updated_at,
        }
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.id = id;
        self
    }

    pub fn with_issue_link(mut self, issue_link: String) -> Self {
        self.issue_link = issue_link;
        self
    }

    pub fn with_background(mut self, background: String) -> Self {
        self.background = background;
        self
    }

    pub fn with_root_cause(mut self, root_cause: String) -> Self {
        self.root_cause = root_cause;
        self
    }

    pub fn with_minimal_fixture(mut self, minimal_fixture: String) -> Self {
        self.minimal_fixture = minimal_fixture;
        self
    }

    pub fn with_task_id(mut self, task_id: String) -> Self {
        self.task_id = task_id;
        self
    }

    pub fn with_expected_result(mut self, expected_result: String) -> Self {
        self.expected_result = expected_result;
        self
    }

    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    pub fn with_execution_level(mut self, execution_level: ExecutionLevel) -> Self {
        self.execution_level = execution_level;
        self
    }

    pub fn with_status(mut self, status: RegressionStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_created_at(mut self, created_at: DateTime<Utc>) -> Self {
        self.created_at = created_at;
        self
    }

    pub fn with_updated_at(mut self, updated_at: DateTime<Utc>) -> Self {
        self.updated_at = updated_at;
        self
    }

    pub fn is_active(&self) -> bool {
        self.status == RegressionStatus::Active
    }

    pub fn is_candidate(&self) -> bool {
        self.status == RegressionStatus::Candidate
    }

    pub fn is_suppressed(&self) -> bool {
        self.status == RegressionStatus::Suppressed
    }

    pub fn is_resolved(&self) -> bool {
        self.status == RegressionStatus::Resolved
    }
}

impl Default for RegressionCase {
    fn default() -> Self {
        Self {
            id: String::new(),
            issue_link: String::new(),
            background: String::new(),
            root_cause: String::new(),
            minimal_fixture: String::new(),
            task_id: String::new(),
            expected_result: String::new(),
            severity: Severity::Medium,
            execution_level: ExecutionLevel::AlwaysOn,
            status: RegressionStatus::Candidate,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regression_case_instantiation_with_all_fields() {
        let now = Utc::now();
        let case = RegressionCase::new(
            "REG-001".to_string(),
            "https://github.com/example/repo/issues/123".to_string(),
            "Background description".to_string(),
            "Root cause summary".to_string(),
            "fixtures/regression/reg-001".to_string(),
            "TASK-001".to_string(),
            "Expected behavior".to_string(),
            Severity::High,
            ExecutionLevel::NightlyOnly,
            RegressionStatus::Candidate,
            now,
            now,
        );

        assert_eq!(case.id, "REG-001");
        assert_eq!(
            case.issue_link,
            "https://github.com/example/repo/issues/123"
        );
        assert_eq!(case.background, "Background description");
        assert_eq!(case.root_cause, "Root cause summary");
        assert_eq!(case.minimal_fixture, "fixtures/regression/reg-001");
        assert_eq!(case.task_id, "TASK-001");
        assert_eq!(case.expected_result, "Expected behavior");
        assert_eq!(case.severity, Severity::High);
        assert_eq!(case.execution_level, ExecutionLevel::NightlyOnly);
        assert_eq!(case.status, RegressionStatus::Candidate);
        assert_eq!(case.created_at, now);
        assert_eq!(case.updated_at, now);
    }

    #[test]
    fn test_regression_status_enum_has_all_variants() {
        let candidate = RegressionStatus::Candidate;
        let approved = RegressionStatus::Approved;
        let active = RegressionStatus::Active;
        let suppressed = RegressionStatus::Suppressed;
        let resolved = RegressionStatus::Resolved;

        assert_eq!(candidate, RegressionStatus::Candidate);
        assert_eq!(approved, RegressionStatus::Approved);
        assert_eq!(active, RegressionStatus::Active);
        assert_eq!(suppressed, RegressionStatus::Suppressed);
        assert_eq!(resolved, RegressionStatus::Resolved);
    }

    #[test]
    fn test_regression_status_partial_eq() {
        assert_eq!(RegressionStatus::Candidate, RegressionStatus::Candidate);
        assert_ne!(RegressionStatus::Candidate, RegressionStatus::Approved);
        assert_ne!(RegressionStatus::Candidate, RegressionStatus::Active);
        assert_ne!(RegressionStatus::Candidate, RegressionStatus::Suppressed);
        assert_ne!(RegressionStatus::Candidate, RegressionStatus::Resolved);
    }

    #[test]
    fn test_regression_case_yaml_serialization_roundtrip() {
        let now = Utc::now();
        let case = RegressionCase::new(
            "REG-002".to_string(),
            "https://github.com/example/repo/issues/456".to_string(),
            "Test background".to_string(),
            "Test root cause".to_string(),
            "fixtures/regression/reg-002".to_string(),
            "TASK-002".to_string(),
            "Test expected result".to_string(),
            Severity::Critical,
            ExecutionLevel::ReleaseOnly,
            RegressionStatus::Approved,
            now,
            now,
        );

        let yaml = serde_yaml::to_string(&case).expect("serialization should succeed");
        let deserialized: RegressionCase =
            serde_yaml::from_str(&yaml).expect("deserialization should succeed");

        assert_eq!(case.id, deserialized.id);
        assert_eq!(case.issue_link, deserialized.issue_link);
        assert_eq!(case.background, deserialized.background);
        assert_eq!(case.root_cause, deserialized.root_cause);
        assert_eq!(case.minimal_fixture, deserialized.minimal_fixture);
        assert_eq!(case.task_id, deserialized.task_id);
        assert_eq!(case.expected_result, deserialized.expected_result);
        assert_eq!(case.severity, deserialized.severity);
        assert_eq!(case.execution_level, deserialized.execution_level);
        assert_eq!(case.status, deserialized.status);
    }

    #[test]
    fn test_severity_levels_are_properly_defined() {
        assert_eq!(Severity::Critical, Severity::Critical);
        assert_eq!(Severity::High, Severity::High);
        assert_eq!(Severity::Medium, Severity::Medium);
        assert_eq!(Severity::Low, Severity::Low);
        assert_eq!(Severity::Cosmetic, Severity::Cosmetic);
    }

    #[test]
    fn test_severity_partial_eq() {
        assert_eq!(Severity::Critical, Severity::Critical);
        assert_ne!(Severity::Critical, Severity::High);
        assert_ne!(Severity::High, Severity::Medium);
        assert_ne!(Severity::Medium, Severity::Low);
        assert_ne!(Severity::Low, Severity::Cosmetic);
    }

    #[test]
    fn test_regression_case_builder_pattern() {
        let now = Utc::now();
        let case = RegressionCase::default()
            .with_id("REG-003".to_string())
            .with_issue_link("https://github.com/example/repo/issues/789".to_string())
            .with_background("Builder background".to_string())
            .with_root_cause("Builder root cause".to_string())
            .with_minimal_fixture("fixtures/reg-003".to_string())
            .with_task_id("TASK-003".to_string())
            .with_expected_result("Builder expected".to_string())
            .with_severity(Severity::Low)
            .with_execution_level(ExecutionLevel::AlwaysOn)
            .with_status(RegressionStatus::Active)
            .with_created_at(now)
            .with_updated_at(now);

        assert_eq!(case.id, "REG-003");
        assert_eq!(case.severity, Severity::Low);
        assert_eq!(case.execution_level, ExecutionLevel::AlwaysOn);
        assert_eq!(case.status, RegressionStatus::Active);
    }

    #[test]
    fn test_regression_case_default() {
        let case = RegressionCase::default();

        assert!(case.id.is_empty());
        assert!(case.issue_link.is_empty());
        assert!(case.severity == Severity::Medium);
        assert!(case.execution_level == ExecutionLevel::AlwaysOn);
        assert!(case.status == RegressionStatus::Candidate);
    }

    #[test]
    fn test_regression_case_status_helpers() {
        let now = Utc::now();

        let candidate_case = RegressionCase::new(
            "REG-004".to_string(),
            "https://example.com/issue".to_string(),
            "background".to_string(),
            "cause".to_string(),
            "fixture".to_string(),
            "TASK-004".to_string(),
            "expected".to_string(),
            Severity::High,
            ExecutionLevel::AlwaysOn,
            RegressionStatus::Candidate,
            now,
            now,
        );
        assert!(candidate_case.is_candidate());
        assert!(!candidate_case.is_active());

        let active_case = RegressionCase::new(
            "REG-005".to_string(),
            "https://example.com/issue".to_string(),
            "background".to_string(),
            "cause".to_string(),
            "fixture".to_string(),
            "TASK-005".to_string(),
            "expected".to_string(),
            Severity::High,
            ExecutionLevel::AlwaysOn,
            RegressionStatus::Active,
            now,
            now,
        );
        assert!(!active_case.is_candidate());
        assert!(active_case.is_active());

        let suppressed_case = RegressionCase::new(
            "REG-006".to_string(),
            "https://example.com/issue".to_string(),
            "background".to_string(),
            "cause".to_string(),
            "fixture".to_string(),
            "TASK-006".to_string(),
            "expected".to_string(),
            Severity::High,
            ExecutionLevel::AlwaysOn,
            RegressionStatus::Suppressed,
            now,
            now,
        );
        assert!(suppressed_case.is_suppressed());

        let resolved_case = RegressionCase::new(
            "REG-007".to_string(),
            "https://example.com/issue".to_string(),
            "background".to_string(),
            "cause".to_string(),
            "fixture".to_string(),
            "TASK-007".to_string(),
            "expected".to_string(),
            Severity::High,
            ExecutionLevel::AlwaysOn,
            RegressionStatus::Resolved,
            now,
            now,
        );
        assert!(resolved_case.is_resolved());
    }

    #[test]
    fn test_regression_status_yaml_serialization() {
        let statuses = vec![
            RegressionStatus::Candidate,
            RegressionStatus::Approved,
            RegressionStatus::Active,
            RegressionStatus::Suppressed,
            RegressionStatus::Resolved,
        ];

        for status in statuses {
            let yaml = serde_yaml::to_string(&status).unwrap();
            let deserialized: RegressionStatus = serde_yaml::from_str(&yaml).unwrap();
            assert_eq!(status, deserialized);
        }
    }

    #[test]
    fn test_regression_case_json_serialization_roundtrip() {
        let now = Utc::now();
        let case = RegressionCase::new(
            "REG-008".to_string(),
            "https://github.com/example/repo/issues/100".to_string(),
            "JSON background".to_string(),
            "JSON root cause".to_string(),
            "fixtures/regression/reg-008".to_string(),
            "TASK-008".to_string(),
            "JSON expected result".to_string(),
            Severity::Medium,
            ExecutionLevel::NightlyOnly,
            RegressionStatus::Approved,
            now,
            now,
        );

        let json = serde_json::to_string(&case).expect("JSON serialization should succeed");
        let deserialized: RegressionCase =
            serde_json::from_str(&json).expect("JSON deserialization should succeed");

        assert_eq!(case.id, deserialized.id);
        assert_eq!(case.status, deserialized.status);
        assert_eq!(case.severity, deserialized.severity);
    }
}
