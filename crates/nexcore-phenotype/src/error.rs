// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Error types for the phenotype system.
//!
//! ## Primitive Grounding: ∂ (Boundary) + Σ (Sum)
//!
//! Errors define boundaries between valid and invalid mutation states.
//! The error enum is a sum type (Σ) of all possible failure modes.

use nexcore_error::Error;

/// Phenotype system errors.
///
/// ## Tier: T2-P (∂ + Σ)
#[derive(Debug, Error)]
pub enum PhenotypeError {
    /// Mutation could not be applied to the given schema.
    #[error("mutation failed for {mutation}: {reason}")]
    MutationFailed {
        /// The mutation that failed.
        mutation: String,
        /// Why it failed.
        reason: String,
    },

    /// Verification against ribosome failed.
    #[error("verification error: {0}")]
    VerificationError(String),

    /// Invalid schema provided for mutation.
    #[error("invalid schema: {0}")]
    InvalidSchema(String),
}

/// Result type for phenotype operations.
pub type PhenotypeResult<T> = Result<T, PhenotypeError>;
