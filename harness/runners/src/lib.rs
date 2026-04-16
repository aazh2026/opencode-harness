pub mod execution_result;

pub use execution_result::ExecutionResult;
pub use legacy_runner::LegacyRunner;
pub use runner_trait::Runner;
pub use rust_runner::RustRunner;

mod runner_trait {
    use super::ExecutionResult;
    use core::error::Result;
    use core::types::task::Task;

    pub trait Runner: Send + Sync {
        fn execute(&self, task: &Task) -> Result<ExecutionResult>;

        fn name(&self) -> &str;
    }
}

mod legacy_runner {
    use super::{ExecutionResult, Runner};
    use core::error::{ErrorType, Result};
    use core::types::task::Task;
    use core::types::task_status::TaskStatus;
    use std::process::Command;
    use std::time::Instant;

    pub struct LegacyRunner {
        name: String,
    }

    impl LegacyRunner {
        pub fn new(name: impl Into<String>) -> Self {
            Self { name: name.into() }
        }

        fn run_command(
            &self,
            command: &str,
            args: &[String],
            cwd: &str,
        ) -> Result<std::process::Output> {
            Command::new(command)
                .args(args)
                .current_dir(cwd)
                .output()
                .map_err(|e| {
                    ErrorType::Runner(format!("Failed to execute command '{}': {}", command, e))
                })
        }
    }

    impl Runner for LegacyRunner {
        fn execute(&self, task: &Task) -> Result<ExecutionResult> {
            let start = Instant::now();
            let task_input = &task.input;

            let output =
                self.run_command(&task_input.command, &task_input.args, &task_input.cwd)?;

            let duration_ms = start.elapsed().as_millis() as u64;
            let exit_code = output.status.code().unwrap_or(-1);
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            let status = TaskStatus::Done;

            Ok(ExecutionResult::new(&task.id)
                .with_status(status)
                .with_exit_code(exit_code)
                .with_stdout(stdout)
                .with_stderr(stderr)
                .with_duration_ms(duration_ms))
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use core::types::task_input::TaskInput;

        fn create_test_task(command: &str, args: Vec<String>, cwd: &str) -> Task {
            Task::new(
                "P2-003",
                "Define Runner Traits",
                core::types::task::TaskCategory::Schema,
                "test-fixture",
                "Create runner traits",
                "Traits defined correctly",
                vec![],
                core::types::entry_mode::EntryMode::CLI,
                core::types::agent_mode::AgentMode::OneShot,
                core::types::provider_mode::ProviderMode::Both,
                TaskInput::new(command, args, cwd),
                vec![],
                core::types::severity::Severity::High,
                core::types::execution_policy::ExecutionPolicy::ManualCheck,
                60,
                core::types::on_missing_dependency::OnMissingDependency::Fail,
            )
        }

        #[test]
        fn test_legacy_runner_creation() {
            let runner = LegacyRunner::new("opencode-rs");
            assert_eq!(runner.name(), "opencode-rs");
        }

        #[test]
        fn test_legacy_runner_execute_echo() {
            let runner = LegacyRunner::new("opencode-rs");
            let task = create_test_task("echo", vec!["hello".to_string()], "/tmp");

            let result = runner.execute(&task).unwrap();
            assert_eq!(result.task_id, "P2-003");
            assert_eq!(result.status, TaskStatus::Done);
            assert_eq!(result.exit_code, Some(0));
            assert!(result.stdout.contains("hello"));
        }

        #[test]
        fn test_legacy_runner_execute_with_stderr() {
            let runner = LegacyRunner::new("opencode-rs");
            let task = create_test_task(
                "sh",
                vec!["-c".to_string(), "echo error 1>&2".to_string()],
                "/tmp",
            );

            let result = runner.execute(&task).unwrap();
            assert_eq!(result.task_id, "P2-003");
            assert!(result.stderr.contains("error"));
        }

        #[test]
        fn test_legacy_runner_execute_nonexistent_command() {
            let runner = LegacyRunner::new("opencode-rs");
            let task = create_test_task("nonexistent_command_xyz", vec![], "/tmp");

            let result = runner.execute(&task);
            assert!(result.is_err());
        }

        #[test]
        fn test_legacy_runner_execute_with_args() {
            let runner = LegacyRunner::new("opencode-rs");
            let task = create_test_task(
                "printf",
                vec!["%s %d\n".to_string(), "test".to_string(), "42".to_string()],
                "/tmp",
            );

            let result = runner.execute(&task).unwrap();
            assert!(result.stdout.contains("test"));
            assert!(result.stdout.contains("42"));
        }

        #[test]
        fn test_legacy_runner_trait_impl() {
            fn assert_runner<T: Runner>() {}
            assert_runner::<LegacyRunner>();
        }

        #[test]
        fn test_legacy_runner_is_send_and_sync() {
            fn assert_send_sync<T: Send + Sync>() {}
            assert_send_sync::<LegacyRunner>();
        }
    }
}

mod rust_runner {
    use super::{ExecutionResult, Runner};
    use core::error::{ErrorType, Result};
    use core::types::task::Task;
    use core::types::task_status::TaskStatus;
    use std::process::Command;
    use std::time::Instant;

    pub struct RustRunner {
        name: String,
    }

    impl RustRunner {
        pub fn new(name: impl Into<String>) -> Self {
            Self { name: name.into() }
        }

        fn run_command(
            &self,
            command: &str,
            args: &[String],
            cwd: &str,
        ) -> Result<std::process::Output> {
            Command::new(command)
                .args(args)
                .current_dir(cwd)
                .output()
                .map_err(|e| {
                    ErrorType::Runner(format!("Failed to execute command '{}': {}", command, e))
                })
        }
    }

