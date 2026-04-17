use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::artifact::Artifact;
use super::capability_summary::CapabilitySummary;
use super::failure_classification::FailureClassification;
use super::session_metadata::SessionMetadata;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunnerOutput {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub stdout_path: PathBuf,
    pub stderr_path: PathBuf,
    pub duration_ms: u64,
    pub artifacts: Vec<Artifact>,
    pub artifact_paths: Vec<PathBuf>,
    pub session_metadata: SessionMetadata,
    pub event_log_path: PathBuf,
    pub side_effect_snapshot_path: Option<PathBuf>,
    pub failure_kind: Option<FailureClassification>,
    pub capability_summary: CapabilitySummary,
}

impl RunnerOutput {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        exit_code: Option<i32>,
        stdout: String,
        stderr: String,
        stdout_path: PathBuf,
        stderr_path: PathBuf,
        duration_ms: u64,
        artifacts: Vec<Artifact>,
        artifact_paths: Vec<PathBuf>,
        session_metadata: SessionMetadata,
        event_log_path: PathBuf,
        side_effect_snapshot_path: Option<PathBuf>,
        failure_kind: Option<FailureClassification>,
        capability_summary: CapabilitySummary,
    ) -> Self {
        Self {
            exit_code,
            stdout,
            stderr,
            stdout_path,
            stderr_path,
            duration_ms,
            artifacts,
            artifact_paths,
            session_metadata,
            event_log_path,
            side_effect_snapshot_path,
            failure_kind,
            capability_summary,
        }
    }

    pub fn with_exit_code(mut self, exit_code: Option<i32>) -> Self {
        self.exit_code = exit_code;
        self
    }

    pub fn with_stdout(mut self, stdout: String) -> Self {
        self.stdout = stdout;
        self
    }

    pub fn with_stderr(mut self, stderr: String) -> Self {
        self.stderr = stderr;
        self
    }

    pub fn with_stdout_path(mut self, stdout_path: PathBuf) -> Self {
        self.stdout_path = stdout_path;
        self
    }

    pub fn with_stderr_path(mut self, stderr_path: PathBuf) -> Self {
        self.stderr_path = stderr_path;
        self
    }

    pub fn with_duration_ms(mut self, duration_ms: u64) -> Self {
        self.duration_ms = duration_ms;
        self
    }

    pub fn with_artifacts(mut self, artifacts: Vec<Artifact>) -> Self {
        self.artifacts = artifacts;
        self
    }

    pub fn with_artifact_paths(mut self, artifact_paths: Vec<PathBuf>) -> Self {
        self.artifact_paths = artifact_paths;
        self
    }

    pub fn with_session_metadata(mut self, session_metadata: SessionMetadata) -> Self {
        self.session_metadata = session_metadata;
        self
    }

    pub fn with_event_log_path(mut self, event_log_path: PathBuf) -> Self {
        self.event_log_path = event_log_path;
        self
    }

    pub fn with_side_effect_snapshot_path(
        mut self,
        side_effect_snapshot_path: Option<PathBuf>,
    ) -> Self {
        self.side_effect_snapshot_path = side_effect_snapshot_path;
        self
    }

    pub fn with_failure_kind(mut self, failure_kind: Option<FailureClassification>) -> Self {
        self.failure_kind = failure_kind;
        self
    }

    pub fn with_capability_summary(mut self, capability_summary: CapabilitySummary) -> Self {
        self.capability_summary = capability_summary;
        self
    }
}

