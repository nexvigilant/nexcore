// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Equivalence Relations on State Machines
//!
//! Different notions of when two state machines are "the same":
//!
//! | Relation | Preserves | Strength |
//! |----------|-----------|----------|
//! | Bisimulation | Branching structure | Strongest |
//! | Trace Equivalence | Linear traces | Medium |
//! | Language Equivalence | Accepted language | Weakest |
//!
//! ## Bisimulation Hierarchy
//!
//! ```text
//! Bisimulation ⊂ Simulation ⊂ Trace Inclusion
//!      ↓              ↓              ↓
//! Both directions  One direction  Set containment
//! ```
//!
//! ## Refinement
//!
//! Refinement is the dual of simulation: M₂ refines M₁ if M₁ simulates M₂.
//! This captures the notion that M₂ is a "more concrete" implementation.

use alloc::collections::BTreeSet;
use alloc::vec::Vec;

// ═══════════════════════════════════════════════════════════
// RELATION TYPES
// ═══════════════════════════════════════════════════════════

/// A binary relation between states of two machines.
#[derive(Debug, Clone)]
pub struct StateRelation {
    /// Related pairs: (state_from_M1, state_from_M2).
    pub pairs: Vec<(u32, u32)>,
}

impl StateRelation {
    /// Create an empty relation.
    #[must_use]
    pub fn empty() -> Self {
        Self { pairs: Vec::new() }
    }

    /// Create a relation from pairs.
    #[must_use]
    pub fn from_pairs(pairs: Vec<(u32, u32)>) -> Self {
        Self { pairs }
    }

    /// Add a pair to the relation.
    pub fn add(&mut self, s1: u32, s2: u32) {
        if !self.contains(s1, s2) {
            self.pairs.push((s1, s2));
        }
    }

    /// Check if a pair is in the relation.
    #[must_use]
    pub fn contains(&self, s1: u32, s2: u32) -> bool {
        self.pairs.contains(&(s1, s2))
    }

    /// Get all states related to a given state.
    #[must_use]
    pub fn related_to(&self, s1: u32) -> Vec<u32> {
        self.pairs
            .iter()
            .filter_map(|&(a, b)| if a == s1 { Some(b) } else { None })
            .collect()
    }

    /// Inverse relation.
    #[must_use]
    pub fn inverse(&self) -> Self {
        Self {
            pairs: self.pairs.iter().map(|&(a, b)| (b, a)).collect(),
        }
    }

    /// Symmetric closure.
    #[must_use]
    pub fn symmetric_closure(&self) -> Self {
        let mut result = self.clone();
        for &(a, b) in &self.pairs {
            result.add(b, a);
        }
        result
    }

    /// Whether the relation is symmetric.
    #[must_use]
    pub fn is_symmetric(&self) -> bool {
        self.pairs.iter().all(|&(a, b)| self.contains(b, a))
    }
}

// ═══════════════════════════════════════════════════════════
// BISIMULATION
// ═══════════════════════════════════════════════════════════

/// Bisimulation between state machines.
///
/// R is a bisimulation if for all (s₁, s₂) ∈ R:
/// 1. If s₁ →ᵃ s₁', then ∃s₂' such that s₂ →ᵃ s₂' and (s₁', s₂') ∈ R
/// 2. If s₂ →ᵃ s₂', then ∃s₁' such that s₁ →ᵃ s₁' and (s₁', s₂') ∈ R
///
/// Two states are bisimilar (s₁ ~ s₂) if there exists a bisimulation R
/// containing (s₁, s₂).
#[derive(Debug, Clone)]
pub struct Bisimulation {
    /// The bisimulation relation.
    pub relation: StateRelation,
    /// Whether verified to be a bisimulation.
    pub verified: bool,
}

impl Bisimulation {
    /// Create a potential bisimulation (not yet verified).
    #[must_use]
    pub fn potential(relation: StateRelation) -> Self {
        Self {
            relation,
            verified: false,
        }
    }

