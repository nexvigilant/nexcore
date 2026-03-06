//! Station-specific error types.

/// Errors that can occur during station resolution.
#[derive(Debug, nexcore_error::Error)]
pub enum StationError {
    /// The requested domain has no config coverage.
    #[error("domain not covered: {domain}")]
    DomainNotCovered {
        /// The domain that was requested.
        domain: String,
    },

    /// The observatory feed is unavailable.
    #[error("feed unavailable: {reason}")]
    FeedUnavailable {
        /// Why the feed is unavailable.
        reason: String,
    },

    /// Resolution failed for a domain.
    #[error("resolution failed for {domain}: {detail}")]
    ResolutionFailed {
        /// The domain that failed.
        domain: String,
        /// Details about the failure.
        detail: String,
    },

    /// Config serialization failed during resolution.
    #[error("serialization failed for {domain}: {detail}")]
    SerializationFailed {
        /// The domain whose config failed to serialize.
        domain: String,
        /// Serialization error detail.
        detail: String,
    },

    /// HTTP request to the Observatory feed failed.
    #[error("http request failed: {reason}")]
    HttpError {
        /// Details about the HTTP failure.
        reason: String,
    },
}
