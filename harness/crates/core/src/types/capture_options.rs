use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaptureOptions {
    pub capture_stdout: bool,
    pub capture_stderr: bool,
    pub capture_timing: bool,
    pub capture_artifacts: bool,
    pub capture_environment: bool,
    pub max_output_size_bytes: Option<usize>,
    pub artifact_filter: Option<Vec<PathBuf>>,
    pub timing_tolerance: Option<f64>,
}

impl Default for CaptureOptions {
    fn default() -> Self {
        Self {
            capture_stdout: true,
            capture_stderr: true,
            capture_timing: true,
            capture_artifacts: true,
            capture_environment: true,
            max_output_size_bytes: None,
            artifact_filter: None,
            timing_tolerance: None,
        }
    }
}

impl CaptureOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capture_stdout(mut self, capture_stdout: bool) -> Self {
        self.capture_stdout = capture_stdout;
        self
    }

    pub fn with_capture_stderr(mut self, capture_stderr: bool) -> Self {
        self.capture_stderr = capture_stderr;
        self
    }

    pub fn with_capture_timing(mut self, capture_timing: bool) -> Self {
        self.capture_timing = capture_timing;
        self
    }

    pub fn with_capture_artifacts(mut self, capture_artifacts: bool) -> Self {
        self.capture_artifacts = capture_artifacts;
        self
    }

    pub fn with_capture_environment(mut self, capture_environment: bool) -> Self {
        self.capture_environment = capture_environment;
        self
    }

    pub fn with_max_output_size_bytes(mut self, max_output_size_bytes: Option<usize>) -> Self {
        self.max_output_size_bytes = max_output_size_bytes;
        self
    }

    pub fn with_artifact_filter(mut self, artifact_filter: Option<Vec<PathBuf>>) -> Self {
        self.artifact_filter = artifact_filter;
        self
    }

    pub fn with_timing_tolerance(mut self, timing_tolerance: Option<f64>) -> Self {
        self.timing_tolerance = timing_tolerance;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_options_default_values() {
        let options = CaptureOptions::new();
        assert!(options.capture_stdout);
        assert!(options.capture_stderr);
        assert!(options.capture_timing);
        assert!(options.capture_artifacts);
        assert!(options.capture_environment);
        assert!(options.max_output_size_bytes.is_none());
        assert!(options.artifact_filter.is_none());
        assert!(options.timing_tolerance.is_none());
    }

    #[test]
    fn test_capture_options_builder_pattern() {
        let options = CaptureOptions::new()
            .with_capture_stdout(false)
            .with_capture_stderr(false)
            .with_capture_timing(false)
            .with_capture_artifacts(false)
            .with_capture_environment(false)
            .with_max_output_size_bytes(Some(1024))
            .with_artifact_filter(Some(vec![PathBuf::from("*.log")]));

        assert!(!options.capture_stdout);
        assert!(!options.capture_stderr);
        assert!(!options.capture_timing);
        assert!(!options.capture_artifacts);
        assert!(!options.capture_environment);
        assert_eq!(options.max_output_size_bytes, Some(1024));
        assert_eq!(options.artifact_filter, Some(vec![PathBuf::from("*.log")]));
    }

    #[test]
    fn test_capture_options_serde_roundtrip() {
        let options = CaptureOptions::new()
            .with_capture_stdout(true)
            .with_capture_stderr(true)
            .with_max_output_size_bytes(Some(2048));

        let serialized = serde_json::to_string(&options).expect("serialization should succeed");
        let deserialized: CaptureOptions =
            serde_json::from_str(&serialized).expect("deserialization should succeed");

        assert_eq!(options, deserialized);
    }

    #[test]
    fn test_capture_options_json_format() {
        let options = CaptureOptions::new();
        let json = serde_json::to_string(&options).expect("serialization should succeed");
        assert!(json.contains("\"capture_stdout\":true"));
        assert!(json.contains("\"capture_stderr\":true"));
    }

    #[test]
    fn test_capture_options_timing_tolerance_default() {
        let options = CaptureOptions::new();
        assert!(options.timing_tolerance.is_none());
    }

    #[test]
    fn test_capture_options_timing_tolerance_builder() {
        let options = CaptureOptions::new().with_timing_tolerance(Some(0.5));
        assert_eq!(options.timing_tolerance, Some(0.5));
    }

    #[test]
    fn test_capture_options_timing_tolerance_builder_with_none() {
        let options = CaptureOptions::new().with_timing_tolerance(None);
        assert!(options.timing_tolerance.is_none());
    }

    #[test]
    fn test_capture_options_timing_tolerance_various_values() {
        let options_025 = CaptureOptions::new().with_timing_tolerance(Some(0.25));
        let options_050 = CaptureOptions::new().with_timing_tolerance(Some(0.50));
        let options_075 = CaptureOptions::new().with_timing_tolerance(Some(0.75));
        let options_100 = CaptureOptions::new().with_timing_tolerance(Some(1.0));

        assert_eq!(options_025.timing_tolerance, Some(0.25));
        assert_eq!(options_050.timing_tolerance, Some(0.50));
        assert_eq!(options_075.timing_tolerance, Some(0.75));
        assert_eq!(options_100.timing_tolerance, Some(1.0));
    }

    #[test]
    fn test_capture_options_timing_tolerance_serde_roundtrip() {
        let options = CaptureOptions::new().with_timing_tolerance(Some(0.3));

        let serialized = serde_json::to_string(&options).expect("serialization should succeed");
        let deserialized: CaptureOptions =
            serde_json::from_str(&serialized).expect("deserialization should succeed");

        assert_eq!(options, deserialized);
        assert_eq!(deserialized.timing_tolerance, Some(0.3));
    }
}
