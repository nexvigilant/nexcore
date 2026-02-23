//! Build orchestrator error types.
//!
//! ## Primitive Foundation
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | T1: Σ (Sum) | Error variants as alternation |
//! | T1: ∂ (Boundary) | Error boundaries between subsystems |

use std::path::PathBuf;

/// Tier: T2-C (Σ + ∂ + σ + λ, dominant Σ)
#[derive(Debug, nexcore_error::Error)]
pub enum BuildOrcError {
    /// I/O operation failed
    #[error("I/O error at {path:?}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },

    /// Pipeline stage failed
    #[error("Stage '{stage}' failed with exit code {exit_code}")]
    StageFailed { stage: String, exit_code: i32 },

    /// Stage timed out
    #[error("Stage '{stage}' timed out after {timeout_secs}s")]
    StageTimeout { stage: String, timeout_secs: u64 },

    /// Pipeline definition error
    #[error("Pipeline definition error: {0}")]
    Definition(String),

    /// Invalid state transition
    #[error("Invalid transition from {from} to {to}")]
    InvalidTransition { from: String, to: String },

    /// Build gate error
    #[error("Build gate: {0}")]
    Gate(String),

    /// Serialization error
    #[error("Serialization: {0}")]
    Serde(String),

    /// Workspace scan error
    #[error("Workspace scan: {0}")]
    WorkspaceScan(String),

    /// History store error
    #[error("History: {0}")]
    History(String),
}

impl From<std::io::Error> for BuildOrcError {
    fn from(e: std::io::Error) -> Self {
        Self::Io {
            path: PathBuf::from("<unknown>"),
            source: e,
        }
    }
}

impl From<serde_json::Error> for BuildOrcError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serde(e.to_string())
    }
}

impl From<nexcore_build_gate::GateError> for BuildOrcError {
    fn from(e: nexcore_build_gate::GateError) -> Self {
        Self::Gate(e.to_string())
    }
}

/// Result type alias for build orchestrator operations.
pub type BuildOrcResult<T> = Result<T, BuildOrcError>;
