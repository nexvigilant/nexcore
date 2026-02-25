//! Error types for the transmission system.
//!
//! Provides structured error types for consistent error handling
//! throughout workflow orchestration.

use nexcore_error::Error;
use serde::{Deserialize, Serialize};

/// Result type alias for transmission operations.
pub type TransmissionResult<T> = Result<T, TransmissionError>;

/// Main error type for the transmission system.
#[derive(Debug, Error)]
pub enum TransmissionError {
    /// Engine-related errors.
    #[error("Engine error [{engine}]: {message}")]
    Engine {
        /// Name of the engine that encountered the error.
        engine: String,
        /// Error message.
        message: String,
        /// HTTP status code if applicable.
        status_code: Option<u16>,
        /// Whether this error is retryable.
        retryable: bool,
    },

    /// Workflow-related errors.
    #[error("Workflow error [{workflow}]: {message}")]
    Workflow {
        /// Name of the workflow that encountered the error.
        workflow: String,
        /// Error message.
        message: String,
        /// Step index where the error occurred.
        step_index: Option<usize>,
        /// Whether this error is retryable.
        retryable: bool,
    },

    /// Configuration errors.
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Authentication/Authorization errors.
    #[error("Authentication error: {0}")]
    Authentication(String),

    /// Rate limiting errors.
    #[error("Rate limit exceeded: {message} (retry after {retry_after_secs}s)")]
    RateLimit {
        /// Error message.
        message: String,
        /// Seconds to wait before retrying.
        retry_after_secs: u64,
    },

    /// Circuit breaker open errors.
    #[error("Circuit breaker open for engine: {engine}")]
    CircuitBreakerOpen {
        /// Name of the engine with open circuit breaker.
        engine: String,
    },

    /// Timeout errors.
    #[error("Operation '{operation}' timed out after {timeout_ms}ms")]
    Timeout {
        /// Name of the operation that timed out.
        operation: String,
        /// Timeout duration in milliseconds.
        timeout_ms: u64,
    },

    /// Validation errors.
    #[error("Validation error for field '{field}': {message}")]
    Validation {
        /// Field that failed validation.
        field: String,
        /// Error message.
        message: String,
    },

    /// Serialization/deserialization errors.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Internal errors.
    #[error("Internal error: {0}")]
    Internal(String),
}

impl TransmissionError {
    /// Create an engine error.
    #[must_use]
    pub fn engine(engine: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Engine {
            engine: engine.into(),
            message: message.into(),
            status_code: None,
            retryable: true,
        }
    }

    /// Create an engine error with status code.
    #[must_use]
    pub fn engine_with_status(
        engine: impl Into<String>,
        message: impl Into<String>,
        status_code: u16,
    ) -> Self {
        Self::Engine {
            engine: engine.into(),
            message: message.into(),
            status_code: Some(status_code),
            retryable: status_code >= 500,
        }
    }

    /// Create a workflow error.
    #[must_use]
    pub fn workflow(workflow: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Workflow {
            workflow: workflow.into(),
            message: message.into(),
            step_index: None,
            retryable: false,
        }
    }

    /// Create a workflow error at a specific step.
    #[must_use]
    pub fn workflow_at_step(
        workflow: impl Into<String>,
        message: impl Into<String>,
        step_index: usize,
    ) -> Self {
        Self::Workflow {
            workflow: workflow.into(),
            message: message.into(),
            step_index: Some(step_index),
            retryable: false,
        }
    }

    /// Create a timeout error.
    #[must_use]
    pub fn timeout(operation: impl Into<String>, timeout_ms: u64) -> Self {
        Self::Timeout {
            operation: operation.into(),
            timeout_ms,
        }
    }

