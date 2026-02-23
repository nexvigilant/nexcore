//! Error types for the GROUNDED system.

use nexcore_error::Error;

#[derive(Debug, Error)]
pub enum GroundedError {
    #[error("confidence value {0} out of range [0.0, 1.0]")]
    ConfidenceOutOfRange(f64),

    #[error("experiment failed: {0}")]
    ExperimentFailed(String),

    #[error("integration failed: {0}")]
    IntegrationFailed(String),

    #[error("persistence failed: {0}")]
    PersistenceFailed(String),

    #[error("context update failed: {0}")]
    ContextUpdateFailed(String),

    #[error("verification failed: expected {expected}, got {actual}")]
    VerificationFailed { expected: String, actual: String },

    #[error("specification violated: {0}")]
    SpecificationViolated(String),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
