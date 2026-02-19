//! Topology primitives: addressing, paths, and route quality.
//!
//! ## Primitive Foundation
//! - `AddressExt`: λ (Location) + σ (Sequence)
//! - `Path`: σ (Sequence) + ∂ (Boundary) — hop sequence with TTL boundary
//! - `RouteQuality`: κ (Comparison) + N (Quantity) + ν (Frequency)

use nexcore_primitives::transfer::TopologicalAddress;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

// ============================================================================
// AddressExt — Extensions to TopologicalAddress for mesh networking
// ============================================================================

/// Extension trait adding mesh-specific operations to `TopologicalAddress`.
///
/// Tier: T2-P | Dominant: λ (Location)
pub trait AddressExt {
    /// Compute mesh distance (shared prefix length from root).
    /// Lower = topologically closer.
    fn mesh_distance(&self, other: &TopologicalAddress) -> usize;

    /// Whether two addresses share a direct parent (are siblings).
    fn is_neighbor(&self, other: &TopologicalAddress) -> bool;

    /// Return all ancestor addresses (parent, grandparent, ..., root).
    fn ancestors(&self) -> Vec<TopologicalAddress>;
}

impl AddressExt for TopologicalAddress {
    fn mesh_distance(&self, other: &TopologicalAddress) -> usize {
        let shared = self
            .segments
            .iter()
            .zip(&other.segments)
            .take_while(|(a, b)| a == b)
            .count();
        // Distance = (depth_a - shared) + (depth_b - shared)
        (self.depth() - shared) + (other.depth() - shared)
    }

    fn is_neighbor(&self, other: &TopologicalAddress) -> bool {
        if self.segments.is_empty() || other.segments.is_empty() {
            return false;
        }
        // Same depth, same parent prefix
        if self.depth() != other.depth() {
            return false;
        }
        let parent_len = self.depth().saturating_sub(1);
        self.segments[..parent_len] == other.segments[..parent_len]
    }

    fn ancestors(&self) -> Vec<TopologicalAddress> {
        let mut result = Vec::new();
        for i in (0..self.segments.len()).rev() {
            if i == 0 {
                // Root: empty segments
                continue;
            }
            result.push(TopologicalAddress::new(
                self.segments[..i].to_vec(),
                self.separator.clone(),
            ));
        }
        result
    }
}

// ============================================================================
// Path — A sequence of hops with TTL boundary
// ============================================================================

/// A mesh path: ordered sequence of node IDs representing a route.
///
/// Tier: T2-P | Dominant: σ (Sequence)
///
/// TTL (Time-To-Live) decrements at each hop, enforcing ∂ (Boundary)
/// on maximum path length to prevent infinite relay loops.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Path {
    /// Ordered sequence of node IDs from source to current position
    pub hops: Vec<String>,
    /// Maximum allowed hops before the message is dropped
    pub max_ttl: u8,
}

impl Path {
    /// Create a new path starting from a source node.
    pub fn new(source: impl Into<String>, max_ttl: u8) -> Self {
        Self {
            hops: vec![source.into()],
            max_ttl: max_ttl.max(1),
        }
    }

    /// Remaining hops before TTL expiry.
    pub fn remaining_ttl(&self) -> u8 {
        let used = self.hops.len().saturating_sub(1) as u8;
        self.max_ttl.saturating_sub(used)
    }

    /// Whether the path has expired (no remaining TTL).
    pub fn is_expired(&self) -> bool {
        self.remaining_ttl() == 0
    }

    /// Number of hops taken so far.
    pub fn hop_count(&self) -> usize {
        self.hops.len().saturating_sub(1)
    }

    /// Add a hop to the path. Returns false if TTL would be exceeded.
    pub fn add_hop(&mut self, node_id: impl Into<String>) -> bool {
        if self.is_expired() {
            return false;
        }
        self.hops.push(node_id.into());
        true
    }

    /// Whether the path contains a loop (duplicate node ID).
    pub fn has_loop(&self) -> bool {
        let mut seen = std::collections::HashSet::new();
        self.hops.iter().any(|h| !seen.insert(h))
    }

    /// The source node (first hop).
    pub fn source(&self) -> Option<&str> {
        self.hops.first().map(|s| s.as_str())
    }

