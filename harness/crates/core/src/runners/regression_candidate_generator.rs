use crate::error::Result;
use crate::loaders::FixtureLoader;
use crate::normalizers::Normalizer;
use crate::types::execution_level::ExecutionLevel;
use crate::types::failure_classification::FailureClassification;
use crate::types::parity_verdict::{DiffCategory, ParityVerdict};
use crate::types::regression_case::{RegressionCase, RegressionStatus};
use crate::types::severity::Severity;
use chrono::Utc;
use std::path::PathBuf;
use std::sync::Arc;

pub struct RegressionCandidateGenerator {
    fixture_loader: Arc<dyn FixtureLoader>,
    normalizer: Arc<dyn Normalizer>,
}

impl RegressionCandidateGenerator {
    pub fn new(fixture_loader: Arc<dyn FixtureLoader>, normalizer: Arc<dyn Normalizer>) -> Self {
        Self {
            fixture_loader,
            normalizer,
        }
    }

    pub fn generate_candidate(
        &self,
        failed_task_id: &str,
        differential_result: &crate::runners::DifferentialResult,
        issue_link: &str,
    ) -> Result<RegressionCase> {
        let minimal_fixture = self.extract_minimal_fixture(differential_result)?;
        let root_cause = self.generate_root_cause_summary(differential_result);
        let severity = self.determine_severity(&differential_result.verdict);
        let execution_level = self.determine_execution_level(
            differential_result.failure_kind,
            &differential_result.verdict,
            severity,
        );
        let background = self.generate_background(failed_task_id, &differential_result.verdict);
        let expected_result = self.generate_expected_result(&differential_result.verdict);

        let now = Utc::now();
        let regression_id = format!(
            "REG-{}-{}",
            failed_task_id.replace("-", "_"),
            now.format("%Y%m%d%H%M%S")
        );

        Ok(RegressionCase::new(
            regression_id,
            issue_link.to_string(),
            background,
            root_cause,
            minimal_fixture,
            failed_task_id.to_string(),
            expected_result,
            severity,
            execution_level,
            RegressionStatus::Candidate,
            now,
            now,
        ))
    }

