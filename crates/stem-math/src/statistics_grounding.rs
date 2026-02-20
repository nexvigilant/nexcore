//! # GroundsTo implementations for statistics types
//!
//! Connects statistical inference types to the Lex Primitiva type system.
//!
//! ## Statistics Primitive Profile
//!
//! | Type | Tier | Dominant T1 | Composition |
//! |------|------|-------------|-------------|
//! | `ConfidenceInterval` | T2-P | Boundary (∂) | ∂ + N |
//! | `Significance` | T1 | State (ς) | ς |
//! | `StatisticalOutcome` | T2-C | Comparison (κ) | κ + N + ∂ + ς + μ |
//! | `ChiSquareResult` | T2-C | Comparison (κ) | κ + N + ∂ + ς |
//! | `Tail` | T1 | State (ς) | ς |

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::statistics::{
    ChiSquareResult, ConfidenceInterval, Significance, StatisticalOutcome, Tail,
};

// ===========================================================================
// ConfidenceInterval — T2-P (Boundary + Quantity), dominant Boundary
// ===========================================================================

/// ConfidenceInterval: T2-P (Boundary + Quantity), dominant Boundary
///
/// A bounded estimation region [lower, upper] at a given confidence level.
/// Boundary-dominant: the defining characteristic IS the upper/lower limits
/// that constrain where the true parameter lies.
impl GroundsTo for ConfidenceInterval {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // partial -- lower/upper bounds
            LexPrimitiva::Quantity, // N -- estimate, margin, level as numbers
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.80)
    }
}

// ===========================================================================
// Significance — T1 (State), dominant State
// ===========================================================================

/// Significance: T1 (State), dominant State
///
/// Four-variant enum classifying evidence strength.
/// Pure state: it IS the decision state of a hypothesis test
/// (reject at various α levels, or fail to reject).
impl GroundsTo for Significance {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State, // varsigma -- reject/accept decision state
        ])
        .with_dominant(LexPrimitiva::State, 0.95)
    }
}

// ===========================================================================
// StatisticalOutcome — T2-C (κ + N + ∂ + ς + μ), dominant Comparison
// ===========================================================================

/// StatisticalOutcome: T2-C (Comparison + Quantity + Boundary + State + Mapping)
///
/// The universal wrapper carrying full statistical evidence.
/// Comparison-dominant: the core purpose IS comparing observed data
/// against the null hypothesis (z-score, p-value).
///
/// Composition:
/// - Comparison (κ): p-value compares observed vs null
/// - Quantity (N): value, z-score, p-value as numbers
/// - Boundary (∂): confidence interval bounds
/// - State (ς): significance classification
/// - Mapping (μ): z-score maps raw → standardized
impl GroundsTo for StatisticalOutcome {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- observed vs null hypothesis
            LexPrimitiva::Quantity,   // N -- numeric measurements
            LexPrimitiva::Boundary,   // partial -- CI bounds
            LexPrimitiva::State,      // varsigma -- significance decision
            LexPrimitiva::Mapping,    // mu -- z-score transformation
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.70)
    }
}

// ===========================================================================
// ChiSquareResult — T2-C (κ + N + ∂ + ς), dominant Comparison
// ===========================================================================

/// ChiSquareResult: T2-C (Comparison + Quantity + Boundary + State)
///
/// Chi-square test result with statistic, df, p-value, and significance.
/// Comparison-dominant: the test IS a comparison of observed vs expected
/// frequencies.
impl GroundsTo for ChiSquareResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- observed vs expected
            LexPrimitiva::Quantity,   // N -- statistic, df, p-value
            LexPrimitiva::Boundary,   // partial -- significance threshold
            LexPrimitiva::State,      // varsigma -- significance classification
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.75)
    }
}

// ===========================================================================
// Tail — T1 (State), dominant State
// ===========================================================================

/// Tail: T1 (State), dominant State
///
/// Three-variant enum: Left, Right, Two.
/// Pure state: it IS the direction of the hypothesis test.
impl GroundsTo for Tail {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State, // varsigma -- test direction
        ])
        .with_dominant(LexPrimitiva::State, 0.95)
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    #[test]
    fn confidence_interval_is_t2p_boundary_dominant() {
        assert_eq!(ConfidenceInterval::tier(), Tier::T2Primitive);
        assert_eq!(
            ConfidenceInterval::primitive_composition().dominant,
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn significance_is_t1_state_dominant() {
        assert_eq!(Significance::tier(), Tier::T1Universal);
        assert_eq!(
            Significance::primitive_composition().dominant,
            Some(LexPrimitiva::State)
        );
    }

    #[test]
    fn statistical_outcome_is_t2c_comparison_dominant() {
        assert_eq!(StatisticalOutcome::tier(), Tier::T2Composite);
        assert_eq!(
            StatisticalOutcome::primitive_composition().dominant,
            Some(LexPrimitiva::Comparison)
        );
        assert_eq!(
            StatisticalOutcome::primitive_composition().primitives.len(),
            5
        );
    }

    #[test]
    fn chi_square_result_is_t2c_comparison_dominant() {
        assert_eq!(ChiSquareResult::tier(), Tier::T2Composite);
        assert_eq!(
            ChiSquareResult::primitive_composition().dominant,
            Some(LexPrimitiva::Comparison)
        );
    }

    #[test]
    fn tail_is_t1_state_dominant() {
        assert_eq!(Tail::tier(), Tier::T1Universal);
        assert_eq!(
            Tail::primitive_composition().dominant,
            Some(LexPrimitiva::State)
        );
    }

    #[test]
    fn statistics_tier_distribution() {
        // Should have: 2 T1, 1 T2-P, 2 T2-C
        let tiers = [
            Significance::tier(),
            Tail::tier(),
            ConfidenceInterval::tier(),
            StatisticalOutcome::tier(),
            ChiSquareResult::tier(),
        ];
        let t1 = tiers.iter().filter(|t| **t == Tier::T1Universal).count();
        let t2p = tiers.iter().filter(|t| **t == Tier::T2Primitive).count();
        let t2c = tiers.iter().filter(|t| **t == Tier::T2Composite).count();
        assert_eq!(t1, 2, "Expected 2 T1 types");
        assert_eq!(t2p, 1, "Expected 1 T2-P type");
        assert_eq!(t2c, 2, "Expected 2 T2-C types");
    }
}
