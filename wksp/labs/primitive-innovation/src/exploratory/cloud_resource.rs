// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # CloudResourceGraph
//!
//! **Tier**: T2-C (Σ + ρ + κ + N + λ)
//! **Dominant**: Σ (Sum)
//! **Bridge**: aggregate × cloud resource topology
//! **Confidence**: 0.83
//!
//! Recursive fold operations on hierarchical cloud resource trees.
//! Models: org → projects → regions → VMs → containers.
//!
//! Aggregates cost, count, and utilization from leaves to root.
//! Detects outliers via IQR at each hierarchy level.

use core::fmt;
use std::collections::BTreeMap;

/// Unique resource node identifier.
pub type ResourceId = u32;

/// Kind of cloud resource at each hierarchy level.
///
/// ## Tier: T2-P (λ + κ)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResourceKind {
    /// Top-level organization.
    Organization,
    /// Project within an org.
    Project,
    /// Geographic region.
    Region,
    /// Virtual machine.
    Vm,
    /// Container within a VM.
    Container,
    /// Serverless function.
    Function,
}

/// Metrics for a single resource.
///
/// ## Tier: T2-P (N + Σ)
#[derive(Debug, Clone, Copy, Default)]
pub struct ResourceMetrics {
    /// Monthly cost in dollars.
    pub cost: f64,
    /// CPU utilization (0.0 to 1.0).
    pub cpu_utilization: f64,
    /// Memory utilization (0.0 to 1.0).
    pub memory_utilization: f64,
    /// Network egress in GB.
    pub network_egress_gb: f64,
}

impl ResourceMetrics {
    /// Combine two metrics (add costs, average utilizations).
    #[must_use]
    pub fn combine(&self, other: &Self) -> Self {
        Self {
            cost: self.cost + other.cost,
            cpu_utilization: (self.cpu_utilization + other.cpu_utilization) / 2.0,
            memory_utilization: (self.memory_utilization + other.memory_utilization) / 2.0,
            network_egress_gb: self.network_egress_gb + other.network_egress_gb,
        }
    }

    /// Fold an iterator of metrics into one aggregate.
    pub fn fold_all(metrics: impl Iterator<Item = Self>) -> Self {
        let collected: Vec<Self> = metrics.collect();
        let count = collected.len();

        if count == 0 {
            return Self::default();
        }

        let total_cost: f64 = collected.iter().map(|m| m.cost).sum();
        let avg_cpu: f64 = collected.iter().map(|m| m.cpu_utilization).sum::<f64>() / count as f64;
        let avg_mem: f64 =
            collected.iter().map(|m| m.memory_utilization).sum::<f64>() / count as f64;
        let total_net: f64 = collected.iter().map(|m| m.network_egress_gb).sum();

        Self {
            cost: total_cost,
            cpu_utilization: avg_cpu,
            memory_utilization: avg_mem,
            network_egress_gb: total_net,
        }
    }
}

/// A node in the resource tree.
#[derive(Debug, Clone)]
pub struct ResourceNode {
    /// Node identifier.
    pub id: ResourceId,
    /// Human-readable name.
    pub name: String,
    /// Resource kind.
    pub kind: ResourceKind,
    /// Direct metrics (leaf nodes have direct; parents get aggregated).
    pub metrics: ResourceMetrics,
    /// Children node IDs.
    pub children: Vec<ResourceId>,
    /// Parent node ID (None for root).
    pub parent: Option<ResourceId>,
}

/// Outlier detection result.
#[derive(Debug, Clone)]
pub struct Outlier {
    /// The outlier resource.
    pub resource_id: ResourceId,
    /// Resource name.
    pub name: String,
    /// The metric that was outlying.
    pub metric_name: String,
    /// The value.
    pub value: f64,
    /// The upper fence (Q3 + 1.5*IQR).
    pub upper_fence: f64,
}

/// Aggregate summary at a tree level.
#[derive(Debug, Clone, Default)]
pub struct LevelSummary {
    /// Level kind.
    pub kind: Option<ResourceKind>,
    /// Number of nodes at this level.
    pub count: usize,
    /// Total cost.
    pub total_cost: f64,
    /// Average CPU utilization.
    pub avg_cpu: f64,
    /// Average memory utilization.
    pub avg_memory: f64,
    /// Total network egress.
    pub total_network_gb: f64,
}

/// Recursive fold over hierarchical cloud resource topology.
///
/// ## Tier: T2-C (Σ + ρ + κ + N + λ)
/// Dominant: Σ (Sum) — recursive aggregation is the core operation
///
/// Primitives:
/// - Σ: fold_subtree aggregation, cost/utilization rollup
/// - ρ: Recursive tree traversal from leaves to root
/// - κ: IQR outlier comparison, metric ranking
/// - N: Cost, utilization percentages, network GB
/// - λ: Resource hierarchy (org → project → region → VM → container)
#[derive(Debug, Clone)]
pub struct CloudResourceGraph {
    /// All resource nodes by ID.
    nodes: BTreeMap<ResourceId, ResourceNode>,
    /// Root node ID.
    root: Option<ResourceId>,
    /// Next node ID.
    next_id: ResourceId,
}