    /// Create the identity bisimulation on a single machine.
    #[must_use]
    pub fn identity(state_count: u32) -> Self {
        let pairs: Vec<(u32, u32)> = (0..state_count).map(|s| (s, s)).collect();
        Self {
            relation: StateRelation::from_pairs(pairs),
            verified: true,
        }
    }

    /// Check if two states are bisimilar in this relation.
    #[must_use]
    pub fn are_bisimilar(&self, s1: u32, s2: u32) -> bool {
        self.verified && self.relation.contains(s1, s2)
    }

    /// Get all states bisimilar to a given state.
    #[must_use]
    pub fn bisimilar_states(&self, s: u32) -> Vec<u32> {
        if self.verified {
            self.relation.related_to(s)
        } else {
            Vec::new()
        }
    }
}

/// Bisimulation quotient: partition of states into equivalence classes.
#[derive(Debug, Clone)]
pub struct BisimulationQuotient {
    /// Equivalence classes (each class is a set of bisimilar states).
    pub classes: Vec<BTreeSet<u32>>,
}

impl BisimulationQuotient {
    /// Create quotient from bisimulation relation.
    #[must_use]
    pub fn from_bisimulation(bisim: &Bisimulation, state_count: u32) -> Self {
        let mut classes: Vec<BTreeSet<u32>> = Vec::new();
        let mut assigned: BTreeSet<u32> = BTreeSet::new();

        for s in 0..state_count {
            if assigned.contains(&s) {
                continue;
            }

            let mut class = BTreeSet::new();
            class.insert(s);

            for t in (s + 1)..state_count {
                if bisim.relation.contains(s, t) {
                    class.insert(t);
                    assigned.insert(t);
                }
            }

            assigned.insert(s);
            classes.push(class);
        }

        Self { classes }
    }

    /// Number of equivalence classes.
    #[must_use]
    pub fn class_count(&self) -> usize {
        self.classes.len()
    }

    /// Find the class containing a state.
    #[must_use]
    pub fn class_of(&self, state: u32) -> Option<usize> {
        self.classes.iter().position(|c| c.contains(&state))
    }

    /// Get the representative of a state's class (smallest element).
    #[must_use]
    pub fn representative(&self, state: u32) -> Option<u32> {
        self.class_of(state)
            .and_then(|i| self.classes.get(i))
            .and_then(|c| c.first().copied())
    }
}

// ═══════════════════════════════════════════════════════════
// SIMULATION
// ═══════════════════════════════════════════════════════════

/// Simulation preorder.
///
/// M₁ ≤ M₂ (M₂ simulates M₁) if there exists a relation R such that:
/// - Initial states are related
/// - For all (s₁, s₂) ∈ R and s₁ →ᵃ s₁', there exists s₂ →ᵃ s₂' with (s₁', s₂') ∈ R
///
/// Simulation is one-directional (unlike bisimulation).
#[derive(Debug, Clone)]
pub struct SimulationPreorder {
    /// The simulation relation.
    pub relation: StateRelation,
    /// Source machine simulated by target.
    pub source_simulated_by_target: bool,
}

impl SimulationPreorder {
    /// Create a simulation where target simulates source.
    #[must_use]
    pub fn new(relation: StateRelation) -> Self {
        Self {
            relation,
            source_simulated_by_target: true,
        }
    }

    /// Whether s₁ is simulated by s₂.
    #[must_use]
    pub fn simulates(&self, s1: u32, s2: u32) -> bool {
        self.source_simulated_by_target && self.relation.contains(s1, s2)
    }
}

// ═══════════════════════════════════════════════════════════
// TRACE EQUIVALENCE
// ═══════════════════════════════════════════════════════════

/// A trace is a sequence of actions/labels.
pub type ActionTrace = Vec<u32>;

/// Trace equivalence: machines have the same set of traces.
#[derive(Debug, Clone)]
pub struct TraceEquivalence {
    /// Traces from machine 1.
    pub traces_m1: BTreeSet<ActionTrace>,
    /// Traces from machine 2.
    pub traces_m2: BTreeSet<ActionTrace>,
}

