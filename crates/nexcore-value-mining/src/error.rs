// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Error types for value mining operations.

use nexcore_error::Error;

/// Result type alias for mining operations.
pub type MiningResult<T> = Result<T, MiningError>;

/// Error types for value mining operations.
#[derive(Debug, Error)]
pub enum MiningError {
    /// Insufficient data for signal detection.
    #[error("Insufficient data: {0}")]
    InsufficientData(String),

    /// Invalid baseline data.
    #[error("Invalid baseline: {0}")]
    InvalidBaseline(String),

    /// Signal detection failed.
    #[error("Detection failed: {0}")]
    DetectionFailed(String),

    /// Social API error (string-wrapped; nexcore-social is Service-layer).
    #[error("Social API error: {0}")]
    SocialError(String),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Invalid configuration.
    #[error("Invalid configuration: {0}")]
    ConfigError(String),
}
