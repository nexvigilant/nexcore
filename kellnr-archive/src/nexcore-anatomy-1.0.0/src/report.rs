//! # Anatomy Report Generation
//!
//! Aggregates graph, layer, and metrics analysis into a unified JSON report.
//!
//! ## Primitive Foundation
//! - μ (Mapping): Assembly of analysis results into report structure
//! - σ (Sequence): Ordered sections, timestamps
//! - Σ (Sum): Aggregation of findings
//! - π (Persistence): Serializable output

use serde::{Deserialize, Serialize};

use crate::graph::DependencyGraph;
use crate::layer::LayerMap;
use crate::metrics::WorkspaceMetrics;

/// Health status of the workspace anatomy.
///
/// Tier: T2-P (κ + ∂)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// All checks pass: no cycles, no violations, density within bounds.
    Healthy,
    /// Minor issues: isolated violations or high coupling.
    Warning,
    /// Structural problems: cycles detected or severe violations.
    Critical,
}

impl HealthStatus {
    /// Returns a human-readable label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Healthy => "Healthy",
            Self::Warning => "Warning",
            Self::Critical => "Critical",
        }
    }
}

/// Summary statistics for the anatomy report.
///
/// Tier: T2-C (N + Σ + κ)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSummary {
    /// Total workspace crates analyzed.
    pub total_crates: usize,
    /// Crates classified as Critical.
    pub critical_count: usize,
    /// Crates classified as Supporting.
    pub supporting_count: usize,
    /// Crates classified as Experimental.
    pub experimental_count: usize,
    /// Number of boundary violations detected.
    pub violation_count: usize,
    /// Number of dependency cycles detected.
    pub cycle_count: usize,
    /// Maximum topological depth.
    pub max_depth: usize,
    /// Bottleneck crate (highest fan-in).
    pub bottleneck: String,
    /// Maximum fan-in value.
    pub max_fan_in: usize,
    /// Average instability index.
    pub avg_instability: f64,
    /// Graph density.
    pub graph_density: f64,
    /// Overall health status.
    pub health: HealthStatus,
}

/// Complete anatomy report combining all analysis results.
///
/// Tier: T3 (μ + σ + Σ + π + N + κ + ∂)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnatomyReport {
    /// Report summary and health status.
    pub summary: ReportSummary,
    /// Full dependency graph data.
    pub graph: DependencyGraph,
    /// Layer classification and violations.
    pub layers: LayerMap,
    /// Structural metrics and criticality scores.
    pub metrics: WorkspaceMetrics,
}

impl AnatomyReport {
    /// Build a complete anatomy report from a dependency graph.
    pub fn from_graph(graph: DependencyGraph) -> Self {
        let layers = LayerMap::from_graph(&graph);
        let metrics = WorkspaceMetrics::from_graph(&graph);

        let health = Self::compute_health(&graph, &layers, &metrics);

        let critical_count = metrics
            .crates_in_tier(crate::metrics::CriticalityTier::Critical)
            .len();
        let supporting_count = metrics
            .crates_in_tier(crate::metrics::CriticalityTier::Supporting)
            .len();
        let experimental_count = metrics
            .crates_in_tier(crate::metrics::CriticalityTier::Experimental)
            .len();

        let summary = ReportSummary {
            total_crates: graph.total_crates,
            critical_count,
            supporting_count,
            experimental_count,
            violation_count: layers.violations.len(),
            cycle_count: graph.cycles.len(),
            max_depth: graph.max_depth(),
            bottleneck: metrics.bottleneck_crate.clone(),
            max_fan_in: metrics.max_fan_in,
            avg_instability: metrics.avg_instability,
            graph_density: metrics.graph_density,
            health,
        };

        Self {
            summary,
            graph,
            layers,
            metrics,
        }
    }

    /// Determine overall workspace health from analysis results.
    fn compute_health(
        graph: &DependencyGraph,
        layers: &LayerMap,
        metrics: &WorkspaceMetrics,
    ) -> HealthStatus {
        // Critical: dependency cycles exist
        if graph.has_cycles() {
            return HealthStatus::Critical;
        }

        // Critical: severe boundary violations (crossing 2+ layers)
        if layers.violations.iter().any(|v| v.severity >= 2) {
            return HealthStatus::Critical;
        }

        // Warning: any boundary violations
        if layers.has_violations() {
            return HealthStatus::Warning;
        }

        // Warning: high graph density (over-coupled)
        if metrics.graph_density > 0.3 {
            return HealthStatus::Warning;
        }

        // Warning: extreme bottleneck (single crate > 50% fan-in)
        let total = graph.total_crates.max(1) as f64;
        if metrics.max_fan_in as f64 / total > 0.5 {
            return HealthStatus::Warning;
        }

        HealthStatus::Healthy
    }

