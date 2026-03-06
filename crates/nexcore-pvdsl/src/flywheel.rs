//! Flywheel loop native functions for PVDSL.
//!
//! Implements the five autonomous flywheel loops as pure Rust functions,
//! inlined to avoid a dependency on `nexcore-flywheel`.
//!
//! Default threshold constants mirror [`nexcore_flywheel::FlywheelThresholds::default()`]:
//! - `RIM_CRITICAL_MARGIN` = 0.10
//! - `MIN_MOMENTUM` = 50.0
//! - `DRAG_COEFFICIENT` = 0.001
//! - `FRICTION_ACCEPTABLE` = 10.0
//! - `FRICTION_WARNING` = 50.0
//! - `GYROSCOPIC_STABLE_RATIO` = 2.0
//! - `DEFAULT_ELASTIC_MODULUS` = 1.0
//!
//! ## T1 Primitive Grounding
//! - Loop 1 Rim: κ (Comparison) + ∂ (Boundary)
//! - Loop 2 Momentum: ν (Frequency) + ∂ (Boundary)
//! - Loop 3 Friction: ν (Frequency) + ∝ (Proportionality)
//! - Loop 4 Gyroscopic: κ + ∂ + ∝
//! - Loop 5 Elastic: κ + ∂ + ν

// ── Threshold defaults (mirror FlywheelThresholds::default()) ────────────
const RIM_CRITICAL_MARGIN: f64 = 0.10;
const MIN_MOMENTUM: f64 = 50.0;
const DRAG_COEFFICIENT: f64 = 0.001;
const FRICTION_ACCEPTABLE: f64 = 10.0;
const FRICTION_WARNING: f64 = 50.0;
const GYROSCOPIC_STABLE_RATIO: f64 = 2.0;
const DEFAULT_ELASTIC_MODULUS: f64 = 1.0;

// ── Loop 1: Rim Integrity ─────────────────────────────────────────────────

/// Result of the rim integrity assessment.
pub struct RimResult {
    /// "thriving", "critical", or "disintegrated"
    pub state: &'static str,
    /// (tensile - centrifugal) / centrifugal  (or 0.0 if centrifugal ≈ 0)
    pub margin: f64,
}

/// Loop 1 — Rim Integrity: value-network self-containment.
///
/// Governing equations:
/// - ratio = tensile / centrifugal
/// - state = thriving  iff ratio > 1 + RIM_CRITICAL_MARGIN
/// - state = critical  iff 1 - RIM_CRITICAL_MARGIN <= ratio <= 1 + RIM_CRITICAL_MARGIN
/// - state = disintegrated otherwise
/// - margin = (tensile - centrifugal) / centrifugal  (or 0.0 when centrifugal ≈ 0)
#[must_use]
pub fn rim_integrity(tensile: f64, centrifugal: f64) -> RimResult {
    let ratio = if centrifugal.abs() < f64::EPSILON {
        if tensile.abs() < f64::EPSILON {
            1.0
        } else {
            f64::MAX
        }
    } else {
        tensile / centrifugal
    };
    let margin = if centrifugal.abs() < f64::EPSILON {
        0.0
    } else {
        (tensile - centrifugal) / centrifugal
    };
    let state = if ratio > 1.0 + RIM_CRITICAL_MARGIN {
        "thriving"
    } else if ratio >= 1.0 - RIM_CRITICAL_MARGIN {
        "critical"
    } else {
        "disintegrated"
    };
    RimResult { state, margin }
}

// ── Loop 2: Momentum Conservation ────────────────────────────────────────

/// Result of the momentum assessment.
pub struct MomentumResult {
    /// Angular momentum L = I * omega − friction
    pub l: f64,
    /// "high", "normal", "low", or "stalled"
    pub classification: &'static str,
}

