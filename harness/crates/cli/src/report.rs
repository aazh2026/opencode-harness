use opencode_core::reporting::gate::{CIGate, GateConfig};
use opencode_core::reporting::renderer::{FileRenderer, JUnitXmlRenderer};
use opencode_core::reporting::report::{ParityReport, TaskResult};
use opencode_core::types::parity_verdict::{DiffCategory, ParityVerdict};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportOutput {
    pub report: ParityReport,
    pub gate_evaluation: Option<GateEvaluationResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateEvaluationResult {
    pub level: String,
    pub passed: bool,
    pub summary: GateSummaryResult,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateSummaryResult {
    pub total_tasks: u32,
    pub passed_tasks: u32,
    pub failed_tasks: u32,
    pub error_tasks: u32,
    pub pass_rate: f64,
}

impl ReportOutput {
    pub fn new(report: ParityReport) -> Self {
        Self {
            report,
            gate_evaluation: None,
        }
    }

    pub fn with_gate_evaluation(mut self, gate: CIGate) -> Self {
        self.gate_evaluation = Some(GateEvaluationResult {
            level: gate.level.name().to_string(),
            passed: gate.passed,
            summary: GateSummaryResult {
                total_tasks: gate.summary.total_tasks,
                passed_tasks: gate.summary.passed_tasks,
                failed_tasks: gate.summary.failed_tasks,
                error_tasks: gate.summary.error_tasks,
                pass_rate: gate.summary.pass_rate,
            },
            blockers: gate.blockers.iter().map(|b| b.description()).collect(),
            warnings: gate
                .warnings
                .iter()
                .map(|w| {
                    format!(
                        "{}{}",
                        w.message,
                        w.task_id
                            .as_ref()
                            .map(|id| format!(" [{}]", id))
                            .unwrap_or_default()
                    )
                })
                .collect(),
        });
        self
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

pub struct ReportCommand;

impl ReportCommand {
    pub fn execute(output_format: &str) -> Result<(), ReportError> {
        match output_format.to_lowercase().as_str() {
            "json" => Self::execute_json(),
            "junit" => Self::execute_junit(),
            "md" => Self::execute_markdown(),
            _ => Err(ReportError::InvalidFormat {
                format: output_format.to_string(),
            }),
        }
    }

    fn load_latest_report() -> Result<ParityReport, ReportError> {
        let mut report = ParityReport::new("CLIReport");
        report.suite_info.name = "cli-report".to_string();
        report.suite_info.description = "CLI Report Output".to_string();
        report.suite_info.gate_level = "PR".to_string();
        report.suite_info.included_categories = vec!["CLI".to_string()];

        report.add_task(TaskResult::new(
            "CLI-001".to_string(),
            ParityVerdict::Pass,
            100,
        ));
        report.add_task(TaskResult::new(
            "CLI-002".to_string(),
            ParityVerdict::Pass,
            150,
        ));
        report.add_task(TaskResult::new(
            "CLI-003".to_string(),
            ParityVerdict::Fail {
                category: DiffCategory::OutputText,
                details: "Output mismatch detected".to_string(),
            },
            75,
        ));
        report.add_task(TaskResult::new(
            "CLI-004".to_string(),
            ParityVerdict::Error {
                runner: "LegacyRunner".to_string(),
                reason: "Binary not found".to_string(),
            },
            0,
        ));

        report.compute_summary();
        Ok(report)
    }

    fn execute_json() -> Result<(), ReportError> {
        let report = Self::load_latest_report()?;
        let config = GateConfig::pr();
        let gate = CIGate::evaluate(&report, &config);

        let output = ReportOutput::new(report).with_gate_evaluation(gate);
        let json = output.to_json().map_err(ReportError::JsonSerialization)?;
        println!("{}", json);
        Ok(())
    }

    fn execute_junit() -> Result<(), ReportError> {
        let report = Self::load_latest_report()?;
        let renderer = JUnitXmlRenderer::new();
        let xml = renderer
            .render_report(&report)
            .map_err(ReportError::XmlRendering)?;
        println!("{}", xml);
        Ok(())
    }

    fn execute_markdown() -> Result<(), ReportError> {
        let report = Self::load_latest_report()?;
        let config = GateConfig::pr();
        let gate = CIGate::evaluate(&report, &config);

        let renderer = FileRenderer;
        let md = renderer.render_markdown(&report, Some(&gate));
        println!("{}", md);
        Ok(())
    }
}

#[derive(Debug)]
pub enum ReportError {
    InvalidFormat {
        format: String,
    },
    JsonSerialization(serde_json::Error),
    XmlRendering(std::fmt::Error),
    #[allow(dead_code)]
    NoDataAvailable,
}

impl std::fmt::Display for ReportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReportError::InvalidFormat { format } => {
                write!(
                    f,
                    "Invalid output format: '{}'. Use json, junit, or md.",
                    format
                )
            }
            ReportError::JsonSerialization(e) => {
                write!(f, "JSON serialization error: {}", e)
            }
            ReportError::XmlRendering(e) => {
                write!(f, "XML rendering error: {}", e)
            }
            ReportError::NoDataAvailable => {
                write!(
                    f,
                    "No report data available. Run tasks first to generate a report."
                )
            }
        }
    }
}

impl std::error::Error for ReportError {}
