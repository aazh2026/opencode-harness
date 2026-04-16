pub trait Normalizer: Send + Sync {
    fn normalize(&self, output: &str) -> String;
    fn name(&self) -> &str;
}

pub struct NoOpNormalizer;

impl Normalizer for NoOpNormalizer {
    fn normalize(&self, output: &str) -> String {
        output.to_string()
    }

    fn name(&self) -> &str {
        "noop"
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
}
