// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Token bucket rate limiter for API requests.
//!
//! ## Primitive Grounding
//!
//! | Concept | T1 Primitive | Symbol |
//! |---------|--------------|--------|
//! | Token Count | Quantity | N |
//! | Refill Rate | Frequency | ν |
//! | Bucket Limit | Boundary | ∂ |
//! | Time Tracking | Sequence | σ |

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Token bucket rate limiter.
///
/// Enforces a maximum request rate by tracking available tokens.
/// Tokens are consumed on each request and refilled over time.
pub struct RateLimiter {
    /// Maximum tokens in bucket (capacity).
    capacity: u64,
    /// Tokens to add per refill.
    refill_amount: u64,
    /// Duration between refills.
    refill_interval: Duration,
    /// Current available tokens.
    tokens: AtomicU64,
    /// Last refill time.
    last_refill: Mutex<Instant>,
}

impl RateLimiter {
    /// Create a new rate limiter.
    ///
    /// # Arguments
    ///
    /// * `requests_per_minute` - Maximum requests allowed per minute
    ///
    /// # Example
    ///
    /// ```
    /// use nexcore_social::ratelimit::RateLimiter;
    ///
    /// // Reddit API: 60 requests per minute
    /// let limiter = RateLimiter::new(60);
    /// ```
    pub fn new(requests_per_minute: u64) -> Self {
        Self {
            capacity: requests_per_minute,
            refill_amount: requests_per_minute,
            refill_interval: Duration::from_secs(60),
            tokens: AtomicU64::new(requests_per_minute),
            last_refill: Mutex::new(Instant::now()),
        }
    }

    /// Create a rate limiter with custom refill parameters.
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum tokens in bucket
    /// * `refill_amount` - Tokens to add per refill
    /// * `refill_interval` - Duration between refills
    pub fn with_params(capacity: u64, refill_amount: u64, refill_interval: Duration) -> Self {
        Self {
            capacity,
            refill_amount,
            refill_interval,
            tokens: AtomicU64::new(capacity),
            last_refill: Mutex::new(Instant::now()),
        }
    }

    /// Attempt to acquire a token.
    ///
    /// Returns `true` if a token was acquired, `false` if rate limited.
    pub async fn try_acquire(&self) -> bool {
        self.refill().await;

        loop {
            let current = self.tokens.load(Ordering::Acquire);
            if current == 0 {
                return false;
            }
            if self
                .tokens
                .compare_exchange_weak(current, current - 1, Ordering::AcqRel, Ordering::Relaxed)
                .is_ok()
            {
                return true;
            }
        }
    }

    /// Wait until a token is available, then acquire it.
    ///
    /// This will block (async) until a token can be acquired.
    pub async fn acquire(&self) {
        loop {
            if self.try_acquire().await {
                return;
            }
            // Wait a bit before retrying
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// Get current available tokens.
    pub fn available(&self) -> u64 {
        self.tokens.load(Ordering::Relaxed)
    }

    /// Refill tokens based on elapsed time.
    async fn refill(&self) {
        let mut last_refill = self.last_refill.lock().await;
        let now = Instant::now();
        let elapsed = now.duration_since(*last_refill);

        if elapsed >= self.refill_interval {
            // Calculate how many refill periods have passed
            let periods = elapsed.as_millis() / self.refill_interval.as_millis();
            let tokens_to_add = (periods as u64) * self.refill_amount;

            if tokens_to_add > 0 {
                loop {
                    let current = self.tokens.load(Ordering::Acquire);
                    let new_value = (current + tokens_to_add).min(self.capacity);
                    if self
                        .tokens
                        .compare_exchange_weak(
                            current,
                            new_value,
                            Ordering::AcqRel,
                            Ordering::Relaxed,
                        )
                        .is_ok()
                    {
                        break;
                    }
                }
                *last_refill = now;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_creation() {
        let limiter = RateLimiter::new(60);
        assert_eq!(limiter.available(), 60);
    }

    #[tokio::test]
    async fn test_acquire_decrements_tokens() {
        let limiter = RateLimiter::new(10);
        assert_eq!(limiter.available(), 10);

        let acquired = limiter.try_acquire().await;
        assert!(acquired);
        assert_eq!(limiter.available(), 9);
    }

    #[tokio::test]
    async fn test_exhausted_limiter_returns_false() {
        let limiter = RateLimiter::new(2);

        assert!(limiter.try_acquire().await);
        assert!(limiter.try_acquire().await);
        assert!(!limiter.try_acquire().await);
    }

    #[tokio::test]
    async fn test_custom_params() {
        let limiter = RateLimiter::with_params(100, 10, Duration::from_secs(1));
        assert_eq!(limiter.available(), 100);
    }
}
