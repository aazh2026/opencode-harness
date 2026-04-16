use opencode_core::types::execution_policy::ExecutionPolicy;

#[test]
fn test_execution_policy_variants_exist() {
    let _ = ExecutionPolicy::ManualCheck;
    let _ = ExecutionPolicy::Blocked;
    let _ = ExecutionPolicy::Skip;
}

#[test]
fn test_execution_policy_serde_roundtrip() {
    let variants = [
        ExecutionPolicy::ManualCheck,
        ExecutionPolicy::Blocked,
        ExecutionPolicy::Skip,
    ];

    for original in variants {
        let serialized = serde_json::to_string(&original).expect("serialization should succeed");
        let deserialized: ExecutionPolicy =
            serde_json::from_str(&serialized).expect("deserialization should succeed");
        assert_eq!(
            original, deserialized,
            "roundtrip should preserve the value"
        );
    }
}

#[test]
fn test_execution_policy_json_format() {
    assert_eq!(
        serde_json::to_string(&ExecutionPolicy::ManualCheck).unwrap(),
        "\"ManualCheck\""
    );
    assert_eq!(
        serde_json::to_string(&ExecutionPolicy::Blocked).unwrap(),
        "\"Blocked\""
    );
    assert_eq!(
        serde_json::to_string(&ExecutionPolicy::Skip).unwrap(),
        "\"Skip\""
    );
}
