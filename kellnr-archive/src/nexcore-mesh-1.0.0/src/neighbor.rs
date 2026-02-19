//! Neighbor management with circuit breaker protection.
//!
//! ## Primitive Foundation
//! - `Neighbor`: ς (State) + ν (Frequency) + ∃ (Existence) + κ (Comparison)
//! - `NeighborRegistry`: μ (Mapping) — DashMap of node_id -> Neighbor

use crate::error::MeshError;
use crate::topology::RouteQuality;
use dashmap::DashMap;
use nexcore_primitives::transfer::{BreakerState, CircuitBreaker};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// ============================================================================
// Neighbor — A directly connected mesh peer
// ============================================================================

/// A directly connected mesh peer with health tracking.
///
/// Tier: T2-C | Dominant: ς (State)
///
/// Each neighbor embeds a `CircuitBreaker` (from transfer primitives)
/// that trips after repeated failures, preventing cascade failures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Neighbor {
    /// Unique identifier of this neighbor node
    pub node_id: String,
    /// Last known route quality to this neighbor
    pub quality: RouteQuality,
    /// Circuit breaker state for this link (not serialized — reconstructed on deser)
    #[serde(skip, default = "default_breaker")]
    pub breaker: CircuitBreaker,
    /// Number of heartbeats received (ν Frequency)
    pub heartbeat_count: u64,
    /// Whether this neighbor has been verified (∃ Existence)
    pub verified: bool,
}

fn default_breaker() -> CircuitBreaker {
    CircuitBreaker::new(3, 2)
}

impl Neighbor {
    /// Create a new neighbor with default circuit breaker (3 failures to trip, 2 to recover).
    pub fn new(node_id: impl Into<String>, quality: RouteQuality) -> Self {
        Self {
            node_id: node_id.into(),
            quality,
            breaker: CircuitBreaker::new(3, 2),
            heartbeat_count: 0,
            verified: false,
        }
    }

    /// Create a neighbor with custom circuit breaker thresholds.
    pub fn with_breaker(
        node_id: impl Into<String>,
        quality: RouteQuality,
        failure_threshold: u64,
        recovery_threshold: u64,
    ) -> Self {
        Self {
            node_id: node_id.into(),
            quality,
            breaker: CircuitBreaker::new(failure_threshold, recovery_threshold),
            heartbeat_count: 0,
            verified: false,
        }
    }

    /// Whether this neighbor is reachable (circuit breaker not open).
    pub fn is_reachable(&self) -> bool {
        self.breaker.is_allowing()
    }

    /// Record a successful communication with this neighbor.
    pub fn record_success(&mut self) {
        self.breaker.record_success();
        self.heartbeat_count += 1;
        self.verified = true;
    }

    /// Record a failed communication with this neighbor.
    pub fn record_failure(&mut self) {
        self.breaker.record_failure();
    }

    /// Attempt to reset the circuit breaker (Open -> HalfOpen).
    pub fn attempt_reset(&mut self) {
        self.breaker.attempt_reset();
    }

    /// Whether the circuit breaker is currently open (link down).
    pub fn is_circuit_open(&self) -> bool {
        self.breaker.state == BreakerState::Open
    }
}

// ============================================================================
// NeighborRegistry — Concurrent neighbor store
// ============================================================================

/// Thread-safe registry of mesh neighbors using DashMap.
///
/// Tier: T3 | Dominant: μ (Mapping)
///
/// Provides O(1) lookup, insertion, and removal of neighbors.
/// Enforces a maximum capacity to prevent unbounded growth.
#[derive(Debug, Clone)]
pub struct NeighborRegistry {
    /// Concurrent map of node_id -> Neighbor
    neighbors: Arc<DashMap<String, Neighbor>>,
    /// Maximum number of neighbors allowed
    max_capacity: usize,
}

impl NeighborRegistry {
    /// Create a new registry with the given capacity limit.
    pub fn new(max_capacity: usize) -> Self {
        Self {
            neighbors: Arc::new(DashMap::new()),
            max_capacity: max_capacity.max(1),
        }
    }

    /// Add a neighbor. Returns error if capacity exceeded.
    pub fn add(&self, neighbor: Neighbor) -> Result<(), MeshError> {
        if self.neighbors.len() >= self.max_capacity
            && !self.neighbors.contains_key(&neighbor.node_id)
        {
            return Err(MeshError::NeighborCapacityExceeded(self.max_capacity));
        }
        self.neighbors.insert(neighbor.node_id.clone(), neighbor);
        Ok(())
    }

