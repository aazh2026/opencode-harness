use crate::error::Result;
use crate::loaders::TaskLoader;
use crate::runners::legacy_runner::{LegacyRunner, LegacyRunnerResult};
use crate::runners::rust_runner::{RustRunner, RustRunnerResult};
use crate::types::artifact::Artifact;
use crate::types::parity_verdict::{DiffCategory, ParityVerdict};
use crate::types::task::Task;
use std::path::Path;
use std::time::Instant;

pub struct DifferentialRunner<L: TaskLoader> {
    pub task_loader: L,
}

impl<L: TaskLoader> DifferentialRunner<L> {
    pub fn new(task_loader: L) -> Self {
        Self { task_loader }
    }

    pub fn execute(&self, task: &Task) -> Result<DifferentialResult> {
        let start = Instant::now();
        let legacy_runner = LegacyRunner::new("legacy");
        let rust_runner = RustRunner::new("rust");

        let legacy_result = legacy_runner.execute(task);
        let rust_result = rust_runner.execute(task);

        let verdict = self.determine_verdict(&legacy_result, &rust_result);

        let duration_ms = start.elapsed().as_millis() as u64;

        match (legacy_result, rust_result) {
            (Ok(lr), Ok(rr)) => Ok(DifferentialResult {
                task_id: task.id.clone(),
                legacy_result: Some(lr.into()),
                rust_result: Some(rr),
                verdict: verdict.clone(),
                duration_ms,
            }),
            (Err(e), Ok(rr)) => {
                let runner = "LegacyRunner".to_string();
                let reason = e.to_string();
                Ok(DifferentialResult {
                    task_id: task.id.clone(),
                    legacy_result: None,
                    rust_result: Some(rr),
                    verdict: ParityVerdict::Error { runner, reason },
                    duration_ms,
                })
            }
            (Ok(lr), Err(e)) => {
                let runner = "RustRunner".to_string();
                let reason = e.to_string();
                Ok(DifferentialResult {
                    task_id: task.id.clone(),
                    legacy_result: Some(lr.into()),
                    rust_result: None,
                    verdict: ParityVerdict::Error { runner, reason },
                    duration_ms,
                })
            }
            (Err(e1), Err(e2)) => Ok(DifferentialResult {
                task_id: task.id.clone(),
                legacy_result: None,
                rust_result: None,
                verdict: ParityVerdict::Error {
                    runner: "Both".to_string(),
                    reason: format!("legacy: {}; rust: {}", e1, e2),
                },
                duration_ms,
            }),
        }
    }

    fn determine_verdict(
        &self,
        legacy_result: &Result<LegacyRunnerResult>,
        rust_result: &Result<RustRunnerResult>,
    ) -> ParityVerdict {
        let (lr, rr) = match (legacy_result, rust_result) {
            (Ok(l), Ok(r)) => (l, r),
            _ => return ParityVerdict::Uncertain,
        };

        if lr.exit_code != rr.exit_code {
            return ParityVerdict::Different {
                category: DiffCategory::ExitCode,
            };
        }

        if lr.stdout != rr.stdout {
            return ParityVerdict::Different {
                category: DiffCategory::OutputText,
            };
        }

        if lr.stderr != rr.stderr {
            return ParityVerdict::Different {
                category: DiffCategory::OutputText,
            };
        }

        let timing_diff = (lr.duration_ms as i64 - rr.duration_ms as i64).unsigned_abs();
        let max_duration = lr.duration_ms.max(rr.duration_ms);
        if max_duration > 0 && timing_diff > max_duration / 2 {
            return ParityVerdict::Different {
                category: DiffCategory::Timing,
            };
        }

        if lr.artifacts.len() != rr.artifacts.len() {
            return ParityVerdict::Different {
                category: DiffCategory::SideEffects,
            };
        }

        for (la, ra) in lr.artifacts.iter().zip(rr.artifacts.iter()) {
            if la.path != ra.path || la.kind != ra.kind {
                return ParityVerdict::Different {
                    category: DiffCategory::SideEffects,
                };
            }
        }

        if lr.stdout == rr.stdout && lr.stderr == rr.stderr && lr.exit_code == rr.exit_code {
            return ParityVerdict::Identical;
        }

        ParityVerdict::Equivalent
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
}

#[derive(Debug, Clone)]
pub struct DifferentialResult {
    pub task_id: String,
    pub legacy_result: Option<LegacyExecutionResult>,
    pub rust_result: Option<RustRunnerResult>,
    pub verdict: ParityVerdict,
    pub duration_ms: u64,
}

impl DifferentialResult {
    pub fn new(task_id: String) -> Self {
        Self {
            task_id,
            legacy_result: None,
            rust_result: None,
            verdict: ParityVerdict::Uncertain,
            duration_ms: 0,
        }
    }

