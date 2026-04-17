use crate::runners::artifact_persister::{FileTreeDiff, FileTreeSnapshot, GitStatus};
use crate::types::parity_verdict::{DiffCategory, ParityVerdict};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedSideEffects {
    pub files_to_create: Vec<PathBuf>,
    pub files_to_modify: Vec<PathBuf>,
    pub files_to_delete: Vec<PathBuf>,
    pub protected_directories: Vec<PathBuf>,
}

impl ExpectedSideEffects {
    pub fn new(
        files_to_create: Vec<PathBuf>,
        files_to_modify: Vec<PathBuf>,
        files_to_delete: Vec<PathBuf>,
        protected_directories: Vec<PathBuf>,
    ) -> Self {
        Self {
            files_to_create,
            files_to_modify,
            files_to_delete,
            protected_directories,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SideEffectVerificationResult {
    pub verdict: ParityVerdict,
    pub expected_files_modified: Vec<PathBuf>,
    pub actual_files_modified: Vec<PathBuf>,
    pub unexpected_modifications: Vec<PathBuf>,
    pub missing_modifications: Vec<PathBuf>,
    pub protected_directory_violations: Vec<PathBuf>,
    pub git_state_inconsistency: Option<String>,
}

impl SideEffectVerificationResult {
    pub fn pass() -> Self {
        Self {
            verdict: ParityVerdict::Pass,
            expected_files_modified: Vec::new(),
            actual_files_modified: Vec::new(),
            unexpected_modifications: Vec::new(),
            missing_modifications: Vec::new(),
            protected_directory_violations: Vec::new(),
            git_state_inconsistency: None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fail(
        category: DiffCategory,
        details: String,
        expected_files_modified: Vec<PathBuf>,
        actual_files_modified: Vec<PathBuf>,
        unexpected_modifications: Vec<PathBuf>,
        missing_modifications: Vec<PathBuf>,
        protected_directory_violations: Vec<PathBuf>,
        git_state_inconsistency: Option<String>,
    ) -> Self {
        Self {
            verdict: ParityVerdict::Fail { category, details },
            expected_files_modified,
            actual_files_modified,
            unexpected_modifications,
            missing_modifications,
            protected_directory_violations,
            git_state_inconsistency,
        }
    }
}

pub trait SideEffectVerifier: Send + Sync {
    fn verify_side_effects(
        &self,
        expected: &ExpectedSideEffects,
        before: &FileTreeSnapshot,
        after: &FileTreeSnapshot,
        git_status: &GitStatus,
    ) -> SideEffectVerificationResult;
}

#[derive(Debug, Clone, Default)]
pub struct DefaultSideEffectVerifier;

impl DefaultSideEffectVerifier {
    pub fn new() -> Self {
        Self
    }
}

impl SideEffectVerifier for DefaultSideEffectVerifier {
    fn verify_side_effects(
        &self,
        expected: &ExpectedSideEffects,
        before: &FileTreeSnapshot,
        after: &FileTreeSnapshot,
        git_status: &GitStatus,
    ) -> SideEffectVerificationResult {
        let mut unexpected_modifications = Vec::new();
        let mut missing_modifications = Vec::new();
        let mut protected_directory_violations = Vec::new();
        let mut git_state_inconsistency = None;

        let diff = self.compute_file_tree_diff(before, after);

        let actual_files_modified: Vec<PathBuf> = diff
            .modified
            .iter()
            .cloned()
            .chain(diff.added.iter().cloned())
            .collect();

        let actual_files_deleted: Vec<PathBuf> = diff.removed.clone();

        for file in &expected.files_to_create {
            if !diff.added.contains(file) {
                missing_modifications.push(file.clone());
            }
        }

        for file in &expected.files_to_modify {
            if !diff.modified.contains(file) {
                missing_modifications.push(file.clone());
            }
        }

        for file in &expected.files_to_delete {
            if !diff.removed.contains(file) {
                missing_modifications.push(file.clone());
            }
        }

        let expected_set: std::collections::HashSet<_> = expected
            .files_to_create
            .iter()
            .chain(expected.files_to_modify.iter())
            .chain(expected.files_to_delete.iter())
            .collect();

        for modified in &actual_files_modified {
            if !expected_set.contains(modified) {
                unexpected_modifications.push(modified.clone());
            }
        }

        for deleted in &actual_files_deleted {
            if !expected.files_to_delete.contains(deleted) {
                unexpected_modifications.push(deleted.clone());
            }
        }

        for protected_dir in &expected.protected_directories {
            if self.is_protected_directory_violated(protected_dir, before, after) {
                protected_directory_violations.push(protected_dir.clone());
            }
        }

        if !git_status.is_clean {
            let changed_files =
                git_status.modified.len() + git_status.staged.len() + git_status.untracked.len();
            if changed_files > 0
                && expected.files_to_modify.is_empty()
                && expected.files_to_create.is_empty()
            {
                git_state_inconsistency = Some(format!(
                    "Git status shows {} file changes but no modifications expected",
                    changed_files
                ));
            }
        }

        let has_missing = !missing_modifications.is_empty();
        let has_unexpected = !unexpected_modifications.is_empty();
        let has_protected_violations = !protected_directory_violations.is_empty();
        let has_git_inconsistency = git_state_inconsistency.is_some();

        let verdict = if has_protected_violations {
            ParityVerdict::Fail {
                category: DiffCategory::SideEffects,
                details: format!(
                    "Protected directory violations: {:?}",
                    protected_directory_violations
                ),
            }
        } else if has_missing && has_unexpected {
            ParityVerdict::Fail {
                category: DiffCategory::SideEffects,
                details: format!(
                    "Missing: {:?}, Unexpected: {:?}",
                    missing_modifications, unexpected_modifications
                ),
            }
        } else if has_missing {
            ParityVerdict::Fail {
                category: DiffCategory::SideEffects,
                details: format!("Missing modifications: {:?}", missing_modifications),
            }
        } else if has_unexpected {
            ParityVerdict::Fail {
                category: DiffCategory::SideEffects,
                details: format!(
                    "Unexpected file modifications: {:?}",
                    unexpected_modifications
                ),
            }
        } else if has_git_inconsistency {
            ParityVerdict::Warn {
                category: DiffCategory::SideEffects,
                message: git_state_inconsistency.clone().unwrap(),
            }
        } else {
            ParityVerdict::Pass
        };

        let expected_files_modified: Vec<PathBuf> = expected
            .files_to_create
            .iter()
            .chain(expected.files_to_modify.iter())
            .cloned()
            .collect();

        SideEffectVerificationResult {
            verdict,
            expected_files_modified,
            actual_files_modified,
            unexpected_modifications,
            missing_modifications,
            protected_directory_violations,
            git_state_inconsistency,
        }
    }
}

impl DefaultSideEffectVerifier {
    fn compute_file_tree_diff(
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

    fn is_protected_directory_violated(
        &self,
        protected_dir: &PathBuf,
        before: &FileTreeSnapshot,
        after: &FileTreeSnapshot,
    ) -> bool {
        let before_entries: std::collections::HashSet<_> = before
            .entries
            .iter()
            .filter(|e| e.path.starts_with(protected_dir) || e.path == *protected_dir)
            .map(|e| e.path.clone())
            .collect();

        let after_entries: std::collections::HashSet<_> = after
            .entries
            .iter()
            .filter(|e| e.path.starts_with(protected_dir) || e.path == *protected_dir)
            .map(|e| e.path.clone())
            .collect();

        let removed: Vec<PathBuf> = before_entries.difference(&after_entries).cloned().collect();
        let added: Vec<PathBuf> = after_entries.difference(&before_entries).cloned().collect();

        let modified: Vec<PathBuf> = before_entries
            .intersection(&after_entries)
            .filter(|k| {
                let before_entry = before.entries.iter().find(|e| e.path == *k.as_path());
                let after_entry = after.entries.iter().find(|e| e.path == *k.as_path());
                match (before_entry, after_entry) {
                    (Some(b), Some(a)) => {
                        b.size_bytes != a.size_bytes || b.permissions != a.permissions
                    }
                    _ => false,
                }
            })
            .cloned()
            .collect();

        !removed.is_empty() || !added.is_empty() || !modified.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runners::artifact_persister::FileTreeEntry;
    use crate::runners::artifact_persister::FileTreeEntryType;
    use chrono::Utc;

    fn create_file_tree_snapshot(entries: Vec<FileTreeEntry>) -> FileTreeSnapshot {
        let total_files = entries
            .iter()
            .filter(|e| e.entry_type == FileTreeEntryType::File)
            .count();
        let total_dirs = entries
            .iter()
            .filter(|e| e.entry_type == FileTreeEntryType::Directory)
            .count();
        FileTreeSnapshot {
            root_path: PathBuf::from("/test"),
            captured_at: Utc::now(),
            entries,
            total_files,
            total_dirs,
        }
    }

    fn create_git_status(is_clean: bool) -> GitStatus {
        GitStatus {
            is_clean,
            staged: Vec::new(),
            modified: Vec::new(),
            untracked: Vec::new(),
            conflicted: Vec::new(),
        }
    }

    #[test]
    fn test_expected_side_effects_struct_fields() {
        let files_to_create = vec![PathBuf::from("new_file.txt")];
        let files_to_modify = vec![PathBuf::from("existing.txt")];
        let files_to_delete = vec![PathBuf::from("to_delete.txt")];
        let protected_directories = vec![PathBuf::from(".git")];

        let expected = ExpectedSideEffects::new(
            files_to_create.clone(),
            files_to_modify.clone(),
            files_to_delete.clone(),
            protected_directories.clone(),
        );

        assert_eq!(expected.files_to_create, files_to_create);
        assert_eq!(expected.files_to_modify, files_to_modify);
        assert_eq!(expected.files_to_delete, files_to_delete);
        assert_eq!(expected.protected_directories, protected_directories);
    }

    #[test]
    fn test_side_effect_verification_result_pass() {
        let result = SideEffectVerificationResult::pass();
        assert!(result.verdict.is_identical());
        assert!(result.expected_files_modified.is_empty());
        assert!(result.actual_files_modified.is_empty());
        assert!(result.unexpected_modifications.is_empty());
        assert!(result.missing_modifications.is_empty());
        assert!(result.protected_directory_violations.is_empty());
        assert!(result.git_state_inconsistency.is_none());
    }

    #[test]
    fn test_side_effect_verification_result_fail() {
        let result = SideEffectVerificationResult::fail(
            DiffCategory::SideEffects,
            "Test failure".to_string(),
            vec![PathBuf::from("expected.txt")],
            vec![PathBuf::from("actual.txt")],
            vec![PathBuf::from("unexpected.txt")],
            vec![PathBuf::from("missing.txt")],
            vec![PathBuf::from(".git")],
            Some("Git inconsistency".to_string()),
        );

        assert!(result.verdict.is_different());
        assert_eq!(
            result.expected_files_modified,
            vec![PathBuf::from("expected.txt")]
        );
        assert_eq!(
            result.actual_files_modified,
            vec![PathBuf::from("actual.txt")]
        );
        assert_eq!(
            result.unexpected_modifications,
            vec![PathBuf::from("unexpected.txt")]
        );
        assert_eq!(
            result.missing_modifications,
            vec![PathBuf::from("missing.txt")]
        );
        assert_eq!(
            result.protected_directory_violations,
            vec![PathBuf::from(".git")]
        );
        assert_eq!(
            result.git_state_inconsistency,
            Some("Git inconsistency".to_string())
        );
    }

    #[test]
    fn test_side_effect_verifier_trait_is_defined() {
        fn assert_side_effect_verifier<T: SideEffectVerifier>() {}
        assert_side_effect_verifier::<DefaultSideEffectVerifier>();
    }

    #[test]
    fn test_default_side_effect_verifier_detects_unexpected_file_modifications() {
        let verifier = DefaultSideEffectVerifier::new();

        let before_entries = vec![FileTreeEntry {
            path: PathBuf::from("target.txt"),
            entry_type: FileTreeEntryType::File,
            size_bytes: Some(100),
            permissions: "644".to_string(),
            modified_at: Some(Utc::now()),
        }];

        let after_entries = vec![
            FileTreeEntry {
                path: PathBuf::from("target.txt"),
                entry_type: FileTreeEntryType::File,
                size_bytes: Some(150),
                permissions: "644".to_string(),
                modified_at: Some(Utc::now()),
            },
            FileTreeEntry {
                path: PathBuf::from("unexpected.txt"),
                entry_type: FileTreeEntryType::File,
                size_bytes: Some(50),
                permissions: "644".to_string(),
                modified_at: Some(Utc::now()),
            },
        ];

        let before = create_file_tree_snapshot(before_entries);
        let after = create_file_tree_snapshot(after_entries);
        let git_status = create_git_status(true);

        let expected =
            ExpectedSideEffects::new(vec![], vec![PathBuf::from("target.txt")], vec![], vec![]);

        let result = verifier.verify_side_effects(&expected, &before, &after, &git_status);

        assert!(result.verdict.is_different());
        assert!(!result.unexpected_modifications.is_empty());
        assert!(result
            .unexpected_modifications
            .contains(&PathBuf::from("unexpected.txt")));
    }

    #[test]
    fn test_default_side_effect_verifier_detects_missing_modifications() {
        let verifier = DefaultSideEffectVerifier::new();

        let before_entries = vec![FileTreeEntry {
            path: PathBuf::from("target.txt"),
            entry_type: FileTreeEntryType::File,
            size_bytes: Some(100),
            permissions: "644".to_string(),
            modified_at: Some(Utc::now()),
        }];

        let after_entries = vec![FileTreeEntry {
            path: PathBuf::from("target.txt"),
            entry_type: FileTreeEntryType::File,
            size_bytes: Some(100),
            permissions: "644".to_string(),
            modified_at: Some(Utc::now()),
        }];

        let before = create_file_tree_snapshot(before_entries);
        let after = create_file_tree_snapshot(after_entries);
        let git_status = create_git_status(true);

        let expected =
            ExpectedSideEffects::new(vec![], vec![PathBuf::from("target.txt")], vec![], vec![]);

        let result = verifier.verify_side_effects(&expected, &before, &after, &git_status);

        assert!(result.verdict.is_different());
        assert!(!result.missing_modifications.is_empty());
        assert!(result
            .missing_modifications
            .contains(&PathBuf::from("target.txt")));
    }

    #[test]
    fn test_default_side_effect_verifier_detects_protected_directory_violations() {
        let verifier = DefaultSideEffectVerifier::new();

        let before_entries = vec![FileTreeEntry {
            path: PathBuf::from(".git"),
            entry_type: FileTreeEntryType::Directory,
            size_bytes: None,
            permissions: "755".to_string(),
            modified_at: Some(Utc::now()),
        }];

        let after_entries = vec![
            FileTreeEntry {
                path: PathBuf::from(".git"),
                entry_type: FileTreeEntryType::Directory,
                size_bytes: None,
                permissions: "777".to_string(),
                modified_at: Some(Utc::now()),
            },
            FileTreeEntry {
                path: PathBuf::from(".git"),
                entry_type: FileTreeEntryType::Directory,
                size_bytes: None,
                permissions: "777".to_string(),
                modified_at: Some(Utc::now()),
            },
        ];

        let before = create_file_tree_snapshot(before_entries);
        let after = create_file_tree_snapshot(after_entries);
        let git_status = create_git_status(true);

        let expected =
            ExpectedSideEffects::new(vec![], vec![], vec![], vec![PathBuf::from(".git")]);

        let result = verifier.verify_side_effects(&expected, &before, &after, &git_status);

        assert!(result.verdict.is_different());
        assert!(!result.protected_directory_violations.is_empty());
        assert!(result
            .protected_directory_violations
            .contains(&PathBuf::from(".git")));
    }

    #[test]
    fn test_default_side_effect_verifier_detects_git_state_inconsistency() {
        let verifier = DefaultSideEffectVerifier::new();

        let before_entries = vec![];
        let after_entries = vec![];

        let before = create_file_tree_snapshot(before_entries);
        let after = create_file_tree_snapshot(after_entries);

        let mut git_status = create_git_status(false);
        git_status
            .modified
            .push(crate::runners::artifact_persister::GitFileChange {
                path: "something.txt".to_string(),
                old_mode: None,
                new_mode: None,
            });

        let expected = ExpectedSideEffects::new(vec![], vec![], vec![], vec![]);

        let result = verifier.verify_side_effects(&expected, &before, &after, &git_status);

        assert!(result.verdict.is_uncertain() || result.git_state_inconsistency.is_some());
        assert!(result.git_state_inconsistency.is_some());
    }

    #[test]
    fn test_default_side_effect_verifier_passes_when_all_expected() {
        let verifier = DefaultSideEffectVerifier::new();

        let before_entries = vec![FileTreeEntry {
            path: PathBuf::from("target.txt"),
            entry_type: FileTreeEntryType::File,
            size_bytes: Some(100),
            permissions: "644".to_string(),
            modified_at: Some(Utc::now()),
        }];

        let after_entries = vec![FileTreeEntry {
            path: PathBuf::from("target.txt"),
            entry_type: FileTreeEntryType::File,
            size_bytes: Some(200),
            permissions: "644".to_string(),
            modified_at: Some(Utc::now()),
        }];

        let before = create_file_tree_snapshot(before_entries);
        let after = create_file_tree_snapshot(after_entries);
        let git_status = create_git_status(true);

        let expected =
            ExpectedSideEffects::new(vec![], vec![PathBuf::from("target.txt")], vec![], vec![]);

        let result = verifier.verify_side_effects(&expected, &before, &after, &git_status);

        assert!(result.verdict.is_identical());
        assert!(result.missing_modifications.is_empty());
        assert!(result.unexpected_modifications.is_empty());
    }

    #[test]
    fn test_default_side_effect_verifier_detects_new_file_creation() {
        let verifier = DefaultSideEffectVerifier::new();

        let before_entries = vec![];
        let after_entries = vec![FileTreeEntry {
            path: PathBuf::from("new_file.txt"),
            entry_type: FileTreeEntryType::File,
            size_bytes: Some(100),
            permissions: "644".to_string(),
            modified_at: Some(Utc::now()),
        }];

        let before = create_file_tree_snapshot(before_entries);
        let after = create_file_tree_snapshot(after_entries);
        let git_status = create_git_status(true);

        let expected =
            ExpectedSideEffects::new(vec![PathBuf::from("new_file.txt")], vec![], vec![], vec![]);

        let result = verifier.verify_side_effects(&expected, &before, &after, &git_status);

        assert!(result.verdict.is_identical());
        assert!(result.missing_modifications.is_empty());
    }

    #[test]
    fn test_default_side_effect_verifier_detects_file_deletion() {
        let verifier = DefaultSideEffectVerifier::new();

        let before_entries = vec![FileTreeEntry {
            path: PathBuf::from("to_delete.txt"),
            entry_type: FileTreeEntryType::File,
            size_bytes: Some(100),
            permissions: "644".to_string(),
            modified_at: Some(Utc::now()),
        }];

        let after_entries = vec![];

        let before = create_file_tree_snapshot(before_entries);
        let after = create_file_tree_snapshot(after_entries);
        let git_status = create_git_status(true);

        let expected =
            ExpectedSideEffects::new(vec![], vec![], vec![PathBuf::from("to_delete.txt")], vec![]);

        let result = verifier.verify_side_effects(&expected, &before, &after, &git_status);

        assert!(result.verdict.is_identical());
        assert!(result.missing_modifications.is_empty());
    }

    #[test]
    fn test_side_effect_verifier_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<DefaultSideEffectVerifier>();
    }

    #[test]
    fn side_effect_verifier_smoke_tests() {
        let verifier = DefaultSideEffectVerifier::new();

        let before_entries = vec![
            FileTreeEntry {
                path: PathBuf::from("src/main.rs"),
                entry_type: FileTreeEntryType::File,
                size_bytes: Some(100),
                permissions: "644".to_string(),
                modified_at: Some(Utc::now()),
            },
            FileTreeEntry {
                path: PathBuf::from("src/lib.rs"),
                entry_type: FileTreeEntryType::File,
                size_bytes: Some(200),
                permissions: "644".to_string(),
                modified_at: Some(Utc::now()),
            },
            FileTreeEntry {
                path: PathBuf::from("target.txt"),
                entry_type: FileTreeEntryType::File,
                size_bytes: Some(300),
                permissions: "644".to_string(),
                modified_at: Some(Utc::now()),
            },
            FileTreeEntry {
                path: PathBuf::from(".git"),
                entry_type: FileTreeEntryType::Directory,
                size_bytes: None,
                permissions: "755".to_string(),
                modified_at: Some(Utc::now()),
            },
        ];

        let after_entries = vec![
            FileTreeEntry {
                path: PathBuf::from("src/main.rs"),
                entry_type: FileTreeEntryType::File,
                size_bytes: Some(150),
                permissions: "644".to_string(),
                modified_at: Some(Utc::now()),
            },
            FileTreeEntry {
                path: PathBuf::from("src/lib.rs"),
                entry_type: FileTreeEntryType::File,
                size_bytes: Some(250),
                permissions: "644".to_string(),
                modified_at: Some(Utc::now()),
            },
            FileTreeEntry {
                path: PathBuf::from("new_file.txt"),
                entry_type: FileTreeEntryType::File,
                size_bytes: Some(100),
                permissions: "644".to_string(),
                modified_at: Some(Utc::now()),
            },
            FileTreeEntry {
                path: PathBuf::from(".git"),
                entry_type: FileTreeEntryType::Directory,
                size_bytes: None,
                permissions: "755".to_string(),
                modified_at: Some(Utc::now()),
            },
        ];

        let before = create_file_tree_snapshot(before_entries);
        let after = create_file_tree_snapshot(after_entries);
        let git_status = create_git_status(true);

        let expected = ExpectedSideEffects::new(
            vec![PathBuf::from("new_file.txt")],
            vec![PathBuf::from("src/main.rs"), PathBuf::from("src/lib.rs")],
            vec![PathBuf::from("target.txt")],
            vec![PathBuf::from(".git")],
        );

        let result = verifier.verify_side_effects(&expected, &before, &after, &git_status);

        assert!(result.verdict.is_identical());
        assert_eq!(result.expected_files_modified.len(), 3);
        assert!(result.unexpected_modifications.is_empty());
        assert!(result.missing_modifications.is_empty());
        assert!(result.protected_directory_violations.is_empty());
        assert!(result.git_state_inconsistency.is_none());

        fn assert_side_effect_verifier<T: SideEffectVerifier>() {}
        assert_side_effect_verifier::<DefaultSideEffectVerifier>();
    }
}
