use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CapabilitySummary {
    pub binary_available: bool,
    pub workspace_prepared: bool,
    pub environment_supported: bool,
    pub timeout_enforced: bool,
    pub artifacts_collected: u32,
    pub side_effects_detected: Vec<PathBuf>,
}

impl CapabilitySummary {
    pub fn new(
        binary_available: bool,
        workspace_prepared: bool,
        environment_supported: bool,
        timeout_enforced: bool,
        artifacts_collected: u32,
        side_effects_detected: Vec<PathBuf>,
    ) -> Self {
        Self {
            binary_available,
            workspace_prepared,
            environment_supported,
            timeout_enforced,
            artifacts_collected,
            side_effects_detected,
        }
    }

    pub fn with_binary_available(mut self, binary_available: bool) -> Self {
        self.binary_available = binary_available;
        self
    }

    pub fn with_workspace_prepared(mut self, workspace_prepared: bool) -> Self {
        self.workspace_prepared = workspace_prepared;
        self
    }

    pub fn with_environment_supported(mut self, environment_supported: bool) -> Self {
        self.environment_supported = environment_supported;
        self
    }

    pub fn with_timeout_enforced(mut self, timeout_enforced: bool) -> Self {
        self.timeout_enforced = timeout_enforced;
        self
    }

    pub fn with_artifacts_collected(mut self, artifacts_collected: u32) -> Self {
        self.artifacts_collected = artifacts_collected;
        self
    }

    pub fn with_side_effects_detected(mut self, side_effects_detected: Vec<PathBuf>) -> Self {
        self.side_effects_detected = side_effects_detected;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_summary_instantiation_with_all_fields() {
        let binary_available = true;
        let workspace_prepared = true;
        let environment_supported = true;
        let timeout_enforced = true;
        let artifacts_collected = 5u32;
        let side_effects_detected = vec![PathBuf::from("/tmp/side-effect-1")];

        let summary = CapabilitySummary::new(
            binary_available,
            workspace_prepared,
            environment_supported,
            timeout_enforced,
            artifacts_collected,
            side_effects_detected.clone(),
        );

        assert_eq!(summary.binary_available, binary_available);
        assert_eq!(summary.workspace_prepared, workspace_prepared);
        assert_eq!(summary.environment_supported, environment_supported);
        assert_eq!(summary.timeout_enforced, timeout_enforced);
        assert_eq!(summary.artifacts_collected, artifacts_collected);
        assert_eq!(summary.side_effects_detected, side_effects_detected);
    }

    #[test]
    fn test_capability_summary_builder_pattern() {
        let summary = CapabilitySummary::default()
            .with_binary_available(true)
            .with_workspace_prepared(true)
            .with_environment_supported(false)
            .with_timeout_enforced(true)
            .with_artifacts_collected(10)
            .with_side_effects_detected(vec![
                PathBuf::from("/tmp/effect1"),
                PathBuf::from("/tmp/effect2"),
            ]);

        assert!(summary.binary_available);
        assert!(summary.workspace_prepared);
        assert!(!summary.environment_supported);
        assert!(summary.timeout_enforced);
        assert_eq!(summary.artifacts_collected, 10);
        assert_eq!(summary.side_effects_detected.len(), 2);
    }

    #[test]
    fn test_capability_summary_serde_roundtrip() {
        let summary = CapabilitySummary::new(
            true,
            true,
            false,
            true,
            3,
            vec![PathBuf::from("/tmp/detected-effect")],
        );

        let serialized = serde_json::to_string(&summary).expect("serialization should succeed");
        let deserialized: CapabilitySummary =
            serde_json::from_str(&serialized).expect("deserialization should succeed");

        assert_eq!(summary.binary_available, deserialized.binary_available);
        assert_eq!(summary.workspace_prepared, deserialized.workspace_prepared);
        assert_eq!(
            summary.environment_supported,
            deserialized.environment_supported
        );
        assert_eq!(summary.timeout_enforced, deserialized.timeout_enforced);
        assert_eq!(
            summary.artifacts_collected,
            deserialized.artifacts_collected
        );
        assert_eq!(
            summary.side_effects_detected.len(),
            deserialized.side_effects_detected.len()
        );
    }

    #[test]
    fn test_capability_summary_default_values() {
        let summary = CapabilitySummary::default();

        assert!(!summary.binary_available);
        assert!(!summary.workspace_prepared);
        assert!(!summary.environment_supported);
        assert!(!summary.timeout_enforced);
        assert_eq!(summary.artifacts_collected, 0);
        assert!(summary.side_effects_detected.is_empty());
    }

    #[test]
    fn test_capability_summary_json_format() {
        let summary = CapabilitySummary::default();
        let json = serde_json::to_string(&summary).expect("serialization should succeed");

        assert!(json.contains("\"binary_available\""));
        assert!(json.contains("\"workspace_prepared\""));
        assert!(json.contains("\"environment_supported\""));
        assert!(json.contains("\"timeout_enforced\""));
        assert!(json.contains("\"artifacts_collected\""));
        assert!(json.contains("\"side_effects_detected\""));
    }

    #[test]
    fn test_capability_summary_empty_side_effects() {
        let summary = CapabilitySummary::new(true, false, true, false, 0, Vec::new());

        assert!(summary.side_effects_detected.is_empty());
        let serialized = serde_json::to_string(&summary).expect("serialization should succeed");
        let deserialized: CapabilitySummary =
            serde_json::from_str(&serialized).expect("deserialization should succeed");
        assert!(deserialized.side_effects_detected.is_empty());
    }
}
