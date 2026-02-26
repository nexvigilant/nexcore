// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Layer 14: Irreversibility Auditor (STOS-IR)
//!
//! **Dominant Primitive**: ∝ (Irreversibility)
//!
//! Tracks irreversible transitions, maintains audit trails,
//! and enforces point-of-no-return semantics.
//!
//! ## Tier Classification
//!
//! `IrreversibilityAuditor` is T2-C (∝ + π + σ) — irreversibility, persistence, sequence.

use alloc::collections::{BTreeMap, VecDeque};
use alloc::string::String;
use alloc::vec::Vec;

use super::state_registry::StateId;
use super::transition_engine::TransitionId;
use crate::MachineId;

/// An audit entry.
#[derive(Debug, Clone)]
pub struct AuditEntry {
    /// Entry ID.
    pub id: u64,
    /// Machine ID.
    pub machine_id: MachineId,
    /// Transition that was executed.
    pub transition_id: TransitionId,
    /// Source state.
    pub from_state: StateId,
    /// Target state.
    pub to_state: StateId,
    /// Timestamp.
    pub timestamp: u64,
    /// Whether the transition is irreversible.
    pub irreversible: bool,
    /// Audit reason/description.
    pub reason: Option<String>,
    /// Hash of the entry (for tamper detection).
    pub hash: u64,
}

impl AuditEntry {
    /// Create a new audit entry.
    #[must_use]
    pub fn new(
        id: u64,
        machine_id: MachineId,
        transition_id: TransitionId,
        from_state: StateId,
        to_state: StateId,
        timestamp: u64,
    ) -> Self {
        let hash = Self::compute_hash(
            id,
            machine_id,
            transition_id,
            from_state,
            to_state,
            timestamp,
        );
        Self {
            id,
            machine_id,
            transition_id,
            from_state,
            to_state,
            timestamp,
            irreversible: false,
            reason: None,
            hash,
        }
    }

    /// Mark as irreversible.
    #[must_use]
    pub fn irreversible(mut self) -> Self {
        self.irreversible = true;
        self
    }

    /// With reason.
    #[must_use]
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    /// Compute hash for tamper detection.
    fn compute_hash(
        id: u64,
        machine_id: MachineId,
        transition_id: TransitionId,
        from_state: StateId,
        to_state: StateId,
        timestamp: u64,
    ) -> u64 {
        // Simple hash combining all fields (cast u32 to u64)
        let mut hash = 0u64;
        hash = hash.wrapping_add(id.wrapping_mul(31));
        hash = hash.wrapping_add(machine_id.wrapping_mul(37));
        hash = hash.wrapping_add(u64::from(transition_id).wrapping_mul(41));
        hash = hash.wrapping_add(u64::from(from_state).wrapping_mul(43));
        hash = hash.wrapping_add(u64::from(to_state).wrapping_mul(47));
        hash = hash.wrapping_add(timestamp.wrapping_mul(53));
        hash
    }

    /// Verify entry integrity.
    #[must_use]
    pub fn verify(&self) -> bool {
        let expected = Self::compute_hash(
            self.id,
            self.machine_id,
            self.transition_id,
            self.from_state,
            self.to_state,
            self.timestamp,
        );
        self.hash == expected
    }
}

/// Irreversibility level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IrreversibilityLevel {
    /// Fully reversible.
    Reversible = 0,
    /// Soft irreversible (can be reversed with effort).
    Soft = 1,
    /// Hard irreversible (requires special authority).
    Hard = 2,
    /// Permanent (cannot be reversed).
    Permanent = 3,
}

/// The irreversibility auditor.
///
/// ## Tier: T2-C (∝ + π + σ)
///
/// Dominant primitive: ∝ (Irreversibility)
#[derive(Debug, Clone)]
pub struct IrreversibilityAuditor {
    /// Audit trail (VecDeque for O(1) front removal during pruning).
    trail: VecDeque<AuditEntry>,
    /// Irreversible transitions registry.
    irreversible_transitions: BTreeMap<TransitionId, IrreversibilityLevel>,
    /// Irreversible states (once entered, cannot leave).
    absorbing_states: BTreeMap<StateId, IrreversibilityLevel>,
    /// Counter.
    counter: u64,
    /// Current timestamp.
    timestamp: u64,
    /// Maximum trail length.
    max_trail_length: usize,
}

impl IrreversibilityAuditor {
    /// Create a new auditor.
    #[must_use]
    pub fn new() -> Self {
        Self {
            trail: VecDeque::new(),
            irreversible_transitions: BTreeMap::new(),
            absorbing_states: BTreeMap::new(),
            counter: 0,
            timestamp: 0,
            max_trail_length: 10000,
        }
    }

    /// Set current timestamp.
    pub fn set_timestamp(&mut self, timestamp: u64) {
        self.timestamp = timestamp;
    }

    /// Register an irreversible transition.
    pub fn register_irreversible_transition(
        &mut self,
        transition_id: TransitionId,
        level: IrreversibilityLevel,
    ) {
        self.irreversible_transitions.insert(transition_id, level);
    }

    /// Register an absorbing (irreversible) state.
    pub fn register_absorbing_state(&mut self, state: StateId, level: IrreversibilityLevel) {
        self.absorbing_states.insert(state, level);
    }

    /// Check if a transition is irreversible.
    #[must_use]
    pub fn is_transition_irreversible(&self, transition_id: TransitionId) -> bool {
        self.irreversible_transitions.contains_key(&transition_id)
    }

    /// Get irreversibility level of a transition.
    #[must_use]
    pub fn transition_level(&self, transition_id: TransitionId) -> Option<IrreversibilityLevel> {
        self.irreversible_transitions.get(&transition_id).copied()
    }

