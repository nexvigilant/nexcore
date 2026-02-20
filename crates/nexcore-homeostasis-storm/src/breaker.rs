//! StormBreaker — circuit breaker for self-sustaining cascade loops.
//!
//! ## T1 Grounding
//! - `∂` (Boundary) — enforces loop gain ceiling
//! - `→` (Causality) — breaks runaway causal chains

use serde::{Deserialize, Serialize};

/// Circuit breaker that trips when cascade loop gain exceeds the safe threshold.
///
/// Once tripped, downstream cascade processing should be halted until the
/// breaker is explicitly reset by a homeostasis recovery routine.
///
/// ## Example
///
/// ```rust
/// use nexcore_homeostasis_storm::breaker::StormBreaker;
///
/// let mut breaker = StormBreaker::new(6.0);
/// assert!(!breaker.is_tripped);
///
/// breaker.trip("loop gain exceeded: 8.0 > 6.0");
/// assert!(breaker.is_tripped);
/// assert_eq!(breaker.trip_count, 1);
///
/// breaker.reset();
/// assert!(!breaker.is_tripped);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StormBreaker {
    /// Whether the breaker is currently tripped.
    pub is_tripped: bool,
    /// Cumulative trip count since the breaker was created.
    pub trip_count: u32,
    /// Human-readable reason for the most recent trip.
    pub last_trip_reason: Option<String>,
    /// Maximum loop gain before the breaker trips (∂ boundary).
    pub max_loop_gain: f64,
}

impl Default for StormBreaker {
    fn default() -> Self {
        Self {
            is_tripped: false,
            trip_count: 0,
            last_trip_reason: None,
            max_loop_gain: 6.0,
        }
    }
}

impl StormBreaker {
    /// Create a new breaker with a custom loop-gain ceiling.
    #[must_use]
    pub fn new(max_loop_gain: f64) -> Self {
        Self {
            max_loop_gain,
            ..Default::default()
        }
    }

    /// Trip the breaker, recording the reason and incrementing the trip count.
    pub fn trip(&mut self, reason: impl Into<String>) {
        self.is_tripped = true;
        self.trip_count += 1;
        self.last_trip_reason = Some(reason.into());
    }

    /// Reset the tripped state (does NOT clear `trip_count` or `last_trip_reason`).
    pub fn reset(&mut self) {
        self.is_tripped = false;
    }

    /// Return a snapshot of the current breaker status.
    #[must_use]
    pub fn status(&self) -> BreakerStatus {
        BreakerStatus {
            is_tripped: self.is_tripped,
            trip_count: self.trip_count,
            last_trip_reason: self.last_trip_reason.clone(),
        }
    }
}

/// Snapshot of breaker state, suitable for serialisation and reporting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakerStatus {
    /// Whether the breaker is currently tripped.
    pub is_tripped: bool,
    /// Cumulative trip count.
    pub trip_count: u32,
    /// Human-readable reason for the most recent trip, if any.
    pub last_trip_reason: Option<String>,
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breaker_default_not_tripped() {
        let breaker = StormBreaker::default();
        assert!(!breaker.is_tripped);
        assert_eq!(breaker.trip_count, 0);
        assert!(breaker.last_trip_reason.is_none());
    }

    #[test]
    fn test_breaker_new_custom_gain() {
        let breaker = StormBreaker::new(8.0);
        assert!((breaker.max_loop_gain - 8.0).abs() < f64::EPSILON);
        assert!(!breaker.is_tripped);
    }

    #[test]
    fn test_breaker_trip_sets_flags() {
        let mut breaker = StormBreaker::default();
        breaker.trip("loop gain 7.2 exceeded ceiling 6.0");
        assert!(breaker.is_tripped);
        assert_eq!(breaker.trip_count, 1);
        assert_eq!(
            breaker.last_trip_reason.as_deref(),
            Some("loop gain 7.2 exceeded ceiling 6.0")
        );
    }

    #[test]
    fn test_breaker_multiple_trips_accumulate() {
        let mut breaker = StormBreaker::default();
        breaker.trip("first");
        breaker.reset();
        breaker.trip("second");
        assert_eq!(breaker.trip_count, 2);
        assert!(breaker.is_tripped);
    }

    #[test]
    fn test_breaker_reset_clears_tripped_flag() {
        let mut breaker = StormBreaker::default();
        breaker.trip("test");
        breaker.reset();
        assert!(!breaker.is_tripped);
        // trip_count and last_trip_reason are preserved after reset
        assert_eq!(breaker.trip_count, 1);
        assert!(breaker.last_trip_reason.is_some());
    }

    #[test]
    fn test_breaker_status_snapshot() {
        let mut breaker = StormBreaker::new(6.0);
        breaker.trip("overflow");
        let status = breaker.status();
        assert!(status.is_tripped);
        assert_eq!(status.trip_count, 1);
        assert_eq!(status.last_trip_reason.as_deref(), Some("overflow"));
    }
}
