//! Error types for the Command Center crate.

use nexcore_error::Error;

/// Result type alias for Command Center operations.
pub type CommandCenterResult<T> = Result<T, CommandCenterError>;

/// Errors that can occur in Command Center operations.
#[derive(Debug, Error)]
pub enum CommandCenterError {
    /// Entity not found.
    #[error("Not found: {0}")]
    NotFound(String),

    /// Validation error.
    #[error("Validation error: {0}")]
    Validation(String),

    /// Access denied - insufficient permissions.
    #[error("Access denied: {0}")]
    AccessDenied(String),

    /// Business rule violation.
    #[error("Business rule violation: {0}")]
    BusinessRule(String),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Invalid UUID.
    #[error("Invalid UUID: {0}")]
    InvalidUuid(#[from] nexcore_id::ParseError),
}
