//! Knowledge engine error types.

use nexcore_error::Error;

/// Knowledge engine errors.
#[derive(Debug, Error)]
pub enum KnowledgeEngineError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Pack not found: {0}")]
    PackNotFound(String),

    /// Returned by [`KnowledgePack::get_fragment`] and [`KnowledgeStore::find_fragment`]
    /// when no fragment matches the given ID.
    #[error("Fragment not found: {0}")]
    FragmentNotFound(String),

    #[error("Empty input")]
    EmptyInput,

    #[error("Store is empty — no packs compiled yet")]
    EmptyStore,

    #[error("Invalid pack name: {0}")]
    InvalidPackName(String),

    #[error("Store error: {0}")]
    Store(String),
}

pub type Result<T> = std::result::Result<T, KnowledgeEngineError>;
