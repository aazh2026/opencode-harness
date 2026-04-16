#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ProviderMode {
    OpenCode,
    OpenCodeRS,
    Both,
    Either,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_mode_variants_exist() {
        let _ = ProviderMode::OpenCode;
        let _ = ProviderMode::OpenCodeRS;
        let _ = ProviderMode::Both;
        let _ = ProviderMode::Either;
    }

    #[test]
    fn test_provider_mode_serde_roundtrip() {
        let variants = [
            ProviderMode::OpenCode,
            ProviderMode::OpenCodeRS,
            ProviderMode::Both,
            ProviderMode::Either,
        ];

        for original in variants {
            let serialized =
                serde_json::to_string(&original).expect("serialization should succeed");
            let deserialized: ProviderMode =
                serde_json::from_str(&serialized).expect("deserialization should succeed");
            assert_eq!(
                original, deserialized,
                "roundtrip should preserve the value"
            );
        }
    }

    #[test]
    fn test_provider_mode_json_format() {
        assert_eq!(
            serde_json::to_string(&ProviderMode::OpenCode).unwrap(),
            "\"OpenCode\""
        );
        assert_eq!(
            serde_json::to_string(&ProviderMode::OpenCodeRS).unwrap(),
            "\"OpenCodeRS\""
        );
        assert_eq!(
            serde_json::to_string(&ProviderMode::Both).unwrap(),
            "\"Both\""
        );
        assert_eq!(
            serde_json::to_string(&ProviderMode::Either).unwrap(),
            "\"Either\""
        );
    }
}
