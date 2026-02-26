// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Error types for the energy system.
//!
//! ## Primitive Grounding: ∂ (Boundary) + Σ (Sum)
//!
//! Errors define boundaries between valid and invalid energy states.
//! The error enum is a sum type (Σ) of all possible failure modes.

use nexcore_error::Error;

/// Energy system errors.
///
/// ## Tier: T2-P (∂ + Σ)
#[derive(Debug, Error)]
pub enum EnergyError {
    /// Energy charge value outside valid range [0.0, 1.0].
    #[error("invalid energy charge {value}: must be in [0.0, 1.0]")]
    InvalidEnergyCharge {
        /// The invalid charge value.
        value: f64,
    },

    /// Conservation law violated: total pool changed unexpectedly.
    #[error("conservation violation: expected total {expected}, got {actual}")]
    ConservationViolation {
        /// Expected total across all pools.
        expected: u64,
        /// Actual total observed.
        actual: u64,
    },

    /// Invalid operation on the energy system.
    #[error("invalid energy operation: {0}")]
    InvalidOperation(String),
}

/// Result type for energy operations.
pub type EnergyResult<T> = Result<T, EnergyError>;
