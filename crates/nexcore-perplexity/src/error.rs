// Copyright 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Error types for Perplexity AI API operations.
//!
//! ## Primitive Grounding
//!
//! `PerplexityError`: T2-C (boundary + causality + void), dominant boundary.
//! HTTP failures, auth errors, rate limits are all boundary violations.

use nexcore_error::Error;

/// Result type alias for Perplexity API operations.
///
/// Tier: T1 (Result is a native Rust algebraic type)
pub type PerplexityResult<T> = Result<T, PerplexityError>;

/// Error types for Perplexity AI API operations.
///
/// Tier: T2-C (Boundary + Causality + Void)
/// Dominant: boundary (most errors are boundary violations: auth, rate limit, API rejection)
#[derive(Debug, Error)]
pub enum PerplexityError {
    /// HTTP request failed (network, timeout, TLS).
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON serialization or deserialization failed.
    #[error("JSON parsing failed: {0}")]
    Json(#[from] serde_json::Error),

    /// Perplexity API returned an error response.
    #[error("API error ({status}): {message}")]
    Api {
        /// HTTP status code from the API.
        status: u16,
        /// Error message from the API response body.
        message: String,
    },

    /// API key not configured in environment.
    #[error("API key not configured: set PERPLEXITY_API_KEY environment variable")]
    MissingApiKey,

    /// Rate limit exceeded (HTTP 429).
    #[error("Rate limited: retry after {retry_after_secs}s")]
    RateLimited {
        /// Seconds to wait before retrying.
        retry_after_secs: u64,
    },

    /// Invalid request parameters.
    #[error("Invalid parameters: {0}")]
    InvalidParams(String),
}
