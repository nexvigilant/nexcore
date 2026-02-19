//! Error types for vocabulary intelligence operations.

use thiserror::Error;

/// Result type for vocabulary operations.
pub type VocabResult<T> = Result<T, VocabError>;

/// Errors that can occur in vocabulary operations.
#[derive(Debug, Error)]
pub enum VocabError {
    /// Invalid vocabulary tier (must be 1-3).
    #[error("invalid vocabulary tier: {0} (must be 1, 2, or 3)")]
    InvalidTier(u8),

    /// Empty term provided.
    #[error("term cannot be empty")]
    EmptyTerm,

    /// Invalid domain identifier.
    #[error("invalid domain: {0}")]
    InvalidDomain(String),

    /// Lexicon not found.
    #[error("lexicon not found for domain: {0}")]
    LexiconNotFound(String),

    /// Entry already exists.
    #[error("entry already exists: {0}")]
    EntryExists(String),

    /// Entry not found.
    #[error("entry not found: {0}")]
    EntryNotFound(String),

    /// JSON serialization error.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Regex compilation error.
    #[error("regex error: {0}")]
    Regex(#[from] regex::Error),
}
