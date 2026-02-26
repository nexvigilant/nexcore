// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Layer 5: Count Metrics (STOS-CT)
//!
//! **Dominant Primitive**: N (Quantity)
//!
//! Tracks quantitative metrics for state machines: state counts,
//! transition counts, visit frequencies, and cardinality bounds.
//!
//! ## Responsibilities
//!
//! - State visit counting
//! - Transition execution counting
//! - Cardinality bound enforcement
//! - Metric aggregation
//!
//! ## Tier Classification
//!
//! `CountMetrics` is T2-P (N + ς) — quantity, state.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use super::state_registry::StateId;
use super::transition_engine::TransitionId;
use crate::MachineId;

/// Metrics for a single machine.
#[derive(Debug, Clone, Default)]
pub struct MachineMetrics {
    /// Total state count.
    pub state_count: usize,
    /// Total transition count.
    pub transition_count: usize,
    /// Total transitions executed.
    pub executions: u64,
    /// State visit counts.
    pub state_visits: BTreeMap<StateId, u64>,
    /// Transition execution counts.
    pub transition_executions: BTreeMap<TransitionId, u64>,
    /// Time in each state (arbitrary units).
    pub state_time: BTreeMap<StateId, u64>,
    /// Current state entry time.
    pub current_state_entry: u64,
}

impl MachineMetrics {
    /// Create new metrics.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Most visited state.
    #[must_use]
    pub fn most_visited_state(&self) -> Option<(StateId, u64)> {
        self.state_visits
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(id, count)| (*id, *count))
    }

    /// Most executed transition.
    #[must_use]
    pub fn most_executed_transition(&self) -> Option<(TransitionId, u64)> {
        self.transition_executions
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(id, count)| (*id, *count))
    }

    /// Average visits per state.
    #[must_use]
    #[allow(
        clippy::cast_precision_loss,
        reason = "usize/u64 -> f64 for ratio computation; precision loss is acceptable for metrics"
    )]
    pub fn average_visits(&self) -> f64 {
        if self.state_count == 0 {
            return 0.0;
        }
        let total: u64 = self.state_visits.values().sum();
        total as f64 / self.state_count as f64
    }

    /// States with zero visits.
    #[must_use]
    pub fn unvisited_states(&self, all_states: &[StateId]) -> Vec<StateId> {
        all_states
            .iter()
            .filter(|s| self.state_visits.get(s).copied().unwrap_or(0) == 0)
            .copied()
            .collect()
    }
}

/// The count metrics tracker.
///
/// ## Tier: T2-P (N + ς)
///
/// Dominant primitive: N (Quantity)
#[derive(Debug, Clone)]
pub struct CountMetrics {
    /// Machine ID.
    _machine_id: MachineId,
    /// Machine metrics.
    metrics: MachineMetrics,
    /// Monotonic counter.
    counter: u64,
    /// Maximum states allowed.
    max_states: usize,
    /// Maximum transitions allowed.
    max_transitions: usize,
}

impl CountMetrics {
    /// Create a new count metrics tracker.
    #[must_use]
    pub fn new(machine_id: MachineId) -> Self {
        Self {
            _machine_id: machine_id,
            metrics: MachineMetrics::new(),
            counter: 0,
            max_states: 1024,
            max_transitions: 4096,
        }
    }

    /// Create with custom limits.
    #[must_use]
    pub fn with_limits(machine_id: MachineId, max_states: usize, max_transitions: usize) -> Self {
        Self {
            _machine_id: machine_id,
            metrics: MachineMetrics::new(),
            counter: 0,
            max_states,
            max_transitions,
        }
    }

    /// Set state count.
    pub fn set_state_count(&mut self, count: usize) {
        self.metrics.state_count = count;
    }

    /// Set transition count.
    pub fn set_transition_count(&mut self, count: usize) {
        self.metrics.transition_count = count;
    }

    /// Record a state visit.
    pub fn record_state_visit(&mut self, state: StateId) {
        self.counter = self.counter.saturating_add(1);
        *self.metrics.state_visits.entry(state).or_insert(0) += 1;
        self.metrics.current_state_entry = self.counter;
    }

