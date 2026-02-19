//! Rate Limiter Parameters
//! Tier: T2-C (Cross-domain composed primitive)
//!
//! Token bucket and sliding window rate limiting for API/MCP call control.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for token bucket rate limiter.
///
/// Classic token bucket: tokens regenerate at a fixed rate up to a max capacity.
/// Each request consumes tokens. Request allowed if tokens >= cost.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RateLimitTokenBucketParams {
    /// Bucket identifier (e.g., "mcp_tools", "faers_api", "user:123")
    pub bucket_id: String,
    /// Maximum token capacity (default: 100)
    #[serde(default)]
    pub capacity: Option<u64>,
    /// Token refill rate per second (default: 10.0)
    #[serde(default)]
    pub refill_rate: Option<f64>,
    /// Tokens to consume for this request (default: 1)
    #[serde(default)]
    pub cost: Option<u64>,
    /// If true, only check without consuming (default: false)
    #[serde(default)]
    pub dry_run: Option<bool>,
}

/// Parameters for sliding window rate limiter.
///
/// Counts requests in a sliding time window. Rejects when count exceeds limit.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RateLimitSlidingWindowParams {
    /// Window identifier (e.g., "api_calls", "user:123:writes")
    pub window_id: String,
    /// Maximum requests allowed in the window (default: 60)
    #[serde(default)]
    pub max_requests: Option<u64>,
    /// Window duration in seconds (default: 60)
    #[serde(default)]
    pub window_secs: Option<u64>,
    /// If true, only check without recording (default: false)
    #[serde(default)]
    pub dry_run: Option<bool>,
}

/// Parameters for rate limit status query.
///
/// Returns current state of all or specific rate limit buckets/windows.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RateLimitStatusParams {
    /// Specific bucket/window ID to query (omit for all)
    #[serde(default)]
    pub id: Option<String>,
}
