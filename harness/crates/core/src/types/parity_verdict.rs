use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffCategory {
    OutputText,
    ExitCode,
    Timing,
    SideEffects,
    Protocol,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VarianceType {
    Timing,
    ExitCode,
    OutputPattern,
    NonDeterministic,
    EnvironmentLimited,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockedReason {
    BinaryNotFound { binary: String },
    DependencyMissing { dependency: String },
    EnvironmentNotSupported { requirement: String },
    PermissionDenied { resource: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MismatchCandidate {
    pub field_name: String,
    pub legacy_value: String,
    pub rust_value: String,
    pub diff_category: DiffCategory,
}

impl MismatchCandidate {
    pub fn new(
        field_name: String,
        legacy_value: String,
        rust_value: String,
        diff_category: DiffCategory,
    ) -> Self {
        Self {
            field_name,
            legacy_value,
            rust_value,
            diff_category,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParityVerdict {
    Pass,
    PassWithAllowedVariance {
        variance_type: VarianceType,
        details: String,
    },
    Warn {
        category: DiffCategory,
        message: String,
    },
    Fail {
        category: DiffCategory,
        details: String,
    },
    ManualCheck {
        reason: String,
        candidates: Vec<MismatchCandidate>,
    },
    Blocked {
        reason: BlockedReason,
    },
    Error {
        runner: String,
        reason: String,
    },
}

impl ParityVerdict {
    pub fn is_identical(&self) -> bool {
        matches!(self, ParityVerdict::Pass)
    }

    pub fn is_equivalent(&self) -> bool {
        matches!(self, ParityVerdict::PassWithAllowedVariance { .. })
    }

    pub fn is_different(&self) -> bool {
        matches!(self, ParityVerdict::Fail { .. })
    }

    pub fn is_uncertain(&self) -> bool {
        matches!(self, ParityVerdict::ManualCheck { .. })
    }

    pub fn is_error(&self) -> bool {
        matches!(self, ParityVerdict::Error { .. })
    }

    pub fn is_pass(&self) -> bool {
        matches!(
            self,
            ParityVerdict::Pass | ParityVerdict::PassWithAllowedVariance { .. }
        )
    }

    pub fn summary(&self) -> String {
        match self {
            ParityVerdict::Pass => "Pass".to_string(),
            ParityVerdict::PassWithAllowedVariance {
                variance_type,
                details,
            } => {
                format!("PassWithAllowedVariance ({:?}): {}", variance_type, details)
            }
            ParityVerdict::Warn { category, message } => {
                format!("Warn ({:?}): {}", category, message)
            }
            ParityVerdict::Fail { category, details } => {
                format!("Fail ({:?}): {}", category, details)
            }
            ParityVerdict::ManualCheck { reason, candidates } => {
                format!("ManualCheck: {} ({} candidates)", reason, candidates.len())
            }
            ParityVerdict::Blocked { reason } => {
                format!("Blocked: {:?}", reason)
            }
            ParityVerdict::Error { runner, reason } => {
                format!("Error ({}: {})", runner, reason)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parity_verdict_pass() {
        let v = ParityVerdict::Pass;
        assert!(v.is_identical());
        assert!(!v.is_equivalent());
        assert!(!v.is_different());
        assert!(!v.is_uncertain());
        assert!(!v.is_error());
        assert!(v.is_pass());
        assert_eq!(v.summary(), "Pass");
    }

    #[test]
    fn test_parity_verdict_pass_with_allowed_variance() {
        let v = ParityVerdict::PassWithAllowedVariance {
            variance_type: VarianceType::Timing,
            details: "Timing diff within tolerance".to_string(),
        };
        assert!(v.is_equivalent());
        assert!(v.is_pass());
        assert!(!v.is_identical());
        assert_eq!(
            v.summary(),
            "PassWithAllowedVariance (Timing): Timing diff within tolerance"
        );
    }

    #[test]
    fn test_parity_verdict_warn() {
        let v = ParityVerdict::Warn {
            category: DiffCategory::Timing,
            message: "Timing slightly off".to_string(),
        };
        assert!(!v.is_pass());
        assert!(!v.is_different());
        assert_eq!(v.summary(), "Warn (Timing): Timing slightly off");
    }

    #[test]
    fn test_parity_verdict_fail() {
        let v = ParityVerdict::Fail {
            category: DiffCategory::OutputText,
            details: "Output mismatch".to_string(),
        };
        assert!(v.is_different());
        assert!(!v.is_pass());
        assert_eq!(v.summary(), "Fail (OutputText): Output mismatch");
    }

    #[test]
    fn test_parity_verdict_manual_check() {
        let candidates = vec![MismatchCandidate::new(
            "stdout".to_string(),
            "legacy output".to_string(),
            "rust output".to_string(),
            DiffCategory::OutputText,
        )];
        let v = ParityVerdict::ManualCheck {
            reason: "Ambiguous output format".to_string(),
            candidates,
        };
        assert!(v.is_uncertain());
        assert!(!v.is_pass());
        assert_eq!(
            v.summary(),
            "ManualCheck: Ambiguous output format (1 candidates)"
        );
    }

    #[test]
    fn test_parity_verdict_blocked() {
        let v = ParityVerdict::Blocked {
            reason: BlockedReason::BinaryNotFound {
                binary: "opencode".to_string(),
            },
        };
        assert!(!v.is_pass());
        assert_eq!(
            v.summary(),
            "Blocked: BinaryNotFound { binary: \"opencode\" }"
        );
    }

    #[test]
    fn test_parity_verdict_error() {
        let v = ParityVerdict::Error {
            runner: "LegacyRunner".to_string(),
            reason: "binary not found".to_string(),
        };
        assert!(v.is_error());
        assert!(!v.is_pass());
        assert_eq!(v.summary(), "Error (LegacyRunner: binary not found)");
    }

    #[test]
    fn test_variance_type_variants() {
        let variants = vec![
            VarianceType::Timing,
            VarianceType::ExitCode,
            VarianceType::OutputPattern,
            VarianceType::NonDeterministic,
            VarianceType::EnvironmentLimited,
        ];
        for variant in variants {
            let v = ParityVerdict::PassWithAllowedVariance {
                variance_type: variant,
                details: "test".to_string(),
            };
            assert!(v.is_pass());
        }
    }

    #[test]
    fn test_blocked_reason_variants() {
        let reasons = vec![
            BlockedReason::BinaryNotFound {
                binary: "test".to_string(),
            },
            BlockedReason::DependencyMissing {
                dependency: "dep".to_string(),
            },
            BlockedReason::EnvironmentNotSupported {
                requirement: "req".to_string(),
            },
            BlockedReason::PermissionDenied {
                resource: "res".to_string(),
            },
        ];
        for reason in reasons {
            let v = ParityVerdict::Blocked { reason };
            assert!(!v.is_pass());
        }
    }

    #[test]
    fn test_blocked_reason_serde_roundtrip() {
        let reason = BlockedReason::BinaryNotFound {
            binary: "opencode".to_string(),
        };
        let json = serde_json::to_string(&reason).unwrap();
        let deserialized: BlockedReason = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, reason);
    }

    #[test]
    fn test_mismatch_candidate_struct() {
        let candidate = MismatchCandidate::new(
            "exit_code".to_string(),
            "0".to_string(),
            "1".to_string(),
            DiffCategory::ExitCode,
        );
        assert_eq!(candidate.field_name, "exit_code");
        assert_eq!(candidate.legacy_value, "0");
        assert_eq!(candidate.rust_value, "1");
        assert_eq!(candidate.diff_category, DiffCategory::ExitCode);
    }

    #[test]
    fn test_mismatch_candidate_serde_roundtrip() {
        let candidate = MismatchCandidate::new(
            "stdout".to_string(),
            "legacy".to_string(),
            "rust".to_string(),
            DiffCategory::OutputText,
        );
        let json = serde_json::to_string(&candidate).unwrap();
        let deserialized: MismatchCandidate = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, candidate);
    }

    #[test]
    fn test_parity_verdict_serde_roundtrip_pass() {
        let v = ParityVerdict::Pass;
        let json = serde_json::to_string(&v).unwrap();
        let deserialized: ParityVerdict = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.summary(), v.summary());
    }

    #[test]
    fn test_parity_verdict_serde_roundtrip_fail() {
        let v = ParityVerdict::Fail {
            category: DiffCategory::ExitCode,
            details: "Exit codes differ".to_string(),
        };
        let json = serde_json::to_string(&v).unwrap();
        let deserialized: ParityVerdict = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.summary(), v.summary());
    }

    #[test]
    fn test_parity_verdict_serde_roundtrip_manual_check() {
        let v = ParityVerdict::ManualCheck {
            reason: "Needs review".to_string(),
            candidates: vec![MismatchCandidate::new(
                "timing".to_string(),
                "100ms".to_string(),
                "150ms".to_string(),
                DiffCategory::Timing,
            )],
        };
        let json = serde_json::to_string(&v).unwrap();
        let deserialized: ParityVerdict = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.summary(), v.summary());
    }

    #[test]
    fn test_diff_category_variants() {
        let categories = vec![
            DiffCategory::OutputText,
            DiffCategory::ExitCode,
            DiffCategory::Timing,
            DiffCategory::SideEffects,
            DiffCategory::Protocol,
        ];
        for cat in categories {
            let v = ParityVerdict::Fail {
                category: cat,
                details: "test".to_string(),
            };
            assert!(v.is_different());
        }
    }
}
