// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Layer 9: Persist Store (STOS-PR)
//!
//! **Dominant Primitive**: π (Persistence)
//!
//! Handles state machine persistence, snapshots, and recovery.
//!
//! ## Tier Classification
//!
//! `PersistStore` is T2-C (π + ς + σ) — persistence, state, sequence.

use alloc::collections::{BTreeMap, VecDeque};
use alloc::string::String;

use super::state_registry::StateId;
use crate::MachineId;

/// A snapshot of machine state.
#[derive(Debug, Clone)]
pub struct Snapshot {
    /// Snapshot ID.
    pub id: u64,
    /// Machine ID.
    pub machine_id: MachineId,
    /// Current state at snapshot time.
    pub current_state: StateId,
    /// Transition count at snapshot time.
    pub transition_count: u64,
    /// Snapshot timestamp.
    pub timestamp: u64,
    /// Custom data.
    pub data: BTreeMap<String, String>,
}

impl Snapshot {
    /// Create a new snapshot.
    #[must_use]
    pub fn new(id: u64, machine_id: MachineId, current_state: StateId, timestamp: u64) -> Self {
        Self {
            id,
            machine_id,
            current_state,
            transition_count: 0,
            timestamp,
            data: BTreeMap::new(),
        }
    }

    /// Add transition count.
    #[must_use]
    pub fn with_transition_count(mut self, count: u64) -> Self {
        self.transition_count = count;
        self
    }

    /// Add custom data.
    #[must_use]
    pub fn with_data(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.data.insert(key.into(), value.into());
        self
    }
}

/// The persist store.
///
/// ## Tier: T2-C (π + ς + σ)
///
/// Dominant primitive: π (Persistence)
#[derive(Debug, Clone)]
pub struct PersistStore {
    /// Machine ID.
    machine_id: MachineId,
    /// Stored snapshots (VecDeque for O(1) front removal during pruning).
    snapshots: VecDeque<Snapshot>,
    /// Counter for snapshot IDs.
    counter: u64,
    /// Maximum snapshots to retain.
    max_snapshots: usize,
    /// Auto-snapshot interval (0 = disabled).
    auto_interval: u64,
    /// Transitions since last snapshot.
    since_last_snapshot: u64,
}

impl PersistStore {
    /// Create a new persist store.
    #[must_use]
    pub fn new(machine_id: MachineId) -> Self {
        Self {
            machine_id,
            snapshots: VecDeque::new(),
            counter: 0,
            max_snapshots: 100,
            auto_interval: 0,
            since_last_snapshot: 0,
        }
    }

    /// Set auto-snapshot interval.
    pub fn set_auto_interval(&mut self, interval: u64) {
        self.auto_interval = interval;
    }

    /// Set maximum snapshots.
    pub fn set_max_snapshots(&mut self, max: usize) {
        self.max_snapshots = max;
    }

    /// Create a snapshot.
    pub fn snapshot(&mut self, current_state: StateId, transition_count: u64) -> u64 {
        self.counter = self.counter.saturating_add(1);
        let snap = Snapshot::new(self.counter, self.machine_id, current_state, self.counter)
            .with_transition_count(transition_count);

        self.snapshots.push_back(snap);
        self.since_last_snapshot = 0;

        // Prune oldest if over limit — O(1) per removal with VecDeque
        while self.snapshots.len() > self.max_snapshots {
            self.snapshots.pop_front();
        }

        self.counter
    }

    /// Record a transition (for auto-snapshot).
    pub fn record_transition(&mut self) -> bool {
        self.since_last_snapshot = self.since_last_snapshot.saturating_add(1);

        // Check if auto-snapshot is due
        self.auto_interval > 0 && self.since_last_snapshot >= self.auto_interval
    }

    /// Get latest snapshot.
    #[must_use]
    pub fn latest(&self) -> Option<&Snapshot> {
        self.snapshots.back()
    }

    /// Get snapshot by ID.
    #[must_use]
    pub fn get(&self, id: u64) -> Option<&Snapshot> {
        self.snapshots.iter().find(|s| s.id == id)
    }

    /// Get all snapshots as a contiguous slice pair (VecDeque may split).
    #[must_use]
    pub fn all(&self) -> (&[Snapshot], &[Snapshot]) {
        self.snapshots.as_slices()
    }

    /// Get snapshots in time range.
    #[must_use]
    pub fn in_range(&self, from: u64, to: u64) -> alloc::vec::Vec<&Snapshot> {
        self.snapshots
            .iter()
            .filter(|s| s.timestamp >= from && s.timestamp <= to)
            .collect()
    }

    /// Snapshot count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.snapshots.len()
    }

    /// Whether no snapshots exist.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.snapshots.is_empty()
    }

    /// Clear all snapshots.
    pub fn clear(&mut self) {
        self.snapshots.clear();
    }

    /// Restore from a snapshot (returns state to restore to).
    #[must_use]
    pub fn restore(&self, snapshot_id: u64) -> Option<StateId> {
        self.get(snapshot_id).map(|s| s.current_state)
    }
}

// ═══════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_creation() {
        let mut store = PersistStore::new(1);

        let id = store.snapshot(5, 100);
        assert!(id > 0);
        assert_eq!(store.len(), 1);

        let snap = store.latest();
        assert!(snap.is_some());
        assert_eq!(snap.map(|s| s.current_state), Some(5));
    }

    #[test]
    fn test_snapshot_pruning() {
        let mut store = PersistStore::new(1);
        store.set_max_snapshots(3);

        store.snapshot(0, 0);
        store.snapshot(1, 1);
        store.snapshot(2, 2);
        store.snapshot(3, 3); // Should trigger pruning

        assert_eq!(store.len(), 3);
        // First snapshot should be gone
        assert!(store.get(1).is_none());
    }

    #[test]
    fn test_auto_snapshot() {
        let mut store = PersistStore::new(1);
        store.set_auto_interval(5);

        for _ in 0..4 {
            assert!(!store.record_transition());
        }
        // 5th transition should trigger
        assert!(store.record_transition());
    }

    #[test]
    fn test_restore() {
        let mut store = PersistStore::new(1);

        let id = store.snapshot(10, 50);
        let restored = store.restore(id);

        assert_eq!(restored, Some(10));
    }
}
