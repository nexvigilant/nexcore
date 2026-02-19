// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Universal Theory of State
//!
//! Domain-agnostic foundations for state machines, extracted from the
//! Theory of Vigilance (ToV) and generalized for universal application.
//!
//! ## Core Thesis
//!
//! **All computation is state transformation.**
//!
//! Every program, system, or process can be modeled as:
//! - A set of **states** (the configuration space)
//! - A set of **transitions** (the transformation rules)
//! - **Conservation laws** (invariants preserved across transitions)
//! - **Boundary conditions** (terminal and initial states)
//!
//! ## Primitive Grounding
//!
//! The theory grounds to the Lex Primitiva:
//!
//! | Symbol | Name | Role in State Theory |
//! |--------|------|---------------------|
//! | ς | State | **Dominant** — the configuration itself |
//! | → | Causality | Transition functions |
//! | ∂ | Boundary | Terminal/initial state markers |
//! | κ | Comparison | Guard predicates |
//! | N | Quantity | State counting, cardinality bounds |
//! | σ | Sequence | Transition ordering |
//! | ρ | Recursion | Cyclic state machines |
//! | ∅ | Void | Unreachable states, empty transitions |
//!
//! ## Axiom System
//!
//! Five axioms govern all state machines:
//!
//! | Axiom | Name | Statement |
//! |-------|------|-----------|
//! | **A1** | Finite Decomposition | Every state machine has finitely many states |
//! | **A2** | Hierarchical Order | States admit a partial order (DAG or cycles) |
//! | **A3** | Conservation | Transitions preserve designated invariants |
//! | **A4** | Safety Manifold | Valid states form interior; terminal states form boundary |
//! | **A5** | Emergence | Complex behavior emerges from simple transition rules |
//!
//! ## Module Structure
//!
//! ```text
//! nexcore-state-theory/
//! ├── axioms/        # A1-A5 formal definitions and witnesses
//! ├── typestate/     # Compile-time state enforcement patterns
//! ├── conservation/  # Invariant preservation proofs
//! └── algebra/       # State composition, products, coproducts
//! ```
//!
//! ## Example: Minimal State Machine
//!
//! ```rust
//! use nexcore_state_theory::prelude::*;
//!
//! // Define states as zero-sized types
//! struct Off;
//! struct On;
//!
//! // Implement the State trait
//! impl State for Off {
//!     fn name() -> &'static str { "off" }
//!     fn is_terminal() -> bool { false }
//! }
//!
//! impl State for On {
//!     fn name() -> &'static str { "on" }
//!     fn is_terminal() -> bool { false }
//! }
//!
//! // Typestate wrapper
//! struct Switch<S: State> {
//!     _state: core::marker::PhantomData<S>,
//! }
//!
//! impl Switch<Off> {
//!     fn turn_on(self) -> Switch<On> {
//!         Switch { _state: core::marker::PhantomData }
//!     }
//! }
//!
//! impl Switch<On> {
//!     fn turn_off(self) -> Switch<Off> {
//!         Switch { _state: core::marker::PhantomData }
//!     }
//! }
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(missing_docs)]

extern crate alloc;

// Core modules
pub mod algebra;
pub mod axioms;
pub mod conservation;
pub mod grounding;
pub mod typestate;

// Advanced modules
pub mod category;
pub mod equivalence;
pub mod proofs;
pub mod temporal;
pub mod theorems;

/// Prelude for convenient imports.
pub mod prelude {
    // Axioms
    pub use crate::axioms::{
        A1FiniteDecomposition, A2HierarchicalOrder, A3Conservation, A4SafetyManifold, A5Emergence,
        Axiom, StateTheoryProof,
    };

    // Core traits
    pub use crate::{State, StatePrimitive, Transition};

    // Typestate
    pub use crate::typestate::{TransitionBuilder, TypesafeWrapper};

    // Conservation
    pub use crate::conservation::{
        ConservationLaw, Invariant, L3SingleState, L4NonTerminalFlux, L11StructureImmutability,
        LawVerification,
    };

    // Algebra
    pub use crate::algebra::{StateComposition, StateCoproduct, StateIteration, StateProduct};

    // Category theory
    pub use crate::category::{CoproductMachine, ProductMachine, Simulation, StateMachineSpec};

    // Temporal logic
    pub use crate::temporal::{
        AtomicProp, CtlFormula, LtlFormula, PropertyClass, StandardProperties, TemporalProperty,
    };

