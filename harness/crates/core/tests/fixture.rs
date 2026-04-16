use opencode_core::types::fixture::{
    ConfigFormat, FixtureConfig, FixtureFile, FixtureProject, ResetStrategy, Transcript,
    TranscriptType, WorkspacePolicy,
};

#[test]
fn test_workspace_policy_instantiation_with_all_fields() {
    let policy = WorkspacePolicy {
        allow_dirty_git: true,
        allow_network: true,
        preserve_on_failure: true,
    };

    assert!(policy.allow_dirty_git);
    assert!(policy.allow_network);
    assert!(policy.preserve_on_failure);
}

#[test]
fn test_workspace_policy_serde_roundtrip() {
    let policy = WorkspacePolicy {
        allow_dirty_git: true,
        allow_network: false,
        preserve_on_failure: true,
    };

    let serialized = serde_json::to_string(&policy).expect("serialization should succeed");
    let deserialized: WorkspacePolicy =
        serde_json::from_str(&serialized).expect("deserialization should succeed");

    assert_eq!(policy.allow_dirty_git, deserialized.allow_dirty_git);
    assert_eq!(policy.allow_network, deserialized.allow_network);
    assert_eq!(policy.preserve_on_failure, deserialized.preserve_on_failure);
}

#[test]
fn test_workspace_policy_json_format() {
    let policy = WorkspacePolicy {
        allow_dirty_git: false,
        allow_network: true,
        preserve_on_failure: false,
    };

    let serialized = serde_json::to_string(&policy).expect("serialization should succeed");
    assert_eq!(
        serialized,
        "{\"allow_dirty_git\":false,\"allow_network\":true,\"preserve_on_failure\":false}"
    );
}

#[test]
fn test_workspace_policy_all_false() {
    let policy = WorkspacePolicy {
        allow_dirty_git: false,
        allow_network: false,
        preserve_on_failure: false,
    };

    assert!(!policy.allow_dirty_git);
    assert!(!policy.allow_network);
    assert!(!policy.preserve_on_failure);
}

#[test]
fn test_workspace_policy_all_true() {
    let policy = WorkspacePolicy {
        allow_dirty_git: true,
        allow_network: true,
        preserve_on_failure: true,
    };

    assert!(policy.allow_dirty_git);
    assert!(policy.allow_network);
    assert!(policy.preserve_on_failure);
}

#[test]
fn test_reset_strategy_has_all_variants() {
    let _ = ResetStrategy::None;
    let _ = ResetStrategy::CleanClone;
    let _ = ResetStrategy::RestoreFiles;
}

#[test]
fn test_reset_strategy_serde_roundtrip() {
    let strategies = [
        ResetStrategy::None,
        ResetStrategy::CleanClone,
        ResetStrategy::RestoreFiles,
    ];

    for strategy in &strategies {
        let serialized = serde_json::to_string(strategy).expect("serialization should succeed");
        let deserialized: ResetStrategy =
            serde_json::from_str(&serialized).expect("deserialization should succeed");
        assert_eq!(*strategy, deserialized);
    }
}

#[test]
fn test_fixture_project_integration_with_workspace_policy_and_reset_strategy() {
    let policy = WorkspacePolicy {
        allow_dirty_git: true,
        allow_network: false,
        preserve_on_failure: true,
    };

    let fixture = FixtureProject::new(
        "cli-basic".to_string(),
        "Basic CLI smoke test fixture".to_string(),
        policy,
        ResetStrategy::CleanClone,
    );

    assert_eq!(fixture.name, "cli-basic");
    assert!(fixture.workspace_policy.allow_dirty_git);
    assert!(!fixture.workspace_policy.allow_network);
    assert!(fixture.workspace_policy.preserve_on_failure);
    assert_eq!(fixture.reset_strategy, ResetStrategy::CleanClone);
}

