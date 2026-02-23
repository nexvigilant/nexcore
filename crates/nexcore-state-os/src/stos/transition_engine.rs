// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Layer 2: Transition Engine (STOS-TR)
//!
//! **Dominant Primitive**: → (Causality)
//!
//! Executes state transitions, managing the causal flow between states.
//!
//! ## Responsibilities
//!
//! - Transition definition and registration
//! - Transition execution
//! - Pre/post transition hooks
//! - Transition history tracking
//!
//! ## Tier Classification
//!
//! `TransitionEngine` is T2-C (→ + ς + κ) — causality, state, comparison.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use super::state_registry::StateId;
use crate::MachineId;

/// Unique identifier for a transition.
pub type TransitionId = u32;

/// A transition specification.
#[derive(Debug, Clone)]
pub struct TransitionSpec {
    /// Unique transition ID.
    pub id: TransitionId,
    /// Human-readable name/action.
    pub name: String,
    /// Source state ID.
    pub from: StateId,
    /// Target state ID.
    pub to: StateId,
    /// Optional guard reference (evaluated by Layer 4).
    pub guard: Option<String>,
    /// Priority (higher = preferred).
    pub priority: u32,
    /// Whether transition is enabled.
    pub enabled: bool,
}

impl TransitionSpec {
    /// Create a new transition.
    #[must_use]
    pub fn new(id: TransitionId, name: impl Into<String>, from: StateId, to: StateId) -> Self {
        Self {
            id,
            name: name.into(),
            from,
            to,
            guard: None,
            priority: 0,
            enabled: true,
        }
    }

    /// Add a guard condition.
    #[must_use]
    pub fn with_guard(mut self, guard: impl Into<String>) -> Self {
        self.guard = Some(guard.into());
        self
    }

    /// Set priority.
    #[must_use]
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// Disable the transition.
    #[must_use]
    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

/// Result of a transition execution.
#[derive(Debug, Clone)]
pub struct TransitionResult {
    /// The transition that was executed.
    pub transition_id: TransitionId,
    /// Source state before transition.
    pub from_state: StateId,
    /// Target state after transition.
    pub to_state: StateId,
    /// Whether the transition succeeded.
    pub success: bool,
    /// Timestamp of execution (monotonic counter).
    pub timestamp: u64,
    /// Optional error message.
    pub error: Option<String>,
}

impl TransitionResult {
    /// Create a successful result.
    #[must_use]
    pub fn success(
        transition_id: TransitionId,
        from: StateId,
        to: StateId,
        timestamp: u64,
    ) -> Self {
        Self {
            transition_id,
            from_state: from,
            to_state: to,
            success: true,
            timestamp,
            error: None,
        }
    }

    /// Create a failed result.
    #[must_use]
    pub fn failure(
        transition_id: TransitionId,
        from: StateId,
        to: StateId,
        timestamp: u64,
        error: impl Into<String>,
    ) -> Self {
        Self {
            transition_id,
            from_state: from,
            to_state: to,
            success: false,
            timestamp,
            error: Some(error.into()),
        }
    }
}

/// Transition engine errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransitionError {
    /// Transition not found.
    TransitionNotFound(TransitionId),
    /// Transition name not found.
    TransitionNameNotFound(String),
    /// No transition from current state with given action.
    NoTransitionAvailable {
        /// The state from which transition was attempted.
        from: StateId,
        /// The action/event name that was not found.
        action: String,
    },
    /// Transition is disabled.
    TransitionDisabled(TransitionId),
    /// Guard failed.
    GuardFailed(String),
    /// Invalid source state.
    InvalidSourceState {
        /// The expected source state for the transition.
        expected: StateId,
        /// The actual current state of the machine.
        actual: StateId,
    },
}

impl fmt::Display for TransitionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TransitionNotFound(id) => write!(f, "Transition not found: {id}"),
            Self::TransitionNameNotFound(name) => write!(f, "Transition not found: {name}"),
            Self::NoTransitionAvailable { from, action } => {
                write!(f, "No transition '{action}' available from state {from}")
            }
            Self::TransitionDisabled(id) => write!(f, "Transition disabled: {id}"),
            Self::GuardFailed(msg) => write!(f, "Guard failed: {msg}"),
            Self::InvalidSourceState { expected, actual } => {
                write!(f, "Invalid source state: expected {expected}, got {actual}")
            }
        }
    }
}

/// The transition engine for a single machine.
///
/// ## Tier: T2-C (→ + ς + κ)
///
/// Dominant primitive: → (Causality)
#[derive(Debug, Clone)]
pub struct TransitionEngine {
    /// Machine this engine belongs to.
    _machine_id: MachineId,
    /// Transitions by ID.
    transitions: BTreeMap<TransitionId, TransitionSpec>,
    /// Name to ID mapping.
    name_index: BTreeMap<String, TransitionId>,
    /// Outgoing transitions by source state.
    outgoing: BTreeMap<StateId, Vec<TransitionId>>,
    /// Next available transition ID.
    next_id: TransitionId,
    /// Execution counter (monotonic timestamp).
    execution_counter: u64,
    /// Transition history.
    history: Vec<TransitionResult>,
    /// Maximum history entries.
    max_history: usize,
}

impl TransitionEngine {
    /// Create a new transition engine.
    #[must_use]
    pub fn new(machine_id: MachineId) -> Self {
        Self {
            _machine_id: machine_id,
            transitions: BTreeMap::new(),
            name_index: BTreeMap::new(),
            outgoing: BTreeMap::new(),
            next_id: 0,
            execution_counter: 0,
            history: Vec::new(),
            max_history: 1000,
        }
    }

