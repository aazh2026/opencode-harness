use opencode_core::types::assertion::AssertionType;

#[test]
fn test_assertion_type_exit_code_equals() {
    let assertion = AssertionType::ExitCodeEquals(0);
    let serialized = serde_json::to_string(&assertion).expect("serialization should succeed");
    assert_eq!(serialized, "{\"type\":\"exit_code_equals\",\"value\":0}");

    let deserialized: AssertionType =
        serde_json::from_str(&serialized).expect("deserialization should succeed");
    assert_eq!(assertion, deserialized);
}

#[test]
fn test_assertion_type_stdout_contains() {
    let assertion = AssertionType::StdoutContains("Usage: opencode".to_string());
    let serialized = serde_json::to_string(&assertion).expect("serialization should succeed");
    assert_eq!(
        serialized,
        "{\"type\":\"stdout_contains\",\"value\":\"Usage: opencode\"}"
    );

    let deserialized: AssertionType =
        serde_json::from_str(&serialized).expect("deserialization should succeed");
    assert_eq!(assertion, deserialized);
}

#[test]
fn test_assertion_type_stderr_contains() {
    let assertion = AssertionType::StderrContains("Error: permission denied".to_string());
    let serialized = serde_json::to_string(&assertion).expect("serialization should succeed");
    assert_eq!(
        serialized,
        "{\"type\":\"stderr_contains\",\"value\":\"Error: permission denied\"}"
    );

    let deserialized: AssertionType =
        serde_json::from_str(&serialized).expect("deserialization should succeed");
    assert_eq!(assertion, deserialized);
}

#[test]
fn test_assertion_type_file_changed() {
    let assertion = AssertionType::FileChanged("src/main.rs".to_string());
    let serialized = serde_json::to_string(&assertion).expect("serialization should succeed");
    assert_eq!(
        serialized,
        "{\"type\":\"file_changed\",\"value\":\"src/main.rs\"}"
    );

    let deserialized: AssertionType =
        serde_json::from_str(&serialized).expect("deserialization should succeed");
    assert_eq!(assertion, deserialized);
}

#[test]
fn test_assertion_type_no_extra_files_changed() {
    let assertion = AssertionType::NoExtraFilesChanged;
    let serialized = serde_json::to_string(&assertion).expect("serialization should succeed");
    assert_eq!(serialized, "{\"type\":\"no_extra_files_changed\"}");

    let deserialized: AssertionType =
        serde_json::from_str(&serialized).expect("deserialization should succeed");
    assert_eq!(assertion, deserialized);
}

#[test]
fn test_assertion_type_permission_prompt_seen() {
    let assertion = AssertionType::PermissionPromptSeen("Allow access?".to_string());
    let serialized = serde_json::to_string(&assertion).expect("serialization should succeed");
    assert_eq!(
        serialized,
        "{\"type\":\"permission_prompt_seen\",\"value\":\"Allow access?\"}"
    );

    let deserialized: AssertionType =
        serde_json::from_str(&serialized).expect("deserialization should succeed");
    assert_eq!(assertion, deserialized);
}

#[test]
fn test_assertion_type_serde_roundtrip() {
    let variants = [
        AssertionType::ExitCodeEquals(0),
        AssertionType::ExitCodeEquals(1),
        AssertionType::StdoutContains("Hello".to_string()),
        AssertionType::StderrContains("Error".to_string()),
        AssertionType::FileChanged("path/to/file".to_string()),
        AssertionType::NoExtraFilesChanged,
        AssertionType::PermissionPromptSeen("?".to_string()),
    ];

    for original in variants {
        let serialized = serde_json::to_string(&original).expect("serialization should succeed");
        let deserialized: AssertionType =
            serde_json::from_str(&serialized).expect("deserialization should succeed");
        assert_eq!(
            original, deserialized,
            "roundtrip should preserve the value"
        );
    }
}

#[test]
fn test_assertion_type_json_format() {
    assert_eq!(
        serde_json::to_string(&AssertionType::ExitCodeEquals(0)).unwrap(),
        "{\"type\":\"exit_code_equals\",\"value\":0}"
    );
    assert_eq!(
        serde_json::to_string(&AssertionType::NoExtraFilesChanged).unwrap(),
        "{\"type\":\"no_extra_files_changed\"}"
    );
}
