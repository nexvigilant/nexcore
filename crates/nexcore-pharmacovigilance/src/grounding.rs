//! # Lex Primitiva Grounding for nexcore-pharmacovigilance
//!
//! GroundsTo implementations for all nexcore-pharmacovigilance public types.
//! This crate encodes the WHO pharmacovigilance taxonomy as typed Rust.
//!
//! ## Type Grounding Table
//!
//! | Type | Primitives | Dominant | Tier | Rationale |
//! |------|-----------|----------|------|-----------|
//! | TaxonomySummary | N ς | N | T2-P | Numeric counts of taxonomy tiers |
//! | LexSymbol | Σ | Σ | T1 | Sum enum of 16 T1 symbol variants |
//! | PrimitiveComposition | σ N | σ | T2-P | Ordered list of primitives with count |
//! | Tier | Σ κ | Σ | T2-P | Ordered sum of tier classifications |
//! | DetectionConcept | Σ ∂ κ | Σ | T2-P | Sum enum of detection concepts with boundary semantics |
//! | AssessmentConcept | Σ → κ | Σ | T2-P | Sum enum of causality assessment concepts |
//! | UnderstandingConcept | Σ ρ μ | Σ | T2-P | Sum enum of understanding concepts with recursive deepening |
//! | PreventionConcept | Σ ∂ → | Σ | T2-P | Sum enum of prevention/intervention concepts |
//! | ScopeConcept | Σ ∂ λ | Σ | T2-P | Sum enum of scope/population concepts |
//! | PvComposite | Σ × | Σ | T2-P | Sum enum of PV composite concepts |
//! | PvPrimitive | Σ | Σ | T1 | Sum enum of PV T2-P primitive concepts |
//! | RegulatoryConcept | Σ ∂ | Σ | T2-P | Sum enum of regulatory concepts with boundary enforcement |
//! | InfrastructureConcept | Σ ς π | Σ | T2-P | Sum enum of infrastructure concepts |
//! | OperationsConcept | Σ σ | Σ | T2-P | Sum enum of operational concepts |
//! | AnalyticsConcept | Σ N κ | Σ | T2-P | Sum enum of analytics concepts |
//! | SafetyCommsConcept | Σ → | Σ | T2-P | Sum enum of safety communication concepts |
//! | SpecialPopulationConcept | Σ ∂ | Σ | T2-P | Sum enum of special population concepts |
//! | ChomskyLevel | Σ κ | Σ | T2-P | Ordered Chomsky hierarchy levels |
//! | PvSubsystem | Σ σ | Σ | T2-P | Sum enum of PV subsystem classifications |
//! | TransferDomain | Σ | Σ | T1 | Sum enum of transfer target domains |
//! | TransferConfidence | N κ × | N | T2-P | Numeric confidence with product structure |

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::TaxonomySummary;
use crate::analytics::{AnalyticsConcept, SafetyCommsConcept, SpecialPopulationConcept};
use crate::assessment::AssessmentConcept;
use crate::chomsky::{ChomskyLevel, PvSubsystem};
use crate::composites::PvComposite;
use crate::detection::DetectionConcept;
use crate::lex::{LexSymbol, PrimitiveComposition as PvPrimitiveComposition, Tier as PvTier};
use crate::prevention::{PreventionConcept, ScopeConcept};
use crate::primitives::PvPrimitive;
use crate::regulatory::{InfrastructureConcept, OperationsConcept, RegulatoryConcept};
use crate::transfer::{TransferConfidence, TransferDomain};
use crate::understanding::UnderstandingConcept;

// ============================================================================
// T1 Universal (1 unique primitive)
// ============================================================================

/// LexSymbol: Sum enum of 16 Lex Primitiva symbols.
/// Tier: T1Universal. Dominant: Σ Sum.
/// WHY: One-of-16 exclusive classification of T1 primitives.
impl GroundsTo for LexSymbol {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum, // Σ -- one-of-16 variant
        ])
        .with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

/// PvPrimitive: Sum enum of T2-P primitive concepts.
/// Tier: T1Universal. Dominant: Σ Sum.
/// WHY: One-of-22 exclusive classification of PV primitives.
impl GroundsTo for PvPrimitive {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum, // Σ -- one-of-22 variant
        ])
        .with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

/// TransferDomain: Sum enum of transfer target domains.
/// Tier: T1Universal. Dominant: Σ Sum.
/// WHY: One-of-N exclusive classification of target domains.
impl GroundsTo for TransferDomain {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum, // Σ -- one-of-N variant
        ])
        .with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

