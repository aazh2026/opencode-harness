use opencode_core::types::agent_mode::AgentMode;
use opencode_core::types::allowed_variance::{AllowedVariance, TimingVariance};
use opencode_core::types::assertion::AssertionType;
use opencode_core::types::entry_mode::EntryMode;
use opencode_core::types::environment::{
    DefaultEnvironmentProbe, EnvironmentInfo, EnvironmentProbe,
};
use opencode_core::types::execution_policy::ExecutionPolicy;
use opencode_core::types::failure_classification::FailureClassification;
use opencode_core::types::fixture::{
    ConfigFormat, FixtureConfig, FixtureFile, FixtureProject, ResetStrategy, Transcript,
    TranscriptType, Workspace as FixtureWorkspace, WorkspacePolicy,
};
use opencode_core::types::on_missing_dependency::OnMissingDependency;
use opencode_core::types::path_convention::PathConvention;
use opencode_core::types::provider_mode::ProviderMode;
use opencode_core::types::report::{Report, TestCase, TestCaseStatus};
use opencode_core::types::severity::Severity;
use opencode_core::types::task::{Task, TaskCategory};
use opencode_core::types::task_input::TaskInput;
use opencode_core::types::task_outputs::TaskOutputs;
use opencode_core::types::task_status::TaskStatus;
use opencode_core::types::workspace::Workspace;

#[test]
fn test_types_module_exports_agent_mode() {
    let _ = AgentMode::Interactive;
    let _ = AgentMode::Batch;
    let _ = AgentMode::Daemon;
    let _ = AgentMode::OneShot;
}

#[test]
fn test_types_module_exports_allowed_variance() {
    let variance = AllowedVariance::new(
        vec![0, 1],
        Some(TimingVariance::new(Some(100), Some(500))),
        vec![],
    );
    assert_eq!(variance.exit_code, vec![0, 1]);
    let timing = variance.timing_ms.unwrap();
    assert_eq!(timing.min, Some(100));
    assert_eq!(timing.max, Some(500));
}

#[test]
fn test_types_module_exports_assertion_type() {
    let _ = AssertionType::ExitCodeEquals(0);
    let _ = AssertionType::StdoutContains("test".to_string());
    let _ = AssertionType::StderrContains("error".to_string());
    let _ = AssertionType::FileChanged("file.rs".to_string());
    let _ = AssertionType::NoExtraFilesChanged;
    let _ = AssertionType::PermissionPromptSeen("prompt?".to_string());
}

#[test]
fn test_types_module_exports_entry_mode() {
    let _ = EntryMode::CLI;
    let _ = EntryMode::API;
    let _ = EntryMode::Session;
    let _ = EntryMode::Permissions;
    let _ = EntryMode::Web;
    let _ = EntryMode::Workspace;
    let _ = EntryMode::Recovery;
}

#[test]
fn test_types_module_exports_environment_probe() {
    let probe: Box<dyn EnvironmentProbe> = Box::new(DefaultEnvironmentProbe::default());
    assert!(!probe.check_binary("this_binary_does_not_exist_12345"));
    let info = probe.get_info();
    assert!(!info.os.is_empty());
}

#[test]
fn test_types_module_exports_environment_info() {
    let info = EnvironmentInfo {
        os: "linux".to_string(),
        arch: "x86_64".to_string(),
        rustc_version: Some("rustc 1.0".to_string()),
        available_binaries: vec!["cargo".to_string()],
    };
    assert_eq!(info.os, "linux");
    assert_eq!(info.arch, "x86_64");
}

#[test]
fn test_types_module_exports_execution_policy() {
    let _ = ExecutionPolicy::ManualCheck;
    let _ = ExecutionPolicy::Blocked;
    let _ = ExecutionPolicy::Skip;
}

