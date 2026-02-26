// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Machine Specification and Building
//!
//! Provides builder patterns for constructing state machines
//! with a fluent API.

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::kernel::MachineId;
use crate::stos::state_registry::{StateId, StateKind};
use crate::stos::transition_engine::TransitionId;

/// A state specification.
/// Tier: T2-P (ς + μ) — state and mapping
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StateSpec {
    /// State ID.
    pub id: StateId,
    /// State name.
    pub name: String,
    /// State kind.
    pub kind: StateKind,
}

/// A transition specification.
/// Tier: T2-P (→ + μ) — causality and mapping
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TransitionSpec {
    /// Transition ID.
    pub id: TransitionId,
    /// Source state name.
    pub from: String,
    /// Target state name.
    pub to: String,
    /// Transition event name.
    pub event: String,
}

/// A complete machine specification.
/// Tier: T3 (ς + → + μ + N) — full state machine domain type
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MachineSpec {
    /// Machine name.
    pub name: String,
    /// States.
    pub states: Vec<StateSpec>,
    /// Transitions.
    pub transitions: Vec<TransitionSpec>,
    /// State name to ID mapping.
    state_ids: BTreeMap<String, StateId>,
    /// Transition counter.
    transition_counter: TransitionId,
}

impl MachineSpec {
    /// Create a new machine spec builder.
    #[must_use]
    pub fn builder(name: impl Into<String>) -> MachineBuilder {
        MachineBuilder::new(name)
    }

    /// Get initial state ID.
    #[must_use]
    pub fn initial_state(&self) -> Option<StateId> {
        self.states
            .iter()
            .find(|s| s.kind == StateKind::Initial)
            .map(|s| s.id)
    }

    /// Get terminal state IDs.
    #[must_use]
    pub fn terminal_states(&self) -> Vec<StateId> {
        self.states
            .iter()
            .filter(|s| s.kind == StateKind::Terminal)
            .map(|s| s.id)
            .collect()
    }

    /// Get state ID by name.
    #[must_use]
    pub fn state_id(&self, name: &str) -> Option<StateId> {
        self.state_ids.get(name).copied()
    }

    /// Get state by ID.
    #[must_use]
    pub fn state(&self, id: StateId) -> Option<&StateSpec> {
        self.states.iter().find(|s| s.id == id)
    }

    /// Get transitions from a state.
    #[must_use]
    pub fn transitions_from(&self, state_name: &str) -> Vec<&TransitionSpec> {
        self.transitions
            .iter()
            .filter(|t| t.from == state_name)
            .collect()
    }
}

/// Builder for machine specifications.
#[derive(Debug, Clone)]
pub struct MachineBuilder {
    name: String,
    states: Vec<StateSpec>,
    transitions: Vec<TransitionSpec>,
    state_ids: BTreeMap<String, StateId>,
    state_counter: StateId,
    transition_counter: TransitionId,
}

impl MachineBuilder {
    /// Create a new builder.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            states: Vec::new(),
            transitions: Vec::new(),
            state_ids: BTreeMap::new(),
            state_counter: 0,
            transition_counter: 0,
        }
    }

    /// Add a state.
    #[must_use]
    pub fn state(mut self, name: impl Into<String>, kind: StateKind) -> Self {
        let name = name.into();
        let id = self.state_counter;
        self.state_counter += 1;

        self.state_ids.insert(name.clone(), id);
        self.states.push(StateSpec { id, name, kind });
        self
    }

    /// Add a transition.
    #[must_use]
    pub fn transition(
        mut self,
        from: impl Into<String>,
        to: impl Into<String>,
        event: impl Into<String>,
    ) -> Self {
        let id = self.transition_counter;
        self.transition_counter += 1;

        self.transitions.push(TransitionSpec {
            id,
            from: from.into(),
            to: to.into(),
            event: event.into(),
        });
        self
    }

    /// Build the machine spec.
    #[must_use]
    pub fn build(self) -> MachineSpec {
        MachineSpec {
            name: self.name,
            states: self.states,
            transitions: self.transitions,
            state_ids: self.state_ids,
            transition_counter: self.transition_counter,
        }
    }
}

/// A running machine instance.
#[derive(Debug)]
pub struct MachineInstance {
    /// Machine ID.
    pub id: MachineId,
    /// Machine specification.
    pub spec: MachineSpec,
    /// Current state.
    pub current_state: StateId,
    /// Transition history.
    pub history: Vec<(TransitionId, StateId, StateId)>,
    /// Whether machine is terminated.
    pub terminated: bool,
}

