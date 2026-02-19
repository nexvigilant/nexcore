// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Kripke Structure
//!
//! A Kripke structure M = (S, S0, R, L) where:
//! - S: finite set of states
//! - S0 ⊆ S: initial states
//! - R ⊆ S × S: transition relation (must be total)
//! - L: S → 2^AP: labeling function (which propositions hold in each state)
//!
//! ## Primitive Grounding
//!
//! `KripkeStructure` is T3 (ρ-dominant):
//! - ρ Recursion: fixpoint iteration over state space
//! - ∂ Boundary: initial/terminal state delineation
//! - → Causality: transition relation
//! - κ Comparison: proposition satisfaction checking
//! - ∃ Existence: counterexample witness search

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};

/// A state identifier in the Kripke structure.
pub type StateId = usize;

/// A proposition identifier.
pub type PropId = u32;

/// A Kripke structure for model checking.
///
/// States are numbered 0..n-1. Propositions are identified by `PropId`.
/// The transition relation must be total: every state has at least one successor.
///
/// ## Tier: T3 (ρ + ∂ + → + κ + ∃)
#[derive(Debug, Clone)]
pub struct KripkeStructure {
    /// Number of states.
    pub state_count: usize,
    /// Initial states (S0 ⊆ S).
    pub initial_states: BTreeSet<StateId>,
    /// Transition relation: state → set of successor states.
    /// Must be total (every state has at least one successor).
    successors: Vec<BTreeSet<StateId>>,
    /// Predecessor relation (computed from successors for efficient backward search).
    predecessors: Vec<BTreeSet<StateId>>,
    /// Labeling function: state → set of propositions that hold.
    labels: Vec<BTreeSet<PropId>>,
    /// Proposition names (for diagnostics).
    prop_names: HashMap<PropId, String>,
}

/// Error type for Kripke structure construction.
#[derive(Debug, Clone)]
pub enum KripkeError {
    /// A state has no successors (relation not total).
    NotTotal {
        /// The state with no successors.
        state: StateId,
    },
    /// State index out of bounds.
    StateOutOfBounds {
        /// The invalid state index.
        state: StateId,
        /// Maximum valid index.
        max: StateId,
    },
    /// No initial states defined.
    NoInitialStates,
    /// Empty structure (zero states).
    Empty,
}

impl core::fmt::Display for KripkeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NotTotal { state } => {
                write!(f, "state {state} has no successors (relation not total)")
            }
            Self::StateOutOfBounds { state, max } => {
                write!(f, "state {state} out of bounds (max {max})")
            }
            Self::NoInitialStates => write!(f, "no initial states defined"),
            Self::Empty => write!(f, "empty Kripke structure (zero states)"),
        }
    }
}

impl KripkeStructure {
    /// Build a Kripke structure using the builder.
    #[must_use]
    pub fn builder(state_count: usize) -> KripkeBuilder {
        KripkeBuilder::new(state_count)
    }

    /// Number of states.
    #[must_use]
    pub fn state_count(&self) -> usize {
        self.state_count
    }

    /// Get successors of a state.
    #[must_use]
    pub fn successors(&self, state: StateId) -> &BTreeSet<StateId> {
        &self.successors[state]
    }

    /// Get predecessors of a state.
    #[must_use]
    pub fn predecessors(&self, state: StateId) -> &BTreeSet<StateId> {
        &self.predecessors[state]
    }

    /// Check if a proposition holds in a state.
    #[must_use]
    pub fn has_prop(&self, state: StateId, prop: PropId) -> bool {
        self.labels[state].contains(&prop)
    }

    /// Get all propositions holding in a state.
    #[must_use]
    pub fn props_at(&self, state: StateId) -> &BTreeSet<PropId> {
        &self.labels[state]
    }

    /// Get all states where a proposition holds.
    #[must_use]
    pub fn states_with_prop(&self, prop: PropId) -> BTreeSet<StateId> {
        (0..self.state_count)
            .filter(|&s| self.labels[s].contains(&prop))
            .collect()
    }

    /// Get the set of all states.
    #[must_use]
    pub fn all_states(&self) -> BTreeSet<StateId> {
        (0..self.state_count).collect()
    }

