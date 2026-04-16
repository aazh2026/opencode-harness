use opencode_core::types::task_input::TaskInput;

#[test]
fn test_task_input_instantiation() {
    let input = TaskInput::new("opencode", vec!["--help".to_string()], "/project");

    assert_eq!(input.command, "opencode");
    assert_eq!(input.args, vec!["--help"]);
    assert_eq!(input.cwd, "/project");
}

#[test]
fn test_task_input_fields_correct_types() {
    let input = TaskInput::new(
        "opencode",
        vec!["--version".to_string(), "--verbose".to_string()],
        "/workspace",
    );

    assert!(input.command.is_empty() == false);
    assert_eq!(input.command, "opencode");

    assert!(input.args.len() == 2);
    assert!(input.args[0] == "--version");
    assert!(input.args[1] == "--verbose");

    assert!(input.cwd.is_empty() == false);
    assert_eq!(input.cwd, "/workspace");
}

#[test]
fn test_task_input_serde_json() {
    let input = TaskInput::new("opencode", vec!["--help".to_string()], "/project");

    let json = serde_json::to_string(&input).expect("serialization should succeed");
    assert!(json.contains("\"command\":\"opencode\""));
    assert!(json.contains("\"args\":[\"--help\"]"));
    assert!(json.contains("\"cwd\":\"/project\""));

    let deserialized: TaskInput =
        serde_json::from_str(&json).expect("deserialization should succeed");
    assert_eq!(deserialized, input);
}

#[test]
fn test_task_input_serde_json_multiple_args() {
    let input = TaskInput::new(
        "cargo",
        vec![
            "test".to_string(),
            "--verbose".to_string(),
            "--".to_string(),
            "--nocapture".to_string(),
        ],
        "/workspace/project",
    );

    let json = serde_json::to_string(&input).expect("serialization should succeed");
    let deserialized: TaskInput =
        serde_json::from_str(&json).expect("deserialization should succeed");
    assert_eq!(deserialized, input);
}

#[test]
fn test_task_input_clone() {
    let input = TaskInput::new("opencode", vec!["--help".to_string()], "/project");
    let cloned = input.clone();

    assert_eq!(cloned, input);
}
