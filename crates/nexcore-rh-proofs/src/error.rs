//! Error types for the RH proof infrastructure.

use nexcore_error::Error;

/// Errors produced by RH proof infrastructure operations.
#[derive(Debug, Error)]
pub enum RhProofError {
    /// Numerical residual exceeds the precision threshold.
    #[error("insufficient numerical precision: residual {residual} exceeds threshold {threshold}")]
    InsufficientPrecision {
        /// Observed residual |ζ(1/2 + it)|.
        residual: f64,
        /// Acceptable threshold for counting as a zero.
        threshold: f64,
    },

    /// Input value is outside the valid range for a given test.
    #[error("value out of range for test: {context}")]
    OutOfRange {
        /// Description of the violated range constraint.
        context: String,
    },

    /// Arithmetic overflow during computation.
    #[error("computation overflow")]
    Overflow,

    /// Error propagated from the number-theory layer.
    #[error("number theory error: {0}")]
    NumberTheory(String),
}
