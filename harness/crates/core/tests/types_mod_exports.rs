use opencode_core::types::{
    AgentMode, AllowedVariance, AssertionType, ConfigFormat, DefaultEnvironmentProbe, EntryMode,
    EnvironmentInfo, EnvironmentProbe, ExecutionPolicy, FailureClassification, FixtureConfig,
    FixtureFile, FixtureProject, FixtureWorkspace, OnMissingDependency, PathConvention,
    ProviderMode, SimpleReport, ResetStrategy, Severity, Task, TaskCategory, TaskInput, TaskOutputs,
    TaskStatus, TestCase, TestCaseStatus, TimingVariance, Transcript, TranscriptType, Workspace,
    WorkspacePolicy,
};

#[test]
fn test_all_p0_type_exports_are_available() {
    let _ = AgentMode::Interactive;
    let _ = AgentMode::Batch;
    let _ = AgentMode::Daemon;
    let _ = AgentMode::OneShot;

    let _ = AllowedVariance::new(vec![0], None, vec![]);
    let _ = TimingVariance::new(Some(0), Some(1000));

    let _ = AssertionType::ExitCodeEquals(0);
    let _ = AssertionType::StdoutContains("test".to_string());
    let _ = AssertionType::StderrContains("error".to_string());
    let _ = AssertionType::FileChanged("file.rs".to_string());
    let _ = AssertionType::NoExtraFilesChanged;
    let _ = AssertionType::PermissionPromptSeen("prompt?".to_string());

    let _ = ConfigFormat::Toml;
    let _ = ConfigFormat::Json;
    let _ = ConfigFormat::Yaml;
    let _ = ConfigFormat::Env;

    let _probe: Box<dyn EnvironmentProbe> = Box::new(DefaultEnvironmentProbe::default());
    let _ = _probe.check_binary("ls");
    let _info = EnvironmentInfo {
        os: "linux".to_string(),
        arch: "x86_64".to_string(),
        rustc_version: Some("rustc 1.0".to_string()),
        available_binaries: vec!["cargo".to_string()],
    };

    let _ = EntryMode::CLI;
    let _ = EntryMode::API;
    let _ = EntryMode::Session;
    let _ = EntryMode::Permissions;
    let _ = EntryMode::Web;
    let _ = EntryMode::Workspace;
    let _ = EntryMode::Recovery;

    let _ = ExecutionPolicy::ManualCheck;
    let _ = ExecutionPolicy::Blocked;
    let _ = ExecutionPolicy::Skip;

    let _ = FailureClassification::ImplementationFailure;
    let _ = FailureClassification::DependencyMissing;
    let _ = FailureClassification::EnvironmentNotSupported;
    let _ = FailureClassification::InfraFailure;
    let _ = FailureClassification::FlakySuspected;

    let policy = WorkspacePolicy {
        allow_dirty_git: false,
        allow_network: false,
        preserve_on_failure: false,
    };
    let _fixture = FixtureProject::new(
        "test".to_string(),
        "Test fixture".to_string(),
        policy,
        ResetStrategy::None,
    );
    let _file = FixtureFile {
        path: "src/main.rs".to_string(),
        content: "fn main() {}".to_string(),
        executable: true,
    };
    let _config = FixtureConfig {
        path: "config.toml".to_string(),
        format: ConfigFormat::Toml,
    };
    let _transcript = Transcript {
        path: "recording.json".to_string(),
        transcript_type: TranscriptType::Recording,
    };
    let _workspace = FixtureWorkspace::new(
        "ws-001".to_string(),
        std::path::PathBuf::from("/tmp/workspace"),
        "test-fixture".to_string(),
    );

    let _ = OnMissingDependency::Fail;
    let _ = OnMissingDependency::Skip;
    let _ = OnMissingDependency::Warn;
    let _ = OnMissingDependency::Blocked;

    assert_eq!(PathConvention::RUN_ARTIFACTS, "artifacts/run-{id}");
    assert_eq!(PathConvention::SESSION_DATA, "sessions/iteration-{n}");
    assert_eq!(
        PathConvention::REPORTS,
        "harness/reports/{suite}/{timestamp}"
    );
    assert_eq!(PathConvention::TASKS, "tasks");
    assert_eq!(PathConvention::FIXTURES, "fixtures/projects");

    let _ = ProviderMode::OpenCode;
    let _ = ProviderMode::OpenCodeRS;
    let _ = ProviderMode::Both;
    let _ = ProviderMode::Either;

    let _report = SimpleReport {
        timestamp: "2026-04-16".to_string(),
        suite: "test".to_string(),
        total: 10,
        passed: 8,
        failed: 1,
        skipped: 1,
        mismatches: 0,
    };
    let _testcase = TestCase {
        id: "test-001".to_string(),
        status: TestCaseStatus::Passed,
        duration: 100,
        failure_classification: FailureClassification::ImplementationFailure,
        error_message: None,
    };
    let _ = TestCaseStatus::Passed;
    let _ = TestCaseStatus::Failed;
    let _ = TestCaseStatus::Skipped;

    let _ = ResetStrategy::None;
    let _ = ResetStrategy::CleanClone;
    let _ = ResetStrategy::RestoreFiles;

    let _ = Severity::Critical;
    let _ = Severity::High;
    let _ = Severity::Medium;
    let _ = Severity::Low;
    let _ = Severity::Cosmetic;

    let _task = Task::new(
        "P0-001",
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
    let _ = TaskCategory::Core;
    let _ = TaskCategory::Schema;
    let _ = TaskCategory::Integration;
    let _ = TaskCategory::Regression;
    let _ = TaskCategory::Smoke;
    let _ = TaskCategory::Performance;
    let _ = TaskCategory::Security;

    let _input = TaskInput::new("opencode", vec!["--help".to_string()], "/project");
    let _outputs = TaskOutputs::new(
        "stdout".to_string(),
        "stderr".to_string(),
        vec!["file1.txt".to_string()],
        vec!["file2.rs".to_string()],
    );

    let _ = TaskStatus::Todo;
    let _ = TaskStatus::InProgress;
    let _ = TaskStatus::Done;
    let _ = TaskStatus::ManualCheck;
    let _ = TaskStatus::Blocked;
    let _ = TaskStatus::Skipped;

    let _ws = Workspace::new(
        "ws-002".to_string(),
        std::path::PathBuf::from("/tmp/workspace2"),
        "test-fixture2".to_string(),
    );
}

#[test]
fn test_types_compile_and_exports_are_correct() {
    use opencode_core::types::*;

    fn _check_all_types_accessible() {
        let _ = AgentMode::OneShot;
        let _ = AllowedVariance::new(vec![], None, vec![]);
        let _ = TimingVariance::new(None, None);
        let _ = AssertionType::ExitCodeEquals(0);
        let _ = ConfigFormat::Toml;
        let _: Box<dyn EnvironmentProbe> = Box::new(DefaultEnvironmentProbe::default());
        let _ = EntryMode::CLI;
        let _ = ExecutionPolicy::ManualCheck;
        let _ = FailureClassification::ImplementationFailure;
        let _ = FixtureProject::new(
            "".to_string(),
            "".to_string(),
            WorkspacePolicy {
                allow_dirty_git: false,
                allow_network: false,
                preserve_on_failure: false,
            },
            ResetStrategy::None,
        );
        let _ = OnMissingDependency::Fail;
        assert_eq!(PathConvention::RUN_ARTIFACTS, "artifacts/run-{id}");
        let _ = ProviderMode::Both;
        let _ = SimpleReport {
            timestamp: "".to_string(),
            suite: "".to_string(),
            total: 0,
            passed: 0,
            failed: 0,
            skipped: 0,
            mismatches: 0,
        };
        let _ = Severity::High;
        let _ = Task::new(
            "TEST-001",
            "Test",
            TaskCategory::Smoke,
            "fixtures/test",
            "desc",
            "outcome",
            vec![],
            EntryMode::CLI,
            AgentMode::OneShot,
            ProviderMode::Both,
            TaskInput::new("cmd", vec![], "/"),
            vec![],
            Severity::Medium,
            ExecutionPolicy::ManualCheck,
            300,
            OnMissingDependency::Fail,
        );
        let _ = TaskInput::new("cmd", vec![], "/");
        let _ = TaskOutputs::new("".to_string(), "".to_string(), vec![], vec![]);
        let _ = TaskStatus::Todo;
        let _ = Workspace::new("".to_string(), std::path::PathBuf::new(), "".to_string());
    }
    _check_all_types_accessible();
}

#[test]
fn test_new_types_from_iteration_7_are_exported() {
    use chrono::Utc;
    use opencode_core::types::*;

    let _ = BaselineMetadata::new()
        .with_source_impl_version("1.0.0".to_string())
        .with_target_impl_version("2.0.0".to_string())
        .with_task_version("1.0.0".to_string())
        .with_fixture_version("1.0.0".to_string())
        .with_normalizer_version("1.0.0".to_string());

    let _ = BaselineRecord::default();

    let _ = ExecutionLevel::AlwaysOn;
    let _ = ExecutionLevel::NightlyOnly;
    let _ = ExecutionLevel::ReleaseOnly;

    let _ = RegressionStatus::Candidate;
    let _ = RegressionStatus::Approved;
    let _ = RegressionStatus::Active;
    let _ = RegressionStatus::Suppressed;
    let _ = RegressionStatus::Resolved;

    let _ = RegressionCase::default();

    let _ = WhitelistScope::Task("TASK-001".to_string());
    let _ = WhitelistScope::Category("timing".to_string());
    let _ = WhitelistScope::Global;

    let future = Utc::now() + chrono::Duration::days(30);
    let _ = WhitelistEntry::new(
        "WL-001".to_string(),
        WhitelistScope::Global,
        "Test reason".to_string(),
        "test-owner".to_string(),
        Some(future),
        None,
        AllowedVariance::new(vec![0], None, vec![]),
        Utc::now(),
        Utc::now(),
    );
}
