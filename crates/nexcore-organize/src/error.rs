//! Error types for the ORGANIZE pipeline.
//!
//! Tier: T2-P (∂ Boundary — error boundary between success and failure)

use std::path::PathBuf;

/// All errors that can occur during the ORGANIZE pipeline.
///
/// Tier: T2-P (dominant: ∂ Boundary)
#[derive(Debug, thiserror::Error)]
pub enum OrganizeError {
    /// I/O error during file operations.
    #[error("I/O error at {path}: {source}")]
    Io {
        /// Path where the error occurred.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },

    /// Directory walk error.
    #[error("walk error: {0}")]
    Walk(#[from] walkdir::Error),

    /// Configuration parse error.
    #[error("config error: {0}")]
    Config(String),

    /// TOML deserialization error.
    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    /// JSON serialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Naming conflict that cannot be auto-resolved.
    #[error("naming conflict: {existing} and {incoming} both target {target}")]
    NamingConflict {
        /// Existing file at target.
        existing: PathBuf,
        /// File attempting to move to target.
        incoming: PathBuf,
        /// Conflicting target path.
        target: PathBuf,
    },

    /// Pipeline step received unexpected input.
    #[error("pipeline error at step {step}: {message}")]
    Pipeline {
        /// Step name (e.g., "observe", "rank").
        step: String,
        /// Description of the error.
        message: String,
    },

    /// Attempted a live mutation in dry-run mode.
    #[error("mutation blocked: dry_run is enabled")]
    DryRunBlocked,
}

/// Result alias for the ORGANIZE pipeline.
pub type OrganizeResult<T> = Result<T, OrganizeError>;
