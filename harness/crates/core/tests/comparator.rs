use comparators::{
    Comparator, ComparisonOutcome, ComparisonResult, ExactComparator, LineByLineComparator,
    NormalizedComparator, SimilarityComparator,
};
use opencode_core::loaders::DefaultTaskLoader;
use opencode_core::runners::DifferentialRunner;

#[test]
fn test_comparator_trait_is_defined() {
    fn assert_comparator<T: Comparator>() {}
    assert_comparator::<ExactComparator>();
    assert_comparator::<NormalizedComparator>();
    assert_comparator::<SimilarityComparator>();
    assert_comparator::<LineByLineComparator>();
}

#[test]
fn test_comparator_trait_has_name_method() {
    let comparator = ExactComparator;
    assert_eq!(comparator.name(), "exact");

    let comparator = NormalizedComparator;
    assert_eq!(comparator.name(), "normalized");

    let comparator = SimilarityComparator::new(0.5);
    assert_eq!(comparator.name(), "similarity");

    let comparator = LineByLineComparator;
    assert_eq!(comparator.name(), "line_by_line");
}

#[test]
fn test_comparator_trait_has_compare_method() {
    fn takes_comparator(c: &dyn Comparator) {
        let _ = c.compare("output1", "output2");
    }

    takes_comparator(&ExactComparator);
    takes_comparator(&NormalizedComparator);
    takes_comparator(&SimilarityComparator::new(0.5));
    takes_comparator(&LineByLineComparator);
}

#[test]
fn test_exact_comparator_compare_method() {
    let comparator = ExactComparator;

    let result = comparator.compare("hello", "hello");
    assert_eq!(result.outcome, ComparisonOutcome::Equal);
    assert!(result.diff.is_none());
    assert_eq!(result.similarity_score, 1.0);

    let result = comparator.compare("hello", "world");
    assert_eq!(result.outcome, ComparisonOutcome::Different);
    assert!(result.diff.is_some());
    assert_eq!(result.similarity_score, 0.0);
}

#[test]
fn test_normalized_comparator_compare_method() {
    let comparator = NormalizedComparator;

    let result = comparator.compare("  hello   world  ", "hello world");
    assert_eq!(result.outcome, ComparisonOutcome::Equal);

    let result = comparator.compare("hello\nworld", "hello world");
    assert_eq!(result.outcome, ComparisonOutcome::Equal);

    let result = comparator.compare("hello", "goodbye");
    assert_eq!(result.outcome, ComparisonOutcome::Different);
}

#[test]
fn test_similarity_comparator_compare_method() {
    let comparator = SimilarityComparator::new(0.5);

    let result = comparator.compare("hello world", "hello world");
    assert_eq!(result.outcome, ComparisonOutcome::Equal);

    let result = comparator.compare("hello world", "hello world!");
    assert!(result.similarity_score > 0.0);

    let result = comparator.compare("hello", "goodbye");
    assert!(result.similarity_score < 1.0);
}

#[test]
fn test_line_by_line_comparator_compare_method() {
    let comparator = LineByLineComparator;

    let result = comparator.compare("line1\nline2", "line1\nline2");
    assert_eq!(result.outcome, ComparisonOutcome::Equal);

    let result = comparator.compare("line1\nline2", "line1\nline3");
    assert_eq!(result.outcome, ComparisonOutcome::Different);
    assert!(result.diff.is_some());
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
    let result = ComparisonResult::different("outputs differ");
    assert_eq!(result.outcome, ComparisonOutcome::Different);
    assert_eq!(result.diff, Some("outputs differ".to_string()));
    assert_eq!(result.similarity_score, 0.0);
}

#[test]
fn test_comparison_result_incomparable() {
    let result = ComparisonResult::incomparable("cannot compare");
    assert_eq!(result.outcome, ComparisonOutcome::Incomparable);
    assert_eq!(result.diff, Some("cannot compare".to_string()));
    assert_eq!(result.similarity_score, 0.0);
}

#[test]
fn test_comparison_outcome_enum() {
    assert_eq!(ComparisonOutcome::Equal, ComparisonOutcome::Equal);
    assert_eq!(ComparisonOutcome::Different, ComparisonOutcome::Different);
    assert_eq!(
        ComparisonOutcome::Incomparable,
        ComparisonOutcome::Incomparable
    );
}

#[test]
fn test_different_comparators_produce_different_results() {
    let comparator_exact = ExactComparator;
    let comparator_normalized = NormalizedComparator;

    let result_exact = comparator_exact.compare("  hello  ", "hello");
    let result_normalized = comparator_normalized.compare("  hello  ", "hello");

    assert_eq!(result_exact.outcome, ComparisonOutcome::Different);
    assert_eq!(result_normalized.outcome, ComparisonOutcome::Equal);
}

#[test]
fn test_comparator_with_empty_strings() {
    let comparator = ExactComparator;
    let result = comparator.compare("", "");
    assert_eq!(result.outcome, ComparisonOutcome::Equal);
}

#[test]
fn test_comparator_with_multiline_output() {
    let comparator = LineByLineComparator;
    let result = comparator.compare("a\nb\nc", "a\nb\nc");
    assert_eq!(result.outcome, ComparisonOutcome::Equal);
}

#[test]
fn test_comparator_integration_with_differential_runner() {
    let loader = DefaultTaskLoader::new();
    let _runner = DifferentialRunner::new(loader);

    let comparator = NormalizedComparator;
    let result1 = comparator.compare("hello", "hello");
    let result2 = comparator.compare("hello", "world");

    assert_eq!(result1.outcome, ComparisonOutcome::Equal);
    assert_eq!(result2.outcome, ComparisonOutcome::Different);
}

#[test]
fn test_comparator_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ExactComparator>();
    assert_send_sync::<NormalizedComparator>();
    assert_send_sync::<SimilarityComparator>();
    assert_send_sync::<LineByLineComparator>();
}

#[test]
fn test_comparison_result_is_cloneable() {
    fn assert_clone<T: Clone>() {}
    assert_clone::<ComparisonOutcome>();
    assert_clone::<ComparisonResult>();
}
