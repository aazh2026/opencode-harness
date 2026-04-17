use opencode_core::reporting::report::{ParityReport, SuiteInfo, TaskResult, WhitelistEntry};
use opencode_core::types::parity_verdict::{DiffCategory, ParityVerdict};
use opencode_core::types::severity::Severity;

#[test]
fn test_parity_report_has_all_required_fields() {
    let report = ParityReport::new("TestRunner");

    assert!(
        !report.run_id.to_string().is_empty(),
        "ParityReport should have run_id"
    );
    assert!(
        !report.timestamp.to_rfc3339().is_empty(),
        "ParityReport should have timestamp"
    );
    assert_eq!(
        report.runner, "TestRunner",
        "ParityReport should have runner"
    );
    assert!(
        report.task_results.is_empty(),
        "ParityReport should have task_results"
    );
    assert_eq!(
        report.summary.total_tasks, 0,
        "ParityReport should have summary"
    );
    assert!(
        report.mismatch_counts.is_empty(),
        "ParityReport should have mismatch_counts"
    );
    assert!(
        report.severity_aggregation.is_empty(),
        "ParityReport should have severity_aggregation"
    );
    assert_eq!(
        report.suite_info.name, "",
        "ParityReport should have suite_info"
    );
}

#[test]
fn test_report_summary_has_mismatch_counts() {
    let report = ParityReport::new("TestRunner");
    assert!(
        report.mismatch_counts.iter().next().is_none(),
        "ParityReport should have mismatch_counts field"
    );
    let mut report = ParityReport::new("TestRunner");
    report.mismatch_counts.insert(DiffCategory::OutputText, 5);
    assert_eq!(
        report.mismatch_counts.get(&DiffCategory::OutputText),
        Some(&5)
    );
}

#[test]
fn test_report_summary_has_severity_aggregation() {
    let report = ParityReport::new("TestRunner");
    assert!(
        report.severity_aggregation.iter().next().is_none(),
        "ParityReport should have severity_aggregation field"
    );
    let mut report = ParityReport::new("TestRunner");
    report.severity_aggregation.insert(Severity::Critical, 3);
    assert_eq!(
        report.severity_aggregation.get(&Severity::Critical),
        Some(&3)
    );
}

#[test]
fn test_parity_report_has_suite_info() {
    let report = ParityReport::new("TestRunner");
    assert_eq!(
        report.suite_info.name, "",
        "ParityReport should have suite_info field"
    );
    let mut report = ParityReport::new("TestRunner");
    report.suite_info = SuiteInfo {
        name: "pr-smoke".to_string(),
        description: "PR smoke suite".to_string(),
        gate_level: "PR".to_string(),
        included_categories: vec!["Quick".to_string(), "Smoke".to_string()],
    };
    assert_eq!(report.suite_info.name, "pr-smoke");
}

#[test]
fn test_report_summary_has_failure_type_breakdown() {
    let mut report = ParityReport::new("TestRunner");
    report.compute_summary();
    assert!(
        report
            .summary
            .failure_type_breakdown
            .iter()
            .next()
            .is_none(),
        "ReportSummary should have failure_type_breakdown field"
    );
}

#[test]
fn test_report_summary_has_manual_check_count() {
    let mut report = ParityReport::new("TestRunner");
    report.compute_summary();
    assert_eq!(
        report.summary.manual_check_count, 0,
        "ReportSummary should have manual_check_count field"
    );
    let mut result = TaskResult::new(
        "TASK-001".to_string(),
        ParityVerdict::ManualCheck {
            reason: "Needs review".to_string(),
            candidates: vec![],
        },
        100,
    );
    result.artifacts.push("artifact1".to_string());
    report.add_task(result);
    report.compute_summary();
    assert_eq!(report.summary.manual_check_count, 1);
}