    /// Record leaving a state.
    pub fn record_state_exit(&mut self, state: StateId) {
        self.counter = self.counter.saturating_add(1);
        let time_in_state = self
            .counter
            .saturating_sub(self.metrics.current_state_entry);
        *self.metrics.state_time.entry(state).or_insert(0) += time_in_state;
    }

    /// Record a transition execution.
    pub fn record_transition(&mut self, transition: TransitionId) {
        self.metrics.executions = self.metrics.executions.saturating_add(1);
        *self
            .metrics
            .transition_executions
            .entry(transition)
            .or_insert(0) += 1;
    }

    /// Check if adding a state would exceed limits.
    #[must_use]
    pub fn can_add_state(&self) -> bool {
        self.metrics.state_count < self.max_states
    }

    /// Check if adding a transition would exceed limits.
    #[must_use]
    pub fn can_add_transition(&self) -> bool {
        self.metrics.transition_count < self.max_transitions
    }

    /// Get current metrics.
    #[must_use]
    pub fn metrics(&self) -> &MachineMetrics {
        &self.metrics
    }

    /// Total executions.
    #[must_use]
    pub fn total_executions(&self) -> u64 {
        self.metrics.executions
    }

    /// State count.
    #[must_use]
    pub fn state_count(&self) -> usize {
        self.metrics.state_count
    }

    /// Transition count.
    #[must_use]
    pub fn transition_count(&self) -> usize {
        self.metrics.transition_count
    }

    /// Visits to a specific state.
    #[must_use]
    pub fn visits_to(&self, state: StateId) -> u64 {
        self.metrics.state_visits.get(&state).copied().unwrap_or(0)
    }

    /// Executions of a specific transition.
    #[must_use]
    pub fn executions_of(&self, transition: TransitionId) -> u64 {
        self.metrics
            .transition_executions
            .get(&transition)
            .copied()
            .unwrap_or(0)
    }

    /// Get limits.
    #[must_use]
    pub fn limits(&self) -> (usize, usize) {
        (self.max_states, self.max_transitions)
    }

    /// Reset all metrics.
    pub fn reset(&mut self) {
        self.metrics = MachineMetrics::new();
        self.counter = 0;
    }
}

// ═══════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_machine_metrics() {
        let mut metrics = MachineMetrics::new();
        metrics.state_count = 5;
        metrics.transition_count = 8;

        assert_eq!(metrics.state_count, 5);
        assert_eq!(metrics.average_visits(), 0.0);
    }

    #[test]
    fn test_record_visits() {
        let mut tracker = CountMetrics::new(1);
        tracker.set_state_count(3);

        tracker.record_state_visit(0);
        tracker.record_state_visit(1);
        tracker.record_state_visit(0);
        tracker.record_state_visit(2);
        tracker.record_state_visit(0);

        assert_eq!(tracker.visits_to(0), 3);
        assert_eq!(tracker.visits_to(1), 1);
        assert_eq!(tracker.visits_to(2), 1);
    }

    #[test]
    fn test_record_transitions() {
        let mut tracker = CountMetrics::new(1);
        tracker.set_transition_count(2);

        tracker.record_transition(0);
        tracker.record_transition(0);
        tracker.record_transition(1);

        assert_eq!(tracker.total_executions(), 3);
        assert_eq!(tracker.executions_of(0), 2);
        assert_eq!(tracker.executions_of(1), 1);
    }

    #[test]
    fn test_most_visited() {
        let mut tracker = CountMetrics::new(1);

        tracker.record_state_visit(0);
        tracker.record_state_visit(1);
        tracker.record_state_visit(1);
        tracker.record_state_visit(1);

        let most_visited = tracker.metrics().most_visited_state();
        assert_eq!(most_visited, Some((1, 3)));
    }

    #[test]
    fn test_limits() {
        let tracker = CountMetrics::with_limits(1, 10, 20);

        assert_eq!(tracker.limits(), (10, 20));
        assert!(tracker.can_add_state());
        assert!(tracker.can_add_transition());
    }
}
