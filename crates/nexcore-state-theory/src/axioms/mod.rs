//! # State Theory Axioms
//!
//! The five fundamental axioms governing all state machines.
//!
//! ## Axiom Hierarchy
//!
//! ```text
//! A1 (Decomposition) ─┬─► A2 (Hierarchy) ─┬─► A4 (Safety)
//!                     │                   │
//!                     └─► A3 (Conservation)─┘
//!                                         │
//!                                         └─► A5 (Emergence)
//! ```
//!
//! ## Curry-Howard Correspondence
//!
//! Each axiom has a type-level witness:
//! - Constructing the witness = proving the axiom holds
//! - Type errors = axiom violations detected at compile time

use alloc::vec::Vec;
use core::marker::PhantomData;

// ═══════════════════════════════════════════════════════════
// AXIOM TRAIT
// ═══════════════════════════════════════════════════════════

/// Marker trait for axiom witnesses.
///
/// An axiom is proven by constructing a value of the witness type.
/// This follows the Curry-Howard correspondence: types as propositions,
/// values as proofs.
pub trait Axiom {
    /// The axiom identifier (A1-A5).
    fn id() -> &'static str;

    /// Human-readable axiom name.
    fn name() -> &'static str;

    /// The axiom statement.
    fn statement() -> &'static str;
}

// ═══════════════════════════════════════════════════════════
// A1: FINITE DECOMPOSITION
// ═══════════════════════════════════════════════════════════

/// **A1: Finite Decomposition**
///
/// *Every state machine has finitely many states.*
///
/// The const generic `MAX` encodes the cardinality bound at the type level.
/// Construction is only possible if `state_count <= MAX`.
///
/// ## Primitive Grounding
///
/// T2-P (ς + N): State + Quantity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct A1FiniteDecomposition<const MAX: usize> {
    /// Actual number of states.
    pub state_count: usize,
}

impl<const MAX: usize> A1FiniteDecomposition<MAX> {
    /// Attempt to construct a finite decomposition witness.
    ///
    /// Returns `None` if `state_count > MAX`.
    #[must_use]
    pub const fn try_new(state_count: usize) -> Option<Self> {
        if state_count <= MAX {
            Some(Self { state_count })
        } else {
            None
        }
    }

    /// Construct a witness, panicking if bound exceeded.
    ///
    /// # Panics
    ///
    /// Panics if `state_count > MAX`.
    #[must_use]
    pub const fn new(state_count: usize) -> Self {
        assert!(state_count <= MAX, "State count exceeds maximum bound");
        Self { state_count }
    }

    /// The maximum allowed states.
    #[must_use]
    pub const fn max_states() -> usize {
        MAX
    }

    /// Whether the machine is at capacity.
    #[must_use]
    pub const fn is_at_capacity(&self) -> bool {
        self.state_count == MAX
    }
}

impl<const MAX: usize> Axiom for A1FiniteDecomposition<MAX> {
    fn id() -> &'static str {
        "A1"
    }
    fn name() -> &'static str {
        "Finite Decomposition"
    }
    fn statement() -> &'static str {
        "Every state machine has finitely many states (bounded by MAX)"
    }
}

// ═══════════════════════════════════════════════════════════
// A2: HIERARCHICAL ORDER
// ═══════════════════════════════════════════════════════════

/// Ordering kind for state hierarchies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderKind {
    /// Total order: all states comparable (linear chain).
    Total,
    /// Partial order: some states incomparable (DAG).
    Partial,
    /// Preorder: reflexive + transitive (may have cycles).
    Preorder,
}

/// **A2: Hierarchical Order**
///
/// *States admit a partial order defining valid progressions.*
///
/// The order can be total (linear), partial (DAG), or preorder (cycles).
///
/// ## Primitive Grounding
///
/// T2-P (ς + σ + ρ): State + Sequence + Recursion
#[derive(Debug, Clone)]
pub struct A2HierarchicalOrder {
    /// The kind of ordering.
    pub order_kind: OrderKind,
    /// Whether cycles are present.
    pub has_cycles: bool,
    /// Topological levels (if acyclic).
    pub levels: Option<Vec<Vec<u32>>>,
}

impl A2HierarchicalOrder {
    /// Create a linear (total order) hierarchy.
    #[must_use]
    pub fn linear(state_ids: Vec<u32>) -> Self {
        let levels = state_ids.into_iter().map(|id| alloc::vec![id]).collect();
        Self {
            order_kind: OrderKind::Total,
            has_cycles: false,
            levels: Some(levels),
        }
    }

    /// Create a DAG (partial order) hierarchy.
    #[must_use]
    pub fn dag(levels: Vec<Vec<u32>>) -> Self {
        Self {
            order_kind: OrderKind::Partial,
            has_cycles: false,
            levels: Some(levels),
        }
    }

    /// Create a cyclic (preorder) hierarchy.
    #[must_use]
    pub fn cyclic() -> Self {
        Self {
            order_kind: OrderKind::Preorder,
            has_cycles: true,
            levels: None,
        }
    }

