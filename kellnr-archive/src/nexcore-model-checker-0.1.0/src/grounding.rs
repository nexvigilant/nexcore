// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Lex Primitiva Grounding for Model Checker Types
//!
//! Maps each model checker type to its T1 primitive composition.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};
use nexcore_lex_primitiva::state_mode::StateMode;

use crate::ctl::CtlChecker;
use crate::kripke::{KripkeBuilder, KripkeStructure};
use crate::ltl::LtlBoundedChecker;
use crate::result::{CheckResult, Counterexample, Witness};

// ── KripkeStructure: T3 (ρ + ∂ + → + κ + ∃ + ς) ──────────

impl GroundsTo for KripkeStructure {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Recursion,  // ρ — fixpoint iteration over state space
            LexPrimitiva::Boundary,   // ∂ — initial/terminal delineation
            LexPrimitiva::Causality,  // → — transition relation
            LexPrimitiva::Comparison, // κ — proposition satisfaction
            LexPrimitiva::Existence,  // ∃ — state existence validation
            LexPrimitiva::State,      // ς — state configuration space
        ])
        .with_dominant(LexPrimitiva::Recursion, 0.90)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

// ── KripkeBuilder: T2-C (ς + ∂ + μ) ───────────────────────

impl GroundsTo for KripkeBuilder {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // ς — building state space
            LexPrimitiva::Boundary, // ∂ — marking initial/terminal
            LexPrimitiva::Mapping,  // μ — label assignment
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

// ── CtlChecker: T2-C (ρ + ∂ + κ + →) ─────────────────────

impl<'a> GroundsTo for CtlChecker<'a> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Recursion,  // ρ — fixpoint iteration (dominant)
            LexPrimitiva::Boundary,   // ∂ — satisfaction set boundaries
            LexPrimitiva::Comparison, // κ — set membership tests
            LexPrimitiva::Causality,  // → — predecessor image
        ])
        .with_dominant(LexPrimitiva::Recursion, 0.92)
    }
}

// ── LtlBoundedChecker: T2-C (σ + ∂ + κ + ρ) ─────────────

impl<'a> GroundsTo for LtlBoundedChecker<'a> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,   // σ — path enumeration (dominant)
            LexPrimitiva::Boundary,   // ∂ — depth bound
            LexPrimitiva::Comparison, // κ — formula satisfaction
            LexPrimitiva::Recursion,  // ρ — recursive formula eval
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.88)
    }
}

// ── CheckResult: T2-P (κ-dominant) ────────────────────────

impl GroundsTo for CheckResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ — binary verdict (dominant)
            LexPrimitiva::Boundary,   // ∂ — satisfied/violated boundary
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

// ── Counterexample: T2-C (σ + → + ∂) ─────────────────────

impl GroundsTo for Counterexample {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,  // σ — ordered state path (dominant)
            LexPrimitiva::Causality, // → — transition witness
            LexPrimitiva::Boundary,  // ∂ — violation boundary
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.90)
    }
}

// ── Witness: T2-P (N + κ) ─────────────────────────────────

impl GroundsTo for Witness {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N — state counts (dominant)
            LexPrimitiva::Comparison, // κ — ratio comparison
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.92)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kripke_grounding() {
        let comp = KripkeStructure::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Recursion));
        assert_eq!(comp.primitives.len(), 6);
        assert!(comp.confidence > 0.85);
    }

    #[test]
    fn test_kripke_tier() {
        // 6 primitives → T3
        let comp = KripkeStructure::primitive_composition();
        assert_eq!(comp.primitives.len(), 6);
    }

    #[test]
    fn test_ctl_checker_grounding() {
        let comp = CtlChecker::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Recursion));
        assert_eq!(comp.primitives.len(), 4);
    }

    #[test]
    fn test_ltl_checker_grounding() {
        let comp = LtlBoundedChecker::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
        assert_eq!(comp.primitives.len(), 4);
    }

    #[test]
    fn test_check_result_grounding() {
        let comp = CheckResult::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
    }

    #[test]
    fn test_counterexample_grounding() {
        let comp = Counterexample::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
    }

    #[test]
    fn test_witness_grounding() {
        let comp = Witness::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
    }

    #[test]
    fn test_kripke_state_mode() {
        assert_eq!(KripkeStructure::state_mode(), Some(StateMode::Modal));
    }

    #[test]
    fn test_builder_state_mode() {
        assert_eq!(KripkeBuilder::state_mode(), Some(StateMode::Accumulated));
    }
}
