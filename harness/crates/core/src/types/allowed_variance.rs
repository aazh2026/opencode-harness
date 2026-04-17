use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WhitelistScope {
    Task(String),
    Category(String),
    Global,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistEntry {
    pub id: String,
    pub scope: WhitelistScope,
    pub reason: String,
    pub owner: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub linked_issue: Option<String>,
    pub allowed_variance: AllowedVariance,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl WhitelistEntry {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        scope: WhitelistScope,
        reason: String,
        owner: String,
        expires_at: Option<DateTime<Utc>>,
        linked_issue: Option<String>,
        allowed_variance: AllowedVariance,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            scope,
            reason,
            owner,
            expires_at,
            linked_issue,
            allowed_variance,
            created_at,
            updated_at,
        }
    }

    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(expires) => Utc::now() > expires,
            None => false,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.owner.is_empty() {
            return Err("owner is required".to_string());
        }
        if self.expires_at.is_none() {
            return Err("expires_at is required".to_string());
        }
        if let Some(expires) = self.expires_at {
            if Utc::now() > expires {
                return Err("expires_at must be in the future".to_string());
            }
        }
        if self.reason.is_empty() {
            return Err("reason is required".to_string());
        }
        Ok(())
    }
}

impl AllowedVariance {
    pub fn to_whitelist_entry(
        self,
        id: String,
        scope: WhitelistScope,
        reason: String,
        owner: String,
        expires_at: Option<DateTime<Utc>>,
        linked_issue: Option<String>,
    ) -> Result<WhitelistEntry, String> {
        let now = Utc::now();
        let entry = WhitelistEntry::new(
            id,
            scope,
            reason,
            owner,
            expires_at,
            linked_issue,
            self,
            now,
            now,
        );
        entry.validate()?;
        Ok(entry)
    }
}

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

    #[test]
    fn test_whitelist_entry_with_all_governance_fields() {
        let now = Utc::now();
        let future = now + chrono::Duration::days(30);
        let entry = WhitelistEntry::new(
            "WL-001".to_string(),
            WhitelistScope::Task("TASK-001".to_string()),
            "Known timing variance".to_string(),
            "team-platform".to_string(),
            Some(future),
            Some("https://github.com/example/repo/issues/123".to_string()),
            AllowedVariance::new(
                vec![0],
                Some(TimingVariance::new(Some(0), Some(1000))),
                vec![],
            ),
            now,
            now,
        );

        assert_eq!(entry.id, "WL-001");
        assert_eq!(entry.scope, WhitelistScope::Task("TASK-001".to_string()));
        assert_eq!(entry.reason, "Known timing variance");
        assert_eq!(entry.owner, "team-platform");
        assert!(entry.expires_at.is_some());
        assert_eq!(
            entry.linked_issue,
            Some("https://github.com/example/repo/issues/123".to_string())
        );
        assert_eq!(entry.allowed_variance.exit_code, vec![0]);
        assert_eq!(entry.created_at, now);
        assert_eq!(entry.updated_at, now);
    }

    #[test]
    fn test_whitelist_scope_enum_has_task_category_global_variants() {
        let task_scope = WhitelistScope::Task("TASK-001".to_string());
        let category_scope = WhitelistScope::Category("timing".to_string());
        let global_scope = WhitelistScope::Global;

        assert_eq!(task_scope, WhitelistScope::Task("TASK-001".to_string()));
        assert_eq!(
            category_scope,
            WhitelistScope::Category("timing".to_string())
        );
        assert_eq!(global_scope, WhitelistScope::Global);
    }

    #[test]
    fn test_whitelist_scope_partial_eq() {
        assert_eq!(
            WhitelistScope::Task("TASK-001".to_string()),
            WhitelistScope::Task("TASK-001".to_string())
        );
        assert_ne!(
            WhitelistScope::Task("TASK-001".to_string()),
            WhitelistScope::Task("TASK-002".to_string())
        );
        assert_ne!(
            WhitelistScope::Task("TASK-001".to_string()),
            WhitelistScope::Category("timing".to_string())
        );
        assert_ne!(
            WhitelistScope::Category("timing".to_string()),
            WhitelistScope::Global
        );
    }

    #[test]
    fn test_to_whitelist_entry_conversion() {
        let variance = AllowedVariance::new(
            vec![0, 1],
            Some(TimingVariance::new(Some(100), Some(500))),
            vec!["pattern1".to_string()],
        );
        let future = Utc::now() + chrono::Duration::days(30);

        let entry = variance
            .to_whitelist_entry(
                "WL-002".to_string(),
                WhitelistScope::Category("timing".to_string()),
                "Historical timing issue".to_string(),
                "team-backend".to_string(),
                Some(future),
                Some("https://github.com/example/repo/issues/456".to_string()),
            )
            .expect("conversion should succeed");

        assert_eq!(entry.id, "WL-002");
        assert_eq!(entry.scope, WhitelistScope::Category("timing".to_string()));
        assert_eq!(entry.reason, "Historical timing issue");
        assert_eq!(entry.owner, "team-backend");
        assert_eq!(entry.allowed_variance.exit_code, vec![0, 1]);
    }

    #[test]
    fn test_whitelist_entry_without_owner_is_rejected() {
        let variance = AllowedVariance::new(vec![0], None, vec![]);
        let future = Utc::now() + chrono::Duration::days(30);

        let result = variance.clone().to_whitelist_entry(
            "WL-003".to_string(),
            WhitelistScope::Global,
            "Some reason".to_string(),
            "".to_string(),
            Some(future),
            None,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "owner is required");
    }

    #[test]
    fn test_whitelist_entry_without_expires_at_is_rejected() {
        let variance = AllowedVariance::new(vec![0], None, vec![]);

        let result = variance.clone().to_whitelist_entry(
            "WL-004".to_string(),
            WhitelistScope::Global,
            "Some reason".to_string(),
            "team-owner".to_string(),
            None,
            None,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "expires_at is required");
    }

    #[test]
    fn test_whitelist_entry_with_past_expires_at_is_rejected() {
        let variance = AllowedVariance::new(vec![0], None, vec![]);
        let past = Utc::now() - chrono::Duration::days(1);

        let entry = WhitelistEntry::new(
            "WL-005".to_string(),
            WhitelistScope::Global,
            "Some reason".to_string(),
            "team-owner".to_string(),
            Some(past),
            None,
            variance,
            Utc::now(),
            Utc::now(),
        );

        let result = entry.validate();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "expires_at must be in the future");
    }

    #[test]
    fn test_whitelist_entry_is_expired() {
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

        assert!(expired_entry.is_expired());
        assert!(!valid_entry.is_expired());
    }

    #[test]
    fn test_whitelist_entry_serde_roundtrip() {
        let now = Utc::now();
        let future = now + chrono::Duration::days(30);
        let entry = WhitelistEntry::new(
            "WL-008".to_string(),
            WhitelistScope::Task("TASK-008".to_string()),
            "JSON test".to_string(),
            "team-json".to_string(),
            Some(future),
            Some("https://github.com/example/repo/issues/789".to_string()),
            AllowedVariance::new(
                vec![0],
                Some(TimingVariance::new(Some(0), Some(2000))),
                vec!["json.*".to_string()],
            ),
            now,
            now,
        );

        let json = serde_json::to_string(&entry).expect("serialization should succeed");
        let deserialized: WhitelistEntry =
            serde_json::from_str(&json).expect("deserialization should succeed");

        assert_eq!(entry.id, deserialized.id);
        assert_eq!(entry.owner, deserialized.owner);
        assert_eq!(entry.scope, deserialized.scope);
        assert_eq!(
            entry.allowed_variance.exit_code,
            deserialized.allowed_variance.exit_code
        );
    }

    #[test]
    fn test_whitelist_entry_yaml_serde_roundtrip() {
        let now = Utc::now();
        let future = now + chrono::Duration::days(30);
        let entry = WhitelistEntry::new(
            "WL-009".to_string(),
            WhitelistScope::Global,
            "YAML test".to_string(),
            "team-yaml".to_string(),
            Some(future),
            None,
            AllowedVariance::new(vec![1], None, vec![]),
            now,
            now,
        );

        let yaml = serde_yaml::to_string(&entry).expect("serialization should succeed");
        let deserialized: WhitelistEntry =
            serde_yaml::from_str(&yaml).expect("deserialization should succeed");

        assert_eq!(entry.id, deserialized.id);
        assert_eq!(entry.owner, deserialized.owner);
    }

    #[test]
    fn test_whitelist_entry_validate_empty_reason() {
        let future = Utc::now() + chrono::Duration::days(30);
        let entry = WhitelistEntry::new(
            "WL-010".to_string(),
            WhitelistScope::Global,
            "".to_string(),
            "team-owner".to_string(),
            Some(future),
            None,
            AllowedVariance::new(vec![0], None, vec![]),
            Utc::now(),
            Utc::now(),
        );

        let result = entry.validate();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "reason is required");
    }

    #[test]
    fn test_whitelist_entry_validate_success() {
        let future = Utc::now() + chrono::Duration::days(30);
        let entry = WhitelistEntry::new(
            "WL-011".to_string(),
            WhitelistScope::Category("timing".to_string()),
            "Valid entry with all fields".to_string(),
            "team-valid".to_string(),
            Some(future),
            Some("https://github.com/example/repo/issues/999".to_string()),
            AllowedVariance::new(
                vec![0],
                Some(TimingVariance::new(Some(0), Some(5000))),
                vec![],
            ),
            Utc::now(),
            Utc::now(),
        );

        let result = entry.validate();
        assert!(result.is_ok());
    }
}