    /// Remove a neighbor by ID. Returns the removed neighbor if it existed.
    pub fn remove(&self, node_id: &str) -> Option<Neighbor> {
        self.neighbors.remove(node_id).map(|(_, v)| v)
    }

    /// Get a clone of a neighbor by ID.
    pub fn get(&self, node_id: &str) -> Option<Neighbor> {
        self.neighbors.get(node_id).map(|r| r.clone())
    }

    /// Check if a neighbor exists.
    pub fn contains(&self, node_id: &str) -> bool {
        self.neighbors.contains_key(node_id)
    }

    /// Number of registered neighbors.
    pub fn len(&self) -> usize {
        self.neighbors.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.neighbors.is_empty()
    }

    /// Get all reachable neighbor IDs (circuit breaker not open).
    pub fn reachable_ids(&self) -> Vec<String> {
        self.neighbors
            .iter()
            .filter(|r| r.value().is_reachable())
            .map(|r| r.key().clone())
            .collect()
    }

    /// Get all neighbor IDs.
    pub fn all_ids(&self) -> Vec<String> {
        self.neighbors.iter().map(|r| r.key().clone()).collect()
    }

    /// Record success for a specific neighbor.
    pub fn record_success(&self, node_id: &str) {
        if let Some(mut entry) = self.neighbors.get_mut(node_id) {
            entry.record_success();
        }
    }

    /// Record failure for a specific neighbor.
    pub fn record_failure(&self, node_id: &str) {
        if let Some(mut entry) = self.neighbors.get_mut(node_id) {
            entry.record_failure();
        }
    }

    /// Attempt reset on all open circuit breakers.
    pub fn attempt_reset_all(&self) {
        for mut entry in self.neighbors.iter_mut() {
            if entry.is_circuit_open() {
                entry.attempt_reset();
            }
        }
    }

