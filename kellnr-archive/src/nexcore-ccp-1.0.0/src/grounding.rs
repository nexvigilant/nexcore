//! # GroundsTo implementations for nexcore-ccp types
//!
//! Connects Claude Care Process pharmacokinetic types to the Lex Primitiva type system.
//!
//! ## Key Primitive Distribution
//!
//! - Phase transitions: sigma (Sequence) -- ordered FSM progression
//! - Episode state: varsigma (State) -- mutable patient state
//! - Therapeutic window: partial (Boundary) -- safe dose boundaries
//! - PK decay curves: proportional (Irreversibility) -- exponential decay
//! - Quality scoring: kappa (Comparison) -- threshold comparison

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::episode::{Episode, Intervention};
use crate::interactions::InteractionEffect;
use crate::quality::{QualityComponents, QualityRating, QualityScore};
use crate::state_machine::PhaseTransition;
use crate::types::{
    BioAvailability, Dose, DosingStrategy, HalfLife, InteractionType, Phase, PlasmaLevel,
    TherapeuticWindow,
};

// ---------------------------------------------------------------------------
// Newtypes -- N (Quantity) dominant
// ---------------------------------------------------------------------------

/// PlasmaLevel: T1 (N), pure quantity
///
/// Newtype wrapping f64 representing plasma concentration.
impl GroundsTo for PlasmaLevel {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Quantity, 1.0)
    }
}

/// BioAvailability: T2-P (N + partial), dominant N
///
/// Fraction [0.0, 1.0] of administered dose that reaches systemic circulation.
/// Quantity-dominant: it IS a numeric fraction.
/// Boundary is secondary (clamped to [0, 1] range).
impl GroundsTo for BioAvailability {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- numeric fraction
            LexPrimitiva::Boundary, // partial -- [0, 1] range boundary
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.90)
    }
}

/// HalfLife: T2-P (N + proportional), dominant N
///
/// Time for plasma level to halve. Drives exponential decay.
/// Quantity-dominant: it is a numeric duration.
/// Irreversibility is secondary (decay is a one-way process).
impl GroundsTo for HalfLife {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,        // N -- numeric duration
            LexPrimitiva::Irreversibility, // proportional -- drives one-way decay
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// Dose: T2-P (N + partial), dominant N
///
/// Amount of intervention administered, bounded [0.0, 1.0].
impl GroundsTo for Dose {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- numeric amount
            LexPrimitiva::Boundary, // partial -- dose range boundary
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.90)
    }
}

// ---------------------------------------------------------------------------
// Classification enums -- Sigma (Sum) dominant
// ---------------------------------------------------------------------------

/// Phase: T2-P (Sigma + sigma), dominant Sigma
///
/// Five-phase FSM: Collect -> Assess -> Plan -> Implement -> FollowUp.
/// Sum-dominant: the type IS a categorical alternation of phases.
/// Sequence is secondary (phases follow an ordered progression).
impl GroundsTo for Phase {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,      // Sigma -- phase variant alternation
            LexPrimitiva::Sequence, // sigma -- ordered phase progression
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

/// DosingStrategy: T2-P (Sigma + kappa), dominant Sigma
///
/// Loading, Maintenance, Therapeutic, Tapering classification.
/// Sum-dominant: categorical alternation of strategies.
/// Comparison is secondary (strategy selection via threshold comparison).
impl GroundsTo for DosingStrategy {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Sigma -- strategy variant
            LexPrimitiva::Comparison, // kappa -- selection via comparison
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

/// InteractionType: T2-P (Sigma + N), dominant Sigma
///
/// Synergistic, Antagonistic, Additive, Potentiating interaction types.
impl GroundsTo for InteractionType {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,      // Sigma -- interaction type variant
            LexPrimitiva::Quantity, // N -- affects numeric outcome
        ])
        .with_dominant(LexPrimitiva::Sum, 0.90)
    }
}

/// QualityRating: T2-P (Sigma + kappa), dominant Sigma
///
/// Subtherapeutic, Therapeutic, Supratherapeutic, Toxic classification.
impl GroundsTo for QualityRating {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Sigma -- rating variant
            LexPrimitiva::Comparison, // kappa -- threshold-based classification
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Boundary types -- partial (Boundary) dominant
// ---------------------------------------------------------------------------

/// TherapeuticWindow: T2-P (partial + N), dominant partial
///
/// Defines the safe dose range [min_effective, max_safe].
/// Boundary-dominant: it IS a boundary defining safe vs unsafe regions.
impl GroundsTo for TherapeuticWindow {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // partial -- safe range boundary
            LexPrimitiva::Quantity, // N -- numeric min/max values
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.90)
    }
}

// ---------------------------------------------------------------------------
// Composite types -- multi-primitive
// ---------------------------------------------------------------------------

/// Intervention: T2-C (N + mu + proportional + partial), dominant N
///
/// A single administered intervention with dose, bioavailability, half-life.
/// Quantity-dominant: dose, bioavailability, half_life are all numeric.
/// Mapping is secondary (transforms dose into plasma level change).
/// Irreversibility is tertiary (decay after administration).
/// Boundary is quaternary (dose limits).
impl GroundsTo for Intervention {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,        // N -- dose, bioavailability, half_life
            LexPrimitiva::Mapping,         // mu -- dose -> plasma level change
            LexPrimitiva::Irreversibility, // proportional -- PK decay
            LexPrimitiva::Boundary,        // partial -- dose boundaries
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.80)
    }
}

