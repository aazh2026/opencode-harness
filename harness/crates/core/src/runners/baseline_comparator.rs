use crate::error::Result;

#[cfg(test)]
use crate::error::ErrorType;
use crate::normalizers::normalizer::Normalizer;
use crate::types::baseline::BaselineRecord;
use crate::types::parity_verdict::{DiffCategory, ParityVerdict};
use crate::types::runner_output::RunnerOutput;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineComparisonResult {
    pub baseline_id: String,
    pub current_legacy_verdict: ParityVerdict,
    pub current_rust_verdict: ParityVerdict,
    pub legacy_regression: bool,
    pub rust_regression: bool,
    pub summary: String,
}

impl BaselineComparisonResult {
    pub fn new(
        baseline_id: String,
        current_legacy_verdict: ParityVerdict,
        current_rust_verdict: ParityVerdict,
        legacy_regression: bool,
        rust_regression: bool,
        summary: String,
    ) -> Self {
        Self {
            baseline_id,
            current_legacy_verdict,
            current_rust_verdict,
            legacy_regression,
            rust_regression,
            summary,
        }
    }

    pub fn no_regression(
        baseline_id: String,
        current_legacy_verdict: ParityVerdict,
        current_rust_verdict: ParityVerdict,
    ) -> Self {
        let summary = format!(
            "No regression detected. Legacy: {}, Rust: {}",
            current_legacy_verdict.summary(),
            current_rust_verdict.summary()
        );
        Self::new(
            baseline_id,
            current_legacy_verdict,
            current_rust_verdict,
            false,
            false,
            summary,
        )
    }

    pub fn legacy_regressed(
        baseline_id: String,
        current_legacy_verdict: ParityVerdict,
        current_rust_verdict: ParityVerdict,
        details: String,
    ) -> Self {
        Self::new(
            baseline_id,
            current_legacy_verdict,
            current_rust_verdict,
            true,
            false,
            format!("Legacy regression detected: {}", details),
        )
    }

    pub fn rust_regressed(
        baseline_id: String,
        current_legacy_verdict: ParityVerdict,
        current_rust_verdict: ParityVerdict,
        details: String,
    ) -> Self {
        Self::new(
            baseline_id,
            current_legacy_verdict,
            current_rust_verdict,
            false,
            true,
            format!("Rust regression detected: {}", details),
        )
    }

    pub fn both_regressed(
        baseline_id: String,
        current_legacy_verdict: ParityVerdict,
        current_rust_verdict: ParityVerdict,
        legacy_details: String,
        rust_details: String,
    ) -> Self {
        Self::new(
            baseline_id,
            current_legacy_verdict,
            current_rust_verdict,
            true,
            true,
            format!(
                "Both regressed. Legacy: {}. Rust: {}",
                legacy_details, rust_details
            ),
        )
    }
}

pub trait BaselineComparator: Send + Sync {
    fn compare_against_baseline(
        &self,
        baseline: &BaselineRecord,
        current_legacy: &Result<RunnerOutput>,
        current_rust: &Result<RunnerOutput>,
    ) -> Result<BaselineComparisonResult>;
}

pub struct DefaultBaselineComparator {
    normalizer: Arc<dyn Normalizer>,
}

impl DefaultBaselineComparator {
    pub fn new(normalizer: Arc<dyn Normalizer>) -> Self {
        Self { normalizer }
    }

    pub fn with_default_normalizer() -> Self {
        Self::new(Arc::new(crate::normalizers::normalizer::NoOpNormalizer))
    }

    fn extract_output_or_default(result: &Result<RunnerOutput>) -> RunnerOutput {
        result.as_ref().ok().cloned().unwrap_or_default()
    }

    fn compute_legacy_verdict(
        baseline_normalized: &str,
        current_output: &RunnerOutput,
        normalizer: &dyn Normalizer,
    ) -> ParityVerdict {
        let current_normalized = normalizer.normalize(&current_output.stdout);
        if current_normalized == baseline_normalized {
            ParityVerdict::Pass
        } else if current_output.exit_code.is_some() && current_output.exit_code != Some(0) {
            ParityVerdict::Fail {
                category: DiffCategory::ExitCode,
                details: format!("Legacy exit code changed: {:?}", current_output.exit_code),
            }
        } else {
            ParityVerdict::Fail {
                category: DiffCategory::OutputText,
                details: format!(
                    "Legacy output differs from baseline. Length: baseline={}, current={}",
                    baseline_normalized.len(),
                    current_normalized.len()
                ),
            }
        }
    }

