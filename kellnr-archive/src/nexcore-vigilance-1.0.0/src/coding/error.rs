//! Error types for medical coding operations.

use thiserror::Error;

/// Errors that can occur during medical coding.
#[derive(Debug, Error)]
pub enum CodingError {
    /// Invalid `MedDRA` code format.
    #[error("Invalid MedDRA code: {0}")]
    InvalidMeddraCode(String),

    /// `MedDRA` code not found in dictionary.
    #[error("MedDRA code not found: {0}")]
    MeddraCodeNotFound(String),

    /// Invalid hierarchy level.
    #[error("Invalid hierarchy level: {0}")]
    InvalidHierarchyLevel(String),

    /// File parsing error.
    #[error("Parse error: {0}")]
    ParseError(String),

    /// IO error during file loading.
    #[error("IO error: {0}")]
    IoError(String),

    /// Dictionary not loaded.
    #[error("Dictionary not loaded: {0}")]
    DictionaryNotLoaded(String),

    /// Search returned no results.
    #[error("No match found for: {0}")]
    NoMatch(String),
}

impl CodingError {
    /// Create a parse error.
    pub fn parse_error(msg: impl Into<String>) -> Self {
        Self::ParseError(msg.into())
    }

    /// Create an IO error.
    pub fn io_error(msg: impl Into<String>) -> Self {
        Self::IoError(msg.into())
    }
}
