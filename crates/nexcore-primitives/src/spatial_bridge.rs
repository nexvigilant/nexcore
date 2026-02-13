//! # Spatial Bridge: nexcore-primitives → stem-math
//!
//! Implements `stem_math::spatial::Orient` for directional transfer primitives
//! and expresses threshold gates as `Neighborhood` containment checks.
//!
//! ## Primitive Foundation
//!
//! Transfer primitives carry implicit orientation:
//! - `DecayFunction` → always Negative (one-way time decay, ∝ Irreversibility)
//! - `FeedbackLoop` → Positive (reinforcing) when error decreasing, Negative when amplifying
//! - `CircuitBreaker` → Positive (Closed→Open state machine has a directed cycle)
//! - `Homeostasis` → Unoriented (self-correcting toward equilibrium)

use stem_math::spatial::{Distance, Neighborhood, Orient, Orientation};

use crate::transfer::{BreakerState, CircuitBreaker, DecayFunction, FeedbackLoop, Homeostasis};

// ============================================================================
// Orient for DecayFunction
// ============================================================================

/// Orient detector for `DecayFunction`.
///
/// Decay is always Negative orientation — one-way, irreversible loss of value.
/// Tier: T2-P (→ Causality + N Quantity + ∝ Irreversibility)
pub struct DecayOrienter;

impl Orient for DecayOrienter {
    type Element = DecayFunction;

    fn orientation(&self, _element: &DecayFunction) -> Orientation {
        // Decay always flows in one direction: from initial to zero
        Orientation::Negative
    }
}

// ============================================================================
// Orient for FeedbackLoop
// ============================================================================

/// Orient detector for `FeedbackLoop`.
///
/// Positive = error is decreasing (damping, converging toward setpoint).
/// Negative = error is increasing (amplifying, diverging from setpoint).
/// Unoriented = at setpoint (zero error).
///
/// Tier: T2-P (ρ Recursion + κ Comparison + → Causality)
pub struct FeedbackOrienter;

impl Orient for FeedbackOrienter {
    type Element = FeedbackLoop;

    fn orientation(&self, element: &FeedbackLoop) -> Orientation {
        let error = element.error();
        if error.abs() < f64::EPSILON {
            Orientation::Unoriented // at setpoint
        } else if element.gain > 0.0 {
            // Positive gain with positive error → correction toward setpoint → Positive
            // This is "damping" — converging
            Orientation::Positive
        } else {
            Orientation::Negative
        }
    }
}

// ============================================================================
// Orient for CircuitBreaker
// ============================================================================

/// Orient detector for `CircuitBreaker`.
///
/// The state machine Closed → Open → HalfOpen → Closed forms a directed cycle:
/// - Closed: normal flow (Positive)
/// - Open: blocked (Negative)
/// - HalfOpen: recovery probe (Unoriented — uncertain direction)
///
/// Tier: T2-P (ς State + ∂ Boundary + κ Comparison)
pub struct BreakerOrienter;

impl Orient for BreakerOrienter {
    type Element = CircuitBreaker;

    fn orientation(&self, element: &CircuitBreaker) -> Orientation {
        match element.state {
            BreakerState::Closed => Orientation::Positive, // normal flow
            BreakerState::Open => Orientation::Negative,   // blocked
            BreakerState::HalfOpen => Orientation::Unoriented, // uncertain recovery
        }
    }
}

// ============================================================================
// Orient for Homeostasis
// ============================================================================

/// Orient detector for `Homeostasis`.
///
/// Homeostasis is self-correcting, so from an external perspective it appears
/// Unoriented (no preferred direction — it resists displacement in either direction).
///
/// When out of tolerance, the correction direction is determined by the error sign:
/// - Positive error (below setpoint) → correction upward
/// - Negative error (above setpoint) → correction downward
///
/// Tier: T2-P (ρ Recursion + κ Comparison + ∂ Boundary)
pub struct HomeostasisOrienter;

impl Orient for HomeostasisOrienter {
    type Element = Homeostasis;

    fn orientation(&self, element: &Homeostasis) -> Orientation {
        if element.in_tolerance() {
            Orientation::Unoriented // at equilibrium
        } else if element.error() > 0.0 {
            Orientation::Positive // correction upward
        } else {
            Orientation::Negative // correction downward
        }
    }
}

// ============================================================================
// Neighborhood expression of threshold logic
// ============================================================================

/// Express a `Homeostasis` tolerance band as a closed `Neighborhood`.
///
/// The homeostasis tolerance is naturally a Neighborhood: the set of all
/// values within `tolerance` distance of the setpoint.
pub fn homeostasis_neighborhood(h: &Homeostasis) -> Neighborhood {
    Neighborhood::closed(Distance::new(h.tolerance))
}

/// Express a `CircuitBreaker` failure threshold as a closed `Neighborhood`.
///
/// The breaker trips when failure count reaches the boundary of this neighborhood.
pub fn breaker_failure_neighborhood(cb: &CircuitBreaker) -> Neighborhood {
    Neighborhood::closed(Distance::new(cb.failure_threshold as f64))
}

