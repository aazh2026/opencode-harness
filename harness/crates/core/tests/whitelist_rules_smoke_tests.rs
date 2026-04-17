use chrono::Utc;
use opencode_core::loaders::whitelist_loader::{DefaultWhitelistLoader, WhitelistLoader};
use opencode_core::normalizers::whitelist_validator::WhitelistValidator;
use opencode_core::types::allowed_variance::{
    AllowedVariance, TimingVariance, WhitelistEntry, WhitelistScope,
};
use tempfile::TempDir;

fn create_valid_whitelist_entry(id: &str, scope: WhitelistScope) -> WhitelistEntry {
    let now = Utc::now();
    let future = now + chrono::Duration::days(30);
    WhitelistEntry::new(
        id.to_string(),
        scope,
        "Known timing variance for testing".to_string(),
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
    )
}

#[test]
fn whitelist_rules_smoke_tests_entry_with_all_governance_fields() {
    let now = Utc::now();
    let future = now + chrono::Duration::days(30);
    let entry = WhitelistEntry::new(
        "WL-GOV-001".to_string(),
        WhitelistScope::Task("TASK-GOV-001".to_string()),
        "Full governance fields test".to_string(),
        "team-governance".to_string(),
        Some(future),
        Some("https://github.com/example/repo/issues/456".to_string()),
        AllowedVariance::new(
            vec![0, 1],
            Some(TimingVariance::new(Some(100), Some(500))),
            vec!["pattern.*".to_string()],
        ),
        now,
        now,
    );

    assert_eq!(entry.id, "WL-GOV-001");
    assert_eq!(
        entry.scope,
        WhitelistScope::Task("TASK-GOV-001".to_string())
    );
    assert_eq!(entry.reason, "Full governance fields test");
    assert_eq!(entry.owner, "team-governance");
    assert!(entry.expires_at.is_some());
    assert_eq!(
        entry.linked_issue,
        Some("https://github.com/example/repo/issues/456".to_string())
    );
    assert_eq!(entry.allowed_variance.exit_code, vec![0, 1]);
    assert_eq!(entry.created_at, now);
    assert_eq!(entry.updated_at, now);
}

#[test]
fn whitelist_rules_smoke_tests_validation_passes_with_valid_entry() {
    let entry = create_valid_whitelist_entry("WL-VALID-001", WhitelistScope::Global);
    let result = WhitelistValidator::validate(&entry).expect("validation should succeed");

    assert!(
        result.valid,
        "Expected valid result for entry with all governance fields"
    );
    assert!(
        result.errors.is_empty(),
        "Expected no errors for valid entry: {:?}",
        result.errors
    );
}

