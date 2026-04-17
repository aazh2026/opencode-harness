use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::types::allowed_variance::WhitelistEntry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

pub struct WhitelistValidator;

impl WhitelistValidator {
    pub fn validate(entry: &WhitelistEntry) -> Result<ValidationResult> {
        let mut errors = Vec::new();

        if entry.owner.is_empty() {
            errors.push(ValidationError {
                field: "owner".to_string(),
                message: "owner is required".to_string(),
            });
        }

        if entry.expires_at.is_none() {
            errors.push(ValidationError {
                field: "expires_at".to_string(),
                message: "expires_at is required".to_string(),
            });
        } else if let Some(expires) = entry.expires_at {
            if Utc::now() > expires {
                errors.push(ValidationError {
                    field: "expires_at".to_string(),
                    message: "expires_at must be in the future".to_string(),
                });
            }
        }

        if entry.reason.is_empty() {
            errors.push(ValidationError {
                field: "reason".to_string(),
                message: "reason is required".to_string(),
            });
        }

        Ok(ValidationResult {
            valid: errors.is_empty(),
            errors,
        })
    }

    pub fn is_expired(entry: &WhitelistEntry) -> bool {
        match entry.expires_at {
            Some(expires) => Utc::now() > expires,
            None => false,
        }
    }

    pub fn validate_or_raise(entry: &WhitelistEntry) -> Result<()> {
        let result = Self::validate(entry)?;
        if !result.valid {
            let messages: Vec<String> = result.errors.iter().map(|e| e.message.clone()).collect();
            Err(crate::error::ErrorType::Config(messages.join("; ")))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::allowed_variance::{AllowedVariance, WhitelistScope};

    fn create_valid_entry() -> WhitelistEntry {
        let now = Utc::now();
        let future = now + chrono::Duration::days(30);
        WhitelistEntry::new(
            "WL-001".to_string(),
            WhitelistScope::Task("TASK-001".to_string()),
            "Known timing variance".to_string(),
            "team-platform".to_string(),
            Some(future),
            Some("https://github.com/example/repo/issues/123".to_string()),
            AllowedVariance::new(
                vec![0],
                Some(crate::types::allowed_variance::TimingVariance::new(
                    Some(0),
                    Some(1000),
                )),
                vec![],
            ),
            now,
            now,
        )
    }

    #[test]
    fn test_validate_passes_with_valid_entry() {
        let entry = create_valid_entry();
        let result = WhitelistValidator::validate(&entry).expect("validation should succeed");
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_fails_without_owner() {
        let now = Utc::now();
        let future = now + chrono::Duration::days(30);
        let entry = WhitelistEntry::new(
            "WL-002".to_string(),
            WhitelistScope::Global,
            "Some reason".to_string(),
            "".to_string(),
            Some(future),
            None,
            AllowedVariance::new(vec![0], None, vec![]),
            now,
            now,
        );

        let result = WhitelistValidator::validate(&entry).expect("validation should succeed");
        assert!(!result.valid);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].field, "owner");
    }

    #[test]
    fn test_validate_fails_without_expires_at() {
        let now = Utc::now();
        let entry = WhitelistEntry::new(
            "WL-003".to_string(),
            WhitelistScope::Global,
            "Some reason".to_string(),
            "team-owner".to_string(),
            None,
            None,
            AllowedVariance::new(vec![0], None, vec![]),
            now,
            now,
        );

        let result = WhitelistValidator::validate(&entry).expect("validation should succeed");
        assert!(!result.valid);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].field, "expires_at");
    }

    #[test]
    fn test_validate_fails_with_expired_entry() {
        let now = Utc::now();
        let past = now - chrono::Duration::days(1);
        let entry = WhitelistEntry::new(
            "WL-004".to_string(),
            WhitelistScope::Global,
            "Some reason".to_string(),
            "team-owner".to_string(),
            Some(past),
            None,
            AllowedVariance::new(vec![0], None, vec![]),
            now,
            now,
        );

        let result = WhitelistValidator::validate(&entry).expect("validation should succeed");
        assert!(!result.valid);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].field, "expires_at");
        assert!(result.errors[0].message.contains("future"));
    }

    #[test]
    fn test_validate_fails_with_empty_reason() {
        let now = Utc::now();
        let future = now + chrono::Duration::days(30);
        let entry = WhitelistEntry::new(
            "WL-005".to_string(),
            WhitelistScope::Global,
            "".to_string(),
            "team-owner".to_string(),
            Some(future),
            None,
            AllowedVariance::new(vec![0], None, vec![]),
            now,
            now,
        );

        let result = WhitelistValidator::validate(&entry).expect("validation should succeed");
        assert!(!result.valid);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].field, "reason");
    }

    #[test]
    fn test_is_expired_correctly_detects_expired_entries() {
        let now = Utc::now();
        let past = now - chrono::Duration::days(1);
        let future = now + chrono::Duration::days(30);

        let expired_entry = WhitelistEntry::new(
            "WL-006".to_string(),
            WhitelistScope::Global,
            "Expired entry".to_string(),
            "team-owner".to_string(),
            Some(past),
            None,
            AllowedVariance::new(vec![0], None, vec![]),
            now,
            now,
        );

        let valid_entry = WhitelistEntry::new(
            "WL-007".to_string(),
            WhitelistScope::Global,
            "Valid entry".to_string(),
            "team-owner".to_string(),
            Some(future),
            None,
            AllowedVariance::new(vec![0], None, vec![]),
            now,
            now,
        );

        assert!(WhitelistValidator::is_expired(&expired_entry));
        assert!(!WhitelistValidator::is_expired(&valid_entry));
    }

    #[test]
    fn test_validate_or_raise_throws_on_invalid_entry() {
        let now = Utc::now();
        let entry = WhitelistEntry::new(
            "WL-008".to_string(),
            WhitelistScope::Global,
            "".to_string(),
            "".to_string(),
            None,
            None,
            AllowedVariance::new(vec![0], None, vec![]),
            now,
            now,
        );

        let result = WhitelistValidator::validate_or_raise(&entry);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_or_raise_succeeds_on_valid_entry() {
        let entry = create_valid_entry();
        let result = WhitelistValidator::validate_or_raise(&entry);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validation_result_serde() {
        let result = ValidationResult {
            valid: false,
            errors: vec![ValidationError {
                field: "owner".to_string(),
                message: "owner is required".to_string(),
            }],
        };

        let json = serde_json::to_string(&result).expect("serialization should succeed");
        let deserialized: ValidationResult =
            serde_json::from_str(&json).expect("deserialization should succeed");

        assert_eq!(result.valid, deserialized.valid);
        assert_eq!(result.errors.len(), deserialized.errors.len());
    }
}
