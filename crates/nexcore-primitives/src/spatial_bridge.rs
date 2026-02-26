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

use crate::transfer::{
    BreakerState, CircuitBreaker, DecayFunction, ExploreExploit, FeedbackLoop, Homeostasis,
    NegativeEvidence, RateLimiter, StagedValidation,
};

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
// Orient for RateLimiter
// ============================================================================

/// Orient detector for `RateLimiter`.
///
/// Positive when tokens remain (capacity available, system can serve).
/// Negative when exhausted (all tokens consumed, requests must wait).
///
/// Tier: T2-P (N Quantity + ∂ Boundary + ν Frequency)
pub struct RateLimiterOrienter;

impl Orient for RateLimiterOrienter {
    type Element = RateLimiter;

    fn orientation(&self, element: &RateLimiter) -> Orientation {
        if element.remaining() > 0 {
            Orientation::Positive // capacity available
        } else {
            Orientation::Negative // exhausted
        }
    }
}

// ============================================================================
// Orient for ExploreExploit
// ============================================================================

/// Orient detector for `ExploreExploit`.
///
/// Positive when exploring (epsilon > 0.5 — biased toward discovery).
/// Negative when exploiting (epsilon < 0.5 — biased toward known-best).
/// Unoriented at the exact balance point (epsilon == 0.5).
///
/// Tier: T2-P (κ Comparison + ∂ Boundary + ς State)
pub struct ExploreExploitOrienter;

impl Orient for ExploreExploitOrienter {
    type Element = ExploreExploit;

    fn orientation(&self, element: &ExploreExploit) -> Orientation {
        if (element.epsilon - 0.5).abs() < f64::EPSILON {
            Orientation::Unoriented // perfectly balanced
        } else if element.epsilon > 0.5 {
            Orientation::Positive // exploring
        } else {
            Orientation::Negative // exploiting
        }
    }
}

// ============================================================================
// Orient for StagedValidation
// ============================================================================

/// Orient detector for `StagedValidation`.
///
/// Positive when progressing (stages remain — validation moves forward).
/// Unoriented when complete (no further direction to move).
///
/// Tier: T2-P (σ Sequence + ∂ Boundary + → Causality)
pub struct StagedValidationOrienter;

impl Orient for StagedValidationOrienter {
    type Element = StagedValidation;

    fn orientation(&self, element: &StagedValidation) -> Orientation {
        if element.is_complete() {
            Orientation::Unoriented // validation finished, no direction
        } else {
            Orientation::Positive // progressing forward
        }
    }
}

// ============================================================================
// Orient for NegativeEvidence
// ============================================================================

/// Orient detector for `NegativeEvidence`.
///
/// Negative when absence is significant (the defining feature of negative evidence).
/// Unoriented when absence is not yet significant (insufficient observation).
///
/// Tier: T2-P (∅ Void + κ Comparison + ∃ Existence)
pub struct NegativeEvidenceOrienter;

impl Orient for NegativeEvidenceOrienter {
    type Element = NegativeEvidence;

    fn orientation(&self, element: &NegativeEvidence) -> Orientation {
        if element.is_significant() {
            Orientation::Negative // absence is meaningful
        } else {
            Orientation::Unoriented // not yet significant
        }
    }
}

// ============================================================================
// Neighborhood expression: RateLimiter capacity
// ============================================================================

