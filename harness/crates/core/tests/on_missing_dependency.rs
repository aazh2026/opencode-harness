use opencode_core::types::on_missing_dependency::OnMissingDependency;

#[test]
fn test_on_missing_dependency_variants_exist() {
    let _ = OnMissingDependency::Fail;
    let _ = OnMissingDependency::Skip;
    let _ = OnMissingDependency::Warn;
    let _ = OnMissingDependency::Blocked;
}

#[test]
fn test_on_missing_dependency_serde_roundtrip() {
    let variants = [
        OnMissingDependency::Fail,
        OnMissingDependency::Skip,
        OnMissingDependency::Warn,
        OnMissingDependency::Blocked,
    ];

    for original in variants {
        let serialized = serde_json::to_string(&original).expect("serialization should succeed");
        let deserialized: OnMissingDependency =
            serde_json::from_str(&serialized).expect("deserialization should succeed");
        assert_eq!(
            original, deserialized,
            "roundtrip should preserve the value"
        );
    }
}

#[test]
fn test_on_missing_dependency_json_format() {
    assert_eq!(
        serde_json::to_string(&OnMissingDependency::Fail).unwrap(),
        "\"Fail\""
    );
    assert_eq!(
        serde_json::to_string(&OnMissingDependency::Skip).unwrap(),
        "\"Skip\""
    );
    assert_eq!(
        serde_json::to_string(&OnMissingDependency::Warn).unwrap(),
        "\"Warn\""
    );
    assert_eq!(
        serde_json::to_string(&OnMissingDependency::Blocked).unwrap(),
        "\"Blocked\""
    );
}
