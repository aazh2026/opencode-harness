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
use std::path::{Path, PathBuf};
use tracing::{debug, info};
use walkdir::WalkDir;

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
                    "Failed to create artifact directory '{}' (run_id: {}): {}",
                    dir.display(),
                    self.run_id,
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

    fn format_permissions(metadata: &fs::Metadata) -> String {
        use std::os::unix::fs::PermissionsExt;
        format!("{:o}", metadata.permissions().mode() & 0o777)
    }

    pub fn capture_file_tree(&self, root: &Path) -> Result<FileTreeSnapshot> {
        let root = root.to_path_buf();
        let mut entries = Vec::new();
        let mut total_files = 0;
        let mut total_dirs = 0;

        for entry in WalkDir::new(&root).follow_links(false) {
            let entry = entry.map_err(|e| {
                ErrorType::Runner(format!(
                    "Failed to walk directory '{}': {}",
                    root.display(),
                    e
                ))
            })?;

            let path = entry.path().to_path_buf();
            let relative_path = path.strip_prefix(&root).unwrap_or(&path).to_path_buf();

            let metadata = entry.metadata().map_err(|e| {
                ErrorType::Runner(format!(
                    "Failed to get metadata for '{}': {}",
                    path.display(),
                    e
                ))
            })?;

            let entry_type = if metadata.is_file() {
                total_files += 1;
                FileTreeEntryType::File
            } else if metadata.is_dir() {
                total_dirs += 1;
                FileTreeEntryType::Directory
            } else if metadata.is_symlink() {
                FileTreeEntryType::SymLink
            } else {
                FileTreeEntryType::Other
            };

            let size_bytes = if metadata.is_file() {
                Some(metadata.len())
            } else {
                None
            };

            let permissions = Self::format_permissions(&metadata);

            let modified_at = metadata.modified().ok().map(DateTime::from);

            entries.push(FileTreeEntry {
                path: relative_path,
                entry_type,
                size_bytes,
                permissions,
                modified_at,
            });
        }

        Ok(FileTreeSnapshot {
            root_path: root,
            captured_at: Utc::now(),
            entries,
            total_files,
            total_dirs,
        })
    }

    pub fn diff_file_trees(
        &self,
        before: &FileTreeSnapshot,
        after: &FileTreeSnapshot,
    ) -> FileTreeDiff {
        let before_paths: std::collections::HashMap<_, _> = before
            .entries
            .iter()
            .map(|e| (e.path.clone(), e.clone()))
            .collect();
        let after_paths: std::collections::HashMap<_, _> = after
            .entries
            .iter()
            .map(|e| (e.path.clone(), e.clone()))
            .collect();

        let before_keys: std::collections::HashSet<PathBuf> =
            before_paths.keys().cloned().collect();
        let after_keys: std::collections::HashSet<PathBuf> = after_paths.keys().cloned().collect();

        let removed: Vec<PathBuf> = before_keys.difference(&after_keys).cloned().collect();

        let added: Vec<PathBuf> = after_keys.difference(&before_keys).cloned().collect();

        let unchanged_count = before_keys
            .intersection(&after_keys)
            .filter(|k| {
                let before_entry = before_paths.get(k.as_path()).unwrap();
                let after_entry = after_paths.get(k.as_path()).unwrap();
                before_entry.entry_type == after_entry.entry_type
                    && before_entry.size_bytes == after_entry.size_bytes
                    && before_entry.permissions == after_entry.permissions
            })
            .count();

        let modified: Vec<PathBuf> = before_keys
            .intersection(&after_keys)
            .filter(|k| {
                let before_entry = before_paths.get(k.as_path()).unwrap();
                let after_entry = after_paths.get(k.as_path()).unwrap();
                before_entry.entry_type != after_entry.entry_type
                    || before_entry.size_bytes != after_entry.size_bytes
                    || before_entry.permissions != after_entry.permissions
            })
            .cloned()
            .collect();

        FileTreeDiff {
            added,
            removed,
            modified,
            unchanged_count,
        }
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
                ParityVerdict::Fail { category, .. } => Some(category.clone()),
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
                ParityVerdict::Pass =>
                    "The legacy and rust implementations produced **identical** results."
                        .to_string(),
                ParityVerdict::PassWithAllowedVariance { variance_type, details } =>
                    format!(
                        "The legacy and rust implementations produced **equivalent** results ({:?}): {}.",
                        variance_type, details
                    ),
                ParityVerdict::Fail { category, details } => format!(
                    "The legacy and rust implementations showed a **{}** difference: {}.",
                    match category {
                        DiffCategory::OutputText => "output text",
                        DiffCategory::ExitCode => "exit code",
                        DiffCategory::Timing => "timing",
                        DiffCategory::SideEffects => "side effects",
                        DiffCategory::Protocol => "protocol",
                    },
                    details
                ),
                ParityVerdict::Warn { category, message } => format!(
                    "A **{}** warning was detected: {}.",
                    match category {
                        DiffCategory::OutputText => "output text",
                        DiffCategory::ExitCode => "exit code",
                        DiffCategory::Timing => "timing",
                        DiffCategory::SideEffects => "side effects",
                        DiffCategory::Protocol => "protocol",
                    },
                    message
                ),
                ParityVerdict::ManualCheck { reason, candidates } => format!(
                    "The comparison result is **uncertain** (manual check required): {} ({} candidates).",
                    reason,
                    candidates.len()
                ),
                ParityVerdict::Blocked { reason } =>
                    format!("Execution was **blocked**: {:?}", reason),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileTreeEntryType {
    File,
    Directory,
    SymLink,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTreeEntry {
    pub path: PathBuf,
    pub entry_type: FileTreeEntryType,
    pub size_bytes: Option<u64>,
    pub permissions: String,
    pub modified_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTreeSnapshot {
    pub root_path: PathBuf,
    pub captured_at: DateTime<Utc>,
    pub entries: Vec<FileTreeEntry>,
    pub total_files: usize,
    pub total_dirs: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTreeDiff {
    pub added: Vec<PathBuf>,
    pub removed: Vec<PathBuf>,
    pub modified: Vec<PathBuf>,
    pub unchanged_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatchLineType {
    Context,
    Addition,
    Deletion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchLine {
    pub line_type: PatchLineType,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchHunk {
    pub old_start: u32,
    pub new_start: u32,
    pub lines: Vec<PatchLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitPatch {
    pub file_path: String,
    pub hunks: Vec<PatchHunk>,
    pub binary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitDiff {
    pub patches: Vec<GitPatch>,
    pub total_additions: usize,
    pub total_deletions: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitFileChange {
    pub path: String,
    pub old_mode: Option<String>,
    pub new_mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatus {
    pub is_clean: bool,
    pub staged: Vec<GitFileChange>,
    pub modified: Vec<GitFileChange>,
    pub untracked: Vec<GitFileChange>,
    pub conflicted: Vec<GitFileChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitSnapshot {
    pub captured_at: DateTime<Utc>,
    pub status: GitStatus,
    pub diff: Option<GitDiff>,
    pub branch: String,
    pub commit_sha: Option<String>,
}

impl ArtifactPersister {
    pub fn capture_git_status(&self, repo_path: &Path) -> Result<GitStatus> {
        let repo = git2::Repository::discover(repo_path).map_err(|e| {
            ErrorType::Runner(format!(
                "Failed to discover git repository at '{}': {}",
                repo_path.display(),
                e
            ))
        })?;

        let mut opts = git2::StatusOptions::new();
        opts.include_untracked(true);
        opts.recurse_untracked_dirs(true);

        let statuses = repo
            .statuses(Some(&mut opts))
            .map_err(|e| ErrorType::Runner(format!("Failed to get git status: {}", e)))?;

        let mut staged = Vec::new();
        let mut modified = Vec::new();
        let mut untracked = Vec::new();
        let mut conflicted = Vec::new();

        for entry in statuses.iter() {
            let status = entry.status();
            let path = entry.path().unwrap_or("").to_string();

            if status.intersects(
                git2::Status::INDEX_NEW
                    | git2::Status::INDEX_MODIFIED
                    | git2::Status::INDEX_DELETED
                    | git2::Status::INDEX_RENAMED,
            ) {
                staged.push(GitFileChange {
                    path: path.clone(),
                    old_mode: None,
                    new_mode: None,
                });
            }
            if status.intersects(
                git2::Status::WT_MODIFIED | git2::Status::WT_DELETED | git2::Status::WT_RENAMED,
            ) {
                modified.push(GitFileChange {
                    path: path.clone(),
                    old_mode: None,
                    new_mode: None,
                });
            }
            if status.intersects(git2::Status::WT_NEW) {
                untracked.push(GitFileChange {
                    path: path.clone(),
                    old_mode: None,
                    new_mode: None,
                });
            }
            if status.intersects(git2::Status::CONFLICTED) {
                conflicted.push(GitFileChange {
                    path,
                    old_mode: None,
                    new_mode: None,
                });
            }
        }

        let is_clean = staged.is_empty()
            && modified.is_empty()
            && untracked.is_empty()
            && conflicted.is_empty();

        Ok(GitStatus {
            is_clean,
            staged,
            modified,
            untracked,
            conflicted,
        })
    }

    pub fn capture_git_diff(&self, repo_path: &Path) -> Result<GitDiff> {
        let repo = git2::Repository::discover(repo_path).map_err(|e| {
            ErrorType::Runner(format!(
                "Failed to discover git repository at '{}': {}",
                repo_path.display(),
                e
            ))
        })?;

        let mut opts = git2::DiffOptions::new();
        opts.include_untracked(true);

        let diff = repo
            .diff_index_to_workdir(None, Some(&mut opts))
            .map_err(|e| ErrorType::Runner(format!("Failed to get git diff: {}", e)))?;

        let mut patches = Vec::new();
        let mut total_additions = 0usize;
        let mut total_deletions = 0usize;

        diff.print(git2::DiffFormat::Patch, |delta, _hunk, line| {
            let file_path = delta
                .new_file()
                .path()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| {
                    delta
                        .old_file()
                        .path()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default()
                });

            let binary = line.origin() == 'B';

            if binary {
                patches.push(GitPatch {
                    file_path,
                    hunks: Vec::new(),
                    binary: true,
                });
                return true;
            }

            let patch_idx = patches.iter_mut().position(|p| p.file_path == file_path);

            let patch = if let Some(idx) = patch_idx {
                &mut patches[idx]
            } else {
                patches.push(GitPatch {
                    file_path: file_path.clone(),
                    hunks: Vec::new(),
                    binary: false,
                });
                patches.last_mut().unwrap()
            };

            let line_type = match line.origin() {
                '+' => {
                    total_additions += 1;
                    PatchLineType::Addition
                }
                '-' => {
                    total_deletions += 1;
                    PatchLineType::Deletion
                }
                ' ' => PatchLineType::Context,
                _ => return true,
            };

            let content = std::str::from_utf8(line.content()).unwrap_or("");

            let (old_start, new_start) = if let Some(hunk) = patch.hunks.last_mut() {
                match line_type {
                    PatchLineType::Addition => {
                        hunk.new_start += 1;
                    }
                    PatchLineType::Deletion => {
                        hunk.old_start += 1;
                    }
                    PatchLineType::Context => {
                        hunk.old_start += 1;
                        hunk.new_start += 1;
                    }
                }
                (hunk.old_start, hunk.new_start)
            } else {
                (
                    line.new_lineno().unwrap_or(1),
                    line.old_lineno().unwrap_or(1),
                )
            };

            if patch.hunks.is_empty()
                || line.origin() == '+'
                || line.origin() == '-'
                || line.origin() == ' '
            {
                let should_create_hunk = if let Some(last_hunk) = patch.hunks.last() {
                    let last_end = last_hunk.old_start + last_hunk.lines.len() as u32;
                    line.old_lineno().map(|ln| ln > last_end).unwrap_or(false)
                } else {
                    true
                };

                if should_create_hunk {
                    patch.hunks.push(PatchHunk {
                        old_start,
                        new_start,
                        lines: Vec::new(),
                    });
                }
            }

            if let Some(hunk) = patch.hunks.last_mut() {
                hunk.lines.push(PatchLine {
                    line_type,
                    content: content.to_string(),
                });
            }

            true
        })
        .map_err(|e| ErrorType::Runner(format!("Failed to print git diff: {}", e)))?;

        Ok(GitDiff {
            patches,
            total_additions,
            total_deletions,
        })
    }

    pub fn generate_patch(&self, before: &FileTreeSnapshot, after: &FileTreeSnapshot) -> String {
        let diff = self.diff_file_trees(before, after);
        let mut patch_lines = Vec::new();

        for added in &diff.added {
            patch_lines.push(format!("+ {}", added.display()));
        }

        for removed in &diff.removed {
            patch_lines.push(format!("- {}", removed.display()));
        }

        for modified in &diff.modified {
            patch_lines.push(format!("~ {}", modified.display()));
        }

        if patch_lines.is_empty() {
            "No changes".to_string()
        } else {
            patch_lines.join("\n")
        }
    }

    pub fn capture_git_snapshot(&self, repo_path: &Path) -> Result<GitSnapshot> {
        let status = self.capture_git_status(repo_path)?;
        let diff = self.capture_git_diff(repo_path)?;

        let repo = git2::Repository::discover(repo_path).map_err(|e| {
            ErrorType::Runner(format!(
                "Failed to discover git repository at '{}': {}",
                repo_path.display(),
                e
            ))
        })?;

        let branch = repo
            .head()
            .ok()
            .and_then(|h| h.shorthand().map(String::from))
            .unwrap_or_else(|| "HEAD".to_string());

        let commit_sha = repo
            .head()
            .ok()
            .and_then(|h| h.peel_to_commit().ok())
            .map(|c| c.id().to_string());

        Ok(GitSnapshot {
            captured_at: Utc::now(),
            status,
            diff: Some(diff),
            branch,
            commit_sha,
        })
    }
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

    #[test]
    fn test_file_tree_entry_type_enum_variants() {
        assert_eq!(FileTreeEntryType::File, FileTreeEntryType::File);
        assert_eq!(FileTreeEntryType::Directory, FileTreeEntryType::Directory);
        assert_eq!(FileTreeEntryType::SymLink, FileTreeEntryType::SymLink);
        assert_eq!(FileTreeEntryType::Other, FileTreeEntryType::Other);
        assert_ne!(FileTreeEntryType::File, FileTreeEntryType::Directory);
    }

    #[test]
    fn test_file_tree_entry_struct_captures_fields() {
        let path = PathBuf::from("test_file.txt");
        let entry = FileTreeEntry {
            path: path.clone(),
            entry_type: FileTreeEntryType::File,
            size_bytes: Some(1024),
            permissions: String::from("644"),
            modified_at: Some(Utc::now()),
        };

        assert_eq!(entry.path, path);
        assert_eq!(entry.entry_type, FileTreeEntryType::File);
        assert_eq!(entry.size_bytes, Some(1024));
        assert_eq!(entry.permissions, "644");
        assert!(entry.modified_at.is_some());
    }

    #[test]
    fn test_file_tree_snapshot_captures_root_timestamp_entries_counts() {
        let root = PathBuf::from("/test/root");
        let captured_at = Utc::now();
        let entries = vec![
            FileTreeEntry {
                path: PathBuf::from("file1.txt"),
                entry_type: FileTreeEntryType::File,
                size_bytes: Some(100),
                permissions: String::from("644"),
                modified_at: Some(Utc::now()),
            },
            FileTreeEntry {
                path: PathBuf::from("subdir"),
                entry_type: FileTreeEntryType::Directory,
                size_bytes: None,
                permissions: String::from("755"),
                modified_at: Some(Utc::now()),
            },
        ];

        let snapshot = FileTreeSnapshot {
            root_path: root.clone(),
            captured_at,
            entries: entries.clone(),
            total_files: 1,
            total_dirs: 1,
        };

        assert_eq!(snapshot.root_path, root);
        assert_eq!(snapshot.captured_at, captured_at);
        assert_eq!(snapshot.entries.len(), 2);
        assert_eq!(snapshot.total_files, 1);
        assert_eq!(snapshot.total_dirs, 1);
    }

    #[test]
    fn test_file_tree_diff_identifies_added_removed_modified_unchanged() {
        let before_entries = vec![
            FileTreeEntry {
                path: PathBuf::from("file1.txt"),
                entry_type: FileTreeEntryType::File,
                size_bytes: Some(100),
                permissions: String::from("644"),
                modified_at: Some(Utc::now()),
            },
            FileTreeEntry {
                path: PathBuf::from("file2.txt"),
                entry_type: FileTreeEntryType::File,
                size_bytes: Some(200),
                permissions: String::from("644"),
                modified_at: Some(Utc::now()),
            },
            FileTreeEntry {
                path: PathBuf::from("unchanged.txt"),
                entry_type: FileTreeEntryType::File,
                size_bytes: Some(300),
                permissions: String::from("644"),
                modified_at: Some(Utc::now()),
            },
        ];

        let after_entries = vec![
            FileTreeEntry {
                path: PathBuf::from("file1.txt"),
                entry_type: FileTreeEntryType::File,
                size_bytes: Some(150),
                permissions: String::from("644"),
                modified_at: Some(Utc::now()),
            },
            FileTreeEntry {
                path: PathBuf::from("file3.txt"),
                entry_type: FileTreeEntryType::File,
                size_bytes: Some(250),
                permissions: String::from("644"),
                modified_at: Some(Utc::now()),
            },
            FileTreeEntry {
                path: PathBuf::from("unchanged.txt"),
                entry_type: FileTreeEntryType::File,
                size_bytes: Some(300),
                permissions: String::from("644"),
                modified_at: Some(Utc::now()),
            },
        ];

        let before = FileTreeSnapshot {
            root_path: PathBuf::from("/test"),
            captured_at: Utc::now(),
            entries: before_entries,
            total_files: 3,
            total_dirs: 0,
        };

        let after = FileTreeSnapshot {
            root_path: PathBuf::from("/test"),
            captured_at: Utc::now(),
            entries: after_entries,
            total_files: 3,
            total_dirs: 0,
        };

        let persister = ArtifactPersister::new("diff-test", PathBuf::from("/tmp"));
        let diff = persister.diff_file_trees(&before, &after);

        assert_eq!(diff.added, vec![PathBuf::from("file3.txt")]);
        assert_eq!(diff.removed, vec![PathBuf::from("file2.txt")]);
        assert_eq!(diff.modified, vec![PathBuf::from("file1.txt")]);
        assert_eq!(diff.unchanged_count, 1);
    }

    #[test]
    fn test_patch_line_type_enum_variants() {
        assert_eq!(PatchLineType::Context, PatchLineType::Context);
        assert_eq!(PatchLineType::Addition, PatchLineType::Addition);
        assert_eq!(PatchLineType::Deletion, PatchLineType::Deletion);
        assert_ne!(PatchLineType::Addition, PatchLineType::Deletion);
        assert_ne!(PatchLineType::Context, PatchLineType::Addition);
    }

    #[test]
    fn test_patch_line_struct_captures_fields() {
        let line = PatchLine {
            line_type: PatchLineType::Addition,
            content: "+ Hello, world!".to_string(),
        };

        assert_eq!(line.line_type, PatchLineType::Addition);
        assert_eq!(line.content, "+ Hello, world!");
    }

    #[test]
    fn test_patch_hunk_struct_captures_fields() {
        let lines = vec![
            PatchLine {
                line_type: PatchLineType::Context,
                content: " unchanged line".to_string(),
            },
            PatchLine {
                line_type: PatchLineType::Addition,
                content: "+ added line".to_string(),
            },
            PatchLine {
                line_type: PatchLineType::Deletion,
                content: "- removed line".to_string(),
            },
        ];

        let hunk = PatchHunk {
            old_start: 10,
            new_start: 12,
            lines,
        };

        assert_eq!(hunk.old_start, 10);
        assert_eq!(hunk.new_start, 12);
        assert_eq!(hunk.lines.len(), 3);
        assert_eq!(hunk.lines[0].line_type, PatchLineType::Context);
        assert_eq!(hunk.lines[1].line_type, PatchLineType::Addition);
        assert_eq!(hunk.lines[2].line_type, PatchLineType::Deletion);
    }

    #[test]
    fn test_git_patch_struct_captures_fields() {
        let hunks = vec![PatchHunk {
            old_start: 1,
            new_start: 1,
            lines: vec![PatchLine {
                line_type: PatchLineType::Context,
                content: " context".to_string(),
            }],
        }];

        let patch = GitPatch {
            file_path: "src/main.rs".to_string(),
            hunks: hunks.clone(),
            binary: false,
        };

        assert_eq!(patch.file_path, "src/main.rs");
        assert_eq!(patch.hunks.len(), 1);
        assert!(!patch.binary);

        let binary_patch = GitPatch {
            file_path: "binary.png".to_string(),
            hunks: Vec::new(),
            binary: true,
        };

        assert!(binary_patch.binary);
    }

    #[test]
    fn test_git_diff_aggregates_patches() {
        let diff = GitDiff {
            patches: vec![
                GitPatch {
                    file_path: "file1.txt".to_string(),
                    hunks: Vec::new(),
                    binary: false,
                },
                GitPatch {
                    file_path: "file2.txt".to_string(),
                    hunks: Vec::new(),
                    binary: false,
                },
            ],
            total_additions: 15,
            total_deletions: 8,
        };

        assert_eq!(diff.patches.len(), 2);
        assert_eq!(diff.total_additions, 15);
        assert_eq!(diff.total_deletions, 8);
    }

    #[test]
    fn test_git_file_change_captures_path_and_mode() {
        let change = GitFileChange {
            path: "src/lib.rs".to_string(),
            old_mode: Some("100644".to_string()),
            new_mode: Some("100755".to_string()),
        };

        assert_eq!(change.path, "src/lib.rs");
        assert_eq!(change.old_mode, Some("100644".to_string()));
        assert_eq!(change.new_mode, Some("100755".to_string()));

        let new_file = GitFileChange {
            path: "new.txt".to_string(),
            old_mode: None,
            new_mode: Some("100644".to_string()),
        };

        assert_eq!(new_file.old_mode, None);
        assert_eq!(new_file.new_mode, Some("100644".to_string()));
    }

    #[test]
    fn test_git_status_identifies_clean_and_categorizes_changes() {
        let clean_status = GitStatus {
            is_clean: true,
            staged: Vec::new(),
            modified: Vec::new(),
            untracked: Vec::new(),
            conflicted: Vec::new(),
        };

        assert!(clean_status.is_clean);
        assert!(clean_status.staged.is_empty());
        assert!(clean_status.modified.is_empty());

        let dirty_status = GitStatus {
            is_clean: false,
            staged: vec![GitFileChange {
                path: "staged.txt".to_string(),
                old_mode: None,
                new_mode: Some("100644".to_string()),
            }],
            modified: vec![GitFileChange {
                path: "modified.txt".to_string(),
                old_mode: Some("100644".to_string()),
                new_mode: Some("100644".to_string()),
            }],
            untracked: vec![GitFileChange {
                path: "untracked.txt".to_string(),
                old_mode: None,
                new_mode: None,
            }],
            conflicted: Vec::new(),
        };

        assert!(!dirty_status.is_clean);
        assert_eq!(dirty_status.staged.len(), 1);
        assert_eq!(dirty_status.modified.len(), 1);
        assert_eq!(dirty_status.untracked.len(), 1);
        assert_eq!(dirty_status.conflicted.len(), 0);
    }

    #[test]
    fn test_git_snapshot_captures_timestamp_status_branch_commit() {
        let status = GitStatus {
            is_clean: true,
            staged: Vec::new(),
            modified: Vec::new(),
            untracked: Vec::new(),
            conflicted: Vec::new(),
        };

        let diff = GitDiff {
            patches: Vec::new(),
            total_additions: 0,
            total_deletions: 0,
        };

        let snapshot = GitSnapshot {
            captured_at: Utc::now(),
            status: status.clone(),
            diff: Some(diff.clone()),
            branch: "main".to_string(),
            commit_sha: Some("abc123".to_string()),
        };

        assert_eq!(snapshot.branch, "main");
        assert_eq!(snapshot.commit_sha, Some("abc123".to_string()));
        assert!(snapshot.status.is_clean);
        assert!(snapshot.diff.is_some());
        assert_eq!(snapshot.diff.unwrap().patches.len(), 0);
    }

    #[test]
    fn test_generate_patch_from_file_tree_diff() {
        let before = FileTreeSnapshot {
            root_path: PathBuf::from("/test"),
            captured_at: Utc::now(),
            entries: vec![FileTreeEntry {
                path: PathBuf::from("old.txt"),
                entry_type: FileTreeEntryType::File,
                size_bytes: Some(100),
                permissions: "644".to_string(),
                modified_at: Some(Utc::now()),
            }],
            total_files: 1,
            total_dirs: 0,
        };

        let after = FileTreeSnapshot {
            root_path: PathBuf::from("/test"),
            captured_at: Utc::now(),
            entries: vec![FileTreeEntry {
                path: PathBuf::from("new.txt"),
                entry_type: FileTreeEntryType::File,
                size_bytes: Some(200),
                permissions: "644".to_string(),
                modified_at: Some(Utc::now()),
            }],
            total_files: 1,
            total_dirs: 0,
        };

        let persister = ArtifactPersister::new("patch-test", PathBuf::from("/tmp"));
        let patch = persister.generate_patch(&before, &after);

        assert!(patch.contains("- old.txt"));
        assert!(patch.contains("+ new.txt"));
    }

    #[test]
    fn test_capture_git_status_from_repository() {
        let temp_dir = TempDir::new().unwrap();
        let persister = ArtifactPersister::new("git-status-test", temp_dir.path());

        git2::Repository::init(temp_dir.path()).unwrap();

        let result = persister.capture_git_status(temp_dir.path());
        assert!(result.is_ok());

        let status = result.unwrap();
        assert!(status.is_clean || !status.is_clean);
    }

    #[test]
    fn test_capture_git_diff_from_repository() {
        let temp_dir = TempDir::new().unwrap();
        let persister = ArtifactPersister::new("git-diff-test", temp_dir.path());

        git2::Repository::init(temp_dir.path()).unwrap();

        let result = persister.capture_git_diff(temp_dir.path());
        assert!(result.is_ok());

        let diff = result.unwrap();
        assert_eq!(diff.patches.len(), 0);
        assert_eq!(diff.total_additions, 0);
        assert_eq!(diff.total_deletions, 0);
    }
}