    fn compute_rust_verdict(
        baseline_normalized: &str,
        current_output: &RunnerOutput,
        normalizer: &dyn Normalizer,
    ) -> ParityVerdict {
        let current_normalized = normalizer.normalize(&current_output.stdout);
        if current_normalized == baseline_normalized {
            ParityVerdict::Pass
        } else if current_output.exit_code.is_some() && current_output.exit_code != Some(0) {
            ParityVerdict::Fail {
                category: DiffCategory::ExitCode,
                details: format!("Rust exit code changed: {:?}", current_output.exit_code),
            }
        } else {
            ParityVerdict::Fail {
                category: DiffCategory::OutputText,
                details: format!(
                    "Rust output differs from baseline. Length: baseline={}, current={}",
                    baseline_normalized.len(),
                    current_normalized.len()
                ),
            }
        }
    }

    fn detect_legacy_regression(
        baseline: &BaselineRecord,
        current_legacy: &Result<RunnerOutput>,
        normalizer: &dyn Normalizer,
    ) -> bool {
        match current_legacy {
            Ok(output) => {
                let baseline_normalized = &baseline.normalized_legacy;
                let current_normalized = normalizer.normalize(&output.stdout);
                baseline_normalized != &current_normalized
            }
            Err(_) => true,
        }
    }

    fn detect_rust_regression(
        baseline: &BaselineRecord,
        current_rust: &Result<RunnerOutput>,
        normalizer: &dyn Normalizer,
    ) -> bool {
        match current_rust {
            Ok(output) => {
                let baseline_normalized = &baseline.normalized_rust;
                let current_normalized = normalizer.normalize(&output.stdout);
                baseline_normalized != &current_normalized
            }
            Err(_) => true,
        }
    }
}