// ============================================================================
// T2-P (2-3 unique primitives)
// ============================================================================

/// TaxonomySummary: Numeric counts of each taxonomy tier.
/// Tier: T2Primitive. Dominant: N Quantity.
/// WHY: Product of five numeric counts (t1, t2p, t2c, t3, total).
impl GroundsTo for TaxonomySummary {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- numeric tier counts
            LexPrimitiva::State,    // ς -- snapshot of taxonomy state
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// PvPrimitiveComposition: Ordered list of primitives forming a type's grounding.
/// Tier: T2Primitive. Dominant: σ Sequence.
/// WHY: Ordered collection of primitives with sequence semantics.
impl GroundsTo for PvPrimitiveComposition {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence, // σ -- ordered list of primitives
            LexPrimitiva::Quantity, // N -- count of primitives
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.80)
    }
}

/// PvTier: Ordered tier classification (T1, T2-P, T2-C, T3).
/// Tier: T2Primitive. Dominant: Σ Sum.
/// WHY: One-of-4 ordered classification with comparison semantics.
impl GroundsTo for PvTier {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Σ -- one-of-4 variant
            LexPrimitiva::Comparison, // κ -- ordered hierarchy
        ])
        .with_dominant(LexPrimitiva::Sum, 0.80)
    }
}

/// DetectionConcept: Sum enum of signal detection concepts.
/// Tier: T2Primitive. Dominant: Σ Sum.
/// WHY: Concept classification with boundary (thresholds) and comparison (metrics).
impl GroundsTo for DetectionConcept {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Σ -- one-of-N variant
            LexPrimitiva::Boundary,   // ∂ -- detection thresholds
            LexPrimitiva::Comparison, // κ -- metric-based comparison
        ])
        .with_dominant(LexPrimitiva::Sum, 0.70)
    }
}

/// AssessmentConcept: Sum enum of causality assessment concepts.
/// Tier: T2Primitive. Dominant: Σ Sum.
/// WHY: Assessment concepts involve causal reasoning and comparison.
impl GroundsTo for AssessmentConcept {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Σ -- one-of-N variant
            LexPrimitiva::Causality,  // → -- causal inference
            LexPrimitiva::Comparison, // κ -- evidence comparison
        ])
        .with_dominant(LexPrimitiva::Sum, 0.70)
    }
}

/// UnderstandingConcept: Sum enum of understanding/deepening concepts.
/// Tier: T2Primitive. Dominant: Σ Sum.
/// WHY: Understanding involves recursive deepening and knowledge mapping.
impl GroundsTo for UnderstandingConcept {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,       // Σ -- one-of-N variant
            LexPrimitiva::Recursion, // ρ -- recursive deepening
            LexPrimitiva::Mapping,   // μ -- knowledge transformation
        ])
        .with_dominant(LexPrimitiva::Sum, 0.70)
    }
}

/// PreventionConcept: Sum enum of prevention/intervention concepts.
/// Tier: T2Primitive. Dominant: Σ Sum.
/// WHY: Prevention enforces safety boundaries through causal actions.
impl GroundsTo for PreventionConcept {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,       // Σ -- one-of-N variant
            LexPrimitiva::Boundary,  // ∂ -- safety boundaries
            LexPrimitiva::Causality, // → -- intervention actions
        ])
        .with_dominant(LexPrimitiva::Sum, 0.70)
    }
}

/// ScopeConcept: Sum enum of scope/population concepts.
/// Tier: T2Primitive. Dominant: Σ Sum.
/// WHY: Scope concepts define population boundaries with location context.
impl GroundsTo for ScopeConcept {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,      // Σ -- one-of-N variant
            LexPrimitiva::Boundary, // ∂ -- population boundaries
            LexPrimitiva::Location, // λ -- geographic/demographic scope
        ])
        .with_dominant(LexPrimitiva::Sum, 0.70)
    }
}

/// PvComposite: Sum enum of composite PV concepts.
/// Tier: T2Primitive. Dominant: Σ Sum.
/// WHY: Each composite is a product of T2-P primitives.
impl GroundsTo for PvComposite {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,     // Σ -- one-of-N variant
            LexPrimitiva::Product, // × -- composite product structure
        ])
        .with_dominant(LexPrimitiva::Sum, 0.75)
    }
}

