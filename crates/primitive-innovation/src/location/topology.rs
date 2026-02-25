// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # TopologyGraph
//!
//! **Tier**: T2-C (lambda + rho + sigma + partial)
//! **Dominant**: lambda (Location)
//!
//! Network topology graph with node addressing and path routing.
//! Models hierarchical network structures where nodes have topological addresses.

use core::fmt;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// A unique node identifier in the topology.
pub type NodeId = u32;

/// A node in the topology graph.
///
/// ## Tier: T2-P (lambda + state)
#[derive(Debug, Clone)]
pub struct TopologyNode {
    /// Node identifier.
    pub id: NodeId,
    /// Human-readable label.
    pub label: String,
    /// Hierarchical address (e.g., "region.zone.rack.host").
    pub address: String,
    /// Node depth in the hierarchy (0 = root).
    pub depth: u32,
}

/// An edge connecting two topology nodes.
///
/// ## Tier: T2-P (lambda + causality)
#[derive(Debug, Clone)]
pub struct TopologyEdge {
    /// Source node.
    pub from: NodeId,
    /// Target node.
    pub to: NodeId,
    /// Edge weight (latency, cost, etc.).
    pub weight: f64,
    /// Whether the edge is bidirectional.
    pub bidirectional: bool,
}

/// Network topology with routing and path discovery.
///
/// ## Tier: T2-C (lambda + rho + sigma + partial)
/// Dominant: lambda (Location)
#[derive(Debug, Clone)]
pub struct TopologyGraph {
    /// Nodes by ID.
    nodes: BTreeMap<NodeId, TopologyNode>,
    /// Adjacency list: node -> [(neighbor, weight)].
    adjacency: BTreeMap<NodeId, Vec<(NodeId, f64)>>,
    /// Next available node ID.
    next_id: NodeId,
}

impl TopologyGraph {
    /// Create an empty topology.
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes: BTreeMap::new(),
            adjacency: BTreeMap::new(),
            next_id: 0,
        }
    }

    /// Add a node to the topology.
    pub fn add_node(
        &mut self,
        label: impl Into<String>,
        address: impl Into<String>,
        depth: u32,
    ) -> NodeId {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);

        let node = TopologyNode {
            id,
            label: label.into(),
            address: address.into(),
            depth,
        };

        self.nodes.insert(id, node);
        self.adjacency.entry(id).or_default();

        id
    }

    /// Connect two nodes with a weighted edge.
    pub fn connect(&mut self, from: NodeId, to: NodeId, weight: f64, bidirectional: bool) {
        self.adjacency.entry(from).or_default().push((to, weight));
        if bidirectional {
            self.adjacency.entry(to).or_default().push((from, weight));
        }
    }

    /// Get a node by ID.
    #[must_use]
    pub fn node(&self, id: NodeId) -> Option<&TopologyNode> {
        self.nodes.get(&id)
    }

    /// Get all neighbors of a node.
    #[must_use]
    pub fn neighbors(&self, id: NodeId) -> Vec<(NodeId, f64)> {
        self.adjacency.get(&id).cloned().unwrap_or_default()
    }

    /// Find shortest path between two nodes using BFS (unweighted).
    #[must_use]
    pub fn shortest_path(&self, from: NodeId, to: NodeId) -> Option<Vec<NodeId>> {
        if from == to {
            return Some(vec![from]);
        }

        let mut visited = BTreeSet::new();
        let mut queue = VecDeque::new();
        let mut parent: BTreeMap<NodeId, NodeId> = BTreeMap::new();

        visited.insert(from);
        queue.push_back(from);

        while let Some(current) = queue.pop_front() {
            for &(neighbor, _) in self.adjacency.get(&current).unwrap_or(&Vec::new()) {
                if visited.insert(neighbor) {
                    parent.insert(neighbor, current);
                    if neighbor == to {
                        // Reconstruct path
                        let mut path = vec![to];
                        let mut node = to;
                        while let Some(&p) = parent.get(&node) {
                            path.push(p);
                            node = p;
                        }
                        path.reverse();
                        return Some(path);
                    }
                    queue.push_back(neighbor);
                }
            }
        }

        None // No path found
    }

    /// Calculate hop count between two nodes.
    #[must_use]
    pub fn hop_count(&self, from: NodeId, to: NodeId) -> Option<usize> {
        self.shortest_path(from, to)
            .map(|path| path.len().saturating_sub(1))
    }

    /// Find all nodes at a given depth.
    #[must_use]
    pub fn nodes_at_depth(&self, depth: u32) -> Vec<&TopologyNode> {
        self.nodes.values().filter(|n| n.depth == depth).collect()
    }

    /// Find nodes by address prefix.
    #[must_use]
    pub fn nodes_in_region(&self, prefix: &str) -> Vec<&TopologyNode> {
        self.nodes
            .values()
            .filter(|n| n.address.starts_with(prefix))
            .collect()
    }

    /// Total number of nodes.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Total number of edges (counting bidirectional as 2).
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.adjacency.values().map(|v| v.len()).sum()
    }

    /// Maximum depth in the topology.
    #[must_use]
    pub fn max_depth(&self) -> u32 {
        self.nodes.values().map(|n| n.depth).max().unwrap_or(0)
    }
}

impl Default for TopologyGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for TopologyGraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TopologyGraph({} nodes, {} edges, depth {})",
            self.node_count(),
            self.edge_count(),
            self.max_depth()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_test_topology() -> TopologyGraph {
        let mut g = TopologyGraph::new();
        let root = g.add_node("root", "dc", 0);
        let zone_a = g.add_node("zone-a", "dc.a", 1);
        let zone_b = g.add_node("zone-b", "dc.b", 1);
        let host_1 = g.add_node("host-1", "dc.a.h1", 2);
        let host_2 = g.add_node("host-2", "dc.a.h2", 2);
        let host_3 = g.add_node("host-3", "dc.b.h3", 2);

        g.connect(root, zone_a, 1.0, true);
        g.connect(root, zone_b, 1.0, true);
        g.connect(zone_a, host_1, 0.5, true);
        g.connect(zone_a, host_2, 0.5, true);
        g.connect(zone_b, host_3, 0.5, true);

        g
    }

    #[test]
    fn test_topology_structure() {
        let g = build_test_topology();
        assert_eq!(g.node_count(), 6);
        assert_eq!(g.max_depth(), 2);
    }

    #[test]
    fn test_shortest_path() {
        let g = build_test_topology();
        // host-1 (id=3) to host-3 (id=5): h1 -> zone-a -> root -> zone-b -> h3
        let path = g.shortest_path(3, 5);
        assert!(path.is_some());
        let path = path.unwrap_or_default();
        assert_eq!(path.len(), 5);
        assert_eq!(path[0], 3); // host-1
        assert_eq!(path[4], 5); // host-3
    }

    #[test]
    fn test_hop_count() {
        let g = build_test_topology();
        assert_eq!(g.hop_count(3, 5), Some(4)); // h1 -> za -> root -> zb -> h3
        assert_eq!(g.hop_count(0, 0), Some(0)); // same node
    }

    #[test]
    fn test_nodes_at_depth() {
        let g = build_test_topology();
        assert_eq!(g.nodes_at_depth(0).len(), 1);
        assert_eq!(g.nodes_at_depth(1).len(), 2);
        assert_eq!(g.nodes_at_depth(2).len(), 3);
    }

    #[test]
    fn test_region_lookup() {
        let g = build_test_topology();
        let zone_a_nodes = g.nodes_in_region("dc.a");
        assert_eq!(zone_a_nodes.len(), 3); // zone-a, host-1, host-2
    }
}