impl CloudResourceGraph {
    /// Create an empty resource graph.
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes: BTreeMap::new(),
            root: None,
            next_id: 0,
        }
    }

    /// Add a resource node. First node becomes root.
    pub fn add_resource(
        &mut self,
        name: impl Into<String>,
        kind: ResourceKind,
        metrics: ResourceMetrics,
        parent: Option<ResourceId>,
    ) -> ResourceId {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);

        let node = ResourceNode {
            id,
            name: name.into(),
            kind,
            metrics,
            children: Vec::new(),
            parent,
        };

        self.nodes.insert(id, node);

        // Register as child of parent
        if let Some(pid) = parent
            && let Some(parent_node) = self.nodes.get_mut(&pid)
        {
            parent_node.children.push(id);
        }

        // First node becomes root
        if self.root.is_none() {
            self.root = Some(id);
        }

        id
    }

    /// Recursively aggregate metrics from leaves to a given node (Σ + ρ).
    ///
    /// Returns the folded metrics for the subtree rooted at `node_id`.
    #[must_use]
    pub fn fold_subtree(&self, node_id: ResourceId) -> ResourceMetrics {
        let Some(node) = self.nodes.get(&node_id) else {
            return ResourceMetrics::default();
        };

        if node.children.is_empty() {
            // Leaf: return own metrics
            return node.metrics;
        }

        // Recursive fold over children
        let child_metrics = node.children.iter().map(|&cid| self.fold_subtree(cid));

        ResourceMetrics::fold_all(child_metrics)
    }

    /// Aggregate entire tree from root.
    #[must_use]
    pub fn total_metrics(&self) -> ResourceMetrics {
        match self.root {
            Some(root_id) => self.fold_subtree(root_id),
            None => ResourceMetrics::default(),
        }
    }

    /// Summarize metrics at each hierarchy level.
    #[must_use]
    pub fn level_summaries(&self) -> Vec<LevelSummary> {
        let mut by_kind: BTreeMap<ResourceKind, Vec<&ResourceNode>> = BTreeMap::new();

        for node in self.nodes.values() {
            by_kind.entry(node.kind).or_default().push(node);
        }

        by_kind
            .into_iter()
            .map(|(kind, nodes)| {
                let count = nodes.len();
                let total_cost: f64 = nodes.iter().map(|n| n.metrics.cost).sum();
                let avg_cpu = if count > 0 {
                    nodes.iter().map(|n| n.metrics.cpu_utilization).sum::<f64>() / count as f64
                } else {
                    0.0
                };
                let avg_memory = if count > 0 {
                    nodes
                        .iter()
                        .map(|n| n.metrics.memory_utilization)
                        .sum::<f64>()
                        / count as f64
                } else {
                    0.0
                };
                let total_network_gb: f64 = nodes.iter().map(|n| n.metrics.network_egress_gb).sum();

                LevelSummary {
                    kind: Some(kind),
                    count,
                    total_cost,
                    avg_cpu,
                    avg_memory,
                    total_network_gb,
                }
            })
            .collect()
    }

    /// Detect cost outliers among siblings using IQR method (κ primitive).
    ///
    /// Returns resources whose cost exceeds Q3 + 1.5 * IQR among their siblings.
    #[must_use]
    pub fn detect_cost_outliers(&self) -> Vec<Outlier> {
        let mut outliers = Vec::new();

        // For each parent, check children for outliers
        for node in self.nodes.values() {
            if node.children.len() < 4 {
                continue; // Need at least 4 siblings for meaningful IQR
            }

            let mut costs: Vec<(ResourceId, f64)> = node
                .children
                .iter()
                .filter_map(|&cid| {
                    self.nodes
                        .get(&cid)
                        .map(|_n| (cid, self.fold_subtree(cid).cost))
                })
                .collect();

            costs.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(core::cmp::Ordering::Equal));

            let n = costs.len();
            let q1 = costs[n / 4].1;
            let q3 = costs[3 * n / 4].1;
            let iqr = q3 - q1;
            let upper_fence = q3 + 1.5 * iqr;

            for &(rid, cost) in &costs {
                if cost > upper_fence {
                    let name = self
                        .nodes
                        .get(&rid)
                        .map(|n| n.name.clone())
                        .unwrap_or_default();

                    outliers.push(Outlier {
                        resource_id: rid,
                        name,
                        metric_name: "cost".into(),
                        value: cost,
                        upper_fence,
                    });
                }
            }
        }

        outliers
    }

    /// Find the most expensive leaf resource.
    #[must_use]
    pub fn most_expensive_leaf(&self) -> Option<(ResourceId, f64)> {
        self.nodes
            .values()
            .filter(|n| n.children.is_empty())
            .max_by(|a, b| {
                a.metrics
                    .cost
                    .partial_cmp(&b.metrics.cost)
                    .unwrap_or(core::cmp::Ordering::Equal)
            })
            .map(|n| (n.id, n.metrics.cost))
    }

    /// Get resource by ID.
    #[must_use]
    pub fn resource(&self, id: ResourceId) -> Option<&ResourceNode> {
        self.nodes.get(&id)
    }

    /// Total number of resources.
    #[must_use]
    pub fn resource_count(&self) -> usize {
        self.nodes.len()
    }

    /// Count of leaf nodes (no children).
    #[must_use]
    pub fn leaf_count(&self) -> usize {
        self.nodes
            .values()
            .filter(|n| n.children.is_empty())
            .count()
    }

    /// Maximum depth of the tree.
    #[must_use]
    pub fn max_depth(&self) -> usize {
        match self.root {
            Some(rid) => self.depth_of(rid),
            None => 0,
        }
    }

    /// Recursive depth calculation.
    fn depth_of(&self, node_id: ResourceId) -> usize {
        let Some(node) = self.nodes.get(&node_id) else {
            return 0;
        };

        if node.children.is_empty() {
            return 1;
        }

        let max_child = node
            .children
            .iter()
            .map(|&cid| self.depth_of(cid))
            .max()
            .unwrap_or(0);

        1 + max_child
    }
}

