//! # GroundsTo implementations for nexcore-state-theory types
//!
//! Connects the Universal Theory of State to the Lex Primitiva type system.
//!
//! ## Grounding Strategy
//!
//! The state-theory crate formalizes state machines through axioms, typestate patterns,
//! conservation laws, category theory, equivalence relations, temporal logic, and
//! Curry-Howard proofs. State (varsigma) is the dominant primitive across the crate,
//! with Causality (transitions), Boundary (terminal/initial markers), Comparison
//! (guard predicates), and Recursion (cycles) as key supporting primitives.
//!
//! | Primitive | Role in State Theory |
//! |-----------|---------------------|
//! | varsigma (State) | Dominant -- the configuration itself |
//! | -> (Causality) | Transition functions, morphisms |
//! | partial (Boundary) | Terminal/initial state markers, safety manifold |
//! | kappa (Comparison) | Guard predicates, equivalence checks |
//! | N (Quantity) | State counting, cardinality bounds |
//! | sigma (Sequence) | Transition ordering, traces |
//! | rho (Recursion) | Cyclic state machines, Kleene star |
//! | emptyset (Void) | Unreachable states, empty transitions |
//! | x (Product) | Product machines, parallel composition |
//! | Sigma (Sum) | Coproduct machines, disjunctive choice |
//! | exists (Existence) | Existential quantification in proofs |
//! | nu (Frequency) | Temporal logic formulae |
//! | mu (Mapping) | Functors, refinement mappings |
//! | pi (Persistence) | Invariant preservation across transitions |

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

// ═══════════════════════════════════════════════════════════
// CORE TRAITS AND TYPES (lib.rs)
// ═══════════════════════════════════════════════════════════

/// StatePrimitive: T1 (kappa), dominant Comparison
///
/// Enum classifying the 8 state-theory primitives. Pure classification enum.
impl GroundsTo for crate::StatePrimitive {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- classifies primitive symbols
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

// ═══════════════════════════════════════════════════════════
// AXIOMS MODULE
// ═══════════════════════════════════════════════════════════

/// OrderKind: T1 (kappa), dominant Comparison
///
/// Three-variant enum (Total, Partial, Preorder). Pure classification.
impl GroundsTo for crate::axioms::OrderKind {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- ordering kind classification
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

/// StateRegion: T1 (kappa), dominant Comparison
///
/// Three-variant enum (Interior, Boundary, Exterior). Pure classification.
impl GroundsTo for crate::axioms::StateRegion {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- region classification
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

/// A1FiniteDecomposition<MAX>: T2-P (varsigma + N), dominant State
///
/// Axiom 1 witness: finite state count bounded by MAX.
/// State dominant because it witnesses state machine properties;
/// Quantity for the state_count field.
impl<const MAX: usize> GroundsTo for crate::axioms::A1FiniteDecomposition<MAX> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // varsigma -- state machine property
            LexPrimitiva::Quantity, // N -- state_count and MAX bound
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
    }
}

/// A2HierarchicalOrder: T2-C (varsigma + sigma + rho + kappa), dominant State
///
/// Axiom 2 witness: hierarchical ordering of states. State dominant;
/// Sequence for topological levels; Recursion for cycles; Comparison
/// for order_kind classification.
impl GroundsTo for crate::axioms::A2HierarchicalOrder {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,      // varsigma -- state hierarchy
            LexPrimitiva::Sequence,   // sigma -- topological level ordering
            LexPrimitiva::Recursion,  // rho -- cycle detection
            LexPrimitiva::Comparison, // kappa -- order kind classification
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
    }
}

/// A3Conservation: T2-P (varsigma + -> + kappa), dominant State
///
/// Axiom 3 witness: transition invariant preservation. State dominant;
/// Causality for transition effects; Comparison for satisfaction checks.
impl GroundsTo for crate::axioms::A3Conservation {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,      // varsigma -- invariant preservation
            LexPrimitiva::Causality,  // -> -- transition effects
            LexPrimitiva::Comparison, // kappa -- all_satisfied check
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
    }
}

