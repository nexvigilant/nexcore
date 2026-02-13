//! # GroundsTo implementations for stem-core types
//!
//! Connects the scientific method traits and composites to the Lex Primitiva
//! type system.
//!
//! ## Crate Primitive Profile
//!
//! stem-core defines the SCIENCE loop: Sense, Classify, Infer, Experiment,
//! Normalize, Codify, Emit, Extend. Each trait is T2-P grounded in a single T1
//! primitive. The composites (Science, Integrity, AutonomousLoop,
//! ContextInjection) combine multiple T1 primitives.
//!
//! - **Confidence**: T1 Quantity (N) -- a single numeric value in [0,1]
//! - **Measured<T>**: T2-P (N + Mapping) -- value paired with confidence
//! - **Correction<T>**: T2-P (Causality + State) -- original -> corrected
//! - **Integrity<T>**: T2-C (State + Comparison + Boundary) -- validated gate
//! - **AutonomousLoop**: T2-C (Frequency + Causality + Recursion) -- retry
//! - **ContextInjection<C>**: T2-C (Persistence + State + Sequence) -- inject
//! - **LoopOutcome<E>**: T2-P (Sum + Boundary) -- terminal vs retryable
//! - **IntegrityError**: T2-P (Boundary) -- validation failure
//! - **ScienceError**: T2-P (Boundary) -- method execution failure
//! - **Machine types**: see machine module grounding below

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::machine::{
    ComponentId, Determinism, Input, Machine, Mechanism, Output, TransferConfidence,
};
use crate::{
    AutonomousLoop, ContextInjection, Integrity, IntegrityError, LoopOutcome, ScienceError,
};

// ===========================================================================
// Bedrock types (re-exported from nexcore-constants)
// ===========================================================================

// Note: Confidence, Measured, Correction, Tier are re-exported from
// nexcore-constants and should be grounded there, not here.
// We ground the types that are DEFINED in this crate.

// ===========================================================================
// Integrity Gate -- State + Comparison + Boundary dominant
// ===========================================================================

/// Integrity<T>: T2-C (State + Comparison + Boundary), dominant State
///
/// A validated gate that holds data which has passed comparison-based
/// boundary checks. State-dominant: the gate IS encapsulated state that
/// enforces invariants.
impl<T> GroundsTo for Integrity<T> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,      // varsigma -- holds validated data
            LexPrimitiva::Comparison, // kappa -- validates against constraints
            LexPrimitiva::Boundary,   // partial -- Result boundary, blocks invalid
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
    }
}

/// IntegrityError: T1 (Boundary), dominant Boundary
///
/// Represents a validation failure at the integrity boundary.
/// Pure boundary: it IS the notification that a crossing was rejected.
impl GroundsTo for IntegrityError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // partial -- validation rejection
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.95)
    }
}

// ===========================================================================
// Autonomous Loop -- Frequency + Causality + Recursion dominant
// ===========================================================================

/// AutonomousLoop: T2-C (Frequency + Causality + Recursion + Boundary),
/// dominant Recursion
///
/// A retry loop that re-enters action logic bounded by frequency limits.
/// Recursion-dominant: the loop IS self-referential re-entry until a
/// boundary condition (max_attempts) is met.
impl GroundsTo for AutonomousLoop {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Recursion, // rho -- re-enters action logic
            LexPrimitiva::Frequency, // nu -- max_attempts rate limit
            LexPrimitiva::Causality, // arrow -- result triggers next attempt
            LexPrimitiva::Boundary,  // partial -- max_attempts termination
        ])
        .with_dominant(LexPrimitiva::Recursion, 0.80)
    }
}

/// LoopOutcome<E>: T2-P (Sum + Boundary), dominant Sum
///
/// A two-variant enum: Terminal or Retryable. This is a sum type
/// that classifies error outcomes for the loop.
impl<E> GroundsTo for LoopOutcome<E> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,      // Sigma -- two-variant sum type
            LexPrimitiva::Boundary, // partial -- terminal vs retryable boundary
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

// ===========================================================================
// Context Injection -- Persistence + State + Sequence dominant
// ===========================================================================

/// ContextInjection<C>: T2-C (Persistence + State + Sequence),
/// dominant State
///
/// Injects persisted context into a sequential execution. The active
/// context snapshot is the primary purpose -- State dominant.
impl<C> GroundsTo for ContextInjection<C> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,       // varsigma -- active context snapshot
            LexPrimitiva::Persistence, // pi -- fetched from long-term storage
            LexPrimitiva::Sequence,    // sigma -- ordered operations using context
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
    }
}

// ===========================================================================
// Science Error -- Boundary dominant
// ===========================================================================

/// ScienceError: T1 (Boundary), dominant Boundary
///
/// Error enum for scientific method execution failures.
/// Pure boundary: each variant represents a failed boundary crossing
/// in the SCIENCE loop.
impl GroundsTo for ScienceError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // partial -- method execution failure
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.95)
    }
}

// ===========================================================================
// Machine module types
// ===========================================================================