    // Equivalence
    pub use crate::equivalence::{
        Bisimulation, BisimulationQuotient, Refinement, SimulationPreorder, StateRelation,
        TraceEquivalence,
    };

    // Proofs
    pub use crate::proofs::{
        And, Exists, ForAll, Implies, Not, Or, Proof, ProofCertificate, ProofRegistry,
        VerificationLevel,
    };

    // Theorems
    pub use crate::theorems::{
        T1BisimulationPreservesCTL, T2ParallelPreservesSafety, T3SequentialPreservesTermination,
        T5RefinementPreservesSafety, Theorem, TheoremRegistry, TheoremSummary,
    };
}

// ═══════════════════════════════════════════════════════════
// CORE TRAIT: State
// ═══════════════════════════════════════════════════════════

/// The fundamental trait for state markers.
///
/// States are zero-sized types that exist only at compile time,
/// enabling the typestate pattern for compile-time state enforcement.
///
/// ## Tier Classification
///
/// `State` implementations are T1 primitives (ς-dominant).
pub trait State: Sized + 'static {
    /// Human-readable state name.
    fn name() -> &'static str;

    /// Whether this state is terminal (no outgoing transitions).
    fn is_terminal() -> bool;

    /// Whether this state is initial (entry point).
    fn is_initial() -> bool {
        false
    }

    /// Numeric identifier for runtime discrimination.
    fn id() -> u32 {
        0
    }
}

// ═══════════════════════════════════════════════════════════
// CORE TRAIT: Transition
// ═══════════════════════════════════════════════════════════

/// A transition between two states.
///
/// Transitions are compile-time verified via method availability:
/// - Method exists → transition allowed
/// - Method absent → compile error
///
/// ## Tier Classification
///
/// `Transition` is T2-P (ς + →).
pub trait Transition<From: State, To: State> {
    /// The guard predicate type (if any).
    type Guard;

    /// The effect type (if any).
    type Effect;

    /// Human-readable transition name.
    fn name() -> &'static str;

    /// Whether this transition is guarded.
    fn is_guarded() -> bool {
        false
    }
}

// ═══════════════════════════════════════════════════════════
// PRIMITIVE SYMBOLS
// ═══════════════════════════════════════════════════════════

/// The 8 primitives relevant to state theory.
///
/// Extracted from the full 15-symbol Lex Primitiva.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatePrimitive {
    /// ς — State (dominant)
    State,
    /// → — Causality (transitions)
    Causality,
    /// ∂ — Boundary (terminal/initial)
    Boundary,
    /// κ — Comparison (guards)
    Comparison,
    /// N — Quantity (cardinality)
    Quantity,
    /// σ — Sequence (ordering)
    Sequence,
    /// ρ — Recursion (cycles)
    Recursion,
    /// ∅ — Void (unreachable)
    Void,
}

impl StatePrimitive {
    /// Unicode symbol for this primitive.
    #[must_use]
    pub const fn symbol(&self) -> char {
        match self {
            Self::State => 'ς',
            Self::Causality => '→',
            Self::Boundary => '∂',
            Self::Comparison => 'κ',
            Self::Quantity => 'N',
            Self::Sequence => 'σ',
            Self::Recursion => 'ρ',
            Self::Void => '∅',
        }
    }

    /// All primitives in canonical order.
    #[must_use]
    pub const fn all() -> [Self; 8] {
        [
            Self::State,
            Self::Causality,
            Self::Boundary,
            Self::Comparison,
            Self::Quantity,
            Self::Sequence,
            Self::Recursion,
            Self::Void,
        ]
    }
}

// ═══════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    struct TestOff;
    struct TestOn;

    impl State for TestOff {
        fn name() -> &'static str {
            "off"
        }
        fn is_terminal() -> bool {
            false
        }
        fn is_initial() -> bool {
            true
        }
    }

    impl State for TestOn {
        fn name() -> &'static str {
            "on"
        }
        fn is_terminal() -> bool {
            false
        }
    }

    #[test]
    fn test_state_trait() {
        assert_eq!(TestOff::name(), "off");
        assert!(TestOff::is_initial());
        assert!(!TestOff::is_terminal());

        assert_eq!(TestOn::name(), "on");
        assert!(!TestOn::is_initial());
        assert!(!TestOn::is_terminal());
    }

    #[test]
    fn test_primitive_symbols() {
        assert_eq!(StatePrimitive::State.symbol(), 'ς');
        assert_eq!(StatePrimitive::Causality.symbol(), '→');
        assert_eq!(StatePrimitive::all().len(), 8);
    }
}
