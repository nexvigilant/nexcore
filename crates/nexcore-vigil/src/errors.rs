use crate::models::ExecutorType;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VigilError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Qdrant error: {0}")]
    Qdrant(String),

    #[error("API error: {0}")]
    Api(String),

    #[error("Executor error: {executor:?} - {message}")]
    Executor {
        executor: ExecutorType,
        message: String,
    },

    #[error("Context assembly failed: {0}")]
    Context(String),

    #[error("Decision failed: {0}")]
    Decision(String),

    #[error("LLM error: {0}")]
    Llm(String),

    #[error("Anyhow error: {0}")]
    Anyhow(nexcore_error::NexError),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<nexcore_error::NexError> for VigilError {
    fn from(err: nexcore_error::NexError) -> Self {
        VigilError::Anyhow(err)
    }
}

pub type Result<T> = std::result::Result<T, VigilError>;

/// Backward-compatible alias for the renamed error type.
#[deprecated(note = "Renamed to VigilError — FridayError was an extinct codename")]
pub type FridayError = VigilError;