    /// Create a validation error.
    #[must_use]
    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Validation {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Check if this error is retryable.
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Engine { retryable, .. } => *retryable,
            Self::Workflow { retryable, .. } => *retryable,
            Self::RateLimit { .. } => true,
            Self::CircuitBreakerOpen { .. } => true,
            Self::Timeout { .. } => true,
            Self::Configuration(_)
            | Self::Authentication(_)
            | Self::Validation { .. }
            | Self::Serialization(_)
            | Self::Internal(_) => false,
        }
    }

    /// Get the HTTP status code for this error.
    #[must_use]
    pub fn status_code(&self) -> u16 {
        match self {
            Self::Engine { status_code, .. } => status_code.unwrap_or(500),
            Self::Workflow { .. } => 500,
            Self::Configuration(_) => 500,
            Self::Authentication(_) => 401,
            Self::RateLimit { .. } => 429,
            Self::CircuitBreakerOpen { .. } => 503,
            Self::Timeout { .. } => 408,
            Self::Validation { .. } => 400,
            Self::Serialization(_) => 400,
            Self::Internal(_) => 500,
        }
    }

    /// Get the error code for this error.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::Engine { .. } => "ENGINE_ERROR",
            Self::Workflow { .. } => "WORKFLOW_ERROR",
            Self::Configuration(_) => "CONFIGURATION_ERROR",
            Self::Authentication(_) => "AUTHENTICATION_ERROR",
            Self::RateLimit { .. } => "RATE_LIMIT_ERROR",
            Self::CircuitBreakerOpen { .. } => "CIRCUIT_BREAKER_OPEN",
            Self::Timeout { .. } => "TIMEOUT_ERROR",
            Self::Validation { .. } => "VALIDATION_ERROR",
            Self::Serialization(_) => "SERIALIZATION_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }
}

/// Error response format for API responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error details.
    pub error: ErrorDetails,
}

/// Error details within an error response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetails {
    /// Error code.
    pub code: String,
    /// Error message.
    pub message: String,
    /// Additional details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    /// Timestamp of the error.
    pub timestamp: String,
    /// Request ID if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// Whether the error is retryable.
    pub retryable: bool,
}

impl ErrorResponse {
    /// Create an error response from a transmission error.
    #[must_use]
    pub fn from_error(error: &TransmissionError, request_id: Option<String>) -> Self {
        Self {
            error: ErrorDetails {
                code: error.code().to_string(),
                message: error.to_string(),
                details: None,
                timestamp: nexcore_chrono::DateTime::now().to_rfc3339(),
                request_id,
                retryable: error.is_retryable(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_error() {
        let err = TransmissionError::engine("test-engine", "connection failed");
        assert!(err.is_retryable());
        assert_eq!(err.status_code(), 500);
        assert_eq!(err.code(), "ENGINE_ERROR");
    }

    #[test]
    fn test_engine_error_with_status() {
        let err = TransmissionError::engine_with_status("test-engine", "not found", 404);
        assert!(!err.is_retryable()); // 404 is not retryable
        assert_eq!(err.status_code(), 404);
    }

    #[test]
    fn test_workflow_error() {
        let err = TransmissionError::workflow_at_step("daily-flow", "step failed", 2);
        assert!(!err.is_retryable());
        assert_eq!(err.status_code(), 500);
        // INVARIANT: we just created a Workflow error above, so this match will succeed
        assert!(matches!(
            err,
            TransmissionError::Workflow {
                step_index: Some(2),
                ..
            }
        ));
    }

    #[test]
    fn test_timeout_error() {
        let err = TransmissionError::timeout("engine_call", 30000);
        assert!(err.is_retryable());
        assert_eq!(err.status_code(), 408);
    }

    #[test]
    fn test_error_response() {
        let err = TransmissionError::validation("payload", "missing required field");
        let response = ErrorResponse::from_error(&err, Some("req-123".to_string()));

        assert_eq!(response.error.code, "VALIDATION_ERROR");
        assert!(!response.error.retryable);
        assert_eq!(response.error.request_id, Some("req-123".to_string()));
    }
}
