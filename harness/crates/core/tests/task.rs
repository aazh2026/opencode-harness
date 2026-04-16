use opencode_core::types::agent_mode::AgentMode;
use opencode_core::types::allowed_variance::{AllowedVariance, TimingVariance};
use opencode_core::types::assertion::AssertionType;
use opencode_core::types::entry_mode::EntryMode;
use opencode_core::types::execution_policy::ExecutionPolicy;
use opencode_core::types::on_missing_dependency::OnMissingDependency;
use opencode_core::types::provider_mode::ProviderMode;
use opencode_core::types::severity::Severity;
use opencode_core::types::task::Task;
use opencode_core::types::task::TaskCategory;
use opencode_core::types::task_input::TaskInput;
use opencode_core::types::task_outputs::TaskOutputs;

fn create_test_task() -> Task {
    Task::new(
        "P2-001",
        "Define Task Schema",
        TaskCategory::Schema,
        "fixtures/projects/example",
        "Create harness/tasks/schema.yaml defining Task struct",
        "Schema validates Task definitions correctly",
        vec![
            "opencode binary exists".to_string(),
            "git available".to_string(),
        ],
        EntryMode::CLI,
        AgentMode::OneShot,
        ProviderMode::Both,
        TaskInput::new("opencode", vec!["--help".to_string()], "/project"),
        vec![
            AssertionType::ExitCodeEquals(0),
            AssertionType::StdoutContains("Usage:".to_string()),
        ],
        Severity::High,
        ExecutionPolicy::ManualCheck,
        300,
        OnMissingDependency::Fail,
    )
}

#[test]
fn test_extended_task_struct_has_all_required_fields() {
    let task = create_test_task();

    assert_eq!(task.id, "P2-001");
    assert_eq!(task.title, "Define Task Schema");
    assert_eq!(task.category, TaskCategory::Schema);
    assert_eq!(task.fixture_project, "fixtures/projects/example");
    assert_eq!(
        task.description,
        "Create harness/tasks/schema.yaml defining Task struct"
    );
    assert_eq!(
        task.expected_outcome,
        "Schema validates Task definitions correctly"
    );

    assert!(!task.preconditions.is_empty());
    assert_eq!(task.entry_mode, EntryMode::CLI);
    assert_eq!(task.agent_mode, AgentMode::OneShot);
    assert_eq!(task.provider_mode, ProviderMode::Both);
    assert!(!task.input.command.is_empty());
    assert!(!task.expected_assertions.is_empty());
    assert_eq!(task.severity, Severity::High);
    assert!(task.tags.is_empty());
    assert_eq!(task.execution_policy, ExecutionPolicy::ManualCheck);
    assert_eq!(task.timeout_seconds, 300);
    assert_eq!(task.on_missing_dependency, OnMissingDependency::Fail);
}

#[test]
fn test_task_struct_can_be_serialized_to_json() {
    let task = create_test_task();

    let json = serde_json::to_string(&task).expect("serialization should succeed");

    assert!(json.contains("\"id\":\"P2-001\""));
    assert!(json.contains("\"title\":\"Define Task Schema\""));
    assert!(json.contains("\"category\":\"schema\""));
    assert!(json.contains("\"fixture_project\":\"fixtures/projects/example\""));
    assert!(json.contains("\"preconditions\""));
    assert!(json.contains("\"entry_mode\":\"CLI\""));
    assert!(json.contains("\"agent_mode\":\"OneShot\""));
    assert!(json.contains("\"provider_mode\":\"Both\""));
    assert!(json.contains("\"severity\":\"High\""));
    assert!(json.contains("\"execution_policy\":\"ManualCheck\""));
    assert!(json.contains("\"timeout_seconds\":300"));
    assert!(json.contains("\"on_missing_dependency\":\"Fail\""));
}

