use crate::reporting::report::ParityReport;

/// Gate levels for CI integration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateLevel {
    /// Pull request gate - moderate strictness.
    PR,
    /// Nightly gate - lower strictness to catch major regressions.
    Nightly,
    /// Release gate - highest strictness, requires all tests to pass.
    Release,
}

impl GateLevel {
    /// Returns the default pass rate threshold for this gate level.
    pub fn pass_rate_threshold(&self) -> f64 {
        match self {
            GateLevel::PR => 0.90,
            GateLevel::Nightly => 0.80,
            GateLevel::Release => 1.0,
        }
    }

    /// Returns the default max blockers for this gate level.
    pub fn max_blockers(&self) -> u32 {
        0
    }

    /// Returns the default max warnings for this gate level.
    pub fn max_warnings(&self) -> u32 {
        match self {
            GateLevel::PR => 5,
            GateLevel::Nightly => 10,
            GateLevel::Release => 0,
        }
    }

    /// Returns a human-readable name for this gate level.
    pub fn name(&self) -> &'static str {
        match self {
            GateLevel::PR => "PR Gate",
            GateLevel::Nightly => "Nightly Gate",
            GateLevel::Release => "Release Gate",
        }
}
    }
#[derive(Debug, Clone)]

pub struct GateWarning {
    /// Warning message describing the issue.
    pub message: String,
    /// Optional task ID associated with this warning.
    pub task_id: Option<String>,
}

impl GateWarning {
    /// Creates a new gate warning with the given message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            task_id: None,
        }
    }

    /// Creates a new gate warning with a task ID.
    pub fn with_task_id(mut self, task_id: impl Into<String>) -> Self {
        self.task_id = Some(task_id.into());
        self
    }
}

/// Failure reasons for CI gate evaluation.
#[derive(Debug, Clone)]
pub enum GateFailure {
    /// A specific task was blocked and requires attention.
    BlockedTask {
        task_id: String,
        reason: String,
    },
    /// Too many regressions detected compared to baseline.
    TooManyRegressions {
        regressed: u32,
        threshold: u32,
    },
    /// Verdict mismatch between expected and actual results.
    VerdictMismatch {
        task_id: String,
        expected: String,
        actual: String,
    },
    /// Task execution exceeded timeout threshold.
    TimeoutExceeded {
        task_id: String,
        duration_ms: u64,
        max_allowed_ms: u64,
    },
    /// Error rate exceeded acceptable threshold.
    ErrorRateExceeded {
        error_count: u32,
        total_tasks: u32,
        threshold: f64,
    },
}

impl GateFailure {
    /// Returns a human-readable description of the failure.
    pub fn description(&self) -> String {
        match self {
            GateFailure::BlockedTask { task_id, reason } => {
                format!("Task '{}' blocked: {}", task_id, reason)
            }
            GateFailure::TooManyRegressions { regressed, threshold } => {
                format!(
                    "Too many regressions: {} (threshold: {})",
                    regressed, threshold
                )
            }
            GateFailure::VerdictMismatch {
                task_id,
                expected,
                actual,
            } => {
                format!(
                    "Task '{}' verdict mismatch: expected {}, got {}",
                    task_id, expected, actual
                )
            }
            GateFailure::TimeoutExceeded {
                task_id,
                duration_ms,
                max_allowed_ms,
            } => {
                format!(
                    "Task '{}' timed out: {}ms (max allowed: {}ms)",
                    task_id, duration_ms, max_allowed_ms
                )
            }
            GateFailure::ErrorRateExceeded {
                error_count,
                total_tasks,
                threshold,
            } => {
                format!(
                    "Error rate exceeded: {}/{} ({:.1}%, threshold: {:.1}%)",
                    error_count,
                    total_tasks,
                    (*error_count as f64 / u64::max(*total_tasks as u64, 1) as f64) * 100.0,
                    threshold * 100.0
                )
            }
        }
    }
}

