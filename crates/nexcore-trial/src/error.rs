//! Error types for the TRIAL framework.

/// Errors that can occur during trial protocol execution.
#[derive(Debug, thiserror::Error)]
pub enum TrialError {
    /// Protocol validation failed before registration.
    #[error("Protocol validation failed: {0}")]
    InvalidProtocol(String),

    /// Statistical power is below the required minimum.
    #[error("Insufficient power: {actual:.2} < {required:.2}")]
    InsufficientPower { actual: f64, required: f64 },

    /// A safety boundary was crossed — trial must stop.
    #[error("Safety boundary crossed: {metric} = {value:.4} > {threshold:.4}")]
    SafetyBoundaryCrossed {
        metric: String,
        value: f64,
        threshold: f64,
    },

    /// A referenced protocol ID was not found.
    #[error("Protocol not found: {0}")]
    NotFound(String),

    /// A phase gate requirement was not satisfied.
    #[error("Phase gate failed: {phase} requires {requirement}")]
    PhaseGateFailed { phase: String, requirement: String },

    /// An adaptation was requested that was not pre-specified in the protocol.
    #[error("Adaptation not pre-specified: {0}")]
    UnauthorizedAdaptation(String),

    /// Invalid input parameters for a statistical computation.
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// JSON serialization/deserialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