impl TraceEquivalence {
    /// Create trace equivalence comparison.
    #[must_use]
    pub fn new(traces_m1: BTreeSet<ActionTrace>, traces_m2: BTreeSet<ActionTrace>) -> Self {
        Self {
            traces_m1,
            traces_m2,
        }
    }

    /// Whether the machines are trace equivalent.
    #[must_use]
    pub fn are_equivalent(&self) -> bool {
        self.traces_m1 == self.traces_m2
    }

    /// Traces in M1 but not M2.
    #[must_use]
    pub fn only_in_m1(&self) -> BTreeSet<ActionTrace> {
        self.traces_m1
            .difference(&self.traces_m2)
            .cloned()
            .collect()
    }

    /// Traces in M2 but not M1.
    #[must_use]
    pub fn only_in_m2(&self) -> BTreeSet<ActionTrace> {
        self.traces_m2
            .difference(&self.traces_m1)
            .cloned()
            .collect()
    }

    /// Whether M1 traces are subset of M2 traces.
    #[must_use]
    pub fn m1_subset_m2(&self) -> bool {
        self.traces_m1.is_subset(&self.traces_m2)
    }
}

// ═══════════════════════════════════════════════════════════
// REFINEMENT
// ═══════════════════════════════════════════════════════════

/// Refinement types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefinementKind {
    /// State refinement: mapping abstract states to concrete states.
    State,
    /// Data refinement: mapping abstract data to concrete data.
    Data,
    /// Action refinement: expanding abstract actions into concrete action sequences.
    Action,
}

/// Refinement relation between specifications.
///
/// M_concrete refines M_abstract if M_abstract simulates M_concrete.
/// This means M_concrete is a valid implementation of M_abstract.
#[derive(Debug, Clone)]
pub struct Refinement {
    /// Kind of refinement.
    pub kind: RefinementKind,
    /// The abstraction function (concrete → abstract).
    pub abstraction: StateRelation,
    /// Whether the refinement has been verified.
    pub verified: bool,
}

impl Refinement {
    /// Create a state refinement.
    #[must_use]
    pub fn state_refinement(abstraction: StateRelation) -> Self {
        Self {
            kind: RefinementKind::State,
            abstraction,
            verified: false,
        }
    }

    /// Create a data refinement.
    #[must_use]
    pub fn data_refinement(abstraction: StateRelation) -> Self {
        Self {
            kind: RefinementKind::Data,
            abstraction,
            verified: false,
        }
    }

    /// Whether a concrete state refines an abstract state.
    #[must_use]
    pub fn refines(&self, concrete: u32, abstract_state: u32) -> bool {
        self.verified && self.abstraction.contains(concrete, abstract_state)
    }
}

// ═══════════════════════════════════════════════════════════
// CONGRUENCE
// ═══════════════════════════════════════════════════════════

/// A congruence is an equivalence preserved by all operators.
///
/// If ≡ is a congruence and a ≡ b, then for any context C[_]:
/// C[a] ≡ C[b]
#[derive(Debug, Clone)]
pub struct Congruence {
    /// The underlying equivalence relation.
    pub equivalence: StateRelation,
    /// Operations that preserve this congruence.
    pub preserved_by: Vec<&'static str>,
}

impl Congruence {
    /// Create a congruence.
    #[must_use]
    pub fn new(equivalence: StateRelation, preserved_by: Vec<&'static str>) -> Self {
        Self {
            equivalence,
            preserved_by,
        }
    }

    /// Check if the congruence is preserved by an operation.
    #[must_use]
    pub fn preserved_by_op(&self, op: &str) -> bool {
        self.preserved_by.iter().any(|&o| o == op)
    }
}

// ═══════════════════════════════════════════════════════════
// EQUIVALENCE CHECKER
// ═══════════════════════════════════════════════════════════

