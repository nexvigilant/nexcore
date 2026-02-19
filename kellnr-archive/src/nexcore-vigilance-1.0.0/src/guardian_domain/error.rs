//! Error types for the Guardian healthcare compliance domain.

use thiserror::Error;

/// Result type alias for Guardian operations.
pub type GuardianResult<T> = Result<T, GuardianError>;

/// Errors that can occur in the Guardian domain.
#[derive(Debug, Error)]
pub enum GuardianError {
    /// Entity not found.
    #[error("Not found: {0}")]
    NotFound(String),

    /// Validation failed.
    #[error("Validation error: {0}")]
    Validation(String),

    /// Field too long.
    #[error("Field '{field}' exceeds maximum length of {max_length}")]
    FieldTooLong { field: String, max_length: usize },

    /// Field too short or empty.
    #[error("Field '{field}' must be at least {min_length} characters")]
    FieldTooShort { field: String, min_length: usize },

    /// Invalid format.
    #[error("Invalid format for '{field}': {reason}")]
    InvalidFormat { field: String, reason: String },

    /// Compliance violation.
    #[error("Compliance violation: {0}")]
    ComplianceViolation(String),

    /// Integrity check failed.
    #[error("Integrity check failed: {0}")]
    IntegrityCheckFailed(String),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Regex error.
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),
}
