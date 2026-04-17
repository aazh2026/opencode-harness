use crate::reporting::gate::CIGate;
use crate::reporting::report::{ParityReport, TaskResult};
use crate::types::parity_verdict::ParityVerdict;
use std::fmt::Write;

/// Trait for rendering parity reports to various output formats.
pub trait ReportRenderer {
    /// Renders the report to the given writer.
    fn render_report(
        &self,
        report: &ParityReport,
        writer: &mut dyn std::fmt::Write,
    ) -> std::fmt::Result;

    /// Renders the CI gate result to the given writer.
    fn render_gate(&self, gate: &CIGate, _writer: &mut dyn std::fmt::Write) -> std::fmt::Result {
        let _ = gate;
        Ok(())
    }
}

/// Renders reports to the console with colors and formatting.
pub struct ConsoleRenderer {
    /// Whether to use colors.
    pub use_color: bool,
}

impl ConsoleRenderer {
    /// Creates a new console renderer.
    pub fn new() -> Self {
        Self { use_color: true }
    }

    /// Creates a new console renderer with optional colors.
    pub fn with_color(use_color: bool) -> Self {
        Self { use_color }
    }

    fn colored(&self, s: &str, color: &str) -> String {
        if self.use_color {
            match color {
                "green" => format!("\x1b[32m{}\x1b[0m", s),
                "red" => format!("\x1b[31m{}\x1b[0m", s),
                "yellow" => format!("\x1b[33m{}\x1b[0m", s),
                "cyan" => format!("\x1b[36m{}\x1b[0m", s),
                "bold" => format!("\x1b[1m{}\x1b[0m", s),
                _ => s.to_string(),
            }
        } else {
            s.to_string()
        }
    }
}

impl Default for ConsoleRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl ReportRenderer for ConsoleRenderer {
    fn render_report(
        &self,
        report: &ParityReport,
        writer: &mut dyn std::fmt::Write,
    ) -> std::fmt::Result {
        writeln!(
            writer,
            "{}",
            self.colored("═══════════════════════════════════════════", "cyan")
        )?;
        writeln!(writer, "{} Parity Report", self.colored("│", "cyan"))?;
        writeln!(
            writer,
            "{} Run ID: {}",
            self.colored("│", "cyan"),
            report.run_id
        )?;
        writeln!(
            writer,
            "{} Runner: {}",
            self.colored("│", "cyan"),
            report.runner
        )?;
        writeln!(
            writer,
            "{} Timestamp: {}",
            self.colored("│", "cyan"),
            report.timestamp
        )?;
        writeln!(
            writer,
            "{}",
            self.colored("═══════════════════════════════════════════", "cyan")
        )?;

        writeln!(writer)?;
        writeln!(writer, "{} Summary", self.colored("▶", "cyan"))?;
        writeln!(writer, "  Total tasks: {}", report.summary.total_tasks)?;
        writeln!(
            writer,
            "  Pass rate: {:.1}%",
            report.summary.pass_rate * 100.0
        )?;
        writeln!(
            writer,
            "  Total time: {}ms",
            report.summary.timing_ms.total_ms
        )?;

        writeln!(writer)?;
        writeln!(writer, "{} Verdict Breakdown", self.colored("▶", "cyan"))?;
        for (verdict, count) in &report.summary.verdict_counts {
            writeln!(writer, "  {}: {}", verdict, count)?;
        }

        writeln!(writer)?;
        writeln!(writer, "{} Task Results", self.colored("▶", "cyan"))?;
        for task in &report.task_results {
            let status = if task.verdict.is_pass() {
                self.colored("✓", "green")
            } else if task.verdict.is_different() {
                self.colored("✗", "red")
            } else if task.verdict.is_error() {
                self.colored("⚠", "yellow")
            } else {
                self.colored("?", "yellow")
            };

            writeln!(
                writer,
                "  {} {} ({}) - {}ms",
                status,
                task.task_id,
                task.verdict.summary(),
                task.duration_ms
            )?;
        }

        Ok(())
    }

