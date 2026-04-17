use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparisonOutcome {
    StronglyEquivalent,
    SemanticallyEquivalent,
    AllowedVariance,
    MildlyIncompatible,
    SeverelyIncompatible,
    Incomparable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    pub outcome: ComparisonOutcome,
    pub diff: Option<String>,
    pub similarity_score: f64,
}

impl ComparisonResult {
    pub fn equal() -> Self {
        Self {
            outcome: ComparisonOutcome::StronglyEquivalent,
            diff: None,
            similarity_score: 1.0,
        }
    }

    pub fn strongly_equivalent() -> Self {
        Self {
            outcome: ComparisonOutcome::StronglyEquivalent,
            diff: None,
            similarity_score: 1.0,
        }
    }

    pub fn semantically_equivalent() -> Self {
        Self {
            outcome: ComparisonOutcome::SemanticallyEquivalent,
            diff: None,
            similarity_score: 1.0,
        }
    }

    pub fn allowed_variance(details: impl Into<String>) -> Self {
        Self {
            outcome: ComparisonOutcome::AllowedVariance,
            diff: Some(details.into()),
            similarity_score: 0.85,
        }
    }

    pub fn mildly_incompatible(diff: impl Into<String>) -> Self {
        Self {
            outcome: ComparisonOutcome::MildlyIncompatible,
            diff: Some(diff.into()),
            similarity_score: 0.5,
        }
    }

    pub fn severely_incompatible(diff: impl Into<String>) -> Self {
        Self {
            outcome: ComparisonOutcome::SeverelyIncompatible,
            diff: Some(diff.into()),
            similarity_score: 0.0,
        }
    }

    pub fn different(diff: impl Into<String>) -> Self {
        Self {
            outcome: ComparisonOutcome::MildlyIncompatible,
            diff: Some(diff.into()),
            similarity_score: 0.5,
        }
    }

    pub fn incomparable(reason: impl Into<String>) -> Self {
        Self {
            outcome: ComparisonOutcome::Incomparable,
            diff: Some(reason.into()),
            similarity_score: 0.0,
        }
    }

    pub fn different_with_score(diff: impl Into<String>, score: f64) -> Self {
        Self {
            outcome: if score >= 0.7 {
                ComparisonOutcome::MildlyIncompatible
            } else {
                ComparisonOutcome::SeverelyIncompatible
            },
            diff: Some(diff.into()),
            similarity_score: score,
        }
    }
}

pub trait Comparator: Send + Sync {
    fn compare(&self, output1: &str, output2: &str) -> ComparisonResult;

    fn name(&self) -> &str;
}

pub struct ExactComparator;

impl Comparator for ExactComparator {
    fn compare(&self, output1: &str, output2: &str) -> ComparisonResult {
        if output1 == output2 {
            ComparisonResult::strongly_equivalent()
        } else {
            ComparisonResult::different(format!(
                "Exact comparison failed: length1={}, length2={}",
                output1.len(),
                output2.len()
            ))
        }
    }

    fn name(&self) -> &str {
        "exact"
    }
}

pub struct NormalizedComparator;

impl NormalizedComparator {
    pub fn new() -> Self {
        Self
    }

    fn normalize(&self, output: &str) -> String {
        let trimmed = output
            .lines()
            .map(|l| l.trim())
            .collect::<Vec<_>>()
            .join("\n");
        let mut result = String::with_capacity(trimmed.len());
        let mut last_was_space = false;

        for c in trimmed.chars() {
            if c.is_whitespace() {
                if !last_was_space {
                    result.push(' ');
                    last_was_space = true;
                }
            } else {
                result.push(c);
                last_was_space = false;
            }
        }
        result.trim().to_string()
    }
}

impl Default for NormalizedComparator {
    fn default() -> Self {
        Self::new()
    }
}

impl Comparator for NormalizedComparator {
    fn compare(&self, output1: &str, output2: &str) -> ComparisonResult {
        let norm1 = self.normalize(output1);
        let norm2 = self.normalize(output2);

        if norm1 == norm2 {
            ComparisonResult::semantically_equivalent()
        } else {
            ComparisonResult::different(format!(
                "Normalized comparison failed:\n  Expected: '{}'\n  Actual: '{}'",
                norm1, norm2
            ))
        }
    }

    fn name(&self) -> &str {
        "normalized"
    }
}

pub struct SimilarityComparator {
    threshold: f64,
}

impl SimilarityComparator {
    pub fn new(threshold: f64) -> Self {
        Self { threshold }
    }