/// Express a `RateLimiter` capacity as a closed `Neighborhood`.
///
/// The neighborhood represents the total capacity of the rate limiter.
/// Current usage can be checked against this boundary.
pub fn rate_limiter_capacity_neighborhood(rl: &RateLimiter) -> Neighborhood {
    Neighborhood::closed(Distance::new(rl.max_events as f64))
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

    // ===== RateLimiter Orient tests =====

    #[test]
    fn rate_limiter_positive_when_tokens_remain() {
        let orienter = RateLimiterOrienter;
        let rl = RateLimiter::new(10, 60);
        assert_eq!(orienter.orientation(&rl), Orientation::Positive);
    }

    #[test]
    fn rate_limiter_negative_when_exhausted() {
        let orienter = RateLimiterOrienter;
        let mut rl = RateLimiter::new(2, 60);
        rl.current_count = 2; // consume all tokens
        assert_eq!(orienter.orientation(&rl), Orientation::Negative);
    }

    #[test]
    fn rate_limiter_positive_partially_consumed() {
        let orienter = RateLimiterOrienter;
        let mut rl = RateLimiter::new(5, 60);
        rl.current_count = 3; // 2 remaining
        assert_eq!(orienter.orientation(&rl), Orientation::Positive);
    }

    // ===== ExploreExploit Orient tests =====

    #[test]
    fn explore_exploit_positive_when_exploring() {
        let orienter = ExploreExploitOrienter;
        let ee = ExploreExploit::new(0.8); // high exploration
        assert_eq!(orienter.orientation(&ee), Orientation::Positive);
    }

    #[test]
    fn explore_exploit_negative_when_exploiting() {
        let orienter = ExploreExploitOrienter;
        let ee = ExploreExploit::new(0.2); // low exploration
        assert_eq!(orienter.orientation(&ee), Orientation::Negative);
    }

    #[test]
    fn explore_exploit_unoriented_at_balance() {
        let orienter = ExploreExploitOrienter;
        let ee = ExploreExploit::new(0.5); // exact balance
        assert_eq!(orienter.orientation(&ee), Orientation::Unoriented);
    }

    // ===== StagedValidation Orient tests =====

    #[test]
    fn staged_validation_positive_when_progressing() {
        let orienter = StagedValidationOrienter;
        let sv = StagedValidation::new(3, 10.0);
        assert_eq!(orienter.orientation(&sv), Orientation::Positive);
    }

    #[test]
    fn staged_validation_unoriented_when_complete() {
        let orienter = StagedValidationOrienter;
        let mut sv = StagedValidation::new(2, 10.0);
        sv.current_stage = 2; // all stages done
        assert_eq!(orienter.orientation(&sv), Orientation::Unoriented);
    }

    #[test]
    fn staged_validation_positive_midway() {
        let orienter = StagedValidationOrienter;
        let mut sv = StagedValidation::new(5, 10.0);
        sv.current_stage = 3; // 3 of 5 done
        assert_eq!(orienter.orientation(&sv), Orientation::Positive);
    }

    // ===== NegativeEvidence Orient tests =====

    #[test]
    fn negative_evidence_negative_when_significant() {
        let orienter = NegativeEvidenceOrienter;
        // threshold=5.0, observed=0 → 0 < 5.0 → significant
        let ne = NegativeEvidence::new("adverse reaction", 100.0, 5.0);
        assert_eq!(orienter.orientation(&ne), Orientation::Negative);
    }

    #[test]
    fn negative_evidence_unoriented_when_not_significant() {
        let orienter = NegativeEvidenceOrienter;
        let mut ne = NegativeEvidence::new("adverse reaction", 100.0, 5.0);
        ne.observed_count = 10; // 10 >= 5.0 → not significant
        assert_eq!(orienter.orientation(&ne), Orientation::Unoriented);
    }

    // ===== RateLimiter Neighborhood tests =====

    #[test]
    fn rate_limiter_capacity_as_neighborhood() {
        let rl = RateLimiter::new(10, 60);
        let n = rate_limiter_capacity_neighborhood(&rl);
        assert!(n.contains(Distance::new(5.0))); // within capacity
        assert!(n.contains(Distance::new(10.0))); // at boundary (closed)
        assert!(!n.contains(Distance::new(11.0))); // exceeds capacity
    }

    // ===== Orientation algebra: new types =====

    #[test]
    fn orientation_compose_rate_limiter_and_negative_evidence() {
        let rl_o = RateLimiterOrienter;
        let ne_o = NegativeEvidenceOrienter;

        let rl = RateLimiter::new(10, 60); // Positive (tokens remain)
        let ne = NegativeEvidence::new("signal", 100.0, 5.0); // Negative (significant absence)

        let rl_orient = rl_o.orientation(&rl);
        let ne_orient = ne_o.orientation(&ne);

        // Positive * Negative = Negative
        assert_eq!(rl_orient.compose(&ne_orient), Orientation::Negative);
    }

    #[test]
    fn explore_exploit_same_orientation_both_exploring() {
        let orienter = ExploreExploitOrienter;
        let ee1 = ExploreExploit::new(0.7);
        let ee2 = ExploreExploit::new(0.9);
        assert!(orienter.same_orientation(&ee1, &ee2));
    }

    #[test]
    fn staged_validation_different_orientation_active_vs_complete() {
        let orienter = StagedValidationOrienter;
        let active = StagedValidation::new(3, 10.0);
        let mut complete = StagedValidation::new(2, 10.0);
        complete.current_stage = 2;
        assert!(!orienter.same_orientation(&active, &complete));
    }
}
