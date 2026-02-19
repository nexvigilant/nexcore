//! # Spatial Bridge: nexcore-lex-primitiva → stem-math
//!
//! Makes the tier = dimension relationship explicit:
//! `PrimitiveComposition.unique().len()` → `stem_math::spatial::Dimension`.
//!
//! ## Primitive Foundation
//!
//! The tier classification is fundamentally a dimensionality measure:
//! - T1: 1 unique primitive = 1-dimensional (scalar identity)
//! - T2-P: 2-3 unique = 2-3 dimensional (cross-domain primitive)
//! - T2-C: 4-5 unique = 4-5 dimensional (cross-domain composite)
//! - T3: 6+ unique = high-dimensional (domain-specific)
//!
//! This module formalizes that relationship using `Dimension` from stem-math.

use stem_math::spatial::Dimension;

use crate::primitiva::PrimitiveComposition;
use crate::tier::Tier;

// ============================================================================
// Dimension extension for PrimitiveComposition
// ============================================================================

/// Extension trait adding spatial dimensionality to `PrimitiveComposition`.
///
/// Tier: T2-P (N Quantity + kappa Comparison)
pub trait Dimensionality {
    /// Returns the spatial dimensionality of this composition.
    ///
    /// Dimensionality = count of unique Lex Primitiva symbols.
    /// This is isomorphic to the tier classification:
    ///
    /// | Dimension | Tier |
    /// |-----------|------|
    /// | 0-1 | T1 Universal |
    /// | 2-3 | T2-P Cross-domain Primitive |
    /// | 4-5 | T2-C Cross-domain Composite |
    /// | 6+ | T3 Domain-specific |
    fn dimensionality(&self) -> Dimension;

    /// Returns the tier corresponding to the spatial dimensionality.
    fn tier_from_dimension(&self) -> Tier;
}

impl Dimensionality for PrimitiveComposition {
    fn dimensionality(&self) -> Dimension {
        Dimension::new(self.unique().len() as u32)
    }

    fn tier_from_dimension(&self) -> Tier {
        Tier::classify(self)
    }
}

// ============================================================================
// Tier ↔ Dimension constants
// ============================================================================

/// Maximum dimension for T1 types (1 unique primitive).
pub const T1_MAX_DIMENSION: Dimension = Dimension::LINE; // 1

/// Maximum dimension for T2-P types (2-3 unique primitives).
pub const T2P_MAX_DIMENSION: Dimension = Dimension::SPACE_3D; // 3

/// Maximum dimension for T2-C types (4-5 unique primitives).
pub const T2C_MAX_DIMENSION: Dimension = Dimension::new(5);

/// Dimension threshold where types become domain-specific (T3).
pub const T3_MIN_DIMENSION: Dimension = Dimension::new(6);

// ============================================================================
// Utility functions
// ============================================================================

/// Classify a dimension count directly into a tier.
///
/// This is the inverse of `Dimensionality::dimensionality()` — given
/// a raw dimension count, determine the tier without needing a full composition.
pub fn tier_from_rank(rank: u32) -> Tier {
    match rank {
        0..=1 => Tier::T1Universal,
        2..=3 => Tier::T2Primitive,
        4..=5 => Tier::T2Composite,
        _ => Tier::T3DomainSpecific,
    }
}

/// Check if a composition's dimensionality is a subspace of another.
///
/// A T1 type's dimension is always a subspace of a T3 type's dimension.
pub fn is_subspace(inner: &PrimitiveComposition, outer: &PrimitiveComposition) -> bool {
    inner
        .dimensionality()
        .is_subspace_of(&outer.dimensionality())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitiva::LexPrimitiva;

    #[test]
    fn t1_has_dimension_1() {
        let comp = PrimitiveComposition::new(vec![LexPrimitiva::Quantity]);
        assert_eq!(comp.dimensionality().rank(), 1);
        assert_eq!(comp.tier_from_dimension(), Tier::T1Universal);
    }

    #[test]
    fn t2p_has_dimension_2_to_3() {
        let comp =
            PrimitiveComposition::new(vec![LexPrimitiva::Quantity, LexPrimitiva::Comparison]);
        assert_eq!(comp.dimensionality().rank(), 2);
        assert_eq!(comp.tier_from_dimension(), Tier::T2Primitive);
    }

    #[test]
    fn t2c_has_dimension_4_to_5() {
        let comp = PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Comparison,
            LexPrimitiva::Mapping,
            LexPrimitiva::Boundary,
        ]);
        assert_eq!(comp.dimensionality().rank(), 4);
        assert_eq!(comp.tier_from_dimension(), Tier::T2Composite);
    }

    #[test]
    fn t3_has_dimension_6_plus() {
        let comp = PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Comparison,
            LexPrimitiva::Mapping,
            LexPrimitiva::Boundary,
            LexPrimitiva::State,
            LexPrimitiva::Sequence,
        ]);
        assert_eq!(comp.dimensionality().rank(), 6);
        assert_eq!(comp.tier_from_dimension(), Tier::T3DomainSpecific);
    }

    #[test]
    fn empty_composition_dimension_zero() {
        let comp = PrimitiveComposition::new(vec![]);
        assert_eq!(comp.dimensionality().rank(), 0);
    }

    #[test]
    fn duplicates_dont_increase_dimension() {
        let comp = PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Quantity,
            LexPrimitiva::Quantity,
        ]);
        assert_eq!(comp.dimensionality().rank(), 1);
    }

    #[test]
    fn tier_from_rank_matches_classify() {
        assert_eq!(tier_from_rank(0), Tier::T1Universal);
        assert_eq!(tier_from_rank(1), Tier::T1Universal);
        assert_eq!(tier_from_rank(2), Tier::T2Primitive);
        assert_eq!(tier_from_rank(3), Tier::T2Primitive);
        assert_eq!(tier_from_rank(4), Tier::T2Composite);
        assert_eq!(tier_from_rank(5), Tier::T2Composite);
        assert_eq!(tier_from_rank(6), Tier::T3DomainSpecific);
        assert_eq!(tier_from_rank(16), Tier::T3DomainSpecific);
    }

    #[test]
    fn subspace_relationship() {
        let t1 = PrimitiveComposition::new(vec![LexPrimitiva::Quantity]);
        let t3 = PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Comparison,
            LexPrimitiva::Mapping,
            LexPrimitiva::Boundary,
            LexPrimitiva::State,
            LexPrimitiva::Sequence,
        ]);
        assert!(is_subspace(&t1, &t3)); // T1 is subspace of T3
        assert!(!is_subspace(&t3, &t1)); // T3 is NOT subspace of T1
    }

    #[test]
    fn dimension_constants_consistent() {
        assert!(T1_MAX_DIMENSION.is_subspace_of(&T2P_MAX_DIMENSION));
        assert!(T2P_MAX_DIMENSION.is_subspace_of(&T2C_MAX_DIMENSION));
        assert!(T2C_MAX_DIMENSION.is_subspace_of(&T3_MIN_DIMENSION));
    }
}
