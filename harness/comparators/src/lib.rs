use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparisonOutcome {
    Equal,
    Different,
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
            outcome: ComparisonOutcome::Equal,
            diff: None,
            similarity_score: 1.0,
        }
    }

    pub fn different(diff: impl Into<String>) -> Self {
        Self {
            outcome: ComparisonOutcome::Different,
            diff: Some(diff.into()),
            similarity_score: 0.0,
        }
    }

    pub fn incomparable(reason: impl Into<String>) -> Self {
        Self {
            outcome: ComparisonOutcome::Incomparable,
            diff: Some(reason.into()),
            similarity_score: 0.0,
        }
    }
}

pub trait Comparator: Send + Sync {
    fn compare(&self, output1: &str, output2: &str) -> ComparisonResult;

    fn name(&self) -> &str;
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
    fn test_comparison_result_equal() {
        let result = ComparisonResult::equal();
        assert_eq!(result.outcome, ComparisonOutcome::Equal);
        assert!(result.diff.is_none());
        assert_eq!(result.similarity_score, 1.0);
    }

    #[test]
    fn test_comparison_result_different() {
        let result = ComparisonResult::different("output differs");
        assert_eq!(result.outcome, ComparisonOutcome::Different);
        assert_eq!(result.diff, Some("output differs".to_string()));
        assert_eq!(result.similarity_score, 0.0);
    }

    #[test]
    fn test_comparison_result_incomparable() {
        let result = ComparisonResult::incomparable("cannot compare binary data");
        assert_eq!(result.outcome, ComparisonOutcome::Incomparable);
        assert_eq!(result.diff, Some("cannot compare binary data".to_string()));
        assert_eq!(result.similarity_score, 0.0);
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
        assert_eq!(result.outcome, ComparisonOutcome::Equal);

        let result = comparator.compare("hello", "world");
        assert_eq!(result.outcome, ComparisonOutcome::Different);
    }

    #[test]
    fn test_comparator_name() {
        let comparator = TestComparator;
        assert_eq!(comparator.name(), "test-comparator");
    }
}
