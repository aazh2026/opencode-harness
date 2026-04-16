use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailureClassification {
    ImplementationFailure,
    DependencyMissing,
    EnvironmentNotSupported,
    InfraFailure,
    FlakySuspected,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_failure_classification_serde_roundtrip() {
        let variants = [
            FailureClassification::ImplementationFailure,
            FailureClassification::DependencyMissing,
            FailureClassification::EnvironmentNotSupported,
            FailureClassification::InfraFailure,
            FailureClassification::FlakySuspected,
        ];

        for variant in variants {
            let serialized = serde_json::to_string(&variant).expect("serialization should succeed");
            let deserialized: FailureClassification =
                serde_json::from_str(&serialized).expect("deserialization should succeed");
            assert_eq!(variant, deserialized, "roundtrip should preserve variant");
        }
    }

    #[test]
    fn test_failure_classification_all_variants_serializable() {
        for variant in [
            FailureClassification::ImplementationFailure,
            FailureClassification::DependencyMissing,
            FailureClassification::EnvironmentNotSupported,
            FailureClassification::InfraFailure,
            FailureClassification::FlakySuspected,
        ] {
            let result = serde_json::to_string(&variant);
            assert!(result.is_ok(), "all variants should be serializable");
        }
    }

    #[test]
    fn test_failure_classification_all_variants_deserializable() {
        let json_values = [
            r#""ImplementationFailure""#,
            r#""DependencyMissing""#,
            r#""EnvironmentNotSupported""#,
            r#""InfraFailure""#,
            r#""FlakySuspected""#,
        ];

        let expected_variants = [
            FailureClassification::ImplementationFailure,
            FailureClassification::DependencyMissing,
            FailureClassification::EnvironmentNotSupported,
            FailureClassification::InfraFailure,
            FailureClassification::FlakySuspected,
        ];

        for (json, expected) in json_values.iter().zip(expected_variants.iter()) {
            let result: Result<FailureClassification, _> = serde_json::from_str(json);
            assert!(
                result.is_ok(),
                "all variants should be deserializable from {}",
                json
            );
            assert_eq!(result.unwrap(), *expected);
        }
    }
}
