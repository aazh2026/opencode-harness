use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{
    Comparator, ComparisonOutcome, ComparisonResult, ExactComparator, NormalizedComparator,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliComparatorConfig {
    pub allowed_exit_codes: Vec<u32>,
    pub output_patterns: Vec<String>,
    pub timing_tolerance_ms: Option<u64>,
}

impl Default for CliComparatorConfig {
    fn default() -> Self {
        Self {
            allowed_exit_codes: vec![0],
            output_patterns: vec![],
            timing_tolerance_ms: None,
        }
    }
}

impl CliComparatorConfig {
    pub fn new(
        allowed_exit_codes: Vec<u32>,
        output_patterns: Vec<String>,
        timing_tolerance_ms: Option<u64>,
    ) -> Self {
        Self {
            allowed_exit_codes,
            output_patterns,
            timing_tolerance_ms,
        }
    }

    pub fn with_exit_codes(mut self, codes: Vec<u32>) -> Self {
        self.allowed_exit_codes = codes;
        self
    }

    pub fn with_patterns(mut self, patterns: Vec<String>) -> Self {
        self.output_patterns = patterns;
        self
    }

    pub fn with_timing_tolerance(mut self, ms: u64) -> Self {
        self.timing_tolerance_ms = Some(ms);
        self
    }
}

#[derive(Debug, Clone)]
pub struct CliComparator {
    #[allow(dead_code)]
    exit_code_comparator: ExactComparator,
    output_comparator: NormalizedComparator,
    config: CliComparatorConfig,
}

impl CliComparator {
    pub fn new(config: CliComparatorConfig) -> Self {
        Self {
            exit_code_comparator: ExactComparator,
            output_comparator: NormalizedComparator,
            config,
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(CliComparatorConfig::default())
    }

    pub fn compare_outputs(&self, output1: &str, output2: &str) -> ComparisonResult {
        self.compare(output1, output2)
    }

    pub fn compare_with_exit_code(
        &self,
        output1: &str,
        output2: &str,
        exit_code1: u32,
        exit_code2: u32,
    ) -> ComparisonResult {
        if exit_code1 == exit_code2 {
            return self.output_comparator.compare(output1, output2);
        }

        if self.config.allowed_exit_codes.contains(&exit_code1)
            && self.config.allowed_exit_codes.contains(&exit_code2)
        {
            let output_result = self.output_comparator.compare(output1, output2);
            if output_result.outcome == ComparisonOutcome::StronglyEquivalent
                || output_result.outcome == ComparisonOutcome::SemanticallyEquivalent
            {
                return ComparisonResult::allowed_variance(format!(
                    "Exit code {} vs {} both allowed, outputs equivalent",
                    exit_code1, exit_code2
                ));
            }
            return ComparisonResult::allowed_variance(format!(
                "Exit code {} vs {} both allowed but outputs differ: {}",
                exit_code1,
                exit_code2,
                output_result.diff.unwrap_or_default()
            ));
        }

        ComparisonResult::mildly_incompatible(format!(
            "Exit code mismatch: {} vs {} (allowed: {:?})",
            exit_code1, exit_code2, self.config.allowed_exit_codes
        ))
    }

    pub fn check_output_patterns(&self, output: &str) -> bool {
        if self.config.output_patterns.is_empty() {
            return true;
        }

        for pattern in &self.config.output_patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if !regex.is_match(output) {
                    return false;
                }
            }
        }
        true
    }

    pub fn compare_with_timing(
        &self,
        output1: &str,
        output2: &str,
        timing1_ms: u64,
        timing2_ms: u64,
    ) -> ComparisonResult {
        let output_result = self.output_comparator.compare(output1, output2);

        if let Some(tolerance) = self.config.timing_tolerance_ms {
            let timing_diff = timing1_ms.abs_diff(timing2_ms);

            if timing_diff <= tolerance
                && (output_result.outcome == ComparisonOutcome::StronglyEquivalent
                    || output_result.outcome == ComparisonOutcome::SemanticallyEquivalent)
            {
                return ComparisonResult::allowed_variance(format!(
                    "Timing diff {}ms within tolerance {}ms, outputs equivalent",
                    timing_diff, tolerance
                ));
            }
        }

        output_result
    }
}

impl Comparator for CliComparator {
    fn compare(&self, output1: &str, output2: &str) -> ComparisonResult {
        self.output_comparator.compare(output1, output2)
    }

