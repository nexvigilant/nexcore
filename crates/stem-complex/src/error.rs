//! Error types for complex number operations.

use nexcore_error::Error;

/// Errors that arise from complex number operations.
///
/// Each variant represents a distinct mathematical failure mode.
#[derive(Debug, Error)]
pub enum ComplexError {
    /// Division by a complex number with zero magnitude.
    #[error("division by zero")]
    DivisionByZero,

    /// Logarithm of zero is undefined.
    #[error("logarithm of zero")]
    LogOfZero,

    /// The operation is undefined at this input.
    #[error("operation undefined: {0}")]
    Undefined(String),
}