/// RegulatoryConcept: Sum enum of regulatory framework concepts.
/// Tier: T2Primitive. Dominant: Σ Sum.
/// WHY: Regulatory concepts enforce compliance boundaries.
impl GroundsTo for RegulatoryConcept {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,      // Σ -- one-of-N variant
            LexPrimitiva::Boundary, // ∂ -- regulatory boundaries
        ])
        .with_dominant(LexPrimitiva::Sum, 0.75)
    }
}

/// InfrastructureConcept: Sum enum of PV infrastructure concepts.
/// Tier: T2Primitive. Dominant: Σ Sum.
/// WHY: Infrastructure involves stateful systems with persistence.
impl GroundsTo for InfrastructureConcept {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,         // Σ -- one-of-N variant
            LexPrimitiva::State,       // ς -- system state
            LexPrimitiva::Persistence, // π -- data persistence
        ])
        .with_dominant(LexPrimitiva::Sum, 0.70)
    }
}

/// OperationsConcept: Sum enum of PV operational concepts.
/// Tier: T2Primitive. Dominant: Σ Sum.
/// WHY: Operations are sequential workflows.
impl GroundsTo for OperationsConcept {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,      // Σ -- one-of-N variant
            LexPrimitiva::Sequence, // σ -- sequential workflows
        ])
        .with_dominant(LexPrimitiva::Sum, 0.80)
    }
}

/// AnalyticsConcept: Sum enum of analytics concepts.
/// Tier: T2Primitive. Dominant: Σ Sum.
/// WHY: Analytics involve numeric comparison and measurement.
impl GroundsTo for AnalyticsConcept {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Σ -- one-of-N variant
            LexPrimitiva::Quantity,   // N -- numeric metrics
            LexPrimitiva::Comparison, // κ -- statistical comparison
        ])
        .with_dominant(LexPrimitiva::Sum, 0.70)
    }
}

/// SafetyCommsConcept: Sum enum of safety communication concepts.
/// Tier: T2Primitive. Dominant: Σ Sum.
/// WHY: Safety communications are causal actions triggering downstream effects.
impl GroundsTo for SafetyCommsConcept {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,       // Σ -- one-of-N variant
            LexPrimitiva::Causality, // → -- communication triggers action
        ])
        .with_dominant(LexPrimitiva::Sum, 0.80)
    }
}

/// SpecialPopulationConcept: Sum enum of special population concepts.
/// Tier: T2Primitive. Dominant: Σ Sum.
/// WHY: Special populations define safety boundary subgroups.
impl GroundsTo for SpecialPopulationConcept {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,      // Σ -- one-of-N variant
            LexPrimitiva::Boundary, // ∂ -- population safety boundaries
        ])
        .with_dominant(LexPrimitiva::Sum, 0.80)
    }
}

/// ChomskyLevel: Ordered Chomsky hierarchy classification.
/// Tier: T2Primitive. Dominant: Σ Sum.
/// WHY: One-of-4 ordered classification (Type-0 through Type-3).
impl GroundsTo for ChomskyLevel {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Σ -- one-of-4 variant
            LexPrimitiva::Comparison, // κ -- hierarchy ordering
        ])
        .with_dominant(LexPrimitiva::Sum, 0.80)
    }
}

/// PvSubsystem: Sum enum of PV subsystem classifications.
/// Tier: T2Primitive. Dominant: Σ Sum.
/// WHY: Classification of PV subsystems with sequential pipeline semantics.
impl GroundsTo for PvSubsystem {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,      // Σ -- one-of-4 variant
            LexPrimitiva::Sequence, // σ -- pipeline ordering
        ])
        .with_dominant(LexPrimitiva::Sum, 0.80)
    }
}