#[test]
fn test_report_summary_has_environment_limited_count() {
    let mut report = ParityReport::new("TestRunner");
    report.compute_summary();
    assert_eq!(
        report.summary.environment_limited_count, 0,
        "ReportSummary should have environment_limited_count field"
    );
    let result = TaskResult::new(
        "TASK-001".to_string(),
        ParityVerdict::PassWithAllowedVariance {
            variance_type: opencode_core::types::parity_verdict::VarianceType::EnvironmentLimited,
            details: "Limited env".to_string(),
        },
        100,
    );
    report.add_task(result);
    report.compute_summary();
    assert_eq!(report.summary.environment_limited_count, 1);
}

#[test]
fn test_report_summary_has_artifact_links() {
    let mut report = ParityReport::new("TestRunner");
    report.compute_summary();
    assert!(
        report.summary.artifact_links.is_empty(),
        "ReportSummary should have artifact_links field"
    );
    let mut result = TaskResult::new("TASK-001".to_string(), ParityVerdict::Pass, 100);
    result
        .artifacts
        .push("artifacts/task-001/output.txt".to_string());
    result
        .artifacts
        .push("artifacts/task-001/log.txt".to_string());
    report.add_task(result);
    report.compute_summary();
    assert_eq!(report.summary.artifact_links.len(), 2);
    assert!(report
        .summary
        .artifact_links
        .contains(&"artifacts/task-001/output.txt".to_string()));
}

#[test]
fn test_report_summary_has_whitelist_applied() {
    let mut report = ParityReport::new("TestRunner");
    report.compute_summary();
    assert!(
        report.summary.whitelist_applied.is_empty(),
        "ReportSummary should have whitelist_applied field"
    );
    let entry = WhitelistEntry {
        task_id: "TASK-WL-001".to_string(),
        reason: "Known variance".to_string(),
        expires_at: None,
    };
    report.summary.whitelist_applied.push(entry);
    assert_eq!(report.summary.whitelist_applied.len(), 1);
    assert_eq!(report.summary.whitelist_applied[0].task_id, "TASK-WL-001");
}

#[test]
fn test_compute_summary_calculates_all_fields() {
    use opencode_core::types::failure_classification::FailureClassification;
    use opencode_core::types::parity_verdict::BlockedReason;

    let mut report = ParityReport::new("TestRunner");

    report.add_task(TaskResult::new(
        "TASK-001".to_string(),
        ParityVerdict::Pass,
        100,
    ));
    report.add_task(TaskResult::new(
        "TASK-002".to_string(),
        ParityVerdict::Fail {
            category: DiffCategory::OutputText,
            details: "Mismatch".to_string(),
        },
        50,
    ));
    report.add_task(TaskResult::new(
        "TASK-003".to_string(),
        ParityVerdict::ManualCheck {
            reason: "Needs review".to_string(),
            candidates: vec![],
        },
        75,
    ));
    report.add_task(TaskResult::new(
        "TASK-004".to_string(),
        ParityVerdict::Blocked {
            reason: BlockedReason::EnvironmentNotSupported {
                requirement: "GPU".to_string(),
            },
        },
        30,
    ));
    report.add_task(TaskResult::new(
        "TASK-005".to_string(),
        ParityVerdict::Error {
            runner: "TestRunner".to_string(),
            reason: "Binary not found".to_string(),
        },
        10,
    ));
    report.add_task(TaskResult::new(
        "TASK-006".to_string(),
        ParityVerdict::PassWithAllowedVariance {
            variance_type: opencode_core::types::parity_verdict::VarianceType::EnvironmentLimited,
            details: "Limited environment".to_string(),
        },
        100,
    ));

    let mut result = TaskResult::new("TASK-007".to_string(), ParityVerdict::Pass, 100);
    result.artifacts.push("artifacts/task-007/".to_string());
    report.add_task(result);

    report.compute_summary();

    assert_eq!(
        report.mismatch_counts.get(&DiffCategory::OutputText),
        Some(&1)
    );

    assert_eq!(
        report
            .summary
            .failure_type_breakdown
            .get(&FailureClassification::ImplementationFailure),
        Some(&1)
    );
    assert_eq!(
        report
            .summary
            .failure_type_breakdown
            .get(&FailureClassification::FlakySuspected),
        Some(&1)
    );
    assert_eq!(
        report
            .summary
            .failure_type_breakdown
            .get(&FailureClassification::EnvironmentNotSupported),
        Some(&1)
    );
    assert_eq!(
        report
            .summary
            .failure_type_breakdown
            .get(&FailureClassification::InfraFailure),
        Some(&1)
    );

    assert_eq!(report.summary.manual_check_count, 1);
    assert_eq!(report.summary.environment_limited_count, 2);
    assert!(!report.summary.artifact_links.is_empty());
}