impl Default for CloudResourceGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for CloudResourceGraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total = self.total_metrics();
        write!(
            f,
            "CloudResourceGraph({} resources, {} leaves, ${:.2} total cost)",
            self.resource_count(),
            self.leaf_count(),
            total.cost,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_test_cloud() -> CloudResourceGraph {
        let mut g = CloudResourceGraph::new();

        let org = g.add_resource(
            "acme-corp",
            ResourceKind::Organization,
            ResourceMetrics::default(),
            None,
        );

        let proj = g.add_resource(
            "prod",
            ResourceKind::Project,
            ResourceMetrics::default(),
            Some(org),
        );

        let region = g.add_resource(
            "us-east-1",
            ResourceKind::Region,
            ResourceMetrics::default(),
            Some(proj),
        );

        g.add_resource(
            "vm-web-1",
            ResourceKind::Vm,
            ResourceMetrics {
                cost: 100.0,
                cpu_utilization: 0.6,
                memory_utilization: 0.7,
                network_egress_gb: 50.0,
            },
            Some(region),
        );

        g.add_resource(
            "vm-web-2",
            ResourceKind::Vm,
            ResourceMetrics {
                cost: 100.0,
                cpu_utilization: 0.4,
                memory_utilization: 0.5,
                network_egress_gb: 30.0,
            },
            Some(region),
        );

        g.add_resource(
            "vm-db-1",
            ResourceKind::Vm,
            ResourceMetrics {
                cost: 250.0,
                cpu_utilization: 0.8,
                memory_utilization: 0.9,
                network_egress_gb: 10.0,
            },
            Some(region),
        );

        g
    }

    #[test]
    fn test_fold_subtree_aggregation() {
        let g = build_test_cloud();
        let total = g.total_metrics();

        // Total cost should be sum of leaf VMs
        assert!((total.cost - 450.0).abs() < 0.01);
        // Total network should be sum
        assert!((total.network_egress_gb - 90.0).abs() < 0.01);
    }

    #[test]
    fn test_level_summaries() {
        let g = build_test_cloud();
        let summaries = g.level_summaries();

        // Should have: Organization, Project, Region, Vm
        assert_eq!(summaries.len(), 4);

        // Find VM summary
        let vm_summary = summaries.iter().find(|s| s.kind == Some(ResourceKind::Vm));
        assert!(vm_summary.is_some());
        let default_summary = LevelSummary::default();
        let vs = vm_summary.unwrap_or(&default_summary);
        assert_eq!(vs.count, 3);
        assert!((vs.total_cost - 450.0).abs() < 0.01);
    }

    #[test]
    fn test_most_expensive_leaf() {
        let g = build_test_cloud();
        let (_, cost) = g.most_expensive_leaf().unwrap_or((0, 0.0));
        assert!((cost - 250.0).abs() < 0.01); // vm-db-1
    }

    #[test]
    fn test_tree_structure() {
        let g = build_test_cloud();
        assert_eq!(g.resource_count(), 6);
        assert_eq!(g.leaf_count(), 3);
        assert_eq!(g.max_depth(), 4); // org -> proj -> region -> vm
    }

    #[test]
    fn test_outlier_detection_with_enough_siblings() {
        let mut g = CloudResourceGraph::new();
        let root = g.add_resource(
            "root",
            ResourceKind::Organization,
            ResourceMetrics::default(),
            None,
        );

        // Add 5 children: 4 normal + 1 outlier
        for i in 0..4 {
            g.add_resource(
                format!("normal-{i}"),
                ResourceKind::Vm,
                ResourceMetrics {
                    cost: 100.0,
                    ..ResourceMetrics::default()
                },
                Some(root),
            );
        }

        g.add_resource(
            "expensive",
            ResourceKind::Vm,
            ResourceMetrics {
                cost: 1000.0,
                ..ResourceMetrics::default()
            },
            Some(root),
        );

        let outliers = g.detect_cost_outliers();
        assert!(!outliers.is_empty());
        assert_eq!(outliers[0].name, "expensive");
    }
}
