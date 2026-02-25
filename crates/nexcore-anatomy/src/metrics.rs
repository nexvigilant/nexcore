//! # Structural Metrics & Criticality Scoring
//!
//! Computes criticality scores, coupling coefficients, and information-theoretic
//! metrics for workspace crates.
//!
//! ## Primitive Foundation
//! - N (Quantity): Numeric scores, ratios, coefficients
//! - κ (Comparison): Ranking, threshold checks
//! - μ (Mapping): Crate → score mapping
//! - Σ (Sum): Aggregation across crates
//! - ∂ (Boundary): Threshold-based criticality tiers

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::graph::DependencyGraph;

/// Criticality tier for a workspace crate.
///
/// Tier: T2-P (κ + ∂)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum CriticalityTier {
    /// Critical: high fan-in, breaking changes cascade widely.
    Critical = 3,
    /// Supporting: moderate fan-in, used by multiple consumers.
    Supporting = 2,
    /// Standard: low fan-in, localized impact.
    Standard = 1,
    /// Experimental: zero or near-zero fan-in, safe to modify.
    Experimental = 0,
}

impl CriticalityTier {
    /// Returns a human-readable label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Critical => "Critical",
            Self::Supporting => "Supporting",
            Self::Standard => "Standard",
            Self::Experimental => "Experimental",
        }
    }
}

/// Criticality score for a single crate.
///
/// Tier: T2-C (N + κ + μ + ∂)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CriticalityScore {
    /// Crate name.
    pub name: String,
    /// Assigned criticality tier.
    pub tier: CriticalityTier,
    /// Raw criticality score (0.0 - 1.0).
    pub score: f64,
    /// Fan-in (number of dependents).
    pub fan_in: usize,
    /// Fan-out (number of dependencies).
    pub fan_out: usize,
    /// Topological depth in the graph.
    pub depth: usize,
    /// Fan-in as percentage of total workspace crates.
    pub fan_in_ratio: f64,
}

/// Workspace-level structural metrics.
///
/// Tier: T3 (N + κ + μ + Σ + ∂ + σ)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct WorkspaceMetrics {
    /// Per-crate criticality scores, sorted by score descending.
    pub criticality: Vec<CriticalityScore>,
    /// Instability index per crate: fan_out / (fan_in + fan_out).
    /// Stable crates (many dependents, few deps) approach 0.0.
    /// Unstable crates (few dependents, many deps) approach 1.0.
    pub instability: HashMap<String, f64>,
    /// Abstractness placeholder (would need AST analysis for trait/impl ratio).
    /// For now, defaults to 0.5 (unknown).
    pub abstractness: HashMap<String, f64>,
    /// Distance from main sequence: |abstractness + instability - 1.0|.
    /// Crates near 0.0 are well-balanced. Near 1.0 = "zone of pain" or "zone of uselessness".
    pub main_sequence_distance: HashMap<String, f64>,
    /// Average instability across workspace.
    pub avg_instability: f64,
    /// Maximum fan-in in the workspace (bottleneck indicator).
    pub max_fan_in: usize,
    /// Crate with maximum fan-in.
    pub bottleneck_crate: String,
    /// Total dependency edges in the workspace graph.
    pub total_edges: usize,
    /// Graph density: edges / (n * (n-1)) for directed graph.
    pub graph_density: f64,
}

/// Criticality thresholds as percentage of total workspace crates.
///
/// A crate is Critical if fan_in_ratio >= critical_threshold, etc.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct CriticalityThresholds {
    /// Fan-in ratio threshold for Critical tier (default: 0.10 = 10% of workspace).
    pub critical: f64,
    /// Fan-in ratio threshold for Supporting tier (default: 0.03 = 3% of workspace).
    pub supporting: f64,
    /// Fan-in ratio threshold for Standard tier (default: 0.01 = 1% of workspace).
    pub standard: f64,
}

