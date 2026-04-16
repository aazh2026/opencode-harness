use crate::error::{ErrorType, Result};
use crate::loaders::TaskLoader;
use crate::types::assertion::AssertionType;
use crate::types::task::Task;
use std::path::Path;
use std::process::{Command, Output};

pub struct DifferentialRunner<L: TaskLoader> {
    pub task_loader: L,
}

impl<L: TaskLoader> DifferentialRunner<L> {
    pub fn new(task_loader: L) -> Self {
        Self { task_loader }
    }

    pub fn execute(&self, task: &Task) -> Result<DifferentialResult> {
        let task_input = &task.input;

        let output = self.run_command(&task_input.command, &task_input.args, &task_input.cwd)?;

        let assertions_passed = self.evaluate_assertions(&output, &task.expected_assertions)?;

        Ok(DifferentialResult {
            task_id: task.id.clone(),
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            assertions_passed,
            output_changed: true,
        })
    }

    pub fn execute_from_path(&self, path: &Path) -> Result<Vec<DifferentialResult>> {
        let tasks = self.task_loader.load_from_dir(path)?;
        self.execute_tasks(&tasks)
    }

    pub fn execute_single(&self, path: &Path) -> Result<DifferentialResult> {
        let task = self.task_loader.load_single(path)?;
        self.execute(&task)
    }

    fn execute_tasks(&self, tasks: &[Task]) -> Result<Vec<DifferentialResult>> {
        let mut results = Vec::new();
        for task in tasks {
            let result = self.execute(task)?;
            results.push(result);
        }
        Ok(results)
    }

    fn run_command(&self, command: &str, args: &[String], cwd: &str) -> Result<Output> {
        let output = Command::new(command)
            .args(args)
            .current_dir(cwd)
            .output()
            .map_err(|e| {
                ErrorType::Runner(format!("Failed to execute command '{}': {}", command, e))
            })?;

        Ok(output)
    }

