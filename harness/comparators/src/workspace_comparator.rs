use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::{Comparator, ComparisonOutcome, ComparisonResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTreeEntry {
    pub path: String,
    pub entry_type: String,
    pub size_bytes: Option<u64>,
    pub permissions: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTreeSnapshotData {
    pub root_path: String,
    pub entries: Vec<FileTreeEntry>,
    pub total_files: usize,
    pub total_dirs: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitFileChange {
    pub path: String,
    pub old_mode: Option<String>,
    pub new_mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatusData {
    pub is_clean: bool,
    pub staged: Vec<GitFileChange>,
    pub modified: Vec<GitFileChange>,
    pub untracked: Vec<GitFileChange>,
    pub conflicted: Vec<GitFileChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTreeDiff {
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub modified: Vec<String>,
    pub unchanged_count: usize,
}

impl FileTreeSnapshotData {
    pub fn parse_from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn paths(&self) -> HashSet<&str> {
        self.entries.iter().map(|e| e.path.as_str()).collect()
    }

    pub fn entry_map(&self) -> HashMap<&str, &FileTreeEntry> {
        self.entries.iter().map(|e| (e.path.as_str(), e)).collect()
    }
}

impl GitStatusData {
    pub fn parse_from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn changed_files_count(&self) -> usize {
        self.staged.len() + self.modified.len() + self.untracked.len() + self.conflicted.len()
    }
}

#[derive(Debug, Clone)]
pub struct FileTreeComparator {
    ignore_directories: Vec<String>,
}

impl FileTreeComparator {
    pub fn new() -> Self {
        Self {
            ignore_directories: vec![".git".to_string(), "node_modules".to_string()],
        }
    }

    pub fn with_ignored_directories(mut self, dirs: Vec<String>) -> Self {
        self.ignore_directories = dirs;
        self
    }

    fn should_ignore(&self, path: &str) -> bool {
        for ignored in &self.ignore_directories {
            if path.contains(ignored) {
                return true;
            }
        }
        false
    }

    pub fn compare_trees(
        &self,
        tree1: &FileTreeSnapshotData,
        tree2: &FileTreeSnapshotData,
    ) -> FileTreeDiff {
        let entries1: HashMap<_, _> = tree1
            .entries
            .iter()
            .filter(|e| !self.should_ignore(&e.path))
            .map(|e| (e.path.clone(), e))
            .collect();

        let entries2: HashMap<_, _> = tree2
            .entries
            .iter()
            .filter(|e| !self.should_ignore(&e.path))
            .map(|e| (e.path.clone(), e))
            .collect();

        let keys1: HashSet<_> = entries1.keys().cloned().collect();
        let keys2: HashSet<_> = entries2.keys().cloned().collect();

        let removed: Vec<String> = keys1.difference(&keys2).cloned().collect();
        let added: Vec<String> = keys2.difference(&keys1).cloned().collect();

        let unchanged_count = keys1
            .intersection(&keys2)
            .filter(|k| {
                let e1 = entries1.get(*k).unwrap();
                let e2 = entries2.get(*k).unwrap();
                e1.entry_type == e2.entry_type
                    && e1.size_bytes == e2.size_bytes
                    && e1.permissions == e2.permissions
            })
            .count();

        let modified: Vec<String> = keys1
            .intersection(&keys2)
            .filter(|k| {
                let e1 = entries1.get(*k).unwrap();
                let e2 = entries2.get(*k).unwrap();
                e1.entry_type != e2.entry_type
                    || e1.size_bytes != e2.size_bytes
                    || e1.permissions != e2.permissions
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

    pub fn compare(&self, json1: &str, json2: &str) -> ComparisonResult {
        let tree1 = match FileTreeSnapshotData::parse_from_json(json1) {
            Ok(t) => t,
            Err(e) => {
                return ComparisonResult::incomparable(format!("Failed to parse tree1: {}", e))
            }
        };

        let tree2 = match FileTreeSnapshotData::parse_from_json(json2) {
            Ok(t) => t,
            Err(e) => {
                return ComparisonResult::incomparable(format!("Failed to parse tree2: {}", e))
            }
        };

        let diff = self.compare_trees(&tree1, &tree2);

        if diff.added.is_empty() && diff.removed.is_empty() && diff.modified.is_empty() {
            return ComparisonResult::strongly_equivalent();
        }

        let diff_summary = format!(
            "Tree diff: {} added, {} removed, {} modified, {} unchanged",
            diff.added.len(),
            diff.removed.len(),
            diff.modified.len(),
            diff.unchanged_count
        );

        if diff.added.is_empty() && diff.removed.is_empty() && !diff.modified.is_empty() {
            return ComparisonResult::mildly_incompatible(diff_summary);
        }

        ComparisonResult::severely_incompatible(diff_summary)
    }
}

impl Default for FileTreeComparator {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct GitStatusComparator {
    strict_mode: bool,
}

impl GitStatusComparator {
    pub fn new() -> Self {
        Self { strict_mode: true }
    }

    pub fn with_strict_mode(mut self, strict: bool) -> Self {
        self.strict_mode = strict;
        self
    }

    pub fn compare_statuses(
        &self,
        status1: &GitStatusData,
        status2: &GitStatusData,
    ) -> ComparisonResult {
        if status1.is_clean && status2.is_clean {
            return ComparisonResult::strongly_equivalent();
        }

        if status1.is_clean != status2.is_clean {
            return ComparisonResult::mildly_incompatible(format!(
                "Clean status differs: {} vs {}",
                status1.is_clean, status2.is_clean
            ));
        }

        let changes1 = status1.changed_files_count();
        let changes2 = status2.changed_files_count();

        if changes1 != changes2 {
            return ComparisonResult::mildly_incompatible(format!(
                "Changed file count differs: {} vs {}",
                changes1, changes2
            ));
        }

        let staged_diff = self.compare_file_lists(&status1.staged, &status2.staged);
        if let Some(diff) = staged_diff {
            return ComparisonResult::mildly_incompatible(format!("Staged files differ: {}", diff));
        }

        let modified_diff = self.compare_file_lists(&status1.modified, &status2.modified);
        if let Some(diff) = modified_diff {
            return ComparisonResult::mildly_incompatible(format!(
                "Modified files differ: {}",
                diff
            ));
        }

        let untracked_diff = self.compare_file_lists(&status1.untracked, &status2.untracked);
        if let Some(diff) = untracked_diff {
            if self.strict_mode {
                return ComparisonResult::mildly_incompatible(format!(
                    "Untracked files differ: {}",
                    diff
                ));
            }
            return ComparisonResult::allowed_variance(format!(
                "Untracked files differ (non-strict): {}",
                diff
            ));
        }

        ComparisonResult::strongly_equivalent()
    }

    fn compare_file_lists(
        &self,
        list1: &[GitFileChange],
        list2: &[GitFileChange],
    ) -> Option<String> {
        let paths1: HashSet<_> = list1.iter().map(|f| f.path.as_str()).collect();
        let paths2: HashSet<_> = list2.iter().map(|f| f.path.as_str()).collect();

        if paths1 != paths2 {
            let only_in_1: Vec<_> = paths1.difference(&paths2).collect();
            let only_in_2: Vec<_> = paths2.difference(&paths1).collect();
            return Some(format!(
                "only in 1: {:?}, only in 2: {:?}",
                only_in_1, only_in_2
            ));
        }
        None
    }

    pub fn compare(&self, json1: &str, json2: &str) -> ComparisonResult {
        let status1 = match GitStatusData::parse_from_json(json1) {
            Ok(s) => s,
            Err(e) => {
                return ComparisonResult::incomparable(format!("Failed to parse status1: {}", e))
            }
        };

        let status2 = match GitStatusData::parse_from_json(json2) {
            Ok(s) => s,
            Err(e) => {
                return ComparisonResult::incomparable(format!("Failed to parse status2: {}", e))
            }
        };

        self.compare_statuses(&status1, &status2)
    }
}

impl Default for GitStatusComparator {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct WorkspaceComparator {
    file_tree_comparator: FileTreeComparator,
    git_comparator: GitStatusComparator,
}

impl WorkspaceComparator {
    pub fn new(
        file_tree_comparator: FileTreeComparator,
        git_comparator: GitStatusComparator,
    ) -> Self {
        Self {
            file_tree_comparator,
            git_comparator,
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(FileTreeComparator::new(), GitStatusComparator::new())
    }

    pub fn compare_file_trees(&self, json1: &str, json2: &str) -> ComparisonResult {
        self.file_tree_comparator.compare(json1, json2)
    }

    pub fn compare_git_statuses(&self, json1: &str, json2: &str) -> ComparisonResult {
        self.git_comparator.compare(json1, json2)
    }

    pub fn compare_workspaces(
        &self,
        tree1_json: &str,
        tree2_json: &str,
        git1_json: &str,
        git2_json: &str,
    ) -> ComparisonResult {
        let tree_result = self.file_tree_comparator.compare(tree1_json, tree2_json);
        let git_result = self.git_comparator.compare(git1_json, git2_json);

        if tree_result.outcome == ComparisonOutcome::StronglyEquivalent
            && git_result.outcome == ComparisonOutcome::StronglyEquivalent
        {
            return ComparisonResult::strongly_equivalent();
        }

        if tree_result.outcome == ComparisonOutcome::SeverelyIncompatible
            || git_result.outcome == ComparisonOutcome::SeverelyIncompatible
        {
            return ComparisonResult::severely_incompatible(format!(
                "Tree: {}, Git: {}",
                tree_result.diff.unwrap_or_default(),
                git_result.diff.unwrap_or_default()
            ));
        }

        ComparisonResult::mildly_incompatible(format!(
            "Tree: {}, Git: {}",
            tree_result.diff.unwrap_or_default(),
            git_result.diff.unwrap_or_default()
        ))
    }
}

impl Comparator for WorkspaceComparator {
    fn compare(&self, output1: &str, output2: &str) -> ComparisonResult {
        if FileTreeSnapshotData::parse_from_json(output1).is_ok()
            && FileTreeSnapshotData::parse_from_json(output2).is_ok()
        {
            return self.file_tree_comparator.compare(output1, output2);
        }

        if GitStatusData::parse_from_json(output1).is_ok()
            && GitStatusData::parse_from_json(output2).is_ok()
        {
            return self.git_comparator.compare(output1, output2);
        }

        ComparisonResult::incomparable(
            "Could not parse output as FileTreeSnapshot or GitStatus".to_string(),
        )
    }

    fn name(&self) -> &str {
        "workspace"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_comparator<T: Comparator>() {}

    #[test]
    fn workspace_comparator_implements_comparator_trait() {
        assert_comparator::<WorkspaceComparator>();
    }

    #[test]
    fn workspace_comparator_default_config() {
        let comparator = WorkspaceComparator::with_default_config();
        assert_eq!(comparator.name(), "workspace");
    }

    #[test]
    fn file_tree_comparator_identical_trees() {
        let comparator = FileTreeComparator::new();
        let tree = r#"{
            "root_path": "/test",
            "entries": [
                {"path": "file1.txt", "entry_type": "File", "size_bytes": 100, "permissions": "644"},
                {"path": "file2.txt", "entry_type": "File", "size_bytes": 200, "permissions": "644"}
            ],
            "total_files": 2,
            "total_dirs": 0
        }"#;

        let result = comparator.compare(tree, tree);
        assert_eq!(result.outcome, ComparisonOutcome::StronglyEquivalent);
    }

    #[test]
    fn file_tree_comparator_modified_file() {
        let comparator = FileTreeComparator::new();
        let tree1 = r#"{
            "root_path": "/test",
            "entries": [
                {"path": "file1.txt", "entry_type": "File", "size_bytes": 100, "permissions": "644"}
            ],
            "total_files": 1,
            "total_dirs": 0
        }"#;
        let tree2 = r#"{
            "root_path": "/test",
            "entries": [
                {"path": "file1.txt", "entry_type": "File", "size_bytes": 200, "permissions": "644"}
            ],
            "total_files": 1,
            "total_dirs": 0
        }"#;

        let result = comparator.compare(tree1, tree2);
        assert_eq!(result.outcome, ComparisonOutcome::MildlyIncompatible);
    }

    #[test]
    fn file_tree_comparator_added_files() {
        let comparator = FileTreeComparator::new();
        let tree1 = r#"{
            "root_path": "/test",
            "entries": [],
            "total_files": 0,
            "total_dirs": 0
        }"#;
        let tree2 = r#"{
            "root_path": "/test",
            "entries": [
                {"path": "new_file.txt", "entry_type": "File", "size_bytes": 100, "permissions": "644"}
            ],
            "total_files": 1,
            "total_dirs": 0
        }"#;

        let result = comparator.compare(tree1, tree2);
        assert_eq!(result.outcome, ComparisonOutcome::SeverelyIncompatible);
    }

    #[test]
    fn file_tree_comparator_ignores_git_directory() {
        let comparator = FileTreeComparator::new();
        let tree1 = r#"{
            "root_path": "/test",
            "entries": [
                {"path": ".git/config", "entry_type": "File", "size_bytes": 100, "permissions": "644"}
            ],
            "total_files": 1,
            "total_dirs": 0
        }"#;
        let tree2 = r#"{
            "root_path": "/test",
            "entries": [],
            "total_files": 0,
            "total_dirs": 0
        }"#;

        let result = comparator.compare(tree1, tree2);
        assert_eq!(result.outcome, ComparisonOutcome::StronglyEquivalent);
    }

    #[test]
    fn file_tree_comparator_invalid_json() {
        let comparator = FileTreeComparator::new();
        let result = comparator.compare("not json", "{}");
        assert_eq!(result.outcome, ComparisonOutcome::Incomparable);
    }

    #[test]
    fn git_status_comparator_both_clean() {
        let comparator = GitStatusComparator::new();
        let status = r#"{
            "is_clean": true,
            "staged": [],
            "modified": [],
            "untracked": [],
            "conflicted": []
        }"#;

        let result = comparator.compare(status, status);
        assert_eq!(result.outcome, ComparisonOutcome::StronglyEquivalent);
    }

    #[test]
    fn git_status_comparator_clean_vs_dirty() {
        let comparator = GitStatusComparator::new();
        let clean = r#"{
            "is_clean": true,
            "staged": [],
            "modified": [],
            "untracked": [],
            "conflicted": []
        }"#;
        let dirty = r#"{
            "is_clean": false,
            "staged": [],
            "modified": [{"path": "file.txt", "old_mode": null, "new_mode": null}],
            "untracked": [],
            "conflicted": []
        }"#;

        let result = comparator.compare(clean, dirty);
        assert_eq!(result.outcome, ComparisonOutcome::MildlyIncompatible);
    }

    #[test]
    fn git_status_comparator_same_changes() {
        let comparator = GitStatusComparator::new();
        let status1 = r#"{
            "is_clean": false,
            "staged": [{"path": "file1.txt", "old_mode": null, "new_mode": null}],
            "modified": [],
            "untracked": [],
            "conflicted": []
        }"#;
        let status2 = r#"{
            "is_clean": false,
            "staged": [{"path": "file1.txt", "old_mode": null, "new_mode": null}],
            "modified": [],
            "untracked": [],
            "conflicted": []
        }"#;

        let result = comparator.compare(status1, status2);
        assert_eq!(result.outcome, ComparisonOutcome::StronglyEquivalent);
    }

    #[test]
    fn git_status_comparator_different_staged() {
        let comparator = GitStatusComparator::new();
        let status1 = r#"{
            "is_clean": false,
            "staged": [{"path": "file1.txt", "old_mode": null, "new_mode": null}],
            "modified": [],
            "untracked": [],
            "conflicted": []
        }"#;
        let status2 = r#"{
            "is_clean": false,
            "staged": [{"path": "file2.txt", "old_mode": null, "new_mode": null}],
            "modified": [],
            "untracked": [],
            "conflicted": []
        }"#;

        let result = comparator.compare(status1, status2);
        assert_eq!(result.outcome, ComparisonOutcome::MildlyIncompatible);
    }

    #[test]
    fn git_status_comparator_invalid_json() {
        let comparator = GitStatusComparator::new();
        let result = comparator.compare("not json", "{}");
        assert_eq!(result.outcome, ComparisonOutcome::Incomparable);
    }

    #[test]
    fn workspace_comparator_parses_as_file_tree() {
        let comparator = WorkspaceComparator::with_default_config();
        let tree = r#"{
            "root_path": "/test",
            "entries": [{"path": "file.txt", "entry_type": "File", "size_bytes": 100, "permissions": "644"}],
            "total_files": 1,
            "total_dirs": 0
        }"#;

        let result = comparator.compare(tree, tree);
        assert_eq!(result.outcome, ComparisonOutcome::StronglyEquivalent);
    }

    #[test]
    fn workspace_comparator_parses_as_git_status() {
        let comparator = WorkspaceComparator::with_default_config();
        let status = r#"{
            "is_clean": true,
            "staged": [],
            "modified": [],
            "untracked": [],
            "conflicted": []
        }"#;

        let result = comparator.compare(status, status);
        assert_eq!(result.outcome, ComparisonOutcome::StronglyEquivalent);
    }

    #[test]
    fn workspace_comparator_incomparable_for_unknown_format() {
        let comparator = WorkspaceComparator::with_default_config();
        let result = comparator.compare("not a tree or status", "also not");
        assert_eq!(result.outcome, ComparisonOutcome::Incomparable);
    }

    #[test]
    fn workspace_comparator_compare_file_trees() {
        let comparator = WorkspaceComparator::with_default_config();
        let tree = r#"{
            "root_path": "/test",
            "entries": [],
            "total_files": 0,
            "total_dirs": 0
        }"#;

        let result = comparator.compare_file_trees(tree, tree);
        assert_eq!(result.outcome, ComparisonOutcome::StronglyEquivalent);
    }

    #[test]
    fn workspace_comparator_compare_git_statuses() {
        let comparator = WorkspaceComparator::with_default_config();
        let status = r#"{
            "is_clean": true,
            "staged": [],
            "modified": [],
            "untracked": [],
            "conflicted": []
        }"#;

        let result = comparator.compare_git_statuses(status, status);
        assert_eq!(result.outcome, ComparisonOutcome::StronglyEquivalent);
    }

    #[test]
    fn workspace_comparator_compare_workspaces() {
        let comparator = WorkspaceComparator::with_default_config();
        let tree = r#"{
            "root_path": "/test",
            "entries": [],
            "total_files": 0,
            "total_dirs": 0
        }"#;
        let status = r#"{
            "is_clean": true,
            "staged": [],
            "modified": [],
            "untracked": [],
            "conflicted": []
        }"#;

        let result = comparator.compare_workspaces(tree, tree, status, status);
        assert_eq!(result.outcome, ComparisonOutcome::StronglyEquivalent);
    }
}
