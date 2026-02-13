// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Category Theory Foundations
//!
//! State machines form a category **StateMach** where:
//! - **Objects**: State machine types (finite sets of states + transitions)
//! - **Morphisms**: Structure-preserving maps (simulations)
//! - **Identity**: Identity simulation
//! - **Composition**: Sequential simulation composition
//!
//! ## Categorical Structure
//!
//! ```text
//! StateMach
//!     │
//!     ├── Products (parallel composition)
//!     │
//!     ├── Coproducts (choice composition)
//!     │
//!     ├── Exponentials (state machine transformers)
//!     │
//!     └── Initial/Terminal objects
//! ```
//!
//! ## Key Functors
//!
//! | Functor | Domain | Codomain | Meaning |
//! |---------|--------|----------|---------|
//! | Trace | StateMach | Set | Extracts trace language |
//! | Reach | StateMach | Set | Extracts reachable states |
//! | Obs | StateMach | StateMach | Observable behavior |
//!
//! ## Adjunctions
//!
//! The free-forgetful adjunction between labeled transition systems
//! and state machines provides a systematic way to construct minimal
//! realizations.

use alloc::string::String;
use alloc::vec::Vec;
use core::marker::PhantomData;

use crate::State;

// ═══════════════════════════════════════════════════════════
// CATEGORY CONCEPTS (Simplified for Rust's type system)
// ═══════════════════════════════════════════════════════════

/// Marker trait for objects in the category of state machines.
pub trait CategoryObject: Sized + 'static {}

/// Marker trait for morphisms (simulations) between state machines.
pub trait CategoryMorphism {
    /// Source object type.
    type Source: CategoryObject;
    /// Target object type.
    type Target: CategoryObject;

    /// Compose with another morphism (if compatible).
    fn compose<M: CategoryMorphism<Source = Self::Target>>(
        &self,
        other: &M,
    ) -> Option<ComposedMorphism>;
}

/// A composed morphism (result of sequential composition).
#[derive(Debug, Clone)]
pub struct ComposedMorphism {
    /// Source machine ID.
    pub source_id: u64,
    /// Target machine ID.
    pub target_id: u64,
    /// Intermediate machine ID.
    pub intermediate_id: u64,
}

/// Endofunctor on the state machine category.
pub trait Endofunctor {
    /// Apply the functor to a machine specification.
    fn apply(&self, spec: &StateMachineSpec) -> StateMachineSpec;

    /// Functor name.
    fn name(&self) -> &'static str;
}

// ═══════════════════════════════════════════════════════════
// STATE MACHINE CATEGORY
// ═══════════════════════════════════════════════════════════

/// The category of state machines.
///
/// Objects are state machine specifications.
/// Morphisms are simulations (structure-preserving maps).
#[derive(Debug, Clone, Copy)]
pub struct StateMachCategory;

/// A state machine specification (object in StateMach).
#[derive(Debug, Clone)]
pub struct StateMachineSpec {
    /// Unique identifier.
    pub id: u64,
    /// Number of states.
    pub state_count: usize,
    /// Number of transitions.
    pub transition_count: usize,
    /// State names.
    pub states: Vec<String>,
    /// Initial state index.
    pub initial: usize,
    /// Terminal state indices.
    pub terminals: Vec<usize>,
}

impl StateMachineSpec {
    /// Create a new specification.
    #[must_use]
    pub fn new(id: u64, states: Vec<String>, initial: usize, terminals: Vec<usize>) -> Self {
        let state_count = states.len();
        Self {
            id,
            state_count,
            transition_count: 0,
            states,
            initial,
            terminals,
        }
    }

    /// Add transitions.
    pub fn with_transitions(mut self, count: usize) -> Self {
        self.transition_count = count;
        self
    }

    /// Whether a state is terminal.
    #[must_use]
    pub fn is_terminal(&self, state: usize) -> bool {
        self.terminals.contains(&state)
    }
}

// ═══════════════════════════════════════════════════════════
// SIMULATION (MORPHISM)
// ═══════════════════════════════════════════════════════════

/// A simulation between state machines.
///
/// A simulation R ⊆ S₁ × S₂ is a relation such that:
/// 1. (s₁_init, s₂_init) ∈ R (initial states related)
/// 2. If (s₁, s₂) ∈ R and s₁ →ᵃ s₁', then ∃s₂' such that s₂ →ᵃ s₂' and (s₁', s₂') ∈ R
///
/// This is the fundamental morphism in the category of state machines.
#[derive(Debug, Clone)]
pub struct Simulation {
    /// Source machine ID.
    pub source_id: u64,
    /// Target machine ID.
    pub target_id: u64,
    /// State mapping: source state → target state.
    pub state_map: Vec<usize>,
    /// Whether this is a bisimulation (both directions).
    pub is_bisimulation: bool,
}

impl Simulation {
    /// Create a simulation with explicit state mapping.
    #[must_use]
    pub fn new(source_id: u64, target_id: u64, state_map: Vec<usize>) -> Self {
        Self {
            source_id,
            target_id,
            state_map,
            is_bisimulation: false,
        }
    }

