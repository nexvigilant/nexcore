// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Error types for the immunity system.
//!
//! ## Primitive Grounding: ∂ (Boundary) + Σ (Sum)
//!
//! Errors define boundaries between valid and invalid states.
//! The error enum is a sum type (Σ) of all possible failure modes.

use thiserror::Error;

/// Immunity system errors.
///
/// ## Tier: T2-P (∂ + Σ)
#[derive(Debug, Error)]
pub enum ImmunityError {
    /// Failed to load antibody registry.
    #[error("failed to load antibody registry: {0}")]
    LoadFailed(String),

    /// Antibody YAML parsing failed.
    #[error("antibody parse error: {0}")]
    ParseError(#[from] serde_yaml::Error),

    /// Regex pattern compilation failed.
    #[error("invalid detection pattern '{pattern}': {reason}")]
    InvalidPattern {
        /// The invalid regex pattern.
        pattern: String,
        /// The reason the pattern is invalid.
        reason: String,
    },

    /// Antibody not found.
    #[error("antibody not found: {0}")]
    NotFound(String),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Response action failed.
    #[error("response action failed: {0}")]
    ResponseFailed(String),
}

/// Result type for immunity operations.
pub type ImmunityResult<T> = Result<T, ImmunityError>;