impl Default for CriticalityThresholds {
    fn default() -> Self {
        Self {
            critical: 0.10,
            supporting: 0.03,
            standard: 0.01,
        }
    }
}

impl WorkspaceMetrics {
    /// Compute all structural metrics from a dependency graph.
    pub fn from_graph(graph: &DependencyGraph) -> Self {
        Self::from_graph_with_thresholds(graph, CriticalityThresholds::default())
    }

    /// Compute metrics with custom criticality thresholds.
    pub fn from_graph_with_thresholds(
        graph: &DependencyGraph,
        thresholds: CriticalityThresholds,
    ) -> Self {
        let total = graph.total_crates;
        let total_f64 = total.max(1) as f64;

        // Compute criticality scores
        let mut criticality: Vec<CriticalityScore> = graph
            .nodes
            .values()
            .map(|node| {
                let fan_in_ratio = node.fan_in as f64 / total_f64;
                let tier = if fan_in_ratio >= thresholds.critical {
                    CriticalityTier::Critical
                } else if fan_in_ratio >= thresholds.supporting {
                    CriticalityTier::Supporting
                } else if fan_in_ratio >= thresholds.standard {
                    CriticalityTier::Standard
                } else {
                    CriticalityTier::Experimental
                };

                // Composite score: weighted combination of fan-in ratio and depth
                let depth_ratio = node.topo_depth as f64 / graph.max_depth().max(1) as f64;
                let score = fan_in_ratio * 0.7 + (1.0 - depth_ratio) * 0.3;

                CriticalityScore {
                    name: node.name.clone(),
                    tier,
                    score,
                    fan_in: node.fan_in,
                    fan_out: node.fan_out,
                    depth: node.topo_depth,
                    fan_in_ratio,
                }
            })
            .collect();

        criticality.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Compute instability: I = fan_out / (fan_in + fan_out)
        let mut instability = HashMap::new();
        for node in graph.nodes.values() {
            let total_coupling = (node.fan_in + node.fan_out) as f64;
            let i = if total_coupling > 0.0 {
                node.fan_out as f64 / total_coupling
            } else {
                0.5 // Isolated crate: neutral instability
            };
            instability.insert(node.name.clone(), i);
        }

        // Abstractness defaults to 0.5 (needs AST analysis for real values)
        let abstractness: HashMap<String, f64> =
            graph.nodes.keys().map(|name| (name.clone(), 0.5)).collect();

        // Main sequence distance: D = |A + I - 1|
        let mut main_sequence_distance = HashMap::new();
        for name in graph.nodes.keys() {
            let a = abstractness.get(name).copied().unwrap_or(0.5);
            let i = instability.get(name).copied().unwrap_or(0.5);
            main_sequence_distance.insert(name.clone(), (a + i - 1.0).abs());
        }

        // Aggregate metrics
        let avg_instability = if instability.is_empty() {
            0.0
        } else {
            instability.values().sum::<f64>() / instability.len() as f64
        };

        let (bottleneck_crate, max_fan_in) = graph
            .nodes
            .values()
            .max_by_key(|n| n.fan_in)
            .map(|n| (n.name.clone(), n.fan_in))
            .unwrap_or_else(|| (String::new(), 0));

        let total_edges: usize = graph.nodes.values().map(|n| n.fan_out).sum();

        let n = total as f64;
        let graph_density = if n > 1.0 {
            total_edges as f64 / (n * (n - 1.0))
        } else {
            0.0
        };

        Self {
            criticality,
            instability,
            abstractness,
            main_sequence_distance,
            avg_instability,
            max_fan_in,
            bottleneck_crate,
            total_edges,
            graph_density,
        }
    }

    /// Get all crates in a specific criticality tier.
    #[must_use]
    pub fn crates_in_tier(&self, tier: CriticalityTier) -> Vec<&CriticalityScore> {
        self.criticality.iter().filter(|c| c.tier == tier).collect()
    }

