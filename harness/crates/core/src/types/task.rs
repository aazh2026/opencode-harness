use serde::{Deserialize, Serialize};

use super::agent_mode::AgentMode;
use super::allowed_variance::AllowedVariance;
use super::assertion::AssertionType;
use super::entry_mode::EntryMode;
use super::execution_policy::ExecutionPolicy;
use super::on_missing_dependency::OnMissingDependency;
use super::provider_mode::ProviderMode;
use super::severity::Severity;
use super::task_input::TaskInput;
use super::task_outputs::TaskOutputs;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub category: TaskCategory,
    pub fixture_project: String,
    pub description: String,
    pub expected_outcome: String,
    pub preconditions: Vec<String>,
    pub entry_mode: EntryMode,
    pub agent_mode: AgentMode,
    pub provider_mode: ProviderMode,
    pub input: TaskInput,
    pub expected_assertions: Vec<AssertionType>,
    pub allowed_variance: Option<AllowedVariance>,
    pub severity: Severity,
    pub tags: Vec<String>,
    pub execution_policy: ExecutionPolicy,
    pub timeout_seconds: u64,
    pub on_missing_dependency: OnMissingDependency,
    pub expected_outputs: Option<TaskOutputs>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskCategory {
    Core,
    Schema,
    Integration,
    Regression,
    Smoke,
    Performance,
    Security,
}

impl Task {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        category: TaskCategory,
        fixture_project: impl Into<String>,
        description: impl Into<String>,
        expected_outcome: impl Into<String>,
        preconditions: Vec<String>,
        entry_mode: EntryMode,
        agent_mode: AgentMode,
        provider_mode: ProviderMode,
        input: TaskInput,
        expected_assertions: Vec<AssertionType>,
        severity: Severity,
        execution_policy: ExecutionPolicy,
        timeout_seconds: u64,
        on_missing_dependency: OnMissingDependency,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            category,
            fixture_project: fixture_project.into(),
            description: description.into(),
            expected_outcome: expected_outcome.into(),
            preconditions,
            entry_mode,
            agent_mode,
            provider_mode,
            input,
            expected_assertions,
            allowed_variance: None,
            severity,
            tags: Vec::new(),
            execution_policy,
            timeout_seconds,
            on_missing_dependency,
            expected_outputs: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = Task::new(
            "P2-001",
            "Define Task Schema",
            TaskCategory::Schema,
            "fixtures/projects/example",
            "Create harness/tasks/schema.yaml defining Task struct",
            "Schema validates Task definitions correctly",
            vec!["opencode binary exists".to_string()],
            EntryMode::CLI,
            AgentMode::OneShot,
            ProviderMode::Both,
            TaskInput::new("opencode", vec!["--help".to_string()], "/project"),
            vec![AssertionType::ExitCodeEquals(0)],
            Severity::High,
            ExecutionPolicy::ManualCheck,
            300,
            OnMissingDependency::Fail,
        );

        assert_eq!(task.id, "P2-001");
        assert_eq!(task.title, "Define Task Schema");
        assert_eq!(task.category, TaskCategory::Schema);
        assert_eq!(task.fixture_project, "fixtures/projects/example");
    }

    #[test]
    fn test_task_serde_yaml() {
        let task = Task::new(
            "P2-001",
            "Define Task Schema",
            TaskCategory::Schema,
            "fixtures/projects/example",
            "Create harness/tasks/schema.yaml defining Task struct",
            "Schema validates Task definitions correctly",
            vec!["opencode binary exists".to_string()],
            EntryMode::CLI,
            AgentMode::OneShot,
            ProviderMode::Both,
            TaskInput::new("opencode", vec!["--help".to_string()], "/project"),
            vec![AssertionType::ExitCodeEquals(0)],
            Severity::High,
            ExecutionPolicy::ManualCheck,
            300,
            OnMissingDependency::Fail,
        );

        let yaml = serde_yaml::to_string(&task).unwrap();
        assert!(yaml.contains("id: P2-001"));
        assert!(yaml.contains("title: Define Task Schema"));
        assert!(yaml.contains("category: schema"));

        let deserialized: Task = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(deserialized, task);
    }

    #[test]
    fn test_task_serde_json() {
        let task = Task::new(
            "P2-001",
            "Define Task Schema",
            TaskCategory::Schema,
            "fixtures/projects/example",
            "Create harness/tasks/schema.yaml defining Task struct",
            "Schema validates Task definitions correctly",
            vec!["opencode binary exists".to_string()],
            EntryMode::CLI,
            AgentMode::OneShot,
            ProviderMode::Both,
            TaskInput::new("opencode", vec!["--help".to_string()], "/project"),
            vec![AssertionType::ExitCodeEquals(0)],
            Severity::High,
            ExecutionPolicy::ManualCheck,
            300,
            OnMissingDependency::Fail,
        );

        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("\"id\":\"P2-001\""));
        assert!(json.contains("\"title\":\"Define Task Schema\""));
        assert!(json.contains("\"category\":\"schema\""));

        let deserialized: Task = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, task);
    }

    #[test]
    fn test_task_category_variants() {
        let categories = vec![
            (TaskCategory::Core, "core"),
            (TaskCategory::Schema, "schema"),
            (TaskCategory::Integration, "integration"),
            (TaskCategory::Regression, "regression"),
            (TaskCategory::Smoke, "smoke"),
            (TaskCategory::Performance, "performance"),
            (TaskCategory::Security, "security"),
        ];

        for (category, expected_str) in categories {
            let json = serde_json::to_string(&category).unwrap();
            assert_eq!(json, format!("\"{}\"", expected_str));
        }
    }

    #[test]
    fn test_task_extended_fields_present() {
        let task = Task::new(
            "P2-001",
            "Define Task Schema",
            TaskCategory::Schema,
            "fixtures/projects/example",
            "Create harness/tasks/schema.yaml defining Task struct",
            "Schema validates Task definitions correctly",
            vec!["opencode binary exists".to_string()],
            EntryMode::CLI,
            AgentMode::OneShot,
            ProviderMode::Both,
            TaskInput::new("opencode", vec!["--help".to_string()], "/project"),
            vec![AssertionType::ExitCodeEquals(0)],
            Severity::High,
            ExecutionPolicy::ManualCheck,
            300,
            OnMissingDependency::Fail,
        );

        assert!(!task.preconditions.is_empty());
        assert_eq!(task.entry_mode, EntryMode::CLI);
        assert_eq!(task.agent_mode, AgentMode::OneShot);
        assert_eq!(task.provider_mode, ProviderMode::Both);
        assert!(task.input.command == "opencode");
        assert!(!task.expected_assertions.is_empty());
        assert!(task.allowed_variance.is_none());
        assert_eq!(task.severity, Severity::High);
        assert!(task.tags.is_empty());
        assert_eq!(task.execution_policy, ExecutionPolicy::ManualCheck);
        assert_eq!(task.timeout_seconds, 300);
        assert_eq!(task.on_missing_dependency, OnMissingDependency::Fail);
        assert!(task.expected_outputs.is_none());
    }
}
