//! # GroundsTo implementations for nexcore-hormones types
//!
//! Connects the endocrine system model to the Lex Primitiva type system.
//!
//! ## Dominant Primitive Distribution
//!
//! - `HormoneLevel` -- Quantity (N) dominates: a bounded numeric measurement (0.0..=1.0)
//!   with clamping. Boundary (partial) is secondary for the clamped range constraint.
//! - `HormoneType` -- Sum (Sigma) dominates: an exhaustive 6-variant enumeration
//!   classifying hormone categories. Comparison (kappa) secondary for variant discrimination.
//! - `EndocrineState` -- State (varsigma) dominates: encapsulated mutable context
//!   of all hormone levels with persistence, temporal tracking, and session counting.
//! - `Stimulus` -- Causality (arrow) dominates: each variant triggers a hormone change,
//!   encoding cause-to-effect relationships. Sum (Sigma) for variant dispatch, Mapping (mu)
//!   for stimulus-to-hormone-delta transformation.
//! - `BehavioralModifiers` -- Mapping (mu) dominates: derived from EndocrineState by
//!   transforming hormone levels into behavioral parameters. Product (times) for the
//!   composite struct, State (varsigma) for encapsulated context.
//! - `EndocrineError` -- Boundary (partial) dominates: error types are boundary
//!   violations. Sum (Sigma) secondary for the 3-variant dispatch.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};
use nexcore_lex_primitiva::state_mode::StateMode;

use crate::{
    BehavioralModifiers, EndocrineError, EndocrineState, HormoneLevel, HormoneType, Stimulus,
};

// ---------------------------------------------------------------------------
// HormoneLevel -- Quantity dominant (T2-P)
// ---------------------------------------------------------------------------

/// HormoneLevel: T2-P (N + partial), dominant N
///
/// A bounded f64 in [0.0, 1.0] representing a hormone concentration.
/// Quantity-dominant: the core purpose is numeric measurement of a level.
/// Boundary is secondary: the clamping constraint enforces valid range.
impl GroundsTo for HormoneLevel {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- numeric hormone level (f64)
            LexPrimitiva::Boundary, // partial -- clamped to [0.0, 1.0]
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

// ---------------------------------------------------------------------------
// HormoneType -- Sum dominant (T2-P)
// ---------------------------------------------------------------------------

/// HormoneType: T2-P (Sigma + kappa), dominant Sigma
///
/// An exhaustive enumeration of 6 hormone categories (Cortisol, Dopamine,
/// Serotonin, Adrenaline, Oxytocin, Melatonin). Sum-dominant: the type
/// IS a sum type partitioning the hormone domain into discrete variants.
/// Comparison is secondary: variant matching discriminates between categories.
impl GroundsTo for HormoneType {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Sigma -- 6-variant sum type
            LexPrimitiva::Comparison, // kappa -- variant discrimination and decay_rate ordering
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

// ---------------------------------------------------------------------------
// EndocrineState -- State dominant (T2-C)
// ---------------------------------------------------------------------------

/// EndocrineState: T2-C (varsigma + pi + N + times + sigma), dominant varsigma
///
/// The complete endocrine dashboard: 6 hormone levels, a timestamp, and a
/// session counter, with load/save persistence. State-dominant: this is
/// encapsulated mutable context that tracks and mutates across sessions.
/// Persistence for load/save to disk. Quantity for numeric levels and counter.
/// Product for composite struct with multiple typed fields. Sequence for
/// session ordering and temporal progression.
impl GroundsTo for EndocrineState {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,       // varsigma -- encapsulated mutable hormone context
            LexPrimitiva::Persistence, // pi -- load/save to JSON on disk
            LexPrimitiva::Quantity,    // N -- numeric levels, session_count
            LexPrimitiva::Product,     // times -- composite struct of 6 levels + metadata
            LexPrimitiva::Sequence,    // sigma -- session ordering, apply_decay progression
        ])
        .with_dominant(LexPrimitiva::State, 0.60)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

// ---------------------------------------------------------------------------
// Stimulus -- Causality dominant (T2-P)
// ---------------------------------------------------------------------------

/// Stimulus: T2-P (arrow + Sigma + mu), dominant arrow
///
/// A 17-variant enum where each variant triggers specific hormone changes.
/// Causality-dominant: the entire purpose is encoding cause (stimulus event)
/// to effect (hormone delta). Sum for the variant dispatch. Mapping for
/// the transformation from stimulus parameters to hormone-level adjustments.
impl GroundsTo for Stimulus {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality, // arrow -- stimulus causes hormone change
            LexPrimitiva::Sum,       // Sigma -- 17-variant enum dispatch
            LexPrimitiva::Mapping,   // mu -- stimulus parameters -> hormone deltas
        ])
        .with_dominant(LexPrimitiva::Causality, 0.70)
    }
}