    /// Get the top N most critical crates.
    #[must_use]
    pub fn top_critical(&self, n: usize) -> &[CriticalityScore] {
        let end = n.min(self.criticality.len());
        &self.criticality[..end]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{CrateNode, DependencyGraph};

    fn test_graph() -> DependencyGraph {
        let mut nodes = HashMap::new();

        // Foundation crate with high fan-in
        nodes.insert(
            "core".to_string(),
            CrateNode {
                name: "core".to_string(),
                fan_in: 8,
                fan_out: 0,
                dependents: (1..=8).map(|i| format!("dep-{i}")).collect(),
                dependencies: vec![],
                topo_depth: 0,
            },
        );

        // Domain crate with moderate fan-in
        nodes.insert(
            "domain".to_string(),
            CrateNode {
                name: "domain".to_string(),
                fan_in: 3,
                fan_out: 1,
                dependents: vec![
                    "svc-a".to_string(),
                    "svc-b".to_string(),
                    "svc-c".to_string(),
                ],
                dependencies: vec!["core".to_string()],
                topo_depth: 1,
            },
        );

        // Service crate with zero fan-in
        nodes.insert(
            "service".to_string(),
            CrateNode {
                name: "service".to_string(),
                fan_in: 0,
                fan_out: 2,
                dependents: vec![],
                dependencies: vec!["core".to_string(), "domain".to_string()],
                topo_depth: 2,
            },
        );

        DependencyGraph {
            nodes,
            cycles: vec![],
            topo_order: vec![
                "core".to_string(),
                "domain".to_string(),
                "service".to_string(),
            ],
            total_crates: 3,
        }
    }

    #[test]
    fn test_criticality_scoring() {
        let graph = test_graph();
        let metrics = WorkspaceMetrics::from_graph(&graph);

        // core has fan_in=8, total=3, ratio=2.67 → Critical
        let core_score = metrics.criticality.iter().find(|c| c.name == "core");
        assert!(core_score.is_some());
        assert_eq!(core_score.map(|c| c.tier), Some(CriticalityTier::Critical));

        // service has fan_in=0 → Experimental
        let svc_score = metrics.criticality.iter().find(|c| c.name == "service");
        assert!(svc_score.is_some());
        assert_eq!(
            svc_score.map(|c| c.tier),
            Some(CriticalityTier::Experimental)
        );
    }

    #[test]
    fn test_instability_index() {
        let graph = test_graph();
        let metrics = WorkspaceMetrics::from_graph(&graph);

        // core: fan_out=0, fan_in=8 → I = 0/(8+0) = 0.0 (maximally stable)
        let core_i = metrics.instability.get("core").copied().unwrap_or(-1.0);
        assert!((core_i - 0.0).abs() < f64::EPSILON);

        // service: fan_out=2, fan_in=0 → I = 2/(0+2) = 1.0 (maximally unstable)
        let svc_i = metrics.instability.get("service").copied().unwrap_or(-1.0);
        assert!((svc_i - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_bottleneck_detection() {
        let graph = test_graph();
        let metrics = WorkspaceMetrics::from_graph(&graph);

        assert_eq!(metrics.bottleneck_crate, "core");
        assert_eq!(metrics.max_fan_in, 8);
    }

    #[test]
    fn test_graph_density() {
        let graph = test_graph();
        let metrics = WorkspaceMetrics::from_graph(&graph);

        // 3 edges total (domain→core, service→core, service→domain)
        assert_eq!(metrics.total_edges, 3);
        // density = 3 / (3 * 2) = 0.5
        assert!((metrics.graph_density - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_crates_in_tier() {
        let graph = test_graph();
        let metrics = WorkspaceMetrics::from_graph(&graph);

        let critical = metrics.crates_in_tier(CriticalityTier::Critical);
        assert!(!critical.is_empty());
        assert!(critical.iter().any(|c| c.name == "core"));
    }
}
