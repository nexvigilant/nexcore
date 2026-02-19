//! # GroundsTo implementations for nexcore-signal-types
//!
//! Primitive grounding for signal detection types: `SignalMethod` and `SignalResult`.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::{SignalMethod, SignalResult};

// ============================================================================
// SignalMethod: Σ (Sum) + κ (Comparison)
// An enum of 9 signal detection algorithms -- a sum type selecting one method.
// Dominant: Sum (exclusive choice among algorithms).
// ============================================================================

impl GroundsTo for SignalMethod {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum, LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Sum, 0.90)
    }
}

// ============================================================================
// SignalResult: N (Quantity) + κ (Comparison) + ∂ (Boundary) + ∃ (Existence)
// Contains numeric estimates (point_estimate, CIs, chi_square), a boolean
// signal detection flag, and case counts. The dominant operation is quantitative
// measurement with boundary comparison.
// ============================================================================

impl GroundsTo for SignalResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Comparison,
            LexPrimitiva::Boundary,
            LexPrimitiva::Existence,
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.75)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    #[test]
    fn signal_method_grounding() {
        let comp = SignalMethod::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
        assert_eq!(comp.primitives.len(), 2);
        assert!(comp.primitives.contains(&LexPrimitiva::Sum));
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
    }

    #[test]
    fn signal_method_tier() {
        let tier = SignalMethod::tier();
        assert_eq!(tier, Tier::T2Primitive);
    }

    #[test]
    fn signal_result_grounding() {
        let comp = SignalResult::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
        assert_eq!(comp.primitives.len(), 4);
        assert!(comp.primitives.contains(&LexPrimitiva::Quantity));
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
        assert!(comp.primitives.contains(&LexPrimitiva::Existence));
    }

    #[test]
    fn signal_result_tier() {
        let tier = SignalResult::tier();
        assert_eq!(tier, Tier::T2Composite);
    }

    #[test]
    fn signal_result_not_pure() {
        assert!(!SignalResult::is_pure_primitive());
    }

    #[test]
    fn signal_method_not_pure() {
        assert!(!SignalMethod::is_pure_primitive());
    }

    #[test]
    fn signal_result_confidence() {
        let comp = SignalResult::primitive_composition();
        assert!((comp.confidence - 0.75).abs() < 1e-10);
    }

    #[test]
    fn signal_method_confidence() {
        let comp = SignalMethod::primitive_composition();
        assert!((comp.confidence - 0.90).abs() < 1e-10);
    }
}