    /// Get the best quality neighbor among reachable ones.
    pub fn best_neighbor(&self) -> Option<Neighbor> {
        self.neighbors
            .iter()
            .filter(|r| r.value().is_reachable())
            .max_by(|a, b| {
                a.value()
                    .quality
                    .partial_cmp(&b.value().quality)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|r| r.value().clone())
    }

    /// Current capacity utilization (0.0 to 1.0).
    pub fn utilization(&self) -> f64 {
        self.neighbors.len() as f64 / self.max_capacity as f64
    }

    /// Create a serializable snapshot of all neighbors.
    pub fn snapshot(&self) -> Vec<Neighbor> {
        self.neighbors.iter().map(|r| r.value().clone()).collect()
    }

    /// Restore neighbors from a snapshot.
    ///
    /// Clears the current registry and repopulates from the snapshot data.
    /// Ignores capacity limits during restore (trusted snapshot).
    pub fn restore(&self, snapshot: &[Neighbor]) {
        self.neighbors.clear();
        for neighbor in snapshot {
            self.neighbors
                .insert(neighbor.node_id.clone(), neighbor.clone());
        }
    }

    /// Get the max capacity.
    pub fn max_capacity(&self) -> usize {
        self.max_capacity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_quality(reliability: f64) -> RouteQuality {
        RouteQuality::new(50.0, reliability, 1)
    }

    #[test]
    fn neighbor_new_defaults() {
        let n = Neighbor::new("node-1", make_quality(0.95));
        assert_eq!(n.node_id, "node-1");
        assert!(n.is_reachable());
        assert!(!n.verified);
        assert_eq!(n.heartbeat_count, 0);
    }

    #[test]
    fn neighbor_success_verifies() {
        let mut n = Neighbor::new("node-1", make_quality(0.95));
        n.record_success();
        assert!(n.verified);
        assert_eq!(n.heartbeat_count, 1);
    }

    #[test]
    fn neighbor_failure_trips_breaker() {
        let mut n = Neighbor::new("node-1", make_quality(0.95));
        // Default threshold is 3 failures
        n.record_failure();
        assert!(n.is_reachable());
        n.record_failure();
        assert!(n.is_reachable());
        n.record_failure();
        assert!(!n.is_reachable());
        assert!(n.is_circuit_open());
    }

    #[test]
    fn neighbor_reset_allows_probe() {
        let mut n = Neighbor::new("node-1", make_quality(0.95));
        for _ in 0..3 {
            n.record_failure();
        }
        assert!(!n.is_reachable());
        n.attempt_reset();
        assert!(n.is_reachable()); // HalfOpen allows
    }

    #[test]
    fn neighbor_custom_breaker() {
        let mut n = Neighbor::with_breaker("node-1", make_quality(0.99), 5, 3);
        for _ in 0..4 {
            n.record_failure();
        }
        assert!(n.is_reachable()); // threshold=5, only 4 failures
        n.record_failure();
        assert!(!n.is_reachable()); // now tripped
    }

    // ---------- NeighborRegistry tests ----------

    #[test]
    fn registry_add_and_get() {
        let reg = NeighborRegistry::new(10);
        let n = Neighbor::new("node-a", make_quality(0.9));
        assert!(reg.add(n).is_ok());
        assert!(reg.contains("node-a"));
        assert_eq!(reg.len(), 1);

        let got = reg.get("node-a");
        assert!(got.is_some());
        assert_eq!(got.as_ref().map(|n| n.node_id.as_str()), Some("node-a"));
    }

    #[test]
    fn registry_capacity_limit() {
        let reg = NeighborRegistry::new(2);
        assert!(reg.add(Neighbor::new("a", make_quality(0.9))).is_ok());
        assert!(reg.add(Neighbor::new("b", make_quality(0.9))).is_ok());
        let result = reg.add(Neighbor::new("c", make_quality(0.9)));
        assert!(result.is_err());
        if let Err(MeshError::NeighborCapacityExceeded(cap)) = result {
            assert_eq!(cap, 2);
        }
    }

    #[test]
    fn registry_upsert_same_id_no_capacity_error() {
        let reg = NeighborRegistry::new(1);
        assert!(reg.add(Neighbor::new("a", make_quality(0.9))).is_ok());
        // Re-adding same ID should succeed (upsert)
        assert!(reg.add(Neighbor::new("a", make_quality(0.99))).is_ok());
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn registry_remove() {
        let reg = NeighborRegistry::new(10);
        assert!(reg.add(Neighbor::new("x", make_quality(0.9))).is_ok());
        let removed = reg.remove("x");
        assert!(removed.is_some());
        assert!(reg.is_empty());
    }

    #[test]
    fn registry_reachable_ids() {
        let reg = NeighborRegistry::new(10);
        assert!(reg.add(Neighbor::new("a", make_quality(0.9))).is_ok());
        assert!(reg.add(Neighbor::new("b", make_quality(0.9))).is_ok());
        // Trip breaker on "a"
        for _ in 0..3 {
            reg.record_failure("a");
        }
        let reachable = reg.reachable_ids();
        assert_eq!(reachable.len(), 1);
        assert!(reachable.contains(&"b".to_string()));
    }

    #[test]
    fn registry_best_neighbor() {
        let reg = NeighborRegistry::new(10);
        assert!(reg.add(Neighbor::new("low", make_quality(0.3))).is_ok());
        assert!(reg.add(Neighbor::new("high", make_quality(0.99))).is_ok());
        let best = reg.best_neighbor();
        assert!(best.is_some());
        assert_eq!(best.as_ref().map(|n| n.node_id.as_str()), Some("high"));
    }

    #[test]
    fn registry_utilization() {
        let reg = NeighborRegistry::new(4);
        assert!((reg.utilization() - 0.0).abs() < f64::EPSILON);
        assert!(reg.add(Neighbor::new("a", make_quality(0.9))).is_ok());
        assert!(reg.add(Neighbor::new("b", make_quality(0.9))).is_ok());
        assert!((reg.utilization() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn registry_attempt_reset_all() {
        let reg = NeighborRegistry::new(10);
        assert!(reg.add(Neighbor::new("a", make_quality(0.9))).is_ok());
        assert!(reg.add(Neighbor::new("b", make_quality(0.9))).is_ok());
        for _ in 0..3 {
            reg.record_failure("a");
            reg.record_failure("b");
        }
        assert_eq!(reg.reachable_ids().len(), 0);
        reg.attempt_reset_all();
        assert_eq!(reg.reachable_ids().len(), 2); // HalfOpen
    }

    #[test]
    fn registry_record_success() {
        let reg = NeighborRegistry::new(10);
        assert!(reg.add(Neighbor::new("x", make_quality(0.9))).is_ok());
        reg.record_success("x");
        let n = reg.get("x");
        assert!(n.is_some());
        assert_eq!(n.as_ref().map(|n| n.heartbeat_count), Some(1));
        assert_eq!(n.as_ref().map(|n| n.verified), Some(true));
    }

    #[test]
    fn registry_all_ids() {
        let reg = NeighborRegistry::new(10);
        assert!(reg.add(Neighbor::new("a", make_quality(0.9))).is_ok());
        assert!(reg.add(Neighbor::new("b", make_quality(0.9))).is_ok());
        let ids = reg.all_ids();
        assert_eq!(ids.len(), 2);
    }
}