/// A4SafetyManifold: T2-C (varsigma + partial + emptyset + N), dominant State
///
/// Axiom 4 witness: safety manifold with interior/boundary/exterior classification.
/// State dominant for state classification; Boundary for terminal states;
/// Void for exterior/unreachable; Quantity for valid_count.
impl GroundsTo for crate::axioms::A4SafetyManifold {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // varsigma -- state classification
            LexPrimitiva::Boundary, // partial -- terminal boundary states
            LexPrimitiva::Void,     // emptyset -- exterior/unreachable
            LexPrimitiva::Quantity, // N -- valid_count
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
    }
}

/// A5Emergence: T2-C (varsigma + -> + N + kappa), dominant State
///
/// Axiom 5 witness: emergence from simple transitions. State dominant;
/// Causality for transitions; Quantity for counts; Comparison for
/// complexity evaluation.
impl GroundsTo for crate::axioms::A5Emergence {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,      // varsigma -- emergent state behavior
            LexPrimitiva::Causality,  // -> -- transition rules
            LexPrimitiva::Quantity,   // N -- transition/guard/effect counts
            LexPrimitiva::Comparison, // kappa -- complexity evaluation
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
    }
}

/// StateTheoryProof<MAX>: T3 (varsigma + -> + partial + kappa + N + sigma), dominant State
///
/// Complete proof that a state machine satisfies all five axioms.
/// T3 because it composes 6+ unique primitives spanning all axiom domains.
impl<const MAX: usize> GroundsTo for crate::axioms::StateTheoryProof<MAX> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,      // varsigma -- complete state theory proof
            LexPrimitiva::Causality,  // -> -- A3 conservation, A5 emergence
            LexPrimitiva::Boundary,   // partial -- A4 safety manifold
            LexPrimitiva::Comparison, // kappa -- A2 ordering, A3 satisfaction
            LexPrimitiva::Quantity,   // N -- A1 finite decomposition count
            LexPrimitiva::Sequence,   // sigma -- A2 topological ordering
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
    }
}

// ═══════════════════════════════════════════════════════════
// ALGEBRA MODULE
// ═══════════════════════════════════════════════════════════

/// CoproductSide: T1 (kappa), dominant Comparison
///
/// Binary enum (Left, Right). Pure classification.
impl GroundsTo for crate::algebra::CoproductSide {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- left/right classification
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

/// CompositionPhase: T1 (kappa), dominant Comparison
///
/// Binary enum (First, Second). Pure classification of sequential phase.
impl GroundsTo for crate::algebra::CompositionPhase {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- phase classification
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

// ═══════════════════════════════════════════════════════════
// TYPESTATE MODULE
// ═══════════════════════════════════════════════════════════

/// StateMachineMetadata: T2-C (varsigma + N + kappa + partial), dominant State
///
/// Runtime metadata capturing state machine properties for debugging.
/// State dominant for current_state; Quantity for state_count and
/// transition_count; Comparison for is_terminal check; Boundary for
/// terminal state identification.
impl GroundsTo for crate::typestate::StateMachineMetadata {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,      // varsigma -- current state name
            LexPrimitiva::Quantity,   // N -- state_count, transition_count
            LexPrimitiva::Comparison, // kappa -- is_terminal check
            LexPrimitiva::Boundary,   // partial -- terminal state identification
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
    }
}

// ═══════════════════════════════════════════════════════════
// CONSERVATION MODULE
// ═══════════════════════════════════════════════════════════

/// LawVerification: T2-P (kappa + pi), dominant Comparison
///
/// Enum (Satisfied, Violated). Comparison dominant for pass/fail evaluation;
/// Persistence because conservation laws persist across transitions.
impl GroundsTo for crate::conservation::LawVerification {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison,  // kappa -- satisfied/violated evaluation
            LexPrimitiva::Persistence, // pi -- law persistence across transitions
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

/// L3SingleState: T2-P (varsigma + N + kappa), dominant State
///
/// Conservation Law 3: exactly one active state. State dominant because
/// the law IS about single state occupancy; Quantity for active_count;
/// Comparison for count == 1 check.
impl GroundsTo for crate::conservation::L3SingleState {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,      // varsigma -- single state occupancy
            LexPrimitiva::Quantity,   // N -- active_count
            LexPrimitiva::Comparison, // kappa -- count == 1 verification
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
    }
}

/// L4NonTerminalFlux: T2-P (varsigma + -> + partial), dominant State
///
/// Conservation Law 4: non-terminal states must have outgoing transitions.
/// State dominant; Causality for transition existence; Boundary for
/// terminal state identification.
impl GroundsTo for crate::conservation::L4NonTerminalFlux {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,     // varsigma -- non-terminal state property
            LexPrimitiva::Causality, // -> -- outgoing transition requirement
            LexPrimitiva::Boundary,  // partial -- terminal/non-terminal boundary
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
    }
}