/// ComponentId: T1 (Quantity), dominant Quantity
///
/// A newtype wrapper around u64 for component identification.
/// Pure quantity: it IS a numeric identifier.
impl GroundsTo for ComponentId {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- numeric identifier
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.95)
    }
}

/// Determinism: T2-P (Comparison + Quantity), dominant Comparison
///
/// Quantified repeatability of a transformation. Comparison-dominant:
/// the core question is "same input -> same output?" which is a
/// comparison of outputs across runs.
impl GroundsTo for Determinism {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- same input -> same output?
            LexPrimitiva::Quantity,   // N -- repeatability score in [0,1]
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

/// Mechanism<I, O>: T2-C (Sequence + Causality + Comparison), dominant Sequence
///
/// An ordered chain of operations producing an aggregate effect.
/// Sequence-dominant: the steps have defined order and each step's
/// output causes the next step's input.
impl<I, O> GroundsTo for Mechanism<I, O> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,   // sigma -- ordered step chain
            LexPrimitiva::Causality,  // arrow -- each output causes next input
            LexPrimitiva::Comparison, // kappa -- determinism assessment
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.80)
    }
}

/// Input<T>: T2-P (State + Sequence), dominant State
///
/// A value entering a machine with sequence tracking.
/// State-dominant: it IS the data state before transformation.
impl<T> GroundsTo for Input<T> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // varsigma -- data state before transform
            LexPrimitiva::Sequence, // sigma -- monotonic sequence number
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
    }
}

/// Output<T>: T2-C (State + Sequence + Quantity), dominant State
///
/// A value exiting a machine with confidence and source tracking.
/// State-dominant: it IS the data state after transformation.
impl<T> GroundsTo for Output<T> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // varsigma -- data state after transform
            LexPrimitiva::Sequence, // sigma -- source_seq linking to input
            LexPrimitiva::Quantity, // N -- confidence value
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
    }
}

/// Machine<I, O>: T3 (Sequence + Mapping + State + Causality + Comparison + Quantity),
/// dominant Mapping
///
/// A system that transforms inputs to outputs through a mechanism.
/// Mapping-dominant: the machine's primary purpose IS transformation
/// of I -> O.
impl<I, O> GroundsTo for Machine<I, O> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,    // mu -- I -> O transformation
            LexPrimitiva::Sequence,   // sigma -- ordered step chain + seq numbers
            LexPrimitiva::State,      // varsigma -- internal next_seq counter
            LexPrimitiva::Causality,  // arrow -- mechanism causal chain
            LexPrimitiva::Comparison, // kappa -- determinism
            LexPrimitiva::Quantity,   // N -- confidence, sequence counts
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

