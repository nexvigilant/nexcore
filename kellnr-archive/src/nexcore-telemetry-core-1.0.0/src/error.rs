//! Error types for telemetry core operations.
//!
//! Provides unified error handling for source parsing,
//! snapshot management, and cross-reference analysis.

use std::path::PathBuf;
use thiserror::Error;

/// Errors during telemetry operations.
#[derive(Debug, Error)]
pub enum TelemetryError {
    /// IO error during file operations
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON parsing error
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    /// Source file not found
    #[error("Source not found: {path}")]
    SourceNotFound { path: PathBuf },

    /// Snapshot not found
    #[error("Snapshot not found: {path}")]
    SnapshotNotFound { path: PathBuf },

    /// Invalid source format
    #[error("Invalid source format: {reason}")]
    InvalidFormat { reason: String },

    /// Project directory not found
    #[error("Project not found: {hash}")]
    ProjectNotFound { hash: String },

    /// Telemetry home directory not found
    #[error("Telemetry home not found")]
    HomeNotFound,

    /// Brain directory not found
    #[error("Brain directory not found")]
    BrainNotFound,
}

/// Result type alias for telemetry operations.
pub type Result<T> = std::result::Result<T, TelemetryError>;