    fn evaluate_assertions(&self, output: &Output, assertions: &[AssertionType]) -> Result<bool> {
        for assertion in assertions {
            if !self.check_assertion(output, assertion)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn check_assertion(&self, output: &Output, assertion: &AssertionType) -> Result<bool> {
        match assertion {
            AssertionType::ExitCodeEquals(expected_code) => {
                let actual_code = output.status.code().unwrap_or(-1) as u32;
                Ok(actual_code == *expected_code)
            }
            AssertionType::StdoutContains(expected) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                Ok(stdout.contains(expected))
            }
            AssertionType::StderrContains(expected) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Ok(stderr.contains(expected))
            }
            AssertionType::FileChanged(_) => Ok(false),
            AssertionType::NoExtraFilesChanged => Ok(true),
            AssertionType::PermissionPromptSeen(_) => Ok(false),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DifferentialResult {
    pub task_id: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub assertions_passed: bool,
    pub output_changed: bool,
}

impl DifferentialResult {
    pub fn new(task_id: String) -> Self {
        Self {
            task_id,
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
            assertions_passed: false,
            output_changed: false,
        }
    }

    pub fn passed(&self) -> bool {
        self.assertions_passed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loaders::DefaultTaskLoader;
    use crate::types::agent_mode::AgentMode;
    use crate::types::entry_mode::EntryMode;
    use crate::types::execution_policy::ExecutionPolicy;
    use crate::types::on_missing_dependency::OnMissingDependency;
    use crate::types::provider_mode::ProviderMode;
    use crate::types::severity::Severity;
    use crate::types::TaskCategory;
    use crate::types::TaskInput;
    use tempfile::TempDir;

    fn create_test_task() -> Task {
        Task::new(
            "TEST-001",
            "Test Task",
            TaskCategory::Smoke,
            "fixtures/projects/cli-basic",
            "Test description",
            "Test expected outcome",
            vec!["echo exists".to_string()],
            EntryMode::CLI,
            AgentMode::OneShot,
            ProviderMode::Both,
            TaskInput::new("echo", vec!["hello".to_string()], "/tmp"),
            vec![AssertionType::ExitCodeEquals(0)],
            Severity::High,
            ExecutionPolicy::ManualCheck,
            60,
            OnMissingDependency::Fail,
        )
    }

    #[test]
    fn test_differential_runner_creation() {
        let loader = DefaultTaskLoader::new();
        let runner = DifferentialRunner::new(loader);
        drop(runner);
    }

    #[test]
    fn test_differential_runner_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        let loader = DefaultTaskLoader::new();
        let _runner = DifferentialRunner::new(loader);
        assert_send_sync::<DifferentialRunner<DefaultTaskLoader>>();
    }

    #[test]
    fn test_differential_result_creation() {
        let result = DifferentialResult::new("TEST-001".to_string());
        assert_eq!(result.task_id, "TEST-001");
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.is_empty());
        assert!(result.stderr.is_empty());
        assert!(!result.passed());
    }

    #[test]
    fn test_differential_result_passed_method() {
        let mut result = DifferentialResult::new("TEST-001".to_string());
        assert!(!result.passed());

        result.assertions_passed = true;
        assert!(result.passed());
    }

    #[test]
    fn test_differential_runner_execute_echo_command() {
        let loader = DefaultTaskLoader::new();
        let runner = DifferentialRunner::new(loader);

        let temp_dir = TempDir::new().unwrap();
        let task_yaml = temp_dir.path().join("task.yaml");
        std::fs::write(
            &task_yaml,
            r#"
id: TEST-EXEC-001
title: Echo Test
category: smoke
fixture_project: fixtures/projects/cli-basic
description: Test echo command
expected_outcome: Echo works
preconditions:
  - echo exists
entry_mode: CLI
agent_mode: OneShot
provider_mode: Both
input:
  command: echo
  args: ["hello"]
  cwd: "/tmp"
expected_assertions:
  - type: exit_code_equals
    value: 0
severity: High
tags: []
execution_policy: ManualCheck
timeout_seconds: 60
on_missing_dependency: Fail
"#,
        )
        .unwrap();

        let result = runner.execute_single(&task_yaml);
        assert!(result.is_ok(), "execute_single failed: {:?}", result.err());

        let differential_result = result.unwrap();
        assert_eq!(differential_result.task_id, "TEST-EXEC-001");
        assert_eq!(differential_result.exit_code, 0);
        assert!(differential_result.stdout.contains("hello"));
        assert!(differential_result.assertions_passed);
    }

    #[test]
    fn test_differential_runner_execute_with_stdout_assertion() {
        let loader = DefaultTaskLoader::new();
        let runner = DifferentialRunner::new(loader);

        let temp_dir = TempDir::new().unwrap();
        let task_yaml = temp_dir.path().join("task.yaml");
        std::fs::write(
            &task_yaml,
            r#"
id: TEST-STDOUT-001
title: Stdout Assertion Test
category: smoke
fixture_project: fixtures/projects/cli-basic
description: Test stdout assertion
expected_outcome: Stdout assertion passes
preconditions: []
entry_mode: CLI
agent_mode: OneShot
provider_mode: Both
input:
  command: echo
  args: ["world"]
  cwd: "/tmp"
expected_assertions:
  - type: stdout_contains
    value: "world"
severity: High
tags: []
execution_policy: ManualCheck
timeout_seconds: 60
on_missing_dependency: Fail
"#,
        )
        .unwrap();

        let result = runner.execute_single(&task_yaml).unwrap();
        assert!(result.stdout.contains("world"));
        assert!(result.assertions_passed);
    }

    #[test]
    fn test_differential_runner_execute_with_failing_assertion() {
        let loader = DefaultTaskLoader::new();
        let runner = DifferentialRunner::new(loader);

        let temp_dir = TempDir::new().unwrap();
        let task_yaml = temp_dir.path().join("task.yaml");
        std::fs::write(
            &task_yaml,
            r#"
id: TEST-FAIL-001
title: Failing Test
category: smoke
fixture_project: fixtures/projects/cli-basic
description: Test failing assertion
expected_outcome: Assertion fails
preconditions: []
entry_mode: CLI
agent_mode: OneShot
provider_mode: Both
input:
  command: echo
  args: ["hello"]
  cwd: "/tmp"
expected_assertions:
  - type: stdout_contains
    value: "goodbye"
severity: High
tags: []
execution_policy: ManualCheck
timeout_seconds: 60
on_missing_dependency: Fail
"#,
        )
        .unwrap();

        let result = runner.execute_single(&task_yaml).unwrap();
        assert!(!result.stdout.contains("goodbye"));
        assert!(!result.assertions_passed);
    }

    #[test]
    fn test_differential_runner_execute_from_directory() {
        let loader = DefaultTaskLoader::new();
        let runner = DifferentialRunner::new(loader);

        let temp_dir = TempDir::new().unwrap();
        std::fs::write(
            temp_dir.path().join("task1.yaml"),
            r#"
id: TEST-DIR-001
title: Dir Test 1
category: smoke
fixture_project: fixtures/projects/cli-basic
description: Test 1
expected_outcome: Passes
preconditions: []
entry_mode: CLI
agent_mode: OneShot
provider_mode: Both
input:
  command: echo
  args: ["task1"]
  cwd: "/tmp"
expected_assertions:
  - type: exit_code_equals
    value: 0
severity: High
tags: []
execution_policy: ManualCheck
timeout_seconds: 60
on_missing_dependency: Fail
"#,
        )
        .unwrap();

        std::fs::write(
            temp_dir.path().join("task2.yaml"),
            r#"
id: TEST-DIR-002
title: Dir Test 2
category: smoke
fixture_project: fixtures/projects/cli-basic
description: Test 2
expected_outcome: Passes
preconditions: []
entry_mode: CLI
agent_mode: OneShot
provider_mode: Both
input:
  command: echo
  args: ["task2"]
  cwd: "/tmp"
expected_assertions:
  - type: exit_code_equals
    value: 0
severity: High
tags: []
execution_policy: ManualCheck
timeout_seconds: 60
on_missing_dependency: Fail
"#,
        )
        .unwrap();

        let results = runner.execute_from_path(temp_dir.path()).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_differential_runner_with_task_loader_integration() {
        let loader = DefaultTaskLoader::new();
        let runner = DifferentialRunner::new(loader);

        let temp_dir = TempDir::new().unwrap();
        let task_yaml = temp_dir.path().join("integration.yaml");
        std::fs::write(
            &task_yaml,
            r#"
id: TEST-INT-001
title: Integration Test
category: integration
fixture_project: fixtures/projects/cli-basic
description: Integration test with TaskLoader
expected_outcome: Works
preconditions: []
entry_mode: CLI
agent_mode: OneShot
provider_mode: Both
input:
  command: echo
  args: ["integration"]
  cwd: "/tmp"
expected_assertions:
  - type: exit_code_equals
    value: 0
  - type: stdout_contains
    value: "integration"
severity: High
tags: []
execution_policy: ManualCheck
timeout_seconds: 60
on_missing_dependency: Fail
"#,
        )
        .unwrap();

        let loaded_task = runner.task_loader.load_single(&task_yaml).unwrap();
        assert_eq!(loaded_task.id, "TEST-INT-001");

        let result = runner.execute(&loaded_task).unwrap();
        assert!(result.assertions_passed);
    }

    #[test]
    fn test_differential_result_equality() {
        let result1 = DifferentialResult::new("TEST-001".to_string());
        let result2 = DifferentialResult::new("TEST-001".to_string());
        assert_eq!(result1, result2);

        let mut result3 = DifferentialResult::new("TEST-002".to_string());
        assert_ne!(result1, result3);

        result3.task_id = "TEST-001".to_string();
        assert_eq!(result1, result3);
    }

    #[test]
    fn test_differential_runner_compare_two_commands() {
        let loader = DefaultTaskLoader::new();
        let runner = DifferentialRunner::new(loader);

        let temp_dir = TempDir::new().unwrap();

        let task1_yaml = temp_dir.path().join("task1.yaml");
        std::fs::write(
            &task1_yaml,
            r#"
id: TEST-CMP-001
title: Compare 1
category: smoke
fixture_project: fixtures/projects/cli-basic
description: Compare test 1
expected_outcome: Passes
preconditions: []
entry_mode: CLI
agent_mode: OneShot
provider_mode: Both
input:
  command: echo
  args: ["first"]
  cwd: "/tmp"
expected_assertions:
  - type: exit_code_equals
    value: 0
severity: High
tags: []
execution_policy: ManualCheck
timeout_seconds: 60
on_missing_dependency: Fail
"#,
        )
        .unwrap();

        let task2_yaml = temp_dir.path().join("task2.yaml");
        std::fs::write(
            &task2_yaml,
            r#"
id: TEST-CMP-002
title: Compare 2
category: smoke
fixture_project: fixtures/projects/cli-basic
description: Compare test 2
expected_outcome: Passes
preconditions: []
entry_mode: CLI
agent_mode: OneShot
provider_mode: Both
input:
  command: echo
  args: ["second"]
  cwd: "/tmp"
expected_assertions:
  - type: exit_code_equals
    value: 0
severity: High
tags: []
execution_policy: ManualCheck
timeout_seconds: 60
on_missing_dependency: Fail
"#,
        )
        .unwrap();

        let result1 = runner.execute_single(&task1_yaml).unwrap();
        let result2 = runner.execute_single(&task2_yaml).unwrap();

        assert_ne!(result1.stdout, result2.stdout);
        assert_eq!(result1.exit_code, result2.exit_code);
    }
}
