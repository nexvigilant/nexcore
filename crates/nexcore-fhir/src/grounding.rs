//! T1 primitive grounding for FHIR pharmacovigilance types.
//!
//! | Type | Primitives | Dominant | Rationale |
//! |------|-----------|----------|-----------|
//! | AdverseEvent | → (causality) + ∂ (boundary) | → | Drug→Event causal link |
//! | SignalInput | → (causality) + N (quantity) | → | Extracted causal signal for aggregation |
//! | Patient | ∃ (existence) + λ (location) | ∃ | Entity identity |
//! | Medication | ∃ (existence) + μ (mapping) | ∃ | Substance identity with code mapping |
//! | SeverityTier | κ (comparison) + ∂ (boundary) | κ | Ordered severity classification |

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::adapter::{SeverityTier, SignalInput};
use crate::resources::{AdverseEvent, Medication, Patient};

impl GroundsTo for AdverseEvent {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,
            LexPrimitiva::Boundary,
            LexPrimitiva::Existence,
        ])
        .with_dominant(LexPrimitiva::Causality, 0.7)
    }
}

impl GroundsTo for SignalInput {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Causality, LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Causality, 0.8)
    }
}

impl GroundsTo for Patient {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Existence, LexPrimitiva::Location])
            .with_dominant(LexPrimitiva::Existence, 0.9)
    }
}

impl GroundsTo for Medication {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Existence, LexPrimitiva::Mapping])
            .with_dominant(LexPrimitiva::Existence, 0.8)
    }
}

impl GroundsTo for SeverityTier {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison, LexPrimitiva::Boundary])
            .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adverse_event_grounds_to_causality() {
        assert_eq!(
            AdverseEvent::dominant_primitive(),
            Some(LexPrimitiva::Causality)
        );
    }

    #[test]
    fn signal_input_grounds_to_causality() {
        assert_eq!(
            SignalInput::dominant_primitive(),
            Some(LexPrimitiva::Causality)
        );
    }

    #[test]
    fn severity_tier_grounds_to_comparison() {
        assert_eq!(
            SeverityTier::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }
}