    /// Compute reachable states from initial states via BFS.
    #[must_use]
    pub fn reachable_states(&self) -> BTreeSet<StateId> {
        let mut visited = BTreeSet::new();
        let mut queue: VecDeque<StateId> = self.initial_states.iter().copied().collect();

        while let Some(s) = queue.pop_front() {
            if visited.insert(s) {
                for &succ in &self.successors[s] {
                    if !visited.contains(&succ) {
                        queue.push_back(succ);
                    }
                }
            }
        }

        visited
    }

    /// Compute predecessor image: pre(Y) = { s | ∃s' ∈ Y. s → s' }
    #[must_use]
    pub fn pre_exists(&self, target: &BTreeSet<StateId>) -> BTreeSet<StateId> {
        let mut result = BTreeSet::new();
        for &t in target {
            for &pred in &self.predecessors[t] {
                result.insert(pred);
            }
        }
        result
    }

    /// Compute universal predecessor image: pre_forall(Y) = { s | ∀s'. s → s' ⟹ s' ∈ Y }
    #[must_use]
    pub fn pre_forall(&self, target: &BTreeSet<StateId>) -> BTreeSet<StateId> {
        (0..self.state_count)
            .filter(|&s| self.successors[s].iter().all(|succ| target.contains(succ)))
            .collect()
    }

    /// Get proposition name (for diagnostics).
    #[must_use]
    pub fn prop_name(&self, prop: PropId) -> Option<&str> {
        self.prop_names.get(&prop).map(|s| s.as_str())
    }
}

/// Builder for constructing Kripke structures.
#[derive(Debug)]
pub struct KripkeBuilder {
    state_count: usize,
    initial_states: BTreeSet<StateId>,
    transitions: Vec<BTreeSet<StateId>>,
    labels: Vec<BTreeSet<PropId>>,
    prop_names: HashMap<PropId, String>,
}

impl KripkeBuilder {
    /// Create a new builder for n states.
    #[must_use]
    pub fn new(state_count: usize) -> Self {
        Self {
            state_count,
            initial_states: BTreeSet::new(),
            transitions: vec![BTreeSet::new(); state_count],
            labels: vec![BTreeSet::new(); state_count],
            prop_names: HashMap::new(),
        }
    }

    /// Mark a state as initial.
    pub fn initial(&mut self, state: StateId) -> &mut Self {
        self.initial_states.insert(state);
        self
    }

    /// Add a transition from `from` to `to`.
    pub fn transition(&mut self, from: StateId, to: StateId) -> &mut Self {
        if from < self.state_count && to < self.state_count {
            self.transitions[from].insert(to);
        }
        self
    }

    /// Label a state with a proposition.
    pub fn label(&mut self, state: StateId, prop: PropId) -> &mut Self {
        if state < self.state_count {
            self.labels[state].insert(prop);
        }
        self
    }

    /// Register a proposition name.
    pub fn prop_name(&mut self, prop: PropId, name: &str) -> &mut Self {
        self.prop_names.insert(prop, name.to_string());
        self
    }

    /// Add a self-loop to a state (ensures totality for terminal states).
    pub fn self_loop(&mut self, state: StateId) -> &mut Self {
        self.transition(state, state)
    }

    /// Build the Kripke structure, validating totality.
    pub fn build(self) -> Result<KripkeStructure, KripkeError> {
        if self.state_count == 0 {
            return Err(KripkeError::Empty);
        }
        if self.initial_states.is_empty() {
            return Err(KripkeError::NoInitialStates);
        }

        // Check totality
        for (i, succs) in self.transitions.iter().enumerate() {
            if succs.is_empty() {
                return Err(KripkeError::NotTotal { state: i });
            }
        }

        // Validate state references
        for s in &self.initial_states {
            if *s >= self.state_count {
                return Err(KripkeError::StateOutOfBounds {
                    state: *s,
                    max: self.state_count - 1,
                });
            }
        }

        // Compute predecessors
        let mut predecessors = vec![BTreeSet::new(); self.state_count];
        for (s, succs) in self.transitions.iter().enumerate() {
            for &t in succs {
                predecessors[t].insert(s);
            }
        }

        Ok(KripkeStructure {
            state_count: self.state_count,
            initial_states: self.initial_states,
            successors: self.transitions,
            predecessors,
            labels: self.labels,
            prop_names: self.prop_names,
        })
    }
}