/// Configuration for CI gate evaluation.
#[derive(Debug, Clone)]
pub struct GateConfig {
    /// The gate level determining thresholds.
    pub level: GateLevel,
    /// Minimum pass rate required (0.0 to 1.0).
    pub pass_rate_threshold: f64,
    /// Maximum number of blocker failures allowed.
    pub max_blockers: u32,
    /// Maximum number of warnings allowed.
    pub max_warnings: u32,
    /// Maximum task duration in milliseconds before timeout.
    pub max_timeout_ms: Option<u64>,
}

impl GateConfig {
    /// Creates a new gate config with default settings for the given level.
    pub fn new(level: GateLevel) -> Self {
        Self {
            level,
            pass_rate_threshold: level.pass_rate_threshold(),
            max_blockers: level.max_blockers(),
            max_warnings: level.max_warnings(),
            max_timeout_ms: None,
        }
    }

    /// Creates a PR gate configuration.
    pub fn pr() -> Self {
        Self::new(GateLevel::PR)
    }

    /// Creates a nightly gate configuration.
    pub fn nightly() -> Self {
        Self::new(GateLevel::Nightly)
    }

    /// Creates a release gate configuration.
    pub fn release() -> Self {
        Self::new(GateLevel::Release)
    }

    /// Sets a custom pass rate threshold.
    pub fn with_pass_rate_threshold(mut self, threshold: f64) -> Self {
        self.pass_rate_threshold = threshold;
        self
    }

    /// Sets maximum blockers.
    pub fn with_max_blockers(mut self, max: u32) -> Self {
        self.max_blockers = max;
        self
    }

    /// Sets maximum warnings.
    pub fn with_max_warnings(mut self, max: u32) -> Self {
        self.max_warnings = max;
        self
    }

    /// Sets maximum timeout in milliseconds.
    pub fn with_max_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.max_timeout_ms = Some(timeout_ms);
        self
    }
}

impl Default for GateConfig {
    fn default() -> Self {
        Self::new(GateLevel::PR)
    }
}

/// Result of CI gate evaluation.
#[derive(Debug, Clone)]
pub struct CIGate {
    /// The gate level that was evaluated.
    pub level: GateLevel,
    /// Whether the gate passed.
    pub passed: bool,
    /// List of blocker failures.
    pub blockers: Vec<GateFailure>,
    /// List of warnings.
    pub warnings: Vec<GateWarning>,
    /// Summary statistics from the evaluated report.
    pub summary: GateSummary,
}

#[derive(Debug, Clone)]
pub struct GateSummary {
    /// Total tasks evaluated.
    pub total_tasks: u32,
    /// Number of passing tasks.
    pub passed_tasks: u32,
    /// Number of failing tasks.
    pub failed_tasks: u32,
    /// Number of error tasks.
    pub error_tasks: u32,
    /// Pass rate as a percentage.
    pub pass_rate: f64,
}