#[test]
fn test_task_struct_can_be_deserialized_from_json() {
    let json = r#"{
        "id": "P2-001",
        "title": "Define Task Schema",
        "category": "schema",
        "fixture_project": "fixtures/projects/example",
        "description": "Create harness/tasks/schema.yaml defining Task struct",
        "expected_outcome": "Schema validates Task definitions correctly",
        "preconditions": ["opencode binary exists"],
        "entry_mode": "CLI",
        "agent_mode": "OneShot",
        "provider_mode": "Both",
        "input": {
            "command": "opencode",
            "args": ["--help"],
            "cwd": "/project"
        },
        "expected_assertions": [
            {"type": "exit_code_equals", "value": 0}
        ],
        "allowed_variance": null,
        "severity": "High",
        "tags": ["smoke", "cli"],
        "execution_policy": "ManualCheck",
        "timeout_seconds": 300,
        "on_missing_dependency": "Fail",
        "expected_outputs": null
    }"#;

    let task: Task = serde_json::from_str(json).expect("deserialization should succeed");

    assert_eq!(task.id, "P2-001");
    assert_eq!(task.title, "Define Task Schema");
    assert_eq!(task.category, TaskCategory::Schema);
    assert_eq!(task.fixture_project, "fixtures/projects/example");
    assert_eq!(task.preconditions, vec!["opencode binary exists"]);
    assert_eq!(task.entry_mode, EntryMode::CLI);
    assert_eq!(task.agent_mode, AgentMode::OneShot);
    assert_eq!(task.provider_mode, ProviderMode::Both);
    assert_eq!(task.input.command, "opencode");
    assert_eq!(task.input.args, vec!["--help"]);
    assert_eq!(task.input.cwd, "/project");
    assert_eq!(task.severity, Severity::High);
    assert_eq!(task.tags, vec!["smoke", "cli"]);
    assert_eq!(task.execution_policy, ExecutionPolicy::ManualCheck);
    assert_eq!(task.timeout_seconds, 300);
    assert_eq!(task.on_missing_dependency, OnMissingDependency::Fail);
}

#[test]
fn test_task_struct_serde_roundtrip() {
    let task = create_test_task();

    let json = serde_json::to_string(&task).expect("serialization should succeed");
    let deserialized: Task = serde_json::from_str(&json).expect("deserialization should succeed");

    assert_eq!(deserialized.id, task.id);
    assert_eq!(deserialized.title, task.title);
    assert_eq!(deserialized.category, task.category);
    assert_eq!(deserialized.fixture_project, task.fixture_project);
    assert_eq!(deserialized.preconditions, task.preconditions);
    assert_eq!(deserialized.entry_mode, task.entry_mode);
    assert_eq!(deserialized.agent_mode, task.agent_mode);
    assert_eq!(deserialized.provider_mode, task.provider_mode);
    assert_eq!(deserialized.input.command, task.input.command);
    assert_eq!(deserialized.expected_assertions, task.expected_assertions);
    assert_eq!(deserialized.severity, task.severity);
    assert_eq!(deserialized.tags, task.tags);
    assert_eq!(deserialized.execution_policy, task.execution_policy);
    assert_eq!(deserialized.timeout_seconds, task.timeout_seconds);
    assert_eq!(
        deserialized.on_missing_dependency,
        task.on_missing_dependency
    );
}

#[test]
fn test_all_new_fields_are_correctly_typed_and_accessible() {
    let preconditions = vec!["opencode binary exists".to_string()];
    let input = TaskInput::new("opencode", vec!["--help".to_string()], "/project");
    let expected_assertions = vec![AssertionType::ExitCodeEquals(0)];
    let allowed_variance = Some(AllowedVariance::new(
        vec![0, 1],
        Some(TimingVariance::new(Some(100), Some(500))),
        vec![r"\d+ items?".to_string()],
    ));
    let tags = vec!["smoke".to_string(), "cli".to_string()];
    let expected_outputs = Some(TaskOutputs::new(
        "stdout content",
        "stderr content",
        vec!["created.txt".to_string()],
        vec!["modified.rs".to_string()],
    ));

    let task = Task::new(
        "P2-001",
        "Test Task",
        TaskCategory::Smoke,
        "fixtures/projects/cli-basic",
        "Test description",
        "Test expected outcome",
        preconditions.clone(),
        EntryMode::CLI,
        AgentMode::OneShot,
        ProviderMode::Both,
        input.clone(),
        expected_assertions.clone(),
        Severity::Critical,
        ExecutionPolicy::ManualCheck,
        600,
        OnMissingDependency::Warn,
    );

    let _: Vec<String> = task.preconditions.clone();
    let _: EntryMode = task.entry_mode;
    let _: AgentMode = task.agent_mode;
    let _: ProviderMode = task.provider_mode;
    let _: TaskInput = task.input.clone();
    let _: Vec<AssertionType> = task.expected_assertions.clone();
    let _: Option<AllowedVariance> = task.allowed_variance.clone();
    let _: Severity = task.severity;
    let _: Vec<String> = task.tags.clone();
    let _: ExecutionPolicy = task.execution_policy;
    let _: u64 = task.timeout_seconds;
    let _: OnMissingDependency = task.on_missing_dependency;
    let _: Option<TaskOutputs> = task.expected_outputs.clone();
}