    /// Register a transition.
    pub fn register(
        &mut self,
        name: impl Into<String>,
        from: StateId,
        to: StateId,
    ) -> TransitionId {
        let name = name.into();
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);

        let spec = TransitionSpec::new(id, name.clone(), from, to);
        self.transitions.insert(id, spec);
        self.name_index.insert(name, id);
        self.outgoing.entry(from).or_default().push(id);

        id
    }

    /// Register a guarded transition.
    pub fn register_guarded(
        &mut self,
        name: impl Into<String>,
        from: StateId,
        to: StateId,
        guard: impl Into<String>,
    ) -> TransitionId {
        let name = name.into();
        let guard = guard.into();
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);

        let spec = TransitionSpec::new(id, name.clone(), from, to).with_guard(guard);
        self.transitions.insert(id, spec);
        self.name_index.insert(name, id);
        self.outgoing.entry(from).or_default().push(id);

        id
    }

    /// Get transition by ID.
    #[must_use]
    pub fn get(&self, id: TransitionId) -> Option<&TransitionSpec> {
        self.transitions.get(&id)
    }

    /// Get transition by name.
    #[must_use]
    pub fn get_by_name(&self, name: &str) -> Option<&TransitionSpec> {
        self.name_index
            .get(name)
            .and_then(|id| self.transitions.get(id))
    }

    /// Get all outgoing transitions from a state.
    #[must_use]
    pub fn outgoing_from(&self, state: StateId) -> Vec<&TransitionSpec> {
        self.outgoing
            .get(&state)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.transitions.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Find transition by action name from a state.
    #[must_use]
    pub fn find_transition(&self, from: StateId, action: &str) -> Option<&TransitionSpec> {
        self.outgoing_from(from)
            .into_iter()
            .find(|t| t.name == action && t.enabled)
    }

    /// Execute a transition (records result, doesn't actually change state).
    ///
    /// State change is managed by the kernel.
    pub fn execute(
        &mut self,
        transition_id: TransitionId,
        current_state: StateId,
    ) -> Result<TransitionResult, TransitionError> {
        let spec = self
            .transitions
            .get(&transition_id)
            .ok_or(TransitionError::TransitionNotFound(transition_id))?;

        // Verify source state
        if spec.from != current_state {
            return Err(TransitionError::InvalidSourceState {
                expected: spec.from,
                actual: current_state,
            });
        }

        // Check enabled
        if !spec.enabled {
            return Err(TransitionError::TransitionDisabled(transition_id));
        }

        // Execute
        self.execution_counter = self.execution_counter.saturating_add(1);
        let result =
            TransitionResult::success(transition_id, spec.from, spec.to, self.execution_counter);

        // Record history
        self.history.push(result.clone());
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }

        Ok(result)
    }

    /// Get transition history.
    #[must_use]
    pub fn history(&self) -> &[TransitionResult] {
        &self.history
    }

    /// Total transitions registered.
    #[must_use]
    pub fn len(&self) -> usize {
        self.transitions.len()
    }

    /// Whether no transitions are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.transitions.is_empty()
    }

    /// Execution counter.
    #[must_use]
    pub fn execution_count(&self) -> u64 {
        self.execution_counter
    }
}

// ═══════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transition_spec() {
        let spec = TransitionSpec::new(0, "confirm", 0, 1)
            .with_guard("order.valid()")
            .with_priority(10);

        assert_eq!(spec.name, "confirm");
        assert_eq!(spec.from, 0);
        assert_eq!(spec.to, 1);
        assert!(spec.guard.is_some());
        assert_eq!(spec.priority, 10);
    }

    #[test]
    fn test_engine_register() {
        let mut engine = TransitionEngine::new(1);

        let t1 = engine.register("start", 0, 1);
        let t2 = engine.register("process", 1, 2);
        let t3 = engine.register("finish", 2, 3);

        assert_eq!(t1, 0);
        assert_eq!(t2, 1);
        assert_eq!(t3, 2);
        assert_eq!(engine.len(), 3);
    }

    #[test]
    fn test_outgoing_transitions() {
        let mut engine = TransitionEngine::new(1);

        engine.register("a", 0, 1);
        engine.register("b", 0, 2);
        engine.register("c", 1, 2);

        let from_0 = engine.outgoing_from(0);
        let from_1 = engine.outgoing_from(1);

        assert_eq!(from_0.len(), 2);
        assert_eq!(from_1.len(), 1);
    }

    #[test]
    fn test_execute_transition() {
        let mut engine = TransitionEngine::new(1);
        let t_id = engine.register("advance", 0, 1);

        let result = engine.execute(t_id, 0);
        assert!(result.is_ok());

        let result = result.ok();
        assert!(result.is_some());
        if let Some(r) = result {
            assert!(r.success);
            assert_eq!(r.from_state, 0);
            assert_eq!(r.to_state, 1);
        }
    }

    #[test]
    fn test_execute_wrong_state() {
        let mut engine = TransitionEngine::new(1);
        let t_id = engine.register("advance", 0, 1);

        // Try to execute from wrong state
        let result = engine.execute(t_id, 5);
        assert!(matches!(
            result,
            Err(TransitionError::InvalidSourceState { .. })
        ));
    }

    #[test]
    fn test_find_transition() {
        let mut engine = TransitionEngine::new(1);
        engine.register("confirm", 0, 1);
        engine.register("cancel", 0, 2);

        let found = engine.find_transition(0, "confirm");
        assert!(found.is_some());
        assert_eq!(found.map(|t| t.to), Some(1));

        let not_found = engine.find_transition(0, "unknown");
        assert!(not_found.is_none());
    }
}