impl Default for RunnerOutput {
    fn default() -> Self {
        Self {
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            stdout_path: PathBuf::from("/tmp/stdout.txt"),
            stderr_path: PathBuf::from("/tmp/stderr.txt"),
            duration_ms: 0,
            artifacts: Vec::new(),
            artifact_paths: Vec::new(),
            session_metadata: SessionMetadata::default(),
            event_log_path: PathBuf::from("/tmp/event.log"),
            side_effect_snapshot_path: None,
            failure_kind: None,
            capability_summary: CapabilitySummary::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runner_output_instantiation_with_all_13_fields() {
        let exit_code = Some(0);
        let stdout = "test stdout content".to_string();
        let stderr = "test stderr content".to_string();
        let stdout_path = PathBuf::from("/artifacts/run-001/legacy/stdout.txt");
        let stderr_path = PathBuf::from("/artifacts/run-001/legacy/stderr.txt");
        let duration_ms = 1500u64;
        let artifacts = vec![
            Artifact::file("/tmp/output1.txt"),
            Artifact::directory("/tmp/output_dir"),
        ];
        let artifact_paths = vec![
            PathBuf::from("/tmp/output1.txt"),
            PathBuf::from("/tmp/output_dir"),
        ];
        let session_metadata = SessionMetadata::default()
            .with_session_id("session-123".to_string())
            .with_runner_name("LegacyRunner".to_string())
            .with_task_id("TEST-001".to_string());
        let event_log_path = PathBuf::from("/artifacts/run-001/legacy/event.log");
        let side_effect_snapshot_path = Some(PathBuf::from(
            "/artifacts/run-001/legacy/side-effects/snapshot.json",
        ));
        let failure_kind = Some(FailureClassification::EnvironmentNotSupported);
        let capability_summary = CapabilitySummary::default()
            .with_binary_available(true)
            .with_workspace_prepared(true)
            .with_artifacts_collected(2);

        let output = RunnerOutput::new(
            exit_code,
            stdout.clone(),
            stderr.clone(),
            stdout_path.clone(),
            stderr_path.clone(),
            duration_ms,
            artifacts.clone(),
            artifact_paths.clone(),
            session_metadata.clone(),
            event_log_path.clone(),
            side_effect_snapshot_path.clone(),
            failure_kind,
            capability_summary.clone(),
        );

        assert_eq!(output.exit_code, exit_code);
        assert_eq!(output.stdout, stdout);
        assert_eq!(output.stderr, stderr);
        assert_eq!(output.stdout_path, stdout_path);
        assert_eq!(output.stderr_path, stderr_path);
        assert_eq!(output.duration_ms, duration_ms);
        assert_eq!(output.artifacts.len(), 2);
        assert_eq!(output.artifact_paths.len(), 2);
        assert_eq!(output.session_metadata.session_id, "session-123");
        assert_eq!(output.event_log_path, event_log_path);
        assert_eq!(output.side_effect_snapshot_path, side_effect_snapshot_path);
        assert_eq!(
            output.failure_kind,
            Some(FailureClassification::EnvironmentNotSupported)
        );
        assert!(output.capability_summary.binary_available);
        assert_eq!(output.capability_summary.artifacts_collected, 2);
    }

    #[test]
    fn test_runner_output_builder_pattern() {
        let output = RunnerOutput::default()
            .with_exit_code(Some(42))
            .with_stdout("builder stdout".to_string())
            .with_stderr("builder stderr".to_string())
            .with_stdout_path(PathBuf::from("/custom/stdout.txt"))
            .with_stderr_path(PathBuf::from("/custom/stderr.txt"))
            .with_duration_ms(999)
            .with_artifacts(vec![Artifact::stdout()])
            .with_artifact_paths(vec![PathBuf::from("/custom/artifact")])
            .with_session_metadata(
                SessionMetadata::default().with_runner_name("RustRunner".to_string()),
            )
            .with_event_log_path(PathBuf::from("/custom/event.log"))
            .with_side_effect_snapshot_path(Some(PathBuf::from("/custom/snapshot.json")))
            .with_failure_kind(Some(FailureClassification::ImplementationFailure))
            .with_capability_summary(CapabilitySummary::default().with_timeout_enforced(true));

        assert_eq!(output.exit_code, Some(42));
        assert_eq!(output.stdout, "builder stdout");
        assert_eq!(output.stderr, "builder stderr");
        assert_eq!(output.stdout_path, PathBuf::from("/custom/stdout.txt"));
        assert_eq!(output.stderr_path, PathBuf::from("/custom/stderr.txt"));
        assert_eq!(output.duration_ms, 999);
        assert_eq!(output.artifacts.len(), 1);
        assert_eq!(output.artifact_paths.len(), 1);
        assert_eq!(output.session_metadata.runner_name, "RustRunner");
        assert_eq!(output.event_log_path, PathBuf::from("/custom/event.log"));
        assert_eq!(
            output.side_effect_snapshot_path,
            Some(PathBuf::from("/custom/snapshot.json"))
        );
        assert_eq!(
            output.failure_kind,
            Some(FailureClassification::ImplementationFailure)
        );
        assert!(output.capability_summary.timeout_enforced);
    }

    #[test]
    fn test_runner_output_serde_serialization_deserialization() {
        let output = RunnerOutput::default()
            .with_exit_code(Some(0))
            .with_stdout("serialized stdout".to_string())
            .with_stderr("serialized stderr".to_string())
            .with_duration_ms(2000)
            .with_artifacts(vec![Artifact::file("/test/file.txt")])
            .with_capability_summary(CapabilitySummary::default().with_artifacts_collected(1));

        let serialized = serde_json::to_string(&output).expect("serialization should succeed");
        let deserialized: RunnerOutput =
            serde_json::from_str(&serialized).expect("deserialization should succeed");

        assert_eq!(output.exit_code, deserialized.exit_code);
        assert_eq!(output.stdout, deserialized.stdout);
        assert_eq!(output.stderr, deserialized.stderr);
        assert_eq!(output.duration_ms, deserialized.duration_ms);
        assert_eq!(output.artifacts.len(), deserialized.artifacts.len());
        assert_eq!(
            output.capability_summary.artifacts_collected,
            deserialized.capability_summary.artifacts_collected
        );
    }

    #[test]
    fn test_runner_output_optional_fields_none_case() {
        let output = RunnerOutput::default()
            .with_exit_code(None)
            .with_side_effect_snapshot_path(None)
            .with_failure_kind(None);

        assert!(output.exit_code.is_none());
        assert!(output.side_effect_snapshot_path.is_none());
        assert!(output.failure_kind.is_none());

        let serialized = serde_json::to_string(&output).expect("serialization should succeed");
        let deserialized: RunnerOutput =
            serde_json::from_str(&serialized).expect("deserialization should succeed");

        assert!(deserialized.exit_code.is_none());
        assert!(deserialized.side_effect_snapshot_path.is_none());
        assert!(deserialized.failure_kind.is_none());
    }

    #[test]
    fn test_runner_output_default_values() {
        let output = RunnerOutput::default();

        assert!(output.exit_code.is_none());
        assert!(output.stdout.is_empty());
        assert!(output.stderr.is_empty());
        assert_eq!(output.stdout_path, PathBuf::from("/tmp/stdout.txt"));
        assert_eq!(output.stderr_path, PathBuf::from("/tmp/stderr.txt"));
        assert_eq!(output.duration_ms, 0);
        assert!(output.artifacts.is_empty());
        assert!(output.artifact_paths.is_empty());
        assert_eq!(output.event_log_path, PathBuf::from("/tmp/event.log"));
        assert!(output.side_effect_snapshot_path.is_none());
        assert!(output.failure_kind.is_none());
        assert!(!output.capability_summary.binary_available);
    }

    #[test]
    fn test_runner_output_json_format() {
        let output = RunnerOutput::default();
        let json = serde_json::to_string(&output).expect("serialization should succeed");

        assert!(json.contains("\"exit_code\""));
        assert!(json.contains("\"stdout\""));
        assert!(json.contains("\"stderr\""));
        assert!(json.contains("\"stdout_path\""));
        assert!(json.contains("\"stderr_path\""));
        assert!(json.contains("\"duration_ms\""));
        assert!(json.contains("\"artifacts\""));
        assert!(json.contains("\"artifact_paths\""));
        assert!(json.contains("\"session_metadata\""));
        assert!(json.contains("\"event_log_path\""));
        assert!(json.contains("\"side_effect_snapshot_path\""));
        assert!(json.contains("\"failure_kind\""));
        assert!(json.contains("\"capability_summary\""));
    }

    #[test]
    fn test_runner_output_all_failure_classification_variants() {
        let variants = [
            FailureClassification::ImplementationFailure,
            FailureClassification::DependencyMissing,
            FailureClassification::EnvironmentNotSupported,
            FailureClassification::InfraFailure,
            FailureClassification::FlakySuspected,
        ];

        for variant in variants {
            let output = RunnerOutput::default().with_failure_kind(Some(variant));
            let serialized = serde_json::to_string(&output).expect("serialization should succeed");
            let deserialized: RunnerOutput =
                serde_json::from_str(&serialized).expect("deserialization should succeed");
            assert_eq!(deserialized.failure_kind, Some(variant));
        }
    }

    #[test]
    fn test_runner_output_all_fields_populated() {
        let output = RunnerOutput::new(
            Some(0),
            "stdout".to_string(),
            "stderr".to_string(),
            PathBuf::from("/out/stdout.txt"),
            PathBuf::from("/out/stderr.txt"),
            100,
            vec![Artifact::file("/out/art1.txt")],
            vec![PathBuf::from("/out/art1.txt")],
            SessionMetadata::default(),
            PathBuf::from("/out/event.log"),
            Some(PathBuf::from("/out/snapshot.json")),
            Some(FailureClassification::ImplementationFailure),
            CapabilitySummary::default(),
        );

        assert!(output.exit_code.is_some());
        assert!(!output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
        assert!(!output.stdout_path.as_os_str().is_empty());
        assert!(!output.stderr_path.as_os_str().is_empty());
        assert!(output.duration_ms > 0);
        assert!(!output.artifacts.is_empty());
        assert!(!output.artifact_paths.is_empty());
        assert!(!output.session_metadata.session_id.is_empty());
        assert!(!output.event_log_path.as_os_str().is_empty());
        assert!(output.side_effect_snapshot_path.is_some());
        assert!(output.failure_kind.is_some());
    }
}
