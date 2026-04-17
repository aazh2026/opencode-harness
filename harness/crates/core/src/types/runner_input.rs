use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use super::capture_options::CaptureOptions;
use super::provider_mode::ProviderMode;
use super::task::Task;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunnerInput {
    pub task: Task,
    pub prepared_workspace_path: PathBuf,
    pub env_overrides: HashMap<String, String>,
    pub timeout_seconds: u64,
    pub binary_path: Option<PathBuf>,
    pub provider_mode: ProviderMode,
    pub capture_options: CaptureOptions,
}

impl RunnerInput {
    pub fn new(
        task: Task,
        prepared_workspace_path: PathBuf,
        env_overrides: HashMap<String, String>,
        timeout_seconds: u64,
        binary_path: Option<PathBuf>,
        provider_mode: ProviderMode,
        capture_options: CaptureOptions,
    ) -> Self {
        Self {
            task,
            prepared_workspace_path,
            env_overrides,
            timeout_seconds,
            binary_path,
            provider_mode,
            capture_options,
        }
    }

    pub fn with_task(mut self, task: Task) -> Self {
        self.task = task;
        self
    }

    pub fn with_prepared_workspace_path(mut self, prepared_workspace_path: PathBuf) -> Self {
        self.prepared_workspace_path = prepared_workspace_path;
        self
    }

    pub fn with_env_overrides(mut self, env_overrides: HashMap<String, String>) -> Self {
        self.env_overrides = env_overrides;
        self
    }

    pub fn with_timeout_seconds(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = timeout_seconds;
        self
    }

    pub fn with_binary_path(mut self, binary_path: Option<PathBuf>) -> Self {
        self.binary_path = binary_path;
        self
    }

    pub fn with_provider_mode(mut self, provider_mode: ProviderMode) -> Self {
        self.provider_mode = provider_mode;
        self
    }

    pub fn with_capture_options(mut self, capture_options: CaptureOptions) -> Self {
        self.capture_options = capture_options;
        self
    }
}

