//! # GroundsTo implementations for nexcore-lymphatic types
//!
//! Connects the lymphatic system model to the Lex Primitiva type system.
//!
//! ## Biological Mapping
//!
//! | Lymphatic Component | Lex Primitiva | Rationale |
//! |---------------------|---------------|-----------|
//! | OutputStyle | Σ sum + μ mapping | Variant selection of presentation mode |
//! | LymphDrainage | σ sequence + ∂ boundary + μ mapping | Ordered overflow reshaping within limits |
//! | LymphNode | ∂ boundary + κ comparison | Inspection point with domain matching |
//! | ThymicSelection | κ comparison + ∂ boundary + ∃ existence | Self vs non-self classification |
//! | ThymicVerdict | Σ sum + κ comparison | Variant outcome of thymic test |
//! | OverflowHandler | ∅ void + ∂ boundary + σ sequence | Absence-aware capacity management |
//! | LymphaticHealth | ς state + κ comparison + ∂ boundary | Observable system condition |

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::{
    LymphDrainage, LymphNode, LymphaticHealth, OutputStyle, OverflowHandler, ThymicSelection,
    ThymicVerdict,
};

// ---------------------------------------------------------------------------
// OutputStyle -- Sum dominant (T2-P)
// Variant selection: Default | Explanatory | Learning | Custom
// Like lymph choosing its collecting channel
// ---------------------------------------------------------------------------
impl GroundsTo for OutputStyle {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,     // Sigma -- 4-variant style selection
            LexPrimitiva::Mapping, // mu -- style maps content to presentation
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

// ---------------------------------------------------------------------------
// LymphDrainage -- Sequence dominant (T2-C)
// Ordered overflow reshaping: content flows through style filter
// Like lymph collecting interstitial fluid in sequence
// ---------------------------------------------------------------------------
impl GroundsTo for LymphDrainage {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence, // sigma -- ordered drain of overflow items
            LexPrimitiva::Boundary, // partial -- max_tokens capacity limit
            LexPrimitiva::Mapping,  // mu -- reshapes content to fit style
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.65)
    }
}

// ---------------------------------------------------------------------------
// LymphNode -- Boundary dominant (T2-P)
// Distributed inspection point: examines content for threats
// Like a lymph node filtering pathogens (~600 in human body)
// ---------------------------------------------------------------------------
impl GroundsTo for LymphNode {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // partial -- domain-specific filter gate
            LexPrimitiva::Comparison, // kappa -- pattern matching on content
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.80)
    }
}

// ---------------------------------------------------------------------------
// ThymicSelection -- Comparison dominant (T2-C)
// Self vs non-self: is this output style "self" (coding context)?
// Like thymic education where 95%+ thymocytes are rejected
// ---------------------------------------------------------------------------
impl GroundsTo for ThymicSelection {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- self/non-self matching
            LexPrimitiva::Boundary,   // partial -- acceptance threshold
            LexPrimitiva::Existence,  // exists -- candidate must exist to be tested
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.65)
    }
}

// ---------------------------------------------------------------------------
// ThymicVerdict -- Sum dominant (T2-P)
// Outcome of thymic test: Self_ | NonSelf | Uncertain
// Like positive/negative selection in the thymus
// ---------------------------------------------------------------------------
impl GroundsTo for ThymicVerdict {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Sigma -- 3-variant outcome
            LexPrimitiva::Comparison, // kappa -- comparison yielded this verdict
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

// ---------------------------------------------------------------------------
// OverflowHandler -- Void dominant (T2-C)
// Absence-aware capacity management: drains excess when edematous
// Like lymphatic drainage preventing tissue edema
// ---------------------------------------------------------------------------
impl GroundsTo for OverflowHandler {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Void,     // empty -- handles absence/overflow conditions
            LexPrimitiva::Boundary, // partial -- capacity limit (80% edema threshold)
            LexPrimitiva::Sequence, // sigma -- ordered overflow queue
        ])
        .with_dominant(LexPrimitiva::Void, 0.65)
    }
}

// ---------------------------------------------------------------------------
// LymphaticHealth -- State dominant (T2-C)
// Observable system condition: drainage active, edema status, etc.
// Like clinical assessment of lymphatic function
// ---------------------------------------------------------------------------
impl GroundsTo for LymphaticHealth {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,      // varsigma -- current system condition
            LexPrimitiva::Comparison, // kappa -- health checks (thresholds)
            LexPrimitiva::Boundary,   // partial -- healthy/unhealthy boundary
        ])
        .with_dominant(LexPrimitiva::State, 0.65)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    #[test]
    fn test_output_style_grounding() {
        let comp = OutputStyle::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
        assert!((comp.confidence - 0.85).abs() < f64::EPSILON);
        assert_eq!(OutputStyle::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_lymph_drainage_grounding() {
        let comp = LymphDrainage::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
        assert!((comp.confidence - 0.65).abs() < f64::EPSILON);
        assert_eq!(LymphDrainage::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_lymph_node_grounding() {
        let comp = LymphNode::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
        assert!((comp.confidence - 0.80).abs() < f64::EPSILON);
        assert_eq!(LymphNode::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_thymic_selection_grounding() {
        let comp = ThymicSelection::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
        assert!((comp.confidence - 0.65).abs() < f64::EPSILON);
        assert_eq!(ThymicSelection::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_thymic_verdict_grounding() {
        let comp = ThymicVerdict::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
        assert!((comp.confidence - 0.85).abs() < f64::EPSILON);
        assert_eq!(ThymicVerdict::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_overflow_handler_grounding() {
        let comp = OverflowHandler::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Void));
        assert!((comp.confidence - 0.65).abs() < f64::EPSILON);
        assert_eq!(OverflowHandler::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_lymphatic_health_grounding() {
        let comp = LymphaticHealth::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert!((comp.confidence - 0.65).abs() < f64::EPSILON);
        assert_eq!(LymphaticHealth::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_all_groundings_have_dominant() {
        assert!(OutputStyle::dominant_primitive().is_some());
        assert!(LymphDrainage::dominant_primitive().is_some());
        assert!(LymphNode::dominant_primitive().is_some());
        assert!(ThymicSelection::dominant_primitive().is_some());
        assert!(ThymicVerdict::dominant_primitive().is_some());
        assert!(OverflowHandler::dominant_primitive().is_some());
        assert!(LymphaticHealth::dominant_primitive().is_some());
    }

    #[test]
    fn test_no_pure_primitives() {
        // All lymphatic types are composed (T2-P or T2-C), none are pure T1
        assert!(!OutputStyle::is_pure_primitive());
        assert!(!LymphDrainage::is_pure_primitive());
        assert!(!LymphNode::is_pure_primitive());
        assert!(!ThymicSelection::is_pure_primitive());
        assert!(!ThymicVerdict::is_pure_primitive());
        assert!(!OverflowHandler::is_pure_primitive());
        assert!(!LymphaticHealth::is_pure_primitive());
    }
}