impl BaselineComparator for DefaultBaselineComparator {
    fn compare_against_baseline(
        &self,
        baseline: &BaselineRecord,
        current_legacy: &Result<RunnerOutput>,
        current_rust: &Result<RunnerOutput>,
    ) -> Result<BaselineComparisonResult> {
        let current_legacy_output = Self::extract_output_or_default(current_legacy);
        let current_rust_output = Self::extract_output_or_default(current_rust);

        let current_legacy_verdict = Self::compute_legacy_verdict(
            &baseline.normalized_legacy,
            &current_legacy_output,
            self.normalizer.as_ref(),
        );

        let current_rust_verdict = Self::compute_rust_verdict(
            &baseline.normalized_rust,
            &current_rust_output,
            self.normalizer.as_ref(),
        );

        let legacy_regression =
            Self::detect_legacy_regression(baseline, current_legacy, self.normalizer.as_ref());
        let rust_regression =
            Self::detect_rust_regression(baseline, current_rust, self.normalizer.as_ref());

        let result = if legacy_regression && rust_regression {
            BaselineComparisonResult::both_regressed(
                baseline.id.clone(),
                current_legacy_verdict,
                current_rust_verdict,
                "Legacy output differs from baseline".to_string(),
                "Rust output differs from baseline".to_string(),
            )
        } else if legacy_regression {
            BaselineComparisonResult::legacy_regressed(
                baseline.id.clone(),
                current_legacy_verdict,
                current_rust_verdict,
                "Legacy output differs from baseline".to_string(),
            )
        } else if rust_regression {
            BaselineComparisonResult::rust_regressed(
                baseline.id.clone(),
                current_legacy_verdict,
                current_rust_verdict,
                "Rust output differs from baseline".to_string(),
            )
        } else {
            BaselineComparisonResult::no_regression(
                baseline.id.clone(),
                current_legacy_verdict,
                current_rust_verdict,
            )
        };

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loaders::baseline_loader::{BaselineLoader, DefaultBaselineLoader};
    use crate::normalizers::normalizer::NoOpNormalizer;
    use crate::types::baseline::BaselineMetadata;
    use crate::types::runner_output::RunnerOutput;
    use chrono::Utc;
    use tempfile::TempDir;

    fn create_test_baseline(
        task_id: &str,
        legacy_stdout: &str,
        rust_stdout: &str,
    ) -> BaselineRecord {
        let metadata = BaselineMetadata::new()
            .with_source_impl_version("1.0.0".to_string())
            .with_target_impl_version("2.0.0".to_string())
            .with_task_version("1.2.0".to_string())
            .with_fixture_version("1.1.0".to_string())
            .with_normalizer_version("1.0.5".to_string());

        BaselineRecord::new(
            "baseline-001".to_string(),
            task_id.to_string(),
            metadata,
            RunnerOutput::default().with_stdout(legacy_stdout.to_string()),
            RunnerOutput::default().with_stdout(rust_stdout.to_string()),
            legacy_stdout.to_string(),
            rust_stdout.to_string(),
            ParityVerdict::Pass,
            Utc::now(),
            None,
            None,
        )
    }

    fn create_test_comparator() -> DefaultBaselineComparator {
        DefaultBaselineComparator::new(Arc::new(NoOpNormalizer))
    }

    #[test]
    fn test_compare_against_baseline_detects_identical_outputs() {
        let comparator = create_test_comparator();
        let baseline = create_test_baseline("TASK-001", "same output", "same output");

        let legacy_result: Result<RunnerOutput> =
            Ok(RunnerOutput::default().with_stdout("same output".to_string()));
        let rust_result: Result<RunnerOutput> =
            Ok(RunnerOutput::default().with_stdout("same output".to_string()));

        let result = comparator
            .compare_against_baseline(&baseline, &legacy_result, &rust_result)
            .unwrap();

        assert!(!result.legacy_regression);
        assert!(!result.rust_regression);
        assert!(result.current_legacy_verdict.is_pass());
        assert!(result.current_rust_verdict.is_pass());
    }

    #[test]
    fn test_compare_against_baseline_detects_legacy_regression() {
        let comparator = create_test_comparator();
        let baseline = create_test_baseline("TASK-001", "baseline legacy", "same output");

        let legacy_result: Result<RunnerOutput> =
            Ok(RunnerOutput::default().with_stdout("changed legacy".to_string()));
        let rust_result: Result<RunnerOutput> =
            Ok(RunnerOutput::default().with_stdout("same output".to_string()));

        let result = comparator
            .compare_against_baseline(&baseline, &legacy_result, &rust_result)
            .unwrap();

        assert!(result.legacy_regression);
        assert!(!result.rust_regression);
        assert!(result.current_legacy_verdict.is_different());
        assert!(result.current_rust_verdict.is_pass());
    }

    #[test]
    fn test_compare_against_baseline_detects_rust_regression() {
        let comparator = create_test_comparator();
        let baseline = create_test_baseline("TASK-001", "same output", "baseline rust");

        let legacy_result: Result<RunnerOutput> =
            Ok(RunnerOutput::default().with_stdout("same output".to_string()));
        let rust_result: Result<RunnerOutput> =
            Ok(RunnerOutput::default().with_stdout("changed rust".to_string()));

        let result = comparator
            .compare_against_baseline(&baseline, &legacy_result, &rust_result)
            .unwrap();

        assert!(!result.legacy_regression);
        assert!(result.rust_regression);
        assert!(result.current_legacy_verdict.is_pass());
        assert!(result.current_rust_verdict.is_different());
    }

    #[test]
    fn test_compare_against_baseline_detects_both_regression() {
        let comparator = create_test_comparator();
        let baseline = create_test_baseline("TASK-001", "baseline legacy", "baseline rust");

        let legacy_result: Result<RunnerOutput> =
            Ok(RunnerOutput::default().with_stdout("changed legacy".to_string()));
        let rust_result: Result<RunnerOutput> =
            Ok(RunnerOutput::default().with_stdout("changed rust".to_string()));

        let result = comparator
            .compare_against_baseline(&baseline, &legacy_result, &rust_result)
            .unwrap();

        assert!(result.legacy_regression);
        assert!(result.rust_regression);
        assert!(result.current_legacy_verdict.is_different());
        assert!(result.current_rust_verdict.is_different());
    }

    #[test]
    fn test_compare_against_baseline_handles_legacy_error() {
        let comparator = create_test_comparator();
        let baseline = create_test_baseline("TASK-001", "baseline output", "same output");

        let legacy_result: Result<RunnerOutput> =
            Err(ErrorType::Runner("Legacy runner failed".to_string()));
        let rust_result: Result<RunnerOutput> =
            Ok(RunnerOutput::default().with_stdout("same output".to_string()));

        let result = comparator
            .compare_against_baseline(&baseline, &legacy_result, &rust_result)
            .unwrap();

        assert!(result.legacy_regression);
        assert!(!result.rust_regression);
    }

    #[test]
    fn test_compare_against_baseline_handles_rust_error() {
        let comparator = create_test_comparator();
        let baseline = create_test_baseline("TASK-001", "same output", "baseline output");

        let legacy_result: Result<RunnerOutput> =
            Ok(RunnerOutput::default().with_stdout("same output".to_string()));
        let rust_result: Result<RunnerOutput> =
            Err(ErrorType::Runner("Rust runner failed".to_string()));

        let result = comparator
            .compare_against_baseline(&baseline, &legacy_result, &rust_result)
            .unwrap();

        assert!(!result.legacy_regression);
        assert!(result.rust_regression);
    }

    #[test]
    fn test_compare_against_baseline_variance_handling_whitespace() {
        let comparator = DefaultBaselineComparator::new(Arc::new(
            crate::normalizers::normalizer::WhitespaceNormalizer,
        ));
        let baseline = create_test_baseline("TASK-001", "hello world", "hello world");

        let legacy_result: Result<RunnerOutput> =
            Ok(RunnerOutput::default().with_stdout("  hello   world  ".to_string()));
        let rust_result: Result<RunnerOutput> =
            Ok(RunnerOutput::default().with_stdout("hello   world".to_string()));

        let result = comparator
            .compare_against_baseline(&baseline, &legacy_result, &rust_result)
            .unwrap();

        assert!(!result.legacy_regression);
        assert!(!result.rust_regression);
    }

    #[test]
    fn test_compare_against_baseline_variance_handling_significant_difference() {
        let comparator = create_test_comparator();
        let baseline = create_test_baseline("TASK-001", "hello world", "hello world");

        let legacy_result: Result<RunnerOutput> =
            Ok(RunnerOutput::default().with_stdout("hello world different".to_string()));
        let rust_result: Result<RunnerOutput> =
            Ok(RunnerOutput::default().with_stdout("hello world".to_string()));

        let result = comparator
            .compare_against_baseline(&baseline, &legacy_result, &rust_result)
            .unwrap();

        assert!(result.legacy_regression);
        assert!(!result.rust_regression);
    }

    #[test]
    fn test_compare_against_baseline_missing_baseline() {
        let comparator = create_test_comparator();
        let baseline = create_test_baseline("TASK-001", "legacy", "rust");

        let legacy_result: Result<RunnerOutput> =
            Ok(RunnerOutput::default().with_stdout("legacy".to_string()));
        let rust_result: Result<RunnerOutput> =
            Ok(RunnerOutput::default().with_stdout("rust".to_string()));

        let result = comparator
            .compare_against_baseline(&baseline, &legacy_result, &rust_result)
            .unwrap();

        assert!(!result.legacy_regression);
        assert!(!result.rust_regression);
    }

    #[test]
    fn test_comparison_result_struct() {
        let result = BaselineComparisonResult::no_regression(
            "baseline-001".to_string(),
            ParityVerdict::Pass,
            ParityVerdict::Pass,
        );

        assert_eq!(result.baseline_id, "baseline-001");
        assert!(!result.legacy_regression);
        assert!(!result.rust_regression);
        assert!(result.current_legacy_verdict.is_pass());
        assert!(result.current_rust_verdict.is_pass());
    }

    #[test]
    fn test_comparator_trait_object() {
        let comparator: Box<dyn BaselineComparator> =
            Box::new(DefaultBaselineComparator::with_default_normalizer());
        let baseline = create_test_baseline("TASK-001", "output", "output");

        let legacy_result: Result<RunnerOutput> =
            Ok(RunnerOutput::default().with_stdout("output".to_string()));
        let rust_result: Result<RunnerOutput> =
            Ok(RunnerOutput::default().with_stdout("output".to_string()));

        let result = comparator
            .compare_against_baseline(&baseline, &legacy_result, &rust_result)
            .unwrap();

        assert!(!result.legacy_regression);
        assert!(!result.rust_regression);
    }

    #[test]
    fn test_comparator_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        let comparator = DefaultBaselineComparator::with_default_normalizer();
        assert_send_sync::<DefaultBaselineComparator>();
    }

