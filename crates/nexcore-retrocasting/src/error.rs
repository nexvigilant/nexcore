// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Error types for retrocasting operations.
//!
//! ## Tier: T2-P (∂ + ∃)
//! Boundary (∂) violations that assert non-existence (¬∃) of valid state.

use thiserror::Error;

/// Errors from retrocasting operations.
#[derive(Debug, Error)]
pub enum RetrocastError {
    /// A required compound could not be resolved to a structure.
    #[error("Compound '{name}' could not be resolved: {reason}")]
    CompoundResolutionFailed { name: String, reason: String },

    /// No signals were found for the given drug.
    #[error("No FAERS signals found for drug '{drug}'")]
    NoSignals { drug: String },

    /// Clustering failed.
    #[error("Structural clustering failed: {0}")]
    ClusteringError(String),

    /// Correlation analysis failed.
    #[error("Alert correlation failed: {0}")]
    CorrelationError(String),

    /// Training data generation failed.
    #[error("Training data generation failed: {0}")]
    TrainingError(String),

    /// Empty input provided where data was required.
    #[error("Empty input: {0}")]
    EmptyInput(String),

    /// Invalid threshold value (must be in [0.0, 1.0]).
    #[error("Invalid similarity threshold {value}: must be in [0.0, 1.0]")]
    InvalidThreshold { value: f64 },

    /// A registry error from compound-registry crate.
    #[error("Registry error: {0}")]
    Registry(String),

    /// I/O error during data export.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for retrocasting operations.
pub type RetrocastResult<T> = Result<T, RetrocastError>;
