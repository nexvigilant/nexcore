//! # GroundsTo implementations for nexcore-compliance types
//!
//! Connects regulatory compliance types to the Lex Primitiva type system.
//!
//! ## ∂ (Boundary) Focus
//!
//! Compliance IS boundary enforcement: controls define boundaries,
//! assessments check boundaries, findings report boundary violations.
//! The grammar is Type-1 (context-sensitive): κ compares control status
//! against thresholds, ∂ classifies the result.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};
use nexcore_lex_primitiva::state_mode::StateMode;

use crate::dsl::{Assessment, ComplianceResult, Finding, FindingSeverity};
use crate::oscal::{Control, ControlCatalog, ControlStatus};
use crate::sam::{Exclusion, ExclusionClassification, ExclusionType};

// ---------------------------------------------------------------------------
// OSCAL types — ∂ dominant (boundary enforcement)
// ---------------------------------------------------------------------------

/// Control: T2-C (∂ · ς · λ · κ), dominant ∂
///
/// A regulatory control from a standards catalog.
/// Boundary-dominant: a control IS a boundary definition — what must be true.
impl GroundsTo for Control {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // ∂ — defines regulatory boundary
            LexPrimitiva::State,      // ς — implementation status
            LexPrimitiva::Location,   // λ — catalog location, control ID
            LexPrimitiva::Comparison, // κ — status evaluation
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.90)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

/// ControlStatus: T2-P (ς · κ), dominant ς
///
/// Implementation lifecycle state: NotImplemented → Partial → Implemented.
/// State-dominant: the status IS a state classification.
impl GroundsTo for ControlStatus {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,      // ς — lifecycle state
            LexPrimitiva::Comparison, // κ — ordered comparison
        ])
        .with_dominant(LexPrimitiva::State, 0.90)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

/// ControlCatalog: T2-C (σ · ∂ · N · λ), dominant σ
///
/// Ordered collection of regulatory controls from a single source.
/// Sequence-dominant: the catalog IS a sequence of controls.
impl GroundsTo for ControlCatalog {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence, // σ — ordered control list
            LexPrimitiva::Boundary, // ∂ — contains boundary definitions
            LexPrimitiva::Quantity, // N — compliance percentage
            LexPrimitiva::Location, // λ — catalog identity
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.85)
    }
}

// ---------------------------------------------------------------------------
// DSL types — compliance assessment
// ---------------------------------------------------------------------------

/// FindingSeverity: T2-P (κ · Σ), dominant κ
///
/// Ordinal severity: Info < Low < Medium < High < Critical.
/// Comparison-dominant: severity exists for ordered comparison.
impl GroundsTo for FindingSeverity {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ — ordinal comparison
            LexPrimitiva::Sum,        // Σ — five-variant enum
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

/// Finding: T2-C (∂ · κ · λ · →), dominant ∂
///
/// A compliance finding reporting a boundary violation.
/// Boundary-dominant: a finding IS a detected boundary condition.
impl GroundsTo for Finding {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // ∂ — boundary violation report
            LexPrimitiva::Comparison, // κ — severity classification
            LexPrimitiva::Location,   // λ — control ID reference
            LexPrimitiva::Causality,  // → — finding → remediation
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// ComplianceResult: T2-P (κ · Σ), dominant κ
///
/// Assessment outcome: Compliant | NonCompliant | Inconclusive.
/// Comparison-dominant: the result IS a pass/fail comparison.
impl GroundsTo for ComplianceResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ — pass/fail judgment
            LexPrimitiva::Sum,        // Σ — three-variant alternation
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.90)
    }
}

