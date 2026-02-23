//! Error types for the academy-forge crate.

use std::path::PathBuf;

/// Errors that can occur during forge operations.
#[derive(Debug, nexcore_error::Error)]
pub enum ForgeError {
    /// Failed to read a file.
    #[error("Failed to read file {path:?}: {source}")]
    IoError {
        /// Path that failed.
        path: PathBuf,
        /// Underlying IO error.
        source: std::io::Error,
    },

    /// Failed to parse Rust source.
    #[error("Failed to parse Rust source in {file}: {message}")]
    ParseError {
        /// File that failed to parse.
        file: String,
        /// Parse error message.
        message: String,
    },

    /// Failed to parse Cargo.toml.
    #[error("Failed to parse Cargo.toml at {path:?}: {message}")]
    CargoTomlError {
        /// Path to Cargo.toml.
        path: PathBuf,
        /// Error message.
        message: String,
    },

    /// Validation error in academy content.
    #[error("Validation error: {rule} — {message}")]
    ValidationError {
        /// Rule ID (e.g. "R1", "R2").
        rule: String,
        /// Human-readable description.
        message: String,
        /// JSON path to the offending field (if applicable).
        field_path: Option<String>,
    },

    /// Crate directory not found.
    #[error("Crate directory not found: {0:?}")]
    CrateNotFound(PathBuf),

    /// Unknown domain plugin.
    #[error("Unknown domain plugin: {0}")]
    UnknownDomain(String),

    /// Atomization error.
    #[error("Atomization error: {message}")]
    AtomizeError {
        /// Human-readable description.
        message: String,
    },

    /// Cycle detected in dependency graph.
    #[error("Cycle detected in ALO dependency graph: {cycle}")]
    CycleDetected {
        /// Cycle description (node IDs).
        cycle: String,
    },

    /// JSON deserialization error.
    #[error("JSON error: {message}")]
    JsonError {
        /// Error message.
        message: String,
    },
}

/// Result alias for forge operations.
pub type ForgeResult<T> = Result<T, ForgeError>;
