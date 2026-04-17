use crate::types::parity_verdict::ParityVerdict;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

/// Represents the outcome of a task execution within a parity report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// Unique identifier for the task.
    pub task_id: String,
    /// The parity verdict for this task.
    pub verdict: ParityVerdict,
    /// Execution duration in milliseconds.
    pub duration_ms: u64,
    /// Paths to artifacts produced by this task.
    pub artifacts: Vec<String>,
    /// Error messages if any occurred during execution.
    pub errors: Vec<String>,
}

impl TaskResult {
    /// Creates a new TaskResult with the given task_id and verdict.
    pub fn new(task_id: String, verdict: ParityVerdict, duration_ms: u64) -> Self {
        Self {
            task_id,
            verdict,
            duration_ms,
            artifacts: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Sets artifacts for this task result.
    pub fn with_artifacts(mut self, artifacts: Vec<String>) -> Self {
        self.artifacts = artifacts;
        self
    }

    /// Adds an error message to this task result.
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    /// Returns true if the task passed (identical or equivalent).
    pub fn is_pass(&self) -> bool {
        self.verdict.is_pass()
    }

    /// Returns true if the task had a failure.
    pub fn is_fail(&self) -> bool {
        self.verdict.is_different()
    }

    /// Returns true if the task encountered an error.
    pub fn is_error(&self) -> bool {
        self.verdict.is_error()
    }
}

/// Summary statistics for a parity report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSummary {
    /// Total number of tasks executed.
    pub total_tasks: u32,
    /// Counts of each verdict type.
    pub verdict_counts: BTreeMap<String, u32>,
    /// Aggregate timing statistics in milliseconds.
    pub timing_ms: TimingStatsSummary,
    /// Pass rate as a floating point number between 0.0 and 1.0.
    pub pass_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingStatsSummary {
    /// Total execution time in milliseconds.
    pub total_ms: u64,
    /// Number of tasks with timing data.
    pub count: u32,
}

/// Parity report containing results from a differential run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParityReport {
    /// Unique identifier for this report run.
    pub run_id: Uuid,
    /// Timestamp when the report was generated.
    pub timestamp: DateTime<Utc>,
    /// Type of runner that produced this report.
    pub runner: String,
    /// Individual task results.
    pub task_results: Vec<TaskResult>,
    /// Computed summary statistics.
    pub summary: ReportSummary,
}

impl ParityReport {
    /// Creates a new ParityReport with the given runner type.
    pub fn new(runner: impl Into<String>) -> Self {
        Self {
            run_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            runner: runner.into(),
            task_results: Vec::new(),
            summary: ReportSummary {
                total_tasks: 0,
                verdict_counts: BTreeMap::new(),
                timing_ms: TimingStatsSummary { total_ms: 0, count: 0 },
                pass_rate: 0.0,
            },
        }
    }

    /// Adds a task result to the report.
    pub fn add_task(&mut self, result: TaskResult) {
        self.task_results.push(result);
    }

    /// Computes and updates the summary statistics.
    pub fn compute_summary(&mut self) {
        let total = self.task_results.len() as u32;
        let mut verdict_counts = BTreeMap::new();
        let mut total_duration = 0u64;
        let mut pass_count = 0u32;

        for task in &self.task_results {
            let verdict_key = task.verdict.summary();
            *verdict_counts.entry(verdict_key).or_insert(0) += 1;
            total_duration += task.duration_ms;

            if task.is_pass() {
                pass_count += 1;
            }
        }

        let pass_rate = if total > 0 {
            pass_count as f64 / total as f64
        } else {
            0.0
        };

        self.summary = ReportSummary {
            total_tasks: total,
            verdict_counts,
            timing_ms: TimingStatsSummary {
                total_ms: total_duration,
                count: total,
            },
            pass_rate,
        };
    }

    /// Returns the summary, computing it first if needed.
    pub fn summary(&self) -> &ReportSummary {
        &self.summary
    }

    /// Converts the report to JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Returns the total number of tasks (computed from task_results).
    pub fn total_tasks(&self) -> u32 {
        self.task_results.len() as u32
    }

    /// Returns the number of passing tasks.
    pub fn passed_count(&self) -> u32 {
        self.task_results.iter().filter(|t| t.is_pass()).count() as u32
    }

    /// Returns the number of failing tasks.
    pub fn failed_count(&self) -> u32 {
        self.task_results.iter().filter(|t| t.is_fail()).count() as u32
    }