#[test]
fn test_types_module_exports_failure_classification() {
    let _ = FailureClassification::ImplementationFailure;
    let _ = FailureClassification::DependencyMissing;
    let _ = FailureClassification::EnvironmentNotSupported;
    let _ = FailureClassification::InfraFailure;
    let _ = FailureClassification::FlakySuspected;
}

#[test]
fn test_types_module_exports_fixture_project() {
    let policy = WorkspacePolicy {
        allow_dirty_git: false,
        allow_network: false,
        preserve_on_failure: false,
    };
    let fixture = FixtureProject::new(
        "test".to_string(),
        "Test fixture".to_string(),
        policy,
        ResetStrategy::None,
    );
    assert_eq!(fixture.name, "test");
}

#[test]
fn test_types_module_exports_workspace_policy() {
    let policy = WorkspacePolicy {
        allow_dirty_git: true,
        allow_network: true,
        preserve_on_failure: true,
    };
    assert!(policy.allow_dirty_git);
    assert!(policy.allow_network);
    assert!(policy.preserve_on_failure);
}

#[test]
fn test_types_module_exports_reset_strategy() {
    let _ = ResetStrategy::None;
    let _ = ResetStrategy::CleanClone;
    let _ = ResetStrategy::RestoreFiles;
}

#[test]
fn test_types_module_exports_fixture_file() {
    let file = FixtureFile {
        path: "src/main.rs".to_string(),
        content: "fn main() {}".to_string(),
        executable: true,
    };
    assert_eq!(file.path, "src/main.rs");
    assert!(file.executable);
}

#[test]
fn test_types_module_exports_fixture_config() {
    let config = FixtureConfig {
        path: "config.toml".to_string(),
        format: ConfigFormat::Toml,
    };
    assert_eq!(config.path, "config.toml");
    assert_eq!(config.format, ConfigFormat::Toml);
}

#[test]
fn test_types_module_exports_config_format() {
    let _ = ConfigFormat::Toml;
    let _ = ConfigFormat::Json;
    let _ = ConfigFormat::Yaml;
    let _ = ConfigFormat::Env;
}

#[test]
fn test_types_module_exports_transcript() {
    let transcript = Transcript {
        path: "recording.json".to_string(),
        transcript_type: TranscriptType::Recording,
    };
    assert_eq!(transcript.path, "recording.json");
    assert_eq!(transcript.transcript_type, TranscriptType::Recording);
}

#[test]
fn test_types_module_exports_transcript_type() {
    let _ = TranscriptType::Recording;
    let _ = TranscriptType::Transcript;
    let _ = TranscriptType::Snapshot;
}

#[test]
fn test_types_module_exports_on_missing_dependency() {
    let _ = OnMissingDependency::Fail;
    let _ = OnMissingDependency::Skip;
    let _ = OnMissingDependency::Warn;
    let _ = OnMissingDependency::Blocked;
}

#[test]
fn test_types_module_exports_path_convention() {
    assert_eq!(PathConvention::RUN_ARTIFACTS, "artifacts/run-{id}");
    assert_eq!(PathConvention::SESSION_DATA, "sessions/iteration-{n}");
    assert_eq!(
        PathConvention::REPORTS,
        "harness/reports/{suite}/{timestamp}"
    );
    assert_eq!(PathConvention::TASKS, "tasks");
    assert_eq!(PathConvention::FIXTURES, "fixtures/projects");
}

#[test]
fn test_types_module_exports_provider_mode() {
    let _ = ProviderMode::OpenCode;
    let _ = ProviderMode::OpenCodeRS;
    let _ = ProviderMode::Both;
    let _ = ProviderMode::Either;
}

#[test]
fn test_types_module_exports_report() {
    let report = Report {
        timestamp: "2026-04-16".to_string(),
        suite: "test".to_string(),
        total: 10,
        passed: 8,
        failed: 1,
        skipped: 1,
        mismatches: 0,
    };
    assert_eq!(report.total, 10);
    assert_eq!(report.passed, 8);
}

