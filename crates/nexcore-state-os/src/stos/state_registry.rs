// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Layer 1: State Registry (STOS-ST)
//!
//! **Dominant Primitive**: ς (State)
//!
//! The foundational layer managing state definitions, registration,
//! and lookup for all state machines in the system.
//!
//! ## Responsibilities
//!
//! - State definition and naming
//! - State ID assignment
//! - State metadata storage
//! - State lookup by name or ID
//!
//! ## Tier Classification
//!
//! `StateRegistry` is T2-C (ς + μ + N) — state, mapping, quantity.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use crate::MachineId;

/// Unique identifier for a state within a machine.
pub type StateId = u32;

/// The kind of state in a machine lifecycle.
/// Tier: T1 (ς) — pure state enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StateKind {
    /// Initial state (entry point).
    Initial,
    /// Normal intermediate state.
    Normal,
    /// Terminal state (absorbing, no outgoing transitions).
    Terminal,
    /// Error state (recoverable or not).
    Error,
}

impl StateKind {
    /// Whether this is an initial state.
    #[must_use]
    pub const fn is_initial(&self) -> bool {
        matches!(self, Self::Initial)
    }

    /// Whether this is a terminal state.
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(self, Self::Terminal)
    }

    /// Whether this is an error state.
    #[must_use]
    pub const fn is_error(&self) -> bool {
        matches!(self, Self::Error)
    }
}

/// An entry in the state registry.
#[derive(Debug, Clone)]
pub struct StateEntry {
    /// Unique state ID.
    pub id: StateId,
    /// Human-readable name.
    pub name: String,
    /// State kind (initial, normal, terminal, error).
    pub kind: StateKind,
    /// Optional description.
    pub description: Option<String>,
    /// Custom metadata.
    pub metadata: BTreeMap<String, String>,
}

impl StateEntry {
    /// Create a new state entry.
    #[must_use]
    pub fn new(id: StateId, name: impl Into<String>, kind: StateKind) -> Self {
        Self {
            id,
            name: name.into(),
            kind,
            description: None,
            metadata: BTreeMap::new(),
        }
    }

    /// Add description.
    #[must_use]
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Add metadata.
    #[must_use]
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Registry error types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryError {
    /// State not found.
    StateNotFound(StateId),
    /// State name not found.
    StateNameNotFound(String),
    /// Duplicate state name.
    DuplicateStateName(String),
    /// No initial state defined.
    NoInitialState,
    /// Multiple initial states defined.
    MultipleInitialStates,
    /// Registry is full.
    RegistryFull,
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StateNotFound(id) => write!(f, "State not found: {id}"),
            Self::StateNameNotFound(name) => write!(f, "State name not found: {name}"),
            Self::DuplicateStateName(name) => write!(f, "Duplicate state name: {name}"),
            Self::NoInitialState => write!(f, "No initial state defined"),
            Self::MultipleInitialStates => write!(f, "Multiple initial states defined"),
            Self::RegistryFull => write!(f, "Registry is full"),
        }
    }
}

/// The state registry for a single machine.
///
/// ## Tier: T2-C (ς + μ + N)
///
/// Dominant primitive: ς (State)
#[derive(Debug, Clone)]
pub struct StateRegistry {
    /// Machine this registry belongs to.
    machine_id: MachineId,
    /// States by ID.
    states: BTreeMap<StateId, StateEntry>,
    /// Name to ID mapping.
    name_index: BTreeMap<String, StateId>,
    /// Next available state ID.
    next_id: StateId,
    /// Maximum states allowed.
    max_states: usize,
}

impl StateRegistry {
    /// Create a new state registry.
    #[must_use]
    pub fn new(machine_id: MachineId) -> Self {
        Self {
            machine_id,
            states: BTreeMap::new(),
            name_index: BTreeMap::new(),
            next_id: 0,
            max_states: 1024,
        }
    }

    /// Create with custom capacity.
    #[must_use]
    pub fn with_capacity(machine_id: MachineId, max_states: usize) -> Self {
        Self {
            machine_id,
            states: BTreeMap::new(),
            name_index: BTreeMap::new(),
            next_id: 0,
            max_states,
        }
    }

    /// Register a new state.
    pub fn register(
        &mut self,
        name: impl Into<String>,
        kind: StateKind,
    ) -> Result<StateId, RegistryError> {
        let name = name.into();

        // Check for duplicate name
        if self.name_index.contains_key(&name) {
            return Err(RegistryError::DuplicateStateName(name));
        }

        // Check capacity
        if self.states.len() >= self.max_states {
            return Err(RegistryError::RegistryFull);
        }

        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);