#[test]
fn whitelist_rules_smoke_tests_validation_fails_without_owner() {
    let now = Utc::now();
    let future = now + chrono::Duration::days(30);
    let entry = WhitelistEntry::new(
        "WL-NO-OWNER".to_string(),
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
    assert!(!result.valid, "Expected validation to fail without owner");
    assert_eq!(result.errors.len(), 1);
    assert_eq!(result.errors[0].field, "owner");
    assert_eq!(result.errors[0].message, "owner is required");
}

#[test]
fn whitelist_rules_smoke_tests_validation_fails_without_expires_at() {
    let now = Utc::now();
    let entry = WhitelistEntry::new(
        "WL-NO-EXPIRES".to_string(),
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
    assert!(
        !result.valid,
        "Expected validation to fail without expires_at"
    );
    assert_eq!(result.errors.len(), 1);
    assert_eq!(result.errors[0].field, "expires_at");
    assert_eq!(result.errors[0].message, "expires_at is required");
}

#[test]
fn whitelist_rules_smoke_tests_validation_fails_with_expired_entry() {
    let now = Utc::now();
    let past = now - chrono::Duration::days(1);
    let entry = WhitelistEntry::new(
        "WL-EXPIRED".to_string(),
        WhitelistScope::Global,
        "Expired entry".to_string(),
        "team-owner".to_string(),
        Some(past),
        None,
        AllowedVariance::new(vec![0], None, vec![]),
        now,
        now,
    );

    let result = WhitelistValidator::validate(&entry).expect("validation should succeed");
    assert!(
        !result.valid,
        "Expected validation to fail with expired entry"
    );
    assert_eq!(result.errors.len(), 1);
    assert_eq!(result.errors[0].field, "expires_at");
    assert!(
        result.errors[0].message.contains("future"),
        "Expected 'future' in error message"
    );
}

#[test]
fn whitelist_rules_smoke_tests_expiration_checking() {
    let now = Utc::now();
    let past = now - chrono::Duration::days(1);
    let future = now + chrono::Duration::days(30);

    let expired_entry = WhitelistEntry::new(
        "WL-EXP-001".to_string(),
        WhitelistScope::Global,
        "Expired".to_string(),
        "team-exp".to_string(),
        Some(past),
        None,
        AllowedVariance::new(vec![0], None, vec![]),
        now,
        now,
    );

    let valid_entry = WhitelistEntry::new(
        "WL-EXP-002".to_string(),
        WhitelistScope::Global,
        "Valid".to_string(),
        "team-valid".to_string(),
        Some(future),
        None,
        AllowedVariance::new(vec![0], None, vec![]),
        now,
        now,
    );

    assert!(
        WhitelistValidator::is_expired(&expired_entry),
        "Expected expired_entry to be detected as expired"
    );
    assert!(
        !WhitelistValidator::is_expired(&valid_entry),
        "Expected valid_entry to not be detected as expired"
    );
}

#[test]
fn whitelist_rules_smoke_tests_expired_entry_detection() {
    let entry = create_valid_whitelist_entry("WL-DETECT-001", WhitelistScope::Global);
    assert!(
        !entry.is_expired(),
        "Newly created entry should not be expired"
    );

    let now = Utc::now();
    let past = now - chrono::Duration::days(60);
    let expired_entry = WhitelistEntry::new(
        "WL-OLD".to_string(),
        WhitelistScope::Global,
        "Old entry".to_string(),
        "team-old".to_string(),
        Some(past),
        None,
        AllowedVariance::new(vec![0], None, vec![]),
        now - chrono::Duration::days(90),
        now - chrono::Duration::days(90),
    );

    assert!(
        expired_entry.is_expired(),
        "Entry with past expires_at should be expired"
    );
}

#[test]
fn whitelist_rules_smoke_tests_loader_save_load_roundtrip() {
    let temp_dir = TempDir::new().unwrap();
    let loader = DefaultWhitelistLoader::new(temp_dir.path().to_path_buf());

    let original =
        create_valid_whitelist_entry("WL-LOAD-001", WhitelistScope::Task("TASK-LOAD".to_string()));

    loader.save(&original).unwrap();

    let loaded = loader.load("WL-LOAD-001").unwrap();
    assert!(loaded.is_some(), "Expected to load saved whitelist entry");

    let loaded = loaded.unwrap();
    assert_eq!(loaded.id, original.id);
    assert_eq!(loaded.scope, original.scope);
    assert_eq!(loaded.reason, original.reason);
    assert_eq!(loaded.owner, original.owner);
    assert_eq!(loaded.expires_at, original.expires_at);
    assert_eq!(loaded.linked_issue, original.linked_issue);
    assert_eq!(
        loaded.allowed_variance.exit_code,
        original.allowed_variance.exit_code
    );
}

#[test]
fn whitelist_rules_smoke_tests_loader_yaml_roundtrip() {
    let temp_dir = TempDir::new().unwrap();
    let loader = DefaultWhitelistLoader::new(temp_dir.path().to_path_buf());

    let original = create_valid_whitelist_entry(
        "WL-YAML-001",
        WhitelistScope::Category("timing".to_string()),
    );

    loader.save(&original).unwrap();

    let yaml_path = temp_dir.path().join("WL-YAML-001.yaml");
    let content = std::fs::read_to_string(&yaml_path).unwrap();
    assert!(content.contains("WL-YAML-001"));
    assert!(content.contains("timing"));
    assert!(content.contains("team-platform"));
}

#[test]
fn whitelist_rules_smoke_tests_load_active_filters_expired() {
    let temp_dir = TempDir::new().unwrap();
    let loader = DefaultWhitelistLoader::new(temp_dir.path().to_path_buf());

    let now = Utc::now();
    let future = now + chrono::Duration::days(30);
    let past = now - chrono::Duration::days(1);

    let active_entry = create_valid_whitelist_entry("WL-ACTIVE-001", WhitelistScope::Global);
    let expired_entry = WhitelistEntry::new(
        "WL-EXP-002".to_string(),
        WhitelistScope::Global,
        "Expired".to_string(),
        "team-exp".to_string(),
        Some(past),
        None,
        AllowedVariance::new(vec![0], None, vec![]),
        now,
        now,
    );

    loader.save(&active_entry).unwrap();
    loader.save(&expired_entry).unwrap();

    let active_entries = loader.load_active().unwrap();
    assert_eq!(active_entries.len(), 1, "Expected only active entries");
    assert_eq!(active_entries[0].id, "WL-ACTIVE-001");
}

#[test]
fn whitelist_rules_smoke_tests_load_for_scope() {
    let temp_dir = TempDir::new().unwrap();
    let loader = DefaultWhitelistLoader::new(temp_dir.path().to_path_buf());

    let entry1 =
        create_valid_whitelist_entry("WL-SCOPE-001", WhitelistScope::Task("TASK-A".to_string()));
    let entry2 =
        create_valid_whitelist_entry("WL-SCOPE-002", WhitelistScope::Task("TASK-B".to_string()));
    let entry3 = create_valid_whitelist_entry(
        "WL-SCOPE-003",
        WhitelistScope::Category("cat-timing".to_string()),
    );

    loader.save(&entry1).unwrap();
    loader.save(&entry2).unwrap();
    loader.save(&entry3).unwrap();

    let task_entries = loader
        .load_for_scope(&WhitelistScope::Task("TASK-A".to_string()))
        .unwrap();
    assert_eq!(task_entries.len(), 1);
    assert_eq!(task_entries[0].id, "WL-SCOPE-001");
}

#[test]
fn whitelist_rules_smoke_tests_validate_or_raise_success() {
    let entry = create_valid_whitelist_entry("WL-RAISE-OK", WhitelistScope::Global);
    let result = WhitelistValidator::validate_or_raise(&entry);
    assert!(
        result.is_ok(),
        "Expected validate_or_raise to succeed for valid entry"
    );
}

#[test]
fn whitelist_rules_smoke_tests_validate_or_raise_failure() {
    let now = Utc::now();
    let entry = WhitelistEntry::new(
        "WL-RAISE-FAIL".to_string(),
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
    assert!(
        result.is_err(),
        "Expected validate_or_raise to fail for invalid entry"
    );
}

#[test]
fn whitelist_rules_smoke_tests_cleanup_expired() {
    let temp_dir = TempDir::new().unwrap();
    let loader = DefaultWhitelistLoader::new(temp_dir.path().to_path_buf());

    let now = Utc::now();
    let future = now + chrono::Duration::days(30);
    let past = now - chrono::Duration::days(1);

    let active_entry = create_valid_whitelist_entry("WL-CLEAN-001", WhitelistScope::Global);
    let expired_entry = WhitelistEntry::new(
        "WL-CLEAN-002".to_string(),
        WhitelistScope::Global,
        "Expired".to_string(),
        "team-clean".to_string(),
        Some(past),
        None,
        AllowedVariance::new(vec![0], None, vec![]),
        now,
        now,
    );

    loader.save(&active_entry).unwrap();
    loader.save(&expired_entry).unwrap();

    let deleted_ids = loader.cleanup_expired().unwrap();
    assert_eq!(deleted_ids.len(), 1);
    assert_eq!(deleted_ids[0], "WL-CLEAN-002");

    let remaining = loader.load_all().unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].id, "WL-CLEAN-001");
}

#[test]
fn whitelist_rules_smoke_tests_full_validation_pipeline() {
    let temp_dir = TempDir::new().unwrap();
    let loader = DefaultWhitelistLoader::new(temp_dir.path().to_path_buf());

    let entry = create_valid_whitelist_entry(
        "WL-PIPELINE-001",
        WhitelistScope::Task("TASK-PIPELINE".to_string()),
    );

    WhitelistValidator::validate_or_raise(&entry).expect("Entry should be valid");

    loader.save(&entry).expect("Should save successfully");

    let loaded = loader
        .load("WL-PIPELINE-001")
        .expect("Should load saved entry");
    assert!(loaded.is_some(), "Loaded entry should exist");

    let loaded_entry = loaded.unwrap();
    WhitelistValidator::validate_or_raise(&loaded_entry)
        .expect("Loaded entry should still be valid");

    let all_entries = loader.load_all().expect("Should load all entries");
    assert_eq!(all_entries.len(), 1);

    let active_entries = loader.load_active().expect("Should load active entries");
    assert_eq!(active_entries.len(), 1);
    assert_eq!(active_entries[0].id, "WL-PIPELINE-001");
}

#[test]
fn whitelist_rules_smoke_tests_whitelist_scopes() {
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
fn whitelist_rules_smoke_tests_allowed_variance_timing() {
    let variance = AllowedVariance::new(
        vec![0, 1],
        Some(TimingVariance::new(Some(100), Some(500))),
        vec!["test.*".to_string()],
    );

    assert_eq!(variance.exit_code, vec![0, 1]);
    assert!(variance.timing_ms.is_some());
    let timing = variance.timing_ms.unwrap();
    assert_eq!(timing.min, Some(100));
    assert_eq!(timing.max, Some(500));
    assert_eq!(variance.output_patterns, vec!["test.*".to_string()]);
}

#[test]
fn whitelist_rules_smoke_tests_to_whitelist_entry_conversion() {
    let variance = AllowedVariance::new(
        vec![0],
        Some(TimingVariance::new(Some(0), Some(1000))),
        vec![],
    );
    let future = Utc::now() + chrono::Duration::days(30);

    let entry = variance
        .to_whitelist_entry(
            "WL-CONV-001".to_string(),
            WhitelistScope::Global,
            "Conversion test".to_string(),
            "team-conv".to_string(),
            Some(future),
            Some("https://github.com/example/repo/issues/999".to_string()),
        )
        .expect("Conversion should succeed");

    assert_eq!(entry.id, "WL-CONV-001");
    assert_eq!(entry.owner, "team-conv");
    assert_eq!(entry.scope, WhitelistScope::Global);
    assert!(entry.expires_at.is_some());
}

#[test]
fn whitelist_rules_smoke_tests_trait_object() {
    let temp_dir = TempDir::new().unwrap();
    let loader: Box<dyn WhitelistLoader> =
        Box::new(DefaultWhitelistLoader::new(temp_dir.path().to_path_buf()));

    let entry = create_valid_whitelist_entry("WL-TRAIT-001", WhitelistScope::Global);
    loader.save(&entry).unwrap();

    let loaded = loader.load("WL-TRAIT-001").unwrap();
    assert!(loaded.is_some());
    assert_eq!(loaded.unwrap().id, "WL-TRAIT-001");
}
