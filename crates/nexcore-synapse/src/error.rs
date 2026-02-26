// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Error types for the synapse system.
//!
//! ## Primitive Grounding: ∂ (Boundary) + Σ (Sum)
//!
//! Errors define boundaries between valid and invalid learning states.
//! The error enum is a sum type (Σ) of all possible failure modes.

use nexcore_error::Error;

/// Synapse system errors.
///
/// ## Tier: T2-P (∂ + Σ)
#[derive(Debug, Error)]
pub enum SynapseError {
    /// Synapse not found in the bank.
    #[error("synapse not found: {0}")]
    NotFound(String),

    /// Amplitude value outside valid range [0.0, 1.0].
    #[error("invalid amplitude {value}: must be in [0.0, 1.0]")]
    InvalidAmplitude {
        /// The invalid amplitude value.
        value: f64,
    },

    /// Invalid synapse configuration.
    #[error("invalid synapse config: {0}")]
    InvalidConfig(String),

    /// Gate control error.
    #[error("gate error: {0}")]
    GateError(String),
}

/// Result type for synapse operations.
pub type SynapseResult<T> = Result<T, SynapseError>;