#[test]
fn test_fixture_project_complete_serde_roundtrip() {
    let policy = WorkspacePolicy {
        allow_dirty_git: false,
        allow_network: true,
        preserve_on_failure: false,
    };

    let fixture = FixtureProject::new(
        "api-project".to_string(),
        "API test fixture".to_string(),
        policy,
        ResetStrategy::RestoreFiles,
    )
    .with_setup_script("scripts/setup.sh".to_string())
    .with_teardown_script("scripts/teardown.sh".to_string())
    .with_files(vec![FixtureFile {
        path: "src/main.rs".to_string(),
        content: "fn main() {}".to_string(),
        executable: true,
    }])
    .with_configs(vec![FixtureConfig {
        path: "config.toml".to_string(),
        format: ConfigFormat::Toml,
    }])
    .with_transcripts(vec![Transcript {
        path: "recording.json".to_string(),
        transcript_type: TranscriptType::Recording,
    }]);

    let serialized = serde_json::to_string(&fixture).expect("serialization should succeed");
    let deserialized: FixtureProject =
        serde_json::from_str(&serialized).expect("deserialization should succeed");

    assert_eq!(fixture.name, deserialized.name);
    assert_eq!(fixture.description, deserialized.description);
    assert_eq!(
        fixture.workspace_policy.allow_dirty_git,
        deserialized.workspace_policy.allow_dirty_git
    );
    assert_eq!(
        fixture.workspace_policy.allow_network,
        deserialized.workspace_policy.allow_network
    );
    assert_eq!(
        fixture.workspace_policy.preserve_on_failure,
        deserialized.workspace_policy.preserve_on_failure
    );
    assert_eq!(fixture.reset_strategy, deserialized.reset_strategy);
    assert_eq!(fixture.setup_script, deserialized.setup_script);
    assert_eq!(fixture.teardown_script, deserialized.teardown_script);
    assert_eq!(fixture.files, deserialized.files);
    assert_eq!(fixture.configs, deserialized.configs);
    assert_eq!(fixture.transcripts, deserialized.transcripts);
}

#[test]
fn test_fixture_project_all_reset_strategies_with_policy() {
    let strategies = [
        (
            ResetStrategy::None,
            WorkspacePolicy {
                allow_dirty_git: false,
                allow_network: false,
                preserve_on_failure: false,
            },
        ),
        (
            ResetStrategy::CleanClone,
            WorkspacePolicy {
                allow_dirty_git: false,
                allow_network: true,
                preserve_on_failure: false,
            },
        ),
        (
            ResetStrategy::RestoreFiles,
            WorkspacePolicy {
                allow_dirty_git: true,
                allow_network: false,
                preserve_on_failure: true,
            },
        ),
    ];

    for (strategy, policy) in strategies {
        let fixture = FixtureProject::new(
            "test-fixture".to_string(),
            "Test fixture".to_string(),
            policy.clone(),
            strategy.clone(),
        );
        assert_eq!(fixture.reset_strategy, strategy);
        assert_eq!(fixture.workspace_policy, policy);
    }
}

#[test]
fn test_config_format_all_variants() {
    let formats = [
        ConfigFormat::Toml,
        ConfigFormat::Json,
        ConfigFormat::Yaml,
        ConfigFormat::Env,
    ];
    for format in &formats {
        let serialized = serde_json::to_string(format).expect("serialization should succeed");
        let deserialized: ConfigFormat =
            serde_json::from_str(&serialized).expect("deserialization should succeed");
        assert_eq!(*format, deserialized);
    }
}

#[test]
fn test_transcript_type_all_variants() {
    let types = [
        TranscriptType::Recording,
        TranscriptType::Transcript,
        TranscriptType::Snapshot,
    ];
    for transcript_type in &types {
        let serialized =
            serde_json::to_string(transcript_type).expect("serialization should succeed");
        let deserialized: TranscriptType =
            serde_json::from_str(&serialized).expect("deserialization should succeed");
        assert_eq!(*transcript_type, deserialized);
    }
}