/// PhaseTransition: T2-C (sigma + Sigma + causality + N), dominant sigma
///
/// A recorded state transition with from/to phases, reason, timestamp.
/// Sequence-dominant: transitions form an ordered progression.
/// Sum is secondary (from/to are Phase enum variants).
/// Causality is tertiary (transition has a reason/cause).
/// Quantity is quaternary (timestamp is numeric).
impl GroundsTo for PhaseTransition {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,  // sigma -- ordered progression
            LexPrimitiva::Sum,       // Sigma -- phase variant endpoints
            LexPrimitiva::Causality, // causality -- transition reason
            LexPrimitiva::Quantity,  // N -- timestamp
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.80)
    }
}

/// InteractionEffect: T2-P (N + Sigma + mu), dominant N
///
/// The computed effect of combining two interventions.
/// Quantity-dominant: combined_level is the numeric output.
impl GroundsTo for InteractionEffect {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- combined numeric level
            LexPrimitiva::Sum,      // Sigma -- interaction type
            LexPrimitiva::Mapping,  // mu -- two levels -> combined level
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// QualityComponents: T2-C (N + kappa + partial + proportional), dominant N
///
/// Four numeric quality sub-scores: bioavailability, stability, safety, persistence.
impl GroundsTo for QualityComponents {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,    // N -- four numeric sub-scores
            LexPrimitiva::Comparison,  // kappa -- comparison against thresholds
            LexPrimitiva::Boundary,    // partial -- safety margin boundaries
            LexPrimitiva::Persistence, // pi -- persistence sub-score
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.80)
    }
}

/// QualityScore: T2-C (N + kappa + Sigma + mu), dominant N
///
/// Composite quality score [0, 10] with components and rating.
impl GroundsTo for QualityScore {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N -- composite numeric score
            LexPrimitiva::Comparison, // kappa -- threshold comparison
            LexPrimitiva::Sum,        // Sigma -- rating classification
            LexPrimitiva::Mapping,    // mu -- components -> composite
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.80)
    }
}

// ---------------------------------------------------------------------------
// Domain state -- T3 composite
// ---------------------------------------------------------------------------

/// Episode: T3 (varsigma + sigma + N + Sigma + partial + proportional + rho), dominant varsigma
///
/// Complete care episode with phase, plasma level, interventions, transitions.
/// State-dominant: this is the full mutable state of a care episode.
/// Sequence is secondary (phase progression and transition history).
/// Recursion is included because episodes can loop back to Collect phase.
impl GroundsTo for Episode {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,           // varsigma -- episode state
            LexPrimitiva::Sequence,        // sigma -- phase progression
            LexPrimitiva::Quantity,        // N -- plasma level, timestamps
            LexPrimitiva::Sum,             // Sigma -- current phase variant
            LexPrimitiva::Boundary,        // partial -- therapeutic window
            LexPrimitiva::Irreversibility, // proportional -- PK decay
            LexPrimitiva::Recursion,       // rho -- re-collection loops
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
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
    fn plasma_level_is_t1() {
        assert_eq!(PlasmaLevel::tier(), Tier::T1Universal);
        assert!(PlasmaLevel::is_pure_primitive());
    }

    #[test]
    fn phase_is_t2p() {
        assert_eq!(Phase::tier(), Tier::T2Primitive);
        assert_eq!(Phase::dominant_primitive(), Some(LexPrimitiva::Sum));
    }

    #[test]
    fn therapeutic_window_is_t2p() {
        assert_eq!(TherapeuticWindow::tier(), Tier::T2Primitive);
        assert_eq!(
            TherapeuticWindow::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn intervention_is_t2c() {
        assert_eq!(Intervention::tier(), Tier::T2Composite);
        assert_eq!(
            Intervention::dominant_primitive(),
            Some(LexPrimitiva::Quantity)
        );
    }

    #[test]
    fn episode_is_t3() {
        assert_eq!(Episode::tier(), Tier::T3DomainSpecific);
        assert_eq!(Episode::dominant_primitive(), Some(LexPrimitiva::State));
    }

    #[test]
    fn quality_score_is_t2c() {
        assert_eq!(QualityScore::tier(), Tier::T2Composite);
    }

    #[test]
    fn all_confidences_valid() {
        let compositions = [
            PlasmaLevel::primitive_composition(),
            BioAvailability::primitive_composition(),
            HalfLife::primitive_composition(),
            Dose::primitive_composition(),
            Phase::primitive_composition(),
            DosingStrategy::primitive_composition(),
            InteractionType::primitive_composition(),
            QualityRating::primitive_composition(),
            TherapeuticWindow::primitive_composition(),
            Intervention::primitive_composition(),
            PhaseTransition::primitive_composition(),
            InteractionEffect::primitive_composition(),
            QualityComponents::primitive_composition(),
            QualityScore::primitive_composition(),
            Episode::primitive_composition(),
        ];
        for comp in &compositions {
            assert!(comp.confidence >= 0.80);
            assert!(comp.confidence <= 1.0);
        }
    }
}
