//! Error types for PV math computations.

use nexcore_error::Error;

/// Errors from PV signal detection and causality calculations.
#[derive(Debug, Error)]
pub enum PvMathError {
    /// Contingency table is invalid (e.g., all-zero).
    #[error("Invalid contingency table: {0}")]
    InvalidTable(String),

    /// Mathematical operation failed (division by zero, overflow).
    #[error("Math error: {0}")]
    MathError(String),

    /// Insufficient data for the requested calculation.
    #[error("Insufficient data: {0}")]
    InsufficientData(String),
}

impl PvMathError {
    /// Create an invalid-table error.
    pub fn invalid_table(msg: impl Into<String>) -> Self {
        Self::InvalidTable(msg.into())
    }

    /// Create a math error.
    pub fn math_error(msg: impl Into<String>) -> Self {
        Self::MathError(msg.into())
    }

    /// Create an insufficient-data error.
    pub fn insufficient_data(msg: impl Into<String>) -> Self {
        Self::InsufficientData(msg.into())
    }
}
