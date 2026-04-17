use crate::error::{ErrorType, Result};
use crate::types::artifact::Artifact;
use crate::types::capability_summary::CapabilitySummary;
use crate::types::failure_classification::FailureClassification;
use crate::types::parity_verdict::{DiffCategory, ParityVerdict};
use crate::types::runner_output::RunnerOutput;
use crate::types::session_metadata::SessionMetadata;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::{debug, info};

#[derive(Debug, Clone)]
pub struct ArtifactPersister {
    run_id: String,
    base_path: PathBuf,
    legacy_path: PathBuf,
    rust_path: PathBuf,
    diff_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataJson {
    pub session_id: String,
    pub runner_name: String,
    pub task_id: String,
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
    pub artifacts_collected: u32,
    pub stdout_size_bytes: usize,
    pub stderr_size_bytes: usize,
    pub side_effects_count: usize,
    pub created_at: DateTime<Utc>,
}

impl ArtifactPersister {
    pub fn new(run_id: impl Into<String>, base_path: impl Into<PathBuf>) -> Self {
        let run_id = run_id.into();
        let base_path = base_path.into();
        let legacy_path = base_path.join(&run_id).join("legacy");
        let rust_path = base_path.join(&run_id).join("rust");
        let diff_path = base_path.join(&run_id).join("diff");

        Self {
            run_id,
            base_path,
            legacy_path,
            rust_path,
            diff_path,
        }
    }

    pub fn create_directory_structure(&self) -> Result<()> {
        let dirs = [
            &self.legacy_path,
            &self.legacy_path.join("artifacts"),
            &self.legacy_path.join("side-effects"),
            &self.rust_path,
            &self.rust_path.join("artifacts"),
            &self.rust_path.join("side-effects"),
            &self.diff_path,
        ];

        for dir in dirs {
            fs::create_dir_all(dir).map_err(|e| {
                ErrorType::Runner(format!(
                    "Failed to create directory '{}': {}",
                    dir.display(),
                    e
                ))
            })?;
        }

        info!(
            "Created artifact directory structure for run_id: {}",
            self.run_id
        );
        debug!(
            "Directories created: legacy={}, rust={}, diff={}",
            self.legacy_path.display(),
            self.rust_path.display(),
            self.diff_path.display()
        );
        Ok(())
    }

    pub fn persist_stdout(&self, content: &str, runner_type: RunnerType) -> Result<PathBuf> {
        let path = self.stdout_path(runner_type);
        debug!(
            "Persisting stdout ({} bytes) to: {}",
            content.len(),
            path.display()
        );
        fs::write(&path, content).map_err(|e| {
            ErrorType::Runner(format!(
                "Failed to write stdout to '{}': {}",
                path.display(),
                e
            ))
        })?;
        Ok(path)
    }

    pub fn persist_stderr(&self, content: &str, runner_type: RunnerType) -> Result<PathBuf> {
        let path = self.stderr_path(runner_type);
        debug!(
            "Persisting stderr ({} bytes) to: {}",
            content.len(),
            path.display()
        );
        fs::write(&path, content).map_err(|e| {
            ErrorType::Runner(format!(
                "Failed to write stderr to '{}': {}",
                path.display(),
                e
            ))
        })?;
        Ok(path)
    }

    pub fn persist_metadata(
        &self,
        metadata: &MetadataJson,
        runner_type: RunnerType,
    ) -> Result<PathBuf> {
        let path = self.metadata_path(runner_type);
        let json = serde_json::to_string_pretty(metadata)
            .map_err(|e| ErrorType::Runner(format!("Failed to serialize metadata: {}", e)))?;
        fs::write(&path, json).map_err(|e| {
            ErrorType::Runner(format!(
                "Failed to write metadata to '{}': {}",
                path.display(),
                e
            ))
        })?;
        Ok(path)
    }

