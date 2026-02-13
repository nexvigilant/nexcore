//! # GroundsTo implementations for nexcore-constants types
//!
//! Primitive grounding for the bedrock type library: Confidence, Measured, Tier,
//! Correction, ConfidenceInterval, BathroomLock, Occupancy, LockError, OccupiedGuard.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};
use nexcore_lex_primitiva::state_mode::StateMode;

use crate::confidence::Confidence;
use crate::correction::Correction;
use crate::interval::ConfidenceInterval;
use crate::measured::Measured;
use crate::tier::Tier;

use crate::bathroom_lock::{BathroomLock, LockError, Occupancy, OccupiedGuard};

// ============================================================================
// Confidence: T2-P (N + κ), dominant N
// A scalar [0.0, 1.0] — primarily a quantity with comparison semantics.
// ============================================================================

impl GroundsTo for Confidence {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity, LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Quantity, 0.90)
    }
}

// ============================================================================
// Measured<T>: T2-C (× + N + κ + ς), dominant ×
// Product of value and confidence — a composite measurement container.
// ============================================================================

impl<T> GroundsTo for Measured<T> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Product,
            LexPrimitiva::Quantity,
            LexPrimitiva::Comparison,
            LexPrimitiva::State,
        ])
        .with_dominant(LexPrimitiva::Product, 0.80)
        .with_state_mode(StateMode::Accumulated)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

// ============================================================================
// Tier: T1 (Σ), dominant Σ
// Pure sum type — four-variant enum classifying tiers.
// ============================================================================

impl GroundsTo for Tier {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum]).with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

// ============================================================================
// Correction<T>: T2-C (ς + → + π + ×), dominant ς
// A state correction record with causality (before/after), persistence
// (timestamp), and product structure (original + corrected + reason).
// ============================================================================

impl<T> GroundsTo for Correction<T> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Causality,
            LexPrimitiva::Persistence,
            LexPrimitiva::Product,
        ])
        .with_dominant(LexPrimitiva::State, 0.75)
        .with_state_mode(StateMode::Accumulated)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

// ============================================================================
// ConfidenceInterval: T2-P (N + κ + ∂), dominant N
// Numeric bounds with comparison semantics and boundary constraints.
// ============================================================================

impl GroundsTo for ConfidenceInterval {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Comparison,
            LexPrimitiva::Boundary,
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.80)
    }
}

// ============================================================================
// BathroomLock: T2-P (ς + ∂ + ∃), dominant ς
// Binary state machine with boundary exclusion and existence-based lock files.
// ============================================================================

impl GroundsTo for BathroomLock {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Boundary,
            LexPrimitiva::Existence,
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

// ============================================================================
// Occupancy: T2-P (ς + Σ), dominant ς
// Two-variant enum (Vacant/Occupied) — observable lock state.
// ============================================================================

impl GroundsTo for Occupancy {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::State, LexPrimitiva::Sum])
            .with_dominant(LexPrimitiva::State, 0.90)
            .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

// ============================================================================
// LockError: T2-P (∂ + Σ), dominant ∂
// Error sum type — boundary violations during lock operations.
// ============================================================================

impl GroundsTo for LockError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Sum])
            .with_dominant(LexPrimitiva::Boundary, 0.90)
    }
}

// ============================================================================
// OccupiedGuard: T2-P (ς + ∂ + ∝), dominant ς
// RAII guard — state with boundary enforcement and irreversible drop.
// ============================================================================

impl GroundsTo for OccupiedGuard<'_> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Boundary,
            LexPrimitiva::Irreversibility,
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier as LexTier;

    #[test]
    fn confidence_is_quantity_dominant() {
        let comp = Confidence::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
        assert_eq!(comp.unique().len(), 2);
        assert_eq!(Confidence::tier(), LexTier::T2Primitive);
    }

    #[test]
    fn measured_is_product_dominant() {
        let comp = <Measured<i32>>::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Product));
        assert_eq!(comp.unique().len(), 4);
        assert_eq!(<Measured<i32>>::tier(), LexTier::T2Composite);
    }

    #[test]
    fn tier_is_pure_sum() {
        let comp = Tier::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
        assert!(comp.is_pure());
        assert_eq!(Tier::tier(), LexTier::T1Universal);
    }

    #[test]
    fn correction_is_state_dominant() {
        let comp = <Correction<i32>>::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert_eq!(comp.unique().len(), 4);
        assert_eq!(<Correction<i32>>::tier(), LexTier::T2Composite);
    }

    #[test]
    fn confidence_interval_is_quantity_dominant() {
        let comp = ConfidenceInterval::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
        assert_eq!(comp.unique().len(), 3);
        assert_eq!(ConfidenceInterval::tier(), LexTier::T2Primitive);
    }

    #[test]
    fn bathroom_lock_is_state_dominant() {
        let comp = BathroomLock::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert_eq!(comp.unique().len(), 3);
        assert_eq!(BathroomLock::tier(), LexTier::T2Primitive);
    }

    #[test]
    fn occupancy_is_state_dominant() {
        let comp = Occupancy::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert_eq!(comp.unique().len(), 2);
        assert_eq!(Occupancy::tier(), LexTier::T2Primitive);
    }

    #[test]
    fn lock_error_is_boundary_dominant() {
        let comp = LockError::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
        assert_eq!(comp.unique().len(), 2);
        assert_eq!(LockError::tier(), LexTier::T2Primitive);
    }

    #[test]
    fn occupied_guard_is_state_dominant() {
        let comp = <OccupiedGuard<'_>>::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert_eq!(comp.unique().len(), 3);
        assert_eq!(<OccupiedGuard<'_>>::tier(), LexTier::T2Primitive);
    }

    #[test]
    fn all_types_have_dominant() {
        assert!(Confidence::dominant_primitive().is_some());
        assert!(<Measured<u8>>::dominant_primitive().is_some());
        assert!(Tier::dominant_primitive().is_some());
        assert!(<Correction<u8>>::dominant_primitive().is_some());
        assert!(ConfidenceInterval::dominant_primitive().is_some());
        assert!(BathroomLock::dominant_primitive().is_some());
        assert!(Occupancy::dominant_primitive().is_some());
        assert!(LockError::dominant_primitive().is_some());
        assert!(<OccupiedGuard<'_>>::dominant_primitive().is_some());
    }

    #[test]
    fn tier_enum_is_pure_primitive() {
        assert!(Tier::is_pure_primitive());
    }

    #[test]
    fn confidence_is_not_pure() {
        assert!(!Confidence::is_pure_primitive());
    }
}