impl CIGate {
    /// Evaluates a parity report against the given gate configuration.
    pub fn evaluate(report: &ParityReport, config: &GateConfig) -> Self {
        let total_tasks = report.summary.total_tasks;
        let pass_rate = report.summary.pass_rate;

        let passed_tasks = report
            .task_results
            .iter()
            .filter(|t| t.verdict.is_pass())
            .count() as u32;

        let failed_tasks = report
            .task_results
            .iter()
            .filter(|t| t.verdict.is_different())
            .count() as u32;

        let error_tasks = report
            .task_results
            .iter()
            .filter(|t| t.verdict.is_error())
            .count() as u32;

        let mut blockers = Vec::new();
        let mut warnings = Vec::new();

        // Check pass rate threshold
        if pass_rate < config.pass_rate_threshold {
            let deficit = (config.pass_rate_threshold - pass_rate) * total_tasks as f64;
            if config.level == GateLevel::Release || deficit >= 1.0 {
                blockers.push(GateFailure::TooManyRegressions {
                    regressed: (deficit.ceil() as u32).max(1),
                    threshold: (config.pass_rate_threshold * total_tasks as f64) as u32,
                });
            } else {
                warnings.push(GateWarning::new(format!(
                    "Pass rate {:.1}% below threshold {:.1}%",
                    pass_rate * 100.0,
                    config.pass_rate_threshold * 100.0
                )));
            }
        }

        // Check for blocked tasks
        for task in &report.task_results {
            if let crate::types::parity_verdict::ParityVerdict::Blocked { reason } = &task.verdict {
                blockers.push(GateFailure::BlockedTask {
                    task_id: task.task_id.clone(),
                    reason: format!("{:?}", reason),
                });
            }
        }

        // Check for error rate
        if total_tasks > 0 {
            let error_rate = error_tasks as f64 / total_tasks as f64;
            if error_rate > 0.1 {
                // More than 10% errors
                blockers.push(GateFailure::ErrorRateExceeded {
                    error_count: error_tasks,
                    total_tasks,
                    threshold: 0.1,
                });
            }
        }

        // Check timeout thresholds
        if let Some(max_timeout) = config.max_timeout_ms {
            for task in &report.task_results {
                if task.duration_ms > max_timeout {
                    warnings.push(GateWarning::new(format!(
                        "Task '{}' exceeded timeout: {}ms > {}ms",
                        task.task_id, task.duration_ms, max_timeout
                    )));
                }
            }
        }

        // Determine overall pass/fail
        let passed = blockers.is_empty()
            && warnings.len() as u32 <= config.max_warnings
            && pass_rate >= config.pass_rate_threshold;

        let summary = GateSummary {
            total_tasks,
            passed_tasks,
            failed_tasks,
            error_tasks,
            pass_rate,
        };

        Self {
            level: config.level,
            passed,
            blockers,
            warnings,
            summary,
        }
    }

    /// Returns true if the gate passed.
    pub fn is_passed(&self) -> bool {
        self.passed
    }

    /// Returns the number of blockers.
    pub fn blocker_count(&self) -> usize {
        self.blockers.len()
    }

