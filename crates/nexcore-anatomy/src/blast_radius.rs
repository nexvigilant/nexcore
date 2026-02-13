//! # Blast Radius Analysis
//!
//! Computes the transitive impact of changes to any crate in the workspace.
//! Given a "source" crate, calculates how many crates would need rebuilding
//! and how deep the cascade propagates.
//!
//! ## Primitive Foundation
//! - ρ (Recursion): Transitive dependent traversal (BFS/DFS)
//! - N (Quantity): Impact metrics (direct, transitive, percentage)
//! - κ (Comparison): Bottleneck ranking by blast radius
//! - μ (Mapping): Crate → impact score
//! - ∂ (Boundary): Change containment boundaries

use std::collections::{HashMap, HashSet, VecDeque};

use serde::{Deserialize, Serialize};

use crate::graph::DependencyGraph;

/// Blast radius for a single crate — measures the impact of changing it.
///
/// Tier: T2-C (ρ + N + κ + μ)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlastRadius {
    /// The crate being analyzed (the "epicenter").
    pub crate_name: String,
    /// Direct dependents (fan-in).
    pub direct_dependents: Vec<String>,
    /// All transitive dependents (full cascade).
    pub transitive_dependents: Vec<String>,
    /// Number of direct dependents.
    pub direct_count: usize,
    /// Number of transitive dependents (includes direct).
    pub transitive_count: usize,
    /// Percentage of workspace affected (transitive_count / total_crates).
    pub impact_ratio: f64,
    /// Maximum cascade depth (longest path from epicenter to a leaf dependent).
    pub cascade_depth: usize,
    /// Cascade width at each depth level (how many crates at each hop).
    pub cascade_width: Vec<usize>,
    /// Containment score: 1.0 - impact_ratio (higher = more contained).
    pub containment: f64,
}

/// Blast radius report for the entire workspace.
///
/// Tier: T3 (ρ + N + κ + μ + Σ + ∂)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlastRadiusReport {
    /// Per-crate blast radius, sorted by impact_ratio descending.
    pub radii: Vec<BlastRadius>,
    /// Top bottleneck crates (highest blast radius).
    pub bottlenecks: Vec<String>,
    /// Average impact ratio across all crates.
    pub avg_impact: f64,
    /// Crate with maximum blast radius.
    pub worst_case_crate: String,
    /// Maximum blast radius (transitive count).
    pub worst_case_count: usize,
    /// Worst case impact ratio.
    pub worst_case_ratio: f64,
    /// Total workspace crates analyzed.
    pub total_crates: usize,
}

impl BlastRadius {
    /// Compute the blast radius for a specific crate in the dependency graph.
    ///
    /// Uses BFS from the epicenter through the reverse dependency graph
    /// (following dependents, not dependencies) to find all transitively
    /// affected crates.
    pub fn for_crate(graph: &DependencyGraph, crate_name: &str) -> Option<Self> {
        let node = graph.nodes.get(crate_name)?;
        let total = graph.total_crates.max(1) as f64;

        // BFS through reverse dependencies (dependents)
        let mut visited: HashSet<&str> = HashSet::new();
        let mut queue: VecDeque<(&str, usize)> = VecDeque::new();
        let mut cascade_levels: Vec<Vec<String>> = Vec::new();

        // Seed with direct dependents at depth 1
        visited.insert(crate_name);
        for dep in &node.dependents {
            if !visited.contains(dep.as_str()) {
                visited.insert(dep.as_str());
                queue.push_back((dep.as_str(), 1));
            }
        }

        while let Some((name, depth)) = queue.pop_front() {
            // Ensure cascade_levels has enough capacity
            while cascade_levels.len() < depth {
                cascade_levels.push(Vec::new());
            }
            cascade_levels[depth - 1].push(name.to_string());

            // Follow this crate's dependents (reverse deps)
            if let Some(n) = graph.nodes.get(name) {
                for dep in &n.dependents {
                    if !visited.contains(dep.as_str()) {
                        visited.insert(dep.as_str());
                        queue.push_back((dep.as_str(), depth + 1));
                    }
                }
            }
        }

        let direct_dependents = node.dependents.clone();
        let direct_count = direct_dependents.len();

        let mut transitive_dependents: Vec<String> = cascade_levels
            .iter()
            .flat_map(|level| level.iter().cloned())
            .collect();
        transitive_dependents.sort();
        let transitive_count = transitive_dependents.len();

        let impact_ratio = transitive_count as f64 / total;
        let cascade_depth = cascade_levels.len();
        let cascade_width: Vec<usize> = cascade_levels.iter().map(|l| l.len()).collect();
        let containment = 1.0 - impact_ratio;

        Some(Self {
            crate_name: crate_name.to_string(),
            direct_dependents,
            transitive_dependents,
            direct_count,
            transitive_count,
            impact_ratio,
            cascade_depth,
            cascade_width,
            containment,
        })
    }
}

