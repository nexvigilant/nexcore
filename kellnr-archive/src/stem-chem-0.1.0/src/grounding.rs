//! # GroundsTo implementations for stem-chem types
//!
//! Connects chemistry primitive types and the Chemistry composite to the
//! Lex Primitiva type system.
//!
//! ## Crate Primitive Profile
//!
//! stem-chem defines the CHEMISTRY loop with 9 traits and 5 core value types.
//! Mapping (mu) is the dominant primitive across the crate: most chemistry
//! types model transformations (substance -> ratio, reactants -> products,
//! energy -> rate).
//!
//! - **Ratio**: T2-P (Quantity + Mapping) -- substance/volume ratio
//! - **Fraction**: T2-P (Quantity + Boundary) -- clamped [0,1] proportion
//! - **Rate**: T1 (Mapping) -- time -> change rate
//! - **Affinity**: T2-P (Mapping + Boundary) -- clamped [0,1] binding strength
//! - **Balance**: T2-C (State + Quantity + Comparison) -- equilibrium state
//! - **MeasuredRatio**: T2-P (Quantity + Mapping) -- ratio with confidence
//! - **MeasuredRate**: T2-P (Quantity + Mapping) -- rate with confidence
//! - **ChemistryError**: T1 (Boundary) -- operation failure

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::{
    Affinity, Balance, ChemistryError, Fraction, MeasuredRate, MeasuredRatio, Rate, Ratio,
};

// ===========================================================================
// Core value types
// ===========================================================================

/// Ratio: T2-P (Quantity + Mapping), dominant Quantity
///
/// Substance-to-volume ratio clamped to non-negative.
/// Quantity-dominant: it IS a numeric value representing concentration.
impl GroundsTo for Ratio {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- numeric ratio value
            LexPrimitiva::Mapping,  // mu -- substance -> ratio transformation
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// Fraction: T2-P (Quantity + Boundary), dominant Quantity
///
/// A value clamped to [0.0, 1.0] representing a proportion.
/// Quantity-dominant: it IS a numeric value. Boundary appears
/// because of the [0,1] clamping.
impl GroundsTo for Fraction {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- numeric proportion value
            LexPrimitiva::Boundary, // partial -- [0,1] clamping
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// Rate: T1 (Mapping), dominant Mapping
///
/// Rate of change over time. The simplest mapping: time -> change.
/// Pure mapping: it IS a transformation rate.
impl GroundsTo for Rate {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping, // mu -- time -> change rate
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.95)
    }
}

/// Affinity: T2-P (Mapping + Boundary), dominant Mapping
///
/// Binding strength clamped to [0.0, 1.0].
/// Mapping-dominant: it IS a mapping from interaction to strength.
/// Boundary appears because of the [0,1] clamping.
impl GroundsTo for Affinity {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // mu -- interaction -> strength
            LexPrimitiva::Boundary, // partial -- [0,1] clamping
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

/// Balance: T2-C (State + Quantity + Comparison), dominant State
///
/// Equilibrium state with forward/reverse rates and constant K.
/// State-dominant: it IS a system state at equilibrium.
/// Comparison appears because equilibrium detection compares rates.
impl GroundsTo for Balance {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,      // varsigma -- equilibrium state
            LexPrimitiva::Quantity,   // N -- forward rate, reverse rate, constant K
            LexPrimitiva::Comparison, // kappa -- rate comparison for equilibrium check
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
    }
}

// ===========================================================================
// Measured types
// ===========================================================================

