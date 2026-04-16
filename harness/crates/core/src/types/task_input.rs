use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskInput {
    pub command: String,
    pub args: Vec<String>,
    pub cwd: String,
}

impl TaskInput {
    pub fn new(command: impl Into<String>, args: Vec<String>, cwd: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args,
            cwd: cwd.into(),
        }
    }
}