/// Express a `DecayFunction` expiry check as neighborhood containment.
///
/// Returns true if the value at time `t` is still within the threshold neighborhood.
pub fn decay_within_threshold(decay: &DecayFunction, t: f64, threshold: f64) -> bool {
    let value = decay.value_at(t);
    let n = Neighborhood::closed(Distance::new(decay.initial_value));
    let distance_from_initial = Distance::new(decay.initial_value - value);
    let max_allowed_decay = Distance::new(decay.initial_value - threshold);

    // Within threshold = distance from initial is less than max allowed decay
    Neighborhood::closed(max_allowed_decay).contains(distance_from_initial)
        && n.contains(Distance::new(value))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ===== DecayFunction Orient tests =====

    #[test]
    fn decay_always_negative() {
        let orienter = DecayOrienter;
        let decay = DecayFunction::new(100.0, 10.0);
        assert_eq!(orienter.orientation(&decay), Orientation::Negative);
    }

    #[test]
    fn decay_same_orientation_any_params() {
        let orienter = DecayOrienter;
        let d1 = DecayFunction::new(100.0, 10.0);
        let d2 = DecayFunction::new(50.0, 5.0);
        assert!(orienter.same_orientation(&d1, &d2));
    }

    // ===== FeedbackLoop Orient tests =====

    #[test]
    fn feedback_positive_when_damping() {
        let orienter = FeedbackOrienter;
        let fb = FeedbackLoop::new(100.0, 0.5); // positive gain
        // current=0, error=100, gain>0 → damping → Positive
        assert_eq!(orienter.orientation(&fb), Orientation::Positive);
    }

    #[test]
    fn feedback_unoriented_at_setpoint() {
        let orienter = FeedbackOrienter;
        let mut fb = FeedbackLoop::new(100.0, 0.5);
        fb.current = 100.0; // at setpoint
        assert_eq!(orienter.orientation(&fb), Orientation::Unoriented);
    }

    // ===== CircuitBreaker Orient tests =====

    #[test]
    fn breaker_positive_when_closed() {
        let orienter = BreakerOrienter;
        let cb = CircuitBreaker::new(3, 1);
        assert_eq!(orienter.orientation(&cb), Orientation::Positive);
    }

    #[test]
    fn breaker_negative_when_open() {
        let orienter = BreakerOrienter;
        let mut cb = CircuitBreaker::new(1, 1);
        cb.record_failure(); // trips to Open
        assert_eq!(orienter.orientation(&cb), Orientation::Negative);
    }

    #[test]
    fn breaker_unoriented_when_halfopen() {
        let orienter = BreakerOrienter;
        let mut cb = CircuitBreaker::new(1, 1);
        cb.record_failure(); // Open
        cb.attempt_reset(); // HalfOpen
        assert_eq!(orienter.orientation(&cb), Orientation::Unoriented);
    }

    // ===== Orientation algebra tests =====

    #[test]
    fn orientation_compose_decay_and_breaker() {
        let decay_o = DecayOrienter;
        let breaker_o = BreakerOrienter;

        let decay = DecayFunction::new(100.0, 10.0);
        let cb = CircuitBreaker::new(3, 1);

        let d_orient = decay_o.orientation(&decay); // Negative
        let b_orient = breaker_o.orientation(&cb); // Positive

        // Negative * Positive = Negative (sign multiplication)
        assert_eq!(d_orient.compose(&b_orient), Orientation::Negative);
    }

    // ===== Homeostasis Orient tests =====

    #[test]
    fn homeostasis_unoriented_in_tolerance() {
        let orienter = HomeostasisOrienter;
        let h = Homeostasis::new(100.0, 5.0, 0.5);
        assert_eq!(orienter.orientation(&h), Orientation::Unoriented);
    }

    #[test]
    fn homeostasis_positive_below_setpoint() {
        let orienter = HomeostasisOrienter;
        let mut h = Homeostasis::new(100.0, 5.0, 0.5);
        h.current = 80.0; // below setpoint, error > 0
        assert_eq!(orienter.orientation(&h), Orientation::Positive);
    }

    #[test]
    fn homeostasis_negative_above_setpoint() {
        let orienter = HomeostasisOrienter;
        let mut h = Homeostasis::new(100.0, 5.0, 0.5);
        h.current = 120.0; // above setpoint, error < 0
        assert_eq!(orienter.orientation(&h), Orientation::Negative);
    }

    // ===== Neighborhood tests =====

    #[test]
    fn homeostasis_tolerance_as_neighborhood() {
        let h = Homeostasis::new(100.0, 5.0, 0.5);
        let n = homeostasis_neighborhood(&h);
        assert!(n.contains(Distance::new(3.0))); // within tolerance
        assert!(n.contains(Distance::new(5.0))); // at boundary (closed)
        assert!(!n.contains(Distance::new(6.0))); // outside tolerance
    }

    #[test]
    fn breaker_threshold_as_neighborhood() {
        let cb = CircuitBreaker::new(3, 1);
        let n = breaker_failure_neighborhood(&cb);
        assert!(n.contains(Distance::new(2.0))); // 2 failures, still closed
        assert!(n.contains(Distance::new(3.0))); // at threshold
        assert!(!n.contains(Distance::new(4.0))); // would be past threshold
    }

    #[test]
    fn decay_threshold_containment() {
        let decay = DecayFunction::new(100.0, 10.0);
        // At t=0, value=100, threshold=50 → within
        assert!(decay_within_threshold(&decay, 0.0, 50.0));
        // At t=10 (one half-life), value=50, threshold=50 → at boundary
        assert!(decay_within_threshold(&decay, 10.0, 50.0));
        // At t=20 (two half-lives), value=25, threshold=50 → expired
        assert!(!decay_within_threshold(&decay, 20.0, 50.0));
    }
}
