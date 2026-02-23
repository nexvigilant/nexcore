//! # GroundsTo Implementations
//!
//! Maps `nexcore-rh-proofs` types to the Lex Primitiva T1 primitive system.
//!
//! ## Primitive Profile
//!
//! | Type | Tier | Primitives | Dominant |
//! |------|------|-----------|---------|
//! | [`RiemannHypothesis`] | T1 | ∃ | ∃ (Existence) |
//! | [`ZeroOnCriticalLine`] | T2-C | λ + N + ∂ | λ (Location) |
//! | [`RhVerifiedToHeight`] | T3 | ∂ + N + σ + ∃ | ∂ (Boundary) |
//! | [`PrimeNumberTheorem`] | T2-C | N + ∂ + ∃ | N (Quantity) |
//! | [`NumericalCertificate`] | T3 | ∂ + N + ∃ + σ + κ | ∂ (Boundary) |
//! | [`RhProofError`] | T1 | ∂ | ∂ (Boundary) |

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::certificates::NumericalCertificate;
use crate::error::RhProofError;
use crate::propositions::{
    MertensBound, PrimeNumberTheorem, RhImpliesSharpPnt, RhVerifiedToHeight, RiemannHypothesis,
    RobinsInequality, ZeroOnCriticalLine,
};

// ============================================================================
// Proposition Types
// ============================================================================

/// RiemannHypothesis: ∃ (Existence) — T1.
///
/// The proposition is purely an existential claim: ∃ statement about zeros.
/// Dominant: Existence (∃) at confidence 1.0.
impl GroundsTo for RiemannHypothesis {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence, // ∃ — the proposition asserts existence of a pattern
        ])
        .with_dominant(LexPrimitiva::Existence, 1.0)
    }
}

/// ZeroOnCriticalLine: λ + N + ∂ — T2-C.
///
/// A zero is a location (λ) on the critical line — a specific point (N)
/// at a boundary (∂) between trivial and non-trivial regions.
/// Dominant: Location (λ) — the defining characteristic is "where" the zero is.
impl GroundsTo for ZeroOnCriticalLine {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Location, // λ — position on the critical line Re(s) = 1/2
            LexPrimitiva::Quantity, // N — the ordinal index and imaginary part t
            LexPrimitiva::Boundary, // ∂ — the critical line is a boundary
        ])
        .with_dominant(LexPrimitiva::Location, 0.80)
    }
}

/// RhVerifiedToHeight: ∂ + N + σ + ∃ — T3.
///
/// Encodes a verification **boundary** (∂) up to a height (N),
/// over a sequence (σ) of zeros, asserting their existence (∃).
/// Dominant: Boundary (∂) — the verification bound is the defining quantity.
impl GroundsTo for RhVerifiedToHeight {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,  // ∂ — the height T is a verification boundary
            LexPrimitiva::Quantity,  // N — count of verified zeros
            LexPrimitiva::Sequence,  // σ — ordered enumeration of zeros
            LexPrimitiva::Existence, // ∃ — assertion that zeros exist on critical line
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.65)
    }
}

/// PrimeNumberTheorem: N + ∂ + ∃ — T2-C.
///
/// PNT is a quantitative claim (N) about prime density up to a boundary (∂),
/// asserting the existence (∃) of the asymptotic regularity.
/// Dominant: Quantity (N) — the core is π(x) ≈ x/ln(x), a numerical fact.
impl GroundsTo for PrimeNumberTheorem {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,  // N — π(x), x/ln(x), relative error
            LexPrimitiva::Boundary,  // ∂ — the argument x defines a counting boundary
            LexPrimitiva::Existence, // ∃ — primes exist in regular density
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.75)
    }
}

/// RhImpliesSharpPnt: → + N + ∂ — T2-C.
///
/// An implication (→) from RH to a sharper quantity (N) bounded by ∂.
/// Dominant: Causality (→) — this is fundamentally an implication.
impl GroundsTo for RhImpliesSharpPnt {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality, // → — the implication RH ⟹ sharp bound
            LexPrimitiva::Quantity,  // N — the sharpened error magnitude
            LexPrimitiva::Boundary,  // ∂ — the error bound
        ])
        .with_dominant(LexPrimitiva::Causality, 0.70)
    }
}

/// RobinsInequality: N + ∂ + κ — T2-C.
///
/// An inequality (κ, Comparison) relating σ(n) (N) to a bound (∂).
/// Dominant: Comparison (κ) — σ(n) < e^γ·n·ln(ln(n)) is a comparison.
impl GroundsTo for RobinsInequality {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ — the < relation
            LexPrimitiva::Quantity,   // N — σ(n), the divisor sum
            LexPrimitiva::Boundary,   // ∂ — the threshold e^γ·n·ln(ln(n))
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.70)
    }
}

/// MertensBound: N + ∂ + κ — T2-C.
///
/// Bounds the Mertens function M(x) (N) by O(x^(1/2+ε)) (∂),
/// using comparison (κ) as the structural primitive.
/// Dominant: Comparison (κ) — |M(x)| = O(x^(1/2+ε)) is an asymptotic comparison.
impl GroundsTo for MertensBound {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ — |M(x)| bounded by O(·)
            LexPrimitiva::Quantity,   // N — the Mertens function values
            LexPrimitiva::Boundary,   // ∂ — the O(x^(1/2+ε)) growth boundary
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.70)
    }
}

// ============================================================================
// Certificate Types
// ============================================================================

/// NumericalCertificate: ∂ + N + ∃ + σ + κ — T3.
///
/// A certificate aggregates a verification boundary (∂), counts (N),
/// existence assertions (∃), an ordered structure of tests (σ),
/// and comparison results (κ).
/// Dominant: Boundary (∂) — the certificate is defined by what it bounds.
impl GroundsTo for NumericalCertificate {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // ∂ — verification height, Robin/Mertens extents
            LexPrimitiva::Quantity,   // N — zero counts, confidence score
            LexPrimitiva::Existence,  // ∃ — zeros exist on critical line
            LexPrimitiva::Sequence,   // σ — ordered collection of tests
            LexPrimitiva::Comparison, // κ — pass/fail comparisons
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.55)
    }
}

// ============================================================================
// Error Type
// ============================================================================

/// RhProofError: ∂ (Boundary) — T1.
///
/// Errors mark violations of domain boundaries: precision thresholds,
/// range constraints, overflow limits.
impl GroundsTo for RhProofError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // ∂ — constraint violation
        ])
        .with_dominant(LexPrimitiva::Boundary, 1.0)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn riemann_hypothesis_grounding() {
        assert_eq!(
            RiemannHypothesis::dominant_primitive(),
            Some(LexPrimitiva::Existence)
        );
    }

    #[test]
    fn zero_on_critical_line_grounding() {
        assert_eq!(
            ZeroOnCriticalLine::dominant_primitive(),
            Some(LexPrimitiva::Location)
        );
    }

    #[test]
    fn rh_verified_to_height_grounding() {
        assert_eq!(
            RhVerifiedToHeight::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn pnt_grounding_is_quantity_dominant() {
        assert_eq!(
            PrimeNumberTheorem::dominant_primitive(),
            Some(LexPrimitiva::Quantity)
        );
    }

    #[test]
    fn numerical_certificate_grounding() {
        assert_eq!(
            NumericalCertificate::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn error_grounding_is_boundary() {
        assert_eq!(
            RhProofError::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn robins_inequality_grounding() {
        assert_eq!(
            RobinsInequality::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }

    #[test]
    fn mertens_bound_grounding() {
        assert_eq!(
            MertensBound::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }
}
