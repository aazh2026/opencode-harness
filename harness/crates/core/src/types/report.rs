use serde::{Deserialize, Serialize};

use super::failure_classification::FailureClassification;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestCase {
    pub id: String,
    pub status: TestCaseStatus,
    pub duration: u64,
    pub failure_classification: FailureClassification,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TestCaseStatus {
    Passed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Report {
    pub timestamp: String,
    pub suite: String,
    pub total: u32,
    pub passed: u32,
    pub failed: u32,
    pub skipped: u32,
    pub mismatches: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_has_all_required_fields() {
        let report = Report {
            timestamp: "2026-04-16T12:00:00Z".to_string(),
            suite: "test-suite".to_string(),
            total: 10,
            passed: 8,
            failed: 1,
            skipped: 0,
            mismatches: 1,
        };

        assert_eq!(report.timestamp, "2026-04-16T12:00:00Z");
        assert_eq!(report.suite, "test-suite");
        assert_eq!(report.total, 10);
        assert_eq!(report.passed, 8);
        assert_eq!(report.failed, 1);
        assert_eq!(report.skipped, 0);
        assert_eq!(report.mismatches, 1);
    }

    #[test]
    fn test_report_serde_roundtrip() {
        let report = Report {
            timestamp: "2026-04-16T12:00:00Z".to_string(),
            suite: "integration".to_string(),
            total: 5,
            passed: 3,
            failed: 1,
            skipped: 1,
            mismatches: 0,
        };

        let json = serde_json::to_string(&report).unwrap();
        let deserialized: Report = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, report);
    }

    #[test]
    fn test_testcase_has_all_required_fields() {
        let testcase = TestCase {
            id: "test-001".to_string(),
            status: TestCaseStatus::Passed,
            duration: 150,
            failure_classification: FailureClassification::ImplementationFailure,
            error_message: None,
        };

        assert_eq!(testcase.id, "test-001");
        assert_eq!(testcase.status, TestCaseStatus::Passed);
        assert_eq!(testcase.duration, 150);
        assert_eq!(
            testcase.failure_classification,
            FailureClassification::ImplementationFailure
        );
        assert_eq!(testcase.error_message, None);
    }

    #[test]
    fn test_testcase_with_failure() {
        let testcase = TestCase {
            id: "test-002".to_string(),
            status: TestCaseStatus::Failed,
            duration: 50,
            failure_classification: FailureClassification::DependencyMissing,
            error_message: Some("Could not find dependency 'foo'".to_string()),
        };

        assert_eq!(testcase.id, "test-002");
        assert_eq!(testcase.status, TestCaseStatus::Failed);
        assert_eq!(testcase.duration, 50);
        assert_eq!(
            testcase.failure_classification,
            FailureClassification::DependencyMissing
        );
        assert_eq!(
            testcase.error_message,
            Some("Could not find dependency 'foo'".to_string())
        );
    }

    #[test]
    fn test_testcase_serde_roundtrip() {
        let testcase = TestCase {
            id: "test-003".to_string(),
            status: TestCaseStatus::Skipped,
            duration: 0,
            failure_classification: FailureClassification::EnvironmentNotSupported,
            error_message: None,
        };

        let json = serde_json::to_string(&testcase).unwrap();
        let deserialized: TestCase = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, testcase);
    }

    #[test]
    fn test_testcase_status_serde() {
        assert_eq!(
            serde_json::to_string(&TestCaseStatus::Passed).unwrap(),
            "\"passed\""
        );
        assert_eq!(
            serde_json::to_string(&TestCaseStatus::Failed).unwrap(),
            "\"failed\""
        );
        assert_eq!(
            serde_json::to_string(&TestCaseStatus::Skipped).unwrap(),
            "\"skipped\""
        );

        let passed: TestCaseStatus = serde_json::from_str("\"passed\"").unwrap();
        let failed: TestCaseStatus = serde_json::from_str("\"failed\"").unwrap();
        let skipped: TestCaseStatus = serde_json::from_str("\"skipped\"").unwrap();

        assert_eq!(passed, TestCaseStatus::Passed);
        assert_eq!(failed, TestCaseStatus::Failed);
        assert_eq!(skipped, TestCaseStatus::Skipped);
    }
}
