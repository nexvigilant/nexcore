//! # Dependency Graph Analysis
//!
//! Builds and analyzes the workspace dependency graph from cargo metadata.
//!
//! ## Primitive Foundation
//! - σ (Sequence): Topological ordering
//! - ρ (Recursion): Cycle detection
//! - μ (Mapping): Package → node mapping
//! - κ (Comparison): Fan-in/fan-out ranking

use std::collections::{HashMap, HashSet, VecDeque};

use serde::{Deserialize, Serialize};

/// A node in the dependency graph representing a workspace crate.
///
/// Tier: T2-C (σ + μ + κ + N)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CrateNode {
    /// Crate name.
    pub name: String,
    /// Number of crates that depend on this crate (reverse deps).
    pub fan_in: usize,
    /// Number of crates this crate depends on (forward deps).
    pub fan_out: usize,
    /// Direct dependents (crates that import this one).
    pub dependents: Vec<String>,
    /// Direct dependencies (crates this one imports).
    pub dependencies: Vec<String>,
    /// Topological depth (0 = leaf with no deps).
    pub topo_depth: usize,
}

/// The full workspace dependency graph.
///
/// Tier: T3 (σ + μ + κ + ρ + N + ∂)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DependencyGraph {
    /// All crate nodes indexed by name.
    pub nodes: HashMap<String, CrateNode>,
    /// Detected dependency cycles (should be empty in a healthy workspace).
    pub cycles: Vec<Vec<String>>,
    /// Topological sort order (foundation first, services last).
    pub topo_order: Vec<String>,
    /// Total workspace crates.
    pub total_crates: usize,
}