/// Loop 2 — Momentum: inertial persistence.
///
/// Governing equations:
/// - L = I * omega − friction
/// - L > 2 * MIN_MOMENTUM → "high"
/// - L > MIN_MOMENTUM     → "normal"
/// - L > 0.5 * MIN_MOMENTUM → "low"
/// - otherwise            → "stalled"
#[must_use]
pub fn momentum(inertia: f64, omega: f64, friction: f64) -> MomentumResult {
    let l = inertia * omega - friction;
    let critical = MIN_MOMENTUM;
    let classification = if l > 2.0 * critical {
        "high"
    } else if l > critical {
        "normal"
    } else if l > 0.5 * critical {
        "low"
    } else {
        "stalled"
    };
    MomentumResult { l, classification }
}

// ── Loop 3: Friction Dissipation ─────────────────────────────────────────

/// Result of the friction assessment.
pub struct FrictionResult {
    /// Net drain after automation offset
    pub net_drain: f64,
    /// "acceptable", "warning", or "critical"
    pub classification: &'static str,
}

/// Loop 3 — Friction: parasitic drain.
///
/// Governing equations:
/// - contact = manual * touchpoints
/// - aero    = velocity^3 * DRAG_COEFFICIENT
/// - total   = contact + aero
/// - net     = total * (1 − clamp(automation, 0, 1))
/// - net < FRICTION_ACCEPTABLE → "acceptable"
/// - net < FRICTION_WARNING    → "warning"
/// - otherwise                 → "critical"
#[must_use]
pub fn friction(
    manual: f64,
    touchpoints: f64,
    velocity: f64,
    automation: f64,
) -> FrictionResult {
    let contact = manual * touchpoints;
    let aero = velocity.powi(3) * DRAG_COEFFICIENT;
    let total = contact + aero;
    let coverage = automation.clamp(0.0, 1.0);
    let net_drain = total * (1.0 - coverage);
    let classification = if net_drain < FRICTION_ACCEPTABLE {
        "acceptable"
    } else if net_drain < FRICTION_WARNING {
        "warning"
    } else {
        "critical"
    };
    FrictionResult {
        net_drain,
        classification,
    }
}

// ── Loop 4: Gyroscopic Stability ──────────────────────────────────────────

/// Result of the gyroscopic stability assessment.
pub struct GyroscopicResult {
    /// Stability score in [0, 1] (or 0 if L < critical_L)
    pub score: f64,
    /// "stable", "precessing", or "gimbal_lock"
    pub state: &'static str,
}

/// Loop 4 — Gyroscopic stability: mission alignment.
///
/// Governing equations:
/// - If L < critical_L → state = "gimbal_lock", score = 0
/// - If perturbation ≈ 0 → state = "stable", score = 1
/// - ratio = L / perturbation
/// - ratio > GYROSCOPIC_STABLE_RATIO → "stable",   score = clamp(1 − p/L, 0, 1)
/// - ratio > 1                        → "precessing", score = clamp(1 − p/L, 0, 1)
/// - otherwise                        → "gimbal_lock", score = 0
#[must_use]
pub fn gyroscopic(l: f64, perturbation: f64, critical_l: f64) -> GyroscopicResult {
    let l_abs = l.abs();
    let critical = critical_l.max(MIN_MOMENTUM);
    if l_abs < critical {
        return GyroscopicResult {
            score: 0.0,
            state: "gimbal_lock",
        };
    }
    let p_abs = perturbation.abs();
    if p_abs < f64::EPSILON {
        return GyroscopicResult {
            score: 1.0,
            state: "stable",
        };
    }
    let ratio = l_abs / p_abs;
    if ratio > GYROSCOPIC_STABLE_RATIO {
        let score = (1.0 - p_abs / l_abs).clamp(0.0, 1.0);
        GyroscopicResult {
            score,
            state: "stable",
        }
    } else if ratio > 1.0 {
        let score = (1.0 - p_abs / l_abs).clamp(0.0, 1.0);
        GyroscopicResult {
            score,
            state: "precessing",
        }
    } else {
        GyroscopicResult {
            score: 0.0,
            state: "gimbal_lock",
        }
    }
}

