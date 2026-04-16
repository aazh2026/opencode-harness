use regex::Regex;

pub trait Normalizer: Send + Sync {
    fn normalize(&self, output: &str) -> String;

    fn name(&self) -> &str;
}

#[derive(Debug, Clone)]
pub struct BasicTextNormalizer {
    timestamp_pattern: Regex,
    id_pattern: Regex,
}

impl BasicTextNormalizer {
    pub fn new() -> Self {
        Self {
            timestamp_pattern: Regex::new(r"\b\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z?\b")
                .unwrap(),
            id_pattern: Regex::new(
                r"\b[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}\b",
            )
            .unwrap(),
        }
    }
}

impl Default for BasicTextNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Normalizer for BasicTextNormalizer {
    fn normalize(&self, output: &str) -> String {
        let result = self.timestamp_pattern.replace_all(output, "<TIMESTAMP>");
        let result = self.id_pattern.replace_all(&result, "<ID>");
        result.into_owned()
    }

    fn name(&self) -> &str {
        "basic-text-normalizer"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalizer_trait_defined() {
        fn assert_normalizer<T: Normalizer>() {}
        assert_normalizer::<BasicTextNormalizer>();
    }

    #[test]
    fn test_normalizer_normalize_method_signature() {
        fn takes_normalizer(n: &dyn Normalizer) {
            let _ = n.normalize("test output");
        }
        takes_normalizer(&BasicTextNormalizer::new());
    }

    #[test]
    fn test_basic_text_normalizer_exists() {
        let normalizer = BasicTextNormalizer::new();
        assert_eq!(normalizer.name(), "basic-text-normalizer");
    }

    #[test]
    fn test_normalize_produces_consistent_output_for_equivalent_inputs() {
        let normalizer = BasicTextNormalizer::new();

        let output1 = "Build completed at 2024-01-15T10:30:00Z";
        let output2 = "Build completed at 2024-01-15T10:30:00Z";

        assert_eq!(normalizer.normalize(output1), normalizer.normalize(output2));

        let output3 =
            "Build completed at 2024-01-15T10:30:00Z with id 550e8400-e29b-41d4-a716-446655440000";
        let output4 =
            "Build completed at 2024-01-15T10:30:00Z with id 6ba7b810-9dad-11d1-80b4-00c04fd430c8";

        assert_eq!(normalizer.normalize(output3), normalizer.normalize(output4));
    }

    #[test]
    fn test_normalize_removes_timestamps() {
        let normalizer = BasicTextNormalizer::new();
        let output = "Log entry at 2024-01-15T10:30:00Z completed";
        let normalized = normalizer.normalize(output);
        assert!(!normalized.contains("2024-01-15T10:30:00Z"));
        assert!(normalized.contains("<TIMESTAMP>"));
    }

    #[test]
    fn test_normalize_removes_uuids() {
        let normalizer = BasicTextNormalizer::new();
        let output = "ID: 550e8400-e29b-41d4-a716-446655440000";
        let normalized = normalizer.normalize(output);
        assert!(!normalized.contains("550e8400-e29b-41d4-a716-446655440000"));
        assert!(normalized.contains("<ID>"));
    }

    #[test]
    fn test_normalize_preserves_static_text() {
        let normalizer = BasicTextNormalizer::new();
        let output = "Hello World";
        let normalized = normalizer.normalize(output);
        assert_eq!(normalized, "Hello World");
    }

    #[test]
    fn test_normalize_handles_empty_string() {
        let normalizer = BasicTextNormalizer::new();
        let output = "";
        let normalized = normalizer.normalize(output);
        assert_eq!(normalized, "");
    }
}
