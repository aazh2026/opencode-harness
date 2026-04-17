use opencode_core::reporting::gate::{CIGate, GateConfig, GateFailure};
use opencode_core::reporting::report::{ParityReport, TaskResult};
use opencode_core::types::parity_verdict::{
    BlockedReason, DiffCategory, MismatchCandidate, ParityVerdict, VarianceType,
};

fn create_report_with_pass_rate(pass_count: u32, fail_count: u32) -> ParityReport {
    let mut report = ParityReport::new("TestRunner");

    for i in 0..pass_count {
        report.add_task(TaskResult::new(
            format!("PASS-{}", i),
            ParityVerdict::Pass,
            100,
        ));
    }

    for i in 0..fail_count {
        report.add_task(TaskResult::new(
            format!("FAIL-{}", i),
            ParityVerdict::Fail {
                category: DiffCategory::OutputText,
                details: "Mismatch".to_string(),
            },
            50,
        ));
    }

    report.compute_summary();
    report
}

fn create_report_with_pass_rate_and_manual_check(
    pass_count: u32,
    fail_count: u32,
    manual_check_count: u32,
) -> ParityReport {
    let mut report = ParityReport::new("TestRunner");

    for i in 0..pass_count {
        report.add_task(TaskResult::new(
            format!("PASS-{}", i),
            ParityVerdict::Pass,
            100,
        ));
    }

    for i in 0..fail_count {
        report.add_task(TaskResult::new(
            format!("FAIL-{}", i),
            ParityVerdict::Fail {
                category: DiffCategory::OutputText,
                details: "Mismatch".to_string(),
            },
            50,
        ));
    }

    for i in 0..manual_check_count {
        report.add_task(TaskResult::new(
            format!("MANUAL-{}", i),
            ParityVerdict::ManualCheck {
                reason: "Requires human review".to_string(),
                candidates: vec![MismatchCandidate::new(
                    format!("field_{}", i),
                    "legacy".to_string(),
                    "rust".to_string(),
                    DiffCategory::OutputText,
                )],
            },
            100,
        ));
    }

    report.compute_summary();
    report
}

fn create_report_with_environment_limited(pass_count: u32, env_limited_count: u32) -> ParityReport {
    let mut report = ParityReport::new("TestRunner");

    for i in 0..pass_count {
        report.add_task(TaskResult::new(
            format!("PASS-{}", i),
            ParityVerdict::Pass,
            100,
        ));
    }

    for i in 0..env_limited_count {
        report.add_task(TaskResult::new(
            format!("ENVLIM-{}", i),
            ParityVerdict::PassWithAllowedVariance {
                variance_type: VarianceType::EnvironmentLimited,
                details: "Environment not supported".to_string(),
            },
            50,
        ));
    }

    report.compute_summary();
    report
}

#[test]
fn test_pr_gate_passes_with_90_percent() {
    let report = create_report_with_pass_rate(9, 1);
    let config = GateConfig::pr();
    let gate = CIGate::evaluate(&report, &config);

    assert!(
        gate.is_passed(),
        "PR gate should pass with exactly 90% pass rate (9/10)"
    );
    assert!(
        gate.blockers.is_empty(),
        "PR gate should have no blockers at 90% pass rate"
    );
}

#[test]
fn test_pr_gate_fails_below_90_percent() {
    let report = create_report_with_pass_rate(80, 10);
    let config = GateConfig::pr();
    let gate = CIGate::evaluate(&report, &config);

    assert!(
        !gate.is_passed(),
        "PR gate should fail with 89% pass rate (80/90)"
    );
    assert!(
        !gate.blockers.is_empty() || gate.warnings.len() as u32 > config.max_warnings,
        "PR gate should have blockers or exceed warnings at 89% pass rate"
    );
}

#[test]
fn test_nightly_gate_passes_with_80_percent() {
    let report = create_report_with_pass_rate(8, 2);
    let config = GateConfig::nightly();
    let gate = CIGate::evaluate(&report, &config);

    assert!(
        gate.is_passed(),
        "Nightly gate should pass with exactly 80% pass rate (8/10)"
    );
    assert!(
        gate.blockers.is_empty(),
        "Nightly gate should have no blockers at 80% pass rate"
    );
}

