//! Knowledge engine error types.

use thiserror::Error;

/// Knowledge engine errors.
#[derive(Debug, Error)]
pub enum KnowledgeEngineError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Pack not found: {0}")]
    PackNotFound(String),

    #[error("Fragment not found: {0}")]
    FragmentNotFound(String),

    #[error("Empty input")]
    EmptyInput,

    #[error("Store error: {0}")]
    Store(String),
}

pub type Result<T> = std::result::Result<T, KnowledgeEngineError>;