    /// Check if a state is absorbing.
    #[must_use]
    pub fn is_state_absorbing(&self, state: StateId) -> bool {
        self.absorbing_states.contains_key(&state)
    }

    /// Get absorbing level of a state.
    #[must_use]
    pub fn state_absorbing_level(&self, state: StateId) -> Option<IrreversibilityLevel> {
        self.absorbing_states.get(&state).copied()
    }

    /// Record a transition in the audit trail.
    pub fn record(
        &mut self,
        machine_id: MachineId,
        transition_id: TransitionId,
        from_state: StateId,
        to_state: StateId,
    ) -> u64 {
        self.counter = self.counter.saturating_add(1);
        self.timestamp = self.timestamp.saturating_add(1);

        let is_irreversible =
            self.is_transition_irreversible(transition_id) || self.is_state_absorbing(to_state);

        let mut entry = AuditEntry::new(
            self.counter,
            machine_id,
            transition_id,
            from_state,
            to_state,
            self.timestamp,
        );

        if is_irreversible {
            entry = entry.irreversible();
        }

        self.trail.push_back(entry);

        // Prune oldest if over limit — O(1) per removal with VecDeque
        while self.trail.len() > self.max_trail_length {
            self.trail.pop_front();
        }

        self.counter
    }

    /// Record with reason.
    pub fn record_with_reason(
        &mut self,
        machine_id: MachineId,
        transition_id: TransitionId,
        from_state: StateId,
        to_state: StateId,
        reason: impl Into<String>,
    ) -> u64 {
        let id = self.record(machine_id, transition_id, from_state, to_state);

        // Add reason to the entry we just added
        if let Some(entry) = self.trail.back_mut() {
            entry.reason = Some(reason.into());
        }

        id
    }

    /// Get the full audit trail as a contiguous slice pair (VecDeque may split).
    #[must_use]
    pub fn trail(&self) -> (&[AuditEntry], &[AuditEntry]) {
        self.trail.as_slices()
    }

    /// Get trail for a specific machine.
    #[must_use]
    pub fn trail_for_machine(&self, machine_id: MachineId) -> Vec<&AuditEntry> {
        self.trail
            .iter()
            .filter(|e| e.machine_id == machine_id)
            .collect()
    }

    /// Get irreversible entries only.
    #[must_use]
    pub fn irreversible_entries(&self) -> Vec<&AuditEntry> {
        self.trail.iter().filter(|e| e.irreversible).collect()
    }

    /// Get entry by ID.
    #[must_use]
    pub fn get_entry(&self, id: u64) -> Option<&AuditEntry> {
        self.trail.iter().find(|e| e.id == id)
    }

    /// Verify all entries.
    #[must_use]
    pub fn verify_all(&self) -> bool {
        self.trail.iter().all(AuditEntry::verify)
    }

    /// Trail length.
    #[must_use]
    pub fn len(&self) -> usize {
        self.trail.len()
    }

    /// Whether trail is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.trail.is_empty()
    }

    /// Count of irreversible entries.
    #[must_use]
    pub fn irreversible_count(&self) -> usize {
        self.trail.iter().filter(|e| e.irreversible).count()
    }

    /// Clear the trail.
    pub fn clear(&mut self) {
        self.trail.clear();
    }
}

impl Default for IrreversibilityAuditor {
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
    fn test_record_audit() {
        let mut auditor = IrreversibilityAuditor::new();

        let id = auditor.record(1, 0, 0, 1);
        assert!(id > 0);
        assert_eq!(auditor.len(), 1);
    }

    #[test]
    fn test_irreversible_transition() {
        let mut auditor = IrreversibilityAuditor::new();

        auditor.register_irreversible_transition(5, IrreversibilityLevel::Permanent);
        assert!(auditor.is_transition_irreversible(5));

        auditor.record(1, 5, 0, 1);
        let entries = auditor.irreversible_entries();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_absorbing_state() {
        let mut auditor = IrreversibilityAuditor::new();

        auditor.register_absorbing_state(99, IrreversibilityLevel::Hard);
        assert!(auditor.is_state_absorbing(99));

        // Transition to absorbing state
        auditor.record(1, 0, 0, 99);
        let entries = auditor.irreversible_entries();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_entry_verification() {
        let mut auditor = IrreversibilityAuditor::new();

        auditor.record(1, 0, 0, 1);
        auditor.record(1, 1, 1, 2);

        assert!(auditor.verify_all());
    }

    #[test]
    fn test_trail_for_machine() {
        let mut auditor = IrreversibilityAuditor::new();

        auditor.record(1, 0, 0, 1);
        auditor.record(2, 0, 0, 1);
        auditor.record(1, 1, 1, 2);

        let machine_1_trail = auditor.trail_for_machine(1);
        assert_eq!(machine_1_trail.len(), 2);
    }

    #[test]
    fn test_irreversibility_levels() {
        let mut auditor = IrreversibilityAuditor::new();

        auditor.register_irreversible_transition(1, IrreversibilityLevel::Soft);
        auditor.register_irreversible_transition(2, IrreversibilityLevel::Hard);
        auditor.register_irreversible_transition(3, IrreversibilityLevel::Permanent);

        assert_eq!(
            auditor.transition_level(1),
            Some(IrreversibilityLevel::Soft)
        );
        assert_eq!(
            auditor.transition_level(2),
            Some(IrreversibilityLevel::Hard)
        );
        assert_eq!(
            auditor.transition_level(3),
            Some(IrreversibilityLevel::Permanent)
        );
    }
}
