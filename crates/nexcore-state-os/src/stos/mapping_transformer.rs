// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Layer 15: Mapping Transformer (STOS-MP)
//!
//! **Dominant Primitive**: μ (Mapping)
//!
//! Transforms between different state machine representations,
//! handles state/event mapping, and provides conversion utilities.
//!
//! ## Tier Classification
//!
//! `MappingTransformer` is T2-C (μ + ς + σ) — mapping, state, sequence.

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use super::state_registry::StateId;
use super::transition_engine::TransitionId;
use crate::MachineId;

/// A state mapping.
#[derive(Debug, Clone)]
pub struct StateMapping {
    /// Source machine.
    pub source_machine: MachineId,
    /// Target machine.
    pub target_machine: MachineId,
    /// State correspondence.
    pub state_map: BTreeMap<StateId, StateId>,
}

impl StateMapping {
    /// Create a new mapping.
    #[must_use]
    pub fn new(source_machine: MachineId, target_machine: MachineId) -> Self {
        Self {
            source_machine,
            target_machine,
            state_map: BTreeMap::new(),
        }
    }

    /// Add a state correspondence.
    pub fn add(&mut self, source_state: StateId, target_state: StateId) {
        self.state_map.insert(source_state, target_state);
    }

    /// Map a state.
    #[must_use]
    pub fn map_state(&self, source_state: StateId) -> Option<StateId> {
        self.state_map.get(&source_state).copied()
    }

    /// Reverse the mapping.
    #[must_use]
    pub fn reverse(&self) -> Self {
        let mut reversed = Self::new(self.target_machine, self.source_machine);
        for (&source, &target) in &self.state_map {
            reversed.add(target, source);
        }
        reversed
    }

    /// Whether mapping is complete for given states.
    #[must_use]
    pub fn is_complete(&self, source_states: &[StateId]) -> bool {
        source_states.iter().all(|s| self.state_map.contains_key(s))
    }

    /// Unmapped source states.
    #[must_use]
    pub fn unmapped(&self, source_states: &[StateId]) -> Vec<StateId> {
        source_states
            .iter()
            .filter(|s| !self.state_map.contains_key(s))
            .copied()
            .collect()
    }
}

/// A transition mapping.
#[derive(Debug, Clone)]
pub struct TransitionMapping {
    /// Source machine.
    pub source_machine: MachineId,
    /// Target machine.
    pub target_machine: MachineId,
    /// Transition correspondence.
    pub transition_map: BTreeMap<TransitionId, TransitionId>,
}

impl TransitionMapping {
    /// Create a new mapping.
    #[must_use]
    pub fn new(source_machine: MachineId, target_machine: MachineId) -> Self {
        Self {
            source_machine,
            target_machine,
            transition_map: BTreeMap::new(),
        }
    }

    /// Add a transition correspondence.
    pub fn add(&mut self, source_transition: TransitionId, target_transition: TransitionId) {
        self.transition_map
            .insert(source_transition, target_transition);
    }

    /// Map a transition.
    #[must_use]
    pub fn map_transition(&self, source_transition: TransitionId) -> Option<TransitionId> {
        self.transition_map.get(&source_transition).copied()
    }
}

/// An event to state mapping.
#[derive(Debug, Clone)]
pub struct EventStateMapping {
    /// Event name to state transitions.
    event_map: BTreeMap<String, Vec<(StateId, StateId)>>,
}

impl EventStateMapping {
    /// Create a new mapping.
    #[must_use]
    pub fn new() -> Self {
        Self {
            event_map: BTreeMap::new(),
        }
    }

    /// Register an event-triggered transition.
    pub fn register(&mut self, event: impl Into<String>, from_state: StateId, to_state: StateId) {
        let event = event.into();
        self.event_map
            .entry(event)
            .or_default()
            .push((from_state, to_state));
    }