// ── Loop 5: Elastic Equilibrium ───────────────────────────────────────────

/// Result of the elastic equilibrium assessment.
pub struct ElasticResult {
    /// "nominal", "yield_exceeded", or "fatigue_failure_imminent"
    pub state: &'static str,
    /// stress / DEFAULT_ELASTIC_MODULUS
    pub strain: f64,
    /// fatigue_limit − fatigue_cycles (saturating, capped at i64::MAX)
    pub cycles_remaining: i64,
}

/// Loop 5 — Elastic equilibrium: adaptive capacity.
///
/// Governing equations:
/// - strain = stress / DEFAULT_ELASTIC_MODULUS
/// - fatigue_cycles > fatigue_limit → "fatigue_failure_imminent"
/// - stress >= yield_pt             → "yield_exceeded"
/// - otherwise                      → "nominal"
/// - cycles_remaining = max(0, fatigue_limit − fatigue_cycles)
#[must_use]
pub fn elastic(
    stress: f64,
    yield_pt: f64,
    fatigue_cycles: i64,
    fatigue_limit: i64,
) -> ElasticResult {
    let strain = stress / DEFAULT_ELASTIC_MODULUS;
    let effective_limit = if fatigue_limit > 0 { fatigue_limit } else { 1000 };
    let cycles_remaining = (effective_limit - fatigue_cycles).max(0);
    let state = if fatigue_cycles > effective_limit {
        "fatigue_failure_imminent"
    } else if stress >= yield_pt {
        "yield_exceeded"
    } else {
        "nominal"
    };
    ElasticResult {
        state,
        strain,
        cycles_remaining,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // Loop 1 — Rim Integrity
    #[test]
    fn rim_thriving() {
        let r = rim_integrity(200.0, 100.0);
        assert_eq!(r.state, "thriving");
        // margin = (200 - 100) / 100 = 1.0
        assert!((r.margin - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn rim_critical_equal() {
        // ratio = 1.0, within critical margin
        let r = rim_integrity(100.0, 100.0);
        assert_eq!(r.state, "critical");
    }

    #[test]
    fn rim_disintegrated() {
        let r = rim_integrity(40.0, 100.0);
        assert_eq!(r.state, "disintegrated");
        // margin = (40 - 100) / 100 = -0.6
        assert!((r.margin - (-0.6)).abs() < f64::EPSILON);
    }

    #[test]
    fn rim_zero_centrifugal_nonzero_tensile() {
        // centrifugal = 0 with nonzero tensile → ratio = MAX → thriving
        let r = rim_integrity(50.0, 0.0);
        assert_eq!(r.state, "thriving");
        assert!((r.margin).abs() < f64::EPSILON);
    }

    // Loop 2 — Momentum
    #[test]
    fn momentum_high() {
        // L = 100*5 - 0 = 500 > 2*50 = 100
        let r = momentum(100.0, 5.0, 0.0);
        assert_eq!(r.classification, "high");
        assert!((r.l - 500.0).abs() < f64::EPSILON);
    }

    #[test]
    fn momentum_stalled() {
        // L = 1*1 - 5 = -4 < 0.5*50 = 25
        let r = momentum(1.0, 1.0, 5.0);
        assert_eq!(r.classification, "stalled");
        assert!(r.l < 0.0);
    }

    #[test]
    fn momentum_normal() {
        // L = 12*5 - 0 = 60; MIN=50, 2*MIN=100 → 50 < 60 <= 100 → normal
        let r = momentum(12.0, 5.0, 0.0);
        assert_eq!(r.classification, "normal");
    }

    #[test]
    fn momentum_low() {
        // L = 10*3 - 0 = 30; 0.5*MIN=25 < 30 <= 50 → low
        let r = momentum(10.0, 3.0, 0.0);
        assert_eq!(r.classification, "low");
    }

    // Loop 3 — Friction
    #[test]
    fn friction_acceptable() {
        // contact = 1*2 = 2, aero = 1^3 * 0.001 = 0.001, total ≈ 2.001
        // net = 2.001 * (1 - 0.5) ≈ 1.0 < 10.0
        let r = friction(1.0, 2.0, 1.0, 0.5);
        assert_eq!(r.classification, "acceptable");
        assert!(r.net_drain < FRICTION_ACCEPTABLE);
    }

    #[test]
    fn friction_critical_no_automation() {
        // contact = 5*5 = 25, aero = 10^3 * 0.001 = 1.0, total = 26
        // net = 26 * (1 - 0) = 26 > 10.0 and < 50 → warning
        // Wait: let's pick numbers that push into critical
        // contact = 10*10 = 100, aero = 10^3 * 0.001 = 1, total = 101, net = 101 > 50 → critical
        let r = friction(10.0, 10.0, 10.0, 0.0);
        assert_eq!(r.classification, "critical");
        assert!(r.net_drain > FRICTION_WARNING);
    }

    #[test]
    fn friction_full_automation_zero_drain() {
        // automation = 1.0 → net = 0
        let r = friction(100.0, 100.0, 100.0, 1.0);
        assert_eq!(r.classification, "acceptable");
        assert!(r.net_drain.abs() < f64::EPSILON);
    }

    #[test]
    fn friction_warning_zone() {
        // net should land between 10 and 50
        // contact = 5*5 = 25, aero ≈ 0, total = 25, net = 25*(1-0) = 25 → warning
        let r = friction(5.0, 5.0, 0.0, 0.0);
        assert_eq!(r.classification, "warning");
    }

    // Loop 4 — Gyroscopic
    #[test]
    fn gyroscopic_stable() {
        // L=100 > critical_l=50, p=10, ratio=10 > 2.0 → stable
        // score = 1 - 10/100 = 0.9
        let r = gyroscopic(100.0, 10.0, 50.0);
        assert_eq!(r.state, "stable");
        assert!((r.score - 0.9).abs() < f64::EPSILON);
    }

    #[test]
    fn gyroscopic_gimbal_lock_low_l() {
        // L=10 < critical_l=50 → gimbal_lock
        let r = gyroscopic(10.0, 5.0, 50.0);
        assert_eq!(r.state, "gimbal_lock");
        assert_eq!(r.score, 0.0);
    }

    #[test]
    fn gyroscopic_precessing() {
        // L=100 > critical=50, p=60 → ratio=100/60≈1.67; 1 < 1.67 <= 2.0 → precessing
        let r = gyroscopic(100.0, 60.0, 50.0);
        assert_eq!(r.state, "precessing");
    }

    #[test]
    fn gyroscopic_gimbal_lock_high_perturbation() {
        // L=60 > critical=50, p=100 → ratio=0.6 <= 1 → gimbal_lock
        let r = gyroscopic(60.0, 100.0, 50.0);
        assert_eq!(r.state, "gimbal_lock");
    }

    // Loop 5 — Elastic
    #[test]
    fn elastic_nominal() {
        let r = elastic(50.0, 100.0, 100, 1000);
        assert_eq!(r.state, "nominal");
        // strain = 50 / 1.0 = 50
        assert!((r.strain - 50.0).abs() < f64::EPSILON);
        assert_eq!(r.cycles_remaining, 900);
    }

    #[test]
    fn elastic_yield_exceeded() {
        let r = elastic(150.0, 100.0, 100, 1000);
        assert_eq!(r.state, "yield_exceeded");
    }

    #[test]
    fn elastic_fatigue_failure() {
        let r = elastic(50.0, 100.0, 1500, 1000);
        assert_eq!(r.state, "fatigue_failure_imminent");
        assert_eq!(r.cycles_remaining, 0);
    }

    #[test]
    fn elastic_at_yield_boundary() {
        // stress == yield_pt → yield_exceeded (>= check)
        let r = elastic(100.0, 100.0, 0, 1000);
        assert_eq!(r.state, "yield_exceeded");
    }
}