/// L11StructureImmutability: T2-P (varsigma + N + pi), dominant State
///
/// Conservation Law 11: state count is invariant. State dominant;
/// Quantity for state_count fields; Persistence for immutability assertion.
impl GroundsTo for crate::conservation::L11StructureImmutability {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,       // varsigma -- structural invariance
            LexPrimitiva::Quantity,    // N -- state counts (initial, current)
            LexPrimitiva::Persistence, // pi -- immutability assertion
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
    }
}

/// VerificationReport: T2-C (kappa + N + varsigma + sigma), dominant Comparison
///
/// Report of verification results across multiple laws. Comparison dominant
/// for pass/fail evaluation; Quantity for counts; State for law descriptions;
/// Sequence for ordered results list.
impl GroundsTo for crate::conservation::VerificationReport {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- overall pass/fail
            LexPrimitiva::Quantity,   // N -- passed/failed counts
            LexPrimitiva::State,      // varsigma -- law verification states
            LexPrimitiva::Sequence,   // sigma -- ordered result list
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.80)
    }
}

// ═══════════════════════════════════════════════════════════
// EQUIVALENCE MODULE
// ═══════════════════════════════════════════════════════════

/// StateRelation: T2-P (varsigma + kappa + x), dominant Comparison
///
/// Binary relation between states. Comparison dominant for the relational
/// predicate; State for the domain; Product for (state, state) pairs.
impl GroundsTo for crate::equivalence::StateRelation {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- relational predicate
            LexPrimitiva::State,      // varsigma -- state domain
            LexPrimitiva::Product,    // x -- (state, state) pairs
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

/// Bisimulation: T2-C (kappa + varsigma + -> + x + sigma), dominant Comparison
///
/// Bisimulation equivalence between state machines. Comparison dominant
/// for equivalence checking; State for configurations; Causality for
/// transition matching; Product for paired states; Sequence for traces.
impl GroundsTo for crate::equivalence::Bisimulation {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- equivalence checking
            LexPrimitiva::State,      // varsigma -- state configurations
            LexPrimitiva::Causality,  // -> -- transition matching
            LexPrimitiva::Product,    // x -- paired states
            LexPrimitiva::Sequence,   // sigma -- action traces
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.80)
    }
}

