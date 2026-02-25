//! CCIM error types.
//!
//! Foundation-layer error enum for the Capability Compound Interest Machine.

use std::fmt;

/// Errors that can occur during CCIM operations.
#[derive(Debug)]
#[non_exhaustive]
pub enum CcimError {
    /// State file not found at expected path.
    StateFileNotFound(String),
    /// Failed to parse state file or telemetry data.
    ParseError(String),
    /// Invalid compounding ratio (NaN, Inf, or outside \[0,1\]).
    InvalidRho { value: f64, reason: String },
    /// Conservation invariant violated: |delta| >= EPSILON.
    ConservationViolation {
        delta: f64,
        c_opening: f64,
        c_closing: f64,
        new_tools_cu: f64,
        depreciation_cu: f64,
    },
    /// Projection arithmetic overflow.
    ProjectionOverflow { rho: f64, directives: u32 },
    /// Underlying I/O error.
    Io(std::io::Error),
    /// JSON error.
    Json(String),
}

impl fmt::Display for CcimError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StateFileNotFound(path) => write!(f, "CCIM state file not found: {path}"),
            Self::ParseError(msg) => write!(f, "CCIM parse error: {msg}"),
            Self::InvalidRho { value, reason } => {
                write!(f, "invalid compounding ratio {value}: {reason}")
            }
            Self::ConservationViolation {
                delta,
                c_opening,
                c_closing,
                new_tools_cu,
                depreciation_cu,
            } => write!(
                f,
                "conservation violation: delta={delta:.6}, \
                 C_opening={c_opening}, C_closing={c_closing}, \
                 new_tools={new_tools_cu}, depreciation={depreciation_cu}"
            ),
            Self::ProjectionOverflow { rho, directives } => {
                write!(f, "projection overflow: rho={rho}, directives={directives}")
            }
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::Json(msg) => write!(f, "JSON error: {msg}"),
        }
    }
}

impl std::error::Error for CcimError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for CcimError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<serde_json::Error> for CcimError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e.to_string())
    }
}
