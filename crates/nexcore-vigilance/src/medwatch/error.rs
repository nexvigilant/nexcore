//! Error types for MedWatch validation

use nexcore_error::Error;

/// MedWatch validation and processing errors
#[derive(Debug, Error)]
pub enum MedWatchError {
    /// Invalid FDA date format (expected dd-mmm-yyyy)
    #[error("Invalid FDA date format '{0}': expected dd-mmm-yyyy (e.g., 16-Oct-2019)")]
    InvalidDateFormat(String),

    /// Date is in the future
    #[error("Date '{0}' cannot be in the future")]
    FutureDate(String),

    /// Field exceeds character limit
    #[error("Field '{field}' exceeds {limit} character limit (got {actual})")]
    CharacterLimitExceeded {
        field: String,
        limit: usize,
        actual: usize,
    },

    /// Required field is missing
    #[error("Required field '{0}' is missing")]
    RequiredFieldMissing(String),

    /// Invalid field value
    #[error("Invalid value for field '{field}': {reason}")]
    InvalidFieldValue { field: String, reason: String },
}

/// Result type alias for MedWatch operations
pub type Result<T> = std::result::Result<T, MedWatchError>;