    /// The current node (last hop).
    pub fn current(&self) -> Option<&str> {
        self.hops.last().map(|s| s.as_str())
    }
}

// ============================================================================
// RouteQuality — Composite quality metric for route selection
// ============================================================================

/// Composite quality metric for comparing routes.
///
/// Tier: T2-C | Dominant: κ (Comparison)
///
/// Higher quality = better route. Used by `RoutingTable` to select
/// the best path to each destination.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteQuality {
    /// Latency estimate in milliseconds (lower is better)
    pub latency_ms: f64,
    /// Packet delivery ratio (0.0 = total loss, 1.0 = perfect delivery)
    pub reliability: f64,
    /// Number of hops (lower is better)
    pub hop_count: u8,
    /// Freshness: epoch seconds when this quality was last measured
    pub last_measured: u64,
}

impl RouteQuality {
    /// Create a new route quality measurement.
    pub fn new(latency_ms: f64, reliability: f64, hop_count: u8) -> Self {
        Self {
            latency_ms: latency_ms.max(0.0),
            reliability: reliability.clamp(0.0, 1.0),
            hop_count,
            last_measured: 0,
        }
    }

    /// Composite score (higher = better). Normalizes components.
    ///
    /// Score = reliability * (1.0 / (1.0 + latency_ms/1000)) * (1.0 / (1.0 + hop_count))
    pub fn score(&self) -> f64 {
        let latency_factor = 1.0 / (1.0 + self.latency_ms / 1000.0);
        let hop_factor = 1.0 / (1.0 + self.hop_count as f64);
        self.reliability * latency_factor * hop_factor
    }

    /// Whether this quality is considered usable (reliability > 0.1).
    pub fn is_usable(&self) -> bool {
        self.reliability > 0.1
    }
}

impl PartialEq for RouteQuality {
    fn eq(&self, other: &Self) -> bool {
        // Equality based on score with epsilon tolerance
        (self.score() - other.score()).abs() < f64::EPSILON
    }
}

impl PartialOrd for RouteQuality {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.score().partial_cmp(&other.score())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------- AddressExt tests ----------

    #[test]
    fn mesh_distance_same_node() {
        let a = TopologicalAddress::parse("mesh.region1.node1", ".");
        assert_eq!(a.mesh_distance(&a), 0);
    }

    #[test]
    fn mesh_distance_siblings() {
        let a = TopologicalAddress::parse("mesh.region1.node1", ".");
        let b = TopologicalAddress::parse("mesh.region1.node2", ".");
        assert_eq!(a.mesh_distance(&b), 2); // 1 up + 1 down
    }

    #[test]
    fn mesh_distance_different_regions() {
        let a = TopologicalAddress::parse("mesh.region1.node1", ".");
        let b = TopologicalAddress::parse("mesh.region2.node1", ".");
        assert_eq!(a.mesh_distance(&b), 4); // 2 up + 2 down
    }

    #[test]
    fn is_neighbor_siblings() {
        let a = TopologicalAddress::parse("mesh.r1.n1", ".");
        let b = TopologicalAddress::parse("mesh.r1.n2", ".");
        assert!(a.is_neighbor(&b));
    }

    #[test]
    fn is_neighbor_different_parents() {
        let a = TopologicalAddress::parse("mesh.r1.n1", ".");
        let b = TopologicalAddress::parse("mesh.r2.n1", ".");
        assert!(!a.is_neighbor(&b));
    }

    #[test]
    fn is_neighbor_different_depths() {
        let a = TopologicalAddress::parse("mesh.r1", ".");
        let b = TopologicalAddress::parse("mesh.r1.n1", ".");
        assert!(!a.is_neighbor(&b));
    }

    #[test]
    fn ancestors_three_levels() {
        let addr = TopologicalAddress::parse("mesh.r1.n1", ".");
        let anc = addr.ancestors();
        assert_eq!(anc.len(), 2); // mesh.r1, mesh
        assert_eq!(anc[0].render(), "mesh.r1");
        assert_eq!(anc[1].render(), "mesh");
    }

    #[test]
    fn ancestors_single_segment() {
        let addr = TopologicalAddress::parse("root", ".");
        let anc = addr.ancestors();
        assert!(anc.is_empty());
    }

