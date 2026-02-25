// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # AdaptivePoller
//!
//! **Tier**: T2-C (nu + kappa + partial + N)
//! **Dominant**: nu (Frequency)
//!
//! Dynamic polling rate controller that adapts frequency based on
//! observed change rates. Polls faster when data changes frequently,
//! slower when stable.

use core::fmt;

/// Polling rate bounds.
///
/// ## Tier: T2-P (nu + partial)
#[derive(Debug, Clone, Copy)]
pub struct PollBounds {
    /// Minimum interval between polls (floor).
    pub min_interval_ms: u64,
    /// Maximum interval between polls (ceiling).
    pub max_interval_ms: u64,
}

impl PollBounds {
    /// Create poll bounds.
    #[must_use]
    pub fn new(min_ms: u64, max_ms: u64) -> Self {
        Self {
            min_interval_ms: min_ms,
            max_interval_ms: max_ms.max(min_ms),
        }
    }

    /// Clamp an interval to bounds.
    #[must_use]
    pub fn clamp(&self, interval: u64) -> u64 {
        interval.clamp(self.min_interval_ms, self.max_interval_ms)
    }
}

/// Adaptive polling rate controller.
///
/// ## Tier: T2-C (nu + kappa + partial + N)
/// Dominant: nu (Frequency)
///
/// Adjusts polling interval based on observed change frequency:
/// - High change rate -> decrease interval (poll faster)
/// - Low change rate -> increase interval (poll slower)
#[derive(Debug, Clone)]
pub struct AdaptivePoller {
    /// Current polling interval in milliseconds.
    current_interval_ms: u64,
    /// Polling bounds.
    bounds: PollBounds,
    /// Number of recent polls.
    poll_count: u64,
    /// Number of polls where data changed.
    change_count: u64,
    /// Window size for change rate calculation.
    window_size: u64,
    /// Speedup factor when changes detected (0.5 = halve interval).
    speedup_factor: f64,
    /// Slowdown factor when no changes (1.5 = 50% increase).
    slowdown_factor: f64,
}

impl AdaptivePoller {
    /// Create a new adaptive poller.
    #[must_use]
    pub fn new(initial_interval_ms: u64, bounds: PollBounds) -> Self {
        Self {
            current_interval_ms: bounds.clamp(initial_interval_ms),
            bounds,
            poll_count: 0,
            change_count: 0,
            window_size: 10,
            speedup_factor: 0.7,
            slowdown_factor: 1.3,
        }
    }

    /// Record a poll result: did the data change?
    pub fn record(&mut self, data_changed: bool) {
        self.poll_count = self.poll_count.saturating_add(1);
        if data_changed {
            self.change_count = self.change_count.saturating_add(1);
        }

        // Adapt every window_size polls
        #[allow(
            clippy::arithmetic_side_effects,
            reason = "modulo check for window boundary; window_size is max(1) so no division by zero"
        )]
        if self.window_size > 0 && self.poll_count % self.window_size == 0 {
            self.adapt();
        }
    }

    /// Get the current recommended polling interval.
    #[must_use]
    pub fn interval_ms(&self) -> u64 {
        self.current_interval_ms
    }

    /// Get the current change rate (0.0 to 1.0).
    #[must_use]
    #[allow(
        clippy::as_conversions,
        reason = "u64 to f64 for ratio calculation; counts fit safely in f64"
    )]
    pub fn change_rate(&self) -> f64 {
        if self.poll_count == 0 {
            return 0.0;
        }
        self.change_count as f64 / self.poll_count as f64
    }

    /// Get the current polling frequency in Hz.
    #[must_use]
    #[allow(
        clippy::as_conversions,
        reason = "u64 to f64 for frequency calculation; interval fits safely in f64"
    )]
    pub fn frequency_hz(&self) -> f64 {
        if self.current_interval_ms == 0 {
            return 0.0;
        }
        1000.0 / self.current_interval_ms as f64
    }

    /// Total polls recorded.
    #[must_use]
    pub fn poll_count(&self) -> u64 {
        self.poll_count
    }

    /// Configure the adaptation window size.
    #[must_use]
    pub fn with_window(mut self, window_size: u64) -> Self {
        self.window_size = window_size.max(1);
        self
    }

    /// Configure speedup/slowdown factors.
    #[must_use]
    pub fn with_factors(mut self, speedup: f64, slowdown: f64) -> Self {
        self.speedup_factor = speedup.clamp(0.1, 1.0);
        self.slowdown_factor = slowdown.clamp(1.0, 10.0);
        self
    }

    /// Internal: adapt the polling interval based on recent change rate.
    #[allow(
        clippy::as_conversions,
        reason = "u64 to/from f64 for interval scaling; values are bounded by poll bounds"
    )]
    fn adapt(&mut self) {
        let rate = self.change_rate();

        let new_interval = if rate > 0.5 {
            // High change rate: speed up
            (self.current_interval_ms as f64 * self.speedup_factor) as u64
        } else if rate < 0.1 {
            // Low change rate: slow down
            (self.current_interval_ms as f64 * self.slowdown_factor) as u64
        } else {
            // Moderate: no change
            self.current_interval_ms
        };

        self.current_interval_ms = self.bounds.clamp(new_interval);

        // Reset counters for next window
        self.poll_count = 0;
        self.change_count = 0;
    }
}

impl fmt::Display for AdaptivePoller {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AdaptivePoller({}ms, {:.1}Hz, {:.0}% change rate)",
            self.current_interval_ms,
            self.frequency_hz(),
            self.change_rate() * 100.0,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let poller = AdaptivePoller::new(1000, PollBounds::new(100, 5000));
        assert_eq!(poller.interval_ms(), 1000);
        assert_eq!(poller.poll_count(), 0);
    }

    #[test]
    fn test_speeds_up_on_changes() {
        let mut poller = AdaptivePoller::new(1000, PollBounds::new(100, 5000)).with_window(5);

        // Record 5 changes (100% rate)
        for _ in 0..5 {
            poller.record(true);
        }

        // Should have sped up
        assert!(poller.interval_ms() < 1000);
    }

    #[test]
    fn test_slows_down_on_no_changes() {
        let mut poller = AdaptivePoller::new(1000, PollBounds::new(100, 5000)).with_window(5);

        // Record 5 no-changes (0% rate)
        for _ in 0..5 {
            poller.record(false);
        }

        // Should have slowed down
        assert!(poller.interval_ms() > 1000);
    }

    #[test]
    fn test_respects_bounds() {
        let mut poller = AdaptivePoller::new(100, PollBounds::new(100, 5000)).with_window(5);

        // Try to speed up from minimum
        for _ in 0..5 {
            poller.record(true);
        }

        // Should not go below minimum
        assert!(poller.interval_ms() >= 100);
    }

    #[test]
    fn test_frequency_hz() {
        let poller = AdaptivePoller::new(500, PollBounds::new(100, 5000));
        assert!((poller.frequency_hz() - 2.0).abs() < 1e-10);
    }
}
