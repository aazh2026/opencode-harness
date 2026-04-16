use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub session_id: String,
    pub runner_name: String,
    pub task_id: String,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
    pub working_directory: PathBuf,
}

impl SessionMetadata {
    pub fn new(
        session_id: String,
        runner_name: String,
        task_id: String,
        started_at: DateTime<Utc>,
        finished_at: DateTime<Utc>,
        working_directory: PathBuf,
    ) -> Self {
        Self {
            session_id,
            runner_name,
            task_id,
            started_at,
            finished_at,
            working_directory,
        }
    }

    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.session_id = session_id;
        self
    }

    pub fn with_runner_name(mut self, runner_name: String) -> Self {
        self.runner_name = runner_name;
        self
    }

    pub fn with_task_id(mut self, task_id: String) -> Self {
        self.task_id = task_id;
        self
    }

    pub fn with_started_at(mut self, started_at: DateTime<Utc>) -> Self {
        self.started_at = started_at;
        self
    }

    pub fn with_finished_at(mut self, finished_at: DateTime<Utc>) -> Self {
        self.finished_at = finished_at;
        self
    }

    pub fn with_working_directory(mut self, working_directory: PathBuf) -> Self {
        self.working_directory = working_directory;
        self
    }
}

impl Default for SessionMetadata {
    fn default() -> Self {
        Self {
            session_id: uuid::Uuid::new_v4().to_string(),
            runner_name: "default".to_string(),
            task_id: "default".to_string(),
            started_at: Utc::now(),
            finished_at: Utc::now(),
            working_directory: PathBuf::from("/tmp"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_metadata_instantiation_with_all_fields() {
        let session_id = "test-session-123".to_string();
        let runner_name = "LegacyRunner".to_string();
        let task_id = "TEST-001".to_string();
        let started_at = Utc::now();
        let finished_at = Utc::now();
        let working_directory = PathBuf::from("/tmp/workspace");

        let metadata = SessionMetadata::new(
            session_id.clone(),
            runner_name.clone(),
            task_id.clone(),
            started_at,
            finished_at,
            working_directory.clone(),
        );

        assert_eq!(metadata.session_id, session_id);
        assert_eq!(metadata.runner_name, runner_name);
        assert_eq!(metadata.task_id, task_id);
        assert_eq!(metadata.started_at, started_at);
        assert_eq!(metadata.finished_at, finished_at);
        assert_eq!(metadata.working_directory, working_directory);
    }

    #[test]
    fn test_session_metadata_builder_pattern() {
        let metadata = SessionMetadata::default()
            .with_session_id("custom-session".to_string())
            .with_runner_name("RustRunner".to_string())
            .with_task_id("CUSTOM-001".to_string())
            .with_working_directory(PathBuf::from("/custom/path"));

        assert_eq!(metadata.session_id, "custom-session");
        assert_eq!(metadata.runner_name, "RustRunner");
        assert_eq!(metadata.task_id, "CUSTOM-001");
        assert_eq!(metadata.working_directory, PathBuf::from("/custom/path"));
    }

    #[test]
    fn test_session_metadata_serde_roundtrip() {
        let metadata = SessionMetadata::new(
            "serde-session".to_string(),
            "DifferentialRunner".to_string(),
            "SERDE-001".to_string(),
            Utc::now(),
            Utc::now(),
            PathBuf::from("/serde/workspace"),
        );

        let serialized = serde_json::to_string(&metadata).expect("serialization should succeed");
        let deserialized: SessionMetadata =
            serde_json::from_str(&serialized).expect("deserialization should succeed");

        assert_eq!(metadata.session_id, deserialized.session_id);
        assert_eq!(metadata.runner_name, deserialized.runner_name);
        assert_eq!(metadata.task_id, deserialized.task_id);
        assert_eq!(metadata.working_directory, deserialized.working_directory);
    }

    #[test]
    fn test_session_metadata_default_values() {
        let metadata = SessionMetadata::default();

        assert!(!metadata.session_id.is_empty());
        assert_eq!(metadata.runner_name, "default");
        assert_eq!(metadata.task_id, "default");
        assert_eq!(metadata.working_directory, PathBuf::from("/tmp"));
    }

    #[test]
    fn test_session_metadata_json_format() {
        let metadata = SessionMetadata::default();
        let json = serde_json::to_string(&metadata).expect("serialization should succeed");

        assert!(json.contains("\"session_id\""));
        assert!(json.contains("\"runner_name\""));
        assert!(json.contains("\"task_id\""));
        assert!(json.contains("\"started_at\""));
        assert!(json.contains("\"finished_at\""));
        assert!(json.contains("\"working_directory\""));
    }
}
