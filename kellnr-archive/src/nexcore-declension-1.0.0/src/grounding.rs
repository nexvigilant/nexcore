//! GroundsTo implementations for declension types.

use crate::agreement::{AgreementDimension, AgreementResult, DimensionCheck};
use crate::case::{CasedComponent, ComponentCase};
use crate::declension::{Declension, DeclinedCrate};
use crate::inflection::{Inflection, InflectionAnalysis, ToolFamily};
use crate::prodrop::{ProDropAnalysis, ProDropContext};
use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

// ---------------------------------------------------------------------------
// Case types
// ---------------------------------------------------------------------------

impl GroundsTo for ComponentCase {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Boundary,
            LexPrimitiva::Mapping,
        ])
        .with_dominant(LexPrimitiva::State, 0.90)
    }
}

impl GroundsTo for CasedComponent {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Boundary,
            LexPrimitiva::Mapping,
            LexPrimitiva::Location,
        ])
        .with_dominant(LexPrimitiva::State, 0.88)
    }
}

// ---------------------------------------------------------------------------
// Declension types
// ---------------------------------------------------------------------------

impl GroundsTo for Declension {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,
            LexPrimitiva::Sequence,
            LexPrimitiva::Comparison,
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.92)
    }
}

impl GroundsTo for DeclinedCrate {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,
            LexPrimitiva::Sequence,
            LexPrimitiva::Comparison,
            LexPrimitiva::Location,
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.88)
    }
}

// ---------------------------------------------------------------------------
// Inflection types
// ---------------------------------------------------------------------------

impl GroundsTo for Inflection {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Mapping, LexPrimitiva::Boundary])
            .with_dominant(LexPrimitiva::Mapping, 0.92)
    }
}

impl GroundsTo for ToolFamily {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,
            LexPrimitiva::Sequence,
            LexPrimitiva::Boundary,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.90)
    }
}

impl GroundsTo for InflectionAnalysis {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,
            LexPrimitiva::Sequence,
            LexPrimitiva::Boundary,
            LexPrimitiva::Quantity,
            LexPrimitiva::Comparison,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Pro-drop types
// ---------------------------------------------------------------------------

impl GroundsTo for ProDropContext {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Void,
            LexPrimitiva::Mapping,
            LexPrimitiva::State,
        ])
        .with_dominant(LexPrimitiva::Void, 0.88)
    }
}

impl GroundsTo for ProDropAnalysis {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Void, LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Void, 0.90)
    }
}

// ---------------------------------------------------------------------------
// Agreement types
// ---------------------------------------------------------------------------

impl GroundsTo for AgreementDimension {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Product])
            .with_dominant(LexPrimitiva::Product, 1.0)
    }
}

impl GroundsTo for DimensionCheck {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Product,
            LexPrimitiva::Comparison,
            LexPrimitiva::Existence,
        ])
        .with_dominant(LexPrimitiva::Product, 0.88)
    }
}

impl GroundsTo for AgreementResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Product,
            LexPrimitiva::Comparison,
            LexPrimitiva::Boundary,
            LexPrimitiva::Existence,
        ])
        .with_dominant(LexPrimitiva::Product, 0.85)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_case_grounding() {
        let comp = ComponentCase::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert_eq!(comp.primitives.len(), 3);
    }

    #[test]
    fn test_declension_grounding() {
        let comp = Declension::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
    }

    #[test]
    fn test_inflection_grounding() {
        let comp = Inflection::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
    }

    #[test]
    fn test_tool_family_grounding() {
        let comp = ToolFamily::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
    }

    #[test]
    fn test_prodrop_context_grounding() {
        let comp = ProDropContext::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Void));
    }

    #[test]
    fn test_agreement_dimension_grounding() {
        let comp = AgreementDimension::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Product));
        assert_eq!(comp.primitives.len(), 1); // T1 pure
    }

    #[test]
    fn test_agreement_result_grounding() {
        let comp = AgreementResult::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Product));
        assert_eq!(comp.primitives.len(), 4);
    }

    #[test]
    fn test_inflection_analysis_tier() {
        let tier = InflectionAnalysis::tier();
        assert!(matches!(
            tier,
            nexcore_lex_primitiva::tier::Tier::T2Composite
        ));
    }

    #[test]
    fn test_prodrop_analysis_tier() {
        let tier = ProDropAnalysis::tier();
        assert!(matches!(
            tier,
            nexcore_lex_primitiva::tier::Tier::T1Universal
                | nexcore_lex_primitiva::tier::Tier::T2Primitive
        ));
    }

    #[test]
    fn test_dimension_check_tier() {
        let tier = DimensionCheck::tier();
        assert!(matches!(
            tier,
            nexcore_lex_primitiva::tier::Tier::T2Primitive
                | nexcore_lex_primitiva::tier::Tier::T2Composite
        ));
    }
}
