//! # Spatial Bridge: nexcore-trust → stem-math
//!
//! Implements `Orient` for trust evidence (positive/negative/neutral)
//! and `Metric` for trust score distance between two engines.
//!
//! ## Primitive Foundation
//!
//! Trust has intrinsic orientation:
//! - Positive evidence → `Orientation::Positive` (trust increasing)
//! - Negative evidence → `Orientation::Negative` (trust decreasing, 2.5× asymmetric)
//! - Neutral evidence → `Orientation::Unoriented`
//!
//! Trust score difference is a valid Metric on the Beta distribution parameter space.
//! The asymmetry_factor (default 2.5) means negative orientation carries 2.5× weight.

use stem_math::spatial::{Distance, Metric, Neighborhood, Orient, Orientation};

use crate::engine::TrustEngine;
use crate::evidence::Evidence;

// ============================================================================
// Orient for Evidence
// ============================================================================

/// Orient detector for `Evidence`.
///
/// Maps the directionality of trust evidence to spatial orientation:
/// - `Positive(w)` → `Orientation::Positive` (trust-building)
/// - `Negative(w)` → `Orientation::Negative` (trust-eroding)
/// - `Neutral` → `Orientation::Unoriented`
///
/// Tier: T2-P (→ Causality + N Quantity + κ Comparison)
pub struct EvidenceOrienter;

impl Orient for EvidenceOrienter {
    type Element = Evidence;

    fn orientation(&self, element: &Evidence) -> Orientation {
        match element {
            Evidence::Positive(_) => Orientation::Positive,
            Evidence::Negative(_) => Orientation::Negative,
            Evidence::Neutral => Orientation::Unoriented,
        }
    }
}

// ============================================================================
// Metric for TrustEngine (score distance)
// ============================================================================

/// Metric measuring the distance between two trust engines' scores.
///
/// Distance = |score(a) - score(b)| where score = alpha / (alpha + beta).
/// This is a valid metric on [0,1] (Beta distribution mean space).
///
/// Tier: T2-C (N Quantity + κ Comparison + → Causality + ∂ Boundary)
pub struct TrustScoreMetric;

impl Metric for TrustScoreMetric {
    type Element = TrustEngine;

    fn distance(&self, a: &TrustEngine, b: &TrustEngine) -> Distance {
        Distance::new((a.score() - b.score()).abs())
    }
}

// ============================================================================
// Neighborhood constructors for trust thresholds
// ============================================================================

/// Trust level boundaries expressed as neighborhoods.
///
/// A trust score is within a level's neighborhood when the distance from
/// the level boundary is ≤ 0.

/// Very high trust neighborhood: scores in [0.8, 1.0].
/// Scores within Distance 0.2 of 1.0.
pub fn high_trust_neighborhood() -> Neighborhood {
    Neighborhood::closed(Distance::new(0.2))
}

/// Moderate trust neighborhood: scores in [0.4, 0.6].
/// Scores within Distance 0.1 of 0.5.
pub fn moderate_trust_neighborhood() -> Neighborhood {
    Neighborhood::closed(Distance::new(0.1))
}

/// Trust significance neighborhood.
/// A trust engine is significant when total evidence >= threshold.
pub fn significance_neighborhood(threshold: f64) -> Neighborhood {
    Neighborhood::closed(Distance::new(threshold))
}

/// Asymmetry-weighted distance between positive and negative evidence.
///
/// In trust, negative evidence has `asymmetry_factor` (default 2.5) more weight.
/// This function returns the oriented distance accounting for the asymmetry.
pub fn asymmetric_evidence_distance(
    positive_weight: f64,
    negative_weight: f64,
    asymmetry_factor: f64,
) -> Distance {
    Distance::new((positive_weight - negative_weight * asymmetry_factor).abs())
}