#[test]
fn test_json_roundtrip_serialization() {
    let mut report = ParityReport::new("DifferentialRunner");

    report.add_task(TaskResult::new(
        "TASK-001".to_string(),
        ParityVerdict::Pass,
        100,
    ));

    let mut result = TaskResult::new(
        "TASK-002".to_string(),
        ParityVerdict::Fail {
            category: DiffCategory::ExitCode,
            details: "Exit code mismatch".to_string(),
        },
        50,
    );
    result.artifacts.push("artifacts/task-002/".to_string());
    report.add_task(result);

    report.suite_info = SuiteInfo {
        name: "pr-smoke".to_string(),
        description: "PR smoke test suite".to_string(),
        gate_level: "PR".to_string(),
        included_categories: vec!["Quick".to_string(), "Smoke".to_string()],
    };

    report.compute_summary();

    let json = report.to_json().expect("JSON serialization should succeed");

    assert!(json.contains("\"run_id\""));
    assert!(json.contains("\"timestamp\""));
    assert!(json.contains("\"runner\""));
    assert!(json.contains("\"DifferentialRunner\""));
    assert!(json.contains("\"task_results\""));
    assert!(json.contains("\"summary\""));
    assert!(json.contains("\"mismatch_counts\""));
    assert!(json.contains("\"severity_aggregation\""));
    assert!(json.contains("\"suite_info\""));
    assert!(json.contains("\"failure_type_breakdown\""));
    assert!(json.contains("\"manual_check_count\""));
    assert!(json.contains("\"environment_limited_count\""));
    assert!(json.contains("\"artifact_links\""));
    assert!(json.contains("\"whitelist_applied\""));

    let deserialized: ParityReport =
        serde_json::from_str(&json).expect("JSON deserialization should succeed");

    assert_eq!(deserialized.run_id, report.run_id);
    assert_eq!(deserialized.runner, report.runner);
    assert_eq!(deserialized.task_results.len(), report.task_results.len());
    assert_eq!(deserialized.summary.total_tasks, 2);
    assert_eq!(
        deserialized.mismatch_counts.get(&DiffCategory::ExitCode),
        Some(&1)
    );
}

#[test]
fn test_existing_parity_report_serialization_still_works() {
    let mut report = ParityReport::new("LegacyRunner");

    report.add_task(TaskResult::new(
        "TASK-A".to_string(),
        ParityVerdict::Pass,
        100,
    ));
    report.add_task(TaskResult::new(
        "TASK-B".to_string(),
        ParityVerdict::Pass,
        200,
    ));

    report.compute_summary();

    let json = report.to_json().expect("Should serialize successfully");

    assert!(json.contains("\"total_tasks\""));
    assert!(json.contains("\"pass_rate\""));
    assert!(json.contains("\"timing_ms\""));
    assert!(json.contains("\"verdict_counts\""));

    let deserialized: ParityReport =
        serde_json::from_str(&json).expect("Should deserialize successfully");

    assert_eq!(deserialized.summary.total_tasks, 2);
    assert!((deserialized.summary.pass_rate - 1.0).abs() < 0.001);
}