    fn extract_minimal_fixture(
        &self,
        result: &crate::runners::DifferentialResult,
    ) -> Result<String> {
        let mut all_artifact_paths: Vec<PathBuf> = Vec::new();
        all_artifact_paths.extend(result.legacy_artifact_paths.clone());
        all_artifact_paths.extend(result.rust_artifact_paths.clone());

        if all_artifact_paths.is_empty() {
            return Ok(String::new());
        }

        let mut sorted_paths: Vec<_> = all_artifact_paths.iter().filter(|p| p.exists()).collect();
        sorted_paths.sort_by_key(|p| std::fs::metadata(p).map(|m| m.len()).unwrap_or(0));

        let smallest_path = sorted_paths.first().map(|p| (*p).clone());

        if let Some(path) = smallest_path {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let normalized = self.normalizer.normalize(&content);
                if !normalized.is_empty() {
                    return Ok(format!(
                        "{} (size: {} bytes)",
                        path.file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| path.to_string_lossy().to_string()),
                        normalized.len()
                    ));
                }
            }
            return Ok(path.to_string_lossy().to_string());
        }

        Ok(String::new())
    }

    fn generate_root_cause_summary(&self, result: &crate::runners::DifferentialResult) -> String {
        match &result.verdict {
            ParityVerdict::Fail { category, details } => {
                format!(
                    "Differential failure in {:?}: {} (legacy artifacts: {}, rust artifacts: {})",
                    category,
                    details,
                    result.legacy_artifact_paths.len(),
                    result.rust_artifact_paths.len()
                )
            }
            ParityVerdict::Error { runner, reason } => {
                format!("Runner {} failed: {}", runner, reason)
            }
            ParityVerdict::ManualCheck { reason, candidates } => {
                format!(
                    "Manual check required: {} ({} mismatch candidates)",
                    reason,
                    candidates.len()
                )
            }
            ParityVerdict::Blocked { reason } => {
                format!("Execution blocked: {:?}", reason)
            }
            _ => {
                format!(
                    "Unexpected verdict: {} (task: {})",
                    result.verdict.summary(),
                    result.task_id
                )
            }
        }
    }

    fn determine_severity(&self, verdict: &ParityVerdict) -> Severity {
        match verdict {
            ParityVerdict::Fail { category, .. } => match category {
                DiffCategory::SideEffects => Severity::Critical,
                DiffCategory::Protocol => Severity::High,
                DiffCategory::OutputText => Severity::Medium,
                DiffCategory::ExitCode => Severity::Medium,
                DiffCategory::Timing => Severity::Low,
            },
            ParityVerdict::Error { .. } => Severity::High,
            ParityVerdict::Blocked { .. } => Severity::High,
            ParityVerdict::ManualCheck { .. } => Severity::Medium,
            _ => Severity::Medium,
        }
    }

    fn determine_execution_level(
        &self,
        failure_kind: Option<FailureClassification>,
        verdict: &ParityVerdict,
        severity: Severity,
    ) -> ExecutionLevel {
        if let Some(kind) = failure_kind {
            match kind {
                FailureClassification::FlakySuspected => {
                    return ExecutionLevel::NightlyOnly;
                }
                FailureClassification::InfraFailure
                | FailureClassification::DependencyMissing
                | FailureClassification::EnvironmentNotSupported => {
                    return ExecutionLevel::ReleaseOnly;
                }
                FailureClassification::ImplementationFailure => {}
            }
        }

        match verdict {
            ParityVerdict::Fail {
                category: DiffCategory::Timing,
                ..
            } => {
                return ExecutionLevel::NightlyOnly;
            }
            ParityVerdict::Fail {
                category: DiffCategory::SideEffects,
                ..
            } => {
                return ExecutionLevel::AlwaysOn;
            }
            _ => {}
        }

        match severity {
            Severity::Critical | Severity::High => ExecutionLevel::AlwaysOn,
            Severity::Medium => ExecutionLevel::NightlyOnly,
            Severity::Low | Severity::Cosmetic => ExecutionLevel::ReleaseOnly,
        }
    }

    fn generate_background(&self, task_id: &str, verdict: &ParityVerdict) -> String {
        match verdict {
            ParityVerdict::Fail { category, details } => {
                format!(
                    "Task '{}' exhibited {:?} failure: {}",
                    task_id, category, details
                )
            }
            ParityVerdict::Error { runner, reason } => {
                format!(
                    "Task '{}' encountered error in {}: {}",
                    task_id, runner, reason
                )
            }
            ParityVerdict::ManualCheck { reason, .. } => {
                format!(
                    "Task '{}' requires manual verification: {}",
                    task_id, reason
                )
            }
            ParityVerdict::Blocked { reason } => {
                format!("Task '{}' was blocked: {:?}", task_id, reason)
            }
            _ => {
                format!(
                    "Task '{}' produced unexpected verdict: {}",
                    task_id,
                    verdict.summary()
                )
            }
        }
    }

    fn generate_expected_result(&self, verdict: &ParityVerdict) -> String {
        match verdict {
            ParityVerdict::Fail { category, details } => {
                format!(
                    "Outputs should match across implementations for {:?}: {}",
                    category, details
                )
            }
            ParityVerdict::Error { runner, .. } => {
                format!("{} should execute without errors", runner)
            }
            ParityVerdict::ManualCheck { reason, .. } => {
                format!(
                    "Results should be reviewed and baseline established: {}",
                    reason
                )
            }
            ParityVerdict::Blocked { reason } => {
                format!("Execution prerequisites should be met: {:?}", reason)
            }
            _ => "Results should match expected baseline".to_string(),
        }
    }
}

impl Default for RegressionCandidateGenerator {
    fn default() -> Self {
        Self {
            fixture_loader: Arc::new(crate::loaders::DefaultFixtureLoader::new(PathBuf::from(
                "fixtures",
            ))),
            normalizer: Arc::new(crate::normalizers::NoOpNormalizer),
        }
    }
}

pub struct DefaultRegressionCandidateGenerator {
    inner: RegressionCandidateGenerator,
}

impl DefaultRegressionCandidateGenerator {
    pub fn new() -> Self {
        Self {
            inner: RegressionCandidateGenerator::default(),
        }
    }

    pub fn with_loader(
        fixture_loader: Arc<dyn FixtureLoader>,
        normalizer: Arc<dyn Normalizer>,
    ) -> Self {
        Self {
            inner: RegressionCandidateGenerator::new(fixture_loader, normalizer),
        }
    }

