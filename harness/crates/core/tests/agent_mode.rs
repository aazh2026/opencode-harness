use opencode_core::types::agent_mode::AgentMode;

#[test]
fn test_agent_mode_variants_exist() {
    let _ = AgentMode::Interactive;
    let _ = AgentMode::Batch;
    let _ = AgentMode::Daemon;
    let _ = AgentMode::OneShot;
}

#[test]
fn test_agent_mode_serde_roundtrip() {
    let variants = [
        AgentMode::Interactive,
        AgentMode::Batch,
        AgentMode::Daemon,
        AgentMode::OneShot,
    ];

    for original in variants {
        let serialized = serde_json::to_string(&original).expect("serialization should succeed");
        let deserialized: AgentMode =
            serde_json::from_str(&serialized).expect("deserialization should succeed");
        assert_eq!(
            original, deserialized,
            "roundtrip should preserve the value"
        );
    }
}

#[test]
fn test_agent_mode_json_format() {
    assert_eq!(
        serde_json::to_string(&AgentMode::Interactive).unwrap(),
        "\"Interactive\""
    );
    assert_eq!(
        serde_json::to_string(&AgentMode::Batch).unwrap(),
        "\"Batch\""
    );
    assert_eq!(
        serde_json::to_string(&AgentMode::Daemon).unwrap(),
        "\"Daemon\""
    );
    assert_eq!(
        serde_json::to_string(&AgentMode::OneShot).unwrap(),
        "\"OneShot\""
    );
}