    /// Create an identity simulation.
    #[must_use]
    pub fn identity(machine_id: u64, state_count: usize) -> Self {
        Self {
            source_id: machine_id,
            target_id: machine_id,
            state_map: (0..state_count).collect(),
            is_bisimulation: true,
        }
    }

    /// Mark as bisimulation.
    #[must_use]
    pub fn as_bisimulation(mut self) -> Self {
        self.is_bisimulation = true;
        self
    }

    /// Compose two simulations: (f ; g) where f: A → B and g: B → C.
    #[must_use]
    pub fn compose(&self, other: &Self) -> Option<Self> {
        if self.target_id != other.source_id {
            return None;
        }

        let composed_map: Vec<usize> = self
            .state_map
            .iter()
            .map(|&s| other.state_map.get(s).copied().unwrap_or(0))
            .collect();

        Some(Self {
            source_id: self.source_id,
            target_id: other.target_id,
            state_map: composed_map,
            is_bisimulation: self.is_bisimulation && other.is_bisimulation,
        })
    }
}

// ═══════════════════════════════════════════════════════════
// PRODUCT IN STATEMACH
// ═══════════════════════════════════════════════════════════

/// Product of two state machines in the category.
///
/// States are pairs (s₁, s₂), transitions are synchronized.
#[derive(Debug, Clone)]
pub struct ProductMachine {
    /// First component specification.
    pub left: StateMachineSpec,
    /// Second component specification.
    pub right: StateMachineSpec,
}

impl ProductMachine {
    /// Compute the product state count.
    #[must_use]
    pub fn state_count(&self) -> usize {
        self.left.state_count * self.right.state_count
    }

    /// Encode a pair (i, j) as a single state index.
    #[must_use]
    pub fn encode_state(&self, left_state: usize, right_state: usize) -> usize {
        left_state * self.right.state_count + right_state
    }

    /// Decode a state index back to (i, j).
    #[must_use]
    pub fn decode_state(&self, state: usize) -> (usize, usize) {
        let left = state / self.right.state_count;
        let right = state % self.right.state_count;
        (left, right)
    }

    /// Project to the left component.
    #[must_use]
    pub fn project_left(&self) -> Simulation {
        let state_map: Vec<usize> = (0..self.state_count())
            .map(|s| self.decode_state(s).0)
            .collect();

        Simulation {
            source_id: 0, // Product ID
            target_id: self.left.id,
            state_map,
            is_bisimulation: false,
        }
    }

    /// Project to the right component.
    #[must_use]
    pub fn project_right(&self) -> Simulation {
        let state_map: Vec<usize> = (0..self.state_count())
            .map(|s| self.decode_state(s).1)
            .collect();

        Simulation {
            source_id: 0,
            target_id: self.right.id,
            state_map,
            is_bisimulation: false,
        }
    }
}

// ═══════════════════════════════════════════════════════════
// COPRODUCT IN STATEMACH
// ═══════════════════════════════════════════════════════════

/// Coproduct (disjoint union) of state machines.
///
/// States are tagged union of states from both machines.
#[derive(Debug, Clone)]
pub struct CoproductMachine {
    /// First component specification.
    pub left: StateMachineSpec,
    /// Second component specification.
    pub right: StateMachineSpec,
}

impl CoproductMachine {
    /// Total state count.
    #[must_use]
    pub fn state_count(&self) -> usize {
        self.left.state_count + self.right.state_count
    }

    /// Injection from left component.
    #[must_use]
    pub fn inject_left(&self) -> Simulation {
        Simulation {
            source_id: self.left.id,
            target_id: 0, // Coproduct ID
            state_map: (0..self.left.state_count).collect(),
            is_bisimulation: false,
        }
    }

    /// Injection from right component.
    #[must_use]
    pub fn inject_right(&self) -> Simulation {
        let offset = self.left.state_count;
        Simulation {
            source_id: self.right.id,
            target_id: 0,
            state_map: (0..self.right.state_count).map(|s| s + offset).collect(),
            is_bisimulation: false,
        }
    }
}

// ═══════════════════════════════════════════════════════════
// INITIAL AND TERMINAL OBJECTS
// ═══════════════════════════════════════════════════════════

/// The initial object in StateMach: empty state machine.
///
/// There is a unique morphism from Initial to any machine.
#[derive(Debug, Clone, Copy, Default)]
pub struct InitialMachine;

impl InitialMachine {
    /// Unique morphism to any machine.
    #[must_use]
    pub fn unique_to(target: &StateMachineSpec) -> Simulation {
        Simulation {
            source_id: 0,
            target_id: target.id,
            state_map: Vec::new(),
            is_bisimulation: false,
        }
    }
}

/// The terminal object in StateMach: single-state machine with self-loop.
///
/// There is a unique morphism from any machine to Terminal.
#[derive(Debug, Clone, Copy, Default)]
pub struct TerminalMachine;