    /// Get transitions for an event.
    #[must_use]
    pub fn transitions_for(&self, event: &str) -> Option<&[(StateId, StateId)]> {
        self.event_map.get(event).map(Vec::as_slice)
    }

    /// Get applicable transition for event from a given state.
    #[must_use]
    pub fn transition_from(&self, event: &str, current_state: StateId) -> Option<StateId> {
        self.event_map
            .get(event)?
            .iter()
            .find(|(from, _)| *from == current_state)
            .map(|(_, to)| *to)
    }

    /// All registered events.
    #[must_use]
    pub fn events(&self) -> Vec<&String> {
        self.event_map.keys().collect()
    }
}

impl Default for EventStateMapping {
    fn default() -> Self {
        Self::new()
    }
}

/// The mapping transformer.
///
/// ## Tier: T2-C (μ + ς + σ)
///
/// Dominant primitive: μ (Mapping)
#[derive(Debug, Clone)]
pub struct MappingTransformer {
    /// State mappings indexed by (source, target).
    state_mappings: BTreeMap<(MachineId, MachineId), StateMapping>,
    /// Transition mappings.
    transition_mappings: BTreeMap<(MachineId, MachineId), TransitionMapping>,
    /// Event-state mappings per machine.
    event_mappings: BTreeMap<MachineId, EventStateMapping>,
    /// Name to state ID mappings.
    name_to_state: BTreeMap<(MachineId, String), StateId>,
    /// State ID to name mappings.
    state_to_name: BTreeMap<(MachineId, StateId), String>,
}

impl MappingTransformer {
    /// Create a new mapping transformer.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state_mappings: BTreeMap::new(),
            transition_mappings: BTreeMap::new(),
            event_mappings: BTreeMap::new(),
            name_to_state: BTreeMap::new(),
            state_to_name: BTreeMap::new(),
        }
    }

    /// Register a state mapping between machines.
    pub fn register_state_mapping(&mut self, mapping: StateMapping) {
        let key = (mapping.source_machine, mapping.target_machine);
        self.state_mappings.insert(key, mapping);
    }

    /// Register a transition mapping.
    pub fn register_transition_mapping(&mut self, mapping: TransitionMapping) {
        let key = (mapping.source_machine, mapping.target_machine);
        self.transition_mappings.insert(key, mapping);
    }

    /// Register event-state mapping for a machine.
    pub fn register_event_mapping(&mut self, machine_id: MachineId, mapping: EventStateMapping) {
        self.event_mappings.insert(machine_id, mapping);
    }

    /// Register a state name.
    pub fn register_state_name(
        &mut self,
        machine_id: MachineId,
        state: StateId,
        name: impl Into<String>,
    ) {
        let name = name.into();
        self.name_to_state.insert((machine_id, name.clone()), state);
        self.state_to_name.insert((machine_id, state), name);
    }

    /// Get state ID from name.
    #[must_use]
    pub fn state_from_name(&self, machine_id: MachineId, name: &str) -> Option<StateId> {
        self.name_to_state
            .get(&(machine_id, name.to_string()))
            .copied()
    }

    /// Get state name from ID.
    #[must_use]
    pub fn name_from_state(&self, machine_id: MachineId, state: StateId) -> Option<&String> {
        self.state_to_name.get(&(machine_id, state))
    }

    /// Map state between machines.
    #[must_use]
    pub fn map_state(
        &self,
        source_machine: MachineId,
        target_machine: MachineId,
        state: StateId,
    ) -> Option<StateId> {
        self.state_mappings
            .get(&(source_machine, target_machine))?
            .map_state(state)
    }

    /// Map transition between machines.
    #[must_use]
    pub fn map_transition(
        &self,
        source_machine: MachineId,
        target_machine: MachineId,
        transition: TransitionId,
    ) -> Option<TransitionId> {
        self.transition_mappings
            .get(&(source_machine, target_machine))?
            .map_transition(transition)
    }

    /// Get target state for event on machine.
    #[must_use]
    pub fn event_transition(
        &self,
        machine_id: MachineId,
        event: &str,
        current_state: StateId,
    ) -> Option<StateId> {
        self.event_mappings
            .get(&machine_id)?
            .transition_from(event, current_state)
    }

    /// Get event mapping for machine.
    #[must_use]
    pub fn get_event_mapping(&self, machine_id: MachineId) -> Option<&EventStateMapping> {
        self.event_mappings.get(&machine_id)
    }

    /// Check if state mapping exists.
    #[must_use]
    pub fn has_state_mapping(&self, source: MachineId, target: MachineId) -> bool {
        self.state_mappings.contains_key(&(source, target))
    }

    /// Count of state mappings.
    #[must_use]
    pub fn state_mapping_count(&self) -> usize {
        self.state_mappings.len()
    }

    /// Count of event mappings.
    #[must_use]
    pub fn event_mapping_count(&self) -> usize {
        self.event_mappings.len()
    }
}