    // ---------- Path tests ----------

    #[test]
    fn path_new_has_source() {
        let p = Path::new("origin", 16);
        assert_eq!(p.source(), Some("origin"));
        assert_eq!(p.current(), Some("origin"));
        assert_eq!(p.hop_count(), 0);
        assert_eq!(p.remaining_ttl(), 16);
    }

    #[test]
    fn path_add_hop_decrements_ttl() {
        let mut p = Path::new("a", 3);
        assert!(p.add_hop("b"));
        assert_eq!(p.remaining_ttl(), 2);
        assert!(p.add_hop("c"));
        assert_eq!(p.remaining_ttl(), 1);
        assert!(p.add_hop("d"));
        assert_eq!(p.remaining_ttl(), 0);
        // Now expired
        assert!(!p.add_hop("e"));
    }

    #[test]
    fn path_ttl_one_allows_no_hops_beyond_source() {
        let mut p = Path::new("source", 1);
        assert_eq!(p.remaining_ttl(), 1);
        assert!(p.add_hop("next"));
        assert_eq!(p.remaining_ttl(), 0);
        assert!(p.is_expired());
    }

    #[test]
    fn path_has_loop_detects_cycle() {
        let mut p = Path::new("a", 10);
        assert!(p.add_hop("b"));
        assert!(p.add_hop("c"));
        assert!(!p.has_loop());
        assert!(p.add_hop("a")); // loop back to origin
        assert!(p.has_loop());
    }

    #[test]
    fn path_no_loop_linear() {
        let mut p = Path::new("a", 10);
        assert!(p.add_hop("b"));
        assert!(p.add_hop("c"));
        assert!(p.add_hop("d"));
        assert!(!p.has_loop());
    }

    #[test]
    fn path_hop_count() {
        let mut p = Path::new("start", 10);
        assert_eq!(p.hop_count(), 0);
        assert!(p.add_hop("mid"));
        assert_eq!(p.hop_count(), 1);
        assert!(p.add_hop("end"));
        assert_eq!(p.hop_count(), 2);
    }

    // ---------- RouteQuality tests ----------

    #[test]
    fn route_quality_score_perfect() {
        let q = RouteQuality::new(0.0, 1.0, 0);
        // 1.0 * 1.0 * 1.0 = 1.0
        assert!((q.score() - 1.0).abs() < 0.001);
    }

    #[test]
    fn route_quality_score_degrades_with_latency() {
        let fast = RouteQuality::new(10.0, 1.0, 1);
        let slow = RouteQuality::new(5000.0, 1.0, 1);
        assert!(fast.score() > slow.score());
    }

    #[test]
    fn route_quality_score_degrades_with_hops() {
        let short = RouteQuality::new(50.0, 0.99, 1);
        let long = RouteQuality::new(50.0, 0.99, 8);
        assert!(short.score() > long.score());
    }

    #[test]
    fn route_quality_score_degrades_with_loss() {
        let reliable = RouteQuality::new(50.0, 0.99, 2);
        let lossy = RouteQuality::new(50.0, 0.5, 2);
        assert!(reliable.score() > lossy.score());
    }

    #[test]
    fn route_quality_usable_threshold() {
        let good = RouteQuality::new(100.0, 0.5, 3);
        assert!(good.is_usable());
        let bad = RouteQuality::new(100.0, 0.05, 3);
        assert!(!bad.is_usable());
    }

    #[test]
    fn route_quality_ordering() {
        let a = RouteQuality::new(10.0, 0.99, 1);
        let b = RouteQuality::new(500.0, 0.6, 5);
        assert!(a > b);
    }

    #[test]
    fn route_quality_clamps_negative_latency() {
        let q = RouteQuality::new(-10.0, 0.8, 2);
        assert!((q.latency_ms - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn route_quality_clamps_reliability() {
        let over = RouteQuality::new(10.0, 1.5, 1);
        assert!((over.reliability - 1.0).abs() < f64::EPSILON);
        let under = RouteQuality::new(10.0, -0.5, 1);
        assert!((under.reliability - 0.0).abs() < f64::EPSILON);
    }
}