    fn render_gate(&self, gate: &CIGate, writer: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(writer)?;
        writeln!(
            writer,
            "{}",
            self.colored("═══════════════════════════════════════════", "cyan")
        )?;
        writeln!(writer, "{} CI Gate Evaluation", self.colored("│", "cyan"))?;
        writeln!(
            writer,
            "{} Level: {}",
            self.colored("│", "cyan"),
            gate.level.name()
        )?;

        if gate.passed {
            writeln!(
                writer,
                "{} Result: {}",
                self.colored("│", "cyan"),
                self.colored("PASSED ✓", "green")
            )?;
        } else {
            writeln!(
                writer,
                "{} Result: {}",
                self.colored("│", "cyan"),
                self.colored("FAILED ✗", "red")
            )?;
        }

        writeln!(
            writer,
            "{}",
            self.colored("═══════════════════════════════════════════", "cyan")
        )?;

        if !gate.blockers.is_empty() {
            writeln!(writer)?;
            writeln!(writer, "{} Blockers:", self.colored("▶", "red"))?;
            for blocker in &gate.blockers {
                writeln!(
                    writer,
                    "  {} {}",
                    self.colored("✗", "red"),
                    blocker.description()
                )?;
            }
        }

        if !gate.warnings.is_empty() {
            writeln!(writer)?;
            writeln!(writer, "{} Warnings:", self.colored("▶", "yellow"))?;
            for warning in &gate.warnings {
                let task_info = warning
                    .task_id
                    .as_ref()
                    .map(|id| format!(" [{}]", id))
                    .unwrap_or_default();
                writeln!(
                    writer,
                    "  {} {}{}",
                    self.colored("⚠", "yellow"),
                    warning.message,
                    task_info
                )?;
            }
        }

        writeln!(writer)?;
        writeln!(writer, "{}", gate.summary_string())?;

        Ok(())
    }
}

/// Renders reports to files in various formats.
pub struct FileRenderer;

impl FileRenderer {
    /// Renders the report as JSON.
    pub fn render_json(&self, report: &ParityReport) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(report)
    }

    /// Renders the report as Markdown.
    pub fn render_markdown(&self, report: &ParityReport, gate: Option<&CIGate>) -> String {
        let mut output = String::new();

        writeln!(&mut output, "# Parity Report").unwrap();
        writeln!(&mut output).unwrap();
        writeln!(&mut output, "**Run ID:** {}", report.run_id).unwrap();
        writeln!(&mut output, "**Runner:** {}", report.runner).unwrap();
        writeln!(&mut output, "**Timestamp:** {}", report.timestamp).unwrap();
        writeln!(
            &mut output,
            "**Pass Rate:** {:.1}%",
            report.summary.pass_rate * 100.0
        )
        .unwrap();

        writeln!(&mut output).unwrap();
        writeln!(&mut output, "## Summary").unwrap();
        writeln!(&mut output).unwrap();
        writeln!(&mut output, "| Metric | Value |").unwrap();
        writeln!(&mut output, "|--------|-------|").unwrap();
        writeln!(
            &mut output,
            "| Total Tasks | {} |",
            report.summary.total_tasks
        )
        .unwrap();
        writeln!(&mut output, "| Passed | {} |", report.passed_count()).unwrap();
        writeln!(&mut output, "| Failed | {} |", report.failed_count()).unwrap();
        writeln!(&mut output, "| Errors | {} |", report.error_count()).unwrap();
        writeln!(
            &mut output,
            "| Total Time | {}ms |",
            report.summary.timing_ms.total_ms
        )
        .unwrap();

        writeln!(&mut output).unwrap();
        writeln!(&mut output, "## Verdict Breakdown").unwrap();
        writeln!(&mut output).unwrap();
        for (verdict, count) in &report.summary.verdict_counts {
            writeln!(&mut output, "- **{}**: {}", verdict, count).unwrap();
        }

        writeln!(&mut output).unwrap();
        writeln!(&mut output, "## Task Results").unwrap();
        writeln!(&mut output).unwrap();
        writeln!(&mut output, "| Task ID | Verdict | Duration |").unwrap();
        writeln!(&mut output, "|---------|---------|----------|").unwrap();
        for task in &report.task_results {
            writeln!(
                &mut output,
                "| {} | {} | {}ms |",
                task.task_id,
                task.verdict.summary(),
                task.duration_ms
            )
            .unwrap();
        }

        if let Some(g) = gate {
            writeln!(&mut output).unwrap();
            writeln!(&mut output, "## CI Gate").unwrap();
            writeln!(&mut output).unwrap();
            writeln!(&mut output, "**Level:** {}", g.level.name()).unwrap();
            writeln!(
                &mut output,
                "**Result:** {}",
                if g.passed { "PASSED" } else { "FAILED" }
            )
            .unwrap();

            if !g.blockers.is_empty() {
                writeln!(&mut output).unwrap();
                writeln!(&mut output, "### Blockers").unwrap();
                for blocker in &g.blockers {
                    writeln!(&mut output, "- {}", blocker.description()).unwrap();
                }
            }

            if !g.warnings.is_empty() {
                writeln!(&mut output).unwrap();
                writeln!(&mut output, "### Warnings").unwrap();
                for warning in &g.warnings {
                    writeln!(
                        &mut output,
                        "- {}{}",
                        warning.message,
                        warning
                            .task_id
                            .as_ref()
                            .map(|id| format!(" [{}]", id))
                            .unwrap_or_default()
                    )
                    .unwrap();
                }
            }
        }

        output
    }
}

