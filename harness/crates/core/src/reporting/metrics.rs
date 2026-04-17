use crate::reporting::report::{ParityReport, TaskResult};
use std::collections::BTreeMap;

/// Timing statistics for task durations.
#[derive(Debug, Clone)]
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

impl TimingStats {
    /// Creates timing stats from a collection of durations in milliseconds.
    pub fn from_durations(durations: &[u64]) -> Self {
        if durations.is_empty() {
            return Self::default();
        }

        let mut sorted = durations.to_vec();
        sorted.sort();

        let total: u64 = sorted.iter().sum();
        let len = sorted.len();

        let p50_ms = if len > 0 {
            if len.is_multiple_of(2) {
                (sorted[len / 2 - 1] as f64 + sorted[len / 2] as f64) / 2.0
            } else {
                sorted[len / 2] as f64
            }
        } else {
            0.0
        };

        let p95_idx = ((len as f64 * 0.95).ceil() as usize).min(len - 1);
        let p99_idx = ((len as f64 * 0.99).ceil() as usize).min(len - 1);

        Self {
            p50_ms,
            p95_ms: sorted[p95_idx] as f64,
            p99_ms: sorted[p99_idx] as f64,
            total_ms: total,
        }
    }
}

/// Collects and computes metrics from task results.
#[derive(Debug, Clone)]
pub struct MetricsCollector {
    /// Total number of tasks processed.
    pub total_tasks: u32,
    /// Counts of each verdict type.
    pub verdict_counts: BTreeMap<String, u32>,
    /// Timing statistics.
    pub timing_stats: TimingStats,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self {
            total_tasks: 0,
            verdict_counts: BTreeMap::new(),
            timing_stats: TimingStats::default(),
        }
    }
}

impl MetricsCollector {
    /// Creates a new empty metrics collector.
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a single task result.
    pub fn record_task(&mut self, task: &TaskResult) {
        self.total_tasks += 1;

        let verdict_key = task.verdict.summary();
        *self.verdict_counts.entry(verdict_key).or_insert(0) += 1;
    }

    /// Computes timing statistics from the recorded durations.
    pub fn compute_stats(&mut self, durations: &[u64]) {
        self.timing_stats = TimingStats::from_durations(durations);
    }

    /// Returns a summary of collected metrics.
    pub fn summary(&self) -> MetricsSummary {
        let pass_count = self
            .verdict_counts
            .iter()
            .filter(|(key, _)| key.starts_with("Pass"))
            .map(|(_, v)| *v)
            .sum::<u32>();

        let total = self.total_tasks.max(1);
        let pass_rate = pass_count as f64 / total as f64;

        MetricsSummary {
            total_tasks: self.total_tasks,
            verdict_counts: self.verdict_counts.clone(),
            timing_stats: self.timing_stats.clone(),
            pass_rate,
        }
    }

    /// Creates a MetricsCollector from a ParityReport.
    pub fn from_report(report: &ParityReport) -> Self {
        let mut collector = Self::new();

        let mut durations = Vec::new();
        for task in &report.task_results {
            collector.record_task(task);
            durations.push(task.duration_ms);
        }

        collector.compute_stats(&durations);
        collector
    }
}

/// Summary of collected metrics.
#[derive(Debug, Clone)]
pub struct MetricsSummary {
    /// Total number of tasks.
    pub total_tasks: u32,
    /// Verdict counts.
    pub verdict_counts: BTreeMap<String, u32>,
    /// Timing statistics.
    pub timing_stats: TimingStats,
    /// Pass rate as a percentage.
    pub pass_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reporting::report::{ParityReport, TaskResult};
    use crate::types::parity_verdict::{DiffCategory, ParityVerdict};

    #[test]
    fn test_timing_stats_from_durations_empty() {
        let stats = TimingStats::from_durations(&[]);
        assert_eq!(stats.p50_ms, 0.0);
        assert_eq!(stats.p95_ms, 0.0);
        assert_eq!(stats.p99_ms, 0.0);
        assert_eq!(stats.total_ms, 0);
    }

    #[test]
    fn test_timing_stats_from_durations_single() {
        let stats = TimingStats::from_durations(&[100]);
        assert_eq!(stats.p50_ms, 100.0);
        assert_eq!(stats.p95_ms, 100.0);
        assert_eq!(stats.p99_ms, 100.0);
        assert_eq!(stats.total_ms, 100);
    }

    #[test]
    fn test_timing_stats_from_durations_multiple() {
        let durations = vec![50, 100, 150, 200, 250];
        let stats = TimingStats::from_durations(&durations);

        assert_eq!(stats.total_ms, 750);
        assert_eq!(stats.p50_ms, 150.0);
        // p95 would be around 225-250
        assert!(stats.p95_ms >= 200.0);
    }

    #[test]
    fn test_metrics_collector_new() {
        let collector = MetricsCollector::new();
        assert_eq!(collector.total_tasks, 0);
        assert!(collector.verdict_counts.is_empty());
    }

    #[test]
    fn test_metrics_collector_record_task() {
        let mut collector = MetricsCollector::new();

        let task = TaskResult::new("TEST-001".to_string(), ParityVerdict::Pass, 100);
        collector.record_task(&task);

        assert_eq!(collector.total_tasks, 1);
        assert_eq!(*collector.verdict_counts.get("Pass").unwrap_or(&0), 1);
    }

    #[test]
    fn test_metrics_collector_from_report() {
        let mut report = ParityReport::new("TestRunner");
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

        let collector = MetricsCollector::from_report(&report);

        assert_eq!(collector.total_tasks, 3);
        assert_eq!(
            *collector.verdict_counts.get("Pass").unwrap_or(&0),
            2
        );
        assert_eq!(
            *collector
                .verdict_counts
                .get("Fail (OutputText): Mismatch")
                .unwrap_or(&0),
            1
        );
        assert_eq!(collector.timing_stats.total_ms, 350);
    }

    #[test]
    fn test_metrics_collector_summary() {
        let mut collector = MetricsCollector::new();

        collector.record_task(&TaskResult::new(
            "TEST-001".to_string(),
            ParityVerdict::Pass,
            100,
        ));
        collector.record_task(&TaskResult::new(
            "TEST-002".to_string(),
            ParityVerdict::Pass,
            200,
        ));
        collector.record_task(&TaskResult::new(
            "TEST-003".to_string(),
            ParityVerdict::Fail {
                category: DiffCategory::ExitCode,
                details: "Code mismatch".to_string(),
            },
            50,
        ));

        collector.compute_stats(&[100, 200, 50]);

        let summary = collector.summary();
        assert_eq!(summary.total_tasks, 3);
        assert!((summary.pass_rate - 0.666).abs() < 0.01);
        assert_eq!(summary.timing_stats.total_ms, 350);
    }
}