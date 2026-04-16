use opencode_core::types::fixture::WorkspacePolicy;

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
