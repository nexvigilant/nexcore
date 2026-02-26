// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Layer 11: Aggregate Coordinator (STOS-AG)
//!
//! **Dominant Primitive**: Σ (Sum)
//!
//! Coordinates operations across multiple state machines,
//! aggregating results and managing distributed state.
//!
//! ## Tier Classification
//!
//! `AggregateCoordinator` is T2-C (Σ + ς + μ) — sum, state, mapping.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use super::state_registry::StateId;
use crate::MachineId;

/// Status of a machine in the aggregate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MachineStatus {
    /// Machine is active.
    Active,
    /// Machine is idle.
    Idle,
    /// Machine is paused.
    Paused,
    /// Machine is terminated.
    Terminated,
    /// Machine has an error.
    Error,
}

/// Summary of a machine's state.
#[derive(Debug, Clone)]
pub struct MachineSummary {
    /// Machine ID.
    pub machine_id: MachineId,
    /// Current state.
    pub current_state: StateId,
    /// Machine status.
    pub status: MachineStatus,
    /// Transition count.
    pub transition_count: u64,
}

impl MachineSummary {
    /// Create a new summary.
    #[must_use]
    pub fn new(machine_id: MachineId, current_state: StateId) -> Self {
        Self {
            machine_id,
            current_state,
            status: MachineStatus::Active,
            transition_count: 0,
        }
    }

    /// With status.
    #[must_use]
    pub fn with_status(mut self, status: MachineStatus) -> Self {
        self.status = status;
        self
    }

    /// With transition count.
    #[must_use]
    pub fn with_transitions(mut self, count: u64) -> Self {
        self.transition_count = count;
        self
    }
}

/// Aggregate statistics.
#[derive(Debug, Clone, Default)]
pub struct AggregateStats {
    /// Total machines.
    pub total_machines: usize,
    /// Active machines.
    pub active_count: usize,
    /// Idle machines.
    pub idle_count: usize,
    /// Paused machines.
    pub paused_count: usize,
    /// Terminated machines.
    pub terminated_count: usize,
    /// Error machines.
    pub error_count: usize,
    /// Total transitions across all machines.
    pub total_transitions: u64,
}

impl AggregateStats {
    /// Compute from summaries.
    #[must_use]
    pub fn from_summaries(summaries: &[MachineSummary]) -> Self {
        let mut stats = Self {
            total_machines: summaries.len(),
            ..Self::default()
        };

        for summary in summaries {
            stats.total_transitions = stats
                .total_transitions
                .saturating_add(summary.transition_count);
            match summary.status {
                MachineStatus::Active => stats.active_count += 1,
                MachineStatus::Idle => stats.idle_count += 1,
                MachineStatus::Paused => stats.paused_count += 1,
                MachineStatus::Terminated => stats.terminated_count += 1,
                MachineStatus::Error => stats.error_count += 1,
            }
        }

        stats
    }

    /// Percentage of healthy machines.
    #[must_use]
    #[allow(
        clippy::cast_precision_loss,
        reason = "usize -> f64 for percentage computation; precision loss is acceptable at realistic machine counts"
    )]
    pub fn health_percentage(&self) -> f64 {
        if self.total_machines == 0 {
            return 100.0;
        }
        let healthy = self.active_count + self.idle_count + self.paused_count;
        (healthy as f64 / self.total_machines as f64) * 100.0
    }
}

/// The aggregate coordinator.
///
/// ## Tier: T2-C (Σ + ς + μ)
///
/// Dominant primitive: Σ (Sum)
#[derive(Debug, Clone)]
pub struct AggregateCoordinator {
    /// Registered machines.
    machines: BTreeMap<MachineId, MachineSummary>,
    /// Machine groups.
    groups: BTreeMap<u64, Vec<MachineId>>,
    /// Group counter.
    group_counter: u64,
}

impl AggregateCoordinator {
    /// Create a new aggregate coordinator.
    #[must_use]
    pub fn new() -> Self {
        Self {
            machines: BTreeMap::new(),
            groups: BTreeMap::new(),
            group_counter: 0,
        }
    }

    /// Register a machine.
    pub fn register(&mut self, machine_id: MachineId, initial_state: StateId) {
        let summary = MachineSummary::new(machine_id, initial_state);
        self.machines.insert(machine_id, summary);
    }

    /// Update a machine's state.
    pub fn update_state(&mut self, machine_id: MachineId, new_state: StateId) {
        if let Some(summary) = self.machines.get_mut(&machine_id) {
            summary.current_state = new_state;
            summary.transition_count = summary.transition_count.saturating_add(1);
        }
    }

    /// Update a machine's status.
    pub fn update_status(&mut self, machine_id: MachineId, status: MachineStatus) {
        if let Some(summary) = self.machines.get_mut(&machine_id) {
            summary.status = status;
        }
    }

    /// Unregister a machine.
    pub fn unregister(&mut self, machine_id: MachineId) {
        self.machines.remove(&machine_id);
        // Remove from all groups
        for group in self.groups.values_mut() {
            group.retain(|&id: &MachineId| id != machine_id);
        }
    }

    /// Get a machine summary.
    #[must_use]
    pub fn get(&self, machine_id: MachineId) -> Option<&MachineSummary> {
        self.machines.get(&machine_id)
    }