#[test]
fn test_nightly_gate_fails_below_80_percent() {
    let report = create_report_with_pass_rate(7, 3);
    let config = GateConfig::nightly();
    let gate = CIGate::evaluate(&report, &config);

    assert!(
        !gate.is_passed(),
        "Nightly gate should fail with 70% pass rate (7/10)"
    );
}

#[test]
fn test_release_gate_requires_100_percent() {
    let report = create_report_with_pass_rate(9, 1);
    let config = GateConfig::release();
    let gate = CIGate::evaluate(&report, &config);

    assert!(
        !gate.is_passed(),
        "Release gate should fail with 90% pass rate (9/10)"
    );

    let report = create_report_with_pass_rate(10, 0);
    let gate = CIGate::evaluate(&report, &config);
    assert!(
        gate.is_passed(),
        "Release gate should pass with exactly 100% pass rate (10/10)"
    );
}

#[test]
fn test_release_gate_fails_on_any_regression() {
    let mut report = ParityReport::new("TestRunner");

    for i in 0..10 {
        report.add_task(TaskResult::new(
            format!("PASS-{}", i),
            ParityVerdict::Pass,
            100,
        ));
    }

    report.add_task(TaskResult::new(
        "REGR-001".to_string(),
        ParityVerdict::Fail {
            category: DiffCategory::ExitCode,
            details: "Exit code mismatch".to_string(),
        },
        50,
    ));

    report.compute_summary();

    let config = GateConfig::release();
    let gate = CIGate::evaluate(&report, &config);

    assert!(
        !gate.is_passed(),
        "Release gate should fail on any regression"
    );
    assert!(
        !gate.blockers.is_empty(),
        "Release gate should have blockers for regression"
    );
}

#[test]
fn test_blocked_tasks_counted_correctly() {
    let mut report = ParityReport::new("TestRunner");

    for i in 0..9 {
        report.add_task(TaskResult::new(
            format!("PASS-{}", i),
            ParityVerdict::Pass,
            100,
        ));
    }

    report.add_task(TaskResult::new(
        "BLOCKED-001".to_string(),
        ParityVerdict::Blocked {
            reason: BlockedReason::BinaryNotFound {
                binary: "opencode".to_string(),
            },
        },
        50,
    ));

    report.compute_summary();

    let config = GateConfig::pr();
    let gate = CIGate::evaluate(&report, &config);

    assert!(
        !gate.is_passed(),
        "Gate should fail when blocked tasks exist"
    );
    assert!(
        gate.blocker_count() >= 1,
        "Should have at least 1 blocker from blocked tasks"
    );

    let has_blocked_failure = gate.blockers.iter().any(|f| {
        matches!(f, GateFailure::BlockedTask { task_id: _, .. })
    });
    assert!(
        has_blocked_failure,
        "Should have BlockedTask failure type"
    );
}

#[test]
fn test_manual_check_handled_per_suite_config() {
    let report = create_report_with_pass_rate_and_manual_check(9, 0, 1);
    let config = GateConfig::pr();
    let gate = CIGate::evaluate(&report, &config);

    assert!(
        gate.is_passed(),
        "PR gate should pass with manual_check task since allow_manual_check=true"
    );

    let report = create_report_with_pass_rate_and_manual_check(9, 0, 1);
    let config = GateConfig::release();
    let gate = CIGate::evaluate(&report, &config);

    assert!(
        !gate.is_passed(),
        "Release gate should fail with manual_check task since allow_manual_check=false"
    );
}

#[test]
fn test_environment_limited_does_not_fail_release() {
    let report = create_report_with_environment_limited(9, 1);
    let config = GateConfig::release();
    let gate = CIGate::evaluate(&report, &config);

    assert!(
        gate.is_passed(),
        "Release gate should pass with environment_limited task as it's PassWithAllowedVariance"
    );
}