#[test]
fn test_types_module_exports_test_case() {
    let testcase = TestCase {
        id: "test-001".to_string(),
        status: TestCaseStatus::Passed,
        duration: 100,
        failure_classification: FailureClassification::ImplementationFailure,
        error_message: None,
    };
    assert_eq!(testcase.id, "test-001");
    assert_eq!(testcase.status, TestCaseStatus::Passed);
}

#[test]
fn test_types_module_exports_test_case_status() {
    let _ = TestCaseStatus::Passed;
    let _ = TestCaseStatus::Failed;
    let _ = TestCaseStatus::Skipped;
}

#[test]
fn test_types_module_exports_severity() {
    let _ = Severity::Critical;
    let _ = Severity::High;
    let _ = Severity::Medium;
    let _ = Severity::Low;
    let _ = Severity::Cosmetic;
}

#[test]
fn test_types_module_exports_task() {
    let task = Task::new(
        "TEST-001",
        "Test Task",
        TaskCategory::Smoke,
        "fixtures/projects/test",
        "Test description",
        "Test outcome",
        vec!["opencode exists".to_string()],
        EntryMode::CLI,
        AgentMode::OneShot,
        ProviderMode::Both,
        TaskInput::new("opencode", vec![], "/project"),
        vec![AssertionType::ExitCodeEquals(0)],
        Severity::High,
        ExecutionPolicy::ManualCheck,
        300,
        OnMissingDependency::Fail,
    );
    assert_eq!(task.id, "TEST-001");
    assert_eq!(task.category, TaskCategory::Smoke);
}

#[test]
fn test_types_module_exports_task_category() {
    let _ = TaskCategory::Core;
    let _ = TaskCategory::Schema;
    let _ = TaskCategory::Integration;
    let _ = TaskCategory::Regression;
    let _ = TaskCategory::Smoke;
    let _ = TaskCategory::Performance;
    let _ = TaskCategory::Security;
}

#[test]
fn test_types_module_exports_task_input() {
    let input = TaskInput::new("opencode", vec!["--help".to_string()], "/project");
    assert_eq!(input.command, "opencode");
    assert_eq!(input.args, vec!["--help"]);
    assert_eq!(input.cwd, "/project");
}

#[test]
fn test_types_module_exports_task_outputs() {
    let outputs = TaskOutputs::new(
        "stdout".to_string(),
        "stderr".to_string(),
        vec!["file1.txt".to_string()],
        vec!["file2.rs".to_string()],
    );
    assert_eq!(outputs.stdout, "stdout");
    assert_eq!(outputs.stderr, "stderr");
    assert_eq!(outputs.files_created.len(), 1);
}

#[test]
fn test_types_module_exports_task_status() {
    let _ = TaskStatus::Todo;
    let _ = TaskStatus::InProgress;
    let _ = TaskStatus::Done;
    let _ = TaskStatus::ManualCheck;
    let _ = TaskStatus::Blocked;
    let _ = TaskStatus::Skipped;
}

#[test]
fn test_types_module_exports_workspace() {
    let workspace = Workspace::new(
        "ws-001".to_string(),
        std::path::PathBuf::from("/tmp/workspace"),
        "test-fixture".to_string(),
    );
    assert_eq!(workspace.id, "ws-001");
    assert_eq!(workspace.fixture_name, "test-fixture");
}

#[test]
fn test_types_module_exports_fixture_workspace() {
    let workspace = FixtureWorkspace::new(
        "ws-002".to_string(),
        std::path::PathBuf::from("/tmp/workspace2"),
        "test-fixture2".to_string(),
    );
    assert_eq!(workspace.id, "ws-002");
    assert_eq!(workspace.fixture_name, "test-fixture2");
}

use opencode_core::loaders::{
    DefaultFixtureLoader, DefaultTaskLoader, DefaultTaskSchemaValidator, FixtureLoader, TaskLoader,
    TaskSchemaValidator,
};

