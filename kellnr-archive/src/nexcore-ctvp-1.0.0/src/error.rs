//! Error types for the CTVP library.

use thiserror::Error;

/// Main error type for CTVP operations.
#[derive(Error, Debug)]
pub enum CtvpError {
    /// Missing required field in builder
    #[error("Builder missing required field: {0}")]
    BuilderMissingField(&'static str),

    /// Validation failed
    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    /// Invalid phase
    #[error("Invalid phase: {0}")]
    InvalidPhase(String),

    /// Evidence not found
    #[error("Evidence not found: {0}")]
    EvidenceNotFound(String),

    /// File system error
    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Parse error
    #[error("Parse error: {0}")]
    Parse(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Threshold not met
    #[error("Threshold not met: expected {expected}, got {actual}")]
    ThresholdNotMet {
        /// Expected value
        expected: String,
        /// Actual value
        actual: String,
    },

    /// Insufficient evidence
    #[error("Insufficient evidence for phase {phase}: {reason}")]
    InsufficientEvidence {
        /// Phase name
        phase: String,
        /// Explanation
        reason: String,
    },

    /// Analysis error
    #[error("Analysis error: {0}")]
    Analysis(String),

    /// Network error (for LLM integration)
    #[cfg(feature = "llm")]
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// Drift detection error
    #[cfg(feature = "drift")]
    #[error("Drift detection error: {0}")]
    DriftDetection(String),

    /// Container error (for testcontainers)
    #[cfg(feature = "testcontainers")]
    #[error("Container error: {0}")]
    Container(String),
}

/// Result type alias for CTVP operations.
pub type CtvpResult<T> = Result<T, CtvpError>;