    #[test]
    fn test_baseline_comparator_with_actual_loader_integration() {
        let temp_dir = TempDir::new().unwrap();
        let loader = DefaultBaselineLoader::new(temp_dir.path().to_path_buf());

        let metadata = BaselineMetadata::new()
            .with_source_impl_version("1.0.0".to_string())
            .with_target_impl_version("2.0.0".to_string())
            .with_task_version("1.2.0".to_string())
            .with_fixture_version("1.1.0".to_string())
            .with_normalizer_version("1.0.5".to_string());

        let baseline = BaselineRecord::new(
            "baseline-integration-001".to_string(),
            "TASK-INTEGRATION".to_string(),
            metadata,
            RunnerOutput::default().with_stdout("original legacy".to_string()),
            RunnerOutput::default().with_stdout("original rust".to_string()),
            "original legacy".to_string(),
            "original rust".to_string(),
            ParityVerdict::Pass,
            Utc::now(),
            None,
            None,
        );

        loader.save(&baseline).unwrap();

        let loaded_baseline = loader.load("TASK-INTEGRATION", "baseline-integration-001");
        assert!(loaded_baseline.is_ok());
        assert!(loaded_baseline.unwrap().is_some());

        let comparator = create_test_comparator();
        let legacy_result: Result<RunnerOutput> =
            Ok(RunnerOutput::default().with_stdout("original legacy".to_string()));
        let rust_result: Result<RunnerOutput> =
            Ok(RunnerOutput::default().with_stdout("original rust".to_string()));

        let result = comparator
            .compare_against_baseline(&baseline, &legacy_result, &rust_result)
            .unwrap();

        assert!(!result.legacy_regression);
        assert!(!result.rust_regression);
    }