/// Assessment: T3 (∂ · σ · ς · κ · → · ∃), dominant ∂
///
/// Full compliance assessment session: controls + findings + result.
/// Boundary-dominant: the assessment IS a systematic boundary check.
impl GroundsTo for Assessment {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // ∂ — boundary verification
            LexPrimitiva::Sequence,   // σ — ordered controls and findings
            LexPrimitiva::State,      // ς — assessment state (result)
            LexPrimitiva::Comparison, // κ — evaluate() comparison
            LexPrimitiva::Causality,  // → — evaluation → result
            LexPrimitiva::Existence,  // ∃ — result existence check
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
        .with_state_mode(StateMode::Accumulated)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

// ---------------------------------------------------------------------------
// SAM.gov types — exclusion boundary enforcement
// ---------------------------------------------------------------------------

/// ExclusionClassification: T2-P (Σ · ∂), dominant ∂
///
/// Government exclusion classification: Individual | Firm | Vessel | Special.
/// Boundary-dominant: classification defines who is excluded (boundary).
impl GroundsTo for ExclusionClassification {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,      // Σ — four-variant alternation
            LexPrimitiva::Boundary, // ∂ — exclusion boundary
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// ExclusionType: T2-P (Σ · ∂), dominant ∂
///
/// Type of exclusion: Reciprocal | Nonreciprocal | Prohibited.
/// Boundary-dominant: defines the nature of the exclusion boundary.
impl GroundsTo for ExclusionType {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,      // Σ — three-variant alternation
            LexPrimitiva::Boundary, // ∂ — exclusion type boundary
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// Exclusion: T3 (∂ · ∃ · λ · κ · ς · π), dominant ∂
///
/// A SAM.gov exclusion record — an entity barred from government contracts.
/// Boundary-dominant: an exclusion IS an enforced boundary.
impl GroundsTo for Exclusion {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,    // ∂ — exclusion boundary
            LexPrimitiva::Existence,   // ∃ — entity exists in exclusion list
            LexPrimitiva::Location,    // λ — address, agency
            LexPrimitiva::Comparison,  // κ — classification comparison
            LexPrimitiva::State,       // ς — active/terminated status
            LexPrimitiva::Persistence, // π — date-bounded persistence
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
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
    fn control_is_boundary_dominant() {
        let comp = Control::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
        assert_eq!(Control::tier(), Tier::T2Composite);
    }

    #[test]
    fn control_status_is_state_dominant() {
        let comp = ControlStatus::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert_eq!(ControlStatus::tier(), Tier::T2Primitive);
    }

    #[test]
    fn control_catalog_is_sequence_dominant() {
        let comp = ControlCatalog::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
    }

    #[test]
    fn finding_severity_is_comparison_dominant() {
        let comp = FindingSeverity::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
    }

    #[test]
    fn finding_is_boundary_dominant() {
        let comp = Finding::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
        assert!(comp.primitives.contains(&LexPrimitiva::Causality));
    }

    #[test]
    fn compliance_result_is_comparison_dominant() {
        let comp = ComplianceResult::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
    }

    #[test]
    fn assessment_is_t3_boundary_dominant() {
        assert_eq!(Assessment::tier(), Tier::T3DomainSpecific);
        assert_eq!(
            Assessment::primitive_composition().dominant,
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn exclusion_classification_is_boundary_dominant() {
        let comp = ExclusionClassification::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
    }

    #[test]
    fn exclusion_is_t3() {
        assert_eq!(Exclusion::tier(), Tier::T3DomainSpecific);
    }

    #[test]
    fn compliance_boundary_coverage() {
        // Compliance crate is ∂-heavy — most types involve boundaries
        let types_with_boundary = [
            Control::primitive_composition()
                .primitives
                .contains(&LexPrimitiva::Boundary),
            Finding::primitive_composition()
                .primitives
                .contains(&LexPrimitiva::Boundary),
            Exclusion::primitive_composition()
                .primitives
                .contains(&LexPrimitiva::Boundary),
            ExclusionClassification::primitive_composition()
                .primitives
                .contains(&LexPrimitiva::Boundary),
        ];
        assert!(types_with_boundary.iter().all(|&has| has));
    }
}