impl Default for FileRenderer {
    fn default() -> Self {
        Self
    }
}

impl ReportRenderer for FileRenderer {
    fn render_report(
        &self,
        report: &ParityReport,
        writer: &mut dyn std::fmt::Write,
    ) -> std::fmt::Result {
        let md = self.render_markdown(report, None);
        write!(writer, "{}", md)
    }
}

pub struct JUnitXmlRenderer;

impl JUnitXmlRenderer {
    pub fn new() -> Self {
        Self
    }

    pub fn render_report(&self, report: &ParityReport) -> Result<String, std::fmt::Error> {
        let mut output = String::new();
        self.write_report_header(&mut output)?;

        let tests_total = report.task_results.len() as u32;
        let failures = report.failed_count();
        let errors = report.error_count();
        let skipped = report
            .task_results
            .iter()
            .filter(|t| {
                matches!(
                    t.verdict,
                    ParityVerdict::ManualCheck { .. } | ParityVerdict::Blocked { .. }
                )
            })
            .count() as u32;

        writeln!(
            output,
            r#"<testsuites name="parity-report" tests="{}" failures="{}" errors="{}" skipped="{}">"#,
            tests_total, failures, errors, skipped
        )?;

        let suite_name = if report.suite_info.name.is_empty() {
            "default"
        } else {
            &report.suite_info.name
        };

        let suite_output = self.render_test_suite(suite_name, &report.task_results);
        output.push_str(&suite_output);

        writeln!(output, "</testsuites>")?;

        Ok(output)
    }