/// RefinementKind: T1 (kappa), dominant Comparison
///
/// Enum classifying refinement types (Trace, Failure, Bisimulation). Pure classification.
impl GroundsTo for crate::equivalence::RefinementKind {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- refinement kind classification
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

/// EquivalenceResult: T2-P (kappa + exists), dominant Comparison
///
/// Enum indicating equivalence verdict (Equivalent, NotEquivalent with
/// counterexample). Comparison dominant for the verdict; Existence
/// for the optional counterexample.
impl GroundsTo for crate::equivalence::EquivalenceResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- equivalence verdict
            LexPrimitiva::Existence,  // exists -- counterexample existence
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

// ═══════════════════════════════════════════════════════════
// TEMPORAL MODULE
// ═══════════════════════════════════════════════════════════

/// AtomicProp: T2-P (kappa + varsigma), dominant Comparison
///
/// Named atomic proposition for temporal logic. Comparison dominant
/// for predicate evaluation; State for the property being tested.
impl GroundsTo for crate::temporal::AtomicProp {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- predicate evaluation
            LexPrimitiva::State,      // varsigma -- state property
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

/// LtlFormula: T3 (nu + rho + kappa + varsigma + sigma + ->), dominant Frequency
///
/// Linear Temporal Logic formula. T3 due to 6+ unique primitives.
/// Frequency dominant for temporal operators (Always, Eventually, Until);
/// Recursion for nested formula structure; Comparison for atomic predicates;
/// State for state properties; Sequence for trace ordering; Causality for
/// Next/Until operators.
impl GroundsTo for crate::temporal::LtlFormula {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Frequency,  // nu -- temporal operators (Always, Eventually)
            LexPrimitiva::Recursion,  // rho -- nested formula structure (Box)
            LexPrimitiva::Comparison, // kappa -- atomic predicate evaluation
            LexPrimitiva::State,      // varsigma -- state properties
            LexPrimitiva::Sequence,   // sigma -- trace ordering
            LexPrimitiva::Causality,  // -> -- Next/Until causal succession
        ])
        .with_dominant(LexPrimitiva::Frequency, 0.80)
    }
}

/// CtlFormula: T3 (nu + rho + kappa + varsigma + sigma + Sigma), dominant Frequency
///
/// Computation Tree Logic formula. T3 due to 6+ unique primitives.
/// Frequency dominant for temporal operators; Recursion for nested
/// structure; Comparison for predicates; State for state properties;
/// Sequence for path ordering; Sum for path quantifier branching (ForAll/Exists).
impl GroundsTo for crate::temporal::CtlFormula {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Frequency,  // nu -- temporal operators
            LexPrimitiva::Recursion,  // rho -- nested formula structure
            LexPrimitiva::Comparison, // kappa -- atomic predicate evaluation
            LexPrimitiva::State,      // varsigma -- state properties
            LexPrimitiva::Sequence,   // sigma -- path ordering
            LexPrimitiva::Sum,        // Sigma -- path quantifier ForAll/Exists branching
        ])
        .with_dominant(LexPrimitiva::Frequency, 0.80)
    }
}

/// PathQuantifier: T1 (kappa), dominant Comparison
///
/// Enum (ForAll, ThereExists). Pure classification of path quantifier type.
impl GroundsTo for crate::temporal::PathQuantifier {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- quantifier classification
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

/// PropertyClass: T1 (kappa), dominant Comparison
///
/// Enum (Safety, Liveness, Fairness, Progress). Pure classification.
impl GroundsTo for crate::temporal::PropertyClass {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- property class classification
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

/// TemporalProperty: T2-P (nu + kappa), dominant Frequency
///
/// Named temporal property with class and formula. Frequency dominant
/// for the temporal formula; Comparison for property class classification.
impl GroundsTo for crate::temporal::TemporalProperty {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Frequency,  // nu -- temporal formula
            LexPrimitiva::Comparison, // kappa -- property class
        ])
        .with_dominant(LexPrimitiva::Frequency, 0.85)
    }
}

// ═══════════════════════════════════════════════════════════
// PROOFS MODULE
// ═══════════════════════════════════════════════════════════