#[test]
fn test_loaders_module_exports_task_loader_trait() {
    fn assert_task_loader<T: TaskLoader>() {}
    assert_task_loader::<DefaultTaskLoader>();
}

#[test]
fn test_loaders_module_exports_default_task_loader() {
    let loader = DefaultTaskLoader::new();
    let _loaded: Box<dyn TaskLoader> = Box::new(loader);
}

#[test]
fn test_loaders_module_exports_task_schema_validator_trait() {
    fn assert_task_validator<T: TaskSchemaValidator>() {}
    assert_task_validator::<DefaultTaskSchemaValidator>();
}

#[test]
fn test_loaders_module_exports_default_task_schema_validator() {
    let validator = DefaultTaskSchemaValidator::new();
    let _validated: Box<dyn TaskSchemaValidator> = Box::new(validator);
}

#[test]
fn test_loaders_module_exports_fixture_loader_trait() {
    fn assert_fixture_loader<T: FixtureLoader>() {}
    assert_fixture_loader::<DefaultFixtureLoader>();
}

#[test]
fn test_loaders_module_exports_default_fixture_loader() {
    let loader = DefaultFixtureLoader::new(std::path::PathBuf::from("/tmp/fixtures"));
    let _loaded: Box<dyn FixtureLoader> = Box::new(loader);
}

#[test]
fn test_module_exports_compile_correctly() {
    use opencode_core::loaders;
    use opencode_core::types;

    fn _check_types_public() {
        let _ = types::agent_mode::AgentMode::OneShot;
        let _ = types::allowed_variance::AllowedVariance::new(vec![], None, vec![]);
        let _ = types::assertion::AssertionType::ExitCodeEquals(0);
        let _ = types::entry_mode::EntryMode::CLI;
        let _ = types::execution_policy::ExecutionPolicy::ManualCheck;
        let _ = types::failure_classification::FailureClassification::ImplementationFailure;
        let _ = types::fixture::FixtureProject::new(
            "".to_string(),
            "".to_string(),
            types::fixture::WorkspacePolicy {
                allow_dirty_git: false,
                allow_network: false,
                preserve_on_failure: false,
            },
            types::fixture::ResetStrategy::None,
        );
        let _ = types::on_missing_dependency::OnMissingDependency::Fail;
        let _ = types::provider_mode::ProviderMode::Both;
        let _ = types::report::Report {
            timestamp: "".to_string(),
            suite: "".to_string(),
            total: 0,
            passed: 0,
            failed: 0,
            skipped: 0,
            mismatches: 0,
        };
        let _ = types::severity::Severity::High;
        let _ = types::task::Task::new(
            "TEST-001",
            "Test",
            types::task::TaskCategory::Smoke,
            "fixtures/test",
            "desc",
            "outcome",
            vec![],
            types::entry_mode::EntryMode::CLI,
            types::agent_mode::AgentMode::OneShot,
            types::provider_mode::ProviderMode::Both,
            types::task_input::TaskInput::new("cmd", vec![], "/"),
            vec![],
            types::severity::Severity::Medium,
            types::execution_policy::ExecutionPolicy::ManualCheck,
            300,
            types::on_missing_dependency::OnMissingDependency::Fail,
        );
        let _ = types::task_input::TaskInput::new("cmd", vec![], "/");
        let _ =
            types::task_outputs::TaskOutputs::new("".to_string(), "".to_string(), vec![], vec![]);
        let _ = types::task_status::TaskStatus::Todo;
        let _ = types::workspace::Workspace::new(
            "".to_string(),
            std::path::PathBuf::new(),
            "".to_string(),
        );
    }

    fn _check_loaders_public() {
        let _task_loader: Box<dyn loaders::TaskLoader> =
            Box::new(loaders::DefaultTaskLoader::new());
        let _validator: Box<dyn loaders::TaskSchemaValidator> =
            Box::new(loaders::DefaultTaskSchemaValidator::new());
        let _fixture_loader: Box<dyn loaders::FixtureLoader> =
            Box::new(loaders::DefaultFixtureLoader::new(std::path::PathBuf::new()));
    }
}