    /// Returns the number of error tasks.
    pub fn error_count(&self) -> u32 {
        self.task_results.iter().filter(|t| t.is_error()).count() as u32
    }
}

/// Helper struct for timing statistics within reports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingStats {
    /// 50th percentile duration in milliseconds.
    pub p50_ms: f64,
    /// 95th percentile duration in milliseconds.
    pub p95_ms: f64,
    /// 99th percentile duration in milliseconds.
    pub p99_ms: f64,
    /// Total duration in milliseconds.
    pub total_ms: u64,
}

impl Default for TimingStats {
    fn default() -> Self {
        Self {
            p50_ms: 0.0,
            p95_ms: 0.0,
            p99_ms: 0.0,
            total_ms: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::parity_verdict::DiffCategory;

    #[test]
    fn test_task_result_creation() {
        let verdict = ParityVerdict::Pass;
        let result = TaskResult::new("TEST-001".to_string(), verdict, 100);

        assert_eq!(result.task_id, "TEST-001");
        assert!(result.is_pass());
        assert!(!result.is_fail());
        assert!(!result.is_error());
        assert_eq!(result.duration_ms, 100);
        assert!(result.artifacts.is_empty());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_task_result_with_fail_verdict() {
        let verdict = ParityVerdict::Fail {
            category: DiffCategory::OutputText,
            details: "Output mismatch".to_string(),
        };
        let result = TaskResult::new("TEST-002".to_string(), verdict, 50);

        assert!(!result.is_pass());
        assert!(result.is_fail());
        assert!(!result.is_error());
    }

    #[test]
    fn test_task_result_add_error() {
        let verdict = ParityVerdict::Pass;
        let mut result = TaskResult::new("TEST-003".to_string(), verdict, 75);
        result.add_error("Something went wrong".to_string());

        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0], "Something went wrong");
    }

    #[test]
    fn test_parity_report_creation() {
        let report = ParityReport::new("DifferentialRunner");

        assert_eq!(report.runner, "DifferentialRunner");
        assert!(!report.run_id.to_string().is_empty());
        assert!(report.task_results.is_empty());
        assert_eq!(report.total_tasks(), 0);
    }

    #[test]
    fn test_parity_report_add_task() {
        let mut report = ParityReport::new("DifferentialRunner");
        let task = TaskResult::new(
            "TEST-001".to_string(),
            ParityVerdict::Pass,
            100,
        );
        report.add_task(task);

        assert_eq!(report.task_results.len(), 1);
        assert_eq!(report.total_tasks(), 1);
    }

    #[test]
    fn test_parity_report_compute_summary() {
        let mut report = ParityReport::new("DifferentialRunner");

        report.add_task(TaskResult::new(
            "TEST-001".to_string(),
            ParityVerdict::Pass,
            100,
        ));
        report.add_task(TaskResult::new(
            "TEST-002".to_string(),
            ParityVerdict::Pass,
            200,
        ));
        report.add_task(TaskResult::new(
            "TEST-003".to_string(),
            ParityVerdict::Fail {
                category: DiffCategory::OutputText,
                details: "Mismatch".to_string(),
            },
            50,
        ));

        report.compute_summary();

        assert_eq!(report.summary.total_tasks, 3);
        assert!((report.summary.pass_rate - 0.666).abs() < 0.01);
        assert_eq!(report.summary.timing_ms.total_ms, 350);
        assert_eq!(report.passed_count(), 2);
        assert_eq!(report.failed_count(), 1);
    }

    #[test]
    fn test_parity_report_to_json() {
        let mut report = ParityReport::new("TestRunner");
        report.add_task(TaskResult::new(
            "TEST-001".to_string(),
            ParityVerdict::Pass,
            100,
        ));
        report.compute_summary();

        let json = report.to_json().unwrap();
        assert!(json.contains("TEST-001"));
        assert!(json.contains("TestRunner"));
        assert!(json.contains("Pass"));
    }

    #[test]
    fn test_timing_stats_default() {
        let stats = TimingStats::default();
        assert_eq!(stats.p50_ms, 0.0);
        assert_eq!(stats.p95_ms, 0.0);
        assert_eq!(stats.p99_ms, 0.0);
        assert_eq!(stats.total_ms, 0);
    }

    #[test]
    fn test_parity_report_pass_rate_calculation() {
        let mut report = ParityReport::new("TestRunner");

        // Add 4 passing tasks
        for i in 0..4 {
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
                details: "Fail".to_string(),
            },
            50,
        ));

        report.compute_summary();

        assert_eq!(report.summary.total_tasks, 5);
        assert!((report.summary.pass_rate - 0.8).abs() < 0.01);
    }
}