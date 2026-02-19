// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Layer 7: Recursion Detector (STOS-RC)
//!
//! **Dominant Primitive**: ρ (Recursion)
//!
//! Detects cycles and recursion in state machines, preventing
//! infinite loops and detecting liveness issues.
//!
//! ## Tier Classification
//!
//! `RecursionDetector` is T2-P (ρ + σ) — recursion, sequence.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;

use super::state_registry::StateId;
use crate::MachineId;

/// Information about a detected cycle.
#[derive(Debug, Clone)]
pub struct CycleInfo {
    /// States in the cycle (in order).
    pub states: Vec<StateId>,
    /// Whether the cycle is intentional (e.g., retry loops).
    pub intentional: bool,
    /// Detection timestamp.
    pub detected_at: u64,
}

impl CycleInfo {
    /// Cycle length.
    #[must_use]
    pub fn len(&self) -> usize {
        self.states.len()
    }

    /// Whether cycle is empty (shouldn't happen).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }
}

/// The recursion detector.
///
/// ## Tier: T2-P (ρ + σ)
///
/// Dominant primitive: ρ (Recursion)
#[derive(Debug, Clone)]
pub struct RecursionDetector {
    /// Machine ID.
    machine_id: MachineId,
    /// Adjacency list (state → successors).
    adjacency: BTreeMap<StateId, BTreeSet<StateId>>,
    /// Detected cycles.
    cycles: Vec<CycleInfo>,
    /// Visit path for DFS.
    visit_path: Vec<StateId>,
    /// Detection counter.
    counter: u64,
    /// Maximum recursion depth allowed.
    max_depth: usize,
}

impl RecursionDetector {
    /// Create a new recursion detector.
    #[must_use]
    pub fn new(machine_id: MachineId) -> Self {
        Self {
            machine_id,
            adjacency: BTreeMap::new(),
            cycles: Vec::new(),
            visit_path: Vec::new(),
            counter: 0,
            max_depth: 100,
        }
    }

    /// Add a transition edge.
    pub fn add_edge(&mut self, from: StateId, to: StateId) {
        self.adjacency.entry(from).or_default().insert(to);
    }

    /// Remove all edges.
    pub fn clear_edges(&mut self) {
        self.adjacency.clear();
    }

    /// Check if there's a path that would create a cycle from current state.
    #[must_use]
    pub fn would_cycle(&self, from: StateId, to: StateId) -> bool {
        // Would cycle if 'to' can reach 'from'
        let mut visited = BTreeSet::new();
        let mut stack = vec![to];

        while let Some(state) = stack.pop() {
            if state == from {
                return true;
            }
            if visited.insert(state) {
                if let Some(successors) = self.adjacency.get(&state) {
                    stack.extend(successors.iter().copied());
                }
            }
        }
        false
    }

    /// Detect all cycles using DFS.
    pub fn detect_cycles(&mut self) -> Vec<CycleInfo> {
        self.cycles.clear();

        let states: Vec<StateId> = self.adjacency.keys().copied().collect();
        let mut visited = BTreeSet::new();

        for start in states {
            if !visited.contains(&start) {
                self.visit_path.clear();
                self.dfs_detect(start, &mut visited, &mut BTreeSet::new());
            }
        }

        self.cycles.clone()
    }

    /// DFS helper for cycle detection.
    fn dfs_detect(
        &mut self,
        state: StateId,
        visited: &mut BTreeSet<StateId>,
        rec_stack: &mut BTreeSet<StateId>,
    ) {
        visited.insert(state);
        rec_stack.insert(state);
        self.visit_path.push(state);

        if let Some(successors) = self.adjacency.get(&state).cloned() {
            for next in successors {
                if !visited.contains(&next) {
                    self.dfs_detect(next, visited, rec_stack);
                } else if rec_stack.contains(&next) {
                    // Found a cycle - extract it
                    if let Some(cycle_start) = self.visit_path.iter().position(|&s| s == next) {
                        let cycle_states: Vec<StateId> = self.visit_path[cycle_start..].to_vec();
                        self.counter = self.counter.saturating_add(1);
                        self.cycles.push(CycleInfo {
                            states: cycle_states,
                            intentional: false,
                            detected_at: self.counter,
                        });
                    }
                }
            }
        }

        self.visit_path.pop();
        rec_stack.remove(&state);
    }

    /// Check if a state is part of any cycle.
    #[must_use]
    pub fn is_in_cycle(&self, state: StateId) -> bool {
        self.cycles.iter().any(|c| c.states.contains(&state))
    }

    /// Get all detected cycles.
    #[must_use]
    pub fn cycles(&self) -> &[CycleInfo] {
        &self.cycles
    }

    /// Check recursion depth during execution.
    #[must_use]
    pub fn check_depth(&self, current_depth: usize) -> bool {
        current_depth <= self.max_depth
    }

    /// Set maximum recursion depth.
    pub fn set_max_depth(&mut self, depth: usize) {
        self.max_depth = depth;
    }

    /// Mark a cycle as intentional.
    pub fn mark_intentional(&mut self, cycle_index: usize) {
        if let Some(cycle) = self.cycles.get_mut(cycle_index) {
            cycle.intentional = true;
        }
    }

    /// Count of unintentional cycles.
    #[must_use]
    pub fn unintentional_cycle_count(&self) -> usize {
        self.cycles.iter().filter(|c| !c.intentional).count()
    }
}

// ═══════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_cycles() {
        let mut detector = RecursionDetector::new(1);
        detector.add_edge(0, 1);
        detector.add_edge(1, 2);
        detector.add_edge(2, 3);

        let cycles = detector.detect_cycles();
        assert!(cycles.is_empty());
    }

    #[test]
    fn test_simple_cycle() {
        let mut detector = RecursionDetector::new(1);
        detector.add_edge(0, 1);
        detector.add_edge(1, 2);
        detector.add_edge(2, 0); // Cycle back

        let cycles = detector.detect_cycles();
        assert!(!cycles.is_empty());
    }

    #[test]
    fn test_self_loop() {
        let mut detector = RecursionDetector::new(1);
        detector.add_edge(0, 0); // Self-loop

        let cycles = detector.detect_cycles();
        assert!(!cycles.is_empty());
        assert_eq!(cycles[0].len(), 1);
    }

    #[test]
    fn test_would_cycle() {
        let mut detector = RecursionDetector::new(1);
        detector.add_edge(0, 1);
        detector.add_edge(1, 2);

        // Adding 2→0 would create a cycle (0→1→2→0)
        assert!(detector.would_cycle(2, 0));

        // Adding 2→3 would not
        assert!(!detector.would_cycle(2, 3));
    }

    #[test]
    fn test_intentional_marking() {
        let mut detector = RecursionDetector::new(1);
        detector.add_edge(0, 1);
        detector.add_edge(1, 0);

        detector.detect_cycles();
        assert_eq!(detector.unintentional_cycle_count(), 1);

        detector.mark_intentional(0);
        assert_eq!(detector.unintentional_cycle_count(), 0);
    }
}