    fn write_report_header(&self, writer: &mut String) -> std::fmt::Result {
        writeln!(writer, r#"<?xml version="1.0" encoding="UTF-8"?>"#)
    }

    fn render_test_suite(&self, suite_name: &str, results: &[TaskResult]) -> String {
        let mut output = String::new();

        let tests = results.len() as u32;
        let failures = results.iter().filter(|t| t.is_fail()).count() as u32;
        let errors = results.iter().filter(|t| t.is_error()).count() as u32;
        let skipped = results
            .iter()
            .filter(|t| {
                matches!(
                    t.verdict,
                    ParityVerdict::ManualCheck { .. } | ParityVerdict::Blocked { .. }
                )
            })
            .count() as u32;
        let total_time_ms: u64 = results.iter().map(|r| r.duration_ms).sum();
        let time_secs = total_time_ms as f64 / 1000.0;

        writeln!(
            output,
            r#"  <testsuite name="{}" tests="{}" failures="{}" errors="{}" skipped="{}" time="{}">"#,
            suite_name,
            tests,
            failures,
            errors,
            skipped,
            time_secs
        )
        .unwrap();

        for result in results {
            let case_output = self.render_test_case(result);
            output.push_str(&case_output);
        }

        writeln!(output, "  </testsuite>").unwrap();

        output
    }

    fn render_test_case(&self, result: &TaskResult) -> String {
        let mut output = String::new();
        let time_secs = result.duration_ms as f64 / 1000.0;

        writeln!(
            output,
            r#"    <testcase name="{}" classname="{}" time="{}">"#,
            result.task_id, result.task_id, time_secs
        )
        .unwrap();

        if let Some((result_type, message, type_attr)) =
            self.verdict_to_junit_result(&result.verdict)
        {
            if result_type == "failure" {
                writeln!(
                    output,
                    r#"      <failure message="{}" type="{}">"#,
                    message, type_attr
                )
                .unwrap();
                if let ParityVerdict::Fail { details, .. } = &result.verdict {
                    writeln!(output, "        {}", details).unwrap();
                }
                writeln!(output, "      </failure>").unwrap();
            } else {
                writeln!(
                    output,
                    r#"      <error message="{}" type="{}">"#,
                    message, type_attr
                )
                .unwrap();
                if let ParityVerdict::Error { reason, .. } = &result.verdict {
                    writeln!(output, "        {}", reason).unwrap();
                }
                if let ParityVerdict::Blocked { reason, .. } = &result.verdict {
                    writeln!(output, "        {:?}", reason).unwrap();
                }
                if let ParityVerdict::ManualCheck { reason, .. } = &result.verdict {
                    writeln!(output, "        {}", reason).unwrap();
                }
                writeln!(output, "      </error>").unwrap();
            }
        } else if matches!(
            result.verdict,
            ParityVerdict::ManualCheck { .. } | ParityVerdict::Blocked { .. }
        ) {
            writeln!(output, r#"      <skipped/>"#).unwrap();
        }

        writeln!(output, "    </testcase>").unwrap();

        output
    }

    fn verdict_to_junit_result(&self, verdict: &ParityVerdict) -> Option<(String, String, String)> {
        match verdict {
            ParityVerdict::Pass => None,
            ParityVerdict::PassWithAllowedVariance { .. } => None,
            ParityVerdict::Fail { details, .. } => Some((
                "failure".to_string(),
                details.clone(),
                "ImplementationFailure".to_string(),
            )),
            ParityVerdict::Warn { message, .. } => Some((
                "failure".to_string(),
                message.clone(),
                "Warning".to_string(),
            )),
            ParityVerdict::ManualCheck { reason, .. } => Some((
                "error".to_string(),
                reason.clone(),
                "ManualCheckRequired".to_string(),
            )),
            ParityVerdict::Blocked { reason, .. } => Some((
                "error".to_string(),
                format!("{:?}", reason),
                "Blocked".to_string(),
            )),
            ParityVerdict::Error { runner, reason } => Some((
                "error".to_string(),
                format!("{}: {}", runner, reason),
                runner.clone(),
            )),
        }
    }
}

impl Default for JUnitXmlRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl ReportRenderer for JUnitXmlRenderer {
    fn render_report(
        &self,
        report: &ParityReport,
        writer: &mut dyn std::fmt::Write,
    ) -> std::fmt::Result {
        let xml = self.render_report(report).map_err(|_| std::fmt::Error)?;
        write!(writer, "{}", xml)
    }
}

/// Renders reports in GitHub Actions annotation format.
pub struct GitHubSummaryRenderer;

impl GitHubSummaryRenderer {
    /// Creates a new GitHub summary renderer.
    pub fn new() -> Self {
        Self
    }
}

impl Default for GitHubSummaryRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl ReportRenderer for GitHubSummaryRenderer {
    fn render_report(
        &self,
        report: &ParityReport,
        writer: &mut dyn std::fmt::Write,
    ) -> std::fmt::Result {
        // Output summary for GitHub Actions
        writeln!(writer, "## Parity Report Summary").unwrap();
        writeln!(writer).unwrap();
        writeln!(writer, "| Metric | Value |").unwrap();
        writeln!(writer, "|--------|-------|").unwrap();
        writeln!(writer, "| Runner | {} |", report.runner).unwrap();
        writeln!(writer, "| Total Tasks | {} |", report.summary.total_tasks).unwrap();
        writeln!(
            writer,
            "| Pass Rate | {:.1}% |",
            report.summary.pass_rate * 100.0
        )
        .unwrap();
        writeln!(writer, "| Passed | {} |", report.passed_count()).unwrap();
        writeln!(writer, "| Failed | {} |", report.failed_count()).unwrap();
        writeln!(writer, "| Errors | {} |", report.error_count()).unwrap();
        writeln!(writer).unwrap();

        Ok(())
    }

    fn render_gate(&self, gate: &CIGate, writer: &mut dyn std::fmt::Write) -> std::fmt::Result {
        if gate.passed {
            writeln!(
                writer,
                "::notice ::[{}] CI Gate PASSED - pass rate: {:.1}%",
                gate.level.name(),
                gate.summary.pass_rate * 100.0
            )?;
        } else {
            writeln!(
                writer,
                "::error ::[{}] CI Gate FAILED - pass rate: {:.1}%",
                gate.level.name(),
                gate.summary.pass_rate * 100.0
            )?;
        }

        for blocker in &gate.blockers {
            writeln!(writer, "::error ::Blocker: {}", blocker.description())?;
        }

        for warning in &gate.warnings {
            writeln!(
                writer,
                "::warning ::Warning: {}{}",
                warning.message,
                warning
                    .task_id
                    .as_ref()
                    .map(|id| format!(" ({})", id))
                    .unwrap_or_default()
            )?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reporting::gate::GateConfig;
    use crate::reporting::report::{ParityReport, TaskResult};
    use crate::types::parity_verdict::{DiffCategory, ParityVerdict};

    fn create_test_report() -> ParityReport {
        let mut report = ParityReport::new("TestRunner");
        report.add_task(TaskResult::new(
            "TEST-001".to_string(),
            ParityVerdict::Pass,
            100,
        ));
        report.add_task(TaskResult::new(
            "TEST-002".to_string(),
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
    fn test_console_renderer_creation() {
        let renderer = ConsoleRenderer::new();
        assert!(renderer.use_color);
    }

    #[test]
    fn test_console_renderer_render_report() {
        let renderer = ConsoleRenderer::new();
        let report = create_test_report();
        let mut output = String::new();
        renderer.render_report(&report, &mut output).unwrap();
        assert!(output.contains("Parity Report"));
        assert!(output.contains("TestRunner"));
    }

    #[test]
    fn test_console_renderer_render_gate() {
        let renderer = ConsoleRenderer::new();
        let report = create_test_report();
        let config = GateConfig::pr();
        let gate = CIGate::evaluate(&report, &config);
        let mut output = String::new();
        renderer.render_gate(&gate, &mut output).unwrap();
        assert!(output.contains("CI Gate"));
    }

    #[test]
    fn test_file_renderer_render_json() {
        let renderer = FileRenderer::default();
        let report = create_test_report();
        let json = renderer.render_json(&report).unwrap();
        assert!(json.contains("TEST-001"));
        assert!(json.contains("TestRunner"));
    }

    #[test]
    fn test_file_renderer_render_markdown() {
        let renderer = FileRenderer::default();
        let report = create_test_report();
        let md = renderer.render_markdown(&report, None);
        assert!(md.contains("# Parity Report"));
        assert!(md.contains("| TEST-001 |"));
    }

    #[test]
    fn test_file_renderer_render_markdown_with_gate() {
        let renderer = FileRenderer::default();
        let report = create_test_report();
        let config = GateConfig::pr();
        let gate = CIGate::evaluate(&report, &config);
        let md = renderer.render_markdown(&report, Some(&gate));
        assert!(md.contains("## CI Gate"));
        assert!(md.contains("PASSED") || md.contains("FAILED"));
    }

    #[test]
    fn test_github_summary_renderer_render_report() {
        let renderer = GitHubSummaryRenderer::new();
        let report = create_test_report();
        let mut output = String::new();
        renderer.render_report(&report, &mut output).unwrap();
        assert!(output.contains("Parity Report Summary"));
    }

    #[test]
    fn test_github_summary_renderer_render_gate_passed() {
        let renderer = GitHubSummaryRenderer::new();
        let report = create_test_report();
        let config = GateConfig::pr();
        let gate = CIGate::evaluate(&report, &config);
        let mut output = String::new();
        renderer.render_gate(&gate, &mut output).unwrap();
        assert!(output.contains("::notice ::") || output.contains("::error ::"));
    }

    #[test]
    fn test_colored_output() {
        let renderer = ConsoleRenderer::with_color(true);
        let green = renderer.colored("PASS", "green");
        let red = renderer.colored("FAIL", "red");
        // In actual terminal would have ANSI codes
        assert!(green.contains("PASS"));
        assert!(red.contains("FAIL"));
    }

    #[test]
    fn test_plain_console_renderer() {
        let renderer = ConsoleRenderer::with_color(false);
        let result = renderer.colored("Test", "green");
        assert_eq!(result, "Test");
    }

    #[test]
    fn test_junit_xml_renderer_new() {
        let _renderer = JUnitXmlRenderer::new();
        assert!(true);
    }

    #[test]
    fn test_junit_xml_renderer_render_report_header() {
        let renderer = JUnitXmlRenderer::new();
        let mut report = ParityReport::new("TestRunner");
        report.add_task(TaskResult::new(
            "TEST-001".to_string(),
            ParityVerdict::Pass,
            100,
        ));
        report.compute_summary();

        let xml = renderer.render_report(&report).unwrap();
        assert!(xml.starts_with(r#"<?xml version="1.0" encoding="UTF-8"?>"#));
        assert!(xml.contains(r#"<testsuites name="parity-report""#));
        assert!(xml.contains(r#"</testsuites>"#));
    }

    #[test]
    fn test_junit_xml_renderer_render_test_suite() {
        let renderer = JUnitXmlRenderer::new();
        let results = vec![
            TaskResult::new("TEST-001".to_string(), ParityVerdict::Pass, 100),
            TaskResult::new(
                "TEST-002".to_string(),
                ParityVerdict::Fail {
                    category: DiffCategory::OutputText,
                    details: "Mismatch".to_string(),
                },
                50,
            ),
        ];

        let xml = renderer.render_test_suite("pr-smoke", &results);
        assert!(xml.contains(r#"<testsuite name="pr-smoke""#));
        assert!(xml.contains(r#"tests="2""#));
        assert!(xml.contains(r#"failures="1""#));
        assert!(xml.contains(r#"errors="0""#));
        assert!(xml.contains(r#"</testsuite>"#));
    }

    #[test]
    fn test_junit_xml_renderer_render_test_case_passed() {
        let renderer = JUnitXmlRenderer::new();
        let result = TaskResult::new("TEST-001".to_string(), ParityVerdict::Pass, 105);

        let xml = renderer.render_test_case(&result);
        assert!(xml.contains(r#"<testcase name="TEST-001""#));
        assert!(xml.contains(r#"classname="TEST-001""#));
        assert!(xml.contains(r#"time="0.105""#));
        assert!(xml.contains(r#"</testcase>"#));
        assert!(!xml.contains("<failure>"));
        assert!(!xml.contains("<error>"));
    }

    #[test]
    fn test_junit_xml_renderer_render_test_case_failed() {
        let renderer = JUnitXmlRenderer::new();
        let result = TaskResult::new(
            "TEST-002".to_string(),
            ParityVerdict::Fail {
                category: DiffCategory::OutputText,
                details: "Output mismatch".to_string(),
            },
            89,
        );

        let xml = renderer.render_test_case(&result);
        assert!(xml.contains(r#"<testcase name="TEST-002""#));
        assert!(xml.contains(r#"time="0.089""#));
        assert!(xml.contains(r#"<failure message="Output mismatch""#));
        assert!(xml.contains("Output mismatch"));
        assert!(xml.contains(r#"</failure>"#));
        assert!(xml.contains(r#"</testcase>"#));
    }

    #[test]
    fn test_junit_xml_verdict_to_junit_pass() {
        let renderer = JUnitXmlRenderer::new();
        let result = renderer.verdict_to_junit_result(&ParityVerdict::Pass);
        assert!(result.is_none());
    }

    #[test]
    fn test_junit_xml_verdict_to_junit_fail() {
        let renderer = JUnitXmlRenderer::new();
        let verdict = ParityVerdict::Fail {
            category: DiffCategory::OutputText,
            details: "Test mismatch".to_string(),
        };
        let result = renderer.verdict_to_junit_result(&verdict);
        assert!(result.is_some());
        let (result_type, message, type_attr) = result.unwrap();
        assert_eq!(result_type, "failure");
        assert_eq!(message, "Test mismatch");
        assert_eq!(type_attr, "ImplementationFailure");
    }

    #[test]
    fn test_junit_xml_full_report_rendering() {
        let renderer = JUnitXmlRenderer::new();
        let mut report = ParityReport::new("TestRunner");
        report.add_task(TaskResult::new(
            "TEST-001".to_string(),
            ParityVerdict::Pass,
            100,
        ));
        report.add_task(TaskResult::new(
            "TEST-002".to_string(),
            ParityVerdict::Fail {
                category: DiffCategory::OutputText,
                details: "Mismatch".to_string(),
            },
            50,
        ));
        report.add_task(TaskResult::new(
            "TEST-003".to_string(),
            ParityVerdict::Error {
                runner: "TestRunner".to_string(),
                reason: "Binary not found".to_string(),
            },
            10,
        ));
        report.suite_info.name = "pr-smoke".to_string();
        report.compute_summary();

        let xml = renderer.render_report(&report).unwrap();

        assert!(xml.starts_with(r#"<?xml version="1.0" encoding="UTF-8"?>"#));
        assert!(xml.contains(r#"<testsuites name="parity-report""#));
        assert!(xml.contains(r#"tests="3""#));
        assert!(xml.contains(r#"failures="1""#));
        assert!(xml.contains(r#"errors="1""#));
        assert!(xml.contains(r#"<testsuite name="pr-smoke""#));
        assert!(xml.contains(r#"<testcase name="TEST-001""#));
        assert!(xml.contains(r#"<failure"#));
        assert!(xml.contains(r#"<error"#));
        assert!(xml.contains(r#"</testsuites>"#));
    }

    #[test]
    fn test_junit_xml_renderer_default() {
        let renderer = JUnitXmlRenderer::default();
        let report = ParityReport::new("TestRunner");
        let xml = renderer.render_report(&report).unwrap();
        assert!(xml.contains(r#"<?xml version="1.0" encoding="UTF-8"?>"#));
    }

    #[test]
    fn test_junit_xml_report_renderer_trait() {
        let renderer = JUnitXmlRenderer::new();
        let mut report = ParityReport::new("TestRunner");
        report.add_task(TaskResult::new(
            "TEST-001".to_string(),
            ParityVerdict::Pass,
            100,
        ));
        report.compute_summary();

        let mut output = String::new();
        <JUnitXmlRenderer as ReportRenderer>::render_report(&renderer, &report, &mut output)
            .unwrap();
        assert!(output.starts_with(r#"<?xml version="1.0" encoding="UTF-8"?>"#));
    }

    #[test]
    fn test_junit_xml_error_verdict() {
        let renderer = JUnitXmlRenderer::new();
        let result = TaskResult::new(
            "TEST-ERROR".to_string(),
            ParityVerdict::Error {
                runner: "TestRunner".to_string(),
                reason: "binary not found".to_string(),
            },
            0,
        );

        let xml = renderer.render_test_case(&result);
        assert!(xml.contains(r#"<error message="TestRunner: binary not found""#));
        assert!(xml.contains(r#"type="TestRunner""#));
    }

    #[test]
    fn test_junit_xml_blocked_verdict() {
        let renderer = JUnitXmlRenderer::new();
        let result = TaskResult::new(
            "TEST-BLOCKED".to_string(),
            ParityVerdict::Blocked {
                reason: crate::types::parity_verdict::BlockedReason::BinaryNotFound {
                    binary: "opencode".to_string(),
                },
            },
            0,
        );

        let xml = renderer.render_test_case(&result);
        assert!(xml.contains(r#"<error"#));
        assert!(xml.contains("BinaryNotFound"));
        assert!(xml.contains(r#"type="Blocked""#));
    }

    #[test]
    fn test_junit_xml_manual_check_verdict() {
        let renderer = JUnitXmlRenderer::new();
        let result = TaskResult::new(
            "TEST-MANUAL".to_string(),
            ParityVerdict::ManualCheck {
                reason: "Ambiguous output".to_string(),
                candidates: vec![],
            },
            0,
        );

        let xml = renderer.render_test_case(&result);
        assert!(xml.contains(r#"<error message="Ambiguous output""#));
        assert!(xml.contains(r#"type="ManualCheckRequired""#));
    }

    #[test]
    fn test_junit_xml_warn_verdict() {
        let renderer = JUnitXmlRenderer::new();
        let result = TaskResult::new(
            "TEST-WARN".to_string(),
            ParityVerdict::Warn {
                category: DiffCategory::Timing,
                message: "Timing slightly off".to_string(),
            },
            0,
        );

        let xml = renderer.render_test_case(&result);
        assert!(xml.contains(r#"<failure message="Timing slightly off""#));
        assert!(xml.contains(r#"type="Warning""#));
    }
}