impl BlastRadiusReport {
    /// Compute blast radius for every crate in the workspace.
    pub fn from_graph(graph: &DependencyGraph) -> Self {
        let mut radii: Vec<BlastRadius> = graph
            .nodes
            .keys()
            .filter_map(|name| BlastRadius::for_crate(graph, name))
            .collect();

        // Sort by impact ratio descending (worst first)
        radii.sort_by(|a, b| {
            b.impact_ratio
                .partial_cmp(&a.impact_ratio)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let avg_impact = if radii.is_empty() {
            0.0
        } else {
            radii.iter().map(|r| r.impact_ratio).sum::<f64>() / radii.len() as f64
        };

        let worst = radii.first();
        let worst_case_crate = worst.map(|r| r.crate_name.clone()).unwrap_or_default();
        let worst_case_count = worst.map(|r| r.transitive_count).unwrap_or(0);
        let worst_case_ratio = worst.map(|r| r.impact_ratio).unwrap_or(0.0);

        // Bottlenecks: crates affecting > 10% of workspace
        let bottlenecks: Vec<String> = radii
            .iter()
            .filter(|r| r.impact_ratio > 0.10)
            .map(|r| r.crate_name.clone())
            .collect();

        Self {
            radii,
            bottlenecks,
            avg_impact,
            worst_case_crate,
            worst_case_count,
            worst_case_ratio,
            total_crates: graph.total_crates,
        }
    }

    /// Get the blast radius for a specific crate.
    #[must_use]
    pub fn get(&self, crate_name: &str) -> Option<&BlastRadius> {
        self.radii.iter().find(|r| r.crate_name == crate_name)
    }

    /// Get the top N crates by blast radius.
    #[must_use]
    pub fn top(&self, n: usize) -> &[BlastRadius] {
        let end = n.min(self.radii.len());
        &self.radii[..end]
    }

    /// Serialize to JSON.
    ///
    /// # Errors
    /// Returns error if serialization fails.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::CrateNode;

    fn diamond_graph() -> DependencyGraph {
        // Diamond: core ← {mid-a, mid-b} ← top
        // core has blast radius of 3 (mid-a, mid-b, top)
        let mut nodes = HashMap::new();

        nodes.insert(
            "core".to_string(),
            CrateNode {
                name: "core".to_string(),
                fan_in: 2,
                fan_out: 0,
                dependents: vec!["mid-a".to_string(), "mid-b".to_string()],
                dependencies: vec![],
                topo_depth: 0,
            },
        );
        nodes.insert(
            "mid-a".to_string(),
            CrateNode {
                name: "mid-a".to_string(),
                fan_in: 1,
                fan_out: 1,
                dependents: vec!["top".to_string()],
                dependencies: vec!["core".to_string()],
                topo_depth: 1,
            },
        );
        nodes.insert(
            "mid-b".to_string(),
            CrateNode {
                name: "mid-b".to_string(),
                fan_in: 1,
                fan_out: 1,
                dependents: vec!["top".to_string()],
                dependencies: vec!["core".to_string()],
                topo_depth: 1,
            },
        );
        nodes.insert(
            "top".to_string(),
            CrateNode {
                name: "top".to_string(),
                fan_in: 0,
                fan_out: 2,
                dependents: vec![],
                dependencies: vec!["mid-a".to_string(), "mid-b".to_string()],
                topo_depth: 2,
            },
        );

        DependencyGraph {
            nodes,
            cycles: vec![],
            topo_order: vec![
                "core".to_string(),
                "mid-a".to_string(),
                "mid-b".to_string(),
                "top".to_string(),
            ],
            total_crates: 4,
        }
    }

    #[test]
    fn test_core_blast_radius() {
        let graph = diamond_graph();
        let radius = BlastRadius::for_crate(&graph, "core");
        assert!(radius.is_some());

        let r = radius.unwrap_or_else(|| unreachable!());
        assert_eq!(r.direct_count, 2); // mid-a, mid-b
        assert_eq!(r.transitive_count, 3); // mid-a, mid-b, top
        assert_eq!(r.cascade_depth, 2); // depth 1: {mid-a, mid-b}, depth 2: {top}
        assert!((r.impact_ratio - 0.75).abs() < f64::EPSILON); // 3/4
        assert!((r.containment - 0.25).abs() < f64::EPSILON); // 1 - 0.75
    }

    #[test]
    fn test_cascade_width() {
        let graph = diamond_graph();
        let r = BlastRadius::for_crate(&graph, "core").unwrap_or_else(|| unreachable!());

        // Depth 1: 2 crates (mid-a, mid-b), Depth 2: 1 crate (top)
        assert_eq!(r.cascade_width, vec![2, 1]);
    }

    #[test]
    fn test_leaf_blast_radius() {
        let graph = diamond_graph();
        let r = BlastRadius::for_crate(&graph, "top").unwrap_or_else(|| unreachable!());

        assert_eq!(r.direct_count, 0);
        assert_eq!(r.transitive_count, 0);
        assert!((r.impact_ratio - 0.0).abs() < f64::EPSILON);
        assert!((r.containment - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_nonexistent_crate() {
        let graph = diamond_graph();
        assert!(BlastRadius::for_crate(&graph, "nonexistent").is_none());
    }

    #[test]
    fn test_blast_radius_report() {
        let graph = diamond_graph();
        let report = BlastRadiusReport::from_graph(&graph);

        assert_eq!(report.total_crates, 4);
        assert_eq!(report.worst_case_crate, "core");
        assert_eq!(report.worst_case_count, 3);
        assert!(!report.bottlenecks.is_empty());
        assert!(report.bottlenecks.contains(&"core".to_string()));
    }

    #[test]
    fn test_report_top() {
        let graph = diamond_graph();
        let report = BlastRadiusReport::from_graph(&graph);

        let top2 = report.top(2);
        assert_eq!(top2.len(), 2);
        // First should be core (highest blast radius)
        assert_eq!(top2[0].crate_name, "core");
    }

    #[test]
    fn test_report_get() {
        let graph = diamond_graph();
        let report = BlastRadiusReport::from_graph(&graph);

        let mid = report.get("mid-a");
        assert!(mid.is_some());
        assert_eq!(mid.map(|r| r.transitive_count), Some(1)); // only top
    }
}
