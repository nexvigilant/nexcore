//! Error types for the quiz platform.
//!
//! Uses `thiserror` for ergonomic error handling with automatic
//! `From` implementations for common error sources.

use nexcore_error::Error;
use nexcore_id::NexId;

/// Result type alias for quiz operations.
pub type QuizResult<T> = Result<T, QuizError>;

/// Comprehensive error type for the quiz platform.
///
/// Covers all error cases from database operations to game logic.
#[derive(Debug, Error)]
pub enum QuizError {
    // === Domain Errors ===
    /// Quiz not found by ID.
    #[error("Quiz not found: {0}")]
    QuizNotFound(NexId),

    /// Game not found by pin.
    #[error("Game not found: {0}")]
    GameNotFound(String),

    /// User not found.
    #[error("User not found: {0}")]
    UserNotFound(NexId),

    /// Game session not found.
    #[error("Game session not found: {0}")]
    SessionNotFound(String),

    // === Authentication Errors ===
    /// Invalid credentials (wrong password or email).
    #[error("Invalid credentials")]
    InvalidCredentials,

    /// JWT token is invalid or expired.
    #[error("Invalid or expired token")]
    InvalidToken,

    /// User is not authorized for this action.
    #[error("Unauthorized")]
    Unauthorized,

    /// API key is invalid.
    #[error("Invalid API key")]
    InvalidApiKey,

    // === Validation Errors ===
    /// Input validation failed.
    #[error("Validation error: {0}")]
    Validation(String),

    /// Question type mismatch (e.g., ABCD answers for RANGE question).
    #[error("Answer type mismatch for question type {question_type}: {message}")]
    AnswerTypeMismatch {
        /// The question type that was expected.
        question_type: String,
        /// Details about what was wrong.
        message: String,
    },

    /// Game pin format invalid.
    #[error("Invalid game pin format: must be 6 digits")]
    InvalidGamePin,

    /// Username already taken in this game.
    #[error("Username '{0}' is already taken in this game")]
    UsernameTaken(String),

    // === Game State Errors ===
    /// Game has not started yet.
    #[error("Game has not started")]
    GameNotStarted,

    /// Game has already started.
    #[error("Game has already started")]
    GameAlreadyStarted,

    /// Game has ended.
    #[error("Game has ended")]
    GameEnded,

    /// Question index out of bounds.
    #[error("Question index {index} out of bounds (max: {max})")]
    QuestionIndexOutOfBounds {
        /// The requested index.
        index: usize,
        /// The maximum valid index.
        max: usize,
    },

    /// Answer already submitted for this question.
    #[error("Answer already submitted for question {0}")]
    AnswerAlreadySubmitted(usize),

    // === Storage Errors ===
    /// File not found in storage.
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// Storage quota exceeded.
    #[error("Storage quota exceeded: used {used} of {limit} bytes")]
    StorageQuotaExceeded {
        /// Bytes currently used.
        used: i64,
        /// Maximum allowed bytes.
        limit: i64,
    },

    /// Invalid MIME type for upload.
    #[error("Invalid MIME type: {0}")]
    InvalidMimeType(String),

    // === External Service Errors ===
    /// Database error.
    #[error("Database error: {0}")]
    Database(String),

    /// Redis/cache error.
    #[error("Cache error: {0}")]
    Cache(String),

    /// External API error (e.g., Kahoot import).
    #[error("External API error: {0}")]
    ExternalApi(String),

    /// Search engine error.
    #[error("Search error: {0}")]
    Search(String),

    // === Internal Errors ===
    /// Serialization/deserialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Internal error that shouldn't happen.
    #[error("Internal error: {0}")]
    Internal(String),
}

impl QuizError {
    /// Returns the HTTP status code for this error.
    ///
    /// Useful for converting errors to HTTP responses in the API layer.
    pub fn status_code(&self) -> u16 {
        match self {
            // 400 Bad Request
            Self::Validation(_)
            | Self::AnswerTypeMismatch { .. }
            | Self::InvalidGamePin
            | Self::InvalidMimeType(_) => 400,

            // 401 Unauthorized
            Self::InvalidCredentials | Self::InvalidToken | Self::InvalidApiKey => 401,

            // 403 Forbidden
            Self::Unauthorized => 403,

            // 404 Not Found
            Self::QuizNotFound(_)
            | Self::GameNotFound(_)
            | Self::UserNotFound(_)
            | Self::SessionNotFound(_)
            | Self::FileNotFound(_) => 404,

            // 409 Conflict
            Self::UsernameTaken(_) | Self::GameAlreadyStarted | Self::AnswerAlreadySubmitted(_) => {
                409
            }

            // 422 Unprocessable Entity
            Self::GameNotStarted | Self::GameEnded | Self::QuestionIndexOutOfBounds { .. } => 422,

            // 507 Insufficient Storage
            Self::StorageQuotaExceeded { .. } => 507,

            // 500 Internal Server Error
            Self::Database(_)
            | Self::Cache(_)
            | Self::ExternalApi(_)
            | Self::Search(_)
            | Self::Serialization(_)
            | Self::Internal(_) => 500,
        }
    }

    /// Returns true if this is a client error (4xx status).
    pub fn is_client_error(&self) -> bool {
        let code = self.status_code();
        (400..500).contains(&code)
    }

    /// Returns true if this is a server error (5xx status).
    pub fn is_server_error(&self) -> bool {
        let code = self.status_code();
        (500..600).contains(&code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quiz_not_found_status() {
        let err = QuizError::QuizNotFound(NexId::v4());
        assert_eq!(err.status_code(), 404);
        assert!(err.is_client_error());
        assert!(!err.is_server_error());
    }

    #[test]
    fn test_database_error_status() {
        let err = QuizError::Database("connection failed".into());
        assert_eq!(err.status_code(), 500);
        assert!(!err.is_client_error());
        assert!(err.is_server_error());
    }

    #[test]
    fn test_validation_error_status() {
        let err = QuizError::Validation("title too long".into());
        assert_eq!(err.status_code(), 400);
    }

    #[test]
    fn test_storage_quota_status() {
        let err = QuizError::StorageQuotaExceeded {
            used: 100_000_000,
            limit: 50_000_000,
        };
        assert_eq!(err.status_code(), 507);
    }
}