    pub fn persist_artifacts(
        &self,
        artifacts: &[Artifact],
        runner_type: RunnerType,
    ) -> Result<Vec<PathBuf>> {
        let artifacts_dir = self.artifacts_dir(runner_type);
        let mut paths = Vec::new();

        for artifact in artifacts {
            let dest_path = artifacts_dir.join(
                PathBuf::from(&artifact.path)
                    .file_name()
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from(&artifact.path)),
            );

            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    ErrorType::Runner(format!(
                        "Failed to create directory '{}': {}",
                        parent.display(),
                        e
                    ))
                })?;
            }

            if std::path::Path::new(&artifact.path).exists() {
                fs::copy(&artifact.path, &dest_path).map_err(|e| {
                    ErrorType::Runner(format!(
                        "Failed to copy artifact '{}': {}",
                        artifact.path, e
                    ))
                })?;
            }
            paths.push(dest_path);
        }

        Ok(paths)
    }

    pub fn persist_side_effects(
        &self,
        side_effects: &[PathBuf],
        runner_type: RunnerType,
    ) -> Result<PathBuf> {
        let side_effects_dir = self.side_effects_dir(runner_type);
        fs::create_dir_all(&side_effects_dir).map_err(|e| {
            ErrorType::Runner(format!("Failed to create side-effects directory: {}", e))
        })?;

        let snapshot_path = side_effects_dir.join("snapshot.json");
        let snapshot_data = serde_json::to_string_pretty(&side_effects)
            .map_err(|e| ErrorType::Runner(format!("Failed to serialize side effects: {}", e)))?;

        fs::write(&snapshot_path, snapshot_data).map_err(|e| {
            ErrorType::Runner(format!("Failed to write side-effects snapshot: {}", e))
        })?;

        Ok(snapshot_path)
    }

    pub fn generate_diff_report(
        &self,
        legacy_output: &RunnerOutput,
        rust_output: &RunnerOutput,
        verdict: &ParityVerdict,
    ) -> Result<PathBuf> {
        let diff_data = DiffReport {
            run_id: self.run_id.clone(),
            legacy_exit_code: legacy_output.exit_code,
            rust_exit_code: rust_output.exit_code,
            legacy_duration_ms: legacy_output.duration_ms,
            rust_duration_ms: rust_output.duration_ms,
            verdict: verdict.summary(),
            verdict_category: match verdict {
                ParityVerdict::Different { category } => Some(category.clone()),
                _ => None,
            },
            legacy_stdout_size: legacy_output.stdout.len(),
            rust_stdout_size: rust_output.stdout.len(),
            legacy_stderr_size: legacy_output.stderr.len(),
            rust_stderr_size: rust_output.stderr.len(),
            artifacts_diff: Self::compare_artifacts(
                &legacy_output.artifacts,
                &rust_output.artifacts,
            ),
            generated_at: Utc::now(),
        };

        let report_path = self.diff_path.join("report.json");
        info!("Generating diff report at: {}", report_path.display());
        let json = serde_json::to_string_pretty(&diff_data)
            .map_err(|e| ErrorType::Runner(format!("Failed to serialize diff report: {}", e)))?;

        fs::write(&report_path, json)
            .map_err(|e| ErrorType::Runner(format!("Failed to write diff report: {}", e)))?;

        Ok(report_path)
    }

    pub fn generate_verdict_md(
        &self,
        verdict: &ParityVerdict,
        legacy_output: &RunnerOutput,
        rust_output: &RunnerOutput,
    ) -> Result<PathBuf> {
        let verdict_path = self.diff_path.join("verdict.md");
        info!("Generating verdict markdown at: {}", verdict_path.display());
        let content = format!(
            "# Differential Verdict\n\n\
            ## Run ID: {}\n\n\
            ## Verdict: {}\n\n\
            ## Summary\n\n\
            | Metric | Legacy | Rust |\n\
            |--------|--------|------|\n\
            | Exit Code | {:?} | {:?} |\n\
            | Duration (ms) | {} | {} |\n\
            | Stdout Size | {} bytes | {} bytes |\n\
            | Stderr Size | {} bytes | {} bytes |\n\
            | Artifacts | {} | {} |\n\n\
            ## Details\n\n\
            ### Legacy Runner\n\
            - Session ID: {}\n\
            - Binary Available: {}\n\
            - Workspace Prepared: {}\n\
            - Artifacts Collected: {}\n\n\
            ### Rust Runner\n\
            - Session ID: {}\n\
            - Binary Available: {}\n\
            - Workspace Prepared: {}\n\
            - Artifacts Collected: {}\n\n\
            ## Conclusion\n\n\
            {}\n",
            self.run_id,
            verdict.summary(),
            legacy_output.exit_code,
            rust_output.exit_code,
            legacy_output.duration_ms,
            rust_output.duration_ms,
            legacy_output.stdout.len(),
            rust_output.stdout.len(),
            legacy_output.stderr.len(),
            rust_output.stderr.len(),
            legacy_output.artifacts.len(),
            rust_output.artifacts.len(),
            legacy_output.session_metadata.session_id,
            legacy_output.capability_summary.binary_available,
            legacy_output.capability_summary.workspace_prepared,
            legacy_output.capability_summary.artifacts_collected,
            rust_output.session_metadata.session_id,
            rust_output.capability_summary.binary_available,
            rust_output.capability_summary.workspace_prepared,
            rust_output.capability_summary.artifacts_collected,
            match verdict {
                ParityVerdict::Identical =>
                    "The legacy and rust implementations produced **identical** results."
                        .to_string(),
                ParityVerdict::Equivalent =>
                    "The legacy and rust implementations produced **equivalent** results."
                        .to_string(),
                ParityVerdict::Different { category } => format!(
                    "The legacy and rust implementations showed a **{}** difference.",
                    match category {
                        DiffCategory::OutputText => "output text",
                        DiffCategory::ExitCode => "exit code",
                        DiffCategory::Timing => "timing",
                        DiffCategory::SideEffects => "side effects",
                        DiffCategory::Protocol => "protocol",
                    }
                ),
                ParityVerdict::Uncertain => "The comparison result is **uncertain**.".to_string(),
                ParityVerdict::Error { runner, reason } =>
                    format!("An **error** occurred in {}: {}", runner, reason),
            }
        );

        fs::write(&verdict_path, content)
            .map_err(|e| ErrorType::Runner(format!("Failed to write verdict markdown: {}", e)))?;

        Ok(verdict_path)
    }

    pub fn build_runner_output(
        &self,
        runner_type: RunnerType,
        exit_code: Option<i32>,
        stdout: String,
        stderr: String,
        duration_ms: u64,
        artifacts: Vec<Artifact>,
        session_metadata: SessionMetadata,
        side_effect_snapshot_path: Option<PathBuf>,
        failure_kind: Option<FailureClassification>,
        capability_summary: CapabilitySummary,
    ) -> Result<RunnerOutput> {
        debug!("Building runner output for {:?}", runner_type);
        let stdout_path = self.persist_stdout(&stdout, runner_type.clone())?;
        let stderr_path = self.persist_stderr(&stderr, runner_type.clone())?;

        let metadata = MetadataJson {
            session_id: session_metadata.session_id.clone(),
            runner_name: session_metadata.runner_name.clone(),
            task_id: session_metadata.task_id.clone(),
            exit_code,
            duration_ms,
            artifacts_collected: artifacts.len() as u32,
            stdout_size_bytes: stdout.len(),
            stderr_size_bytes: stderr.len(),
            side_effects_count: side_effect_snapshot_path.is_some().into(),
            created_at: Utc::now(),
        };
        let _ = self.persist_metadata(&metadata, runner_type.clone())?;

        let artifact_paths = self.persist_artifacts(&artifacts, runner_type.clone())?;
        debug!("Persisted {} artifacts", artifact_paths.len());

        let event_log_path = self.event_log_path(runner_type);
        fs::write(&event_log_path, format!("{:?}", session_metadata))
            .map_err(|e| ErrorType::Runner(format!("Failed to write event log: {}", e)))?;

        info!(
            "Runner output built for task_id: {}",
            session_metadata.task_id
        );
        Ok(RunnerOutput::new(
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
        ))
    }

    fn stdout_path(&self, runner_type: RunnerType) -> PathBuf {
        match runner_type {
            RunnerType::Legacy => self.legacy_path.join("stdout.txt"),
            RunnerType::Rust => self.rust_path.join("stdout.txt"),
        }
    }

    fn stderr_path(&self, runner_type: RunnerType) -> PathBuf {
        match runner_type {
            RunnerType::Legacy => self.legacy_path.join("stderr.txt"),
            RunnerType::Rust => self.rust_path.join("stderr.txt"),
        }
    }

    fn metadata_path(&self, runner_type: RunnerType) -> PathBuf {
        match runner_type {
            RunnerType::Legacy => self.legacy_path.join("metadata.json"),
            RunnerType::Rust => self.rust_path.join("metadata.json"),
        }
    }

    fn artifacts_dir(&self, runner_type: RunnerType) -> PathBuf {
        match runner_type {
            RunnerType::Legacy => self.legacy_path.join("artifacts"),
            RunnerType::Rust => self.rust_path.join("artifacts"),
        }
    }

    fn side_effects_dir(&self, runner_type: RunnerType) -> PathBuf {
        match runner_type {
            RunnerType::Legacy => self.legacy_path.join("side-effects"),
            RunnerType::Rust => self.rust_path.join("side-effects"),
        }
    }

    fn event_log_path(&self, runner_type: RunnerType) -> PathBuf {
        match runner_type {
            RunnerType::Legacy => self.legacy_path.join("event.log"),
            RunnerType::Rust => self.rust_path.join("event.log"),
        }
    }

    fn compare_artifacts(legacy: &[Artifact], rust: &[Artifact]) -> ArtifactDiff {
        let legacy_paths: std::collections::HashSet<_> = legacy.iter().map(|a| &a.path).collect();
        let rust_paths: std::collections::HashSet<_> = rust.iter().map(|a| &a.path).collect();

        let only_in_legacy: Vec<String> = legacy_paths
            .difference(&rust_paths)
            .map(|s| (*s).clone())
            .collect();
        let only_in_rust: Vec<String> = rust_paths
            .difference(&legacy_paths)
            .map(|s| (*s).clone())
            .collect();
        let in_both: Vec<String> = legacy_paths
            .intersection(&rust_paths)
            .map(|s| (*s).clone())
            .collect();

        ArtifactDiff {
            only_in_legacy,
            only_in_rust,
            in_both,
        }
    }

    pub fn base_path(&self) -> &PathBuf {
        &self.base_path
    }

    pub fn run_id(&self) -> &str {
        &self.run_id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffReport {
    pub run_id: String,
    pub legacy_exit_code: Option<i32>,
    pub rust_exit_code: Option<i32>,
    pub legacy_duration_ms: u64,
    pub rust_duration_ms: u64,
    pub verdict: String,
    pub verdict_category: Option<DiffCategory>,
    pub legacy_stdout_size: usize,
    pub rust_stdout_size: usize,
    pub legacy_stderr_size: usize,
    pub rust_stderr_size: usize,
    pub artifacts_diff: ArtifactDiff,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactDiff {
    pub only_in_legacy: Vec<String>,
    pub only_in_rust: Vec<String>,
    pub in_both: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunnerType {
    Legacy,
    Rust,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_artifact_persister_creates_directory_structure() {
        let temp_dir = TempDir::new().unwrap();
        let persister = ArtifactPersister::new("test-run-001", temp_dir.path());

        persister.create_directory_structure().unwrap();

        assert!(temp_dir.path().join("test-run-001/legacy").exists());
        assert!(temp_dir
            .path()
            .join("test-run-001/legacy/artifacts")
            .exists());
        assert!(temp_dir
            .path()
            .join("test-run-001/legacy/side-effects")
            .exists());
        assert!(temp_dir.path().join("test-run-001/rust").exists());
        assert!(temp_dir.path().join("test-run-001/rust/artifacts").exists());
        assert!(temp_dir
            .path()
            .join("test-run-001/rust/side-effects")
            .exists());
        assert!(temp_dir.path().join("test-run-001/diff").exists());
    }

    #[test]
    fn test_persist_stdout_writes_file_and_returns_correct_path() {
        let temp_dir = TempDir::new().unwrap();
        let persister = ArtifactPersister::new("test-run-002", temp_dir.path());
        persister.create_directory_structure().unwrap();

        let content = "test stdout content";
        let path = persister
            .persist_stdout(content, RunnerType::Legacy)
            .unwrap();

        assert_eq!(path, temp_dir.path().join("test-run-002/legacy/stdout.txt"));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), content);
    }

    #[test]
    fn test_persist_stderr_writes_file_and_returns_correct_path() {
        let temp_dir = TempDir::new().unwrap();
        let persister = ArtifactPersister::new("test-run-003", temp_dir.path());
        persister.create_directory_structure().unwrap();

        let content = "test stderr content";
        let path = persister.persist_stderr(content, RunnerType::Rust).unwrap();

        assert_eq!(path, temp_dir.path().join("test-run-003/rust/stderr.txt"));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), content);
    }

    #[test]
    fn test_persist_metadata_writes_valid_json() {
        let temp_dir = TempDir::new().unwrap();
        let persister = ArtifactPersister::new("test-run-004", temp_dir.path());
        persister.create_directory_structure().unwrap();

        let metadata = MetadataJson {
            session_id: "session-123".to_string(),
            runner_name: "LegacyRunner".to_string(),
            task_id: "TASK-001".to_string(),
            exit_code: Some(0),
            duration_ms: 1500,
            artifacts_collected: 5,
            stdout_size_bytes: 1024,
            stderr_size_bytes: 256,
            side_effects_count: 2,
            created_at: Utc::now(),
        };

        let path = persister
            .persist_metadata(&metadata, RunnerType::Legacy)
            .unwrap();

        let read_content = std::fs::read_to_string(&path).unwrap();
        let parsed: MetadataJson = serde_json::from_str(&read_content).unwrap();

        assert_eq!(parsed.session_id, "session-123");
        assert_eq!(parsed.runner_name, "LegacyRunner");
        assert_eq!(parsed.task_id, "TASK-001");
        assert_eq!(parsed.exit_code, Some(0));
        assert_eq!(parsed.duration_ms, 1500);
        assert_eq!(parsed.artifacts_collected, 5);
    }

    #[test]
    fn test_persist_stdout_and_stderr_for_both_runner_types() {
        let temp_dir = TempDir::new().unwrap();
        let persister = ArtifactPersister::new("test-run-005", temp_dir.path());
        persister.create_directory_structure().unwrap();

        let legacy_out = persister
            .persist_stdout("legacy stdout", RunnerType::Legacy)
            .unwrap();
        let legacy_err = persister
            .persist_stderr("legacy stderr", RunnerType::Legacy)
            .unwrap();
        let rust_out = persister
            .persist_stdout("rust stdout", RunnerType::Rust)
            .unwrap();
        let rust_err = persister
            .persist_stderr("rust stderr", RunnerType::Rust)
            .unwrap();

        assert!(legacy_out
            .display()
            .to_string()
            .contains("legacy/stdout.txt"));
        assert!(legacy_err
            .display()
            .to_string()
            .contains("legacy/stderr.txt"));
        assert!(rust_out.display().to_string().contains("rust/stdout.txt"));
        assert!(rust_err.display().to_string().contains("rust/stderr.txt"));
    }

    #[test]
    fn test_runner_type_enum_values() {
        assert_eq!(RunnerType::Legacy, RunnerType::Legacy);
        assert_eq!(RunnerType::Rust, RunnerType::Rust);
        assert_ne!(RunnerType::Legacy, RunnerType::Rust);
    }

    #[test]
    fn test_artifact_persister_new_sets_correct_paths() {
        let temp_dir = TempDir::new().unwrap();
        let persister = ArtifactPersister::new("run-abc", temp_dir.path());

        assert_eq!(persister.run_id(), "run-abc");
        assert_eq!(persister.base_path(), temp_dir.path());
    }

    #[test]
    fn test_compare_artifacts_finds_differences() {
        let legacy = vec![
            Artifact::file("/tmp/file1.txt"),
            Artifact::file("/tmp/file2.txt"),
            Artifact::file("/tmp/shared.txt"),
        ];
        let rust = vec![
            Artifact::file("/tmp/file2.txt"),
            Artifact::file("/tmp/file3.txt"),
            Artifact::file("/tmp/shared.txt"),
        ];

        let diff = ArtifactPersister::compare_artifacts(&legacy, &rust);

        assert_eq!(diff.only_in_legacy, vec!["/tmp/file1.txt"]);
        assert_eq!(diff.only_in_rust, vec!["/tmp/file3.txt"]);
        let mut expected = diff.in_both.clone();
        expected.sort();
        assert_eq!(expected, vec!["/tmp/file2.txt", "/tmp/shared.txt"]);
    }
}
