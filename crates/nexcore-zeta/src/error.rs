//! Error types for zeta function operations.

use nexcore_error::Error;

/// Errors produced by zeta function computations.
#[derive(Debug, Error)]
pub enum ZetaError {
    /// Error propagated from complex arithmetic.
    #[error("complex arithmetic error: {0}")]
    Complex(#[from] stem_complex::ComplexError),

    /// Series did not converge within the iteration budget.
    #[error("convergence failure after {iterations} iterations")]
    ConvergenceFailure {
        /// Number of iterations attempted.
        iterations: usize,
    },

    /// The zeta function has a pole at s = 1.
    #[error("zeta undefined at s = 1 (pole)")]
    PoleAtOne,

    /// An input parameter was invalid.
    #[error("invalid parameter: {0}")]
    InvalidParameter(String),

    /// Numerical overflow in computation.
    #[error("numerical overflow in computation")]
    Overflow,
}