/// TransferConfidence: Numeric confidence scores for cross-domain transfer.
/// Tier: T2Primitive. Dominant: N Quantity.
/// WHY: Product of three numeric confidence components.
impl GroundsTo for TransferConfidence {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N -- numeric scores
            LexPrimitiva::Comparison, // κ -- confidence comparison
            LexPrimitiva::Product,    // × -- product of components
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.75)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::grounding::GroundsTo;
    use nexcore_lex_primitiva::tier::Tier as LexTier;

    #[test]
    fn lex_symbol_is_t1() {
        assert_eq!(<LexSymbol as GroundsTo>::tier(), LexTier::T1Universal);
        assert_eq!(LexSymbol::dominant_primitive(), Some(LexPrimitiva::Sum));
    }

    #[test]
    fn pv_primitive_is_t1() {
        // PvPrimitive has its own tier(&self) method, so use fully-qualified syntax
        assert_eq!(<PvPrimitive as GroundsTo>::tier(), LexTier::T1Universal);
        assert_eq!(
            <PvPrimitive as GroundsTo>::dominant_primitive(),
            Some(LexPrimitiva::Sum)
        );
    }

    #[test]
    fn transfer_domain_is_t1() {
        assert_eq!(<TransferDomain as GroundsTo>::tier(), LexTier::T1Universal);
    }

    #[test]
    fn taxonomy_summary_is_t2p() {
        assert_eq!(<TaxonomySummary as GroundsTo>::tier(), LexTier::T2Primitive);
        assert_eq!(
            TaxonomySummary::dominant_primitive(),
            Some(LexPrimitiva::Quantity)
        );
    }

    #[test]
    fn pv_tier_is_t2p() {
        assert_eq!(<PvTier as GroundsTo>::tier(), LexTier::T2Primitive);
        assert_eq!(
            <PvTier as GroundsTo>::dominant_primitive(),
            Some(LexPrimitiva::Sum)
        );
    }

    #[test]
    fn detection_concept_is_t2p() {
        assert_eq!(
            <DetectionConcept as GroundsTo>::tier(),
            LexTier::T2Primitive
        );
        assert_eq!(
            DetectionConcept::dominant_primitive(),
            Some(LexPrimitiva::Sum)
        );
    }

    #[test]
    fn assessment_concept_is_t2p() {
        assert_eq!(
            <AssessmentConcept as GroundsTo>::tier(),
            LexTier::T2Primitive
        );
    }

    #[test]
    fn understanding_concept_is_t2p() {
        assert_eq!(
            <UnderstandingConcept as GroundsTo>::tier(),
            LexTier::T2Primitive
        );
    }

    #[test]
    fn prevention_concept_is_t2p() {
        assert_eq!(
            <PreventionConcept as GroundsTo>::tier(),
            LexTier::T2Primitive
        );
    }

    #[test]
    fn scope_concept_is_t2p() {
        assert_eq!(<ScopeConcept as GroundsTo>::tier(), LexTier::T2Primitive);
    }

    #[test]
    fn pv_composite_is_t2p() {
        assert_eq!(<PvComposite as GroundsTo>::tier(), LexTier::T2Primitive);
    }

    #[test]
    fn chomsky_level_is_t2p() {
        assert_eq!(<ChomskyLevel as GroundsTo>::tier(), LexTier::T2Primitive);
    }

    #[test]
    fn pv_subsystem_is_t2p() {
        assert_eq!(<PvSubsystem as GroundsTo>::tier(), LexTier::T2Primitive);
    }

    #[test]
    fn transfer_confidence_is_t2p() {
        assert_eq!(
            <TransferConfidence as GroundsTo>::tier(),
            LexTier::T2Primitive
        );
        assert_eq!(
            TransferConfidence::dominant_primitive(),
            Some(LexPrimitiva::Quantity)
        );
    }

    #[test]
    fn regulatory_concept_is_t2p() {
        assert_eq!(
            <RegulatoryConcept as GroundsTo>::tier(),
            LexTier::T2Primitive
        );
    }

    #[test]
    fn infrastructure_concept_is_t2p() {
        assert_eq!(
            <InfrastructureConcept as GroundsTo>::tier(),
            LexTier::T2Primitive
        );
    }

    #[test]
    fn operations_concept_is_t2p() {
        assert_eq!(
            <OperationsConcept as GroundsTo>::tier(),
            LexTier::T2Primitive
        );
    }

    #[test]
    fn analytics_concept_is_t2p() {
        assert_eq!(
            <AnalyticsConcept as GroundsTo>::tier(),
            LexTier::T2Primitive
        );
    }

    #[test]
    fn safety_comms_concept_is_t2p() {
        assert_eq!(
            <SafetyCommsConcept as GroundsTo>::tier(),
            LexTier::T2Primitive
        );
    }

    #[test]
    fn special_population_concept_is_t2p() {
        assert_eq!(
            <SpecialPopulationConcept as GroundsTo>::tier(),
            LexTier::T2Primitive
        );
    }

    #[test]
    fn primitive_composition_is_t2p() {
        // PvPrimitiveComposition has its own tier(&self) method, so use fully-qualified syntax
        assert_eq!(
            <PvPrimitiveComposition as GroundsTo>::tier(),
            LexTier::T2Primitive
        );
        assert_eq!(
            <PvPrimitiveComposition as GroundsTo>::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
    }
}