use opencode_core::runners::{
    ArtifactDiff, ArtifactPersister, BinaryResolver, DiffReport, DifferentialResult,
    DifferentialRunner, LegacyRunner, LegacyRunnerResult, MetadataJson, RunnerType, RustRunner,
    RustRunnerResult,
};

#[test]
fn test_runners_module_exports_artifact_persister() {
    let persister = ArtifactPersister::new("test-run", std::path::PathBuf::from("/tmp/artifacts"));
    assert_eq!(persister.run_id(), "test-run");
    let _ = ArtifactDiff {
        only_in_legacy: vec![],
        only_in_rust: vec![],
        in_both: vec![],
    };
    let _ = MetadataJson {
        session_id: "session-1".to_string(),
        runner_name: "LegacyRunner".to_string(),
        task_id: "TASK-001".to_string(),
        exit_code: Some(0),
        duration_ms: 100,
        artifacts_collected: 0,
        stdout_size_bytes: 10,
        stderr_size_bytes: 5,
        side_effects_count: 0,
        created_at: chrono::Utc::now(),
    };
    assert_eq!(RunnerType::Legacy, RunnerType::Legacy);
    assert_eq!(RunnerType::Rust, RunnerType::Rust);
}

#[test]
fn test_runners_module_exports_binary_resolver() {
    let resolver = BinaryResolver::new();
    let _ = resolver;
}

#[test]
fn test_runners_module_exports_differential_runner() {
    use opencode_core::loaders::DefaultTaskLoader;
    let loader = DefaultTaskLoader::new();
    let runner = DifferentialRunner::new(loader);
    let _ = runner;
    let _ = DifferentialResult::new("TEST-001".to_string());
}

#[test]
fn test_runners_module_exports_legacy_runner() {
    let runner = LegacyRunner::new("legacy");
    let _ = runner;
    let _ = LegacyRunnerResult::new("task-1");
}

#[test]
fn test_runners_module_exports_rust_runner() {
    let runner = RustRunner::new("rust");
    let _ = runner;
    let _ = RustRunnerResult::new("task-1");
}

#[test]
fn test_runners_all_exports_accessible() {
    fn _check_artifact_persister_public() {
        let _persister: ArtifactPersister =
            ArtifactPersister::new("run", std::path::PathBuf::from("/tmp"));
    }
    fn _check_runner_type_public() {
        let _rt: RunnerType = RunnerType::Legacy;
    }
    fn _check_diff_report_public() {
        let _report: DiffReport = DiffReport {
            run_id: "run-1".to_string(),
            legacy_exit_code: Some(0),
            rust_exit_code: Some(0),
            legacy_duration_ms: 100,
            rust_duration_ms: 100,
            verdict: "Identical".to_string(),
            verdict_category: None,
            legacy_stdout_size: 10,
            rust_stdout_size: 10,
            legacy_stderr_size: 5,
            rust_stderr_size: 5,
            artifacts_diff: ArtifactDiff {
                only_in_legacy: vec![],
                only_in_rust: vec![],
                in_both: vec![],
            },
            generated_at: chrono::Utc::now(),
        };
    }
    fn _check_metadata_json_public() {
        let _meta: MetadataJson = MetadataJson {
            session_id: "session-1".to_string(),
            runner_name: "LegacyRunner".to_string(),
            task_id: "TASK-001".to_string(),
            exit_code: Some(0),
            duration_ms: 100,
            artifacts_collected: 0,
            stdout_size_bytes: 10,
            stderr_size_bytes: 5,
            side_effects_count: 0,
            created_at: chrono::Utc::now(),
        };
    }
}