impl Default for MappingTransformer {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_mapping() {
        let mut mapping = StateMapping::new(1, 2);
        mapping.add(0, 10);
        mapping.add(1, 11);
        mapping.add(2, 12);

        assert_eq!(mapping.map_state(0), Some(10));
        assert_eq!(mapping.map_state(1), Some(11));
        assert_eq!(mapping.map_state(99), None);
    }

    #[test]
    fn test_reverse_mapping() {
        let mut mapping = StateMapping::new(1, 2);
        mapping.add(0, 10);
        mapping.add(1, 11);

        let reversed = mapping.reverse();
        assert_eq!(reversed.map_state(10), Some(0));
        assert_eq!(reversed.map_state(11), Some(1));
    }

    #[test]
    fn test_event_state_mapping() {
        let mut mapping = EventStateMapping::new();
        mapping.register("start", 0, 1);
        mapping.register("process", 1, 2);
        mapping.register("complete", 2, 3);

        assert_eq!(mapping.transition_from("start", 0), Some(1));
        assert_eq!(mapping.transition_from("process", 1), Some(2));
        assert_eq!(mapping.transition_from("start", 1), None); // Wrong state
    }

    #[test]
    fn test_transformer_state_names() {
        let mut transformer = MappingTransformer::new();

        transformer.register_state_name(1, 0, "idle");
        transformer.register_state_name(1, 1, "running");
        transformer.register_state_name(1, 2, "stopped");

        assert_eq!(transformer.state_from_name(1, "idle"), Some(0));
        assert_eq!(
            transformer.name_from_state(1, 1).map(String::as_str),
            Some("running")
        );
    }

    #[test]
    fn test_transformer_event_transition() {
        let mut transformer = MappingTransformer::new();

        let mut event_map = EventStateMapping::new();
        event_map.register("start", 0, 1);
        event_map.register("stop", 1, 0);

        transformer.register_event_mapping(1, event_map);

        assert_eq!(transformer.event_transition(1, "start", 0), Some(1));
        assert_eq!(transformer.event_transition(1, "stop", 1), Some(0));
    }

    #[test]
    fn test_cross_machine_mapping() {
        let mut transformer = MappingTransformer::new();

        let mut mapping = StateMapping::new(1, 2);
        mapping.add(0, 100);
        mapping.add(1, 101);

        transformer.register_state_mapping(mapping);

        assert_eq!(transformer.map_state(1, 2, 0), Some(100));
        assert_eq!(transformer.map_state(1, 2, 1), Some(101));
    }

    #[test]
    fn test_completeness_check() {
        let mut mapping = StateMapping::new(1, 2);
        mapping.add(0, 10);
        mapping.add(1, 11);

        assert!(mapping.is_complete(&[0, 1]));
        assert!(!mapping.is_complete(&[0, 1, 2]));
        assert_eq!(mapping.unmapped(&[0, 1, 2]), vec![2]);
    }
}