    fn name(&self) -> &str {
        "cli"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_comparator<T: Comparator>() {}

    #[test]
    fn cli_comparator_timing_exceeds_tolerance() {
        let comparator =
            CliComparator::new(CliComparatorConfig::default().with_timing_tolerance(50));

        let result = comparator.compare_with_timing("output", "output", 50, 150);
        assert_eq!(result.outcome, ComparisonOutcome::SemanticallyEquivalent);
    }

    #[test]
    fn cli_comparator_default_config() {
        let comparator = CliComparator::with_default_config();
        assert_eq!(comparator.name(), "cli");
        assert_eq!(comparator.config.allowed_exit_codes, vec![0]);
        assert!(comparator.config.output_patterns.is_empty());
        assert!(comparator.config.timing_tolerance_ms.is_none());
    }

    #[test]
    fn cli_comparator_compare_outputs() {
        let comparator = CliComparator::with_default_config();
        let result = comparator.compare("hello world", "hello world");
        assert_eq!(result.outcome, ComparisonOutcome::SemanticallyEquivalent);
    }

    #[test]
    fn cli_comparator_compare_outputs_normalized() {
        let comparator = CliComparator::with_default_config();
        let result = comparator.compare("  hello   world  ", "hello world");
        assert_eq!(result.outcome, ComparisonOutcome::SemanticallyEquivalent);
    }

    #[test]
    fn cli_comparator_exit_code_both_match() {
        let comparator =
            CliComparator::new(CliComparatorConfig::default().with_exit_codes(vec![0, 1]));

        let result = comparator.compare_with_exit_code("output", "output", 0, 1);
        assert_eq!(result.outcome, ComparisonOutcome::AllowedVariance);
    }

    #[test]
    fn cli_comparator_exit_code_both_match_with_different_output() {
        let comparator =
            CliComparator::new(CliComparatorConfig::default().with_exit_codes(vec![0, 1]));

        let result = comparator.compare_with_exit_code("hello", "world", 0, 1);
        assert_eq!(result.outcome, ComparisonOutcome::AllowedVariance);
    }

    #[test]
    fn cli_comparator_exit_code_mismatch() {
        let comparator =
            CliComparator::new(CliComparatorConfig::default().with_exit_codes(vec![0, 1]));

        let result = comparator.compare_with_exit_code("output", "output", 0, 2);
        assert_eq!(result.outcome, ComparisonOutcome::MildlyIncompatible);
    }

    #[test]
    fn cli_comparator_exit_code_both_same() {
        let comparator =
            CliComparator::new(CliComparatorConfig::default().with_exit_codes(vec![0, 1]));

        let result = comparator.compare_with_exit_code("hello", "world", 0, 0);
        assert_eq!(result.outcome, ComparisonOutcome::MildlyIncompatible);
    }

    #[test]
    fn cli_comparator_output_patterns_match() {
        let comparator = CliComparator::new(
            CliComparatorConfig::default().with_patterns(vec![r"\d+ items?".to_string()]),
        );

        assert!(comparator.check_output_patterns("5 items found"));
        assert!(comparator.check_output_patterns("0 items"));
        assert!(!comparator.check_output_patterns("no items here"));
    }

    #[test]
    fn cli_comparator_output_patterns_empty_matches_anything() {
        let comparator = CliComparator::with_default_config();
        assert!(comparator.check_output_patterns("anything"));
        assert!(comparator.check_output_patterns(""));
    }

    #[test]
    fn cli_comparator_timing_within_tolerance() {
        let comparator =
            CliComparator::new(CliComparatorConfig::default().with_timing_tolerance(100));

        let result = comparator.compare_with_timing("output", "output", 50, 100);
        assert_eq!(result.outcome, ComparisonOutcome::AllowedVariance);
    }

    #[test]
    fn cli_comparator_config_builder() {
        let config = CliComparatorConfig::new(
            vec![0, 1, 2],
            vec![r"test".to_string(), r"pattern".to_string()],
            Some(500),
        );

        let comparator = CliComparator::new(config);

        assert_eq!(comparator.config.allowed_exit_codes, vec![0, 1, 2]);
        assert_eq!(comparator.config.output_patterns.len(), 2);
        assert_eq!(comparator.config.timing_tolerance_ms, Some(500));
    }

    #[test]
    fn cli_comparator_config_fluent_api() {
        let comparator = CliComparator::new(
            CliComparatorConfig::default()
                .with_exit_codes(vec![0, 1])
                .with_patterns(vec![r"success".to_string()])
                .with_timing_tolerance(200),
        );

        assert!(comparator.check_output_patterns("operation success"));
    }
}