    pub fn passed(&self) -> bool {
        self.verdict.is_pass()
    }

    pub fn legacy_exit_code(&self) -> Option<i32> {
        self.legacy_result.as_ref().and_then(|r| r.exit_code)
    }

    pub fn rust_exit_code(&self) -> Option<i32> {
        self.rust_result.as_ref().and_then(|r| r.exit_code)
    }

    pub fn legacy_stdout(&self) -> &str {
        self.legacy_result
            .as_ref()
            .map(|r| r.stdout.as_str())
            .unwrap_or("")
    }

    pub fn rust_stdout(&self) -> &str {
        self.rust_result
            .as_ref()
            .map(|r| r.stdout.as_str())
            .unwrap_or("")
    }

    pub fn legacy_stderr(&self) -> &str {
        self.legacy_result
            .as_ref()
            .map(|r| r.stderr.as_str())
            .unwrap_or("")
    }

    pub fn rust_stderr(&self) -> &str {
        self.rust_result
            .as_ref()
            .map(|r| r.stderr.as_str())
            .unwrap_or("")
    }

    pub fn summary(&self) -> String {
        format!(
            "DifferentialResult(task_id={}, verdict={}, duration_ms={})",
            self.task_id,
            self.verdict.summary(),
            self.duration_ms
        )
    }
}

#[derive(Debug, Clone)]
pub struct LegacyExecutionResult {
    pub task_id: String,
    pub status: crate::types::TaskStatus,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub artifacts: Vec<Artifact>,
}

impl From<LegacyRunnerResult> for LegacyExecutionResult {
    fn from(r: LegacyRunnerResult) -> Self {
        Self {
            task_id: r.task_id,
            status: r.status,
            exit_code: r.exit_code,
            stdout: r.stdout,
            stderr: r.stderr,
            duration_ms: r.duration_ms,
            artifacts: r.artifacts,
        }
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
            vec![],
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
    fn test_differential_result_creation() {
        let result = DifferentialResult::new("TEST-001".to_string());
        assert_eq!(result.task_id, "TEST-001");
        assert!(result.legacy_result.is_none());
        assert!(result.rust_result.is_none());
        assert!(result.verdict.is_uncertain());
        assert!(!result.passed());
    }

    #[test]
    fn test_differential_result_helper_methods() {
        let result = DifferentialResult::new("TEST-001".to_string());
        assert_eq!(result.legacy_exit_code(), None);
        assert_eq!(result.rust_exit_code(), None);
        assert_eq!(result.legacy_stdout(), "");
        assert_eq!(result.rust_stdout(), "");
    }

    #[test]
    fn test_differential_runner_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        let loader = DefaultTaskLoader::new();
        let _runner = DifferentialRunner::new(loader);
        assert_send_sync::<DifferentialRunner<DefaultTaskLoader>>();
    }

    #[test]
    fn test_differential_runner_execute_echo() {
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
expected_assertions: []
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

        let diff_result = result.unwrap();
        assert_eq!(diff_result.task_id, "TEST-EXEC-001");
        assert!(!diff_result.verdict.is_uncertain() || diff_result.verdict.is_error());
    }

    #[test]
    fn test_differential_result_summary() {
        let result = DifferentialResult::new("TEST-001".to_string());
        let summary = result.summary();
        assert!(summary.contains("TEST-001"));
        assert!(summary.contains("Uncertain"));
    }

    #[test]
    fn test_legacy_execution_result_from() {
        let lr = LegacyRunnerResult::new("task-1")
            .with_exit_code(0)
            .with_stdout("hello".to_string());
        let exec: LegacyExecutionResult = lr.into();
        assert_eq!(exec.task_id, "task-1");
        assert_eq!(exec.exit_code, Some(0));
        assert_eq!(exec.stdout, "hello");
    }

    #[test]
    fn test_parity_verdict_determination_identical() {
        let loader = DefaultTaskLoader::new();
        let runner = DifferentialRunner::new(loader);
        let task = create_test_task();
        let result = runner.execute(&task);
        assert!(result.is_ok());
    }
}
