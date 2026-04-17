use chrono::Utc;
use opencode_core::runners::artifact_persister::{
    ArtifactPersister, DiffReport, FileTreeEntryType, MetadataJson, RunnerType,
};
use opencode_core::types::artifact::{Artifact, ArtifactKind};
use opencode_core::types::capability_summary::CapabilitySummary;
use opencode_core::types::failure_classification::FailureClassification;
use opencode_core::types::parity_verdict::{DiffCategory, ParityVerdict};
use opencode_core::types::runner_output::RunnerOutput;
use opencode_core::types::session_metadata::SessionMetadata;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_full_artifact_persistence_workflow_creates_all_expected_files() {
    let temp_dir = TempDir::new().unwrap();
    let run_id = "integration-test-001";
    let persister = ArtifactPersister::new(run_id, temp_dir.path());

    persister.create_directory_structure().unwrap();

    let legacy_stdout = "legacy stdout content - line 1\nline 2\nline 3";
    let legacy_stderr = "legacy stderr content - error log";
    let rust_stdout = "rust stdout content - line 1\nline 2\nline 3";
    let rust_stderr = "rust stderr content - error log";

    let legacy_stdout_path = persister
        .persist_stdout(legacy_stdout, RunnerType::Legacy)
        .unwrap();
    let legacy_stderr_path = persister
        .persist_stderr(legacy_stderr, RunnerType::Legacy)
        .unwrap();
    let rust_stdout_path = persister
        .persist_stdout(rust_stdout, RunnerType::Rust)
        .unwrap();
    let rust_stderr_path = persister
        .persist_stderr(rust_stderr, RunnerType::Rust)
        .unwrap();

    assert!(legacy_stdout_path.exists());
    assert!(legacy_stderr_path.exists());
    assert!(rust_stdout_path.exists());
    assert!(rust_stderr_path.exists());

    let legacy_metadata = MetadataJson {
        session_id: "session-legacy-001".to_string(),
        runner_name: "LegacyRunner".to_string(),
        task_id: "TASK-001".to_string(),
        exit_code: Some(0),
        duration_ms: 1500,
        artifacts_collected: 2,
        stdout_size_bytes: legacy_stdout.len(),
        stderr_size_bytes: legacy_stderr.len(),
        side_effects_count: 1,
        created_at: Utc::now(),
    };

    let rust_metadata = MetadataJson {
        session_id: "session-rust-001".to_string(),
        runner_name: "RustRunner".to_string(),
        task_id: "TASK-001".to_string(),
        exit_code: Some(0),
        duration_ms: 1200,
        artifacts_collected: 2,
        stdout_size_bytes: rust_stdout.len(),
        stderr_size_bytes: rust_stderr.len(),
        side_effects_count: 1,
        created_at: Utc::now(),
    };

    let legacy_metadata_path = persister
        .persist_metadata(&legacy_metadata, RunnerType::Legacy)
        .unwrap();
    let rust_metadata_path = persister
        .persist_metadata(&rust_metadata, RunnerType::Rust)
        .unwrap();

    assert!(legacy_metadata_path.exists());
    assert!(rust_metadata_path.exists());

    let legacy_artifacts = vec![
        Artifact::new("/tmp/legacy_output1.txt", ArtifactKind::File),
        Artifact::new("/tmp/legacy_dir", ArtifactKind::Directory),
    ];
    let rust_artifacts = vec![
        Artifact::new("/tmp/rust_output1.txt", ArtifactKind::File),
        Artifact::new("/tmp/rust_dir", ArtifactKind::Directory),
    ];

    let legacy_artifact_paths = persister
        .persist_artifacts(&legacy_artifacts, RunnerType::Legacy)
        .unwrap();
    let rust_artifact_paths = persister
        .persist_artifacts(&rust_artifacts, RunnerType::Rust)
        .unwrap();

    assert_eq!(legacy_artifact_paths.len(), 2);
    assert_eq!(rust_artifact_paths.len(), 2);

    let legacy_side_effects = vec![
        PathBuf::from("/tmp/legacy_side_effect_1"),
        PathBuf::from("/tmp/legacy_side_effect_2"),
    ];
    let rust_side_effects = vec![PathBuf::from("/tmp/rust_side_effect_1")];

    let legacy_side_effect_path = persister
        .persist_side_effects(&legacy_side_effects, RunnerType::Legacy)
        .unwrap();
    let rust_side_effect_path = persister
        .persist_side_effects(&rust_side_effects, RunnerType::Rust)
        .unwrap();

    assert!(legacy_side_effect_path.exists());
    assert!(rust_side_effect_path.exists());

    let legacy_session = SessionMetadata::new(
        "session-legacy-001".to_string(),
        "LegacyRunner".to_string(),
        "TASK-001".to_string(),
        Utc::now(),
        Utc::now(),
        PathBuf::from("/tmp/workspace"),
    );

    let rust_session = SessionMetadata::new(
        "session-rust-001".to_string(),
        "RustRunner".to_string(),
        "TASK-001".to_string(),
        Utc::now(),
        Utc::now(),
        PathBuf::from("/tmp/workspace"),
    );

    let legacy_output = RunnerOutput::new(
        Some(0),
        legacy_stdout.to_string(),
        legacy_stderr.to_string(),
        legacy_stdout_path.clone(),
        legacy_stderr_path.clone(),
        1500,
        legacy_artifacts,
        legacy_artifact_paths.clone(),
        legacy_session,
        legacy_stdout_path.parent().unwrap().join("event.log"),
        Some(legacy_side_effect_path),
        None,
        CapabilitySummary::default()
            .with_binary_available(true)
            .with_workspace_prepared(true)
            .with_artifacts_collected(2),
    );

    let rust_output = RunnerOutput::new(
        Some(0),
        rust_stdout.to_string(),
        rust_stderr.to_string(),
        rust_stdout_path.clone(),
        rust_stderr_path.clone(),
        1200,
        rust_artifacts,
        rust_artifact_paths.clone(),
        rust_session,
        rust_stdout_path.parent().unwrap().join("event.log"),
        Some(rust_side_effect_path),
        None,
        CapabilitySummary::default()
            .with_binary_available(true)
            .with_workspace_prepared(true)
            .with_artifacts_collected(2),
    );

    let verdict = ParityVerdict::Pass;

    let diff_report_path = persister
        .generate_diff_report(&legacy_output, &rust_output, &verdict)
        .unwrap();
    let verdict_md_path = persister
        .generate_verdict_md(&verdict, &legacy_output, &rust_output)
        .unwrap();

    assert!(diff_report_path.exists());
    assert!(verdict_md_path.exists());

    let diff_content = std::fs::read_to_string(&diff_report_path).unwrap();
    let diff_parsed: DiffReport = serde_json::from_str(&diff_content).unwrap();
    assert_eq!(diff_parsed.run_id, run_id);
    assert_eq!(diff_parsed.legacy_exit_code, Some(0));
    assert_eq!(diff_parsed.rust_exit_code, Some(0));

    let verdict_content = std::fs::read_to_string(&verdict_md_path).unwrap();
    assert!(verdict_content.contains("# Differential Verdict"));
    assert!(verdict_content.contains(run_id));

    let expected_dirs = vec![
        format!("{}/integration-test-001/legacy", temp_dir.path().display()),
        format!(
            "{}/integration-test-001/legacy/artifacts",
            temp_dir.path().display()
        ),
        format!(
            "{}/integration-test-001/legacy/side-effects",
            temp_dir.path().display()
        ),
        format!("{}/integration-test-001/rust", temp_dir.path().display()),
        format!(
            "{}/integration-test-001/rust/artifacts",
            temp_dir.path().display()
        ),
        format!(
            "{}/integration-test-001/rust/side-effects",
            temp_dir.path().display()
        ),
        format!("{}/integration-test-001/diff", temp_dir.path().display()),
    ];

    for dir in expected_dirs {
        assert!(
            std::path::Path::new(&dir).exists(),
            "Expected directory {} to exist",
            dir
        );
    }
}

