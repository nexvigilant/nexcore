// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # RetryStrategy
//!
//! **Tier**: T2-C (nu + irrev + partial + N)
//! **Dominant**: nu (Frequency)
//!
//! Backoff/retry controller with frequency decay.
//! Models the diminishing-frequency pattern of retries over time.

use core::fmt;

/// Backoff algorithm.
///
/// ## Tier: T2-P (nu + N)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackoffKind {
    /// Constant delay between retries.
    Constant,
    /// Delay doubles each retry.
    Exponential,
    /// Delay increases linearly.
    Linear,
}

/// Outcome of a retry decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RetryDecision {
    /// Should retry after the given delay (ms).
    RetryAfter(u64),
    /// Maximum attempts exhausted — give up.
    Exhausted,
}

/// Configurable retry strategy with frequency decay.
///
/// ## Tier: T2-C (nu + irrev + partial + N)
/// Dominant: nu (Frequency)
///
/// The retry frequency decays over time:
/// - Attempt 1: base_delay_ms
/// - Attempt 2: base_delay_ms * multiplier
/// - Attempt N: base_delay_ms * multiplier^(N-1)
/// - Capped at max_delay_ms
#[derive(Debug, Clone)]
pub struct RetryStrategy {
    /// Base delay in milliseconds.
    base_delay_ms: u64,
    /// Maximum delay cap.
    max_delay_ms: u64,
    /// Maximum number of attempts.
    max_attempts: u32,
    /// Backoff algorithm.
    backoff: BackoffKind,
    /// Current attempt number (0-indexed).
    current_attempt: u32,
    /// Multiplier for exponential/linear backoff.
    multiplier: f64,
}

impl RetryStrategy {
    /// Create a new retry strategy.
    #[must_use]
    pub fn new(base_delay_ms: u64, max_attempts: u32) -> Self {
        Self {
            base_delay_ms,
            max_delay_ms: 30_000, // 30 seconds default cap
            max_attempts,
            backoff: BackoffKind::Exponential,
            current_attempt: 0,
            multiplier: 2.0,
        }
    }

    /// Use exponential backoff (default).
    #[must_use]
    pub fn exponential(mut self) -> Self {
        self.backoff = BackoffKind::Exponential;
        self
    }

    /// Use constant delay.
    #[must_use]
    pub fn constant(mut self) -> Self {
        self.backoff = BackoffKind::Constant;
        self
    }

    /// Use linear backoff.
    #[must_use]
    pub fn linear(mut self) -> Self {
        self.backoff = BackoffKind::Linear;
        self
    }

    /// Set the maximum delay cap.
    #[must_use]
    pub fn with_max_delay(mut self, max_ms: u64) -> Self {
        self.max_delay_ms = max_ms;
        self
    }

    /// Set the multiplier.
    #[must_use]
    pub fn with_multiplier(mut self, multiplier: f64) -> Self {
        self.multiplier = multiplier.max(1.0);
        self
    }

    /// Record a failure and get the retry decision.
    pub fn on_failure(&mut self) -> RetryDecision {
        if self.current_attempt >= self.max_attempts {
            return RetryDecision::Exhausted;
        }

        let delay = self.compute_delay();
        self.current_attempt += 1;

        RetryDecision::RetryAfter(delay)
    }

    /// Record a success — resets the attempt counter.
    pub fn on_success(&mut self) {
        self.current_attempt = 0;
    }

    /// Current attempt number (0-indexed).
    #[must_use]
    pub fn current_attempt(&self) -> u32 {
        self.current_attempt
    }

    /// Remaining attempts before exhaustion.
    #[must_use]
    pub fn remaining_attempts(&self) -> u32 {
        self.max_attempts.saturating_sub(self.current_attempt)
    }

    /// Whether all retries are exhausted.
    #[must_use]
    pub fn is_exhausted(&self) -> bool {
        self.current_attempt >= self.max_attempts
    }

    /// Current retry frequency (retries per second, accounting for delay).
    #[must_use]
    pub fn current_frequency_hz(&self) -> f64 {
        let delay = self.compute_delay();
        if delay == 0 {
            return 0.0;
        }
        1000.0 / delay as f64
    }

    /// Compute the delay for the current attempt.
    fn compute_delay(&self) -> u64 {
        let raw = match self.backoff {
            BackoffKind::Constant => self.base_delay_ms,
            BackoffKind::Exponential => {
                (self.base_delay_ms as f64 * self.multiplier.powi(self.current_attempt as i32))
                    as u64
            }
            BackoffKind::Linear => {
                self.base_delay_ms
                    + (self.base_delay_ms as f64 * self.current_attempt as f64) as u64
            }
        };

        raw.min(self.max_delay_ms)
    }
}

impl fmt::Display for RetryStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "RetryStrategy({:?}, attempt {}/{}, next delay {}ms)",
            self.backoff,
            self.current_attempt,
            self.max_attempts,
            self.compute_delay(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exponential_backoff() {
        let mut strategy = RetryStrategy::new(100, 5).exponential();

        // Attempt 0: 100ms
        assert!(matches!(
            strategy.on_failure(),
            RetryDecision::RetryAfter(100)
        ));
        // Attempt 1: 200ms
        assert!(matches!(
            strategy.on_failure(),
            RetryDecision::RetryAfter(200)
        ));
        // Attempt 2: 400ms
        assert!(matches!(
            strategy.on_failure(),
            RetryDecision::RetryAfter(400)
        ));
    }

    #[test]
    fn test_constant_backoff() {
        let mut strategy = RetryStrategy::new(500, 3).constant();

        assert!(matches!(
            strategy.on_failure(),
            RetryDecision::RetryAfter(500)
        ));
        assert!(matches!(
            strategy.on_failure(),
            RetryDecision::RetryAfter(500)
        ));
        assert!(matches!(
            strategy.on_failure(),
            RetryDecision::RetryAfter(500)
        ));
        assert_eq!(strategy.on_failure(), RetryDecision::Exhausted);
    }

    #[test]
    fn test_linear_backoff() {
        let mut strategy = RetryStrategy::new(100, 5).linear();

        assert!(matches!(
            strategy.on_failure(),
            RetryDecision::RetryAfter(100)
        ));
        assert!(matches!(
            strategy.on_failure(),
            RetryDecision::RetryAfter(200)
        ));
        assert!(matches!(
            strategy.on_failure(),
            RetryDecision::RetryAfter(300)
        ));
    }

    #[test]
    fn test_max_delay_cap() {
        let mut strategy = RetryStrategy::new(1000, 10)
            .exponential()
            .with_max_delay(5000);

        // Will hit cap before attempt 3 (1000 -> 2000 -> 4000 -> 8000 capped to 5000)
        strategy.on_failure(); // 1000
        strategy.on_failure(); // 2000
        strategy.on_failure(); // 4000
        let decision = strategy.on_failure(); // would be 8000, capped to 5000
        assert!(matches!(decision, RetryDecision::RetryAfter(5000)));
    }

    #[test]
    fn test_exhaustion() {
        let mut strategy = RetryStrategy::new(100, 2);
        strategy.on_failure();
        strategy.on_failure();
        assert_eq!(strategy.on_failure(), RetryDecision::Exhausted);
        assert!(strategy.is_exhausted());
    }

    #[test]
    fn test_reset_on_success() {
        let mut strategy = RetryStrategy::new(100, 3);
        strategy.on_failure();
        strategy.on_failure();
        assert_eq!(strategy.remaining_attempts(), 1);

        strategy.on_success();
        assert_eq!(strategy.remaining_attempts(), 3);
        assert!(!strategy.is_exhausted());
    }
}
