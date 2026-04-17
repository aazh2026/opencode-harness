#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ExecutionLevel {
    AlwaysOn,
    NightlyOnly,
    ReleaseOnly,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_level_has_all_variants() {
        let always_on = ExecutionLevel::AlwaysOn;
        let nightly_only = ExecutionLevel::NightlyOnly;
        let release_only = ExecutionLevel::ReleaseOnly;

        assert_eq!(always_on, ExecutionLevel::AlwaysOn);
        assert_eq!(nightly_only, ExecutionLevel::NightlyOnly);
        assert_eq!(release_only, ExecutionLevel::ReleaseOnly);
    }

    #[test]
    fn test_execution_level_serde_roundtrip_always_on() {
        let level = ExecutionLevel::AlwaysOn;
        let json = serde_json::to_string(&level).unwrap();
        let deserialized: ExecutionLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, level);
    }

    #[test]
    fn test_execution_level_serde_roundtrip_nightly_only() {
        let level = ExecutionLevel::NightlyOnly;
        let json = serde_json::to_string(&level).unwrap();
        let deserialized: ExecutionLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, level);
    }

    #[test]
    fn test_execution_level_serde_roundtrip_release_only() {
        let level = ExecutionLevel::ReleaseOnly;
        let json = serde_json::to_string(&level).unwrap();
        let deserialized: ExecutionLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, level);
    }

    #[test]
    fn test_execution_level_all_variants_serde() {
        let variants = vec![
            ExecutionLevel::AlwaysOn,
            ExecutionLevel::NightlyOnly,
            ExecutionLevel::ReleaseOnly,
        ];
        for variant in variants {
            let json = serde_json::to_string(&variant).unwrap();
            let deserialized: ExecutionLevel = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, variant);
        }
    }

    #[test]
    fn test_execution_level_partial_eq() {
        assert_eq!(ExecutionLevel::AlwaysOn, ExecutionLevel::AlwaysOn);
        assert_eq!(ExecutionLevel::NightlyOnly, ExecutionLevel::NightlyOnly);
        assert_eq!(ExecutionLevel::ReleaseOnly, ExecutionLevel::ReleaseOnly);

        assert_ne!(ExecutionLevel::AlwaysOn, ExecutionLevel::NightlyOnly);
        assert_ne!(ExecutionLevel::AlwaysOn, ExecutionLevel::ReleaseOnly);
        assert_ne!(ExecutionLevel::NightlyOnly, ExecutionLevel::ReleaseOnly);
    }

    #[test]
    fn test_execution_level_debug_format() {
        assert_eq!(format!("{:?}", ExecutionLevel::AlwaysOn), "AlwaysOn");
        assert_eq!(format!("{:?}", ExecutionLevel::NightlyOnly), "NightlyOnly");
        assert_eq!(format!("{:?}", ExecutionLevel::ReleaseOnly), "ReleaseOnly");
    }
}
