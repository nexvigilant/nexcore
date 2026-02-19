//! # Foundation Error Types
//!
//! Standard error types for foundation operations including algorithms,
//! data processing, execution, and state management.

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Unified error type for foundation operations
#[derive(Error, Debug)]
pub enum FoundationError {
    /// Input validation failed
    #[error("Validation error: {0}")]
    Validation(String),

    /// Execution logic failed
    #[error("Execution error: {0}")]
    Execution(String),

    /// Configuration error
    #[error("Config error: {0}")]
    Config(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Graph cycle detected
    #[error("Cycle detected: {0:?}")]
    CycleDetected(Vec<String>),

    /// State management error
    #[error("State error: {0}")]
    State(String),

    /// Unknown or unspecified error
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Result type alias for foundation operations
pub type FoundationResult<T> = Result<T, FoundationError>;

/// Legacy skill error codes (for RSK compatibility)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillErrorCode {
    /// Input validation failed
    ValidationError,
    /// Execution logic failed
    ExecutionError,
    /// Output generation failed
    OutputError,
    /// Configuration error
    ConfigError,
    /// Unknown or unspecified error
    Unknown,
}

impl fmt::Display for SkillErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ValidationError => write!(f, "ValidationError"),
            Self::ExecutionError => write!(f, "ExecutionError"),
            Self::OutputError => write!(f, "OutputError"),
            Self::ConfigError => write!(f, "ConfigError"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Legacy skill error type (for RSK compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillError {
    /// Error code indicating the type of error
    pub code: SkillErrorCode,
    /// Human-readable error message
    pub message: String,
}

impl SkillError {
    /// Create a new `SkillError`
    #[must_use]
    pub fn new(code: SkillErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    /// Create a validation error
    #[must_use]
    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(SkillErrorCode::ValidationError, message)
    }

    /// Create an execution error
    #[must_use]
    pub fn execution(message: impl Into<String>) -> Self {
        Self::new(SkillErrorCode::ExecutionError, message)
    }
}

impl fmt::Display for SkillError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for SkillError {}

impl From<SkillError> for FoundationError {
    fn from(err: SkillError) -> Self {
        match err.code {
            SkillErrorCode::ValidationError => Self::Validation(err.message),
            SkillErrorCode::ExecutionError => Self::Execution(err.message),
            SkillErrorCode::ConfigError => Self::Config(err.message),
            _ => Self::Unknown(err.message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_error_creation() {
        let err = SkillError::new(SkillErrorCode::ValidationError, "test error");
        assert_eq!(err.code, SkillErrorCode::ValidationError);
        assert_eq!(err.message, "test error");
    }

    #[test]
    fn test_skill_error_display() {
        let err = SkillError::validation("invalid input");
        assert!(err.to_string().contains("ValidationError"));
        assert!(err.to_string().contains("invalid input"));
    }

    #[test]
    fn test_foundation_error_conversion() {
        let skill_err = SkillError::validation("test");
        let foundation_err: FoundationError = skill_err.into();
        assert!(matches!(foundation_err, FoundationError::Validation(_)));
    }
}