    /// Returns the number of warnings.
    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }

    /// Returns a formatted summary string.
    pub fn summary_string(&self) -> String {
        format!(
            "{}: {} passed, {} failed, {} errors (pass rate: {:.1}%)",
            self.level.name(),
            self.summary.passed_tasks,
            self.summary.failed_tasks,
            self.summary.error_tasks,
            self.summary.pass_rate * 100.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reporting::report::{ParityReport, TaskResult};
    use crate::types::parity_verdict::{DiffCategory, ParityVerdict};

    fn create_test_report() -> ParityReport {
        let mut report = ParityReport::new("TestRunner");

        // Add 9 passing tasks
        for i in 0..9 {
            report.add_task(TaskResult::new(
                format!("PASS-{}", i),
                ParityVerdict::Pass,
                100,
            ));
        }

        // Add 1 failing task
        report.add_task(TaskResult::new(
            "FAIL-001".to_string(),
            ParityVerdict::Fail {
                category: DiffCategory::OutputText,
                details: "Mismatch".to_string(),
            },
            50,
        ));

        report.compute_summary();
        report
    }

    #[test]
    fn test_gate_level_pass_rate_thresholds() {
        assert!((GateLevel::PR.pass_rate_threshold() - 0.90).abs() < 0.01);
        assert!((GateLevel::Nightly.pass_rate_threshold() - 0.80).abs() < 0.01);
        assert!((GateLevel::Release.pass_rate_threshold() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_gate_level_names() {
        assert_eq!(GateLevel::PR.name(), "PR Gate");
        assert_eq!(GateLevel::Nightly.name(), "Nightly Gate");
        assert_eq!(GateLevel::Release.name(), "Release Gate");
    }

    #[test]
    fn test_gate_config_pr_defaults() {
        let config = GateConfig::pr();
        assert_eq!(config.level, GateLevel::PR);
        assert!((config.pass_rate_threshold - 0.90).abs() < 0.01);
        assert_eq!(config.max_blockers, 0);
        assert_eq!(config.max_warnings, 5);
    }

    #[test]
    fn test_gate_config_nightly_defaults() {
        let config = GateConfig::nightly();
        assert_eq!(config.level, GateLevel::Nightly);
        assert!((config.pass_rate_threshold - 0.80).abs() < 0.01);
    }

    #[test]
    fn test_gate_config_release_defaults() {
        let config = GateConfig::release();
        assert_eq!(config.level, GateLevel::Release);
        assert!((config.pass_rate_threshold - 1.0).abs() < 0.01);
        assert_eq!(config.max_warnings, 0);
    }

    #[test]
    fn test_gate_evaluate_pass() {
        let report = create_test_report();
        let config = GateConfig::pr();
        let gate = CIGate::evaluate(&report, &config);

        assert!(gate.is_passed());
        assert!(gate.blockers.is_empty());
    }

    #[test]
    fn test_gate_evaluate_fail_on_low_pass_rate() {
        let mut report = ParityReport::new("TestRunner");

        // Only 5 out of 10 pass = 50% pass rate, below PR threshold
        for i in 0..5 {
            report.add_task(TaskResult::new(
                format!("PASS-{}", i),
                ParityVerdict::Pass,
                100,
            ));
        }
        for i in 0..5 {
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

        let config = GateConfig::release();
        let gate = CIGate::evaluate(&report, &config);

        assert!(!gate.is_passed());
        assert!(!gate.blockers.is_empty());
    }

    #[test]
    fn test_gate_evaluate_blocked_task() {
        let mut report = ParityReport::new("TestRunner");
        report.add_task(TaskResult::new(
            "TEST-001".to_string(),
            ParityVerdict::Blocked {
                reason: crate::types::parity_verdict::BlockedReason::BinaryNotFound {
                    binary: "opencode".to_string(),
                },
            },
            100,
        ));
        report.compute_summary();

        let config = GateConfig::pr();
        let gate = CIGate::evaluate(&report, &config);

        assert!(!gate.is_passed());
        assert!(!gate.blockers.is_empty());
    }

    #[test]
    fn test_gate_evaluate_error_rate() {
        let mut report = ParityReport::new("TestRunner");

        // 8 errors out of 10 = 80% error rate, should trigger failure
        for i in 0..8 {
            report.add_task(TaskResult::new(
                format!("ERROR-{}", i),
                ParityVerdict::Error {
                    runner: "TestRunner".to_string(),
                    reason: "Test error".to_string(),
                },
                100,
            ));
        }
        for i in 0..2 {
            report.add_task(TaskResult::new(
                format!("PASS-{}", i),
                ParityVerdict::Pass,
                100,
            ));
        }
        report.compute_summary();

        let config = GateConfig::pr();
        let gate = CIGate::evaluate(&report, &config);

        assert!(!gate.is_passed());
        let has_error_rate_failure = gate
            .blockers
            .iter()
            .any(|f| matches!(f, GateFailure::ErrorRateExceeded { .. }));
        assert!(has_error_rate_failure);
    }

    #[test]
    fn test_gate_failure_description() {
        let failure = GateFailure::TooManyRegressions {
            regressed: 5,
            threshold: 10,
        };
        assert!(failure.description().contains("5"));
        assert!(failure.description().contains("10"));

        let blocked = GateFailure::BlockedTask {
            task_id: "TEST-001".to_string(),
            reason: "Binary not found".to_string(),
        };
        assert!(blocked.description().contains("TEST-001"));
        assert!(blocked.description().contains("Binary not found"));
    }

    #[test]
    fn test_gate_warning_with_task_id() {
        let warning = GateWarning::new("Timeout warning").with_task_id("TEST-001");
        assert_eq!(warning.message, "Timeout warning");
        assert_eq!(warning.task_id, Some("TEST-001".to_string()));
    }

    #[test]
    fn test_ci_gate_summary_string() {
        let report = create_test_report();
        let config = GateConfig::pr();
        let gate = CIGate::evaluate(&report, &config);

        let summary = gate.summary_string();
        assert!(summary.contains("PR Gate"));
        assert!(summary.contains("90"));
    }
}