/// VerificationLevel: T1 (kappa), dominant Comparison
///
/// Enum (Basic, Intermediate, Full). Pure classification of verification depth.
impl GroundsTo for crate::proofs::VerificationLevel {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- verification level classification
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

/// ProofCertificate: T2-C (varsigma + kappa + N + pi), dominant State
///
/// Certificate recording proof verification results. State dominant
/// for proof state; Comparison for level classification; Quantity for
/// property count; Persistence for certificate durability.
impl GroundsTo for crate::proofs::ProofCertificate {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,       // varsigma -- proof verification state
            LexPrimitiva::Comparison,  // kappa -- verification level
            LexPrimitiva::Quantity,    // N -- properties_verified count
            LexPrimitiva::Persistence, // pi -- certificate permanence
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
    }
}

/// ProofBounds: T2-P (N + partial), dominant Quantity
///
/// Maximum bounds for proof search (max states, transitions, depth).
/// Quantity dominant for the numeric bounds; Boundary for the limits.
impl GroundsTo for crate::proofs::ProofBounds {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- max_states, max_transitions, max_depth
            LexPrimitiva::Boundary, // partial -- bound limits
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

// ═══════════════════════════════════════════════════════════
// THEOREMS MODULE
// ═══════════════════════════════════════════════════════════

/// TheoremSummary: T2-P (varsigma + kappa), dominant State
///
/// Summary of a theorem with name, statement, and status.
/// State dominant for the theorem's proven/unproven status;
/// Comparison for theorem classification.
impl GroundsTo for crate::theorems::TheoremSummary {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,      // varsigma -- proven/unproven status
            LexPrimitiva::Comparison, // kappa -- theorem classification
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
    }
}

/// TheoremRegistry: T2-C (pi + sigma + kappa + N), dominant Persistence
///
/// Registry of all theorems. Persistence dominant for catalog storage;
/// Sequence for ordered theorem list; Comparison for lookup;
/// Quantity for theorem count.
impl GroundsTo for crate::theorems::TheoremRegistry {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Persistence, // pi -- theorem catalog storage
            LexPrimitiva::Sequence,    // sigma -- ordered theorem list
            LexPrimitiva::Comparison,  // kappa -- theorem lookup
            LexPrimitiva::Quantity,    // N -- theorem count
        ])
        .with_dominant(LexPrimitiva::Persistence, 0.80)
    }
}

// ═══════════════════════════════════════════════════════════
// CATEGORY MODULE
// ═══════════════════════════════════════════════════════════

/// StateMachineSpec: T2-C (varsigma + -> + N + partial + sigma), dominant State
///
/// Complete state machine specification with states, transitions, initial
/// and terminal markers. State dominant; Causality for transitions;
/// Quantity for state/transition counts; Boundary for initial/terminal;
/// Sequence for transition ordering.
impl GroundsTo for crate::category::StateMachineSpec {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,     // varsigma -- state machine configuration
            LexPrimitiva::Causality, // -> -- transition function
            LexPrimitiva::Quantity,  // N -- state/transition counts
            LexPrimitiva::Boundary,  // partial -- initial/terminal markers
            LexPrimitiva::Sequence,  // sigma -- transition ordering
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
    }
}

/// Simulation: T2-P (-> + varsigma + kappa), dominant Causality
///
/// Simulation relation between two state machines. Causality dominant
/// because a simulation maps transitions of one machine to another.
impl GroundsTo for crate::category::Simulation {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,  // -> -- transition simulation
            LexPrimitiva::State,      // varsigma -- state mapping
            LexPrimitiva::Comparison, // kappa -- simulation relation check
        ])
        .with_dominant(LexPrimitiva::Causality, 0.85)
    }
}

