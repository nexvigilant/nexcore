//! Rate Limiter — token bucket and sliding window rate limiting.
//!
//! Inspired by AI Engineering Bible Section 14 (API Development & Integration):
//! implements the four standard rate limiting algorithms. Provides in-process
//! rate limiting for MCP tool calls and API requests.
//!
//! # Algorithms
//!
//! 1. **Token Bucket**: Tokens refill at a fixed rate up to capacity.
//!    Requests consume tokens. Allows bursts up to capacity.
//!
//! 2. **Sliding Window**: Counts requests within a rolling time window.
//!    Smoother than fixed window, prevents boundary burst attacks.
//!
//! # T1 Grounding: ν(Frequency) + ∂(Boundary) + ς(State) + N(Quantity)
//! - ν: Request rate tracking
//! - ∂: Rate limits as boundaries
//! - ς: Bucket/window state
//! - N: Token counts and request counts

use crate::params::{
    RateLimitSlidingWindowParams, RateLimitStatusParams, RateLimitTokenBucketParams,
};
use parking_lot::RwLock;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::LazyLock;

// ============================================================================
// State
// ============================================================================

fn now_secs() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

/// Token bucket state.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TokenBucket {
    capacity: u64,
    tokens: f64,
    refill_rate: f64, // tokens per second
    last_refill: f64, // timestamp
    total_allowed: u64,
    total_rejected: u64,
    created_at: f64,
}

impl TokenBucket {
    fn new(capacity: u64, refill_rate: f64) -> Self {
        let now = now_secs();
        Self {
            capacity,
            tokens: capacity as f64,
            refill_rate,
            last_refill: now,
            total_allowed: 0,
            total_rejected: 0,
            created_at: now,
        }
    }

    /// Refill tokens based on elapsed time.
    fn refill(&mut self) {
        let now = now_secs();
        let elapsed = now - self.last_refill;
        if elapsed > 0.0 {
            self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.capacity as f64);
            self.last_refill = now;
        }
    }

    /// Try to consume tokens. Returns true if allowed.
    fn try_consume(&mut self, cost: u64) -> bool {
        self.refill();
        if self.tokens >= cost as f64 {
            self.tokens -= cost as f64;
            self.total_allowed += 1;
            true
        } else {
            self.total_rejected += 1;
            false
        }
    }

    /// Check without consuming.
    fn check(&mut self, cost: u64) -> bool {
        self.refill();
        self.tokens >= cost as f64
    }

    fn utilization(&self) -> f64 {
        1.0 - (self.tokens / self.capacity as f64)
    }
}

/// Sliding window state.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SlidingWindow {
    max_requests: u64,
    window_secs: u64,
    timestamps: Vec<f64>,
    total_allowed: u64,
    total_rejected: u64,
    created_at: f64,
}

impl SlidingWindow {
    fn new(max_requests: u64, window_secs: u64) -> Self {
        Self {
            max_requests,
            window_secs,
            timestamps: Vec::new(),
            total_allowed: 0,
            total_rejected: 0,
            created_at: now_secs(),
        }
    }

    /// Remove expired timestamps.
    fn prune(&mut self) {
        let cutoff = now_secs() - self.window_secs as f64;
        self.timestamps.retain(|&t| t > cutoff);
    }

    /// Try to record a request. Returns true if allowed.
    fn try_record(&mut self) -> bool {
        self.prune();
        if (self.timestamps.len() as u64) < self.max_requests {
            self.timestamps.push(now_secs());
            self.total_allowed += 1;
            true
        } else {
            self.total_rejected += 1;
            false
        }
    }

    /// Check without recording.
    fn check(&mut self) -> bool {
        self.prune();
        (self.timestamps.len() as u64) < self.max_requests
    }

    fn remaining(&mut self) -> u64 {
        self.prune();
        self.max_requests
            .saturating_sub(self.timestamps.len() as u64)
    }

    fn utilization(&mut self) -> f64 {
        self.prune();
        self.timestamps.len() as f64 / self.max_requests as f64
    }

    fn reset_after_secs(&self) -> f64 {
        if self.timestamps.is_empty() {
            return 0.0;
        }
        let oldest = self.timestamps.first().copied().unwrap_or(0.0);
        let expires = oldest + self.window_secs as f64;
        (expires - now_secs()).max(0.0)
    }
}

/// Combined rate limiter state.
#[derive(Debug, Default)]
struct RateLimiterState {
    buckets: HashMap<String, TokenBucket>,
    windows: HashMap<String, SlidingWindow>,
}

static STATE: LazyLock<RwLock<RateLimiterState>> =
    LazyLock::new(|| RwLock::new(RateLimiterState::default()));

// ============================================================================
// MCP Tools
// ============================================================================