    #[test]
    fn baseline_compare_smoke_tests() {
        use crate::loaders::baseline_loader::{BaselineLoader, DefaultBaselineLoader};
        use crate::normalizers::normalizer::NoOpNormalizer;
        use crate::types::baseline::BaselineMetadata;
        use crate::types::runner_output::RunnerOutput;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let loader = DefaultBaselineLoader::new(temp_dir.path().to_path_buf());
        let normalizer = NoOpNormalizer;
        let comparator = DefaultBaselineComparator::new(Arc::new(normalizer));

        let metadata = BaselineMetadata::new()
            .with_source_impl_version("1.0.0".to_string())
            .with_target_impl_version("2.0.0".to_string())
            .with_task_version("1.2.0".to_string())
            .with_fixture_version("1.1.0".to_string())
            .with_normalizer_version("1.0.5".to_string());

        let baseline_legacy_output = RunnerOutput::default()
            .with_exit_code(Some(0))
            .with_stdout("baseline legacy output".to_string())
            .with_stderr("".to_string());

        let baseline_rust_output = RunnerOutput::default()
            .with_exit_code(Some(0))
            .with_stdout("baseline rust output".to_string())
            .with_stderr("".to_string());

        let baseline = BaselineRecord::new(
            "smoke-baseline-001".to_string(),
            "TASK-SMOKE-001".to_string(),
            metadata,
            baseline_legacy_output.clone(),
            baseline_rust_output.clone(),
            "baseline legacy output".to_string(),
            "baseline rust output".to_string(),
            ParityVerdict::Pass,
            Utc::now(),
            None,
            None,
        );

        loader.save(&baseline).unwrap();

        let loaded = loader.load("TASK-SMOKE-001", "smoke-baseline-001");
        assert!(loaded.is_ok(), "Loading baseline should succeed");
        assert!(
            loaded.as_ref().unwrap().is_some(),
            "Baseline should exist after save"
        );

        let loaded_baseline = loaded.unwrap().unwrap();
        assert_eq!(
            loaded_baseline.id, "smoke-baseline-001",
            "Loaded baseline ID should match"
        );
        assert_eq!(
            loaded_baseline.normalized_legacy, "baseline legacy output",
            "Loaded normalized legacy should match"
        );
        assert_eq!(
            loaded_baseline.normalized_rust, "baseline rust output",
            "Loaded normalized rust should match"
        );

        let identical_legacy = RunnerOutput::default()
            .with_exit_code(Some(0))
            .with_stdout("baseline legacy output".to_string());
        let identical_rust = RunnerOutput::default()
            .with_exit_code(Some(0))
            .with_stdout("baseline rust output".to_string());

        let legacy_result: Result<RunnerOutput> = Ok(identical_legacy.clone());
        let rust_result: Result<RunnerOutput> = Ok(identical_rust.clone());

        let identical_result = comparator
            .compare_against_baseline(&loaded_baseline, &legacy_result, &rust_result)
            .unwrap();

        assert!(
            !identical_result.legacy_regression,
            "Identical legacy output should not show regression"
        );
        assert!(
            !identical_result.rust_regression,
            "Identical rust output should not show regression"
        );
        assert!(
            identical_result.current_legacy_verdict.is_pass(),
            "Identical legacy should produce Pass verdict"
        );
        assert!(
            identical_result.current_rust_verdict.is_pass(),
            "Identical rust should produce Pass verdict"
        );

        let changed_legacy = RunnerOutput::default()
            .with_exit_code(Some(0))
            .with_stdout("changed legacy output".to_string());
        let unchanged_rust = RunnerOutput::default()
            .with_exit_code(Some(0))
            .with_stdout("baseline rust output".to_string());

        let legacy_regress_result = comparator
            .compare_against_baseline(&loaded_baseline, &Ok(changed_legacy), &Ok(unchanged_rust))
            .unwrap();

        assert!(
            legacy_regress_result.legacy_regression,
            "Changed legacy output should show legacy regression"
        );
        assert!(
            !legacy_regress_result.rust_regression,
            "Unchanged rust output should not show rust regression"
        );
        assert!(
            legacy_regress_result.current_legacy_verdict.is_different(),
            "Changed legacy should produce Fail verdict"
        );
        assert!(
            legacy_regress_result.current_rust_verdict.is_pass(),
            "Unchanged rust should produce Pass verdict"
        );

        let unchanged_legacy = RunnerOutput::default()
            .with_exit_code(Some(0))
            .with_stdout("baseline legacy output".to_string());
        let changed_rust = RunnerOutput::default()
            .with_exit_code(Some(0))
            .with_stdout("changed rust output".to_string());

        let rust_regress_result = comparator
            .compare_against_baseline(&loaded_baseline, &Ok(unchanged_legacy), &Ok(changed_rust))
            .unwrap();

        assert!(
            !rust_regress_result.legacy_regression,
            "Unchanged legacy output should not show legacy regression"
        );
        assert!(
            rust_regress_result.rust_regression,
            "Changed rust output should show rust regression"
        );
        assert!(
            rust_regress_result.current_legacy_verdict.is_pass(),
            "Unchanged legacy should produce Pass verdict"
        );
        assert!(
            rust_regress_result.current_rust_verdict.is_different(),
            "Changed rust should produce Fail verdict"
        );

        let legacy_error: Result<RunnerOutput> =
            Err(ErrorType::Runner("Legacy runner failed".to_string()));
        let rust_success = RunnerOutput::default()
            .with_exit_code(Some(0))
            .with_stdout("baseline rust output".to_string());

        let error_result = comparator
            .compare_against_baseline(&loaded_baseline, &legacy_error, &Ok(rust_success))
            .unwrap();

        assert!(
            error_result.legacy_regression,
            "Legacy error should show as regression"
        );
        assert!(
            !error_result.rust_regression,
            "Successful rust should not show regression"
        );

        loader
            .delete("TASK-SMOKE-001", "smoke-baseline-001")
            .unwrap();
        let missing = loader.load("TASK-SMOKE-001", "smoke-baseline-001").unwrap();
        assert!(missing.is_none(), "Baseline should be deleted");
    }
}
