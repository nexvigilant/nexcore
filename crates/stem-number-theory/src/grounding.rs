//! # GroundsTo Implementations
//!
//! Connects number-theory types to the Lex Primitiva type system.
//!
//! ## Crate Primitive Profile
//!
//! stem-number-theory spans these T1 primitives:
//!
//! | Type | Primitives | Rationale |
//! |------|-----------|-----------|
//! | Factorization (Vec<(u64,u32)>) | N + σ + ρ | Quantities, prime sequence, recursive decomposition |
//! | [`MertensFunction`] | N + Σ | Count over range |
//! | [`ChebyshevTheta`] | N + Σ + ν | Logarithmic prime density |
//! | [`ChebyshevPsi`] | N + Σ + ν | Prime power density |
//! | [`NumberTheoryError`] | ∂ | Boundary violation |
//!
//! Root primitive: **N** (Quantity) dominates — number theory IS the science of quantity structure.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::{
    NumberTheoryError,
    summatory::{ChebyshevPsi, ChebyshevTheta, MertensFunction},
};

// ============================================================================
// Grounding Implementations
// ============================================================================

/// MertensFunction: Σ(Sum) + N(Quantity), dominant Sum.
///
/// M(n) is fundamentally a cumulative sum (Σ) of μ(k) quantities (N)
/// over a range. Sigma-dominant: the defining operation is accumulation.
impl GroundsTo for MertensFunction {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,      // Sigma -- cumulative summation M(n) = Σμ(k)
            LexPrimitiva::Quantity, // N     -- integer quantities being summed
        ])
        .with_dominant(LexPrimitiva::Sum, 0.80)
    }
}

/// ChebyshevTheta: Σ + N + ν(Frequency), dominant Frequency.
///
/// θ(x) = Σ ln(p) for primes p ≤ x measures prime frequency/density
/// in logarithmic scale. Frequency-dominant: the core question is
/// "how often do primes appear?" (ν).
impl GroundsTo for ChebyshevTheta {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Frequency, // nu    -- prime occurrence frequency in [1,x]
            LexPrimitiva::Sum,       // Sigma -- logarithmic sum over primes
            LexPrimitiva::Quantity,  // N     -- integer bound x
        ])
        .with_dominant(LexPrimitiva::Frequency, 0.70)
    }
}

/// ChebyshevPsi: Σ + N + ν, dominant Frequency.
///
/// ψ(x) = Σ Λ(n) for n ≤ x captures prime POWER frequency.
/// Same T1 profile as ChebyshevTheta but denser: counts prime powers
/// (p^k for all k ≥ 1) not just primes.
impl GroundsTo for ChebyshevPsi {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Frequency, // nu    -- prime power occurrence density
            LexPrimitiva::Sum,       // Sigma -- von Mangoldt accumulation
            LexPrimitiva::Quantity,  // N     -- integer bound x
        ])
        .with_dominant(LexPrimitiva::Frequency, 0.70)
    }
}

/// NumberTheoryError: ∂(Boundary).
///
/// Errors mark where computation violates a boundary constraint:
/// domain boundary (n must be positive), overflow boundary,
/// or decidability boundary (factorization failure).
impl GroundsTo for NumberTheoryError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // partial -- constraint violation marker
        ])
        .with_dominant(LexPrimitiva::Boundary, 1.00)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mertens_grounding_is_sum_dominant() {
        let dominant = MertensFunction::dominant_primitive();
        assert!(dominant.is_some());
        assert_eq!(dominant.unwrap(), LexPrimitiva::Sum);
    }

    #[test]
    fn chebyshev_theta_grounding_is_frequency_dominant() {
        let dominant = ChebyshevTheta::dominant_primitive();
        assert!(dominant.is_some());
        assert_eq!(dominant.unwrap(), LexPrimitiva::Frequency);
    }

    #[test]
    fn chebyshev_psi_grounding_is_frequency_dominant() {
        let dominant = ChebyshevPsi::dominant_primitive();
        assert!(dominant.is_some());
        assert_eq!(dominant.unwrap(), LexPrimitiva::Frequency);
    }

    #[test]
    fn error_grounding_is_boundary() {
        let dominant = NumberTheoryError::dominant_primitive();
        assert!(dominant.is_some());
        assert_eq!(dominant.unwrap(), LexPrimitiva::Boundary);
    }
}