/// `rate_limit_token_bucket` — Token bucket rate limiter.
///
/// Allows bursts up to capacity, then throttles to refill rate.
/// Tokens regenerate continuously over time.
pub fn rate_limit_token_bucket(
    params: RateLimitTokenBucketParams,
) -> Result<CallToolResult, McpError> {
    let capacity = params.capacity.unwrap_or(100);
    let refill_rate = params.refill_rate.unwrap_or(10.0);
    let cost = params.cost.unwrap_or(1);
    let dry_run = params.dry_run.unwrap_or(false);

    let mut state = STATE.write();

    let bucket = state
        .buckets
        .entry(params.bucket_id.clone())
        .or_insert_with(|| TokenBucket::new(capacity, refill_rate));

    // Update config if changed
    if bucket.capacity != capacity {
        bucket.capacity = capacity;
    }
    if (bucket.refill_rate - refill_rate).abs() > f64::EPSILON {
        bucket.refill_rate = refill_rate;
    }

    let allowed = if dry_run {
        bucket.check(cost)
    } else {
        bucket.try_consume(cost)
    };

    bucket.refill(); // Ensure latest state for response

    let wait_secs = if allowed {
        0.0
    } else {
        // Time until enough tokens are available
        let deficit = cost as f64 - bucket.tokens;
        if bucket.refill_rate > 0.0 {
            deficit / bucket.refill_rate
        } else {
            f64::INFINITY
        }
    };

    let result = json!({
        "bucket_id": params.bucket_id,
        "type": "token_bucket",
        "allowed": allowed,
        "dry_run": dry_run,
        "tokens_remaining": (bucket.tokens * 100.0).round() / 100.0,
        "capacity": capacity,
        "refill_rate": refill_rate,
        "cost": cost,
        "utilization": (bucket.utilization() * 1000.0).round() / 1000.0,
        "retry_after_secs": if !allowed { Some((wait_secs * 100.0).round() / 100.0) } else { None },
        "stats": {
            "total_allowed": bucket.total_allowed,
            "total_rejected": bucket.total_rejected,
            "rejection_rate": if bucket.total_allowed + bucket.total_rejected > 0 {
                (bucket.total_rejected as f64 / (bucket.total_allowed + bucket.total_rejected) as f64 * 1000.0).round() / 1000.0
            } else {
                0.0
            },
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// `rate_limit_sliding_window` — Sliding window rate limiter.
///
/// Counts requests in a rolling time window. Smoother than fixed windows
/// and prevents boundary burst attacks.
pub fn rate_limit_sliding_window(
    params: RateLimitSlidingWindowParams,
) -> Result<CallToolResult, McpError> {
    let max_requests = params.max_requests.unwrap_or(60);
    let window_secs = params.window_secs.unwrap_or(60);
    let dry_run = params.dry_run.unwrap_or(false);

    let mut state = STATE.write();

    let window = state
        .windows
        .entry(params.window_id.clone())
        .or_insert_with(|| SlidingWindow::new(max_requests, window_secs));

    // Update config if changed
    if window.max_requests != max_requests {
        window.max_requests = max_requests;
    }
    if window.window_secs != window_secs {
        window.window_secs = window_secs;
    }

    let allowed = if dry_run {
        window.check()
    } else {
        window.try_record()
    };

    let remaining = window.remaining();
    let utilization = window.utilization();
    let reset_after = window.reset_after_secs();

    let result = json!({
        "window_id": params.window_id,
        "type": "sliding_window",
        "allowed": allowed,
        "dry_run": dry_run,
        "remaining": remaining,
        "max_requests": max_requests,
        "window_secs": window_secs,
        "utilization": (utilization * 1000.0).round() / 1000.0,
        "reset_after_secs": (reset_after * 100.0).round() / 100.0,
        "stats": {
            "total_allowed": window.total_allowed,
            "total_rejected": window.total_rejected,
            "rejection_rate": if window.total_allowed + window.total_rejected > 0 {
                (window.total_rejected as f64 / (window.total_allowed + window.total_rejected) as f64 * 1000.0).round() / 1000.0
            } else {
                0.0
            },
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// `rate_limit_status` — Query rate limiter state.
///
/// Returns current state of all or specific buckets/windows.
pub fn rate_limit_status(params: RateLimitStatusParams) -> Result<CallToolResult, McpError> {
    let mut state = STATE.write();

    if let Some(ref id) = params.id {
        // Check specific bucket or window
        if let Some(bucket) = state.buckets.get_mut(id) {
            bucket.refill();
            let result = json!({
                "id": id,
                "type": "token_bucket",
                "tokens_remaining": (bucket.tokens * 100.0).round() / 100.0,
                "capacity": bucket.capacity,
                "refill_rate": bucket.refill_rate,
                "utilization": (bucket.utilization() * 1000.0).round() / 1000.0,
                "total_allowed": bucket.total_allowed,
                "total_rejected": bucket.total_rejected,
                "age_secs": (now_secs() - bucket.created_at).round(),
            });
            return Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
            )]));
        }

        if let Some(window) = state.windows.get_mut(id) {
            let remaining = window.remaining();
            let utilization = window.utilization();
            let result = json!({
                "id": id,
                "type": "sliding_window",
                "remaining": remaining,
                "max_requests": window.max_requests,
                "window_secs": window.window_secs,
                "utilization": (utilization * 1000.0).round() / 1000.0,
                "total_allowed": window.total_allowed,
                "total_rejected": window.total_rejected,
                "age_secs": (now_secs() - window.created_at).round(),
            });
            return Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
            )]));
        }

        return Err(McpError::invalid_params(
            format!("Rate limiter '{}' not found", id),
            None,
        ));
    }

    // List all
    let buckets: Vec<serde_json::Value> = state
        .buckets
        .iter_mut()
        .map(|(id, b)| {
            b.refill();
            json!({
                "id": id,
                "type": "token_bucket",
                "tokens": (b.tokens * 10.0).round() / 10.0,
                "capacity": b.capacity,
                "utilization": (b.utilization() * 1000.0).round() / 1000.0,
                "allowed": b.total_allowed,
                "rejected": b.total_rejected,
            })
        })
        .collect();

    let windows: Vec<serde_json::Value> = state
        .windows
        .iter_mut()
        .map(|(id, w)| {
            let remaining = w.remaining();
            let utilization = w.utilization();
            json!({
                "id": id,
                "type": "sliding_window",
                "remaining": remaining,
                "max": w.max_requests,
                "utilization": (utilization * 1000.0).round() / 1000.0,
                "allowed": w.total_allowed,
                "rejected": w.total_rejected,
            })
        })
        .collect();

    let result = json!({
        "token_buckets": buckets,
        "sliding_windows": windows,
        "total_limiters": buckets.len() + windows.len(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}