impl MachineInstance {
    /// Create a new instance from a spec.
    #[must_use]
    pub fn new(id: MachineId, spec: MachineSpec) -> Option<Self> {
        let initial = spec.initial_state()?;
        Some(Self {
            id,
            spec,
            current_state: initial,
            history: Vec::new(),
            terminated: false,
        })
    }

    /// Get current state name.
    #[must_use]
    pub fn current_state_name(&self) -> Option<&str> {
        self.spec.state(self.current_state).map(|s| s.name.as_str())
    }

    /// Get available events from current state.
    #[must_use]
    pub fn available_events(&self) -> Vec<&str> {
        self.spec
            .state(self.current_state)
            .map_or_else(Vec::new, |state| {
                self.spec
                    .transitions_from(&state.name)
                    .iter()
                    .map(|t| t.event.as_str())
                    .collect()
            })
    }

    /// Check if an event is available.
    #[must_use]
    pub fn can_handle(&self, event: &str) -> bool {
        self.available_events().contains(&event)
    }

    /// Handle an event (simulate transition).
    pub fn handle(&mut self, event: &str) -> bool {
        if self.terminated {
            return false;
        }

        let current_name = match self.current_state_name() {
            Some(n) => n.to_string(),
            None => return false,
        };

        // Find matching transition
        let transition = self
            .spec
            .transitions
            .iter()
            .find(|t| t.from == current_name && t.event == event);

        if let Some(t) = transition {
            let Some(to_id) = self.spec.state_id(&t.to) else {
                return false;
            };

            let from_id = self.current_state;
            self.history.push((t.id, from_id, to_id));
            self.current_state = to_id;

            // Check if terminal
            if let Some(state) = self.spec.state(to_id) {
                if state.kind == StateKind::Terminal {
                    self.terminated = true;
                }
            }

            true
        } else {
            false
        }
    }

    /// Transition count.
    #[must_use]
    pub fn transition_count(&self) -> usize {
        self.history.len()
    }
}

// ═══════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_machine_builder() {
        let spec = MachineSpec::builder("order")
            .state("pending", StateKind::Initial)
            .state("confirmed", StateKind::Normal)
            .state("shipped", StateKind::Normal)
            .state("delivered", StateKind::Terminal)
            .transition("pending", "confirmed", "confirm")
            .transition("confirmed", "shipped", "ship")
            .transition("shipped", "delivered", "deliver")
            .build();

        assert_eq!(spec.name, "order");
        assert_eq!(spec.states.len(), 4);
        assert_eq!(spec.transitions.len(), 3);
    }

    #[test]
    fn test_initial_terminal() {
        let spec = MachineSpec::builder("test")
            .state("start", StateKind::Initial)
            .state("middle", StateKind::Normal)
            .state("end", StateKind::Terminal)
            .build();

        assert_eq!(spec.initial_state(), Some(0));
        assert_eq!(spec.terminal_states(), vec![2]);
    }

    #[test]
    fn test_machine_instance() {
        let spec = MachineSpec::builder("order")
            .state("pending", StateKind::Initial)
            .state("confirmed", StateKind::Normal)
            .state("delivered", StateKind::Terminal)
            .transition("pending", "confirmed", "confirm")
            .transition("confirmed", "delivered", "deliver")
            .build();

        let mut instance = MachineInstance::new(1, spec);
        assert!(instance.is_some());

        let i = match instance.as_mut() {
            Some(i) => i,
            None => return, // MachineInstance::new returned None — test setup failed
        };

        assert_eq!(i.current_state_name(), Some("pending"));
        assert!(i.can_handle("confirm"));
        assert!(!i.can_handle("deliver"));

        // Handle confirm
        assert!(i.handle("confirm"));
        assert_eq!(i.current_state_name(), Some("confirmed"));

        // Handle deliver
        assert!(i.handle("deliver"));
        assert_eq!(i.current_state_name(), Some("delivered"));
        assert!(i.terminated);

        // Can't handle more events when terminated
        assert!(!i.handle("confirm"));
    }

    #[test]
    fn test_transitions_from() {
        let spec = MachineSpec::builder("test")
            .state("a", StateKind::Initial)
            .state("b", StateKind::Normal)
            .state("c", StateKind::Terminal)
            .transition("a", "b", "go_b")
            .transition("a", "c", "go_c")
            .transition("b", "c", "finish")
            .build();

        let from_a = spec.transitions_from("a");
        assert_eq!(from_a.len(), 2);
    }
}