/// TransferConfidence: T2-C (Quantity + Comparison + Mapping), dominant Quantity
///
/// Three-dimensional confidence assessment for cross-domain transfer.
/// Quantity-dominant: the struct IS three numeric scores that quantify
/// transfer quality.
impl GroundsTo for TransferConfidence {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N -- structural/functional/contextual scores
            LexPrimitiva::Comparison, // kappa -- limiting_factor comparison
            LexPrimitiva::Mapping,    // mu -- combined() formula
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.80)
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    // -----------------------------------------------------------------------
    // Integrity Gate
    // -----------------------------------------------------------------------

    #[test]
    fn integrity_is_t2p_state_dominant() {
        assert_eq!(Integrity::<i32>::tier(), Tier::T2Primitive);
        assert_eq!(
            Integrity::<i32>::primitive_composition().dominant,
            Some(LexPrimitiva::State)
        );
        let comp = Integrity::<i32>::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
    }

    #[test]
    fn integrity_error_is_t1_boundary_dominant() {
        assert_eq!(IntegrityError::tier(), Tier::T1Universal);
        assert_eq!(
            IntegrityError::primitive_composition().dominant,
            Some(LexPrimitiva::Boundary)
        );
        assert!(IntegrityError::is_pure_primitive());
    }

    // -----------------------------------------------------------------------
    // Autonomous Loop
    // -----------------------------------------------------------------------

    #[test]
    fn autonomous_loop_is_t2c_recursion_dominant() {
        assert_eq!(AutonomousLoop::tier(), Tier::T2Composite);
        assert_eq!(
            AutonomousLoop::primitive_composition().dominant,
            Some(LexPrimitiva::Recursion)
        );
        let comp = AutonomousLoop::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Frequency));
        assert!(comp.primitives.contains(&LexPrimitiva::Causality));
    }

    #[test]
    fn loop_outcome_is_t2p_sum_dominant() {
        assert_eq!(LoopOutcome::<String>::tier(), Tier::T2Primitive);
        assert_eq!(
            LoopOutcome::<String>::primitive_composition().dominant,
            Some(LexPrimitiva::Sum)
        );
    }

    // -----------------------------------------------------------------------
    // Context Injection
    // -----------------------------------------------------------------------

    #[test]
    fn context_injection_is_t2p_state_dominant() {
        assert_eq!(ContextInjection::<()>::tier(), Tier::T2Primitive);
        assert_eq!(
            ContextInjection::<()>::primitive_composition().dominant,
            Some(LexPrimitiva::State)
        );
        let comp = ContextInjection::<()>::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Persistence));
        assert!(comp.primitives.contains(&LexPrimitiva::Sequence));
    }

    // -----------------------------------------------------------------------
    // Science Error
    // -----------------------------------------------------------------------

    #[test]
    fn science_error_is_t1_boundary_dominant() {
        assert_eq!(ScienceError::tier(), Tier::T1Universal);
        assert_eq!(
            ScienceError::primitive_composition().dominant,
            Some(LexPrimitiva::Boundary)
        );
    }

    // -----------------------------------------------------------------------
    // Machine module types
    // -----------------------------------------------------------------------

    #[test]
    fn component_id_is_t1_quantity_dominant() {
        assert_eq!(ComponentId::tier(), Tier::T1Universal);
        assert_eq!(
            ComponentId::primitive_composition().dominant,
            Some(LexPrimitiva::Quantity)
        );
        assert!(ComponentId::is_pure_primitive());
    }

    #[test]
    fn determinism_is_t2p_comparison_dominant() {
        assert_eq!(<Determinism as GroundsTo>::tier(), Tier::T2Primitive);
        assert_eq!(
            Determinism::primitive_composition().dominant,
            Some(LexPrimitiva::Comparison)
        );
    }

    #[test]
    fn mechanism_is_t2p_sequence_dominant() {
        assert_eq!(<Mechanism<(), ()> as GroundsTo>::tier(), Tier::T2Primitive);
        assert_eq!(
            Mechanism::<(), ()>::primitive_composition().dominant,
            Some(LexPrimitiva::Sequence)
        );
    }

    #[test]
    fn input_is_t2p_state_dominant() {
        assert_eq!(Input::<i32>::tier(), Tier::T2Primitive);
        assert_eq!(
            Input::<i32>::primitive_composition().dominant,
            Some(LexPrimitiva::State)
        );
    }

    #[test]
    fn output_is_t2p_state_dominant() {
        assert_eq!(Output::<i32>::tier(), Tier::T2Primitive);
        assert_eq!(
            Output::<i32>::primitive_composition().dominant,
            Some(LexPrimitiva::State)
        );
    }

    #[test]
    fn machine_is_t3_mapping_dominant() {
        assert_eq!(
            <Machine<(), ()> as GroundsTo>::tier(),
            Tier::T3DomainSpecific
        );
        assert_eq!(
            Machine::<(), ()>::primitive_composition().dominant,
            Some(LexPrimitiva::Mapping)
        );
    }

    #[test]
    fn transfer_confidence_is_t2p_quantity_dominant() {
        assert_eq!(TransferConfidence::tier(), Tier::T2Primitive);
        assert_eq!(
            TransferConfidence::primitive_composition().dominant,
            Some(LexPrimitiva::Quantity)
        );
    }

    // -----------------------------------------------------------------------
    // Cross-cutting assertions
    // -----------------------------------------------------------------------

    #[test]
    fn all_error_types_are_boundary_dominant() {
        let errors = [
            IntegrityError::dominant_primitive(),
            ScienceError::dominant_primitive(),
        ];
        for dom in errors {
            assert_eq!(dom, Some(LexPrimitiva::Boundary));
        }
    }

    #[test]
    fn tier_distribution_is_reasonable() {
        // T1: IntegrityError, ScienceError, ComponentId = 3
        let t1_count = [
            IntegrityError::tier(),
            ScienceError::tier(),
            ComponentId::tier(),
        ]
        .iter()
        .filter(|t| **t == Tier::T1Universal)
        .count();

        // T2-P (2-3 primitives): LoopOutcome, Determinism, Input,
        //   Integrity, ContextInjection, Mechanism, Output,
        //   TransferConfidence = 8
        let t2p_count = [
            LoopOutcome::<()>::tier(),
            <Determinism as GroundsTo>::tier(),
            Input::<()>::tier(),
            Integrity::<()>::tier(),
            ContextInjection::<()>::tier(),
            <Mechanism<(), ()> as GroundsTo>::tier(),
            Output::<()>::tier(),
            TransferConfidence::tier(),
        ]
        .iter()
        .filter(|t| **t == Tier::T2Primitive)
        .count();

        // T2-C (4-5 primitives): AutonomousLoop = 1
        let t2c_count = [AutonomousLoop::tier()]
            .iter()
            .filter(|t| **t == Tier::T2Composite)
            .count();

        // T3 (6+ primitives): Machine = 1
        let t3_count = [<Machine<(), ()> as GroundsTo>::tier()]
            .iter()
            .filter(|t| **t == Tier::T3DomainSpecific)
            .count();

        assert_eq!(t1_count, 3, "expected 3 T1 types");
        assert_eq!(t2p_count, 8, "expected 8 T2-P types");
        assert_eq!(t2c_count, 1, "expected 1 T2-C type");
        assert_eq!(t3_count, 1, "expected 1 T3 type");
    }
}