    /// Serialize the report to JSON.
    ///
    /// # Errors
    /// Returns error if serialization fails.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Serialize the report to compact JSON.
    ///
    /// # Errors
    /// Returns error if serialization fails.
    pub fn to_json_compact(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::CrateNode;
    use std::collections::HashMap;

    fn healthy_graph() -> DependencyGraph {
        // A sparse graph with 5 nodes and 4 edges → density = 4/(5*4) = 0.2 < 0.3
        let mut nodes = HashMap::new();
        nodes.insert(
            "foundation".to_string(),
            CrateNode {
                name: "foundation".to_string(),
                fan_in: 2,
                fan_out: 0,
                dependents: vec!["domain-a".to_string(), "domain-b".to_string()],
                dependencies: vec![],
                topo_depth: 0,
            },
        );
        nodes.insert(
            "domain-a".to_string(),
            CrateNode {
                name: "domain-a".to_string(),
                fan_in: 1,
                fan_out: 1,
                dependents: vec!["service".to_string()],
                dependencies: vec!["foundation".to_string()],
                topo_depth: 1,
            },
        );
        nodes.insert(
            "domain-b".to_string(),
            CrateNode {
                name: "domain-b".to_string(),
                fan_in: 1,
                fan_out: 1,
                dependents: vec!["service".to_string()],
                dependencies: vec!["foundation".to_string()],
                topo_depth: 1,
            },
        );
        nodes.insert(
            "service".to_string(),
            CrateNode {
                name: "service".to_string(),
                fan_in: 0,
                fan_out: 2,
                dependents: vec![],
                dependencies: vec!["domain-a".to_string(), "domain-b".to_string()],
                topo_depth: 2,
            },
        );
        nodes.insert(
            "isolated".to_string(),
            CrateNode {
                name: "isolated".to_string(),
                fan_in: 0,
                fan_out: 0,
                dependents: vec![],
                dependencies: vec![],
                topo_depth: 0,
            },
        );

        DependencyGraph {
            nodes,
            cycles: vec![],
            topo_order: vec![
                "foundation".to_string(),
                "isolated".to_string(),
                "domain-a".to_string(),
                "domain-b".to_string(),
                "service".to_string(),
            ],
            total_crates: 5,
        }
    }

    #[test]
    fn test_healthy_report() {
        let graph = healthy_graph();
        let report = AnatomyReport::from_graph(graph);

        assert_eq!(report.summary.health, HealthStatus::Healthy);
        assert_eq!(report.summary.total_crates, 5);
        assert_eq!(report.summary.cycle_count, 0);
        assert_eq!(report.summary.violation_count, 0);
    }

    #[test]
    fn test_critical_with_cycles() {
        let mut graph = healthy_graph();
        graph.cycles = vec![vec!["a".to_string(), "b".to_string()]];

        let report = AnatomyReport::from_graph(graph);
        assert_eq!(report.summary.health, HealthStatus::Critical);
    }

    #[test]
    fn test_json_serialization() {
        let graph = healthy_graph();
        let report = AnatomyReport::from_graph(graph);

        let json = report.to_json();
        assert!(json.is_ok());

        let json_str = json.unwrap_or_default();
        assert!(json_str.contains("\"health\""));
        assert!(json_str.contains("\"total_crates\""));
    }

    #[test]
    fn test_json_compact_serialization() {
        let graph = healthy_graph();
        let report = AnatomyReport::from_graph(graph);

        let compact = report.to_json_compact();
        assert!(compact.is_ok());

        // Compact should have no newlines within the JSON body
        let compact_str = compact.unwrap_or_default();
        assert!(!compact_str.is_empty());
    }

    #[test]
    fn test_health_status_labels() {
        assert_eq!(HealthStatus::Healthy.label(), "Healthy");
        assert_eq!(HealthStatus::Warning.label(), "Warning");
        assert_eq!(HealthStatus::Critical.label(), "Critical");
    }
}