    /// Get all summaries.
    #[must_use]
    pub fn all_summaries(&self) -> Vec<&MachineSummary> {
        self.machines.values().collect()
    }

    /// Create a group of machines.
    pub fn create_group(&mut self, machines: Vec<MachineId>) -> u64 {
        self.group_counter = self.group_counter.saturating_add(1);
        self.groups.insert(self.group_counter, machines);
        self.group_counter
    }

    /// Add machine to a group.
    pub fn add_to_group(&mut self, group_id: u64, machine_id: MachineId) {
        if let Some(group) = self.groups.get_mut(&group_id) {
            if !group.contains(&machine_id) {
                group.push(machine_id);
            }
        }
    }

    /// Remove machine from a group.
    pub fn remove_from_group(&mut self, group_id: u64, machine_id: MachineId) {
        if let Some(group) = self.groups.get_mut(&group_id) {
            group.retain(|&id: &MachineId| id != machine_id);
        }
    }

    /// Get machines in a group.
    #[must_use]
    pub fn group_machines(&self, group_id: u64) -> Option<&[MachineId]> {
        self.groups.get(&group_id).map(Vec::as_slice)
    }

    /// Get aggregate stats.
    #[must_use]
    pub fn stats(&self) -> AggregateStats {
        let summaries: Vec<MachineSummary> = self.machines.values().cloned().collect();
        AggregateStats::from_summaries(&summaries)
    }

    /// Get stats for a group.
    #[must_use]
    pub fn group_stats(&self, group_id: u64) -> Option<AggregateStats> {
        let machine_ids = self.groups.get(&group_id)?;
        let summaries: Vec<MachineSummary> = machine_ids
            .iter()
            .filter_map(|id| self.machines.get(id).cloned())
            .collect();
        Some(AggregateStats::from_summaries(&summaries))
    }

    /// Filter machines by status.
    #[must_use]
    pub fn by_status(&self, status: MachineStatus) -> Vec<MachineId> {
        self.machines
            .iter()
            .filter(|(_, s)| s.status == status)
            .map(|(&id, _)| id)
            .collect()
    }

    /// Filter machines by state.
    #[must_use]
    pub fn by_state(&self, state: StateId) -> Vec<MachineId> {
        self.machines
            .iter()
            .filter(|(_, s)| s.current_state == state)
            .map(|(&id, _)| id)
            .collect()
    }

    /// Total machine count.
    #[must_use]
    pub fn machine_count(&self) -> usize {
        self.machines.len()
    }

    /// Total group count.
    #[must_use]
    pub fn group_count(&self) -> usize {
        self.groups.len()
    }
}

impl Default for AggregateCoordinator {
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
    fn test_register_machine() {
        let mut coordinator = AggregateCoordinator::new();

        coordinator.register(1, 0);
        coordinator.register(2, 0);

        assert_eq!(coordinator.machine_count(), 2);
        assert!(coordinator.get(1).is_some());
    }

    #[test]
    fn test_update_state() {
        let mut coordinator = AggregateCoordinator::new();

        coordinator.register(1, 0);
        coordinator.update_state(1, 5);

        let summary = coordinator.get(1);
        assert!(summary.is_some());
        assert_eq!(summary.map(|s| s.current_state), Some(5));
        assert_eq!(summary.map(|s| s.transition_count), Some(1));
    }

    #[test]
    fn test_aggregate_stats() {
        let mut coordinator = AggregateCoordinator::new();

        coordinator.register(1, 0);
        coordinator.register(2, 0);
        coordinator.register(3, 0);

        coordinator.update_status(2, MachineStatus::Idle);
        coordinator.update_status(3, MachineStatus::Error);

        let stats = coordinator.stats();
        assert_eq!(stats.total_machines, 3);
        assert_eq!(stats.active_count, 1);
        assert_eq!(stats.idle_count, 1);
        assert_eq!(stats.error_count, 1);
    }

    #[test]
    fn test_groups() {
        let mut coordinator = AggregateCoordinator::new();

        coordinator.register(1, 0);
        coordinator.register(2, 0);
        coordinator.register(3, 0);

        let group_id = coordinator.create_group(vec![1, 2]);
        assert_eq!(
            coordinator.group_machines(group_id).map(|g| g.len()),
            Some(2)
        );

        coordinator.add_to_group(group_id, 3);
        assert_eq!(
            coordinator.group_machines(group_id).map(|g| g.len()),
            Some(3)
        );
    }

    #[test]
    fn test_filter_by_status() {
        let mut coordinator = AggregateCoordinator::new();

        coordinator.register(1, 0);
        coordinator.register(2, 0);
        coordinator.register(3, 0);

        coordinator.update_status(1, MachineStatus::Paused);
        coordinator.update_status(2, MachineStatus::Paused);

        let paused = coordinator.by_status(MachineStatus::Paused);
        assert_eq!(paused.len(), 2);
    }

    #[test]
    fn test_health_percentage() {
        let mut coordinator = AggregateCoordinator::new();

        coordinator.register(1, 0);
        coordinator.register(2, 0);
        coordinator.register(3, 0);
        coordinator.register(4, 0);

        coordinator.update_status(4, MachineStatus::Error);

        let stats = coordinator.stats();
        assert!((stats.health_percentage() - 75.0).abs() < 0.001);
    }
}