#[test]
fn test_gate_output_includes_mismatches_and_artifacts() {
    let mut report = ParityReport::new("TestRunner");

    report.add_task(TaskResult::new(
        "PASS-001".to_string(),
        ParityVerdict::Pass,
        100,
    ));

    report.add_task(TaskResult::new(
        "FAIL-001".to_string(),
        ParityVerdict::Fail {
            category: DiffCategory::OutputText,
            details: "Output mismatch".to_string(),
        },
        50,
    ));

    report.add_task(TaskResult::new(
        "FAIL-002".to_string(),
        ParityVerdict::Fail {
            category: DiffCategory::ExitCode,
            details: "Exit code mismatch".to_string(),
        },
        50,
    ));

    report.add_task(TaskResult::new(
        "ARTIFACT-001".to_string(),
        ParityVerdict::Pass,
        100,
    ));

    report.compute_summary();

    assert!(
        !report.mismatch_counts.is_empty(),
        "Report should have mismatch_counts after compute_summary"
    );

    assert!(
        report.summary.artifact_links.is_empty(),
        "No artifacts yet since none were set with with_artifacts()"
    );

    let mut report = ParityReport::new("TestRunner");
    report.add_task(
        TaskResult::new("ARTIFACT-001".to_string(), ParityVerdict::Pass, 100)
            .with_artifacts(vec!["artifacts/ARTIFACT-001/output.txt".to_string()]),
    );
    report.add_task(
        TaskResult::new("ARTIFACT-002".to_string(), ParityVerdict::Pass, 100)
            .with_artifacts(vec!["artifacts/ARTIFACT-002/diff.md".to_string()]),
    );
    report.compute_summary();

    assert!(
        !report.summary.artifact_links.is_empty(),
        "Report should have artifact_links when tasks have artifacts"
    );
    assert_eq!(
        report.summary.artifact_links.len(),
        2,
        "Should have 2 artifact links"
    );

    let config = GateConfig::release();
    let gate = CIGate::evaluate(&report, &config);

    assert!(
        gate.is_passed(),
        "Gate with passing tasks and artifacts should pass"
    );
}

#[test]
fn test_pr_gate_threshold_edge_case() {
    let report = create_report_with_pass_rate(18, 2);
    let config = GateConfig::pr();
    let gate = CIGate::evaluate(&report, &config);

    assert!(
        gate.is_passed(),
        "PR gate should pass with 90% pass rate (18/20)"
    );
}

#[test]
fn test_nightly_gate_threshold_edge_case() {
    let report = create_report_with_pass_rate(16, 4);
    let config = GateConfig::nightly();
    let gate = CIGate::evaluate(&report, &config);

    assert!(
        gate.is_passed(),
        "Nightly gate should pass with 80% pass rate (16/20)"
    );
}

#[test]
fn test_gate_summary_contains_correct_counts() {
    let report = create_report_with_pass_rate(7, 3);
    let config = GateConfig::pr();
    let gate = CIGate::evaluate(&report, &config);

    assert_eq!(
        gate.summary.total_tasks, 10,
        "Summary should show 10 total tasks"
    );
    assert_eq!(
        gate.summary.passed_tasks, 7,
        "Summary should show 7 passed tasks"
    );
    assert_eq!(
        gate.summary.failed_tasks, 3,
        "Summary should show 3 failed tasks"
    );
    assert!((gate.summary.pass_rate - 0.7).abs() < 0.01);
}

#[test]
fn test_gate_warnings_vs_blockers() {
    let mut report = ParityReport::new("TestRunner");

    for i in 0..9 {
        report.add_task(TaskResult::new(
            format!("PASS-{}", i),
            ParityVerdict::Pass,
            100,
        ));
    }

    report.add_task(TaskResult::new(
        "WARN-001".to_string(),
        ParityVerdict::Warn {
            category: DiffCategory::OutputText,
            message: "Minor variance".to_string(),
        },
        50,
    ));

    report.compute_summary();

    let config = GateConfig::pr();
    let gate = CIGate::evaluate(&report, &config);

    assert!(
        gate.is_passed(),
        "PR gate should pass with 1 warning since max_warnings=5"
    );
}

#[test]
fn test_gate_fails_when_warnings_exceed_max() {
    let mut report = ParityReport::new("TestRunner");

    for i in 0..5 {
        report.add_task(TaskResult::new(
            format!("PASS-{}", i),
            ParityVerdict::Pass,
            100,
        ));
    }

    for i in 0..6 {
        report.add_task(TaskResult::new(
            format!("WARN-{}", i),
            ParityVerdict::Warn {
                category: DiffCategory::OutputText,
                message: "Minor variance".to_string(),
            },
            50,
        ));
    }

    report.compute_summary();

    let config = GateConfig::pr();
    let gate = CIGate::evaluate(&report, &config);

    assert!(
        !gate.is_passed(),
        "PR gate should fail when warnings exceed max_warnings=5"
    );
}