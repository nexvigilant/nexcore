//! NCBI E-utilities error types. ∂(Boundary) violations for network API calls.

use core::fmt;

/// Errors from NCBI E-utilities API interactions.
///
/// Tier: T2-C (∂ + μ + →)
#[derive(Debug)]
pub enum NcbiError {
    /// HTTP transport error.
    Http(reqwest::Error),

    /// JSON deserialization error.
    Json(serde_json::Error),

    /// NCBI API returned a non-success status code.
    Api { status: u16, message: String },

    /// Rate limited by NCBI (HTTP 429).
    RateLimited { retry_after_secs: u64 },

    /// Invalid parameters before sending request.
    InvalidParams(String),

    /// DNA parsing error from processing fetched sequences.
    Dna(crate::error::DnaError),
}

impl fmt::Display for NcbiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Http(e) => write!(f, "NCBI HTTP error: {e}"),
            Self::Json(e) => write!(f, "NCBI JSON parse error: {e}"),
            Self::Api { status, message } => {
                write!(f, "NCBI API error (HTTP {status}): {message}")
            }
            Self::RateLimited { retry_after_secs } => {
                write!(f, "NCBI rate limited — retry after {retry_after_secs}s")
            }
            Self::InvalidParams(msg) => write!(f, "NCBI invalid parameters: {msg}"),
            Self::Dna(e) => write!(f, "NCBI DNA parse error: {e}"),
        }
    }
}

impl std::error::Error for NcbiError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Http(e) => Some(e),
            Self::Json(e) => Some(e),
            Self::Dna(e) => Some(e),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for NcbiError {
    fn from(e: reqwest::Error) -> Self {
        Self::Http(e)
    }
}

impl From<serde_json::Error> for NcbiError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

impl From<crate::error::DnaError> for NcbiError {
    fn from(e: crate::error::DnaError) -> Self {
        Self::Dna(e)
    }
}

/// Convenience Result alias for NCBI operations.
pub type Result<T> = std::result::Result<T, NcbiError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_api_error() {
        let e = NcbiError::Api {
            status: 400,
            message: "bad request".into(),
        };
        let s = format!("{e}");
        assert!(s.contains("400"));
        assert!(s.contains("bad request"));
    }

    #[test]
    fn display_rate_limited() {
        let e = NcbiError::RateLimited {
            retry_after_secs: 5,
        };
        let s = format!("{e}");
        assert!(s.contains("rate limited"));
        assert!(s.contains("5s"));
    }

    #[test]
    fn display_invalid_params() {
        let e = NcbiError::InvalidParams("empty query".into());
        let s = format!("{e}");
        assert!(s.contains("invalid parameters"));
        assert!(s.contains("empty query"));
    }

    #[test]
    fn display_dna_error() {
        let e = NcbiError::Dna(crate::error::DnaError::InvalidBase('X'));
        let s = format!("{e}");
        assert!(s.contains("DNA parse error"));
    }

    #[test]
    fn implements_error_trait() {
        let e = NcbiError::InvalidParams("test".into());
        let _: &dyn std::error::Error = &e;
    }
}
