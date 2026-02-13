//! # Vigilance Error Types
//!
//! All error types for the vigilance subsystem.
//!
//! ## Tier: T2-C (∂ + → + ∅ + Σ)
//! Errors are boundary violations that cause failure.

use thiserror::Error;

/// Errors that can occur in the vigilance subsystem.
///
/// Tier: T2-C (∂ + → + ∅ + Σ), dominant ∂
#[derive(Error, Debug)]
pub enum VigilError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Ledger integrity violation: {0}")]
    LedgerIntegrity(String),

    #[error("WAL recovery failed: {0}")]
    WalRecovery(String),

    #[error("Watcher error: {source_name} - {message}")]
    Watcher {
        source_name: String,
        message: String,
    },

    #[error("Boundary specification error: {0}")]
    BoundarySpec(String),

    #[error("Consequence execution failed: {consequence} - {message}")]
    Consequence {
        consequence: String,
        message: String,
    },

    #[error("Daemon lifecycle error: {0}")]
    Daemon(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Channel closed: {0}")]
    ChannelClosed(String),

    #[error("Shutdown requested")]
    Shutdown,
}

/// Result type for vigilance operations.
pub type VigilResult<T> = std::result::Result<T, VigilError>;