// ---------------------------------------------------------------------------
// BehavioralModifiers -- Mapping dominant (T2-P)
// ---------------------------------------------------------------------------

/// BehavioralModifiers: T2-P (mu + times + varsigma), dominant mu
///
/// Derived output computed from EndocrineState via `From<&EndocrineState>`.
/// Mapping-dominant: the type exists to transform hormone levels into
/// behavioral parameters (risk_tolerance, validation_depth, etc.).
/// Product for the composite struct. State for encapsulated context.
impl GroundsTo for BehavioralModifiers {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping, // mu -- transforms EndocrineState -> behavioral params
            LexPrimitiva::Product, // times -- composite struct of 7 fields
            LexPrimitiva::State,   // varsigma -- encapsulated behavioral context
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.75)
        .with_state_mode(StateMode::Accumulated)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

// ---------------------------------------------------------------------------
// EndocrineError -- Boundary dominant (T2-P)
// ---------------------------------------------------------------------------

/// EndocrineError: T2-P (partial + Sigma), dominant partial
///
/// Error type with 3 variants (ReadError, ParseError, NoHomeDir).
/// Boundary-dominant: errors represent boundary violations -- failed I/O,
/// invalid JSON, missing environment. Sum for the 3-variant dispatch.
impl GroundsTo for EndocrineError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // partial -- error = boundary violation
            LexPrimitiva::Sum,      // Sigma -- 3-variant error enum
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    // --- HormoneLevel ---

    #[test]
    fn hormone_level_grounds_to_quantity() {
        let comp = HormoneLevel::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
    }

    #[test]
    fn hormone_level_has_boundary() {
        let comp = HormoneLevel::primitive_composition();
        assert!(comp.unique().contains(&LexPrimitiva::Boundary));
    }

    #[test]
    fn hormone_level_is_t2_primitive() {
        assert_eq!(HormoneLevel::tier(), Tier::T2Primitive);
    }

    #[test]
    fn hormone_level_is_not_pure() {
        assert!(!HormoneLevel::is_pure_primitive());
    }

    // --- HormoneType ---

    #[test]
    fn hormone_type_grounds_to_sum() {
        let comp = HormoneType::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
    }

    #[test]
    fn hormone_type_has_comparison() {
        let comp = HormoneType::primitive_composition();
        assert!(comp.unique().contains(&LexPrimitiva::Comparison));
    }

    #[test]
    fn hormone_type_is_t2_primitive() {
        assert_eq!(HormoneType::tier(), Tier::T2Primitive);
    }

    // --- EndocrineState ---

    #[test]
    fn endocrine_state_grounds_to_state() {
        let comp = EndocrineState::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
    }

    #[test]
    fn endocrine_state_has_persistence() {
        let comp = EndocrineState::primitive_composition();
        assert!(comp.unique().contains(&LexPrimitiva::Persistence));
    }

    #[test]
    fn endocrine_state_has_five_primitives() {
        let comp = EndocrineState::primitive_composition();
        assert_eq!(comp.unique().len(), 5);
    }

    #[test]
    fn endocrine_state_is_t2_composite() {
        assert_eq!(EndocrineState::tier(), Tier::T2Composite);
    }

    // --- Stimulus ---

    #[test]
    fn stimulus_grounds_to_causality() {
        let comp = Stimulus::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Causality));
    }

    #[test]
    fn stimulus_has_sum_and_mapping() {
        let comp = Stimulus::primitive_composition();
        let unique = comp.unique();
        assert!(unique.contains(&LexPrimitiva::Sum));
        assert!(unique.contains(&LexPrimitiva::Mapping));
    }

    #[test]
    fn stimulus_is_t2_primitive() {
        assert_eq!(Stimulus::tier(), Tier::T2Primitive);
    }

    // --- BehavioralModifiers ---

    #[test]
    fn behavioral_modifiers_grounds_to_mapping() {
        let comp = BehavioralModifiers::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
    }

    #[test]
    fn behavioral_modifiers_has_product_and_state() {
        let comp = BehavioralModifiers::primitive_composition();
        let unique = comp.unique();
        assert!(unique.contains(&LexPrimitiva::Product));
        assert!(unique.contains(&LexPrimitiva::State));
    }

    #[test]
    fn behavioral_modifiers_is_t2_primitive() {
        assert_eq!(BehavioralModifiers::tier(), Tier::T2Primitive);
    }

    // --- EndocrineError ---

    #[test]
    fn endocrine_error_grounds_to_boundary() {
        let comp = EndocrineError::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
    }

    #[test]
    fn endocrine_error_has_sum() {
        let comp = EndocrineError::primitive_composition();
        assert!(comp.unique().contains(&LexPrimitiva::Sum));
    }

    #[test]
    fn endocrine_error_is_t2_primitive() {
        assert_eq!(EndocrineError::tier(), Tier::T2Primitive);
    }

    // --- Cross-cutting ---

    #[test]
    fn all_types_have_dominant_primitive() {
        assert!(HormoneLevel::dominant_primitive().is_some());
        assert!(HormoneType::dominant_primitive().is_some());
        assert!(EndocrineState::dominant_primitive().is_some());
        assert!(Stimulus::dominant_primitive().is_some());
        assert!(BehavioralModifiers::dominant_primitive().is_some());
        assert!(EndocrineError::dominant_primitive().is_some());
    }

    #[test]
    fn all_dominants_are_unique_across_types() {
        // Each type should have a distinct dominant primitive to avoid
        // semantic collisions in the endocrine domain
        let dominants = vec![
            HormoneLevel::dominant_primitive(),
            HormoneType::dominant_primitive(),
            EndocrineState::dominant_primitive(),
            Stimulus::dominant_primitive(),
            BehavioralModifiers::dominant_primitive(),
            EndocrineError::dominant_primitive(),
        ];

        // Verify: Quantity, Sum, State, Causality, Mapping, Boundary -- all distinct
        let mut seen = std::collections::HashSet::new();
        for d in &dominants {
            assert!(seen.insert(d), "Duplicate dominant primitive found: {d:?}");
        }
    }

    #[test]
    fn tier_distribution_matches_expectations() {
        // 1 T2-C, 5 T2-P -- no T1 (domain types) or T3 (not enough primitives)
        let tiers = vec![
            HormoneLevel::tier(),
            HormoneType::tier(),
            EndocrineState::tier(),
            Stimulus::tier(),
            BehavioralModifiers::tier(),
            EndocrineError::tier(),
        ];

        let t2p_count = tiers.iter().filter(|t| **t == Tier::T2Primitive).count();
        let t2c_count = tiers.iter().filter(|t| **t == Tier::T2Composite).count();

        assert_eq!(t2p_count, 5, "Expected 5 T2-P types");
        assert_eq!(t2c_count, 1, "Expected 1 T2-C type (EndocrineState)");
    }
}