impl DependencyGraph {
    /// Create an empty dependency graph.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            nodes: HashMap::new(),
            cycles: Vec::new(),
            topo_order: Vec::new(),
            total_crates: 0,
        }
    }

    /// Build the dependency graph from cargo metadata.
    ///
    /// Only includes workspace members, not external crates.
    pub fn from_metadata(metadata: &cargo_metadata::Metadata) -> Self {
        let workspace_members: HashSet<&str> = metadata
            .workspace_packages()
            .iter()
            .map(|p| p.name.as_str())
            .collect();

        let mut nodes: HashMap<String, CrateNode> = HashMap::new();

        // Initialize all workspace member nodes
        for pkg in metadata.workspace_packages() {
            let workspace_deps: Vec<String> = pkg
                .dependencies
                .iter()
                .filter(|d| {
                    workspace_members.contains(d.name.as_str())
                        && d.kind == cargo_metadata::DependencyKind::Normal
                })
                .map(|d| d.name.clone())
                .collect();

            nodes.insert(
                pkg.name.clone(),
                CrateNode {
                    name: pkg.name.clone(),
                    fan_in: 0,
                    fan_out: workspace_deps.len(),
                    dependents: Vec::new(),
                    dependencies: workspace_deps,
                    topo_depth: 0,
                },
            );
        }

        // Compute fan-in (reverse dependencies)
        let names: Vec<String> = nodes.keys().cloned().collect();
        for name in &names {
            let deps: Vec<String> = nodes
                .get(name)
                .map(|n| n.dependencies.clone())
                .unwrap_or_default();
            for dep in deps {
                if let Some(dep_node) = nodes.get_mut(&dep) {
                    dep_node.fan_in += 1;
                    dep_node.dependents.push(name.clone());
                }
            }
        }

        // Topological sort + depth computation (Kahn's algorithm)
        let (topo_order, cycles) = Self::topo_sort(&nodes);

        // Compute topological depth
        let mut depth_map: HashMap<String, usize> = HashMap::new();
        for name in &topo_order {
            let max_dep_depth = nodes
                .get(name)
                .map(|n| {
                    n.dependencies
                        .iter()
                        .filter_map(|d| depth_map.get(d))
                        .copied()
                        .max()
                        .unwrap_or(0)
                })
                .unwrap_or(0);

            let depth = if nodes
                .get(name)
                .map(|n| n.dependencies.is_empty())
                .unwrap_or(true)
            {
                0
            } else {
                max_dep_depth + 1
            };

            depth_map.insert(name.clone(), depth);
        }

        for (name, depth) in &depth_map {
            if let Some(node) = nodes.get_mut(name) {
                node.topo_depth = *depth;
            }
        }

        let total_crates = nodes.len();

        Self {
            nodes,
            cycles,
            topo_order,
            total_crates,
        }
    }

    /// Build the dependency graph from a workspace manifest path.
    ///
    /// Convenience wrapper that runs `cargo_metadata::MetadataCommand` and
    /// then calls `from_metadata`.
    ///
    /// # Errors
    /// Returns an error string if cargo metadata execution fails.
    pub fn from_manifest_path(manifest_path: &std::path::Path) -> Result<Self, String> {
        let metadata = cargo_metadata::MetadataCommand::new()
            .manifest_path(manifest_path)
            .exec()
            .map_err(|e| format!("cargo metadata failed: {e}"))?;
        Ok(Self::from_metadata(&metadata))
    }

    /// Kahn's algorithm for topological sort with cycle detection.
    fn topo_sort(nodes: &HashMap<String, CrateNode>) -> (Vec<String>, Vec<Vec<String>>) {
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();

        for (name, node) in nodes {
            in_degree.entry(name.as_str()).or_insert(0);
            for dep in &node.dependencies {
                if nodes.contains_key(dep) {
                    adj.entry(dep.as_str()).or_default().push(name.as_str());
                    *in_degree.entry(name.as_str()).or_insert(0) += 1;
                }
            }
        }

        let mut queue: VecDeque<&str> = in_degree
            .iter()
            .filter(|(_, deg)| **deg == 0)
            .map(|(name, _)| *name)
            .collect();

        let mut order = Vec::new();

        while let Some(name) = queue.pop_front() {
            order.push(name.to_string());
            if let Some(dependents) = adj.get(name) {
                for &dep in dependents {
                    if let Some(deg) = in_degree.get_mut(dep) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(dep);
                        }
                    }
                }
            }
        }

        // Any remaining nodes with in_degree > 0 are in cycles
        let cycles = if order.len() < nodes.len() {
            let sorted_set: HashSet<&str> = order.iter().map(|s| s.as_str()).collect();
            let cycle_members: Vec<String> = nodes
                .keys()
                .filter(|k| !sorted_set.contains(k.as_str()))
                .cloned()
                .collect();
            if cycle_members.is_empty() {
                vec![]
            } else {
                vec![cycle_members]
            }
        } else {
            vec![]
        };

        (order, cycles)
    }

    /// Get the top N crates by fan-in (most depended upon).
    #[must_use]
    pub fn top_by_fan_in(&self, n: usize) -> Vec<&CrateNode> {
        let mut nodes: Vec<&CrateNode> = self.nodes.values().collect();
        nodes.sort_by(|a, b| b.fan_in.cmp(&a.fan_in));
        nodes.truncate(n);
        nodes
    }

    /// Get the top N crates by fan-out (most dependencies).
    #[must_use]
    pub fn top_by_fan_out(&self, n: usize) -> Vec<&CrateNode> {
        let mut nodes: Vec<&CrateNode> = self.nodes.values().collect();
        nodes.sort_by(|a, b| b.fan_out.cmp(&a.fan_out));
        nodes.truncate(n);
        nodes
    }

    /// Check if the graph has any cycles.
    #[must_use]
    pub fn has_cycles(&self) -> bool {
        !self.cycles.is_empty()
    }

    /// Maximum topological depth in the graph.
    #[must_use]
    pub fn max_depth(&self) -> usize {
        self.nodes.values().map(|n| n.topo_depth).max().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topo_sort_no_cycles() {
        let mut nodes = HashMap::new();
        nodes.insert(
            "a".to_string(),
            CrateNode {
                name: "a".to_string(),
                fan_in: 0,
                fan_out: 0,
                dependents: vec![],
                dependencies: vec![],
                topo_depth: 0,
            },
        );
        nodes.insert(
            "b".to_string(),
            CrateNode {
                name: "b".to_string(),
                fan_in: 0,
                fan_out: 1,
                dependents: vec![],
                dependencies: vec!["a".to_string()],
                topo_depth: 0,
            },
        );

        let (order, cycles) = DependencyGraph::topo_sort(&nodes);
        assert!(cycles.is_empty());
        assert_eq!(order.len(), 2);
        // a should come before b
        let a_pos = order.iter().position(|x| x == "a");
        let b_pos = order.iter().position(|x| x == "b");
        assert!(a_pos < b_pos);
    }

    #[test]
    fn test_top_by_fan_in() {
        let mut nodes = HashMap::new();
        for (name, fi) in [("core", 10), ("util", 5), ("app", 0)] {
            nodes.insert(
                name.to_string(),
                CrateNode {
                    name: name.to_string(),
                    fan_in: fi,
                    fan_out: 0,
                    dependents: vec![],
                    dependencies: vec![],
                    topo_depth: 0,
                },
            );
        }

        let graph = DependencyGraph {
            nodes,
            cycles: vec![],
            topo_order: vec![],
            total_crates: 3,
        };

        let top = graph.top_by_fan_in(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].name, "core");
        assert_eq!(top[1].name, "util");
    }
}
