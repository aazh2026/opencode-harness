use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtifactKind {
    File,
    Directory,
    Symlink,
    Environment,
    Stdout,
    Stderr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub path: String,
    pub kind: ArtifactKind,
    pub metadata: HashMap<String, String>,
}

impl Artifact {
    pub fn new(path: impl Into<String>, kind: ArtifactKind) -> Self {
        Self {
            path: path.into(),
            kind,
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn file(path: impl Into<String>) -> Self {
        Self::new(path, ArtifactKind::File)
    }

    pub fn directory(path: impl Into<String>) -> Self {
        Self::new(path, ArtifactKind::Directory)
    }

    pub fn symlink(path: impl Into<String>) -> Self {
        Self::new(path, ArtifactKind::Symlink)
    }

    pub fn environment(key: impl Into<String>) -> Self {
        Self::new(key, ArtifactKind::Environment)
    }

    pub fn stdout() -> Self {
        Self::new("stdout", ArtifactKind::Stdout)
    }

    pub fn stderr() -> Self {
        Self::new("stderr", ArtifactKind::Stderr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_artifact_creation() {
        let artifact = Artifact::new("/path/to/file", ArtifactKind::File);
        assert_eq!(artifact.path, "/path/to/file");
        assert_eq!(artifact.kind, ArtifactKind::File);
        assert!(artifact.metadata.is_empty());
    }

    #[test]
    fn test_artifact_with_metadata() {
        let artifact =
            Artifact::new("/path/to/file", ArtifactKind::File).with_metadata("size", "1024");
        assert_eq!(artifact.metadata.get("size"), Some(&"1024".to_string()));
    }

    #[test]
    fn test_artifact_helper_methods() {
        let f = Artifact::file("/tmp/test.txt");
        assert_eq!(f.kind, ArtifactKind::File);

        let d = Artifact::directory("/tmp/dir");
        assert_eq!(d.kind, ArtifactKind::Directory);

        let s = Artifact::symlink("/tmp/link");
        assert_eq!(s.kind, ArtifactKind::Symlink);

        let e = Artifact::environment("PATH");
        assert_eq!(e.kind, ArtifactKind::Environment);

        let out = Artifact::stdout();
        assert_eq!(out.kind, ArtifactKind::Stdout);

        let err = Artifact::stderr();
        assert_eq!(err.kind, ArtifactKind::Stderr);
    }

    #[test]
    fn test_artifact_serde() {
        let artifact = Artifact::file("/tmp/test.txt").with_metadata("size", "100");

        let json = serde_json::to_string(&artifact).unwrap();
        let deserialized: Artifact = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.path, artifact.path);
        assert_eq!(deserialized.kind, artifact.kind);
    }
}
