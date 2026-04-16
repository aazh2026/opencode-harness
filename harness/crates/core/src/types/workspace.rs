use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub path: PathBuf,
    pub fixture_name: String,
    pub created_at: String,
}

impl Workspace {
    pub fn new(id: String, path: PathBuf, fixture_name: String) -> Self {
        Self {
            id,
            path,
            fixture_name,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn fixture_name(&self) -> &str {
        &self.fixture_name
    }
}