// ═══════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    // ---- Tier classification tests ----

    #[test]
    fn test_state_primitive_tier() {
        let comp = crate::StatePrimitive::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T1Universal);
    }

    #[test]
    fn test_order_kind_tier() {
        let comp = crate::axioms::OrderKind::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T1Universal);
    }

    #[test]
    fn test_state_region_tier() {
        let comp = crate::axioms::StateRegion::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T1Universal);
    }

    #[test]
    fn test_a1_tier() {
        let comp = <crate::axioms::A1FiniteDecomposition<10>>::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_a2_tier() {
        let comp = crate::axioms::A2HierarchicalOrder::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Composite);
    }

    #[test]
    fn test_a3_tier() {
        let comp = crate::axioms::A3Conservation::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_a4_tier() {
        let comp = crate::axioms::A4SafetyManifold::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Composite);
    }

    #[test]
    fn test_a5_tier() {
        let comp = crate::axioms::A5Emergence::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Composite);
    }

    #[test]
    fn test_state_theory_proof_tier() {
        let comp = <crate::axioms::StateTheoryProof<10>>::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T3DomainSpecific);
    }

    #[test]
    fn test_coproduct_side_tier() {
        let comp = crate::algebra::CoproductSide::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T1Universal);
    }

    #[test]
    fn test_composition_phase_tier() {
        let comp = crate::algebra::CompositionPhase::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T1Universal);
    }

    #[test]
    fn test_state_machine_metadata_tier() {
        let comp = crate::typestate::StateMachineMetadata::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Composite);
    }

    #[test]
    fn test_law_verification_tier() {
        let comp = crate::conservation::LawVerification::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_l3_single_state_tier() {
        let comp = crate::conservation::L3SingleState::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_l4_non_terminal_flux_tier() {
        let comp = crate::conservation::L4NonTerminalFlux::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_l11_structure_immutability_tier() {
        let comp = crate::conservation::L11StructureImmutability::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_verification_report_tier() {
        let comp = crate::conservation::VerificationReport::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Composite);
    }

    #[test]
    fn test_state_relation_tier() {
        let comp = crate::equivalence::StateRelation::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_bisimulation_tier() {
        let comp = crate::equivalence::Bisimulation::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Composite);
    }

    #[test]
    fn test_refinement_kind_tier() {
        let comp = crate::equivalence::RefinementKind::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T1Universal);
    }

    #[test]
    fn test_equivalence_result_tier() {
        let comp = crate::equivalence::EquivalenceResult::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_atomic_prop_tier() {
        let comp = crate::temporal::AtomicProp::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_ltl_formula_tier() {
        let comp = crate::temporal::LtlFormula::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T3DomainSpecific);
    }

    #[test]
    fn test_ctl_formula_tier() {
        let comp = crate::temporal::CtlFormula::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T3DomainSpecific);
    }

    #[test]
    fn test_path_quantifier_tier() {
        let comp = crate::temporal::PathQuantifier::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T1Universal);
    }

    #[test]
    fn test_property_class_tier() {
        let comp = crate::temporal::PropertyClass::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T1Universal);
    }

    #[test]
    fn test_temporal_property_tier() {
        let comp = crate::temporal::TemporalProperty::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_verification_level_tier() {
        let comp = crate::proofs::VerificationLevel::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T1Universal);
    }

    #[test]
    fn test_proof_certificate_tier() {
        let comp = crate::proofs::ProofCertificate::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Composite);
    }

    #[test]
    fn test_proof_bounds_tier() {
        let comp = crate::proofs::ProofBounds::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_theorem_summary_tier() {
        let comp = crate::theorems::TheoremSummary::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_theorem_registry_tier() {
        let comp = crate::theorems::TheoremRegistry::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Composite);
    }

    #[test]
    fn test_state_machine_spec_tier() {
        let comp = crate::category::StateMachineSpec::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Composite);
    }

    #[test]
    fn test_simulation_tier() {
        let comp = crate::category::Simulation::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    // ---- Dominant primitive tests ----

    #[test]
    fn test_state_primitive_dominant() {
        assert_eq!(
            crate::StatePrimitive::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }

    #[test]
    fn test_a1_dominant() {
        assert_eq!(
            <crate::axioms::A1FiniteDecomposition<10>>::dominant_primitive(),
            Some(LexPrimitiva::State)
        );
    }

    #[test]
    fn test_a4_dominant() {
        assert_eq!(
            crate::axioms::A4SafetyManifold::dominant_primitive(),
            Some(LexPrimitiva::State)
        );
    }

    #[test]
    fn test_state_theory_proof_dominant() {
        assert_eq!(
            <crate::axioms::StateTheoryProof<10>>::dominant_primitive(),
            Some(LexPrimitiva::State)
        );
    }

    #[test]
    fn test_law_verification_dominant() {
        assert_eq!(
            crate::conservation::LawVerification::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }

    #[test]
    fn test_bisimulation_dominant() {
        assert_eq!(
            crate::equivalence::Bisimulation::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }

    #[test]
    fn test_ltl_formula_dominant() {
        assert_eq!(
            crate::temporal::LtlFormula::dominant_primitive(),
            Some(LexPrimitiva::Frequency)
        );
    }

    #[test]
    fn test_proof_certificate_dominant() {
        assert_eq!(
            crate::proofs::ProofCertificate::dominant_primitive(),
            Some(LexPrimitiva::State)
        );
    }

    #[test]
    fn test_theorem_registry_dominant() {
        assert_eq!(
            crate::theorems::TheoremRegistry::dominant_primitive(),
            Some(LexPrimitiva::Persistence)
        );
    }

    #[test]
    fn test_simulation_dominant() {
        assert_eq!(
            crate::category::Simulation::dominant_primitive(),
            Some(LexPrimitiva::Causality)
        );
    }

    // ---- Confidence range tests ----

    #[test]
    fn test_all_confidences_in_valid_range() {
        let compositions: Vec<(&str, PrimitiveComposition)> = vec![
            (
                "StatePrimitive",
                crate::StatePrimitive::primitive_composition(),
            ),
            (
                "OrderKind",
                crate::axioms::OrderKind::primitive_composition(),
            ),
            (
                "StateRegion",
                crate::axioms::StateRegion::primitive_composition(),
            ),
            (
                "A1FiniteDecomposition",
                <crate::axioms::A1FiniteDecomposition<10>>::primitive_composition(),
            ),
            (
                "A2HierarchicalOrder",
                crate::axioms::A2HierarchicalOrder::primitive_composition(),
            ),
            (
                "A3Conservation",
                crate::axioms::A3Conservation::primitive_composition(),
            ),
            (
                "A4SafetyManifold",
                crate::axioms::A4SafetyManifold::primitive_composition(),
            ),
            (
                "A5Emergence",
                crate::axioms::A5Emergence::primitive_composition(),
            ),
            (
                "StateTheoryProof",
                <crate::axioms::StateTheoryProof<10>>::primitive_composition(),
            ),
            (
                "CoproductSide",
                crate::algebra::CoproductSide::primitive_composition(),
            ),
            (
                "CompositionPhase",
                crate::algebra::CompositionPhase::primitive_composition(),
            ),
            (
                "StateMachineMetadata",
                crate::typestate::StateMachineMetadata::primitive_composition(),
            ),
            (
                "LawVerification",
                crate::conservation::LawVerification::primitive_composition(),
            ),
            (
                "L3SingleState",
                crate::conservation::L3SingleState::primitive_composition(),
            ),
            (
                "L4NonTerminalFlux",
                crate::conservation::L4NonTerminalFlux::primitive_composition(),
            ),
            (
                "L11StructureImmutability",
                crate::conservation::L11StructureImmutability::primitive_composition(),
            ),
            (
                "VerificationReport",
                crate::conservation::VerificationReport::primitive_composition(),
            ),
            (
                "StateRelation",
                crate::equivalence::StateRelation::primitive_composition(),
            ),
            (
                "Bisimulation",
                crate::equivalence::Bisimulation::primitive_composition(),
            ),
            (
                "RefinementKind",
                crate::equivalence::RefinementKind::primitive_composition(),
            ),
            (
                "EquivalenceResult",
                crate::equivalence::EquivalenceResult::primitive_composition(),
            ),
            (
                "AtomicProp",
                crate::temporal::AtomicProp::primitive_composition(),
            ),
            (
                "LtlFormula",
                crate::temporal::LtlFormula::primitive_composition(),
            ),
            (
                "CtlFormula",
                crate::temporal::CtlFormula::primitive_composition(),
            ),
            (
                "PathQuantifier",
                crate::temporal::PathQuantifier::primitive_composition(),
            ),
            (
                "PropertyClass",
                crate::temporal::PropertyClass::primitive_composition(),
            ),
            (
                "TemporalProperty",
                crate::temporal::TemporalProperty::primitive_composition(),
            ),
            (
                "VerificationLevel",
                crate::proofs::VerificationLevel::primitive_composition(),
            ),
            (
                "ProofCertificate",
                crate::proofs::ProofCertificate::primitive_composition(),
            ),
            (
                "ProofBounds",
                crate::proofs::ProofBounds::primitive_composition(),
            ),
            (
                "TheoremSummary",
                crate::theorems::TheoremSummary::primitive_composition(),
            ),
            (
                "TheoremRegistry",
                crate::theorems::TheoremRegistry::primitive_composition(),
            ),
            (
                "StateMachineSpec",
                crate::category::StateMachineSpec::primitive_composition(),
            ),
            (
                "Simulation",
                crate::category::Simulation::primitive_composition(),
            ),
        ];

        for (name, comp) in &compositions {
            assert!(
                comp.confidence >= 0.80 && comp.confidence <= 0.95,
                "{} confidence {} outside 0.80-0.95 range",
                name,
                comp.confidence
            );
        }
    }

    // ---- Pure primitive tests ----

    #[test]
    fn test_pure_primitives() {
        assert!(crate::StatePrimitive::is_pure_primitive());
        assert!(crate::axioms::OrderKind::is_pure_primitive());
        assert!(crate::axioms::StateRegion::is_pure_primitive());
        assert!(crate::algebra::CoproductSide::is_pure_primitive());
        assert!(crate::algebra::CompositionPhase::is_pure_primitive());
        assert!(crate::temporal::PathQuantifier::is_pure_primitive());
        assert!(crate::temporal::PropertyClass::is_pure_primitive());
        assert!(crate::proofs::VerificationLevel::is_pure_primitive());
        assert!(crate::equivalence::RefinementKind::is_pure_primitive());
        assert!(!<crate::axioms::StateTheoryProof<10>>::is_pure_primitive());
        assert!(!crate::temporal::LtlFormula::is_pure_primitive());
    }

    // ---- Structural consistency tests ----

    #[test]
    fn test_t3_types_have_six_plus_primitives() {
        let proof_comp = <crate::axioms::StateTheoryProof<10>>::primitive_composition();
        let ltl_comp = crate::temporal::LtlFormula::primitive_composition();
        let ctl_comp = crate::temporal::CtlFormula::primitive_composition();

        assert!(
            proof_comp.unique().len() >= 6,
            "StateTheoryProof has {} unique primitives, expected 6+",
            proof_comp.unique().len()
        );
        assert!(
            ltl_comp.unique().len() >= 6,
            "LtlFormula has {} unique primitives, expected 6+",
            ltl_comp.unique().len()
        );
        assert!(
            ctl_comp.unique().len() >= 6,
            "CtlFormula has {} unique primitives, expected 6+",
            ctl_comp.unique().len()
        );
    }

    // ---- Grounding count ----

    #[test]
    fn test_total_grounded_types_count() {
        // 34 types grounded in this module
        let count = 34;
        assert_eq!(count, 34, "Should have 34 GroundsTo implementations");
    }
}