#[test]
fn test_diff_report_with_different_category() {
    let temp_dir = TempDir::new().unwrap();
    let persister = ArtifactPersister::new("diff-test-002", temp_dir.path());

    persister.create_directory_structure().unwrap();

    let session = SessionMetadata::new(
        "session-001".to_string(),
        "TestRunner".to_string(),
        "TASK-001".to_string(),
        Utc::now(),
        Utc::now(),
        PathBuf::from("/tmp"),
    );

    let output = RunnerOutput::new(
        Some(0),
        "stdout".to_string(),
        "stderr".to_string(),
        PathBuf::from("/tmp/stdout.txt"),
        PathBuf::from("/tmp/stderr.txt"),
        1000,
        vec![],
        vec![],
        session,
        PathBuf::from("/tmp/event.log"),
        None,
        None,
        CapabilitySummary::default(),
    );

    let verdict = ParityVerdict::Fail {
        category: DiffCategory::OutputText,
        details: "Output differs".to_string(),
    };

    let report_path = persister
        .generate_diff_report(&output, &output, &verdict)
        .unwrap();
    let verdict_path = persister
        .generate_verdict_md(&verdict, &output, &output)
        .unwrap();

    assert!(report_path.exists());
    assert!(verdict_path.exists());

    let verdict_content = std::fs::read_to_string(&verdict_path).unwrap();
    assert!(verdict_content.contains("output text"));
}

