use opencode_core::types::severity::Severity;

#[test]
fn test_severity_variants_exist() {
    let _ = Severity::Critical;
    let _ = Severity::High;
    let _ = Severity::Medium;
    let _ = Severity::Low;
    let _ = Severity::Cosmetic;
}

#[test]
fn test_severity_serde_roundtrip() {
    let variants = [
        Severity::Critical,
        Severity::High,
        Severity::Medium,
        Severity::Low,
        Severity::Cosmetic,
    ];

    for original in variants {
        let serialized = serde_json::to_string(&original).expect("serialization should succeed");
        let deserialized: Severity =
            serde_json::from_str(&serialized).expect("deserialization should succeed");
        assert_eq!(
            original, deserialized,
            "roundtrip should preserve the value"
        );
    }
}

#[test]
fn test_severity_json_format() {
    assert_eq!(
        serde_json::to_string(&Severity::Critical).unwrap(),
        "\"Critical\""
    );
    assert_eq!(serde_json::to_string(&Severity::High).unwrap(), "\"High\"");
    assert_eq!(
        serde_json::to_string(&Severity::Medium).unwrap(),
        "\"Medium\""
    );
    assert_eq!(serde_json::to_string(&Severity::Low).unwrap(), "\"Low\"");
    assert_eq!(
        serde_json::to_string(&Severity::Cosmetic).unwrap(),
        "\"Cosmetic\""
    );
}