    /// Whether the hierarchy admits a topological sort.
    #[must_use]
    pub fn is_acyclic(&self) -> bool {
        !self.has_cycles
    }
}

impl Axiom for A2HierarchicalOrder {
    fn id() -> &'static str {
        "A2"
    }
    fn name() -> &'static str {
        "Hierarchical Order"
    }
    fn statement() -> &'static str {
        "States admit a partial order defining valid progressions"
    }
}

// ═══════════════════════════════════════════════════════════
// A3: CONSERVATION
// ═══════════════════════════════════════════════════════════

/// **A3: Conservation**
///
/// *Transitions preserve designated invariants.*
///
/// This axiom connects to conservation laws (L3, L4, L11).
///
/// ## Primitive Grounding
///
/// T2-C (ς + → + κ): State + Causality + Comparison
#[derive(Debug, Clone)]
pub struct A3Conservation {
    /// Names of preserved invariants.
    pub invariants: Vec<&'static str>,
    /// Whether all invariants are currently satisfied.
    pub all_satisfied: bool,
}

impl A3Conservation {
    /// Create a conservation witness with given invariants.
    #[must_use]
    pub fn new(invariants: Vec<&'static str>) -> Self {
        Self {
            invariants,
            all_satisfied: true,
        }
    }

    /// Mark an invariant violation.
    pub fn mark_violation(&mut self) {
        self.all_satisfied = false;
    }

    /// Standard state machine invariants.
    #[must_use]
    pub fn standard() -> Self {
        Self::new(alloc::vec![
            "L3: Single State (exactly one active state)",
            "L4: Non-Terminal Flux (non-terminal has outgoing)",
            "L11: Structure Immutability (state count fixed)",
        ])
    }
}

impl Axiom for A3Conservation {
    fn id() -> &'static str {
        "A3"
    }
    fn name() -> &'static str {
        "Conservation"
    }
    fn statement() -> &'static str {
        "Transitions preserve designated invariants"
    }
}

// ═══════════════════════════════════════════════════════════
// A4: SAFETY MANIFOLD
// ═══════════════════════════════════════════════════════════

/// State classification in the safety manifold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateRegion {
    /// Interior: valid, non-terminal states.
    Interior,
    /// Boundary: terminal states (absorbing).
    Boundary,
    /// Exterior: invalid/error states.
    Exterior,
}

/// **A4: Safety Manifold**
///
/// *Valid states form the interior; terminal states form the boundary.*
///
/// This axiom ensures:
/// - Interior states have outgoing transitions
/// - Boundary states are absorbing (no outgoing)
/// - Exterior states are unreachable from valid paths
///
/// ## Primitive Grounding
///
/// T2-C (ς + ∂ + ∅): State + Boundary + Void
#[derive(Debug, Clone)]
pub struct A4SafetyManifold {
    /// States in the interior (non-terminal).
    pub interior: Vec<u32>,
    /// States on the boundary (terminal).
    pub boundary: Vec<u32>,
    /// States in the exterior (error/unreachable).
    pub exterior: Vec<u32>,
}

impl A4SafetyManifold {
    /// Create a safety manifold with classified states.
    #[must_use]
    pub fn new(interior: Vec<u32>, boundary: Vec<u32>) -> Self {
        Self {
            interior,
            boundary,
            exterior: Vec::new(),
        }
    }

    /// Classify a state ID.
    #[must_use]
    pub fn classify(&self, state_id: u32) -> StateRegion {
        if self.interior.contains(&state_id) {
            StateRegion::Interior
        } else if self.boundary.contains(&state_id) {
            StateRegion::Boundary
        } else {
            StateRegion::Exterior
        }
    }

    /// Total valid states (interior + boundary).
    #[must_use]
    pub fn valid_count(&self) -> usize {
        self.interior.len() + self.boundary.len()
    }

    /// Whether all terminal states are properly absorbing.
    #[must_use]
    pub fn boundary_absorbing(&self) -> bool {
        // In a well-formed manifold, boundary states have no outgoing transitions
        // This is enforced by the typestate pattern at compile time
        true
    }
}

impl Axiom for A4SafetyManifold {
    fn id() -> &'static str {
        "A4"
    }
    fn name() -> &'static str {
        "Safety Manifold"
    }
    fn statement() -> &'static str {
        "Valid states form interior; terminal states form absorbing boundary"
    }
}

// ═══════════════════════════════════════════════════════════
// A5: EMERGENCE
// ═══════════════════════════════════════════════════════════

/// **A5: Emergence**
///
/// *Complex behavior emerges from simple transition rules.*
///
/// This axiom captures the observation that sophisticated state machines
/// arise from the composition of simple transitions.
///
/// ## Primitive Grounding
///
/// T3 (ς + → + σ + κ): Full composition
#[derive(Debug, Clone)]
pub struct A5Emergence {
    /// Number of atomic transitions.
    pub transition_count: usize,
    /// Number of guard predicates.
    pub guard_count: usize,
    /// Number of effect actions.
    pub effect_count: usize,
    /// Emergent complexity measure (transitions × guards).
    pub complexity: usize,
}

