//! GroundsTo implementations for caesura types.
//!
//! Maps each public type to its Lex Primitiva composition.

use crate::detector::CaesuraDetector;
use crate::metrics::{ArchMetrics, DepMetrics, StyleMetrics};
use crate::types::{Caesura, CaesuraScore, CaesuraSeverity, CaesuraType, Stratum, StratumLocation};
use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

// ---------------------------------------------------------------------------
// T2-P types (1-3 unique primitives)
// ---------------------------------------------------------------------------

impl GroundsTo for CaesuraType {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary])
            .with_dominant(LexPrimitiva::Boundary, 1.0)
    }
}

impl GroundsTo for Stratum {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary])
            .with_dominant(LexPrimitiva::Boundary, 1.0)
    }
}

impl GroundsTo for CaesuraScore {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity, LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Quantity, 0.95)
    }
}

impl GroundsTo for CaesuraSeverity {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Comparison, 1.0)
    }
}

impl GroundsTo for StratumLocation {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Location])
            .with_dominant(LexPrimitiva::Location, 1.0)
    }
}

// ---------------------------------------------------------------------------
// T2-C types (4-5 unique primitives)
// ---------------------------------------------------------------------------

impl GroundsTo for Caesura {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,
            LexPrimitiva::State,
            LexPrimitiva::Irreversibility,
            LexPrimitiva::Frequency,
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.90)
    }
}

impl GroundsTo for StyleMetrics {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Comparison,
            LexPrimitiva::Sequence,
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

impl GroundsTo for ArchMetrics {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Sum,
            LexPrimitiva::Boundary,
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

impl GroundsTo for DepMetrics {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Boundary,
            LexPrimitiva::Irreversibility,
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

// ---------------------------------------------------------------------------
// T3 types (6+ unique primitives)
// ---------------------------------------------------------------------------

impl GroundsTo for CaesuraDetector {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,
            LexPrimitiva::State,
            LexPrimitiva::Irreversibility,
            LexPrimitiva::Frequency,
            LexPrimitiva::Quantity,
            LexPrimitiva::Comparison,
            LexPrimitiva::Sequence,
            LexPrimitiva::Sum,
            LexPrimitiva::Location,
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::grounding::GroundsTo;

    #[test]
    fn test_caesura_type_grounding() {
        let comp = CaesuraType::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
        assert_eq!(comp.primitives.len(), 1);
    }

    #[test]
    fn test_caesura_score_grounding() {
        let comp = CaesuraScore::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
        assert_eq!(comp.primitives.len(), 2);
    }

    #[test]
    fn test_caesura_grounding() {
        let comp = Caesura::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
        assert_eq!(comp.primitives.len(), 4);
    }

    #[test]
    fn test_style_metrics_grounding() {
        let comp = StyleMetrics::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
    }

    #[test]
    fn test_caesura_detector_grounding() {
        let comp = CaesuraDetector::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
        assert!(comp.primitives.len() >= 6); // T3
    }

    #[test]
    fn test_caesura_type_tier() {
        let tier = CaesuraType::tier();
        assert!(matches!(
            tier,
            nexcore_lex_primitiva::tier::Tier::T1Universal
                | nexcore_lex_primitiva::tier::Tier::T2Primitive
        ));
    }

    #[test]
    fn test_caesura_detector_tier() {
        let tier = CaesuraDetector::tier();
        assert!(matches!(
            tier,
            nexcore_lex_primitiva::tier::Tier::T3DomainSpecific
        ));
    }

    #[test]
    fn test_stratum_location_grounding() {
        let comp = StratumLocation::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Location));
    }

    #[test]
    fn test_dep_metrics_grounding() {
        let comp = DepMetrics::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
        assert_eq!(comp.primitives.len(), 3);
    }
}