        let entry = StateEntry::new(id, name.clone(), kind);
        self.states.insert(id, entry);
        self.name_index.insert(name, id);

        Ok(id)
    }

    /// Get state by ID.
    #[must_use]
    pub fn get(&self, id: StateId) -> Option<&StateEntry> {
        self.states.get(&id)
    }

    /// Get state by name.
    #[must_use]
    pub fn get_by_name(&self, name: &str) -> Option<&StateEntry> {
        self.name_index.get(name).and_then(|id| self.states.get(id))
    }

    /// Get state ID by name.
    #[must_use]
    pub fn id_of(&self, name: &str) -> Option<StateId> {
        self.name_index.get(name).copied()
    }

    /// Get the initial state.
    #[must_use]
    pub fn initial_state(&self) -> Option<&StateEntry> {
        self.states.values().find(|s| s.kind.is_initial())
    }

    /// Get all terminal states.
    #[must_use]
    pub fn terminal_states(&self) -> Vec<&StateEntry> {
        self.states
            .values()
            .filter(|s| s.kind.is_terminal())
            .collect()
    }

    /// Total number of registered states.
    #[must_use]
    pub fn len(&self) -> usize {
        self.states.len()
    }

    /// Whether the registry is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }

    /// Machine ID this registry belongs to.
    #[must_use]
    pub fn machine_id(&self) -> MachineId {
        self.machine_id
    }

    /// Iterate over all states.
    pub fn iter(&self) -> impl Iterator<Item = &StateEntry> {
        self.states.values()
    }

    /// Validate the registry.
    pub fn validate(&self) -> Result<(), RegistryError> {
        let initial_count = self.states.values().filter(|s| s.kind.is_initial()).count();

        if initial_count == 0 {
            return Err(RegistryError::NoInitialState);
        }
        if initial_count > 1 {
            return Err(RegistryError::MultipleInitialStates);
        }

        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_entry_creation() {
        let entry = StateEntry::new(0, "pending", StateKind::Initial)
            .with_description("Waiting for confirmation");

        assert_eq!(entry.id, 0);
        assert_eq!(entry.name, "pending");
        assert!(entry.kind.is_initial());
        assert!(entry.description.is_some());
    }

    #[test]
    fn test_registry_register() {
        let mut registry = StateRegistry::new(1);

        let id1 = registry.register("initial", StateKind::Initial);
        let id2 = registry.register("processing", StateKind::Normal);
        let id3 = registry.register("done", StateKind::Terminal);

        assert!(id1.is_ok());
        assert!(id2.is_ok());
        assert!(id3.is_ok());
        assert_eq!(registry.len(), 3);
    }

    #[test]
    fn test_registry_duplicate_name() {
        let mut registry = StateRegistry::new(1);

        let _ = registry.register("state1", StateKind::Initial);
        let result = registry.register("state1", StateKind::Normal);

        assert!(matches!(result, Err(RegistryError::DuplicateStateName(_))));
    }

    #[test]
    fn test_registry_lookup() {
        let mut registry = StateRegistry::new(1);
        let id = registry.register("pending", StateKind::Initial);
        assert!(id.is_ok());
        let id = id.ok();

        let by_id = registry.get(id.unwrap_or(0));
        let by_name = registry.get_by_name("pending");

        assert!(by_id.is_some());
        assert!(by_name.is_some());
        assert_eq!(by_id.map(|e| &e.name), by_name.map(|e| &e.name));
    }

    #[test]
    fn test_registry_validation() {
        let mut registry = StateRegistry::new(1);
        let _ = registry.register("state1", StateKind::Normal);

        // No initial state
        assert!(matches!(
            registry.validate(),
            Err(RegistryError::NoInitialState)
        ));

        // Add initial state
        let _ = registry.register("start", StateKind::Initial);
        assert!(registry.validate().is_ok());
    }

    #[test]
    fn test_terminal_states() {
        let mut registry = StateRegistry::new(1);
        let _ = registry.register("start", StateKind::Initial);
        let _ = registry.register("middle", StateKind::Normal);
        let _ = registry.register("end1", StateKind::Terminal);
        let _ = registry.register("end2", StateKind::Terminal);

        let terminals = registry.terminal_states();
        assert_eq!(terminals.len(), 2);
    }
}
