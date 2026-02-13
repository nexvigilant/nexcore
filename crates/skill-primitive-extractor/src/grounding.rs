//! # GroundsTo implementations for skill-primitive-extractor types
//!
//! Primitive extraction types grounded to the Lex Primitiva type system.
//!
//! ## Dominant Primitive Distribution
//!
//! - `PrimitiveExtractor` -- Mapping (mu) dominant as it maps text to extracted primitives.
//! - `Primitive` -- State (varsigma) dominant as it captures classification state.
//! - `PrimitiveTier` -- Comparison (kappa) dominant as it classifies tier levels.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::{Primitive, PrimitiveExtractor, PrimitiveTier};

/// PrimitiveExtractor: T2-P (mu + kappa + sigma), dominant mu
///
/// Extracts irreducible conceptual primitives from text.
/// Mapping-dominant because the core operation is text -> Vec<Primitive>.
/// Comparison is secondary (tier classification of each term).
/// Sequence is tertiary (ordered extraction pipeline).
impl GroundsTo for PrimitiveExtractor {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,    // mu -- text -> primitives
            LexPrimitiva::Comparison, // kappa -- tier classification
            LexPrimitiva::Sequence,   // sigma -- extraction pipeline
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.90)
    }
}

/// Primitive: T2-C (varsigma + kappa + N + mu), dominant varsigma
///
/// A single extracted primitive with tier, grounding, and confidence.
/// State-dominant as it encapsulates the classification result.
/// Comparison is secondary (tier classification).
/// Quantity is tertiary (transfer_confidence f64).
/// Mapping is quaternary (term -> definition).
impl GroundsTo for Primitive {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,      // varsigma -- classification state
            LexPrimitiva::Comparison, // kappa -- tier classification
            LexPrimitiva::Quantity,   // N -- transfer_confidence score
            LexPrimitiva::Mapping,    // mu -- term -> definition
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
    }
}

/// PrimitiveTier: T1-Universal (kappa), dominant kappa
///
/// Four-level tier classification: T1, T2P, T2C, T3.
/// Pure comparison -- discriminates primitive complexity levels.
impl GroundsTo for PrimitiveTier {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- tier level comparison
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    #[test]
    fn primitive_extractor_is_t2p() {
        assert_eq!(PrimitiveExtractor::tier(), Tier::T2Primitive);
    }

    #[test]
    fn primitive_is_t2c() {
        assert_eq!(Primitive::tier(), Tier::T2Composite);
    }

    #[test]
    fn primitive_tier_is_t1() {
        assert_eq!(PrimitiveTier::tier(), Tier::T1Universal);
    }

    #[test]
    fn extractor_dominant_is_mapping() {
        let comp = PrimitiveExtractor::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
    }

    #[test]
    fn primitive_dominant_is_state() {
        let comp = Primitive::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
    }

    #[test]
    fn primitive_tier_dominant_is_comparison() {
        let comp = PrimitiveTier::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
    }
}
