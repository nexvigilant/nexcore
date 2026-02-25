//! QBR error types.
//!
//! Typed, named failure paths for every invalid state.
//! Combined with workspace `deny(clippy::unwrap_used)`, every failure
//! is explicit and testable.

use nexcore_error::Error;

/// Errors during QBR computation.
#[derive(Debug, Error, PartialEq, Clone)]
pub enum QbrError {
    /// No benefit contingency tables provided.
    #[error("No benefit tables provided")]
    NoBenefitTables,

    /// No risk contingency tables provided.
    #[error("No risk tables provided")]
    NoRiskTables,

    /// Weight vector length does not match table count.
    #[error("Weight count ({weights}) does not match table count ({tables})")]
    WeightMismatch {
        /// Number of weights provided.
        weights: usize,
        /// Number of tables provided.
        tables: usize,
    },

    /// Underlying signal detection algorithm failed.
    #[error("Signal detection failed: {0}")]
    SignalDetection(String),

    /// Invalid Hill curve parameters.
    #[error("Invalid Hill parameters: {0}")]
    InvalidHillParams(String),

    /// Numerical integration failed.
    #[error("Integration error: {0}")]
    Integration(String),

    /// Division by zero (risk signal strength is zero).
    #[error("Risk signal strength is zero — cannot compute ratio")]
    ZeroRiskSignal,
}