/// Convert a `StateMachineSpec` into a `KripkeBuilder` with terminal/initial propositions.
///
/// Since `StateMachineSpec` doesn't store transitions, this creates the structure
/// with states and labels only. Transitions must be added manually.
impl From<&nexcore_state_theory::category::StateMachineSpec> for KripkeBuilder {
    fn from(spec: &nexcore_state_theory::category::StateMachineSpec) -> Self {
        let mut builder = KripkeBuilder::new(spec.state_count);
        builder.initial(spec.initial);

        // Label initial state
        builder.label(spec.initial, 0); // prop 0 = initial
        builder.prop_name(0, "initial");

        // Label terminal states
        builder.prop_name(1, "terminal");
        for &t in &spec.terminals {
            builder.label(t, 1);
        }

        builder
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn traffic_light() -> KripkeStructure {
        // States: 0=Red, 1=Green, 2=Yellow
        // Props: 10=red, 11=green, 12=yellow
        let mut b = KripkeStructure::builder(3);
        b.initial(0);
        b.transition(0, 1); // Red → Green
        b.transition(1, 2); // Green → Yellow
        b.transition(2, 0); // Yellow → Red
        b.label(0, 10).prop_name(10, "red");
        b.label(1, 11).prop_name(11, "green");
        b.label(2, 12).prop_name(12, "yellow");
        b.build().ok().unwrap_or_else(|| unreachable!())
    }

    #[test]
    fn test_builder_basic() {
        let k = traffic_light();
        assert_eq!(k.state_count(), 3);
        assert!(k.initial_states.contains(&0));
        assert!(k.has_prop(0, 10));
        assert!(k.has_prop(1, 11));
    }

    #[test]
    fn test_successors_predecessors() {
        let k = traffic_light();
        assert!(k.successors(0).contains(&1));
        assert!(k.predecessors(1).contains(&0));
        assert!(k.predecessors(0).contains(&2));
    }

    #[test]
    fn test_reachable() {
        let k = traffic_light();
        let reach = k.reachable_states();
        assert_eq!(reach.len(), 3);
    }

    #[test]
    fn test_pre_exists() {
        let k = traffic_light();
        let greens = BTreeSet::from([1]);
        let pre = k.pre_exists(&greens);
        assert!(pre.contains(&0)); // Red → Green
        assert!(!pre.contains(&1));
    }

    #[test]
    fn test_pre_forall() {
        let k = traffic_light();
        let all_but_red = BTreeSet::from([1, 2]);
        let pre = k.pre_forall(&all_but_red);
        // State 0 (Red) → {1 (Green)} ⊆ {1,2} → yes
        assert!(pre.contains(&0));
        // State 1 (Green) → {2 (Yellow)} ⊆ {1,2} → yes
        assert!(pre.contains(&1));
        // State 2 (Yellow) → {0 (Red)} ⊄ {1,2} → no
        assert!(!pre.contains(&2));
    }

    #[test]
    fn test_not_total_fails() {
        let mut b = KripkeStructure::builder(2);
        b.initial(0);
        b.transition(0, 1);
        // State 1 has no successors
        let result = b.build();
        assert!(result.is_err());
    }

    #[test]
    fn test_self_loop_totality() {
        let mut b = KripkeStructure::builder(2);
        b.initial(0);
        b.transition(0, 1);
        b.self_loop(1); // Terminal state gets self-loop
        let result = b.build();
        assert!(result.is_ok());
    }

    #[test]
    fn test_states_with_prop() {
        let k = traffic_light();
        let reds = k.states_with_prop(10);
        assert_eq!(reds, BTreeSet::from([0]));
    }

    #[test]
    fn test_from_spec() {
        let spec = nexcore_state_theory::category::StateMachineSpec::new(
            1,
            vec!["A".into(), "B".into(), "C".into()],
            0,
            vec![2],
        );
        let builder = KripkeBuilder::from(&spec);
        assert!(builder.initial_states.contains(&0));
        assert!(builder.labels[0].contains(&0)); // initial prop
        assert!(builder.labels[2].contains(&1)); // terminal prop
    }
}
