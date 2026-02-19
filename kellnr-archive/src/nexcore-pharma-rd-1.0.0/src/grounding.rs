//! # GroundsTo implementations for nexcore-pharma-rd types
//!
//! Connects pharmaceutical R&D taxonomy types to the Lex Primitiva type system.
//!
//! ## Domain Signature
//!
//! - **N (Quantity)**: present in 9/9 R&D stages -- fundamentally quantitative
//! - **∂ (Boundary)**: present in 7/9 stages -- defined by thresholds
//!
//! Note: This crate has its own internal `LexSymbol`/`Tier`/`PrimitiveComposition`
//! for the taxonomy. These GroundsTo impls bridge to the canonical
//! `nexcore-lex-primitiva` trait.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::chomsky::ChomskyLevel;
use crate::pipeline::PipelineStage;
use crate::taxonomy::{PharmaComposite, PharmaDomainConcept, PharmaPrimitive, TaxonomySummary};
use crate::transfer::{TransferConfidence, TransferDomain};

// ---------------------------------------------------------------------------
// T2-P: Classification types
// ---------------------------------------------------------------------------

/// PharmaPrimitive: T2-P (Σ + N + ∂), dominant Σ
///
/// The 24 irreducible cross-domain pharmaceutical primitives.
/// Sum-dominant: enumeration of fundamental concepts.
impl GroundsTo for PharmaPrimitive {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,      // Σ -- variant enumeration
            LexPrimitiva::Quantity, // N -- quantitative nature
            LexPrimitiva::Boundary, // ∂ -- threshold-defined
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

/// PharmaComposite: T2-P (Σ + × + N), dominant Σ
///
/// The 14 composed pharmaceutical concepts (e.g., AdmetProfile, ClinicalTrialDesign).
impl GroundsTo for PharmaComposite {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,      // Σ -- variant enumeration
            LexPrimitiva::Product,  // x -- cross-product composition
            LexPrimitiva::Quantity, // N -- quantitative metrics
        ])
        .with_dominant(LexPrimitiva::Sum, 0.80)
    }
}

/// PharmaDomainConcept: T2-P (Σ + ×), dominant Σ
///
/// The 8 full domain concepts (e.g., IndApplication, NdaSubmission, Rems).
impl GroundsTo for PharmaDomainConcept {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,     // Σ -- variant enumeration
            LexPrimitiva::Product, // x -- multi-component composition
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

/// ChomskyLevel: T2-P (κ + ρ), dominant κ
///
/// Chomsky hierarchy classification. Comparison-dominant: level ordering.
impl GroundsTo for ChomskyLevel {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ -- level comparison
            LexPrimitiva::Recursion,  // ρ -- hierarchy self-reference
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.90)
    }
}

/// PipelineStage: T2-P (σ + κ), dominant σ
///
/// R&D pipeline stage. Sequence-dominant: ordered progression.
impl GroundsTo for PipelineStage {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,   // σ -- pipeline ordering
            LexPrimitiva::Comparison, // κ -- go/no-go gates
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.85)
    }
}

/// TransferDomain: T2-P (λ + μ), dominant λ
///
/// Industry domain for cross-domain transfer analysis.
impl GroundsTo for TransferDomain {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Location, // λ -- domain identity
            LexPrimitiva::Mapping,  // μ -- transfer mapping
        ])
        .with_dominant(LexPrimitiva::Location, 0.85)
    }
}

/// TransferConfidence: T2-C (N + μ + κ + λ), dominant N
///
/// Three-dimensional transfer confidence score: structural, functional, contextual.
impl GroundsTo for TransferConfidence {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N -- confidence scores
            LexPrimitiva::Mapping,    // μ -- source → target domain
            LexPrimitiva::Comparison, // κ -- similarity assessment
            LexPrimitiva::Location,   // λ -- domain pair identity
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.80)
    }
}

/// TaxonomySummary: T2-C (Σ + N + κ + ρ), dominant Σ
///
/// Summary statistics of the taxonomy. Sum-dominant: counts per tier.
impl GroundsTo for TaxonomySummary {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Σ -- counting aggregation
            LexPrimitiva::Quantity,   // N -- counts
            LexPrimitiva::Comparison, // κ -- tier classification
            LexPrimitiva::Recursion,  // ρ -- hierarchy structure
        ])
        .with_dominant(LexPrimitiva::Sum, 0.80)
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
    fn pharma_primitive_is_sum_dominant() {
        assert_eq!(
            <PharmaPrimitive as GroundsTo>::dominant_primitive(),
            Some(LexPrimitiva::Sum)
        );
        assert_eq!(<PharmaPrimitive as GroundsTo>::tier(), Tier::T2Primitive);
    }

    #[test]
    fn chomsky_level_is_comparison_dominant() {
        assert_eq!(
            <ChomskyLevel as GroundsTo>::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }

    #[test]
    fn pipeline_stage_is_sequence_dominant() {
        assert_eq!(
            <PipelineStage as GroundsTo>::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
    }

    #[test]
    fn transfer_confidence_is_quantity_dominant() {
        assert_eq!(
            <TransferConfidence as GroundsTo>::dominant_primitive(),
            Some(LexPrimitiva::Quantity)
        );
        assert_eq!(<TransferConfidence as GroundsTo>::tier(), Tier::T2Composite);
    }

    #[test]
    fn taxonomy_summary_is_sum_dominant() {
        assert_eq!(
            <TaxonomySummary as GroundsTo>::dominant_primitive(),
            Some(LexPrimitiva::Sum)
        );
    }

    #[test]
    fn pharma_domain_concept_is_sum_dominant() {
        assert_eq!(
            <PharmaDomainConcept as GroundsTo>::dominant_primitive(),
            Some(LexPrimitiva::Sum)
        );
    }
}
