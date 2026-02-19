//! Error types for the bioinfo crate.

use thiserror::Error;

/// Result type alias for bioinfo operations.
pub type BioinfoResult<T> = Result<T, BioinfoError>;

/// Errors that can occur in bioinfo operations.
#[derive(Debug, Error)]
pub enum BioinfoError {
    /// Invalid entity type provided.
    #[error("Invalid entity type: {0}")]
    InvalidEntityType(String),

    /// Invalid pathway ID format.
    #[error("Invalid pathway ID format: {0}")]
    InvalidPathwayId(String),

    /// Invalid KEGG ID format.
    #[error("Invalid KEGG ID format: {0}")]
    InvalidKeggId(String),

    /// Entity not found.
    #[error("Entity not found: {0}")]
    NotFound(String),

    /// Convergence analysis requires at least 2 entities.
    #[error("Convergence analysis requires at least 2 entities, got {0}")]
    InsufficientEntities(usize),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
