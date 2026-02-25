//! # Layer Classification & Boundary Enforcement
//!
//! Classifies workspace crates into architectural layers and detects
//! dependency direction violations.
//!
//! ## Primitive Foundation
//! - σ (Sequence): Layer ordering (Foundation → Domain → Orchestration → Service)
//! - κ (Comparison): Boundary violation detection (reverse dep check)
//! - μ (Mapping): Crate name → layer classification
//! - ∂ (Boundary): Layer boundaries, violation detection

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::graph::DependencyGraph;

/// Architectural layer in the workspace dependency hierarchy.
///
/// Tier: T2-P (σ + Σ)
///
/// Ordering: Foundation(0) < Domain(1) < Orchestration(2) < Service(3)
/// Valid dependency direction: higher layers may depend on lower layers.
/// Reverse dependencies are violations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Layer {
    /// Foundation crates: primitives, constants, math, shared types.
    Foundation = 0,
    /// Domain crates: business logic, PV, guardian, signal processing.
    Domain = 1,
    /// Orchestration crates: brain, friday, event bus, coordination.
    Orchestration = 2,
    /// Service crates: MCP server, REST API, CLI.
    Service = 3,
}

impl Layer {
    /// Returns the numeric rank (0-3) of this layer.
    #[must_use]
    pub const fn rank(self) -> u8 {
        self as u8
    }

    /// Returns a human-readable label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Foundation => "Foundation",
            Self::Domain => "Domain",
            Self::Orchestration => "Orchestration",
            Self::Service => "Service",
        }
    }
}

/// A dependency that crosses layer boundaries in the wrong direction.
///
/// Tier: T2-C (κ + ∂ + σ + μ)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BoundaryViolation {
    /// The crate with the reverse dependency.
    pub from_crate: String,
    /// The crate being depended upon (in a higher layer).
    pub to_crate: String,
    /// Layer of the depending crate.
    pub from_layer: Layer,
    /// Layer of the dependency target.
    pub to_layer: Layer,
    /// Severity: how many layers the violation crosses.
    pub severity: u8,
}

/// Layer classification for the entire workspace.
///
/// Tier: T3 (σ + μ + κ + ∂ + Σ + N)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LayerMap {
    /// Crate name → assigned layer.
    pub assignments: HashMap<String, Layer>,
    /// Detected boundary violations (should be empty in healthy workspace).
    pub violations: Vec<BoundaryViolation>,
    /// Count of crates per layer.
    pub layer_counts: HashMap<String, usize>,
}

/// Default classification rules based on crate name patterns.
///
/// This uses the NexCore naming conventions documented in CLAUDE.md:
/// - Service: `nexcore-mcp`, `nexcore-api`, `nexcore-cli`
/// - Orchestration: `nexcore-brain`, `nexcore-friday`, `nexcore-cortex`
/// - Foundation: `nexcore-primitives`, `nexcore-lex-primitiva`, `nexcore-constants`, `stem-*`
/// - Domain: everything else in the workspace
#[must_use]
pub fn classify_crate(name: &str) -> Layer {
    // Service layer: external-facing binaries
    if matches!(
        name,
        "nexcore-mcp"
            | "nexcore-api"
            | "nexcore-cli"
            | "nexcore-docs-mcp"
            | "nexcore-nvrepl"
            | "nexcore-claude-hooks"
            | "nexcore-shell"
            | "nexcore-compositor"
            | "nexcore-init"
    ) {
        return Layer::Service;
    }

    // Orchestration layer: coordination and event routing
    if matches!(
        name,
        "nexcore-brain"
            | "nexcore-friday"
            | "nexcore-cortex"
            | "nexcore-energy"
            | "nexcore-sentinel"
            | "nexcore-vigil"
            | "nexcore-core"
            | "nexcore-os"
            | "nexcore-os-demo"
    ) {
        return Layer::Orchestration;
    }

    // Foundation layer: primitives, constants, math, shared infrastructure
    let foundation_prefixes = [
        "nexcore-primitives",
        "nexcore-lex-primitiva",
        "nexcore-constants",
        "nexcore-id",
        "nexcore-aggregate",
        "nexcore-measure",
        "nexcore-edit-distance",
        "nexcore-grammar-lab",
        "nexcore-macros",
        "nexcore-macros-core",
        "nexcore-hormones",
        "nexcore-signal-types",
        "nexcore-state-theory",
        "nexcore-config",
        "nexcore-anatomy",
    ];

    if foundation_prefixes.contains(&name) || name.starts_with("stem-") {
        return Layer::Foundation;
    }

    // Default: Domain layer
    Layer::Domain
}

