//! Error types for the nexcore-openfda client.

/// Errors from OpenFDA API operations.
#[derive(Debug, thiserror::Error)]
pub enum OpenFdaError {
    /// Failed to build the HTTP client.
    #[error("Failed to build HTTP client: {0}")]
    ClientBuild(#[source] reqwest::Error),

    /// Network request failed.
    #[error("OpenFDA API request failed: {0}")]
    NetworkError(#[source] reqwest::Error),

    /// Non-2xx HTTP response.
    #[error("OpenFDA returned HTTP {status}: {message}")]
    InvalidResponse { status: u16, message: String },

    /// JSON deserialization failed.
    #[error("Failed to parse OpenFDA response: {0}")]
    ParseError(#[source] serde_json::Error),

    /// Rate limited — caller should respect `retry_after_secs`.
    #[error("OpenFDA rate limit exceeded, retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },

    /// API unavailable and no cached response exists.
    #[error("OpenFDA unavailable: {reason} (no cached fallback)")]
    Unavailable { reason: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_limited_display() {
        let err = OpenFdaError::RateLimited {
            retry_after_secs: 60,
        };
        let msg = err.to_string();
        assert!(msg.contains("rate limit"));
        assert!(msg.contains("60"));
    }

    #[test]
    fn unavailable_display() {
        let err = OpenFdaError::Unavailable {
            reason: "503".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("unavailable"));
        assert!(msg.contains("no cached fallback"));
    }

    #[test]
    fn invalid_response_display() {
        let err = OpenFdaError::InvalidResponse {
            status: 404,
            message: "not found".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("404"));
        assert!(msg.contains("not found"));
    }
}