    pub fn generate_candidate(
        &self,
        failed_task_id: &str,
        differential_result: &crate::runners::DifferentialResult,
        issue_link: &str,
    ) -> Result<RegressionCase> {
        self.inner
            .generate_candidate(failed_task_id, differential_result, issue_link)
    }
}

impl Default for DefaultRegressionCandidateGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runners::DifferentialResult;
    use crate::types::parity_verdict::{DiffCategory, ParityVerdict};
    use tempfile::TempDir;

    fn create_test_differential_result(
        verdict: ParityVerdict,
        failure_kind: Option<FailureClassification>,
    ) -> DifferentialResult {
        DifferentialResult {
            task_id: "TASK-001".to_string(),
            legacy_result: None,
            rust_result: None,
            verdict,
            duration_ms: 100,
            diff_report_path: None,
            verdict_path: None,
            legacy_artifact_paths: Vec::new(),
            rust_artifact_paths: Vec::new(),
            failure_kind,
        }
    }

    #[test]
    fn test_candidate_generation_extracts_minimal_fixture() {
        let temp_dir = TempDir::new().unwrap();
        let fixture_loader = Arc::new(crate::loaders::DefaultFixtureLoader::new(
            temp_dir.path().to_path_buf(),
        ));
        let normalizer = Arc::new(crate::normalizers::NoOpNormalizer);
        let generator = RegressionCandidateGenerator::new(fixture_loader, normalizer);

        let result = create_test_differential_result(
            ParityVerdict::Fail {
                category: DiffCategory::OutputText,
                details: "Output mismatch".to_string(),
            },
            Some(FailureClassification::ImplementationFailure),
        );

        let regression_case = generator
            .generate_candidate("TASK-001", &result, "https://github.com/example/issues/1")
            .unwrap();

        assert!(!regression_case.id.is_empty());
        assert!(regression_case.id.starts_with("REG-"));
        assert_eq!(regression_case.task_id, "TASK-001");
        assert_eq!(regression_case.status, RegressionStatus::Candidate);
        assert_eq!(regression_case.minimal_fixture, "");
    }

    #[test]
    fn test_candidate_generation_with_artifact_paths() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test_artifact.txt");
        std::fs::write(&test_file, "test content").unwrap();

        let fixture_loader = Arc::new(crate::loaders::DefaultFixtureLoader::new(
            temp_dir.path().to_path_buf(),
        ));
        let normalizer = Arc::new(crate::normalizers::NoOpNormalizer);
        let generator = RegressionCandidateGenerator::new(fixture_loader, normalizer);

        let mut result = create_test_differential_result(
            ParityVerdict::Fail {
                category: DiffCategory::OutputText,
                details: "Output mismatch".to_string(),
            },
            Some(FailureClassification::ImplementationFailure),
        );
        result.legacy_artifact_paths.push(test_file.clone());
        result.rust_artifact_paths.push(test_file);

        let regression_case = generator
            .generate_candidate("TASK-001", &result, "https://github.com/example/issues/1")
            .unwrap();

        assert!(regression_case
            .minimal_fixture
            .contains("test_artifact.txt"));
    }

    #[test]
    fn test_root_cause_summary_generated_correctly_for_fail() {
        let temp_dir = TempDir::new().unwrap();
        let fixture_loader = Arc::new(crate::loaders::DefaultFixtureLoader::new(
            temp_dir.path().to_path_buf(),
        ));
        let normalizer = Arc::new(crate::normalizers::NoOpNormalizer);
        let generator = RegressionCandidateGenerator::new(fixture_loader, normalizer);

        let result = create_test_differential_result(
            ParityVerdict::Fail {
                category: DiffCategory::ExitCode,
                details: "Exit codes differ: 0 vs 1".to_string(),
            },
            Some(FailureClassification::ImplementationFailure),
        );

        let regression_case = generator
            .generate_candidate("TASK-002", &result, "https://github.com/example/issues/2")
            .unwrap();

        assert!(regression_case.root_cause.contains("ExitCode"));
        assert!(regression_case.root_cause.contains("Exit codes differ"));
    }

    #[test]
    fn test_root_cause_summary_generated_correctly_for_error() {
        let temp_dir = TempDir::new().unwrap();
        let fixture_loader = Arc::new(crate::loaders::DefaultFixtureLoader::new(
            temp_dir.path().to_path_buf(),
        ));
        let normalizer = Arc::new(crate::normalizers::NoOpNormalizer);
        let generator = RegressionCandidateGenerator::new(fixture_loader, normalizer);

        let result = create_test_differential_result(
            ParityVerdict::Error {
                runner: "RustRunner".to_string(),
                reason: "Binary not found".to_string(),
            },
            Some(FailureClassification::DependencyMissing),
        );

        let regression_case = generator
            .generate_candidate("TASK-003", &result, "https://github.com/example/issues/3")
            .unwrap();

        assert!(regression_case.root_cause.contains("RustRunner"));
        assert!(regression_case.root_cause.contains("Binary not found"));
    }

    #[test]
    fn test_root_cause_summary_generated_correctly_for_manual_check() {
        let temp_dir = TempDir::new().unwrap();
        let fixture_loader = Arc::new(crate::loaders::DefaultFixtureLoader::new(
            temp_dir.path().to_path_buf(),
        ));
        let normalizer = Arc::new(crate::normalizers::NoOpNormalizer);
        let generator = RegressionCandidateGenerator::new(fixture_loader, normalizer);

        let result = create_test_differential_result(
            ParityVerdict::ManualCheck {
                reason: "Ambiguous output".to_string(),
                candidates: vec![],
            },
            None,
        );

        let regression_case = generator
            .generate_candidate("TASK-004", &result, "https://github.com/example/issues/4")
            .unwrap();

        assert!(regression_case.root_cause.contains("Manual check"));
        assert!(regression_case.root_cause.contains("Ambiguous output"));
    }

    #[test]
    fn test_execution_level_set_appropriately_for_implementation_failure() {
        let temp_dir = TempDir::new().unwrap();
        let fixture_loader = Arc::new(crate::loaders::DefaultFixtureLoader::new(
            temp_dir.path().to_path_buf(),
        ));
        let normalizer = Arc::new(crate::normalizers::NoOpNormalizer);
        let generator = RegressionCandidateGenerator::new(fixture_loader, normalizer);

        let result = create_test_differential_result(
            ParityVerdict::Fail {
                category: DiffCategory::SideEffects,
                details: "Side effects differ".to_string(),
            },
            Some(FailureClassification::ImplementationFailure),
        );

        let regression_case = generator
            .generate_candidate("TASK-005", &result, "https://github.com/example/issues/5")
            .unwrap();

        assert_eq!(regression_case.execution_level, ExecutionLevel::AlwaysOn);
    }

    #[test]
    fn test_execution_level_set_appropriately_for_flaky() {
        let temp_dir = TempDir::new().unwrap();
        let fixture_loader = Arc::new(crate::loaders::DefaultFixtureLoader::new(
            temp_dir.path().to_path_buf(),
        ));
        let normalizer = Arc::new(crate::normalizers::NoOpNormalizer);
        let generator = RegressionCandidateGenerator::new(fixture_loader, normalizer);

        let result = create_test_differential_result(
            ParityVerdict::Fail {
                category: DiffCategory::Timing,
                details: "Timing differs".to_string(),
            },
            Some(FailureClassification::FlakySuspected),
        );

        let regression_case = generator
            .generate_candidate("TASK-006", &result, "https://github.com/example/issues/6")
            .unwrap();

        assert_eq!(regression_case.execution_level, ExecutionLevel::NightlyOnly);
    }

    #[test]
    fn test_execution_level_set_appropriately_for_infra_failure() {
        let temp_dir = TempDir::new().unwrap();
        let fixture_loader = Arc::new(crate::loaders::DefaultFixtureLoader::new(
            temp_dir.path().to_path_buf(),
        ));
        let normalizer = Arc::new(crate::normalizers::NoOpNormalizer);
        let generator = RegressionCandidateGenerator::new(fixture_loader, normalizer);

        let result = create_test_differential_result(
            ParityVerdict::Error {
                runner: "LegacyRunner".to_string(),
                reason: "Infra issue".to_string(),
            },
            Some(FailureClassification::InfraFailure),
        );

        let regression_case = generator
            .generate_candidate("TASK-007", &result, "https://github.com/example/issues/7")
            .unwrap();

        assert_eq!(regression_case.execution_level, ExecutionLevel::ReleaseOnly);
    }

    #[test]
    fn test_execution_level_set_appropriately_for_dependency_missing() {
        let temp_dir = TempDir::new().unwrap();
        let fixture_loader = Arc::new(crate::loaders::DefaultFixtureLoader::new(
            temp_dir.path().to_path_buf(),
        ));
        let normalizer = Arc::new(crate::normalizers::NoOpNormalizer);
        let generator = RegressionCandidateGenerator::new(fixture_loader, normalizer);

        let result = create_test_differential_result(
            ParityVerdict::Blocked {
                reason: crate::types::parity_verdict::BlockedReason::DependencyMissing {
                    dependency: "opencode".to_string(),
                },
            },
            Some(FailureClassification::DependencyMissing),
        );

        let regression_case = generator
            .generate_candidate("TASK-008", &result, "https://github.com/example/issues/8")
            .unwrap();

        assert_eq!(regression_case.execution_level, ExecutionLevel::ReleaseOnly);
    }

    #[test]
    fn test_severity_determined_correctly_for_output_text() {
        let temp_dir = TempDir::new().unwrap();
        let fixture_loader = Arc::new(crate::loaders::DefaultFixtureLoader::new(
            temp_dir.path().to_path_buf(),
        ));
        let normalizer = Arc::new(crate::normalizers::NoOpNormalizer);
        let generator = RegressionCandidateGenerator::new(fixture_loader, normalizer);

        let result = create_test_differential_result(
            ParityVerdict::Fail {
                category: DiffCategory::OutputText,
                details: "Output differs".to_string(),
            },
            None,
        );

        let regression_case = generator
            .generate_candidate("TASK-009", &result, "https://github.com/example/issues/9")
            .unwrap();

        assert_eq!(regression_case.severity, Severity::Medium);
    }

    #[test]
    fn test_severity_determined_correctly_for_side_effects() {
        let temp_dir = TempDir::new().unwrap();
        let fixture_loader = Arc::new(crate::loaders::DefaultFixtureLoader::new(
            temp_dir.path().to_path_buf(),
        ));
        let normalizer = Arc::new(crate::normalizers::NoOpNormalizer);
        let generator = RegressionCandidateGenerator::new(fixture_loader, normalizer);

        let result = create_test_differential_result(
            ParityVerdict::Fail {
                category: DiffCategory::SideEffects,
                details: "Side effects differ".to_string(),
            },
            None,
        );

        let regression_case = generator
            .generate_candidate("TASK-010", &result, "https://github.com/example/issues/10")
            .unwrap();

        assert_eq!(regression_case.severity, Severity::Critical);
    }

    #[test]
    fn test_background_generated_correctly() {
        let temp_dir = TempDir::new().unwrap();
        let fixture_loader = Arc::new(crate::loaders::DefaultFixtureLoader::new(
            temp_dir.path().to_path_buf(),
        ));
        let normalizer = Arc::new(crate::normalizers::NoOpNormalizer);
        let generator = RegressionCandidateGenerator::new(fixture_loader, normalizer);

        let result = create_test_differential_result(
            ParityVerdict::Fail {
                category: DiffCategory::OutputText,
                details: "Output differs".to_string(),
            },
            None,
        );

        let regression_case = generator
            .generate_candidate("TASK-011", &result, "https://github.com/example/issues/11")
            .unwrap();

        assert!(regression_case.background.contains("TASK-011"));
        assert!(regression_case.background.contains("OutputText"));
    }

    #[test]
    fn test_expected_result_generated_correctly() {
        let temp_dir = TempDir::new().unwrap();
        let fixture_loader = Arc::new(crate::loaders::DefaultFixtureLoader::new(
            temp_dir.path().to_path_buf(),
        ));
        let normalizer = Arc::new(crate::normalizers::NoOpNormalizer);
        let generator = RegressionCandidateGenerator::new(fixture_loader, normalizer);

        let result = create_test_differential_result(
            ParityVerdict::Fail {
                category: DiffCategory::ExitCode,
                details: "Exit codes differ".to_string(),
            },
            None,
        );

        let regression_case = generator
            .generate_candidate("TASK-012", &result, "https://github.com/example/issues/12")
            .unwrap();

        assert!(regression_case.expected_result.contains("match"));
    }

    #[test]
    fn test_regression_id_format() {
        let temp_dir = TempDir::new().unwrap();
        let fixture_loader = Arc::new(crate::loaders::DefaultFixtureLoader::new(
            temp_dir.path().to_path_buf(),
        ));
        let normalizer = Arc::new(crate::normalizers::NoOpNormalizer);
        let generator = RegressionCandidateGenerator::new(fixture_loader, normalizer);

        let result = create_test_differential_result(ParityVerdict::Pass, None);

        let regression_case = generator
            .generate_candidate(
                "MY-TASK-999",
                &result,
                "https://github.com/example/issues/13",
            )
            .unwrap();

        assert!(regression_case.id.starts_with("REG-MY_TASK_999-"));
    }

    #[test]
    fn test_issue_link_preserved() {
        let temp_dir = TempDir::new().unwrap();
        let fixture_loader = Arc::new(crate::loaders::DefaultFixtureLoader::new(
            temp_dir.path().to_path_buf(),
        ));
        let normalizer = Arc::new(crate::normalizers::NoOpNormalizer);
        let generator = RegressionCandidateGenerator::new(fixture_loader, normalizer);

        let result = create_test_differential_result(
            ParityVerdict::Fail {
                category: DiffCategory::OutputText,
                details: "Output differs".to_string(),
            },
            None,
        );

        let issue_link = "https://github.com/example/project/issues/999";
        let regression_case = generator
            .generate_candidate("TASK-014", &result, issue_link)
            .unwrap();

        assert_eq!(regression_case.issue_link, issue_link);
    }

    #[test]
    fn test_default_regression_candidate_generator() {
        let generator = DefaultRegressionCandidateGenerator::new();
        let result = create_test_differential_result(
            ParityVerdict::Fail {
                category: DiffCategory::OutputText,
                details: "Output differs".to_string(),
            },
            Some(FailureClassification::ImplementationFailure),
        );

        let regression_case = generator
            .generate_candidate("TASK-015", &result, "https://github.com/example/issues/15")
            .unwrap();

        assert_eq!(regression_case.task_id, "TASK-015");
        assert_eq!(regression_case.status, RegressionStatus::Candidate);
    }

    #[test]
    fn test_pipeline_handles_missing_artifacts() {
        let temp_dir = TempDir::new().unwrap();
        let fixture_loader = Arc::new(crate::loaders::DefaultFixtureLoader::new(
            temp_dir.path().to_path_buf(),
        ));
        let normalizer = Arc::new(crate::normalizers::NoOpNormalizer);
        let generator = RegressionCandidateGenerator::new(fixture_loader, normalizer);

        let mut result = create_test_differential_result(
            ParityVerdict::Fail {
                category: DiffCategory::OutputText,
                details: "Output differs".to_string(),
            },
            Some(FailureClassification::ImplementationFailure),
        );
        result
            .legacy_artifact_paths
            .push(temp_dir.path().join("nonexistent.txt"));
        result
            .rust_artifact_paths
            .push(temp_dir.path().join("also_nonexistent.txt"));

        let regression_case = generator
            .generate_candidate("TASK-016", &result, "https://github.com/example/issues/16")
            .unwrap();

        assert_eq!(regression_case.minimal_fixture, "");
        assert_eq!(regression_case.task_id, "TASK-016");
    }

    #[test]
    fn test_timestamps_are_set() {
        let temp_dir = TempDir::new().unwrap();
        let fixture_loader = Arc::new(crate::loaders::DefaultFixtureLoader::new(
            temp_dir.path().to_path_buf(),
        ));
        let normalizer = Arc::new(crate::normalizers::NoOpNormalizer);
        let generator = RegressionCandidateGenerator::new(fixture_loader, normalizer);

        let result = create_test_differential_result(
            ParityVerdict::Fail {
                category: DiffCategory::OutputText,
                details: "Output differs".to_string(),
            },
            None,
        );

        let regression_case = generator
            .generate_candidate("TASK-017", &result, "https://github.com/example/issues/17")
            .unwrap();

        let now = Utc::now();
        assert!(regression_case.created_at <= now);
        assert!(regression_case.created_at >= now - chrono::Duration::seconds(5));
        assert_eq!(regression_case.created_at, regression_case.updated_at);
    }
}