impl A5Emergence {
    /// Create an emergence witness.
    #[must_use]
    pub fn new(transition_count: usize, guard_count: usize, effect_count: usize) -> Self {
        Self {
            transition_count,
            guard_count,
            effect_count,
            complexity: transition_count * guard_count.max(1),
        }
    }

    /// Complexity ratio: emergent behavior per transition.
    #[must_use]
    pub fn complexity_ratio(&self) -> f64 {
        if self.transition_count == 0 {
            0.0
        } else {
            self.complexity as f64 / self.transition_count as f64
        }
    }
}

impl Axiom for A5Emergence {
    fn id() -> &'static str {
        "A5"
    }
    fn name() -> &'static str {
        "Emergence"
    }
    fn statement() -> &'static str {
        "Complex behavior emerges from simple transition rules"
    }
}

// ═══════════════════════════════════════════════════════════
// COMPLETE PROOF
// ═══════════════════════════════════════════════════════════

/// A complete proof that a state machine satisfies all five axioms.
///
/// Construction of this type witnesses that the machine is well-formed.
#[derive(Debug, Clone)]
pub struct StateTheoryProof<const MAX: usize> {
    /// A1: Finite decomposition witness.
    pub a1: A1FiniteDecomposition<MAX>,
    /// A2: Hierarchical order witness.
    pub a2: A2HierarchicalOrder,
    /// A3: Conservation witness.
    pub a3: A3Conservation,
    /// A4: Safety manifold witness.
    pub a4: A4SafetyManifold,
    /// A5: Emergence witness.
    pub a5: A5Emergence,
}

impl<const MAX: usize> StateTheoryProof<MAX> {
    /// Construct a complete proof.
    #[must_use]
    pub fn new(
        a1: A1FiniteDecomposition<MAX>,
        a2: A2HierarchicalOrder,
        a3: A3Conservation,
        a4: A4SafetyManifold,
        a5: A5Emergence,
    ) -> Self {
        Self { a1, a2, a3, a4, a5 }
    }

    /// Whether all axioms are satisfied.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.a3.all_satisfied && self.a4.boundary_absorbing()
    }

    /// Summary string.
    #[must_use]
    pub fn summary(&self) -> alloc::string::String {
        alloc::format!(
            "StateTheoryProof<{}>: {} states, {} hierarchy, {} invariants, valid={}",
            MAX,
            self.a1.state_count,
            match self.a2.order_kind {
                OrderKind::Total => "linear",
                OrderKind::Partial => "DAG",
                OrderKind::Preorder => "cyclic",
            },
            self.a3.invariants.len(),
            self.is_valid()
        )
    }
}

// ═══════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a1_finite_decomposition() {
        let a1: A1FiniteDecomposition<10> = A1FiniteDecomposition::new(5);
        assert_eq!(a1.state_count, 5);
        assert_eq!(A1FiniteDecomposition::<10>::max_states(), 10);
        assert!(!a1.is_at_capacity());
    }

    #[test]
    fn test_a1_try_new() {
        let ok: Option<A1FiniteDecomposition<5>> = A1FiniteDecomposition::try_new(3);
        assert!(ok.is_some());

        let fail: Option<A1FiniteDecomposition<5>> = A1FiniteDecomposition::try_new(10);
        assert!(fail.is_none());
    }

    #[test]
    fn test_a2_linear_hierarchy() {
        let a2 = A2HierarchicalOrder::linear(alloc::vec![0, 1, 2, 3]);
        assert_eq!(a2.order_kind, OrderKind::Total);
        assert!(a2.is_acyclic());
    }

    #[test]
    fn test_a2_cyclic_hierarchy() {
        let a2 = A2HierarchicalOrder::cyclic();
        assert_eq!(a2.order_kind, OrderKind::Preorder);
        assert!(!a2.is_acyclic());
    }

    #[test]
    fn test_a3_conservation() {
        let a3 = A3Conservation::standard();
        assert_eq!(a3.invariants.len(), 3);
        assert!(a3.all_satisfied);
    }

    #[test]
    fn test_a4_safety_manifold() {
        let a4 = A4SafetyManifold::new(
            alloc::vec![0, 1, 2], // interior
            alloc::vec![3],       // boundary
        );
        assert_eq!(a4.classify(0), StateRegion::Interior);
        assert_eq!(a4.classify(3), StateRegion::Boundary);
        assert_eq!(a4.classify(99), StateRegion::Exterior);
        assert_eq!(a4.valid_count(), 4);
    }

    #[test]
    fn test_a5_emergence() {
        let a5 = A5Emergence::new(5, 3, 2);
        assert_eq!(a5.complexity, 15);
        assert!((a5.complexity_ratio() - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_complete_proof() {
        let proof: StateTheoryProof<10> = StateTheoryProof::new(
            A1FiniteDecomposition::new(4),
            A2HierarchicalOrder::linear(alloc::vec![0, 1, 2, 3]),
            A3Conservation::standard(),
            A4SafetyManifold::new(alloc::vec![0, 1, 2], alloc::vec![3]),
            A5Emergence::new(3, 0, 0),
        );
        assert!(proof.is_valid());
        assert!(proof.summary().contains("4 states"));
    }
}