/// Result of equivalence checking.
#[derive(Debug, Clone)]
pub enum EquivalenceResult {
    /// Machines are equivalent.
    Equivalent,
    /// Machines are not equivalent, with counterexample.
    NotEquivalent {
        /// Distinguishing trace.
        distinguishing_trace: ActionTrace,
        /// Which machine accepts the trace.
        accepted_by: u8, // 1 or 2
    },
    /// Could not determine (timeout or resource limit).
    Unknown,
}

impl EquivalenceResult {
    /// Whether equivalence was established.
    #[must_use]
    pub fn is_equivalent(&self) -> bool {
        matches!(self, Self::Equivalent)
    }

    /// Get the counterexample if not equivalent.
    #[must_use]
    pub fn counterexample(&self) -> Option<&ActionTrace> {
        match self {
            Self::NotEquivalent {
                distinguishing_trace,
                ..
            } => Some(distinguishing_trace),
            _ => None,
        }
    }
}

// ═══════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_relation() {
        let mut rel = StateRelation::empty();
        rel.add(0, 1);
        rel.add(1, 2);

        assert!(rel.contains(0, 1));
        assert!(!rel.contains(1, 0));

        let symmetric = rel.symmetric_closure();
        assert!(symmetric.contains(1, 0));
        assert!(symmetric.is_symmetric());
    }

    #[test]
    fn test_bisimulation_identity() {
        let bisim = Bisimulation::identity(5);
        assert!(bisim.verified);
        assert!(bisim.are_bisimilar(0, 0));
        assert!(bisim.are_bisimilar(3, 3));
        assert!(!bisim.are_bisimilar(0, 1));
    }

    #[test]
    fn test_bisimulation_quotient() {
        // Create a bisimulation where {0, 2} and {1, 3} are equivalent
        let pairs = vec![
            (0, 0),
            (0, 2),
            (2, 0),
            (2, 2),
            (1, 1),
            (1, 3),
            (3, 1),
            (3, 3),
        ];
        let bisim = Bisimulation {
            relation: StateRelation::from_pairs(pairs),
            verified: true,
        };

        let quotient = BisimulationQuotient::from_bisimulation(&bisim, 4);
        assert_eq!(quotient.class_count(), 2);
        assert_eq!(quotient.class_of(0), quotient.class_of(2));
        assert_eq!(quotient.class_of(1), quotient.class_of(3));
        assert_ne!(quotient.class_of(0), quotient.class_of(1));
    }

    #[test]
    fn test_trace_equivalence() {
        let traces1: BTreeSet<ActionTrace> = [vec![0, 1], vec![0, 2]].into_iter().collect();
        let traces2: BTreeSet<ActionTrace> = [vec![0, 1], vec![0, 2]].into_iter().collect();

        let te = TraceEquivalence::new(traces1, traces2);
        assert!(te.are_equivalent());
    }

    #[test]
    fn test_trace_not_equivalent() {
        let traces1: BTreeSet<ActionTrace> = [vec![0, 1], vec![0, 2]].into_iter().collect();
        let traces2: BTreeSet<ActionTrace> = [vec![0, 1]].into_iter().collect();

        let te = TraceEquivalence::new(traces1, traces2);
        assert!(!te.are_equivalent());
        assert!(!te.only_in_m1().is_empty());
    }

    #[test]
    fn test_refinement() {
        let abstraction = StateRelation::from_pairs(vec![(0, 0), (1, 0), (2, 1)]);
        let mut refinement = Refinement::state_refinement(abstraction);

        assert!(!refinement.refines(0, 0)); // Not verified yet

        refinement.verified = true;
        assert!(refinement.refines(0, 0));
        assert!(refinement.refines(1, 0)); // Both 0 and 1 map to abstract 0
    }

    #[test]
    fn test_equivalence_result() {
        let equiv = EquivalenceResult::Equivalent;
        assert!(equiv.is_equivalent());
        assert!(equiv.counterexample().is_none());

        let not_equiv = EquivalenceResult::NotEquivalent {
            distinguishing_trace: vec![0, 1, 2],
            accepted_by: 1,
        };
        assert!(!not_equiv.is_equivalent());
        assert_eq!(not_equiv.counterexample(), Some(&vec![0, 1, 2]));
    }
}
