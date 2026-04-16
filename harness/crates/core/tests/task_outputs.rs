use opencode_core::types::task_outputs::TaskOutputs;

#[test]
fn test_task_outputs_instantiation() {
    let outputs = TaskOutputs::new(
        "test stdout".to_string(),
        "test stderr".to_string(),
        vec!["file1.txt".to_string(), "file2.rs".to_string()],
        vec!["file3.rs".to_string()],
    );

    assert_eq!(outputs.stdout, "test stdout");
    assert_eq!(outputs.stderr, "test stderr");
    assert_eq!(outputs.files_created.len(), 2);
    assert_eq!(outputs.files_modified.len(), 1);
}

#[test]
fn test_task_outputs_can_be_serialized_to_json() {
    let outputs = TaskOutputs::new(
        "hello world".to_string(),
        "error message".to_string(),
        vec!["new_file.txt".to_string()],
        vec!["modified.rs".to_string()],
    );

    let json = serde_json::to_string(&outputs).expect("serialization should succeed");
    assert!(json.contains("\"stdout\":\"hello world\""));
    assert!(json.contains("\"stderr\":\"error message\""));
    assert!(json.contains("\"files_created\":[\"new_file.txt\"]"));
    assert!(json.contains("\"files_modified\":[\"modified.rs\"]"));
}

#[test]
fn test_task_outputs_can_be_deserialized_from_json() {
    let json = r#"{
        "stdout": "output text",
        "stderr": "error text",
        "files_created": ["a.txt", "b.rs"],
        "files_modified": ["c.rs"]
    }"#;

    let outputs: TaskOutputs = serde_json::from_str(json).expect("deserialization should succeed");

    assert_eq!(outputs.stdout, "output text");
    assert_eq!(outputs.stderr, "error text");
    assert_eq!(outputs.files_created, vec!["a.txt", "b.rs"]);
    assert_eq!(outputs.files_modified, vec!["c.rs"]);
}

#[test]
fn test_task_outputs_serde_roundtrip() {
    let outputs = TaskOutputs::new(
        "stdout content",
        "stderr content",
        vec!["created1.txt".to_string()],
        vec!["modified1.rs".to_string(), "modified2.rs".to_string()],
    );

    let serialized = serde_json::to_string(&outputs).expect("serialization should succeed");
    let deserialized: TaskOutputs =
        serde_json::from_str(&serialized).expect("deserialization should succeed");

    assert_eq!(outputs, deserialized);
}

#[test]
fn test_task_outputs_fields_are_accessible_and_correctly_typed() {
    let outputs = TaskOutputs::new(
        "out".to_string(),
        "err".to_string(),
        vec!["a.txt".to_string()],
        vec!["b.txt".to_string()],
    );

    assert_eq!(outputs.stdout, "out");
    assert_eq!(outputs.stderr, "err");
    assert_eq!(outputs.files_created, vec!["a.txt"]);
    assert_eq!(outputs.files_modified, vec!["b.txt"]);
}

#[test]
fn test_task_outputs_empty_vectors() {
    let outputs = TaskOutputs::new(String::new(), String::new(), vec![], vec![]);

    assert!(outputs.stdout.is_empty());
    assert!(outputs.stderr.is_empty());
    assert!(outputs.files_created.is_empty());
    assert!(outputs.files_modified.is_empty());
}

#[test]
fn test_task_outputs_clone() {
    let outputs = TaskOutputs::new(
        "original stdout".to_string(),
        "original stderr".to_string(),
        vec!["file1.txt".to_string()],
        vec!["file2.txt".to_string()],
    );

    let cloned = outputs.clone();

    assert_eq!(cloned, outputs);
}