#[test]
fn test_task_integrates_with_entry_mode_enum() {
    let entry_modes = vec![
        EntryMode::CLI,
        EntryMode::API,
        EntryMode::Session,
        EntryMode::Permissions,
        EntryMode::Web,
        EntryMode::Workspace,
        EntryMode::Recovery,
    ];

    for entry_mode in entry_modes {
        let task = Task::new(
            "TEST-001",
            "Test Task",
            TaskCategory::Smoke,
            "fixtures/projects/example",
            "Test description",
            "Test outcome",
            vec![],
            entry_mode,
            AgentMode::OneShot,
            ProviderMode::Both,
            TaskInput::new("cmd", vec![], "/"),
            vec![],
            Severity::Low,
            ExecutionPolicy::Skip,
            30,
            OnMissingDependency::Skip,
        );
        assert_eq!(task.entry_mode, entry_mode);
    }
}

#[test]
fn test_task_integrates_with_agent_mode_enum() {
    let agent_modes = vec![
        AgentMode::Interactive,
        AgentMode::Batch,
        AgentMode::Daemon,
        AgentMode::OneShot,
    ];

    for agent_mode in agent_modes {
        let task = Task::new(
            "TEST-001",
            "Test Task",
            TaskCategory::Smoke,
            "fixtures/projects/example",
            "Test description",
            "Test outcome",
            vec![],
            EntryMode::CLI,
            agent_mode,
            ProviderMode::Both,
            TaskInput::new("cmd", vec![], "/"),
            vec![],
            Severity::Low,
            ExecutionPolicy::Skip,
            30,
            OnMissingDependency::Skip,
        );
        assert_eq!(task.agent_mode, agent_mode);
    }
}

#[test]
fn test_task_integrates_with_provider_mode_enum() {
    let provider_modes = vec![
        ProviderMode::OpenCode,
        ProviderMode::OpenCodeRS,
        ProviderMode::Both,
        ProviderMode::Either,
    ];

    for provider_mode in provider_modes {
        let task = Task::new(
            "TEST-001",
            "Test Task",
            TaskCategory::Smoke,
            "fixtures/projects/example",
            "Test description",
            "Test outcome",
            vec![],
            EntryMode::CLI,
            AgentMode::OneShot,
            provider_mode,
            TaskInput::new("cmd", vec![], "/"),
            vec![],
            Severity::Low,
            ExecutionPolicy::Skip,
            30,
            OnMissingDependency::Skip,
        );
        assert_eq!(task.provider_mode, provider_mode);
    }
}

#[test]
fn test_task_integrates_with_severity_enum() {
    let severities = vec![
        Severity::Critical,
        Severity::High,
        Severity::Medium,
        Severity::Low,
        Severity::Cosmetic,
    ];

    for severity in severities {
        let task = Task::new(
            "TEST-001",
            "Test Task",
            TaskCategory::Smoke,
            "fixtures/projects/example",
            "Test description",
            "Test outcome",
            vec![],
            EntryMode::CLI,
            AgentMode::OneShot,
            ProviderMode::Both,
            TaskInput::new("cmd", vec![], "/"),
            vec![],
            severity,
            ExecutionPolicy::Skip,
            30,
            OnMissingDependency::Skip,
        );
        assert_eq!(task.severity, severity);
    }
}

#[test]
fn test_task_yaml_serde_roundtrip() {
    let task = create_test_task();

    let yaml = serde_yaml::to_string(&task).expect("serialization should succeed");
    let deserialized: Task = serde_yaml::from_str(&yaml).expect("deserialization should succeed");

    assert_eq!(deserialized.id, task.id);
    assert_eq!(deserialized.title, task.title);
    assert_eq!(deserialized.category, task.category);
    assert_eq!(deserialized.entry_mode, task.entry_mode);
    assert_eq!(deserialized.agent_mode, task.agent_mode);
    assert_eq!(deserialized.provider_mode, task.provider_mode);
    assert_eq!(deserialized.severity, task.severity);
    assert_eq!(deserialized.timeout_seconds, task.timeout_seconds);
}

#[test]
fn test_task_with_all_optional_fields_set() {
    let allowed_variance = Some(AllowedVariance::new(
        vec![0],
        Some(TimingVariance::new(Some(0), Some(1000))),
        vec![".*".to_string()],
    ));
    let tags = vec!["smoke".to_string(), "cli".to_string(), "help".to_string()];
    let expected_outputs = Some(TaskOutputs::new("Success", "", vec![], vec![]));

    let mut task = create_test_task();
    task.allowed_variance = allowed_variance;
    task.tags = tags;
    task.expected_outputs = expected_outputs;

    assert!(task.allowed_variance.is_some());
    assert_eq!(task.tags.len(), 3);
    assert!(task.expected_outputs.is_some());
}

#[test]
fn test_task_clone() {
    let task = create_test_task();
    let cloned = task.clone();

    assert_eq!(cloned, task);
}
