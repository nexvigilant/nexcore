//! T1 primitive grounding for bicone geometry types.
//!
//! | Type | Primitives | Dominant | Rationale |
//! |------|-----------|----------|-----------|
//! | BiconeProfile | N (quantity) + σ (sequence) + ∂ (boundary) | σ | Ordered width sequence with boundary singularity |
//! | BiconeMetrics | N (quantity) + κ (comparison) | N | Quantitative measurements for comparison |
//! | ShapeComparison | κ (comparison) + N (quantity) | κ | Similarity classification between shapes |
//! | HillActivation | N (quantity) + ∂ (boundary) | N | Response value with bottleneck threshold boundary |

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::types::{BiconeMetrics, BiconeProfile, HillActivation, ShapeComparison};

impl GroundsTo for BiconeProfile {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Sequence,
            LexPrimitiva::Boundary,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.7)
    }
}

impl GroundsTo for BiconeMetrics {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity, LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

impl GroundsTo for ShapeComparison {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison, LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Comparison, 0.9)
    }
}

impl GroundsTo for HillActivation {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity, LexPrimitiva::Boundary])
            .with_dominant(LexPrimitiva::Quantity, 0.8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bicone_profile_grounds_to_sequence() {
        assert_eq!(
            BiconeProfile::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
    }

    #[test]
    fn bicone_metrics_grounds_to_quantity() {
        assert_eq!(
            BiconeMetrics::dominant_primitive(),
            Some(LexPrimitiva::Quantity)
        );
    }

    #[test]
    fn shape_comparison_grounds_to_comparison() {
        assert_eq!(
            ShapeComparison::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }

    #[test]
    fn hill_activation_grounds_to_quantity() {
        assert_eq!(
            HillActivation::dominant_primitive(),
            Some(LexPrimitiva::Quantity)
        );
    }
}
