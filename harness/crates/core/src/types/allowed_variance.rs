use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimingVariance {
    pub min: Option<u64>,
    pub max: Option<u64>,
}

impl TimingVariance {
    pub fn new(min: Option<u64>, max: Option<u64>) -> Self {
        Self { min, max }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AllowedVariance {
    #[serde(default)]
    pub exit_code: Vec<u32>,
    #[serde(default)]
    pub timing_ms: Option<TimingVariance>,
    #[serde(default)]
    pub output_patterns: Vec<String>,
}

impl AllowedVariance {
    pub fn new(
        exit_code: Vec<u32>,
        timing_ms: Option<TimingVariance>,
        output_patterns: Vec<String>,
    ) -> Self {
        Self {
            exit_code,
            timing_ms,
            output_patterns,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allowed_variance_instantiation() {
        let variance = AllowedVariance::new(
            vec![0, 1],
            Some(TimingVariance::new(Some(100), Some(500))),
            vec![r"\d+ items?".to_string()],
        );

        assert_eq!(variance.exit_code, vec![0, 1]);
        assert!(variance.timing_ms.is_some());
        let timing = variance.timing_ms.unwrap();
        assert_eq!(timing.min, Some(100));
        assert_eq!(timing.max, Some(500));
        assert_eq!(variance.output_patterns, vec![r"\d+ items?".to_string()]);
    }

    #[test]
    fn test_allowed_variance_serde_roundtrip() {
        let variance = AllowedVariance::new(
            vec![0],
            Some(TimingVariance::new(Some(0), Some(1000))),
            vec![".*".to_string()],
        );

        let serialized = serde_json::to_string(&variance).expect("serialization should succeed");
        let deserialized: AllowedVariance =
            serde_json::from_str(&serialized).expect("deserialization should succeed");
        assert_eq!(variance, deserialized);
    }

    #[test]
    fn test_allowed_variance_json_format() {
        let variance = AllowedVariance::new(
            vec![0, 1],
            Some(TimingVariance::new(Some(100), Some(500))),
            vec!["pattern1".to_string(), "pattern2".to_string()],
        );

        let json = serde_json::to_string(&variance).unwrap();
        assert!(json.contains("\"exit_code\":[0,1]"));
        assert!(json.contains("\"timing_ms\""));
        assert!(json.contains("\"min\":100"));
        assert!(json.contains("\"max\":500"));
        assert!(json.contains("\"output_patterns\""));
    }

    #[test]
    fn test_allowed_variance_default_fields() {
        let variance = AllowedVariance::new(vec![], None, vec![]);
        assert!(variance.exit_code.is_empty());
        assert!(variance.timing_ms.is_none());
        assert!(variance.output_patterns.is_empty());
    }

    #[test]
    fn test_allowed_variance_fields_accessible_and_typed() {
        let variance = AllowedVariance::new(
            vec![42],
            Some(TimingVariance::new(None, Some(200))),
            vec!["test".to_string()],
        );

        let _exit_codes: Vec<u32> = variance.exit_code;
        let _timing: Option<TimingVariance> = variance.timing_ms;
        let _patterns: Vec<String> = variance.output_patterns;
    }

    #[test]
    fn test_timing_variance_instantiation() {
        let timing = TimingVariance::new(Some(50), Some(150));
        assert_eq!(timing.min, Some(50));
        assert_eq!(timing.max, Some(150));
    }

    #[test]
    fn test_timing_variance_optional_bounds() {
        let timing_min_only = TimingVariance::new(Some(10), None);
        assert_eq!(timing_min_only.min, Some(10));
        assert_eq!(timing_min_only.max, None);

        let timing_max_only = TimingVariance::new(None, Some(100));
        assert_eq!(timing_max_only.min, None);
        assert_eq!(timing_max_only.max, Some(100));

        let timing_none = TimingVariance::new(None, None);
        assert_eq!(timing_none.min, None);
        assert_eq!(timing_none.max, None);
    }

    #[test]
    fn test_timing_variance_serde_roundtrip() {
        let timing = TimingVariance::new(Some(100), Some(500));
        let serialized = serde_json::to_string(&timing).expect("serialization should succeed");
        let deserialized: TimingVariance =
            serde_json::from_str(&serialized).expect("deserialization should succeed");
        assert_eq!(timing, deserialized);
    }
}
