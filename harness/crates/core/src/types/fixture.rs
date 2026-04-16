use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixtureProject {
    pub name: String,
    pub description: String,
    pub workspace_policy: WorkspacePolicy,
    pub reset_strategy: ResetStrategy,
    pub setup_script: Option<String>,
    pub teardown_script: Option<String>,
    pub files: Vec<FixtureFile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspacePolicy {
    pub allow_dirty_git: bool,
    pub allow_network: bool,
    pub preserve_on_failure: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResetStrategy {
    None,
    CleanClone,
    RestoreFiles,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixtureFile {
    pub path: String,
    pub content: String,
    pub executable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    pub id: String,
    pub path: PathBuf,
    pub fixture_name: String,
}

impl FixtureProject {
    pub fn new(
        name: String,
        description: String,
        workspace_policy: WorkspacePolicy,
        reset_strategy: ResetStrategy,
    ) -> Self {
        Self {
            name,
            description,
            workspace_policy,
            reset_strategy,
            setup_script: None,
            teardown_script: None,
            files: Vec::new(),
        }
    }

    pub fn with_setup_script(mut self, script: String) -> Self {
        self.setup_script = Some(script);
        self
    }

    pub fn with_teardown_script(mut self, script: String) -> Self {
        self.teardown_script = Some(script);
        self
    }

    pub fn with_files(mut self, files: Vec<FixtureFile>) -> Self {
        self.files = files;
        self
    }
}

impl Workspace {
    pub fn new(id: String, path: PathBuf, fixture_name: String) -> Self {
        Self {
            id,
            path,
            fixture_name,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixture_project_creation() {
        let policy = WorkspacePolicy {
            allow_dirty_git: false,
            allow_network: false,
            preserve_on_failure: false,
        };
        let fixture = FixtureProject::new(
            "cli-basic".to_string(),
            "Basic CLI smoke test fixture".to_string(),
            policy,
            ResetStrategy::CleanClone,
        );
        assert_eq!(fixture.name, "cli-basic");
        assert_eq!(fixture.reset_strategy, ResetStrategy::CleanClone);
    }

    #[test]
    fn test_workspace_creation() {
        let workspace = Workspace::new(
            "ws-001".to_string(),
            PathBuf::from("/tmp/workspace"),
            "cli-basic".to_string(),
        );
        assert_eq!(workspace.fixture_name, "cli-basic");
        assert!(workspace.path.exists() || !workspace.path.as_os_str().is_empty());
    }

    #[test]
    fn test_reset_strategy_variants() {
        assert_eq!(
            serde_json::to_string(&ResetStrategy::None).unwrap(),
            "\"none\""
        );
        assert_eq!(
            serde_json::to_string(&ResetStrategy::CleanClone).unwrap(),
            "\"clean_clone\""
        );
        assert_eq!(
            serde_json::to_string(&ResetStrategy::RestoreFiles).unwrap(),
            "\"restore_files\""
        );
    }

    #[test]
    fn test_workspace_policy_default() {
        let policy = WorkspacePolicy {
            allow_dirty_git: false,
            allow_network: false,
            preserve_on_failure: false,
        };
        assert!(!policy.allow_dirty_git);
        assert!(!policy.allow_network);
        assert!(!policy.preserve_on_failure);
    }
}