impl Default for RunnerInput {
    fn default() -> Self {
        Self {
            task: Task::new(
                "default".to_string(),
                "Default Task".to_string(),
                super::task::TaskCategory::Core,
                "default".to_string(),
                "Default task description".to_string(),
                "Default expected outcome".to_string(),
                Vec::new(),
                super::entry_mode::EntryMode::CLI,
                super::agent_mode::AgentMode::OneShot,
                ProviderMode::Both,
                super::task_input::TaskInput::new("echo", Vec::new(), "/tmp"),
                Vec::new(),
                super::severity::Severity::Medium,
                super::execution_policy::ExecutionPolicy::ManualCheck,
                300,
                super::on_missing_dependency::OnMissingDependency::Fail,
            ),
            prepared_workspace_path: PathBuf::from("/tmp"),
            env_overrides: HashMap::new(),
            timeout_seconds: 300,
            binary_path: None,
            provider_mode: ProviderMode::Both,
            capture_options: CaptureOptions::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        AgentMode, EntryMode, ExecutionPolicy, OnMissingDependency, ProviderMode, Severity,
        TaskCategory, TaskInput,
    };

    fn create_test_task() -> Task {
        Task::new(
            "TEST-001".to_string(),
            "Test Task".to_string(),
            TaskCategory::Core,
            "test/project".to_string(),
            "Test task description".to_string(),
            "Test expected outcome".to_string(),
            vec!["opencode exists".to_string()],
            EntryMode::CLI,
            AgentMode::OneShot,
            ProviderMode::Both,
            TaskInput::new("opencode", vec!["--help".to_string()], "/project"),
            vec![],
            Severity::High,
            ExecutionPolicy::ManualCheck,
            600,
            OnMissingDependency::Fail,
        )
    }

    #[test]
    fn test_runner_input_instantiation_with_all_fields() {
        let task = create_test_task();
        let prepared_workspace_path = PathBuf::from("/tmp/workspace");
        let mut env_overrides = HashMap::new();
        env_overrides.insert("RUST_LOG".to_string(), "debug".to_string());
        let timeout_seconds = 500u64;
        let binary_path = Some(PathBuf::from("/usr/bin/opencode"));
        let provider_mode = ProviderMode::OpenCode;
        let capture_options = CaptureOptions::new()
            .with_capture_stdout(true)
            .with_capture_stderr(true);

        let runner_input = RunnerInput::new(
            task.clone(),
            prepared_workspace_path.clone(),
            env_overrides.clone(),
            timeout_seconds,
            binary_path.clone(),
            provider_mode,
            capture_options.clone(),
        );

        assert_eq!(runner_input.task, task);
        assert_eq!(
            runner_input.prepared_workspace_path,
            prepared_workspace_path
        );
        assert_eq!(runner_input.env_overrides, env_overrides);
        assert_eq!(runner_input.timeout_seconds, timeout_seconds);
        assert_eq!(runner_input.binary_path, binary_path);
        assert_eq!(runner_input.provider_mode, provider_mode);
        assert_eq!(runner_input.capture_options, capture_options);
    }

    #[test]
    fn test_runner_input_builder_pattern() {
        let task = create_test_task();
        let prepared_workspace_path = PathBuf::from("/tmp/workspace");
        let mut env_overrides = HashMap::new();
        env_overrides.insert("TEST_VAR".to_string(), "test_value".to_string());

        let runner_input = RunnerInput::default()
            .with_task(task.clone())
            .with_prepared_workspace_path(prepared_workspace_path.clone())
            .with_env_overrides(env_overrides.clone())
            .with_timeout_seconds(700)
            .with_binary_path(Some(PathBuf::from("/custom/path/opencode")))
            .with_provider_mode(ProviderMode::OpenCodeRS)
            .with_capture_options(CaptureOptions::new().with_capture_artifacts(false));

        assert_eq!(runner_input.task, task);
        assert_eq!(
            runner_input.prepared_workspace_path,
            prepared_workspace_path
        );
        assert_eq!(runner_input.env_overrides, env_overrides);
        assert_eq!(runner_input.timeout_seconds, 700);
        assert_eq!(
            runner_input.binary_path,
            Some(PathBuf::from("/custom/path/opencode"))
        );
        assert_eq!(runner_input.provider_mode, ProviderMode::OpenCodeRS);
        assert!(!runner_input.capture_options.capture_artifacts);
    }

    #[test]
    fn test_runner_input_serde_roundtrip() {
        let mut env_overrides = HashMap::new();
        env_overrides.insert("KEY1".to_string(), "VALUE1".to_string());
        env_overrides.insert("KEY2".to_string(), "VALUE2".to_string());

        let runner_input = RunnerInput::new(
            create_test_task(),
            PathBuf::from("/test/workspace"),
            env_overrides.clone(),
            450,
            Some(PathBuf::from("/usr/local/bin/opencode")),
            ProviderMode::Both,
            CaptureOptions::new().with_max_output_size_bytes(Some(4096)),
        );

        let serialized =
            serde_json::to_string(&runner_input).expect("serialization should succeed");
        let deserialized: RunnerInput =
            serde_json::from_str(&serialized).expect("deserialization should succeed");

        assert_eq!(runner_input.task, deserialized.task);
        assert_eq!(
            runner_input.prepared_workspace_path,
            deserialized.prepared_workspace_path
        );
        assert_eq!(runner_input.env_overrides, deserialized.env_overrides);
        assert_eq!(runner_input.timeout_seconds, deserialized.timeout_seconds);
        assert_eq!(runner_input.binary_path, deserialized.binary_path);
        assert_eq!(runner_input.provider_mode, deserialized.provider_mode);
        assert_eq!(
            runner_input.capture_options.max_output_size_bytes,
            deserialized.capture_options.max_output_size_bytes
        );
    }

    #[test]
    fn test_runner_input_default_values() {
        let runner_input = RunnerInput::default();

        assert_eq!(runner_input.prepared_workspace_path, PathBuf::from("/tmp"));
        assert!(runner_input.env_overrides.is_empty());
        assert_eq!(runner_input.timeout_seconds, 300);
        assert!(runner_input.binary_path.is_none());
        assert_eq!(runner_input.provider_mode, ProviderMode::Both);
        assert!(runner_input.capture_options.capture_stdout);
        assert!(runner_input.capture_options.capture_stderr);
    }

    #[test]
    fn test_runner_input_json_format() {
        let runner_input = RunnerInput::default();
        let json = serde_json::to_string(&runner_input).expect("serialization should succeed");

        assert!(json.contains("\"prepared_workspace_path\""));
        assert!(json.contains("\"env_overrides\""));
        assert!(json.contains("\"timeout_seconds\""));
        assert!(json.contains("\"binary_path\""));
        assert!(json.contains("\"provider_mode\""));
        assert!(json.contains("\"capture_options\""));
    }
}