/// MeasuredRatio: T2-P (Quantity + Mapping), dominant Quantity
///
/// A ratio paired with confidence. Quantity-dominant: two numeric
/// values (ratio + confidence) are the primary content.
impl GroundsTo for MeasuredRatio {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- ratio value + confidence value
            LexPrimitiva::Mapping,  // mu -- substance -> measured ratio
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// MeasuredRate: T2-P (Quantity + Mapping), dominant Quantity
///
/// A rate paired with confidence. Quantity-dominant: two numeric
/// values (rate + confidence) are the primary content.
impl GroundsTo for MeasuredRate {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- rate value + confidence value
            LexPrimitiva::Mapping,  // mu -- time -> measured rate
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

// ===========================================================================
// Error type
// ===========================================================================

/// ChemistryError: T1 (Boundary), dominant Boundary
///
/// Error enum for chemistry operations. Pure boundary: each variant
/// represents a failed boundary crossing in a chemistry operation.
impl GroundsTo for ChemistryError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // partial -- operation failure boundary
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.95)
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
    fn ratio_is_t2p_quantity_dominant() {
        assert_eq!(Ratio::tier(), Tier::T2Primitive);
        assert_eq!(
            Ratio::primitive_composition().dominant,
            Some(LexPrimitiva::Quantity)
        );
    }

    #[test]
    fn fraction_is_t2p_quantity_dominant() {
        assert_eq!(Fraction::tier(), Tier::T2Primitive);
        assert_eq!(
            Fraction::primitive_composition().dominant,
            Some(LexPrimitiva::Quantity)
        );
        let comp = Fraction::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
    }

    #[test]
    fn rate_is_t1_mapping_dominant() {
        assert_eq!(Rate::tier(), Tier::T1Universal);
        assert_eq!(
            Rate::primitive_composition().dominant,
            Some(LexPrimitiva::Mapping)
        );
        assert!(Rate::is_pure_primitive());
    }

    #[test]
    fn affinity_is_t2p_mapping_dominant() {
        assert_eq!(Affinity::tier(), Tier::T2Primitive);
        assert_eq!(
            Affinity::primitive_composition().dominant,
            Some(LexPrimitiva::Mapping)
        );
        let comp = Affinity::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
    }

    #[test]
    fn balance_is_t2p_state_dominant() {
        // 3 primitives = T2-P
        assert_eq!(Balance::tier(), Tier::T2Primitive);
        assert_eq!(
            Balance::primitive_composition().dominant,
            Some(LexPrimitiva::State)
        );
        let comp = Balance::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Quantity));
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
    }

    #[test]
    fn measured_ratio_is_t2p_quantity_dominant() {
        assert_eq!(MeasuredRatio::tier(), Tier::T2Primitive);
        assert_eq!(
            MeasuredRatio::primitive_composition().dominant,
            Some(LexPrimitiva::Quantity)
        );
    }

    #[test]
    fn measured_rate_is_t2p_quantity_dominant() {
        assert_eq!(MeasuredRate::tier(), Tier::T2Primitive);
        assert_eq!(
            MeasuredRate::primitive_composition().dominant,
            Some(LexPrimitiva::Quantity)
        );
    }

    #[test]
    fn chemistry_error_is_t1_boundary_dominant() {
        assert_eq!(ChemistryError::tier(), Tier::T1Universal);
        assert_eq!(
            ChemistryError::primitive_composition().dominant,
            Some(LexPrimitiva::Boundary)
        );
        assert!(ChemistryError::is_pure_primitive());
    }

    #[test]
    fn tier_distribution_is_reasonable() {
        // T1: Rate, ChemistryError = 2
        let t1_count = [Rate::tier(), ChemistryError::tier()]
            .iter()
            .filter(|t| **t == Tier::T1Universal)
            .count();

        // T2-P (2-3 primitives): Ratio, Fraction, Affinity, Balance,
        //   MeasuredRatio, MeasuredRate = 6
        let t2p_count = [
            Ratio::tier(),
            Fraction::tier(),
            Affinity::tier(),
            Balance::tier(),
            MeasuredRatio::tier(),
            MeasuredRate::tier(),
        ]
        .iter()
        .filter(|t| **t == Tier::T2Primitive)
        .count();

        assert_eq!(t1_count, 2, "expected 2 T1 types");
        assert_eq!(t2p_count, 6, "expected 6 T2-P types");
    }
}