#[test]
fn test_build_runner_output_integration() {
    let temp_dir = TempDir::new().unwrap();
    let persister = ArtifactPersister::new("build-output-test", temp_dir.path());

    persister.create_directory_structure().unwrap();

    let session = SessionMetadata::new(
        "session-build".to_string(),
        "LegacyRunner".to_string(),
        "BUILD-001".to_string(),
        Utc::now(),
        Utc::now(),
        PathBuf::from("/tmp/workspace"),
    );

    let artifacts = vec![
        Artifact::file("/tmp/output1.txt"),
        Artifact::directory("/tmp/output_dir"),
    ];

    let capability = CapabilitySummary::default()
        .with_binary_available(true)
        .with_workspace_prepared(true)
        .with_artifacts_collected(2)
        .with_timeout_enforced(true);

    let result = persister
        .build_runner_output(
            RunnerType::Legacy,
            Some(0),
            "test stdout".to_string(),
            "test stderr".to_string(),
            500,
            artifacts,
            session,
            None,
            Some(FailureClassification::ImplementationFailure),
            capability,
        )
        .unwrap();

    assert_eq!(result.exit_code, Some(0));
    assert_eq!(result.stdout, "test stdout");
    assert_eq!(result.stderr, "test stderr");
    assert!(result.stdout_path.exists());
    assert!(result.stderr_path.exists());
    assert_eq!(result.artifacts.len(), 2);
    assert_eq!(result.artifact_paths.len(), 2);
}

#[test]
fn test_capture_file_tree_produces_valid_snapshot() {
    let temp_dir = TempDir::new().unwrap();
    let persister = ArtifactPersister::new("file-tree-test", temp_dir.path());

    let test_dir = temp_dir.path().join("test_data");
    fs::create_dir_all(&test_dir).unwrap();

    fs::write(test_dir.join("file1.txt"), "content 1").unwrap();
    fs::write(test_dir.join("file2.txt"), "content 2").unwrap();
    fs::create_dir(test_dir.join("subdir")).unwrap();
    fs::write(test_dir.join("subdir/nested.txt"), "nested content").unwrap();

    let snapshot = persister.capture_file_tree(&test_dir).unwrap();

    assert_eq!(snapshot.root_path, test_dir);
    assert!(snapshot.captured_at <= Utc::now());
    assert!(snapshot.total_files >= 3);
    assert!(snapshot.total_dirs >= 1);

    let entry_paths: Vec<_> = snapshot.entries.iter().map(|e| e.path.clone()).collect();
    assert!(entry_paths.contains(&PathBuf::from("file1.txt")));
    assert!(entry_paths.contains(&PathBuf::from("file2.txt")));
    assert!(entry_paths.contains(&PathBuf::from("subdir")));
    assert!(entry_paths.contains(&PathBuf::from("subdir/nested.txt")));

    for entry in &snapshot.entries {
        if entry.path == PathBuf::from("file1.txt") {
            assert_eq!(entry.entry_type, FileTreeEntryType::File);
            assert!(entry.size_bytes.is_some());
            assert!(!entry.permissions.is_empty());
        }
        if entry.path == PathBuf::from("subdir") {
            assert_eq!(entry.entry_type, FileTreeEntryType::Directory);
            assert!(entry.size_bytes.is_none());
        }
    }
}

#[test]
fn test_diff_file_trees_identifies_added_removed_modified() {
    let temp_dir = TempDir::new().unwrap();
    let persister = ArtifactPersister::new("diff-tree-test", temp_dir.path());

    let before_dir = temp_dir.path().join("before");
    let after_dir = temp_dir.path().join("after");
    fs::create_dir_all(&before_dir).unwrap();
    fs::create_dir_all(&after_dir).unwrap();

    fs::write(before_dir.join("existing.txt"), "original content").unwrap();
    fs::write(before_dir.join("to_modify.txt"), "short").unwrap();

    fs::write(after_dir.join("existing.txt"), "original content").unwrap();
    fs::write(after_dir.join("to_modify.txt"), "new longer content").unwrap();
    fs::write(after_dir.join("new_file.txt"), "brand new").unwrap();

    let before_snapshot = persister.capture_file_tree(&before_dir).unwrap();
    let after_snapshot = persister.capture_file_tree(&after_dir).unwrap();

    let diff = persister.diff_file_trees(&before_snapshot, &after_snapshot);

    assert!(diff.added.iter().any(|p| p.as_os_str() == "new_file.txt"));
    assert!(diff.removed.is_empty());
    assert!(diff
        .modified
        .iter()
        .any(|p| p.as_os_str() == "to_modify.txt"));

    let existing_entry = after_snapshot
        .entries
        .iter()
        .find(|e| e.path == PathBuf::from("existing.txt"))
        .unwrap();
    assert_eq!(existing_entry.entry_type, FileTreeEntryType::File);
}
