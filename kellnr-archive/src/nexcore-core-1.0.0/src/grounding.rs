//! # GroundsTo implementations for nexcore-core types
//!
//! Primitive grounding for the PV execution kernel: SignalAnalysisResult,
//! SignalMetrics, and PersistenceManager.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::persistence::PersistenceManager;
use crate::SignalAnalysisResult;
use crate::SignalMetrics;

// ============================================================================
// SignalAnalysisResult: T3 (ς + × + N + → + ν + ∂ + σ), dominant ς
// Full domain type — signal analysis record with metrics, risk level,
// timestamps, and recommended actions. The core execution molecule.
// ============================================================================

impl GroundsTo for SignalAnalysisResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Product,
            LexPrimitiva::Quantity,
            LexPrimitiva::Causality,
            LexPrimitiva::Frequency,
            LexPrimitiva::Boundary,
            LexPrimitiva::Sequence,
        ])
        .with_dominant(LexPrimitiva::State, 0.65)
    }
}

// ============================================================================
// SignalMetrics: T2-C (N + × + κ + ∂), dominant N
// Numeric measurements — contingency table counts (a,b,c,d) and derived
// statistics (PRR, ROR, EBGM). Product of quantities with comparison
// and boundary semantics for threshold evaluation.
// ============================================================================

impl GroundsTo for SignalMetrics {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Product,
            LexPrimitiva::Comparison,
            LexPrimitiva::Boundary,
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

// ============================================================================
// PersistenceManager: T2-P (π + ς + →), dominant π
// Database connection wrapper — persistence with state and causal
// read/write operations.
// ============================================================================

impl GroundsTo for PersistenceManager {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Persistence,
            LexPrimitiva::State,
            LexPrimitiva::Causality,
        ])
        .with_dominant(LexPrimitiva::Persistence, 0.85)
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
    fn signal_analysis_molecule_is_state_dominant_t3() {
        let comp = SignalAnalysisResult::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert_eq!(comp.unique().len(), 7);
        assert_eq!(SignalAnalysisResult::tier(), Tier::T3DomainSpecific);
    }

    #[test]
    fn signal_metrics_is_quantity_dominant() {
        let comp = SignalMetrics::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
        assert_eq!(comp.unique().len(), 4);
        assert_eq!(SignalMetrics::tier(), Tier::T2Composite);
    }

    #[test]
    fn persistence_manager_is_persistence_dominant() {
        let comp = PersistenceManager::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Persistence));
        assert_eq!(comp.unique().len(), 3);
        assert_eq!(PersistenceManager::tier(), Tier::T2Primitive);
    }

    #[test]
    fn all_types_have_dominant() {
        assert!(SignalAnalysisResult::dominant_primitive().is_some());
        assert!(SignalMetrics::dominant_primitive().is_some());
        assert!(PersistenceManager::dominant_primitive().is_some());
    }

    #[test]
    fn no_core_types_are_pure_primitive() {
        // All nexcore-core types compose multiple primitives
        assert!(!SignalAnalysisResult::is_pure_primitive());
        assert!(!SignalMetrics::is_pure_primitive());
        assert!(!PersistenceManager::is_pure_primitive());
    }
}
