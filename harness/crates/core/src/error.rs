#[derive(Debug, thiserror::Error)]
pub enum ErrorType {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Task not found: {0}")]
    TaskNotFound(String),

    #[error("Runner error: {0}")]
    Runner(String),

    #[error("Environment error: {0}")]
    Environment(String),

    #[error("Artifact error: {0}")]
    Artifact(String),
}

pub type Result<T> = std::result::Result<T, ErrorType>;