impl TerminalMachine {
    /// Unique morphism from any machine.
    #[must_use]
    pub fn unique_from(source: &StateMachineSpec) -> Simulation {
        Simulation {
            source_id: source.id,
            target_id: u64::MAX,                    // Terminal ID
            state_map: vec![0; source.state_count], // All states map to single state
            is_bisimulation: false,
        }
    }
}

// ═══════════════════════════════════════════════════════════
// ENDOFUNCTOR: POWERSET
// ═══════════════════════════════════════════════════════════

/// The powerset endofunctor on StateMach.
///
/// Maps a machine M to P(M) where states are subsets of M's states.
/// This is used in determinization of nondeterministic machines.
#[derive(Debug, Clone)]
pub struct PowersetFunctor<S: State> {
    _marker: PhantomData<S>,
}

impl<S: State> PowersetFunctor<S> {
    /// Number of states in the powerset machine.
    #[must_use]
    pub fn state_count(original_states: usize) -> usize {
        1 << original_states // 2^n
    }

    /// Encode a subset as a state index (bitmask).
    #[must_use]
    pub fn encode_subset(members: &[usize]) -> usize {
        members.iter().fold(0, |acc, &m| acc | (1 << m))
    }

    /// Decode a state index back to subset members.
    #[must_use]
    pub fn decode_subset(state: usize, max_states: usize) -> Vec<usize> {
        (0..max_states)
            .filter(|&i| (state & (1 << i)) != 0)
            .collect()
    }
}

// ═══════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // Test state type implementing State trait
    struct TestState;
    impl crate::State for TestState {
        fn name() -> &'static str {
            "test"
        }
        fn is_terminal() -> bool {
            false
        }
    }

    #[test]
    fn test_state_machine_spec() {
        let spec = StateMachineSpec::new(
            1,
            alloc::vec!["A".into(), "B".into(), "C".into()],
            0,
            alloc::vec![2],
        )
        .with_transitions(3);

        assert_eq!(spec.state_count, 3);
        assert_eq!(spec.transition_count, 3);
        assert!(!spec.is_terminal(0));
        assert!(spec.is_terminal(2));
    }

    #[test]
    fn test_simulation_identity() {
        let sim = Simulation::identity(1, 5);
        assert_eq!(sim.state_map, vec![0, 1, 2, 3, 4]);
        assert!(sim.is_bisimulation);
    }

    #[test]
    fn test_simulation_composition() {
        let f = Simulation::new(1, 2, vec![0, 1, 0]); // 3 states → 2 states
        let g = Simulation::new(2, 3, vec![1, 0]); // 2 states → 2 states

        let h = f.compose(&g);
        assert!(h.is_some());
        if let Some(h) = h {
            assert_eq!(h.source_id, 1);
            assert_eq!(h.target_id, 3);
            assert_eq!(h.state_map, vec![1, 0, 1]); // f;g composition
        }
    }

    #[test]
    fn test_product_machine() {
        let left = StateMachineSpec::new(1, alloc::vec!["A".into(), "B".into()], 0, alloc::vec![1]);
        let right = StateMachineSpec::new(
            2,
            alloc::vec!["X".into(), "Y".into(), "Z".into()],
            0,
            alloc::vec![2],
        );

        let product = ProductMachine { left, right };

        assert_eq!(product.state_count(), 6); // 2 × 3
        assert_eq!(product.encode_state(1, 2), 5); // (1, 2) → 1*3 + 2 = 5
        assert_eq!(product.decode_state(5), (1, 2));
    }

    #[test]
    fn test_coproduct_machine() {
        let left = StateMachineSpec::new(1, alloc::vec!["A".into(), "B".into()], 0, alloc::vec![1]);
        let right =
            StateMachineSpec::new(2, alloc::vec!["X".into(), "Y".into()], 0, alloc::vec![1]);

        let coproduct = CoproductMachine { left, right };

        assert_eq!(coproduct.state_count(), 4); // 2 + 2

        let inj_left = coproduct.inject_left();
        assert_eq!(inj_left.state_map, vec![0, 1]);

        let inj_right = coproduct.inject_right();
        assert_eq!(inj_right.state_map, vec![2, 3]);
    }

    #[test]
    fn test_powerset_functor() {
        assert_eq!(PowersetFunctor::<TestState>::state_count(3), 8); // 2³

        let subset = PowersetFunctor::<TestState>::encode_subset(&[0, 2]);
        assert_eq!(subset, 0b101); // bits 0 and 2

        let members = PowersetFunctor::<TestState>::decode_subset(0b101, 3);
        assert_eq!(members, vec![0, 2]);
    }

    #[test]
    fn test_terminal_machine() {
        let spec = StateMachineSpec::new(
            1,
            alloc::vec!["A".into(), "B".into(), "C".into()],
            0,
            alloc::vec![2],
        );
        let to_terminal = TerminalMachine::unique_from(&spec);

        assert_eq!(to_terminal.state_map, vec![0, 0, 0]); // All collapse to 0
    }
}
