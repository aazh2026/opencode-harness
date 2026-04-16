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
    use core::error::Result;
    use core::types::task::Task;
    use core::types::task_status::TaskStatus;

    pub struct LegacyRunner {
        name: String,
    }

    impl LegacyRunner {
        pub fn new(name: impl Into<String>) -> Self {
            Self { name: name.into() }
        }
    }

    impl Runner for LegacyRunner {
        fn execute(&self, task: &Task) -> Result<ExecutionResult> {
            Ok(ExecutionResult::new(&task.id)
                .with_status(TaskStatus::Done)
                .with_exit_code(0)
                .with_stdout(format!(
                    "LegacyRunner '{}' executed task {}",
                    self.name, task.id
                ))
                .with_duration_ms(0))
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_legacy_runner_creation() {
            let runner = LegacyRunner::new("opencode-rs");
            assert_eq!(runner.name(), "opencode-rs");
        }

        #[test]
        fn test_legacy_runner_execute() {
            let runner = LegacyRunner::new("opencode-rs");
            let task = Task::new(
                "P2-003",
                "Define Runner Traits",
                core::types::task::TaskCategory::Schema,
                "test-fixture",
                "Create runner traits",
                "Traits defined correctly",
            );

            let result = runner.execute(&task).unwrap();
            assert_eq!(result.task_id, "P2-003");
            assert_eq!(result.status, TaskStatus::Done);
            assert_eq!(result.exit_code, Some(0));
            assert!(result.stdout.contains("LegacyRunner"));
            assert!(result.stdout.contains("P2-003"));
        }

        #[test]
        fn test_legacy_runner_trait_impl() {
            fn assert_runner<T: Runner>() {}
            assert_runner::<LegacyRunner>();
        }
    }
}

mod rust_runner {
    use super::{ExecutionResult, Runner};
    use core::error::Result;
    use core::types::task::Task;
    use core::types::task_status::TaskStatus;

    pub struct RustRunner {
        name: String,
    }

    impl RustRunner {
        pub fn new(name: impl Into<String>) -> Self {
            Self { name: name.into() }
        }
    }

    impl Runner for RustRunner {
        fn execute(&self, task: &Task) -> Result<ExecutionResult> {
            Ok(ExecutionResult::new(&task.id)
                .with_status(TaskStatus::Done)
                .with_exit_code(0)
                .with_stdout(format!(
                    "RustRunner '{}' executed task {}",
                    self.name, task.id
                ))
                .with_duration_ms(0))
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_rust_runner_creation() {
            let runner = RustRunner::new("opencode");
            assert_eq!(runner.name(), "opencode");
        }

        #[test]
        fn test_rust_runner_execute() {
            let runner = RustRunner::new("opencode");
            let task = Task::new(
                "P2-003",
                "Define Runner Traits",
                core::types::task::TaskCategory::Schema,
                "test-fixture",
                "Create runner traits",
                "Traits defined correctly",
            );

            let result = runner.execute(&task).unwrap();
            assert_eq!(result.task_id, "P2-003");
            assert_eq!(result.status, TaskStatus::Done);
            assert_eq!(result.exit_code, Some(0));
            assert!(result.stdout.contains("RustRunner"));
            assert!(result.stdout.contains("P2-003"));
        }

        #[test]
        fn test_rust_runner_trait_impl() {
            fn assert_runner<T: Runner>() {}
            assert_runner::<RustRunner>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::error::Result;
    use core::types::task::Task;

    #[test]
    fn test_runner_trait_defines_execute_method() {
        fn assert_exec_signature<R: Runner>(_: &R) {
            let task = Task::new(
                "test",
                "Test Task",
                core::types::task::TaskCategory::Core,
                "fixture",
                "desc",
                "outcome",
            );
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
        let task = Task::new(
            "P2-003",
            "Define Runner Traits",
            core::types::task::TaskCategory::Schema,
            "test-fixture",
            "Create runner traits",
            "Traits defined correctly",
        );

        let legacy_result = execute_task(&legacy, &task).unwrap();
        let rust_result = execute_task(&rust, &task).unwrap();

        assert!(legacy_result.is_success());
        assert!(rust_result.is_success());
    }
}
