// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Layer 3: Boundary Manager (STOS-BD)
//!
//! **Dominant Primitive**: ∂ (Boundary)
//!
//! Manages initial and terminal state boundaries, enforcing
//! entry/exit conditions and absorbing behavior.
//!
//! ## Responsibilities
//!
//! - Initial state entry validation
//! - Terminal state exit prevention
//! - Boundary crossing detection
//! - Entry/exit hooks
//!
//! ## Tier Classification
//!
//! `BoundaryManager` is T2-P (∂ + ς) — boundary, state.

use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;

use super::state_registry::StateId;
use crate::MachineId;

/// The kind of boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BoundaryKind {
    /// Entry boundary (initial states).
    Entry,
    /// Exit boundary (terminal states).
    Exit,
    /// Error boundary (error states).
    Error,
}

/// A boundary crossing event.
#[derive(Debug, Clone)]
pub struct BoundaryCrossing {
    /// The state crossed into/out of.
    pub state: StateId,
    /// Kind of boundary.
    pub kind: BoundaryKind,
    /// Direction (true = entering boundary, false = exiting).
    pub entering: bool,
    /// Timestamp.
    pub timestamp: u64,
}

/// The boundary manager for a machine.
///
/// ## Tier: T2-P (∂ + ς)
///
/// Dominant primitive: ∂ (Boundary)
#[derive(Debug, Clone)]
pub struct BoundaryManager {
    /// Machine ID.
    machine_id: MachineId,
    /// Initial states (entry boundary).
    initial_states: BTreeSet<StateId>,
    /// Terminal states (exit boundary).
    terminal_states: BTreeSet<StateId>,
    /// Error states.
    error_states: BTreeSet<StateId>,
    /// Boundary crossing history.
    crossings: Vec<BoundaryCrossing>,
    /// Crossing counter.
    crossing_counter: u64,
}

impl BoundaryManager {
    /// Create a new boundary manager.
    #[must_use]
    pub fn new(machine_id: MachineId) -> Self {
        Self {
            machine_id,
            initial_states: BTreeSet::new(),
            terminal_states: BTreeSet::new(),
            error_states: BTreeSet::new(),
            crossings: Vec::new(),
            crossing_counter: 0,
        }
    }

    /// Register an initial state.
    pub fn register_initial(&mut self, state: StateId) {
        self.initial_states.insert(state);
    }

    /// Register a terminal state.
    pub fn register_terminal(&mut self, state: StateId) {
        self.terminal_states.insert(state);
    }

    /// Register an error state.
    pub fn register_error(&mut self, state: StateId) {
        self.error_states.insert(state);
    }

    /// Check if state is initial.
    #[must_use]
    pub fn is_initial(&self, state: StateId) -> bool {
        self.initial_states.contains(&state)
    }

    /// Check if state is terminal.
    #[must_use]
    pub fn is_terminal(&self, state: StateId) -> bool {
        self.terminal_states.contains(&state)
    }

    /// Check if state is an error state.
    #[must_use]
    pub fn is_error(&self, state: StateId) -> bool {
        self.error_states.contains(&state)
    }

    /// Check if state is any boundary.
    #[must_use]
    pub fn is_boundary(&self, state: StateId) -> bool {
        self.is_initial(state) || self.is_terminal(state) || self.is_error(state)
    }

    /// Get boundary kind for a state.
    #[must_use]
    pub fn boundary_kind(&self, state: StateId) -> Option<BoundaryKind> {
        if self.is_initial(state) {
            Some(BoundaryKind::Entry)
        } else if self.is_terminal(state) {
            Some(BoundaryKind::Exit)
        } else if self.is_error(state) {
            Some(BoundaryKind::Error)
        } else {
            None
        }
    }

    /// Validate that a transition respects boundaries.
    ///
    /// Returns error message if invalid, None if valid.
    #[must_use]
    pub fn validate_transition(&self, from: StateId, to: StateId) -> Option<String> {
        // Cannot transition out of terminal state
        if self.is_terminal(from) {
            return Some(alloc::format!(
                "Cannot transition from terminal state {from}"
            ));
        }

        // Can always transition to any non-boundary state
        None
    }

    /// Record a boundary crossing.
    pub fn record_crossing(&mut self, state: StateId, entering: bool) {
        if let Some(kind) = self.boundary_kind(state) {
            self.crossing_counter = self.crossing_counter.saturating_add(1);
            self.crossings.push(BoundaryCrossing {
                state,
                kind,
                entering,
                timestamp: self.crossing_counter,
            });
        }
    }

    /// Get all initial states.
    #[must_use]
    pub fn initial_states(&self) -> Vec<StateId> {
        self.initial_states.iter().copied().collect()
    }

    /// Get all terminal states.
    #[must_use]
    pub fn terminal_states(&self) -> Vec<StateId> {
        self.terminal_states.iter().copied().collect()
    }

    /// Get crossing history.
    #[must_use]
    pub fn crossings(&self) -> &[BoundaryCrossing] {
        &self.crossings
    }

    /// Count of each boundary type.
    #[must_use]
    pub fn counts(&self) -> (usize, usize, usize) {
        (
            self.initial_states.len(),
            self.terminal_states.len(),
            self.error_states.len(),
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
    fn test_boundary_registration() {
        let mut manager = BoundaryManager::new(1);

        manager.register_initial(0);
        manager.register_terminal(5);
        manager.register_error(99);

        assert!(manager.is_initial(0));
        assert!(manager.is_terminal(5));
        assert!(manager.is_error(99));
        assert!(!manager.is_boundary(2));
    }

    #[test]
    fn test_boundary_kind() {
        let mut manager = BoundaryManager::new(1);

        manager.register_initial(0);
        manager.register_terminal(1);

        assert_eq!(manager.boundary_kind(0), Some(BoundaryKind::Entry));
        assert_eq!(manager.boundary_kind(1), Some(BoundaryKind::Exit));
        assert_eq!(manager.boundary_kind(2), None);
    }

    #[test]
    fn test_validate_terminal_exit() {
        let mut manager = BoundaryManager::new(1);
        manager.register_terminal(5);

        // Cannot exit terminal
        let result = manager.validate_transition(5, 6);
        assert!(result.is_some());

        // Can enter terminal
        let result = manager.validate_transition(4, 5);
        assert!(result.is_none());
    }

    #[test]
    fn test_crossing_recording() {
        let mut manager = BoundaryManager::new(1);
        manager.register_initial(0);
        manager.register_terminal(5);

        manager.record_crossing(0, false); // Leaving initial
        manager.record_crossing(5, true); // Entering terminal

        assert_eq!(manager.crossings().len(), 2);
    }
}