    fn calculate_similarity(&self, s1: &str, s2: &str) -> f64 {
        if s1 == s2 {
            return 1.0;
        }

        let words1: Vec<&str> = s1.split_whitespace().collect();
        let words2: Vec<&str> = s2.split_whitespace().collect();

        if words1.is_empty() && words2.is_empty() {
            return 1.0;
        }
        if words1.is_empty() || words2.is_empty() {
            return 0.0;
        }

        let mut matching_words = 0;
        for w1 in &words1 {
            if words2.contains(w1) {
                matching_words += 1;
            }
        }

        let union_size = words1.len() + words2.len() - matching_words;
        if union_size == 0 {
            return 1.0;
        }

        matching_words as f64 / union_size as f64
    }
}

impl Comparator for SimilarityComparator {
    fn compare(&self, output1: &str, output2: &str) -> ComparisonResult {
        let similarity = self.calculate_similarity(output1, output2);

        if similarity >= self.threshold {
            ComparisonResult::semantically_equivalent()
        } else {
            ComparisonResult::different_with_score(
                format!(
                    "Similarity {} below threshold {}",
                    similarity, self.threshold
                ),
                similarity,
            )
        }
    }

    fn name(&self) -> &str {
        "similarity"
    }
}

pub struct LineByLineComparator;

impl LineByLineComparator {
    pub fn new() -> Self {
        Self
    }

    fn normalize_line(&self, line: &str) -> String {
        line.trim().to_string()
    }
}

impl Default for LineByLineComparator {
    fn default() -> Self {
        Self::new()
    }
}

impl Comparator for LineByLineComparator {
    fn compare(&self, output1: &str, output2: &str) -> ComparisonResult {
        let lines1: Vec<String> = output1
            .lines()
            .map(|l| self.normalize_line(l))
            .filter(|l| !l.is_empty())
            .collect();
        let lines2: Vec<String> = output2
            .lines()
            .map(|l| self.normalize_line(l))
            .filter(|l| !l.is_empty())
            .collect();

        if lines1 == lines2 {
            return ComparisonResult::strongly_equivalent();
        }

        let mut diff_lines = Vec::new();

        let max_len = lines1.len().max(lines2.len());
        for i in 0..max_len {
            match (lines1.get(i), lines2.get(i)) {
                (Some(l1), Some(l2)) if l1 != l2 => {
                    diff_lines.push(format!("Line {}: '{}' != '{}'", i + 1, l1, l2));
                }
                (Some(l1), None) => {
                    diff_lines.push(format!("Line {}: extra line '{}'", i + 1, l1));
                }
                (None, Some(l2)) => {
                    diff_lines.push(format!("Line {}: missing line '{}'", i + 1, l2));
                }
                _ => {}
            }
        }

        let similarity = if lines1.is_empty() && lines2.is_empty() {
            1.0
        } else {
            let matching = lines1.iter().filter(|l| lines2.contains(l)).count();
            matching as f64 / lines1.len().max(lines2.len()) as f64
        };

        ComparisonResult::different_with_score(diff_lines.join("\n"), similarity)
    }