    impl Runner for RustRunner {
        fn execute(&self, task: &Task) -> Result<ExecutionResult> {
            let start = Instant::now();
            let task_input = &task.input;

            let output =
                self.run_command(&task_input.command, &task_input.args, &task_input.cwd)?;

            let duration_ms = start.elapsed().as_millis() as u64;
            let exit_code = output.status.code().unwrap_or(-1);
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            let status = TaskStatus::Done;

            Ok(ExecutionResult::new(&task.id)
                .with_status(status)
                .with_exit_code(exit_code)
                .with_stdout(stdout)
                .with_stderr(stderr)
                .with_duration_ms(duration_ms))
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use core::types::task_input::TaskInput;

        fn create_test_task(command: &str, args: Vec<String>, cwd: &str) -> Task {
            Task::new(
                "P2-008",
                "Implement RustRunner",
                core::types::task::TaskCategory::Schema,
                "test-fixture",
                "Implement RustRunner with actual binary invocation",
                "RustRunner executes binaries correctly",
                vec![],
                core::types::entry_mode::EntryMode::CLI,
                core::types::agent_mode::AgentMode::OneShot,
                core::types::provider_mode::ProviderMode::Both,
                TaskInput::new(command, args, cwd),
                vec![],
                core::types::severity::Severity::High,
                core::types::execution_policy::ExecutionPolicy::ManualCheck,
                60,
                core::types::on_missing_dependency::OnMissingDependency::Fail,
            )
        }

        #[test]
        fn test_rust_runner_creation() {
            let runner = RustRunner::new("opencode");
            assert_eq!(runner.name(), "opencode");
        }

        #[test]
        fn test_rust_runner_execute_echo() {
            let runner = RustRunner::new("opencode");
            let task = create_test_task("echo", vec!["hello".to_string()], "/tmp");

            let result = runner.execute(&task).unwrap();
            assert_eq!(result.task_id, "P2-008");
            assert_eq!(result.status, TaskStatus::Done);
            assert_eq!(result.exit_code, Some(0));
            assert!(result.stdout.contains("hello"));
        }

        #[test]
        fn test_rust_runner_execute_with_stderr() {
            let runner = RustRunner::new("opencode");
            let task = create_test_task(
                "sh",
                vec!["-c".to_string(), "echo error 1>&2".to_string()],
                "/tmp",
            );

            let result = runner.execute(&task).unwrap();
            assert_eq!(result.task_id, "P2-008");
            assert!(result.stderr.contains("error"));
        }

        #[test]
        fn test_rust_runner_execute_nonexistent_command() {
            let runner = RustRunner::new("opencode");
            let task = create_test_task("nonexistent_command_xyz", vec![], "/tmp");

            let result = runner.execute(&task);
            assert!(result.is_err());
        }

        #[test]
        fn test_rust_runner_execute_with_args() {
            let runner = RustRunner::new("opencode");
            let task = create_test_task(
                "printf",
                vec!["%s %d\n".to_string(), "test".to_string(), "42".to_string()],
                "/tmp",
            );

            let result = runner.execute(&task).unwrap();
            assert!(result.stdout.contains("test"));
            assert!(result.stdout.contains("42"));
        }

        #[test]
        fn test_rust_runner_trait_impl() {
            fn assert_runner<T: Runner>() {}
            assert_runner::<RustRunner>();
        }

        #[test]
        fn test_rust_runner_is_send_and_sync() {
            fn assert_send_sync<T: Send + Sync>() {}
            assert_send_sync::<RustRunner>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::error::Result;
    use core::types::task::Task;
    use core::types::task_input::TaskInput;

    fn create_test_task() -> Task {
        Task::new(
            "test",
            "Test Task",
            core::types::task::TaskCategory::Core,
            "fixture",
            "desc",
            "outcome",
            vec![],
            core::types::entry_mode::EntryMode::CLI,
            core::types::agent_mode::AgentMode::OneShot,
            core::types::provider_mode::ProviderMode::Both,
            TaskInput::new("echo", vec!["test".to_string()], "/tmp"),
            vec![],
            core::types::severity::Severity::High,
            core::types::execution_policy::ExecutionPolicy::ManualCheck,
            60,
            core::types::on_missing_dependency::OnMissingDependency::Fail,
        )
    }

    #[test]
    fn test_runner_trait_defines_execute_method() {
        fn assert_exec_signature<R: Runner>(_: &R) {
            let task = create_test_task();
            let _ = task;
        }
        assert_exec_signature(&LegacyRunner::new("test"));
        assert_exec_signature(&RustRunner::new("test"));
    }

    #[test]
    fn test_legacy_runner_and_rust_runner_different_names() {
        let legacy = LegacyRunner::new("opencode-rs");
        let rust = RustRunner::new("opencode");

        assert_eq!(legacy.name(), "opencode-rs");
        assert_eq!(rust.name(), "opencode");
    }

    #[test]
    fn test_both_runners_implement_runner_trait() {
        fn execute_task<R: Runner>(runner: &R, task: &Task) -> Result<ExecutionResult> {
            runner.execute(task)
        }

        let legacy = LegacyRunner::new("opencode-rs");
        let rust = RustRunner::new("opencode");
        let task = create_test_task();

        let legacy_result = execute_task(&legacy, &task).unwrap();
        let rust_result = execute_task(&rust, &task).unwrap();

        assert!(legacy_result.is_success());
        assert!(rust_result.is_success());
    }
}
