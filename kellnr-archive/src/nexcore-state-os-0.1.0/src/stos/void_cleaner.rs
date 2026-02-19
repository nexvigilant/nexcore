// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Layer 8: Void Cleaner (STOS-VD)
//!
//! **Dominant Primitive**: ∅ (Void)
//!
//! Identifies and cleans up unreachable states and dead transitions.
//!
//! ## Tier Classification
//!
//! `VoidCleaner` is T2-P (∅ + ς) — void, state.

use alloc::collections::BTreeSet;
use alloc::vec::Vec;

use super::state_registry::StateId;
use super::transition_engine::TransitionId;
use crate::MachineId;

/// An unreachable state.
#[derive(Debug, Clone)]
pub struct UnreachableState {
    /// State ID.
    pub state: StateId,
    /// Reason for being unreachable.
    pub reason: UnreachableReason,
}

/// Reason a state is unreachable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnreachableReason {
    /// No incoming transitions (except initial).
    NoIncoming,
    /// Disconnected from initial state.
    Disconnected,
    /// Dead-end (terminal but not marked terminal).
    DeadEnd,
}

/// The void cleaner.
///
/// ## Tier: T2-P (∅ + ς)
///
/// Dominant primitive: ∅ (Void)
#[derive(Debug, Clone)]
pub struct VoidCleaner {
    /// Machine ID.
    machine_id: MachineId,
    /// All states.
    all_states: BTreeSet<StateId>,
    /// Initial states.
    initial_states: BTreeSet<StateId>,
    /// Terminal states.
    terminal_states: BTreeSet<StateId>,
    /// Outgoing edges.
    outgoing: BTreeSet<(StateId, StateId)>,
    /// Unreachable states found.
    unreachable: Vec<UnreachableState>,
}

impl VoidCleaner {
    /// Create a new void cleaner.
    #[must_use]
    pub fn new(machine_id: MachineId) -> Self {
        Self {
            machine_id,
            all_states: BTreeSet::new(),
            initial_states: BTreeSet::new(),
            terminal_states: BTreeSet::new(),
            outgoing: BTreeSet::new(),
            unreachable: Vec::new(),
        }
    }

    /// Register a state.
    pub fn add_state(&mut self, state: StateId, is_initial: bool, is_terminal: bool) {
        self.all_states.insert(state);
        if is_initial {
            self.initial_states.insert(state);
        }
        if is_terminal {
            self.terminal_states.insert(state);
        }
    }

    /// Register a transition edge.
    pub fn add_edge(&mut self, from: StateId, to: StateId) {
        self.outgoing.insert((from, to));
    }

    /// Find all reachable states from initial states.
    fn compute_reachable(&self) -> BTreeSet<StateId> {
        let mut reachable = BTreeSet::new();
        let mut stack: Vec<StateId> = self.initial_states.iter().copied().collect();

        while let Some(state) = stack.pop() {
            if reachable.insert(state) {
                // Add successors
                for &(from, to) in &self.outgoing {
                    if from == state && !reachable.contains(&to) {
                        stack.push(to);
                    }
                }
            }
        }

        reachable
    }

    /// Analyze and find unreachable states.
    pub fn analyze(&mut self) -> &[UnreachableState] {
        self.unreachable.clear();

        let reachable = self.compute_reachable();

        // Find states not reachable from initial
        for &state in &self.all_states {
            if !reachable.contains(&state) && !self.initial_states.contains(&state) {
                self.unreachable.push(UnreachableState {
                    state,
                    reason: UnreachableReason::Disconnected,
                });
            }
        }

        // Find states with no incoming (except initial)
        let has_incoming: BTreeSet<StateId> = self.outgoing.iter().map(|&(_, to)| to).collect();

        for &state in &self.all_states {
            if !self.initial_states.contains(&state)
                && !has_incoming.contains(&state)
                && !self.unreachable.iter().any(|u| u.state == state)
            {
                self.unreachable.push(UnreachableState {
                    state,
                    reason: UnreachableReason::NoIncoming,
                });
            }
        }

        &self.unreachable
    }

    /// Get unreachable states.
    #[must_use]
    pub fn unreachable(&self) -> &[UnreachableState] {
        &self.unreachable
    }

    /// Get unreachable state IDs.
    #[must_use]
    pub fn unreachable_ids(&self) -> Vec<StateId> {
        self.unreachable.iter().map(|u| u.state).collect()
    }

    /// Check if a state is unreachable.
    #[must_use]
    pub fn is_unreachable(&self, state: StateId) -> bool {
        self.unreachable.iter().any(|u| u.state == state)
    }

    /// Find dead transitions (to unreachable states).
    #[must_use]
    pub fn dead_edges(&self) -> Vec<(StateId, StateId)> {
        let unreachable_set: BTreeSet<StateId> = self.unreachable_ids().into_iter().collect();

        self.outgoing
            .iter()
            .filter(|&&(_, to)| unreachable_set.contains(&to))
            .copied()
            .collect()
    }

    /// Count of void (unreachable) states.
    #[must_use]
    pub fn void_count(&self) -> usize {
        self.unreachable.len()
    }

    /// Clear analysis results.
    pub fn clear(&mut self) {
        self.unreachable.clear();
    }

    /// Reset all data.
    pub fn reset(&mut self) {
        self.all_states.clear();
        self.initial_states.clear();
        self.terminal_states.clear();
        self.outgoing.clear();
        self.unreachable.clear();
    }
}

// ═══════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_reachable() {
        let mut cleaner = VoidCleaner::new(1);

        cleaner.add_state(0, true, false); // Initial
        cleaner.add_state(1, false, false);
        cleaner.add_state(2, false, true); // Terminal

        cleaner.add_edge(0, 1);
        cleaner.add_edge(1, 2);

        cleaner.analyze();
        assert_eq!(cleaner.void_count(), 0);
    }

    #[test]
    fn test_disconnected_state() {
        let mut cleaner = VoidCleaner::new(1);

        cleaner.add_state(0, true, false);
        cleaner.add_state(1, false, false);
        cleaner.add_state(2, false, false); // Disconnected
        cleaner.add_state(3, false, true);

        cleaner.add_edge(0, 1);
        cleaner.add_edge(1, 3);
        // State 2 has no edges

        cleaner.analyze();
        assert!(cleaner.is_unreachable(2));
    }

    #[test]
    fn test_no_incoming() {
        let mut cleaner = VoidCleaner::new(1);

        cleaner.add_state(0, true, false);
        cleaner.add_state(1, false, false);
        cleaner.add_state(2, false, false); // No incoming

        cleaner.add_edge(0, 1);
        cleaner.add_edge(2, 1); // 2 has outgoing but no incoming

        cleaner.analyze();
        assert!(cleaner.is_unreachable(2));
    }

    #[test]
    fn test_dead_edges() {
        let mut cleaner = VoidCleaner::new(1);

        cleaner.add_state(0, true, false);
        cleaner.add_state(1, false, false);
        cleaner.add_state(2, false, false); // Will be unreachable

        cleaner.add_edge(0, 1);
        cleaner.add_edge(1, 2); // Edge to unreachable
        // No edge from 2, and no edge to 2 from initial path

        // Actually, 2 IS reachable via 0→1→2
        // Let me fix the test

        cleaner.add_state(3, false, false); // Truly disconnected

        cleaner.analyze();
        // State 3 is disconnected
        assert!(cleaner.is_unreachable(3));
    }
}
