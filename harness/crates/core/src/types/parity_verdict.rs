use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffCategory {
    OutputText,
    ExitCode,
    Timing,
    SideEffects,
    Protocol,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParityVerdict {
    Identical,
    Equivalent,
    Different { category: DiffCategory },
    Uncertain,
    Error { runner: String, reason: String },
}

impl ParityVerdict {
    pub fn is_identical(&self) -> bool {
        matches!(self, ParityVerdict::Identical)
    }

    pub fn is_equivalent(&self) -> bool {
        matches!(self, ParityVerdict::Equivalent)
    }

    pub fn is_different(&self) -> bool {
        matches!(self, ParityVerdict::Different { .. })
    }

    pub fn is_uncertain(&self) -> bool {
        matches!(self, ParityVerdict::Uncertain)
    }

    pub fn is_error(&self) -> bool {
        matches!(self, ParityVerdict::Error { .. })
    }

    pub fn is_pass(&self) -> bool {
        matches!(self, ParityVerdict::Identical | ParityVerdict::Equivalent)
    }

    pub fn summary(&self) -> String {
        match self {
            ParityVerdict::Identical => "Identical".to_string(),
            ParityVerdict::Equivalent => "Equivalent".to_string(),
            ParityVerdict::Different { category } => format!("Different ({:?})", category),
            ParityVerdict::Uncertain => "Uncertain".to_string(),
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
    fn test_parity_verdict_identical() {
        let v = ParityVerdict::Identical;
        assert!(v.is_identical());
        assert!(!v.is_equivalent());
        assert!(!v.is_different());
        assert!(!v.is_uncertain());
        assert!(!v.is_error());
        assert!(v.is_pass());
        assert_eq!(v.summary(), "Identical");
    }

    #[test]
    fn test_parity_verdict_equivalent() {
        let v = ParityVerdict::Equivalent;
        assert!(v.is_equivalent());
        assert!(v.is_pass());
        assert_eq!(v.summary(), "Equivalent");
    }

    #[test]
    fn test_parity_verdict_different() {
        let v = ParityVerdict::Different {
            category: DiffCategory::OutputText,
        };
        assert!(v.is_different());
        assert!(!v.is_pass());
        assert_eq!(v.summary(), "Different (OutputText)");
    }

    #[test]
    fn test_parity_verdict_uncertain() {
        let v = ParityVerdict::Uncertain;
        assert!(v.is_uncertain());
        assert_eq!(v.summary(), "Uncertain");
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
    fn test_diff_category_variants() {
        let categories = vec![
            DiffCategory::OutputText,
            DiffCategory::ExitCode,
            DiffCategory::Timing,
            DiffCategory::SideEffects,
            DiffCategory::Protocol,
        ];
        for cat in categories {
            let v = ParityVerdict::Different { category: cat };
            assert!(v.is_different());
        }
    }

    #[test]
    fn test_parity_verdict_serde() {
        let v = ParityVerdict::Different {
            category: DiffCategory::ExitCode,
        };
        let json = serde_json::to_string(&v).unwrap();
        let deserialized: ParityVerdict = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.summary(), v.summary());
    }
}
