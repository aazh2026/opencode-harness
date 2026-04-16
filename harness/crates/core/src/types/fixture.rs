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
    pub configs: Vec<FixtureConfig>,
    pub transcripts: Vec<Transcript>,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixtureConfig {
    pub path: String,
    pub format: ConfigFormat,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigFormat {
    Toml,
    Json,
    Yaml,
    Env,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transcript {
    pub path: String,
    pub transcript_type: TranscriptType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscriptType {
    Recording,
    Transcript,
    Snapshot,
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
            configs: Vec::new(),
            transcripts: Vec::new(),
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

    pub fn with_configs(mut self, configs: Vec<FixtureConfig>) -> Self {
        self.configs = configs;
        self
    }

    pub fn with_transcripts(mut self, transcripts: Vec<Transcript>) -> Self {
        self.transcripts = transcripts;
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
    fn test_fixture_project_has_all_required_fields() {
        let policy = WorkspacePolicy {
            allow_dirty_git: false,
            allow_network: false,
            preserve_on_failure: false,
        };
        let fixture = FixtureProject::new(
            "cli-basic".to_string(),
            "Basic CLI smoke test fixture".to_string(),
            policy.clone(),
            ResetStrategy::CleanClone,
        );
        assert_eq!(fixture.name, "cli-basic");
        assert_eq!(fixture.description, "Basic CLI smoke test fixture");
        assert_eq!(fixture.workspace_policy, policy);
        assert_eq!(fixture.reset_strategy, ResetStrategy::CleanClone);
        assert!(fixture.setup_script.is_none());
        assert!(fixture.teardown_script.is_none());
        assert!(fixture.files.is_empty());
        assert!(fixture.configs.is_empty());
        assert!(fixture.transcripts.is_empty());
    }

    #[test]
    fn test_fixture_project_with_workspace_policy_and_reset_strategy() {
        let policy = WorkspacePolicy {
            allow_dirty_git: true,
            allow_network: true,
            preserve_on_failure: true,
        };
        let fixture = FixtureProject::new(
            "api-project".to_string(),
            "API test fixture".to_string(),
            policy.clone(),
            ResetStrategy::RestoreFiles,
        );
        assert!(fixture.workspace_policy.allow_dirty_git);
        assert!(fixture.workspace_policy.allow_network);
        assert!(fixture.workspace_policy.preserve_on_failure);
        assert_eq!(fixture.reset_strategy, ResetStrategy::RestoreFiles);
    }

    #[test]
    fn test_fixture_project_serde_roundtrip() {
        let policy = WorkspacePolicy {
            allow_dirty_git: false,
            allow_network: true,
            preserve_on_failure: false,
        };
        let fixture = FixtureProject::new(
            "cli-basic".to_string(),
            "Basic CLI smoke test fixture".to_string(),
            policy,
            ResetStrategy::CleanClone,
        )
        .with_setup_script("scripts/setup.sh".to_string())
        .with_files(vec![FixtureFile {
            path: "src/main.rs".to_string(),
            content: "fn main() {}".to_string(),
            executable: true,
        }])
        .with_configs(vec![FixtureConfig {
            path: "config.toml".to_string(),
            format: ConfigFormat::Toml,
        }])
        .with_transcripts(vec![Transcript {
            path: "recording.json".to_string(),
            transcript_type: TranscriptType::Recording,
        }]);

        let serialized = serde_json::to_string(&fixture).expect("serialization should succeed");
        let deserialized: FixtureProject =
            serde_json::from_str(&serialized).expect("deserialization should succeed");

        assert_eq!(fixture.name, deserialized.name);
        assert_eq!(fixture.description, deserialized.description);
        assert_eq!(fixture.workspace_policy, deserialized.workspace_policy);
        assert_eq!(fixture.reset_strategy, deserialized.reset_strategy);
        assert_eq!(fixture.setup_script, deserialized.setup_script);
        assert_eq!(fixture.files, deserialized.files);
        assert_eq!(fixture.configs, deserialized.configs);
        assert_eq!(fixture.transcripts, deserialized.transcripts);
    }

    #[test]
    fn test_fixture_project_json_format() {
        let policy = WorkspacePolicy {
            allow_dirty_git: false,
            allow_network: false,
            preserve_on_failure: false,
        };
        let fixture = FixtureProject::new(
            "test-fixture".to_string(),
            "Test fixture".to_string(),
            policy,
            ResetStrategy::None,
        );

        let serialized = serde_json::to_string(&fixture).expect("serialization should succeed");
        assert!(serialized.contains("\"name\":\"test-fixture\""));
        assert!(serialized.contains("\"description\":\"Test fixture\""));
        assert!(serialized.contains("\"reset_strategy\":\"none\""));
    }

    #[test]
    fn test_config_format_serde() {
        assert_eq!(
            serde_json::to_string(&ConfigFormat::Toml).unwrap(),
            "\"toml\""
        );
        assert_eq!(
            serde_json::to_string(&ConfigFormat::Json).unwrap(),
            "\"json\""
        );
        assert_eq!(
            serde_json::to_string(&ConfigFormat::Yaml).unwrap(),
            "\"yaml\""
        );
        assert_eq!(
            serde_json::to_string(&ConfigFormat::Env).unwrap(),
            "\"env\""
        );
    }

    #[test]
    fn test_transcript_type_serde() {
        assert_eq!(
            serde_json::to_string(&TranscriptType::Recording).unwrap(),
            "\"recording\""
        );
        assert_eq!(
            serde_json::to_string(&TranscriptType::Transcript).unwrap(),
            "\"transcript\""
        );
        assert_eq!(
            serde_json::to_string(&TranscriptType::Snapshot).unwrap(),
            "\"snapshot\""
        );
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
