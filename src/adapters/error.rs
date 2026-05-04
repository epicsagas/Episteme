#[derive(Debug, thiserror::Error)]
pub enum InfraError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("Database error: {0}")]
    Database(String),
    #[error("Config error: {0}")]
    Config(String),
    #[error("Embedding error: {0}")]
    Embedding(String),
    #[error("Entity not found: {0}")]
    EntityNotFound(String),
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),
    #[error("Parse error: {0}")]
    Parse(String),
}

pub type Result<T> = std::result::Result<T, InfraError>;