    fn name(&self) -> &str {
        "line_by_line"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestComparator;

    impl Comparator for TestComparator {
        fn compare(&self, output1: &str, output2: &str) -> ComparisonResult {
            if output1 == output2 {
                ComparisonResult::equal()
            } else {
                ComparisonResult::different(format!("'{}' != '{}'", output1, output2))
            }
        }

        fn name(&self) -> &str {
            "test-comparator"
        }
    }

    #[test]
    fn test_strongly_equivalent_variant_exists() {
        let outcome = ComparisonOutcome::StronglyEquivalent;
        assert_eq!(outcome, ComparisonOutcome::StronglyEquivalent);
    }

    #[test]
    fn test_comparison_result_equal_returns_strongly_equivalent() {
        let result = ComparisonResult::equal();
        assert_eq!(result.outcome, ComparisonOutcome::StronglyEquivalent);
        assert!(result.diff.is_none());
        assert_eq!(result.similarity_score, 1.0);
    }

    #[test]
    fn test_comparison_result_strongly_equivalent() {
        let result = ComparisonResult::strongly_equivalent();
        assert_eq!(result.outcome, ComparisonOutcome::StronglyEquivalent);
        assert!(result.diff.is_none());
        assert_eq!(result.similarity_score, 1.0);
    }

    #[test]
    fn test_semantically_equivalent_variant_exists() {
        let outcome = ComparisonOutcome::SemanticallyEquivalent;
        assert_eq!(outcome, ComparisonOutcome::SemanticallyEquivalent);
    }

    #[test]
    fn test_comparison_result_semantically_equivalent() {
        let result = ComparisonResult::semantically_equivalent();
        assert_eq!(result.outcome, ComparisonOutcome::SemanticallyEquivalent);
        assert!(result.diff.is_none());
        assert_eq!(result.similarity_score, 1.0);
    }

    #[test]
    fn test_normalized_comparator_returns_semantically_equivalent() {
        let comparator = NormalizedComparator;
        let result = comparator.compare("  hello   world  ", "hello world");
        assert_eq!(result.outcome, ComparisonOutcome::SemanticallyEquivalent);
    }

    #[test]
    fn test_similarity_comparator_returns_semantically_equivalent() {
        let comparator = SimilarityComparator::new(0.5);
        let result = comparator.compare("hello world", "hello world");
        assert_eq!(result.outcome, ComparisonOutcome::SemanticallyEquivalent);
    }

    #[test]
    fn test_allowed_variance_variant_exists() {
        let outcome = ComparisonOutcome::AllowedVariance;
        assert_eq!(outcome, ComparisonOutcome::AllowedVariance);
    }

    #[test]
    fn test_comparison_result_allowed_variance_with_timing() {
        let result = ComparisonResult::allowed_variance("Timing diff 50ms within tolerance");
        assert_eq!(result.outcome, ComparisonOutcome::AllowedVariance);
        assert_eq!(
            result.diff,
            Some("Timing diff 50ms within tolerance".to_string())
        );
        assert_eq!(result.similarity_score, 0.85);
    }

    #[test]
    fn test_comparison_result_allowed_variance_with_exit_code() {
        let result = ComparisonResult::allowed_variance("Exit code 0 and 1 both allowed");
        assert_eq!(result.outcome, ComparisonOutcome::AllowedVariance);
        assert_eq!(
            result.diff,
            Some("Exit code 0 and 1 both allowed".to_string())
        );
    }

    #[test]
    fn test_comparison_result_allowed_variance_with_output_pattern() {
        let result = ComparisonResult::allowed_variance("Output matches allowed pattern");
        assert_eq!(result.outcome, ComparisonOutcome::AllowedVariance);
        assert_eq!(
            result.diff,
            Some("Output matches allowed pattern".to_string())
        );
    }

    #[test]
    fn test_mildly_incompatible_variant_exists() {
        let outcome = ComparisonOutcome::MildlyIncompatible;
        assert_eq!(outcome, ComparisonOutcome::MildlyIncompatible);
    }

    #[test]
    fn test_comparison_result_mildly_incompatible() {
        let result = ComparisonResult::mildly_incompatible("Minor formatting difference");
        assert_eq!(result.outcome, ComparisonOutcome::MildlyIncompatible);
        assert_eq!(result.diff, Some("Minor formatting difference".to_string()));
        assert_eq!(result.similarity_score, 0.5);
    }

    #[test]
    fn test_severely_incompatible_variant_exists() {
        let outcome = ComparisonOutcome::SeverelyIncompatible;
        assert_eq!(outcome, ComparisonOutcome::SeverelyIncompatible);
    }

    #[test]
    fn test_comparison_result_severely_incompatible() {
        let result = ComparisonResult::severely_incompatible("Complete output mismatch");
        assert_eq!(result.outcome, ComparisonOutcome::SeverelyIncompatible);
        assert_eq!(result.diff, Some("Complete output mismatch".to_string()));
        assert_eq!(result.similarity_score, 0.0);
    }

    #[test]
    fn test_comparison_result_different_returns_mildly_incompatible() {
        let result = ComparisonResult::different("output differs");
        assert_eq!(result.outcome, ComparisonOutcome::MildlyIncompatible);
        assert_eq!(result.diff, Some("output differs".to_string()));
        assert_eq!(result.similarity_score, 0.5);
    }

    #[test]
    fn test_comparison_result_incomparable() {
        let result = ComparisonResult::incomparable("cannot compare binary data");
        assert_eq!(result.outcome, ComparisonOutcome::Incomparable);
        assert_eq!(result.diff, Some("cannot compare binary data".to_string()));
        assert_eq!(result.similarity_score, 0.0);
    }

    #[test]
    fn test_helper_methods_return_correct_variants() {
        assert_eq!(
            ComparisonResult::strongly_equivalent().outcome,
            ComparisonOutcome::StronglyEquivalent
        );
        assert_eq!(
            ComparisonResult::semantically_equivalent().outcome,
            ComparisonOutcome::SemanticallyEquivalent
        );
        assert_eq!(
            ComparisonResult::allowed_variance("test").outcome,
            ComparisonOutcome::AllowedVariance
        );
        assert_eq!(
            ComparisonResult::mildly_incompatible("test").outcome,
            ComparisonOutcome::MildlyIncompatible
        );
        assert_eq!(
            ComparisonResult::severely_incompatible("test").outcome,
            ComparisonOutcome::SeverelyIncompatible
        );
    }

    #[test]
    fn test_different_with_score_mildly_incompatible_when_high_score() {
        let result = ComparisonResult::different_with_score("test diff", 0.75);
        assert_eq!(result.outcome, ComparisonOutcome::MildlyIncompatible);
    }

    #[test]
    fn test_different_with_score_severely_incompatible_when_low_score() {
        let result = ComparisonResult::different_with_score("test diff", 0.3);
        assert_eq!(result.outcome, ComparisonOutcome::SeverelyIncompatible);
    }

    #[test]
    fn test_comparator_trait_defined() {
        fn assert_comparator<T: Comparator>() {}
        assert_comparator::<TestComparator>();
    }

    #[test]
    fn test_comparator_compare_method_signature() {
        fn takes_comparator(c: &dyn Comparator) {
            let _ = c.compare("output1", "output2");
        }
        takes_comparator(&TestComparator);
    }

    #[test]
    fn test_comparator_accepts_two_string_outputs() {
        let comparator = TestComparator;
        let result = comparator.compare("hello", "hello");
        assert_eq!(result.outcome, ComparisonOutcome::StronglyEquivalent);

        let result = comparator.compare("hello", "world");
        assert_eq!(result.outcome, ComparisonOutcome::MildlyIncompatible);
    }

    #[test]
    fn test_comparator_name() {
        let comparator = TestComparator;
        assert_eq!(comparator.name(), "test-comparator");
    }

    #[test]
    fn test_exact_comparator_returns_strongly_equivalent() {
        let comparator = ExactComparator;
        let result = comparator.compare("hello world", "hello world");
        assert_eq!(result.outcome, ComparisonOutcome::StronglyEquivalent);
    }

    #[test]
    fn test_exact_comparator_returns_mildly_incompatible() {
        let comparator = ExactComparator;
        let result = comparator.compare("hello", "world");
        assert_eq!(result.outcome, ComparisonOutcome::MildlyIncompatible);
        assert!(result.diff.is_some());
    }

    #[test]
    fn test_normalized_comparator_equal() {
        let comparator = NormalizedComparator;
        let result = comparator.compare("  hello   world  ", "hello world");
        assert_eq!(result.outcome, ComparisonOutcome::SemanticallyEquivalent);
    }

    #[test]
    fn test_normalized_comparator_different() {
        let comparator = NormalizedComparator;
        let result = comparator.compare("hello world", "hello world!");
        assert_eq!(result.outcome, ComparisonOutcome::MildlyIncompatible);
    }

    #[test]
    fn test_normalized_comparator_multiline() {
        let comparator = NormalizedComparator;
        let result = comparator.compare("  line1  \n  line2  ", "line1 line2");
        assert_eq!(result.outcome, ComparisonOutcome::SemanticallyEquivalent);
    }

    #[test]
    fn test_similarity_comparator_identical() {
        let comparator = SimilarityComparator::new(0.5);
        let result = comparator.compare("hello world", "hello world");
        assert_eq!(result.outcome, ComparisonOutcome::SemanticallyEquivalent);
    }

    #[test]
    fn test_similarity_comparator_similar() {
        let comparator = SimilarityComparator::new(0.3);
        let result = comparator.compare("hello world", "hello world!");
        assert_eq!(result.outcome, ComparisonOutcome::SemanticallyEquivalent);
    }

    #[test]
    fn test_similarity_comparator_different() {
        let comparator = SimilarityComparator::new(0.9);
        let result = comparator.compare("hello world", "goodbye world");
        assert_eq!(result.outcome, ComparisonOutcome::SeverelyIncompatible);
    }

    #[test]
    fn test_line_by_line_comparator_equal() {
        let comparator = LineByLineComparator;
        let result = comparator.compare("line1\nline2", "line1\nline2");
        assert_eq!(result.outcome, ComparisonOutcome::StronglyEquivalent);
    }

    #[test]
    fn test_line_by_line_comparator_different() {
        let comparator = LineByLineComparator;
        let result = comparator.compare("line1\nline2", "line1\nline3");
        assert!(
            result.outcome == ComparisonOutcome::MildlyIncompatible
                || result.outcome == ComparisonOutcome::SeverelyIncompatible
        );
    }

    #[test]
    fn test_all_comparators_implement_trait() {
        fn assert_comparator<T: Comparator>() {}
        assert_comparator::<ExactComparator>();
        assert_comparator::<NormalizedComparator>();
        assert_comparator::<SimilarityComparator>();
        assert_comparator::<LineByLineComparator>();
    }
}