/// Check if two trust engines have "similar" trust (within tolerance).
pub fn trust_within(a: &TrustEngine, b: &TrustEngine, tolerance: f64) -> bool {
    let n = Neighborhood::closed(Distance::new(tolerance));
    let m = TrustScoreMetric;
    m.within(a, b, &n)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::TrustConfig;

    fn engine_with_score(alpha: f64, beta: f64) -> TrustEngine {
        TrustEngine::from_state(alpha, beta, 0, TrustConfig::default())
    }

    // ===== Evidence Orient tests =====

    #[test]
    fn positive_evidence_positive_orientation() {
        let orienter = EvidenceOrienter;
        assert_eq!(
            orienter.orientation(&Evidence::positive()),
            Orientation::Positive
        );
    }

    #[test]
    fn negative_evidence_negative_orientation() {
        let orienter = EvidenceOrienter;
        assert_eq!(
            orienter.orientation(&Evidence::negative()),
            Orientation::Negative
        );
    }

    #[test]
    fn neutral_evidence_unoriented() {
        let orienter = EvidenceOrienter;
        assert_eq!(
            orienter.orientation(&Evidence::Neutral),
            Orientation::Unoriented
        );
    }

    #[test]
    fn evidence_orientation_compose() {
        let orienter = EvidenceOrienter;
        let pos = orienter.orientation(&Evidence::positive());
        let neg = orienter.orientation(&Evidence::negative());

        // Positive + Negative = Negative (composition in sign algebra)
        assert_eq!(pos.compose(&neg), Orientation::Negative);
        // Positive + Positive = Positive
        assert_eq!(pos.compose(&pos), Orientation::Positive);
    }

    #[test]
    fn same_orientation_check() {
        let orienter = EvidenceOrienter;
        let e1 = Evidence::positive();
        let e2 = Evidence::positive_weighted(0.5);
        assert!(orienter.same_orientation(&e1, &e2));

        let e3 = Evidence::negative();
        assert!(!orienter.same_orientation(&e1, &e3));
    }

    // ===== TrustScoreMetric tests =====

    #[test]
    fn metric_non_negativity() {
        let m = TrustScoreMetric;
        let e1 = engine_with_score(5.0, 5.0);
        let e2 = engine_with_score(9.0, 1.0);
        assert!(m.distance(&e1, &e2).value() >= 0.0);
    }

    #[test]
    fn metric_identity() {
        let m = TrustScoreMetric;
        let e = engine_with_score(5.0, 5.0);
        assert!(m.distance(&e, &e).approx_eq(&Distance::ZERO, f64::EPSILON));
    }

    #[test]
    fn metric_symmetry() {
        let m = TrustScoreMetric;
        let e1 = engine_with_score(3.0, 7.0);
        let e2 = engine_with_score(8.0, 2.0);
        assert!(m.is_symmetric(&e1, &e2, f64::EPSILON));
    }

    #[test]
    fn metric_triangle_inequality() {
        let m = TrustScoreMetric;
        let a = engine_with_score(1.0, 9.0); // score ≈ 0.1
        let b = engine_with_score(5.0, 5.0); // score ≈ 0.5
        let c = engine_with_score(9.0, 1.0); // score ≈ 0.9

        let d_ab = m.distance(&a, &b);
        let d_bc = m.distance(&b, &c);
        let d_ac = m.distance(&a, &c);
        assert!(Distance::triangle_valid(d_ab, d_bc, d_ac));
    }

    // ===== Neighborhood tests =====

    #[test]
    fn high_trust_neighborhood_contains_close_scores() {
        let n = high_trust_neighborhood();
        assert!(n.contains(Distance::new(0.1))); // 0.1 < 0.2
        assert!(n.contains(Distance::new(0.2))); // boundary, closed
        assert!(!n.contains(Distance::new(0.3))); // outside
    }

    #[test]
    fn trust_within_check() {
        let a = engine_with_score(5.0, 5.0); // score = 0.5
        let b = engine_with_score(6.0, 4.0); // score = 0.6
        assert!(trust_within(&a, &b, 0.15)); // |0.5 - 0.6| = 0.1 < 0.15
        assert!(!trust_within(&a, &b, 0.05)); // 0.1 > 0.05
    }

    #[test]
    fn asymmetric_distance_reflects_factor() {
        let d_symmetric = asymmetric_evidence_distance(5.0, 5.0, 1.0);
        let d_asymmetric = asymmetric_evidence_distance(5.0, 5.0, 2.5);

        // With factor 1.0: |5 - 5*1.0| = 0
        assert!(d_symmetric.approx_eq(&Distance::ZERO, f64::EPSILON));
        // With factor 2.5: |5 - 5*2.5| = |5 - 12.5| = 7.5
        assert!((d_asymmetric.value() - 7.5).abs() < f64::EPSILON);
    }
}