impl LayerMap {
    /// Build layer classification from a dependency graph.
    ///
    /// Classifies each crate, then scans for boundary violations
    /// where a lower-layer crate depends on a higher-layer crate.
    pub fn from_graph(graph: &DependencyGraph) -> Self {
        let mut assignments = HashMap::new();
        let mut layer_counts: HashMap<String, usize> = HashMap::new();

        // Classify all crates
        for name in graph.nodes.keys() {
            let layer = classify_crate(name);
            assignments.insert(name.clone(), layer);
            *layer_counts.entry(layer.label().to_string()).or_insert(0) += 1;
        }

        // Detect boundary violations
        let mut violations = Vec::new();
        for (name, node) in &graph.nodes {
            let from_layer = assignments.get(name).copied().unwrap_or(Layer::Domain);

            for dep in &node.dependencies {
                let to_layer = assignments.get(dep).copied().unwrap_or(Layer::Domain);

                // Violation: depending on a HIGHER layer (lower rank depends on higher rank)
                // Valid: Service(3) → Foundation(0)
                // Invalid: Foundation(0) → Service(3)
                if from_layer.rank() < to_layer.rank() {
                    violations.push(BoundaryViolation {
                        from_crate: name.clone(),
                        to_crate: dep.clone(),
                        from_layer,
                        to_layer,
                        severity: to_layer.rank() - from_layer.rank(),
                    });
                }
            }
        }

        // Sort violations by severity (worst first)
        violations.sort_by(|a, b| b.severity.cmp(&a.severity));

        Self {
            assignments,
            violations,
            layer_counts,
        }
    }

    /// Check if the workspace has any boundary violations.
    #[must_use]
    pub fn has_violations(&self) -> bool {
        !self.violations.is_empty()
    }

    /// Get all crates assigned to a specific layer.
    #[must_use]
    pub fn crates_in_layer(&self, layer: Layer) -> Vec<&str> {
        let mut crates: Vec<&str> = self
            .assignments
            .iter()
            .filter(|(_, l)| **l == layer)
            .map(|(name, _)| name.as_str())
            .collect();
        crates.sort_unstable();
        crates
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_ordering() {
        assert!(Layer::Foundation < Layer::Domain);
        assert!(Layer::Domain < Layer::Orchestration);
        assert!(Layer::Orchestration < Layer::Service);
    }

    #[test]
    fn test_classify_known_crates() {
        assert_eq!(classify_crate("nexcore-mcp"), Layer::Service);
        assert_eq!(classify_crate("nexcore-api"), Layer::Service);
        assert_eq!(classify_crate("nexcore-cli"), Layer::Service);
        assert_eq!(classify_crate("nexcore-brain"), Layer::Orchestration);
        assert_eq!(classify_crate("nexcore-friday"), Layer::Orchestration);
        assert_eq!(classify_crate("nexcore-primitives"), Layer::Foundation);
        assert_eq!(classify_crate("nexcore-lex-primitiva"), Layer::Foundation);
        assert_eq!(classify_crate("stem-core"), Layer::Foundation);
        assert_eq!(classify_crate("stem-derive"), Layer::Foundation);
        assert_eq!(classify_crate("nexcore-vigilance"), Layer::Domain);
        assert_eq!(classify_crate("nexcore-signal"), Layer::Domain);
    }

    #[test]
    fn test_layer_rank_and_label() {
        assert_eq!(Layer::Foundation.rank(), 0);
        assert_eq!(Layer::Service.rank(), 3);
        assert_eq!(Layer::Foundation.label(), "Foundation");
        assert_eq!(Layer::Service.label(), "Service");
    }

    #[test]
    fn test_boundary_violation_detection() {
        use crate::graph::CrateNode;

        let mut nodes = HashMap::new();
        // Foundation crate depending on Service crate = VIOLATION
        nodes.insert(
            "nexcore-primitives".to_string(),
            CrateNode {
                name: "nexcore-primitives".to_string(),
                fan_in: 0,
                fan_out: 1,
                dependents: vec![],
                dependencies: vec!["nexcore-mcp".to_string()],
                topo_depth: 0,
            },
        );
        nodes.insert(
            "nexcore-mcp".to_string(),
            CrateNode {
                name: "nexcore-mcp".to_string(),
                fan_in: 1,
                fan_out: 0,
                dependents: vec!["nexcore-primitives".to_string()],
                dependencies: vec![],
                topo_depth: 0,
            },
        );

        let graph = DependencyGraph {
            nodes,
            cycles: vec![],
            topo_order: vec!["nexcore-mcp".to_string(), "nexcore-primitives".to_string()],
            total_crates: 2,
        };

        let layer_map = LayerMap::from_graph(&graph);
        assert!(layer_map.has_violations());
        assert_eq!(layer_map.violations.len(), 1);
        assert_eq!(layer_map.violations[0].from_crate, "nexcore-primitives");
        assert_eq!(layer_map.violations[0].to_crate, "nexcore-mcp");
        assert_eq!(layer_map.violations[0].severity, 3); // Foundation(0) → Service(3)
    }

    #[test]
    fn test_valid_dependency_no_violation() {
        use crate::graph::CrateNode;

        let mut nodes = HashMap::new();
        // Service depending on Foundation = VALID
        nodes.insert(
            "nexcore-mcp".to_string(),
            CrateNode {
                name: "nexcore-mcp".to_string(),
                fan_in: 0,
                fan_out: 1,
                dependents: vec![],
                dependencies: vec!["nexcore-primitives".to_string()],
                topo_depth: 1,
            },
        );
        nodes.insert(
            "nexcore-primitives".to_string(),
            CrateNode {
                name: "nexcore-primitives".to_string(),
                fan_in: 1,
                fan_out: 0,
                dependents: vec!["nexcore-mcp".to_string()],
                dependencies: vec![],
                topo_depth: 0,
            },
        );

        let graph = DependencyGraph {
            nodes,
            cycles: vec![],
            topo_order: vec!["nexcore-primitives".to_string(), "nexcore-mcp".to_string()],
            total_crates: 2,
        };

        let layer_map = LayerMap::from_graph(&graph);
        assert!(!layer_map.has_violations());
    }
}
