//! Error types for the perception pipeline.

use nexcore_error::Error;

/// Errors produced by the Environmental Perception pipeline.
#[derive(Debug, Error)]
pub enum PerceptionError {
    /// A source connector failed to fetch or stream data.
    #[error("source connector failed for '{source}': {reason}")]
    ConnectorFailure {
        /// The source identifier.
        source: String,
        /// Human-readable reason.
        reason: String,
    },

    /// Record normalization failed.
    #[error("normalization failed for record '{record_id}': {reason}")]
    NormalizationFailure {
        /// The record identifier.
        record_id: String,
        /// Human-readable reason.
        reason: String,
    },

    /// Fusion engine encountered an unresolvable conflict.
    #[error("fusion conflict on entity '{entity_id}': delta={delta:.3}")]
    FusionConflict {
        /// The entity identifier.
        entity_id: String,
        /// Computed conflict delta (confidence spread).
        delta: f64,
    },

    /// World model state store error.
    #[error("state store error: {0}")]
    StateStore(String),

    /// YAML configuration parsing failed.
    #[error("config parse error: {0}")]
    ConfigParse(String),

    /// HTTP request error from a connector.
    #[error("http error: {0}")]
    Http(String),

    /// JSON serialization / deserialization error.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// An entity was not found in the world model.
    #[error("entity not found: {0}")]
    NotFound(String),

    /// A required field was missing from the incoming record.
    #[error("missing required field: {0}")]
    MissingField(String),

    /// Channel send failed (receiver dropped).
    #[error("channel send error: {0}")]
    ChannelSend(String),
}

/// Convenience result alias for perception operations.
pub type Result<T> = std::result::Result<T, PerceptionError>;
