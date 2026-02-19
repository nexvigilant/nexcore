// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Error types for social media API operations.

use thiserror::Error;

/// Result type alias for social media operations.
pub type SocialResult<T> = Result<T, SocialError>;

/// Error types for social media API operations.
#[derive(Debug, Error)]
pub enum SocialError {
    /// HTTP request failed.
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    /// JSON parsing failed.
    #[error("JSON parsing failed: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Authentication failed.
    #[error("Authentication failed: {0}")]
    AuthError(String),

    /// Rate limit exceeded.
    #[error("Rate limit exceeded: {0}")]
    RateLimitError(String),

    /// API returned error response.
    #[error("API error: {0}")]
    ApiError(String),

    /// Invalid configuration.
    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    /// Resource not found.
    #[error("Resource not found: {0}")]
    NotFound(String),
}
