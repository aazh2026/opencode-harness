use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizerAudit {
    pub applied_rules: Vec<AppliedRule>,
    pub input_hash: String,
    pub output_hash: String,
    pub transformations: Vec<Transformation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppliedRule {
    pub rule_name: String,
    pub rule_version: String,
    pub conditions_matched: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transformation {
    pub transformation_type: String,
    pub description: String,
    pub before_length: usize,
    pub after_length: usize,
}

pub trait Normalizer: Send + Sync {
    fn normalize(&self, output: &str) -> String;
    fn name(&self) -> &str;
    fn describe_rule(&self) -> String;
    fn audit_normalize(&self, output: &str) -> (String, Transformation);
}

pub struct NoOpNormalizer;

impl NoOpNormalizer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NoOpNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Normalizer for NoOpNormalizer {
    fn normalize(&self, output: &str) -> String {
        output.to_string()
    }

    fn name(&self) -> &str {
        "noop"
    }

    fn describe_rule(&self) -> String {
        "NoOp".to_string()
    }

    fn audit_normalize(&self, output: &str) -> (String, Transformation) {
        let before_len = output.len();
        let after_len = before_len;
        (
            output.to_string(),
            Transformation {
                transformation_type: "none".to_string(),
                description: "No transformation applied".to_string(),
                before_length: before_len,
                after_length: after_len,
            },
        )
    }
}

pub struct WhitespaceNormalizer;

impl WhitespaceNormalizer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WhitespaceNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Normalizer for WhitespaceNormalizer {
    fn normalize(&self, output: &str) -> String {
        crate::normalizers::whitespace::normalize(output)
    }

    fn name(&self) -> &str {
        "whitespace"
    }

    fn describe_rule(&self) -> String {
        "Whitespace normalization rules: trim lines, collapse multiple spaces/tabs to single space, join lines with spaces".to_string()
    }

    fn audit_normalize(&self, output: &str) -> (String, Transformation) {
        let before_len = output.len();
        let normalized = self.normalize(output);
        let after_len = normalized.len();
        (
            normalized,
            Transformation {
                transformation_type: "whitespace".to_string(),
                description: "Trim lines, collapse whitespace, join with spaces".to_string(),
                before_length: before_len,
                after_length: after_len,
            },
        )
    }
}

pub struct LineEndingNormalizer;

impl LineEndingNormalizer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LineEndingNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Normalizer for LineEndingNormalizer {
    fn normalize(&self, output: &str) -> String {
        crate::normalizers::line_endings::normalize(output)
    }

    fn name(&self) -> &str {
        "line_endings"
    }

    fn describe_rule(&self) -> String {
        "Line ending normalization rules: convert CRLF (\\r\\n) to LF (\\n), convert CR (\\r) to LF (\\n)".to_string()
    }

    fn audit_normalize(&self, output: &str) -> (String, Transformation) {
        let before_len = output.len();
        let normalized = self.normalize(output);
        let after_len = normalized.len();
        (
            normalized,
            Transformation {
                transformation_type: "line_endings".to_string(),
                description: "Normalize line endings to LF".to_string(),
                before_length: before_len,
                after_length: after_len,
            },
        )
    }
}

pub struct PathNormalizer;

impl PathNormalizer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PathNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Normalizer for PathNormalizer {
    fn normalize(&self, output: &str) -> String {
        crate::normalizers::paths::normalize(output)
    }

    fn name(&self) -> &str {
        "path"
    }

    fn describe_rule(&self) -> String {
        "Path normalization rules: convert backslashes to forward slashes on Windows".to_string()
    }

    fn audit_normalize(&self, output: &str) -> (String, Transformation) {
        let before_len = output.len();
        let normalized = self.normalize(output);
        let after_len = normalized.len();
        (
            normalized,
            Transformation {
                transformation_type: "path".to_string(),
                description: "Normalize path separators".to_string(),
                before_length: before_len,
                after_length: after_len,
            },
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedOutput {
    pub stdout: String,
    pub stderr: String,
    pub normalized: bool,
}

impl NormalizedOutput {
    pub fn new(stdout: impl Into<String>, stderr: impl Into<String>) -> Self {
        Self {
            stdout: stdout.into(),
            stderr: stderr.into(),
            normalized: false,
        }
    }

    pub fn with_normalized(stdout: impl Into<String>, stderr: impl Into<String>) -> Self {
        Self {
            stdout: stdout.into(),
            stderr: stderr.into(),
            normalized: true,
        }
    }

    pub fn from_output(output: &std::process::Output) -> Self {
        Self::new(
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        )
    }

    pub fn apply<N: Normalizer + ?Sized>(&self, normalizer: &N) -> Self {
        Self::with_normalized(
            normalizer.normalize(&self.stdout),
            normalizer.normalize(&self.stderr),
        )
    }

    pub fn apply_multiple<N: Normalizer + ?Sized>(&self, normalizers: &[&N]) -> Self {
        let mut result = self.clone();
        for normalizer in normalizers {
            result = result.apply(*normalizer);
        }
        result
    }

    pub fn stdout_normalized(&self) -> String {
        if self.normalized {
            self.stdout.clone()
        } else {
            WhitespaceNormalizer.normalize(&self.stdout)
        }
    }

    pub fn stderr_normalized(&self) -> String {
        if self.normalized {
            self.stderr.clone()
        } else {
            WhitespaceNormalizer.normalize(&self.stderr)
        }
    }
}

pub fn normalize_output(output: &str) -> String {
    crate::normalizers::whitespace::normalize(output)
}

pub fn normalize_for_comparison(left: &str, right: &str) -> bool {
    let left_norm = normalize_output(left);
    let right_norm = normalize_output(right);
    left_norm == right_norm
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noop_normalizer_returns_unchanged() {
        let normalizer = NoOpNormalizer;
        let input = "  hello world  \n";
        assert_eq!(normalizer.normalize(input), input);
    }

    #[test]
    fn test_normalizer_audit_struct_fields() {
        let audit = NormalizerAudit {
            applied_rules: vec![AppliedRule {
                rule_name: "whitespace".to_string(),
                rule_version: "1.0.0".to_string(),
                conditions_matched: vec!["trim".to_string(), "collapse".to_string()],
            }],
            input_hash: "abc123".to_string(),
            output_hash: "def456".to_string(),
            transformations: vec![Transformation {
                transformation_type: "whitespace".to_string(),
                description: "Trim and collapse whitespace".to_string(),
                before_length: 20,
                after_length: 15,
            }],
        };
        assert_eq!(audit.applied_rules.len(), 1);
        assert_eq!(audit.applied_rules[0].rule_name, "whitespace");
        assert_eq!(audit.applied_rules[0].rule_version, "1.0.0");
        assert_eq!(audit.applied_rules[0].conditions_matched.len(), 2);
        assert_eq!(audit.transformations.len(), 1);
        assert_eq!(audit.transformations[0].before_length, 20);
        assert_eq!(audit.transformations[0].after_length, 15);
    }

    #[test]
    fn test_applied_rule_struct_captures_fields() {
        let rule = AppliedRule {
            rule_name: "test_rule".to_string(),
            rule_version: "2.1.0".to_string(),
            conditions_matched: vec!["condition1".to_string(), "condition2".to_string()],
        };
        assert_eq!(rule.rule_name, "test_rule");
        assert_eq!(rule.rule_version, "2.1.0");
        assert_eq!(rule.conditions_matched.len(), 2);
    }

    #[test]
    fn test_transformation_struct_tracks_fields() {
        let transformation = Transformation {
            transformation_type: "test_type".to_string(),
            description: "Test transformation".to_string(),
            before_length: 100,
            after_length: 80,
        };
        assert_eq!(transformation.transformation_type, "test_type");
        assert_eq!(transformation.description, "Test transformation");
        assert_eq!(transformation.before_length, 100);
        assert_eq!(transformation.after_length, 80);
    }

    #[test]
    fn test_noop_normalizer_describe_rule() {
        let normalizer = NoOpNormalizer;
        assert_eq!(normalizer.describe_rule(), "NoOp");
    }

    #[test]
    fn test_whitespace_normalizer_describe_rule() {
        let normalizer = WhitespaceNormalizer;
        let description = normalizer.describe_rule();
        assert!(
            description.contains("whitespace") || description.contains("Whitespace"),
            "Expected 'whitespace' in description: {}",
            description
        );
        assert!(
            description.contains("trim"),
            "Expected 'trim' in description: {}",
            description
        );
        assert!(
            description.contains("collapse"),
            "Expected 'collapse' in description: {}",
            description
        );
    }

    #[test]
    fn test_line_ending_normalizer_describe_rule() {
        let normalizer = LineEndingNormalizer;
        let description = normalizer.describe_rule().to_lowercase();
        assert!(
            description.contains("line ending")
                || (description.contains("line") && description.contains("ending")),
            "Expected 'line ending' in description: {}",
            description
        );
        assert!(
            description.contains("crlf") || description.contains("\\r\\n"),
            "Expected CRLF in description: {}",
            description
        );
    }

    #[test]
    fn test_path_normalizer_describe_rule() {
        let normalizer = PathNormalizer;
        let description = normalizer.describe_rule();
        assert!(
            description.contains("path") || description.contains("Path"),
            "Expected 'path' in description: {}",
            description
        );
    }

    #[test]
    fn test_audit_normalize_returns_tuple_for_noop() {
        let normalizer = NoOpNormalizer;
        let input = "hello world";
        let (output, transformation) = normalizer.audit_normalize(input);
        assert_eq!(output, input);
        assert_eq!(transformation.transformation_type, "none");
        assert_eq!(transformation.before_length, input.len());
        assert_eq!(transformation.after_length, input.len());
    }

    #[test]
    fn test_audit_normalize_returns_tuple_for_whitespace() {
        let normalizer = WhitespaceNormalizer;
        let input = "  hello   world  \n";
        let (output, transformation) = normalizer.audit_normalize(input);
        assert_eq!(output, "hello world");
        assert_eq!(transformation.transformation_type, "whitespace");
        assert_eq!(transformation.before_length, input.len());
        assert_eq!(transformation.after_length, output.len());
    }

    #[test]
    fn test_audit_normalize_returns_tuple_for_line_ending() {
        let normalizer = LineEndingNormalizer;
        let input = "hello\r\nworld\r";
        let (output, transformation) = normalizer.audit_normalize(input);
        assert!(!output.contains("\r\n") && !output.contains("\r"));
        assert_eq!(transformation.transformation_type, "line_endings");
        assert_eq!(transformation.before_length, input.len());
        assert_eq!(transformation.after_length, output.len());
    }

    #[test]
    fn test_audit_normalize_returns_tuple_for_path() {
        let normalizer = PathNormalizer;
        #[cfg(not(windows))]
        {
            let input = "/home/test/file.txt";
            let (output, transformation) = normalizer.audit_normalize(input);
            assert_eq!(output, input);
            assert_eq!(transformation.transformation_type, "path");
        }
        #[cfg(windows)]
        {
            let input = "C:\\Users\\test\\file.txt";
            let (output, transformation) = normalizer.audit_normalize(input);
            assert!(output.contains('/'));
            assert_eq!(transformation.transformation_type, "path");
        }
    }

    #[test]
    fn test_whitespace_normalizer_trims() {
        let normalizer = WhitespaceNormalizer;
        let input = "  hello world  ";
        assert_eq!(normalizer.normalize(input), "hello world");
    }

    #[test]
    fn test_whitespace_normalizer_normalizes_tabs() {
        let normalizer = WhitespaceNormalizer;
        let input = "hello\t\tworld";
        assert_eq!(normalizer.normalize(input), "hello world");
    }

    #[test]
    fn test_whitespace_normalizer_collapses_multiple_spaces() {
        let normalizer = WhitespaceNormalizer;
        let input = "hello    world";
        assert_eq!(normalizer.normalize(input), "hello world");
    }

    #[test]
    fn test_normalized_output_from_output() {
        let output = std::process::Output {
            stdout: b"hello".to_vec(),
            stderr: b"world".to_vec(),
            status: std::process::ExitStatus::default(),
        };
        let normalized = NormalizedOutput::from_output(&output);
        assert_eq!(normalized.stdout, "hello");
        assert_eq!(normalized.stderr, "world");
        assert!(!normalized.normalized);
    }

    #[test]
    fn test_normalized_output_apply() {
        let normalized = NormalizedOutput::new("  hello world  ", "  error  ");
        let result = normalized.apply(&WhitespaceNormalizer);
        assert_eq!(result.stdout, "hello world");
        assert_eq!(result.stderr, "error");
        assert!(result.normalized);
    }

    #[test]
    fn test_normalized_output_apply_multiple() {
        let normalized = NormalizedOutput::new("  hello \t world  ", "");
        let result = normalized.apply_multiple(&[&WhitespaceNormalizer as &dyn Normalizer]);
        assert_eq!(result.stdout, "hello world");
        assert!(result.normalized);
    }

    #[test]
    fn test_normalize_output_function() {
        let input = "  line1  \n  line2  \n";
        let result = normalize_output(input);
        assert_eq!(result, "line1 line2");
    }

    #[test]
    fn test_normalize_for_comparison_equal() {
        let left = "  hello world  ";
        let right = "hello world";
        assert!(normalize_for_comparison(left, right));
    }

    #[test]
    fn test_normalize_for_comparison_not_equal() {
        let left = "hello world";
        let right = "hello world!";
        assert!(!normalize_for_comparison(left, right));
    }

    #[test]
    fn test_path_normalizer() {
        let normalizer = PathNormalizer;
        #[cfg(windows)]
        {
            let input = "C:\\Users\\test\\file.txt";
            let result = normalizer.normalize(input);
            assert!(result.contains('/'));
        }
        #[cfg(not(windows))]
        {
            let input = "/home/test/file.txt";
            let result = normalizer.normalize(input);
            assert_eq!(result, "/home/test/file.txt");
        }
    }

    #[test]
    fn normalizer_smoke_tests() {
        fn assert_normalizer<T: Normalizer>() {}
        assert_normalizer::<NoOpNormalizer>();
        assert_normalizer::<WhitespaceNormalizer>();
        assert_normalizer::<LineEndingNormalizer>();
        assert_normalizer::<PathNormalizer>();

        let noop = NoOpNormalizer;
        let whitespace = WhitespaceNormalizer;
        let line_ending = LineEndingNormalizer;
        let path = PathNormalizer;

        assert!(!noop.name().is_empty());
        assert!(!whitespace.name().is_empty());
        assert!(!line_ending.name().is_empty());
        assert!(!path.name().is_empty());

        assert!(!noop.describe_rule().is_empty());
        assert!(!whitespace.describe_rule().is_empty());
        assert!(!line_ending.describe_rule().is_empty());
        assert!(!path.describe_rule().is_empty());

        let (noop_out, noop_trans) = noop.audit_normalize("  test  ");
        assert_eq!(noop_out, "  test  ");
        assert_eq!(noop_trans.transformation_type, "none");
        assert_eq!(noop_trans.before_length, 8);
        assert_eq!(noop_trans.after_length, 8);

        let (ws_out, ws_trans) = whitespace.audit_normalize("  hello \t world  ");
        assert_eq!(ws_out, "hello world");
        assert_eq!(ws_trans.transformation_type, "whitespace");
        assert_eq!(ws_trans.before_length, 17);
        assert_eq!(ws_trans.after_length, 11);

        let (le_out, le_trans) = line_ending.audit_normalize("line1\r\nline2\r");
        assert!(!le_out.contains("\r\n") && !le_out.contains("\r"));
        assert_eq!(le_trans.transformation_type, "line_endings");

        let output = NormalizedOutput::new("  hello \t world  \r\ntest", "");
        let result = output.apply_multiple(&[&whitespace as &dyn Normalizer]);
        assert_eq!(result.stdout, "hello world test");

        let output2 = NormalizedOutput::new("  hello  world  ", "");
        let result2 = output2.apply_multiple(&[
            &whitespace as &dyn Normalizer,
            &line_ending as &dyn Normalizer,
        ]);
        assert!(result2.normalized);

        let composed = NormalizedOutput::new("  hello \r\n world  \t  ", "");
        let result3 = composed.apply(&WhitespaceNormalizer);
        assert!(result3.stdout.contains("hello"));
    }
}
