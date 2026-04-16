use super::Normalizer;
use std::path::Path;

pub struct VarianceNormalizer {
    rules: Vec<(String, String)>,
}

impl VarianceNormalizer {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn with_pattern(mut self, pattern: &str, replacement: &str) -> Self {
        self.rules
            .push((pattern.to_string(), replacement.to_string()));
        self
    }

    pub fn add_temporary_directory_replacement(&mut self, temp_dir: &Path) {
        let temp_str = temp_dir.to_string_lossy();
        self.rules
            .push((temp_str.to_string(), "<TMPDIR>".to_string()));
    }

    pub fn add_timestamp_pattern(&mut self) {
        self.rules.push((
            r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}".to_string(),
            "<TIMESTAMP>".to_string(),
        ));
    }

    pub fn add_uuid_pattern(&mut self) {
        self.rules.push((
            r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}".to_string(),
            "<UUID>".to_string(),
        ));
    }

    pub fn add_process_id_pattern(&mut self) {
        self.rules
            .push((r"pid:\s*\d+".to_string(), "pid: <PID>".to_string()));
    }
}

impl Default for VarianceNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Normalizer for VarianceNormalizer {
    fn normalize(&self, output: &str) -> String {
        let mut result = output.to_string();
        for (pattern, replacement) in &self.rules {
            if let Ok(re) = regex::Regex::new(pattern) {
                result = re.replace_all(&result, replacement.as_str()).to_string();
            }
        }
        result
    }

    fn name(&self) -> &str {
        "variance"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variance_normalizer_basic() {
        let normalizer = VarianceNormalizer::new().with_pattern("foo", "bar");
        assert_eq!(normalizer.normalize("hello foo world"), "hello bar world");
    }

    #[test]
    fn test_variance_normalizer_default_is_empty() {
        let normalizer = VarianceNormalizer::new();
        let input = "hello world";
        assert_eq!(normalizer.normalize(input), input);
    }

    #[test]
    fn test_variance_normalizer_with_multiple_patterns() {
        let normalizer = VarianceNormalizer::new()
            .with_pattern("error", "ERROR")
            .with_pattern("warning", "WARN");
        let result = normalizer.normalize("error: something warning: also this");
        assert!(result.contains("ERROR"));
        assert!(result.contains("WARN"));
    }

    #[test]
    fn test_variance_normalizer_with_temp_dir() {
        let mut normalizer = VarianceNormalizer::new();
        normalizer.add_temporary_directory_replacement(Path::new("/tmp/test123"));
        let result = normalizer.normalize("/tmp/test123/file.txt");
        assert!(result.contains("<TMPDIR>"));
        assert!(!result.contains("/tmp/test123"));
    }

    #[test]
    fn test_variance_normalizer_with_timestamp() {
        let mut normalizer = VarianceNormalizer::new();
        normalizer.add_timestamp_pattern();
        let result = normalizer.normalize("2024-01-15T10:30:00 event");
        assert!(result.contains("<TIMESTAMP>"));
        assert!(!result.contains("2024-01-15T10:30:00"));
    }

    #[test]
    fn test_variance_normalizer_with_uuid() {
        let mut normalizer = VarianceNormalizer::new();
        normalizer.add_uuid_pattern();
        let result = normalizer.normalize("id: 550e8400-e29b-41d4-a716-446655440000");
        assert!(result.contains("<UUID>"));
    }

    #[test]
    fn test_variance_normalizer_with_process_id() {
        let mut normalizer = VarianceNormalizer::new();
        normalizer.add_process_id_pattern();
        let result = normalizer.normalize("process pid: 12345 started");
        assert!(result.contains("pid: <PID>"));
        assert!(!result.contains("pid: 12345"));
    }

    #[test]
    fn test_variance_normalizer_name() {
        let normalizer = VarianceNormalizer::new();
        assert_eq!(normalizer.name(), "variance");
    }

    #[test]
    fn test_variance_normalizer_no_match() {
        let normalizer = VarianceNormalizer::new().with_pattern("xyz", "abc");
        let result = normalizer.normalize("hello world");
        assert_eq!(result, "hello world");
    }
}
