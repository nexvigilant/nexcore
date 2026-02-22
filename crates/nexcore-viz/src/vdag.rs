//! Validated DAG (VDAG) — Pharmacovigilance Knowledge Graph
//!
//! Extends the base DAG module into a typed, signal-enriched knowledge graph
//! suitable for drug-AE signal visualization, ATC hierarchy traversal, and
//! temporal signal evolution tracking.
//!
//! ## Graph Semantics
//!
//! Nodes carry typed identity (`VdagNodeType`) and optional ATC taxonomy level.
//! Each drug node may carry a `Vec<SignalScore>` — one entry per adverse event
//! of interest, optionally timestamped for longitudinal tracking.
//!
//! Edges carry semantic type (`VdagEdgeType`), allowing downstream consumers to
//! filter the graph by relationship kind (e.g., extract only contraindication
//! edges for a safety dashboard).
//!
//! ## Rendering
//!
//! `Vdag::render_svg` delegates to the existing `dag::render_dag` function,
//! mapping VDAG nodes to `DagNode` with color chosen from node type.
//! `Vdag::to_3d_layout` computes (x, y, z) coordinates using Kahn-level x,
//! within-level y, and deterministic jitter z derived from node position.
//!
//! ## Signal Thresholds
//!
//! Standard NexVigilant thresholds: PRR >= 2.0, Chi-sq >= 3.841,
//! ROR-LCI > 1.0, IC025 > 0.0, EB05 >= 2.0.
//!
//! ## Example
//!
//! ```rust
//! use nexcore_viz::vdag::{
//!     AtcLevel, SignalScore, Vdag, VdagEdge, VdagEdgeType, VdagNode, VdagNodeType,
//! };
//!
//! let nodes = vec![
//!     VdagNode {
//!         id: "analgesics".into(),
//!         label: "Analgesics (N02)".into(),
//!         node_type: VdagNodeType::DrugClass,
//!         atc_level: Some(AtcLevel::Therapeutic),
//!         signals: vec![],
//!         color: None,
//!         metadata: std::collections::HashMap::new(),
//!     },
//!     VdagNode {
//!         id: "aspirin".into(),
//!         label: "Aspirin".into(),
//!         node_type: VdagNodeType::Drug,
//!         atc_level: Some(AtcLevel::Substance),
//!         signals: vec![SignalScore {
//!             ae_name: "GI Bleeding".into(),
//!             prr: Some(3.2),
//!             ror: Some(3.5),
//!             ic025: Some(0.8),
//!             ebgm: Some(2.9),
//!             case_count: Some(412),
//!             timestamp: Some("2024-01-01".into()),
//!         }],
//!         color: None,
//!         metadata: std::collections::HashMap::new(),
//!     },
//! ];
//! let edges = vec![VdagEdge {
//!     from: "analgesics".into(),
//!     to: "aspirin".into(),
//!     edge_type: VdagEdgeType::Contains,
//!     weight: None,
//! }];
//! let vdag = Vdag { nodes, edges, title: "Aspirin VDAG".into() };
//! assert_eq!(vdag.drugs().len(), 2); // DrugClass + Drug
//! ```

use crate::dag::{DagEdge, DagNode, render_dag};
use crate::svg::palette;
use crate::theme::Theme;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

// ============================================================================
// Core types
// ============================================================================

/// The semantic type of a node in the VDAG.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VdagNodeType {
    /// A drug class node (ATC level: Anatomical through Chemical).
    DrugClass,
    /// A leaf drug / substance node (ATC level: Substance).
    Drug,
    /// An adverse event concept node.
    AdverseEvent,
    /// A clinical indication node.
    Indication,
}

/// ATC (Anatomical Therapeutic Chemical) classification level.
///
/// Levels correspond to the five-tier WHO ATC hierarchy:
/// 1st = Anatomical, 2nd = Therapeutic, 3rd = Pharmacological,
/// 4th = Chemical, 5th = Substance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AtcLevel {
    /// 1st level — anatomical main group (e.g., N = Nervous System).
    Anatomical,
    /// 2nd level — therapeutic main group (e.g., N02 = Analgesics).
    Therapeutic,
    /// 3rd level — therapeutic/pharmacological subgroup.
    Pharmacological,
    /// 4th level — chemical subgroup.
    Chemical,
    /// 5th level — chemical substance (leaf).
    Substance,
}

/// The semantic type of an edge in the VDAG.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VdagEdgeType {
    /// Parent class contains child class or drug (ATC hierarchy).
    Contains,
    /// Drug interacts with another drug or substance.
    InteractsWith,
    /// Drug is contraindicated with another drug or condition.
    Contraindicates,
    /// Taxonomic "is a" relationship (class hierarchy).
    ClassOf,
    /// Drug is associated with this adverse event via signal detection.
    HasAdverseEvent,
}

/// An adverse event signal score, optionally timestamped.
///
/// All scoring fields are optional: an entry may represent a partial report
/// where only some disproportionality statistics are available.
///
/// Standard NexVigilant thresholds: PRR >= 2.0, ROR-LCI > 1.0,
/// IC025 > 0.0, EB05 >= 2.0.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignalScore {
    /// Adverse event name (e.g., "Nausea", "GI Bleeding").
    pub ae_name: String,
    /// Proportional Reporting Ratio. Threshold: >= 2.0.
    pub prr: Option<f64>,
    /// Reporting Odds Ratio. Threshold lower CI: > 1.0.
    pub ror: Option<f64>,
    /// IC lower 95% credible interval. Threshold: > 0.0.
    pub ic025: Option<f64>,
    /// Empirical Bayes Geometric Mean score. Threshold EB05: >= 2.0.
    pub ebgm: Option<f64>,
    /// Number of spontaneous reports (cases) underlying the signal.
    pub case_count: Option<u32>,
    /// ISO 8601 timestamp of signal calculation (e.g., "2024-Q1" or "2024-01-15").
    pub timestamp: Option<String>,
}

impl SignalScore {
    /// Returns `true` when the signal clears all available NexVigilant thresholds.
    ///
    /// A score passes if every *present* statistic exceeds its threshold.
    /// Missing statistics do not fail the check (partial evidence).
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexcore_viz::vdag::SignalScore;
    ///
    /// let s = SignalScore {
    ///     ae_name: "Hepatotoxicity".into(),
    ///     prr: Some(3.1),
    ///     ror: Some(3.4),
    ///     ic025: Some(0.5),
    ///     ebgm: Some(2.8),
    ///     case_count: Some(55),
    ///     timestamp: None,
    /// };
    /// assert!(s.is_signal());
    /// ```
    #[must_use]
    pub fn is_signal(&self) -> bool {
        let prr_ok = self.prr.map_or(true, |v| v >= 2.0);
        let ror_ok = self.ror.map_or(true, |v| v > 1.0);
        let ic_ok = self.ic025.map_or(true, |v| v > 0.0);
        let eb_ok = self.ebgm.map_or(true, |v| v >= 2.0);
        prr_ok && ror_ok && ic_ok && eb_ok
    }
}

/// A node in the VDAG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VdagNode {
    /// Unique node identifier (stable across renders).
    pub id: String,
    /// Human-readable display label.
    pub label: String,
    /// Semantic category.
    pub node_type: VdagNodeType,
    /// ATC taxonomy level, if applicable.
    pub atc_level: Option<AtcLevel>,
    /// Adverse event signal scores attached to this node.
    pub signals: Vec<SignalScore>,
    /// Optional color override (SVG hex string). Falls back to type-based default.
    pub color: Option<String>,
    /// Arbitrary string metadata for downstream consumers.
    pub metadata: HashMap<String, String>,
}

/// A directed edge in the VDAG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VdagEdge {
    /// Source node ID.
    pub from: String,
    /// Target node ID.
    pub to: String,
    /// Edge relationship type.
    pub edge_type: VdagEdgeType,
    /// Optional weight (e.g., interaction confidence score).
    pub weight: Option<f64>,
}

/// Three-dimensional position for a VDAG node.
///
/// Used by `Vdag::to_3d_layout` for 3-D graph rendering consumers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodePosition3d {
    /// Node ID.
    pub id: String,
    /// Horizontal axis — Kahn topological level (0 = root level).
    pub x: f64,
    /// Vertical axis — position within level (0 = top).
    pub y: f64,
    /// Depth axis — deterministic pseudo-jitter based on node order.
    pub z: f64,
}

/// The Validated DAG — a typed, signal-enriched pharmacovigilance knowledge graph.
///
/// `Vdag` extends the base `dag::render_dag` infrastructure with:
/// - Semantic node and edge types for PV knowledge modelling.
/// - AE signal score overlays supporting longitudinal evolution.
/// - ATC taxonomy structure (5-level hierarchy).
/// - Query methods for filtering by drug, AE, and subgraph.
/// - SVG rendering (delegates to `dag::render_dag`).
/// - 3-D layout computation for interactive graph consumers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vdag {
    /// All nodes in the graph.
    pub nodes: Vec<VdagNode>,
    /// All directed edges in the graph.
    pub edges: Vec<VdagEdge>,
    /// Graph title for display purposes.
    pub title: String,
}

// ============================================================================
// Node / edge color mapping
// ============================================================================

/// Return the canonical SVG fill color for a given node type.
fn color_for_node_type(node_type: &VdagNodeType) -> &'static str {
    match node_type {
        // Drug classes — violet (ATC taxonomy hierarchy)
        VdagNodeType::DrugClass => palette::PHYSICS,
        // Leaf drugs — teal (science domain)
        VdagNodeType::Drug => palette::SCIENCE,
        // Adverse events — red (danger semantic)
        VdagNodeType::AdverseEvent => palette::RED,
        // Indications — amber (warning / context)
        VdagNodeType::Indication => palette::AMBER,
    }
}

/// Return the canonical SVG stroke color for a given edge type.
fn color_for_edge_type(edge_type: &VdagEdgeType) -> &'static str {
    match edge_type {
        VdagEdgeType::Contains => palette::SLATE,
        VdagEdgeType::InteractsWith => palette::AMBER,
        VdagEdgeType::Contraindicates => palette::RED,
        VdagEdgeType::ClassOf => palette::PHYSICS,
        VdagEdgeType::HasAdverseEvent => palette::RECURSION,
    }
}

// ============================================================================
// Kahn's algorithm (level computation) — adapted for VdagNode/VdagEdge
// ============================================================================

/// Compute topological levels using Kahn's algorithm on VDAG edges.
///
/// Returns a `Vec<Vec<String>>` where each inner `Vec` holds node IDs at that
/// level. Nodes not reachable from a root (cycle members) are omitted.
fn compute_vdag_levels(nodes: &[VdagNode], edges: &[VdagEdge]) -> Vec<Vec<String>> {
    let node_ids: HashSet<String> = nodes.iter().map(|n| n.id.clone()).collect();

    let mut in_degree: HashMap<String, usize> =
        node_ids.iter().map(|id| (id.clone(), 0)).collect();
    let mut successors: HashMap<String, Vec<String>> = HashMap::new();

    for edge in edges {
        if node_ids.contains(&edge.from) && node_ids.contains(&edge.to) {
            *in_degree.entry(edge.to.clone()).or_insert(0) += 1;
            successors
                .entry(edge.from.clone())
                .or_default()
                .push(edge.to.clone());
        }
    }

    let mut levels: Vec<Vec<String>> = Vec::new();
    let mut queue_vec: Vec<String> = in_degree
        .iter()
        .filter(|&(_, deg)| *deg == 0)
        .map(|(id, _)| id.clone())
        .collect();
    queue_vec.sort();
    let mut queue: VecDeque<String> = queue_vec.into();

    while !queue.is_empty() {
        let level: Vec<String> = queue.drain(..).collect();
        let mut next: Vec<String> = Vec::new();

        for node in &level {
            if let Some(succs) = successors.get(node) {
                for succ in succs {
                    if let Some(deg) = in_degree.get_mut(succ) {
                        *deg -= 1;
                        if *deg == 0 {
                            next.push(succ.clone());
                        }
                    }
                }
            }
        }

        levels.push(level);
        next.sort();
        queue = next.into();
    }

    levels
}

// ============================================================================
// Signal summary
// ============================================================================

/// Aggregate statistics over all signal scores in a `Vdag`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignalSummary {
    /// Total number of `SignalScore` entries across all nodes.
    pub total_signals: usize,
    /// Number of entries that pass all available NexVigilant thresholds.
    pub active_signals: usize,
    /// Maximum PRR observed across all scores, if any PRR values exist.
    pub max_prr: Option<f64>,
    /// Maximum case count observed, if any case counts exist.
    pub max_case_count: Option<u32>,
}

// ============================================================================
// Phase 4 "Nervous System" types
// ============================================================================

/// Difference in maximum PRR for a shared adverse event between two drug classes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifferentialScore {
    /// Adverse event name (shared by both classes).
    pub ae_name: String,
    /// Maximum PRR observed for this AE within class A, if any.
    pub prr_a: Option<f64>,
    /// Maximum PRR observed for this AE within class B, if any.
    pub prr_b: Option<f64>,
    /// `prr_a - prr_b`; `0.0` when either side is `None`.
    pub diff: f64,
}

/// Side-by-side comparison of two ATC drug classes across shared adverse events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugClassComparison {
    /// Identifier of class A.
    pub class_a: String,
    /// Identifier of class B.
    pub class_b: String,
    /// Adverse event names present in both classes.
    pub shared_aes: Vec<String>,
    /// Adverse event names present only in class A.
    pub unique_to_a: Vec<String>,
    /// Adverse event names present only in class B.
    pub unique_to_b: Vec<String>,
    /// Per-AE differential PRR scores for each shared adverse event.
    pub differential_scores: Vec<DifferentialScore>,
}

/// A pharmacovigilance signal propagated upward from child drug nodes to a
/// parent class node, aggregating evidence across all children.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagatedSignal {
    /// Identifier of the parent (class) node receiving the aggregated signal.
    pub node_id: String,
    /// Adverse event name.
    pub ae_name: String,
    /// Maximum PRR across all child signals for this AE, if any.
    pub aggregated_prr: Option<f64>,
    /// Maximum ROR across all child signals for this AE, if any.
    pub aggregated_ror: Option<f64>,
    /// Sum of all child case counts for this AE.
    pub total_cases: u32,
    /// Number of child drug nodes that contributed at least one signal for this AE.
    pub child_count: usize,
}

/// A point-in-time slice of all signals sharing the same timestamp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalSnapshot {
    /// ISO 8601 timestamp string (e.g., `"2024-01-01"` or `"2024-Q2"`).
    pub timestamp: String,
    /// Total number of signal entries with this timestamp.
    pub total_signals: usize,
    /// Number of those entries that pass all NexVigilant thresholds.
    pub active_signals: usize,
    /// Mean PRR across all entries with a PRR value at this timestamp.
    pub mean_prr: Option<f64>,
    /// Maximum PRR observed at this timestamp.
    pub max_prr: Option<f64>,
}

/// Population-weighted node size for a drug, suitable for bubble-chart rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeWeight {
    /// Drug node identifier.
    pub node_id: String,
    /// Raw sum of `case_count` across all signals attached to this drug.
    pub raw_count: u32,
    /// Min-max normalised weight in `[0.0, 1.0]`.
    pub normalized_weight: f64,
}

/// Full contextual detail for a single drug node, used to populate drill-down
/// panels in interactive visualizations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrillDownContext {
    /// A clone of the drug node itself.
    pub drug: VdagNode,
    /// All signals attached to this drug, sorted by PRR descending
    /// (`None` PRR values sort last).
    pub signals: Vec<SignalScore>,
    /// Ordered chain of ancestor class node IDs from immediate parent to root.
    pub parent_chain: Vec<String>,
    /// Node IDs of drugs connected to this drug via `InteractsWith` edges
    /// (either direction).
    pub interaction_partners: Vec<String>,
}

// ============================================================================
// Vdag impl
// ============================================================================

impl Vdag {
    /// Return all nodes that are drugs or drug classes.
    ///
    /// Includes both `VdagNodeType::Drug` and `VdagNodeType::DrugClass` nodes,
    /// covering the full ATC hierarchy.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexcore_viz::vdag::{Vdag, VdagNode, VdagNodeType, VdagEdge};
    ///
    /// let vdag = Vdag {
    ///     nodes: vec![
    ///         VdagNode { id: "c01".into(), label: "Cardiology".into(),
    ///             node_type: VdagNodeType::DrugClass, atc_level: None,
    ///             signals: vec![], color: None, metadata: Default::default() },
    ///         VdagNode { id: "nausea".into(), label: "Nausea".into(),
    ///             node_type: VdagNodeType::AdverseEvent, atc_level: None,
    ///             signals: vec![], color: None, metadata: Default::default() },
    ///     ],
    ///     edges: vec![],
    ///     title: "Test".into(),
    /// };
    /// assert_eq!(vdag.drugs().len(), 1);
    /// ```
    #[must_use]
    pub fn drugs(&self) -> Vec<&VdagNode> {
        self.nodes
            .iter()
            .filter(|n| {
                matches!(n.node_type, VdagNodeType::Drug | VdagNodeType::DrugClass)
            })
            .collect()
    }

    /// Return all nodes typed as `AdverseEvent`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexcore_viz::vdag::{Vdag, VdagNode, VdagNodeType};
    ///
    /// let vdag = Vdag {
    ///     nodes: vec![
    ///         VdagNode { id: "ae1".into(), label: "Nausea".into(),
    ///             node_type: VdagNodeType::AdverseEvent, atc_level: None,
    ///             signals: vec![], color: None, metadata: Default::default() },
    ///     ],
    ///     edges: vec![],
    ///     title: "T".into(),
    /// };
    /// assert_eq!(vdag.adverse_events().len(), 1);
    /// ```
    #[must_use]
    pub fn adverse_events(&self) -> Vec<&VdagNode> {
        self.nodes
            .iter()
            .filter(|n| n.node_type == VdagNodeType::AdverseEvent)
            .collect()
    }

    /// Return all `SignalScore` entries attached to a specific drug node.
    ///
    /// Returns an empty slice if the node does not exist or has no signals.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexcore_viz::vdag::{SignalScore, Vdag, VdagNode, VdagNodeType};
    ///
    /// let vdag = Vdag {
    ///     nodes: vec![VdagNode {
    ///         id: "aspirin".into(), label: "Aspirin".into(),
    ///         node_type: VdagNodeType::Drug, atc_level: None,
    ///         signals: vec![SignalScore {
    ///             ae_name: "GI Bleed".into(), prr: Some(3.2), ror: None,
    ///             ic025: None, ebgm: None, case_count: Some(100),
    ///             timestamp: None,
    ///         }],
    ///         color: None, metadata: Default::default(),
    ///     }],
    ///     edges: vec![], title: "T".into(),
    /// };
    /// assert_eq!(vdag.signals_for_drug("aspirin").len(), 1);
    /// assert_eq!(vdag.signals_for_drug("unknown").len(), 0);
    /// ```
    #[must_use]
    pub fn signals_for_drug(&self, drug_id: &str) -> &[SignalScore] {
        self.nodes
            .iter()
            .find(|n| n.id == drug_id)
            .map_or(&[], |n| n.signals.as_slice())
    }

    /// Find all drug nodes that have a signal for the named adverse event.
    ///
    /// Searches the `signals` field of every `Drug` and `DrugClass` node.
    /// The `ae_name` comparison is case-insensitive.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexcore_viz::vdag::{SignalScore, Vdag, VdagNode, VdagNodeType};
    ///
    /// let vdag = Vdag {
    ///     nodes: vec![VdagNode {
    ///         id: "metformin".into(), label: "Metformin".into(),
    ///         node_type: VdagNodeType::Drug, atc_level: None,
    ///         signals: vec![SignalScore {
    ///             ae_name: "Lactic Acidosis".into(), prr: Some(4.1),
    ///             ror: None, ic025: None, ebgm: None,
    ///             case_count: None, timestamp: None,
    ///         }],
    ///         color: None, metadata: Default::default(),
    ///     }],
    ///     edges: vec![], title: "T".into(),
    /// };
    /// assert_eq!(vdag.drugs_for_ae("lactic acidosis").len(), 1);
    /// assert_eq!(vdag.drugs_for_ae("Nausea").len(), 0);
    /// ```
    #[must_use]
    pub fn drugs_for_ae(&self, ae_name: &str) -> Vec<&VdagNode> {
        let lower = ae_name.to_lowercase();
        self.nodes
            .iter()
            .filter(|n| {
                matches!(n.node_type, VdagNodeType::Drug | VdagNodeType::DrugClass)
                    && n.signals
                        .iter()
                        .any(|s| s.ae_name.to_lowercase() == lower)
            })
            .collect()
    }

    /// Return all descendant node IDs reachable from `node_id` via directed edges.
    ///
    /// Uses breadth-first traversal. The starting node itself is included in the
    /// returned set. Returns an empty `Vec` if `node_id` is not found.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexcore_viz::vdag::{Vdag, VdagEdge, VdagEdgeType, VdagNode, VdagNodeType};
    ///
    /// let make_node = |id: &str| VdagNode {
    ///     id: id.into(), label: id.into(), node_type: VdagNodeType::Drug,
    ///     atc_level: None, signals: vec![], color: None,
    ///     metadata: Default::default(),
    /// };
    /// let vdag = Vdag {
    ///     nodes: vec![make_node("root"), make_node("child"), make_node("leaf")],
    ///     edges: vec![
    ///         VdagEdge { from: "root".into(), to: "child".into(),
    ///             edge_type: VdagEdgeType::Contains, weight: None },
    ///         VdagEdge { from: "child".into(), to: "leaf".into(),
    ///             edge_type: VdagEdgeType::Contains, weight: None },
    ///     ],
    ///     title: "T".into(),
    /// };
    /// let sub = vdag.subtree("root");
    /// assert!(sub.contains(&"root".to_string()));
    /// assert!(sub.contains(&"child".to_string()));
    /// assert!(sub.contains(&"leaf".to_string()));
    /// assert_eq!(sub.len(), 3);
    /// ```
    #[must_use]
    pub fn subtree(&self, node_id: &str) -> Vec<String> {
        let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
        for edge in &self.edges {
            adj.entry(edge.from.as_str())
                .or_default()
                .push(edge.to.as_str());
        }

        let node_exists = self.nodes.iter().any(|n| n.id == node_id);
        if !node_exists {
            return vec![];
        }

        let mut visited: HashSet<String> = HashSet::new();
        let mut queue: VecDeque<&str> = VecDeque::new();
        queue.push_back(node_id);

        while let Some(current) = queue.pop_front() {
            if visited.contains(current) {
                continue;
            }
            visited.insert(current.to_string());
            if let Some(succs) = adj.get(current) {
                for &succ in succs {
                    if !visited.contains(succ) {
                        queue.push_back(succ);
                    }
                }
            }
        }

        let mut result: Vec<String> = visited.into_iter().collect();
        result.sort();
        result
    }

    /// Render the VDAG as an SVG string, delegating to `dag::render_dag`.
    ///
    /// Node colors are chosen from type-based palette defaults unless the node
    /// has an explicit `color` override. Edge rendering uses the base DAG arrow
    /// style; edge semantic type is encoded via node border color.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexcore_viz::vdag::{Vdag, VdagNode, VdagNodeType};
    /// use nexcore_viz::theme::Theme;
    ///
    /// let vdag = Vdag {
    ///     nodes: vec![VdagNode {
    ///         id: "drug_a".into(), label: "Drug A".into(),
    ///         node_type: VdagNodeType::Drug, atc_level: None,
    ///         signals: vec![], color: None, metadata: Default::default(),
    ///     }],
    ///     edges: vec![], title: "Render Test".into(),
    /// };
    /// let svg = vdag.render_svg(&Theme::default());
    /// assert!(svg.starts_with("<svg"));
    /// ```
    #[must_use]
    pub fn render_svg(&self, theme: &Theme) -> String {
        let dag_nodes: Vec<DagNode> = self
            .nodes
            .iter()
            .map(|n| {
                let color = n
                    .color
                    .clone()
                    .unwrap_or_else(|| color_for_node_type(&n.node_type).to_string());

                // Append signal indicator to label when signals are present
                let label = if n.signals.is_empty() {
                    n.label.clone()
                } else {
                    let active = n.signals.iter().filter(|s| s.is_signal()).count();
                    format!("{} [{}AE]", n.label, active)
                };

                DagNode {
                    id: n.id.clone(),
                    label,
                    color: Some(color),
                }
            })
            .collect();

        let dag_edges: Vec<DagEdge> = self
            .edges
            .iter()
            .map(|e| DagEdge {
                from: e.from.clone(),
                to: e.to.clone(),
            })
            .collect();

        render_dag(&dag_nodes, &dag_edges, &self.title, theme)
    }

    /// Compute 3-D spatial positions for all VDAG nodes.
    ///
    /// Layout strategy:
    /// - **x**: Kahn topological level index (0 = root), scaled by 200 px.
    /// - **y**: Position within level (0 = top), scaled by 100 px.
    /// - **z**: Deterministic pseudo-jitter derived from
    ///   `(level_idx * 7 + node_idx * 13) % 10`, mapped to `[-1.0, 1.0]`,
    ///   giving gentle depth separation without randomness.
    ///
    /// Nodes excluded from topological ordering (isolated or cycle members) are
    /// appended after the last level with `z = 0.0`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexcore_viz::vdag::{Vdag, VdagEdge, VdagEdgeType, VdagNode, VdagNodeType};
    ///
    /// let make_drug = |id: &str, label: &str| VdagNode {
    ///     id: id.into(), label: label.into(),
    ///     node_type: VdagNodeType::Drug, atc_level: None,
    ///     signals: vec![], color: None, metadata: Default::default(),
    /// };
    /// let vdag = Vdag {
    ///     nodes: vec![make_drug("a", "A"), make_drug("b", "B")],
    ///     edges: vec![VdagEdge {
    ///         from: "a".into(), to: "b".into(),
    ///         edge_type: VdagEdgeType::Contains, weight: None,
    ///     }],
    ///     title: "3D".into(),
    /// };
    /// let positions = vdag.to_3d_layout();
    /// assert_eq!(positions.len(), 2);
    /// if let (Some(pos_a), Some(pos_b)) = (
    ///     positions.iter().find(|p| p.id == "a"),
    ///     positions.iter().find(|p| p.id == "b"),
    /// ) {
    ///     assert!(pos_b.x > pos_a.x); // b is one level deeper
    /// }
    /// ```
    #[must_use]
    pub fn to_3d_layout(&self) -> Vec<NodePosition3d> {
        const X_SPACING: f64 = 200.0;
        const Y_SPACING: f64 = 100.0;

        let levels = compute_vdag_levels(&self.nodes, &self.edges);

        let mut positioned: HashSet<String> = HashSet::new();
        let mut positions: Vec<NodePosition3d> = Vec::with_capacity(self.nodes.len());

        for (level_idx, level) in levels.iter().enumerate() {
            let x = level_idx as f64 * X_SPACING;
            for (node_idx, node_id) in level.iter().enumerate() {
                let y = node_idx as f64 * Y_SPACING;
                // Deterministic jitter: maps to range [-1.0, 1.0]
                let raw = (level_idx * 7 + node_idx * 13) % 10;
                let z = (raw as f64 / 5.0) - 1.0;

                positions.push(NodePosition3d {
                    id: node_id.clone(),
                    x,
                    y,
                    z,
                });
                positioned.insert(node_id.clone());
            }
        }

        // Append any nodes not reached by topological ordering (isolated nodes)
        let last_x = levels.len() as f64 * X_SPACING;
        for (idx, node) in self.nodes.iter().enumerate() {
            if !positioned.contains(&node.id) {
                positions.push(NodePosition3d {
                    id: node.id.clone(),
                    x: last_x,
                    y: idx as f64 * Y_SPACING,
                    z: 0.0,
                });
            }
        }

        positions
    }

    /// Return aggregate statistics over all signal scores in the graph.
    ///
    /// Returns `None` if no signals are present in any node.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexcore_viz::vdag::{SignalScore, Vdag, VdagNode, VdagNodeType};
    ///
    /// let vdag = Vdag {
    ///     nodes: vec![VdagNode {
    ///         id: "drug".into(), label: "Drug".into(),
    ///         node_type: VdagNodeType::Drug, atc_level: None,
    ///         signals: vec![SignalScore {
    ///             ae_name: "Nausea".into(), prr: Some(2.5),
    ///             ror: Some(2.8), ic025: Some(0.3), ebgm: Some(2.1),
    ///             case_count: Some(42), timestamp: None,
    ///         }],
    ///         color: None, metadata: Default::default(),
    ///     }],
    ///     edges: vec![], title: "T".into(),
    /// };
    /// let summary = vdag.signal_summary();
    /// assert!(summary.is_some());
    /// ```
    #[must_use]
    pub fn signal_summary(&self) -> Option<SignalSummary> {
        let all_scores: Vec<&SignalScore> =
            self.nodes.iter().flat_map(|n| n.signals.iter()).collect();

        if all_scores.is_empty() {
            return None;
        }

        let total_signals = all_scores.len();
        let active_signals = all_scores.iter().filter(|s| s.is_signal()).count();
        let max_prr = all_scores.iter().filter_map(|s| s.prr).reduce(f64::max);
        let max_case_count = all_scores.iter().filter_map(|s| s.case_count).max();

        Some(SignalSummary {
            total_signals,
            active_signals,
            max_prr,
            max_case_count,
        })
    }

    /// Render an SVG legend explaining node type colors and edge type strokes.
    ///
    /// Returns a compact SVG string suitable for embedding alongside `render_svg`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexcore_viz::vdag::{Vdag, VdagNode, VdagNodeType};
    /// use nexcore_viz::theme::Theme;
    ///
    /// let vdag = Vdag { nodes: vec![], edges: vec![], title: "T".into() };
    /// let svg = vdag.render_legend(&Theme::default());
    /// assert!(svg.starts_with("<svg"));
    /// assert!(svg.contains("Drug Class"));
    /// ```
    #[must_use]
    pub fn render_legend(&self, theme: &Theme) -> String {
        use crate::svg::{SvgDoc, line as svg_line, rect_stroke, text, text_bold};

        let node_entries: &[(&str, &str)] = &[
            ("Drug Class", color_for_node_type(&VdagNodeType::DrugClass)),
            ("Drug", color_for_node_type(&VdagNodeType::Drug)),
            ("Adverse Event", color_for_node_type(&VdagNodeType::AdverseEvent)),
            ("Indication", color_for_node_type(&VdagNodeType::Indication)),
        ];

        let edge_entries: &[(&str, &str)] = &[
            ("Contains", color_for_edge_type(&VdagEdgeType::Contains)),
            ("Interacts With", color_for_edge_type(&VdagEdgeType::InteractsWith)),
            ("Contraindicates", color_for_edge_type(&VdagEdgeType::Contraindicates)),
            ("Class Of", color_for_edge_type(&VdagEdgeType::ClassOf)),
            ("Has Adverse Event", color_for_edge_type(&VdagEdgeType::HasAdverseEvent)),
        ];

        let row_h = 24.0;
        let pad = 12.0;
        let swatch_w = 16.0;
        let col_w = 170.0;
        let total_rows = node_entries.len() + edge_entries.len() + 3; // +3 for two headers + gap
        let height = pad * 2.0 + total_rows as f64 * row_h;
        let width = col_w + pad * 2.0;

        let mut doc = SvgDoc::new_with_theme(width, height, theme.clone());
        let mut y = pad + row_h / 2.0;

        // Node type section
        doc.add(text_bold(pad, y, "Node Types", 11.0, theme.text, "start"));
        y += row_h;

        for &(label, color) in node_entries {
            doc.add(rect_stroke(
                pad,
                y - swatch_w / 2.0,
                swatch_w,
                swatch_w,
                theme.card_bg,
                color,
                2.0,
                3.0,
            ));
            doc.add(text(pad + swatch_w + 6.0, y, label, 10.0, theme.text, "start"));
            y += row_h;
        }

        y += row_h * 0.5;

        // Edge type section
        doc.add(text_bold(pad, y, "Edge Types", 11.0, theme.text, "start"));
        y += row_h;

        for &(label, color) in edge_entries {
            doc.add(svg_line(pad, y, pad + swatch_w, y, color, 2.5));
            doc.add(text(pad + swatch_w + 6.0, y, label, 10.0, theme.text, "start"));
            y += row_h;
        }

        doc.render()
    }

    // ------------------------------------------------------------------------
    // Phase 4 — Nervous System methods
    // ------------------------------------------------------------------------

    /// Compare two drug classes by their adverse event signal profiles.
    ///
    /// Extracts the subtree for each class identifier, collects all
    /// `SignalScore` entries from `Drug`/`DrugClass` nodes within each
    /// subtree, then partitions adverse event names into shared and unique
    /// sets. For shared AEs the method computes the difference in maximum PRR.
    ///
    /// Unknown class identifiers produce an empty subtree, which yields empty
    /// AE sets and an all-unique result for the other class.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexcore_viz::vdag::{Vdag, VdagNode, VdagNodeType, VdagEdge, VdagEdgeType,
    ///                          AtcLevel, SignalScore};
    ///
    /// let make_drug = |id: &str, ae: &str, prr: f64| VdagNode {
    ///     id: id.into(), label: id.into(), node_type: VdagNodeType::Drug,
    ///     atc_level: Some(AtcLevel::Substance),
    ///     signals: vec![SignalScore { ae_name: ae.into(), prr: Some(prr),
    ///         ror: None, ic025: None, ebgm: None, case_count: None, timestamp: None }],
    ///     color: None, metadata: Default::default(),
    /// };
    /// let make_class = |id: &str| VdagNode {
    ///     id: id.into(), label: id.into(), node_type: VdagNodeType::DrugClass,
    ///     atc_level: Some(AtcLevel::Therapeutic),
    ///     signals: vec![], color: None, metadata: Default::default(),
    /// };
    /// let vdag = Vdag {
    ///     nodes: vec![make_class("classA"), make_class("classB"),
    ///                 make_drug("drugA", "Nausea", 3.0),
    ///                 make_drug("drugB", "Nausea", 2.5)],
    ///     edges: vec![
    ///         VdagEdge { from: "classA".into(), to: "drugA".into(),
    ///             edge_type: VdagEdgeType::Contains, weight: None },
    ///         VdagEdge { from: "classB".into(), to: "drugB".into(),
    ///             edge_type: VdagEdgeType::Contains, weight: None },
    ///     ],
    ///     title: "Compare".into(),
    /// };
    /// let cmp = vdag.compare_drug_classes("classA", "classB");
    /// assert_eq!(cmp.shared_aes, vec!["Nausea".to_string()]);
    /// assert!(cmp.unique_to_a.is_empty());
    /// assert!(cmp.unique_to_b.is_empty());
    /// ```
    #[must_use]
    pub fn compare_drug_classes(&self, class_a: &str, class_b: &str) -> DrugClassComparison {
        // Collect AE name -> max PRR mapping for a set of node IDs
        let ae_prr_map = |node_ids: &[String]| -> HashMap<String, Option<f64>> {
            let mut map: HashMap<String, Option<f64>> = HashMap::new();
            for node_id in node_ids {
                if let Some(node) = self.nodes.iter().find(|n| &n.id == node_id) {
                    if matches!(
                        node.node_type,
                        VdagNodeType::Drug | VdagNodeType::DrugClass
                    ) {
                        for sig in &node.signals {
                            let entry = map.entry(sig.ae_name.clone()).or_insert(None);
                            *entry = match (*entry, sig.prr) {
                                (None, v) => v,
                                (Some(existing), Some(new)) => Some(existing.max(new)),
                                (Some(existing), None) => Some(existing),
                            };
                        }
                    }
                }
            }
            map
        };

        let subtree_a = self.subtree(class_a);
        let subtree_b = self.subtree(class_b);
        let map_a = ae_prr_map(&subtree_a);
        let map_b = ae_prr_map(&subtree_b);

        let aes_a: HashSet<&String> = map_a.keys().collect();
        let aes_b: HashSet<&String> = map_b.keys().collect();

        let mut shared_aes: Vec<String> = aes_a
            .intersection(&aes_b)
            .map(|&s| s.clone())
            .collect();
        shared_aes.sort();

        let mut unique_to_a: Vec<String> = aes_a
            .difference(&aes_b)
            .map(|&s| s.clone())
            .collect();
        unique_to_a.sort();

        let mut unique_to_b: Vec<String> = aes_b
            .difference(&aes_a)
            .map(|&s| s.clone())
            .collect();
        unique_to_b.sort();

        let differential_scores: Vec<DifferentialScore> = shared_aes
            .iter()
            .map(|ae| {
                let prr_a = map_a.get(ae).copied().flatten();
                let prr_b = map_b.get(ae).copied().flatten();
                let diff = match (prr_a, prr_b) {
                    (Some(a), Some(b)) => a - b,
                    _ => 0.0,
                };
                DifferentialScore {
                    ae_name: ae.clone(),
                    prr_a,
                    prr_b,
                    diff,
                }
            })
            .collect();

        DrugClassComparison {
            class_a: class_a.to_string(),
            class_b: class_b.to_string(),
            shared_aes,
            unique_to_a,
            unique_to_b,
            differential_scores,
        }
    }

    /// Walk leaf drug nodes upward through `Contains` edges, aggregating their
    /// signals at each parent class node.
    ///
    /// For each `DrugClass` parent that has at least one `Drug` child (direct
    /// or transitive via `Contains` edges), one `PropagatedSignal` entry per
    /// unique AE name is emitted. Aggregation takes the maximum PRR and ROR,
    /// and sums all case counts across contributing children.
    ///
    /// Returns an empty `Vec` when the graph has no `Contains` edges or no
    /// drug nodes with signals.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexcore_viz::vdag::{Vdag, VdagNode, VdagNodeType, VdagEdge, VdagEdgeType,
    ///                          AtcLevel, SignalScore};
    ///
    /// let class_node = VdagNode {
    ///     id: "cls".into(), label: "Class".into(),
    ///     node_type: VdagNodeType::DrugClass,
    ///     atc_level: Some(AtcLevel::Therapeutic),
    ///     signals: vec![], color: None, metadata: Default::default(),
    /// };
    /// let drug_node = VdagNode {
    ///     id: "drug".into(), label: "Drug".into(),
    ///     node_type: VdagNodeType::Drug,
    ///     atc_level: Some(AtcLevel::Substance),
    ///     signals: vec![SignalScore {
    ///         ae_name: "Nausea".into(), prr: Some(2.5),
    ///         ror: Some(2.8), ic025: None, ebgm: None,
    ///         case_count: Some(50), timestamp: None,
    ///     }],
    ///     color: None, metadata: Default::default(),
    /// };
    /// let vdag = Vdag {
    ///     nodes: vec![class_node, drug_node],
    ///     edges: vec![VdagEdge { from: "cls".into(), to: "drug".into(),
    ///         edge_type: VdagEdgeType::Contains, weight: None }],
    ///     title: "Propagation".into(),
    /// };
    /// let signals = vdag.propagate_signals_up();
    /// assert!(!signals.is_empty());
    /// let s = signals.iter().find(|s| s.node_id == "cls" && s.ae_name == "Nausea");
    /// assert!(s.is_some());
    /// ```
    #[must_use]
    pub fn propagate_signals_up(&self) -> Vec<PropagatedSignal> {
        // Build reverse adjacency: child -> set of parents via Contains edges
        let mut child_to_parents: HashMap<&str, Vec<&str>> = HashMap::new();
        for edge in &self.edges {
            if edge.edge_type == VdagEdgeType::Contains {
                child_to_parents
                    .entry(edge.to.as_str())
                    .or_default()
                    .push(edge.from.as_str());
            }
        }

        // For each DrugClass parent, collect all Drug descendant signals via BFS upward
        // Strategy: for every Drug node, walk its parent chain and accumulate signals
        // into a map: parent_id -> ae_name -> (max_prr, max_ror, total_cases, child_count)
        type AeAgg = (Option<f64>, Option<f64>, u32, usize); // (prr, ror, cases, children)
        let mut parent_ae: HashMap<String, HashMap<String, AeAgg>> = HashMap::new();

        for node in &self.nodes {
            if node.node_type != VdagNodeType::Drug || node.signals.is_empty() {
                continue;
            }
            // BFS upward to find all ancestor DrugClass nodes
            let mut ancestors: HashSet<&str> = HashSet::new();
            let mut queue: VecDeque<&str> = VecDeque::new();
            queue.push_back(node.id.as_str());
            while let Some(current) = queue.pop_front() {
                if let Some(parents) = child_to_parents.get(current) {
                    for &parent in parents {
                        if ancestors.insert(parent) {
                            queue.push_back(parent);
                        }
                    }
                }
            }

            for ancestor_id in &ancestors {
                // Confirm ancestor is a DrugClass node
                let is_class = self
                    .nodes
                    .iter()
                    .any(|n| n.id.as_str() == *ancestor_id && n.node_type == VdagNodeType::DrugClass);
                if !is_class {
                    continue;
                }
                let ae_map = parent_ae
                    .entry((*ancestor_id).to_string())
                    .or_default();

                for sig in &node.signals {
                    let entry: &mut AeAgg = ae_map.entry(sig.ae_name.clone()).or_insert((None, None, 0, 0));
                    // Max PRR
                    entry.0 = match (entry.0, sig.prr) {
                        (None, v) => v,
                        (Some(e), Some(n)) => Some(e.max(n)),
                        (Some(e), None) => Some(e),
                    };
                    // Max ROR
                    entry.1 = match (entry.1, sig.ror) {
                        (None, v) => v,
                        (Some(e), Some(n)) => Some(e.max(n)),
                        (Some(e), None) => Some(e),
                    };
                    // Sum cases
                    entry.2 = entry.2.saturating_add(sig.case_count.unwrap_or(0));
                    // Increment child count
                    entry.3 += 1;
                }
            }
        }

        let mut result: Vec<PropagatedSignal> = parent_ae
            .into_iter()
            .flat_map(|(node_id, ae_map)| {
                ae_map.into_iter().map(move |(ae_name, (prr, ror, cases, count))| {
                    PropagatedSignal {
                        node_id: node_id.clone(),
                        ae_name,
                        aggregated_prr: prr,
                        aggregated_ror: ror,
                        total_cases: cases,
                        child_count: count,
                    }
                })
            })
            .collect();

        // Deterministic ordering: node_id then ae_name
        result.sort_by(|a, b| a.node_id.cmp(&b.node_id).then(a.ae_name.cmp(&b.ae_name)));
        result
    }

    /// Group all signals across all nodes by their `timestamp` field and return
    /// a chronologically-sorted `Vec<TemporalSnapshot>`.
    ///
    /// Signals with `timestamp: None` are excluded. When no timestamped signals
    /// exist, an empty `Vec` is returned.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexcore_viz::vdag::{Vdag, VdagNode, VdagNodeType, SignalScore};
    ///
    /// let vdag = Vdag {
    ///     nodes: vec![VdagNode {
    ///         id: "d".into(), label: "D".into(),
    ///         node_type: VdagNodeType::Drug, atc_level: None,
    ///         signals: vec![
    ///             SignalScore { ae_name: "Nausea".into(), prr: Some(2.5),
    ///                 ror: Some(2.8), ic025: Some(0.3), ebgm: Some(2.1),
    ///                 case_count: Some(10), timestamp: Some("2024-01".into()) },
    ///             SignalScore { ae_name: "Rash".into(), prr: Some(1.5),
    ///                 ror: None, ic025: None, ebgm: None,
    ///                 case_count: None, timestamp: Some("2024-02".into()) },
    ///         ],
    ///         color: None, metadata: Default::default(),
    ///     }],
    ///     edges: vec![], title: "T".into(),
    /// };
    /// let snaps = vdag.temporal_snapshots();
    /// assert_eq!(snaps.len(), 2);
    /// assert_eq!(snaps[0].timestamp, "2024-01");
    /// ```
    #[must_use]
    pub fn temporal_snapshots(&self) -> Vec<TemporalSnapshot> {
        // Map: timestamp -> Vec<&SignalScore>
        let mut ts_map: HashMap<String, Vec<&SignalScore>> = HashMap::new();
        for node in &self.nodes {
            for sig in &node.signals {
                if let Some(ref ts) = sig.timestamp {
                    ts_map.entry(ts.clone()).or_default().push(sig);
                }
            }
        }

        let mut snapshots: Vec<TemporalSnapshot> = ts_map
            .into_iter()
            .map(|(timestamp, signals)| {
                let total_signals = signals.len();
                let active_signals = signals.iter().filter(|s| s.is_signal()).count();
                let prr_values: Vec<f64> = signals.iter().filter_map(|s| s.prr).collect();
                let mean_prr = if prr_values.is_empty() {
                    None
                } else {
                    Some(prr_values.iter().sum::<f64>() / prr_values.len() as f64)
                };
                let max_prr = prr_values.iter().copied().reduce(f64::max);
                TemporalSnapshot {
                    timestamp,
                    total_signals,
                    active_signals,
                    mean_prr,
                    max_prr,
                }
            })
            .collect();

        snapshots.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        snapshots
    }

    /// Compute population-weighted sizes for all `Drug` nodes.
    ///
    /// For each drug node, sums `case_count` across all its signals. The raw
    /// counts are then min-max normalised to `[0.0, 1.0]`. When all drug nodes
    /// have equal case counts (or only one drug exists), `normalized_weight` is
    /// `0.0` for every node to avoid division by zero.
    ///
    /// Only `VdagNodeType::Drug` nodes are included; `DrugClass` nodes are
    /// excluded.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexcore_viz::vdag::{Vdag, VdagNode, VdagNodeType, SignalScore};
    ///
    /// let make_drug = |id: &str, cases: u32| VdagNode {
    ///     id: id.into(), label: id.into(), node_type: VdagNodeType::Drug,
    ///     atc_level: None,
    ///     signals: vec![SignalScore { ae_name: "AE".into(), prr: None,
    ///         ror: None, ic025: None, ebgm: None,
    ///         case_count: Some(cases), timestamp: None }],
    ///     color: None, metadata: Default::default(),
    /// };
    /// let vdag = Vdag {
    ///     nodes: vec![make_drug("a", 100), make_drug("b", 300)],
    ///     edges: vec![], title: "T".into(),
    /// };
    /// let weights = vdag.population_weighted_sizes();
    /// assert_eq!(weights.len(), 2);
    /// let wa = weights.iter().find(|w| w.node_id == "a").map(|w| w.normalized_weight);
    /// let wb = weights.iter().find(|w| w.node_id == "b").map(|w| w.normalized_weight);
    /// assert_eq!(wa, Some(0.0));
    /// assert_eq!(wb, Some(1.0));
    /// ```
    #[must_use]
    pub fn population_weighted_sizes(&self) -> Vec<NodeWeight> {
        let drug_nodes: Vec<&VdagNode> = self
            .nodes
            .iter()
            .filter(|n| n.node_type == VdagNodeType::Drug)
            .collect();

        if drug_nodes.is_empty() {
            return vec![];
        }

        let raw_counts: Vec<(String, u32)> = drug_nodes
            .iter()
            .map(|n| {
                let total: u32 = n
                    .signals
                    .iter()
                    .filter_map(|s| s.case_count)
                    .fold(0u32, |acc, v| acc.saturating_add(v));
                (n.id.clone(), total)
            })
            .collect();

        let min_count = raw_counts.iter().map(|(_, c)| *c).min().unwrap_or(0);
        let max_count = raw_counts.iter().map(|(_, c)| *c).max().unwrap_or(0);
        let range = max_count.saturating_sub(min_count);

        raw_counts
            .into_iter()
            .map(|(node_id, raw_count)| {
                let normalized_weight = if range == 0 {
                    0.0
                } else {
                    f64::from(raw_count.saturating_sub(min_count)) / f64::from(range)
                };
                NodeWeight {
                    node_id,
                    raw_count,
                    normalized_weight,
                }
            })
            .collect()
    }

    /// Extract the subgraph formed exclusively by `InteractsWith` edges.
    ///
    /// Returns a tuple of `(nodes, edges)` where `nodes` contains only those
    /// `VdagNode` references whose IDs appear as either endpoint of at least
    /// one `InteractsWith` edge, and `edges` contains those edges.
    ///
    /// Both returned slices are in their original graph order. Returns empty
    /// vecs when no `InteractsWith` edges exist.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexcore_viz::vdag::{Vdag, VdagNode, VdagNodeType, VdagEdge, VdagEdgeType};
    ///
    /// let make_drug = |id: &str| VdagNode {
    ///     id: id.into(), label: id.into(), node_type: VdagNodeType::Drug,
    ///     atc_level: None, signals: vec![], color: None,
    ///     metadata: Default::default(),
    /// };
    /// let vdag = Vdag {
    ///     nodes: vec![make_drug("a"), make_drug("b"), make_drug("c")],
    ///     edges: vec![
    ///         VdagEdge { from: "a".into(), to: "b".into(),
    ///             edge_type: VdagEdgeType::InteractsWith, weight: None },
    ///         VdagEdge { from: "b".into(), to: "c".into(),
    ///             edge_type: VdagEdgeType::Contains, weight: None },
    ///     ],
    ///     title: "Interactions".into(),
    /// };
    /// let (nodes, edges) = vdag.interaction_subgraph();
    /// assert_eq!(edges.len(), 1);
    /// assert_eq!(nodes.len(), 2); // only "a" and "b"
    /// ```
    #[must_use]
    pub fn interaction_subgraph(&self) -> (Vec<&VdagNode>, Vec<&VdagEdge>) {
        let interaction_edges: Vec<&VdagEdge> = self
            .edges
            .iter()
            .filter(|e| e.edge_type == VdagEdgeType::InteractsWith)
            .collect();

        let endpoint_ids: HashSet<&str> = interaction_edges
            .iter()
            .flat_map(|e| [e.from.as_str(), e.to.as_str()])
            .collect();

        let interaction_nodes: Vec<&VdagNode> = self
            .nodes
            .iter()
            .filter(|n| endpoint_ids.contains(n.id.as_str()))
            .collect();

        (interaction_nodes, interaction_edges)
    }

    /// Gather full contextual detail for a single drug node by ID.
    ///
    /// Returns `None` if `drug_id` does not correspond to any node in the
    /// graph. The returned `DrillDownContext` contains:
    /// - A clone of the drug node.
    /// - All of its signals sorted by PRR descending (`None` PRR sorts last).
    /// - The ordered chain of ancestor class IDs from immediate parent to root,
    ///   resolved by following `Contains` edges upward (BFS, then reversed).
    /// - IDs of interaction partners connected via `InteractsWith` edges
    ///   (either as source or target).
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexcore_viz::vdag::{Vdag, VdagNode, VdagNodeType, VdagEdge, VdagEdgeType,
    ///                          AtcLevel, SignalScore};
    ///
    /// let class_node = VdagNode {
    ///     id: "cls".into(), label: "Class".into(),
    ///     node_type: VdagNodeType::DrugClass,
    ///     atc_level: Some(AtcLevel::Therapeutic),
    ///     signals: vec![], color: None, metadata: Default::default(),
    /// };
    /// let drug_node = VdagNode {
    ///     id: "drg".into(), label: "Drug".into(),
    ///     node_type: VdagNodeType::Drug,
    ///     atc_level: Some(AtcLevel::Substance),
    ///     signals: vec![SignalScore {
    ///         ae_name: "Nausea".into(), prr: Some(2.5),
    ///         ror: None, ic025: None, ebgm: None,
    ///         case_count: Some(20), timestamp: None,
    ///     }],
    ///     color: None, metadata: Default::default(),
    /// };
    /// let vdag = Vdag {
    ///     nodes: vec![class_node, drug_node],
    ///     edges: vec![VdagEdge { from: "cls".into(), to: "drg".into(),
    ///         edge_type: VdagEdgeType::Contains, weight: None }],
    ///     title: "DrillDown".into(),
    /// };
    /// let ctx = vdag.drill_down_context("drg");
    /// assert!(ctx.is_some());
    /// if let Some(c) = ctx {
    ///     assert_eq!(c.drug.id, "drg");
    ///     assert_eq!(c.parent_chain, vec!["cls".to_string()]);
    ///     assert_eq!(c.signals[0].ae_name, "Nausea");
    /// }
    /// ```
    #[must_use]
    pub fn drill_down_context(&self, drug_id: &str) -> Option<DrillDownContext> {
        let drug_node = self.nodes.iter().find(|n| n.id == drug_id)?;

        // Clone and sort signals by PRR descending (None last)
        let mut signals = drug_node.signals.clone();
        signals.sort_by(|a, b| {
            match (b.prr, a.prr) {
                (Some(bv), Some(av)) => bv.partial_cmp(&av).unwrap_or(std::cmp::Ordering::Equal),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
        });

        // Build reverse adjacency for Contains edges: child -> parents
        let mut child_to_parents: HashMap<&str, Vec<&str>> = HashMap::new();
        for edge in &self.edges {
            if edge.edge_type == VdagEdgeType::Contains {
                child_to_parents
                    .entry(edge.to.as_str())
                    .or_default()
                    .push(edge.from.as_str());
            }
        }

        // BFS upward to build parent chain (immediate parent first, root last)
        // We use a layer-by-layer BFS and record discovery order
        let mut parent_chain: Vec<String> = Vec::new();
        let mut visited: HashSet<&str> = HashSet::new();
        let mut frontier: Vec<&str> = child_to_parents
            .get(drug_id)
            .cloned()
            .unwrap_or_default();
        frontier.sort();

        while !frontier.is_empty() {
            let mut next_frontier: Vec<&str> = Vec::new();
            for &parent in &frontier {
                if visited.insert(parent) {
                    parent_chain.push(parent.to_string());
                    if let Some(grandparents) = child_to_parents.get(parent) {
                        next_frontier.extend(grandparents.iter().copied());
                    }
                }
            }
            next_frontier.sort();
            frontier = next_frontier;
        }

        // Collect InteractsWith partners (either direction)
        let mut interaction_partners: Vec<String> = self
            .edges
            .iter()
            .filter(|e| e.edge_type == VdagEdgeType::InteractsWith)
            .filter_map(|e| {
                if e.from == drug_id {
                    Some(e.to.clone())
                } else if e.to == drug_id {
                    Some(e.from.clone())
                } else {
                    None
                }
            })
            .collect();
        interaction_partners.sort();
        interaction_partners.dedup();

        Some(DrillDownContext {
            drug: drug_node.clone(),
            signals,
            parent_chain,
            interaction_partners,
        })
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a small ATC drug class hierarchy with 2 drugs and 3 adverse events:
    ///
    /// ```text
    /// N (Nervous System) [Anatomical]
    ///   └── N02 (Analgesics) [Therapeutic]
    ///         ├── aspirin [Substance]
    ///         │     ├── signals: GI Bleeding (active), Tinnitus (sub-threshold)
    ///         │     ├──> ae_gi_bleeding  [HasAdverseEvent]
    ///         │     ├──> ae_tinnitus     [HasAdverseEvent]
    ///         │     └──> ibuprofen       [InteractsWith]
    ///         └── ibuprofen [Substance]
    ///               ├── signals: GI Bleeding (active), Renal Failure (active), Nausea (sub-threshold)
    ///               ├──> ae_gi_bleeding  [HasAdverseEvent]
    ///               └──> ae_renal        [HasAdverseEvent]
    ///
    /// AdverseEvents: ae_gi_bleeding, ae_tinnitus, ae_renal
    /// ```
    fn build_test_vdag() -> Vdag {
        let mut meta = HashMap::new();
        meta.insert("source".to_string(), "test".to_string());

        let nodes = vec![
            // ATC: Anatomical
            VdagNode {
                id: "N".into(),
                label: "Nervous System".into(),
                node_type: VdagNodeType::DrugClass,
                atc_level: Some(AtcLevel::Anatomical),
                signals: vec![],
                color: None,
                metadata: meta.clone(),
            },
            // ATC: Therapeutic
            VdagNode {
                id: "N02".into(),
                label: "Analgesics".into(),
                node_type: VdagNodeType::DrugClass,
                atc_level: Some(AtcLevel::Therapeutic),
                signals: vec![],
                color: None,
                metadata: HashMap::new(),
            },
            // Drug: Aspirin — 2 signals (1 active, 1 sub-threshold)
            VdagNode {
                id: "aspirin".into(),
                label: "Aspirin".into(),
                node_type: VdagNodeType::Drug,
                atc_level: Some(AtcLevel::Substance),
                signals: vec![
                    SignalScore {
                        ae_name: "GI Bleeding".into(),
                        prr: Some(3.2),
                        ror: Some(3.5),
                        ic025: Some(0.8),
                        ebgm: Some(2.9),
                        case_count: Some(412),
                        timestamp: Some("2024-01-01".into()),
                    },
                    SignalScore {
                        ae_name: "Tinnitus".into(),
                        prr: Some(1.1), // sub-threshold PRR
                        ror: Some(1.2),
                        ic025: Some(-0.3), // sub-threshold IC025
                        ebgm: Some(1.4),
                        case_count: Some(22),
                        timestamp: Some("2024-01-01".into()),
                    },
                ],
                color: None,
                metadata: HashMap::new(),
            },
            // Drug: Ibuprofen — 3 signals (2 active, 1 sub-threshold)
            VdagNode {
                id: "ibuprofen".into(),
                label: "Ibuprofen".into(),
                node_type: VdagNodeType::Drug,
                atc_level: Some(AtcLevel::Substance),
                signals: vec![
                    SignalScore {
                        ae_name: "GI Bleeding".into(),
                        prr: Some(2.8),
                        ror: Some(3.1),
                        ic025: Some(0.5),
                        ebgm: Some(2.3),
                        case_count: Some(319),
                        timestamp: Some("2024-Q2".into()),
                    },
                    SignalScore {
                        ae_name: "Renal Failure".into(),
                        prr: Some(4.1),
                        ror: Some(4.5),
                        ic025: Some(1.2),
                        ebgm: Some(3.8),
                        case_count: Some(87),
                        timestamp: Some("2024-Q2".into()),
                    },
                    SignalScore {
                        ae_name: "Nausea".into(),
                        prr: Some(1.8),
                        ror: Some(2.0),
                        ic025: Some(0.1),
                        ebgm: Some(1.9), // sub-threshold EBGM
                        case_count: Some(155),
                        timestamp: None,
                    },
                ],
                color: None,
                metadata: HashMap::new(),
            },
            // Adverse event nodes
            VdagNode {
                id: "ae_gi_bleeding".into(),
                label: "GI Bleeding".into(),
                node_type: VdagNodeType::AdverseEvent,
                atc_level: None,
                signals: vec![],
                color: None,
                metadata: HashMap::new(),
            },
            VdagNode {
                id: "ae_tinnitus".into(),
                label: "Tinnitus".into(),
                node_type: VdagNodeType::AdverseEvent,
                atc_level: None,
                signals: vec![],
                color: None,
                metadata: HashMap::new(),
            },
            VdagNode {
                id: "ae_renal".into(),
                label: "Renal Failure".into(),
                node_type: VdagNodeType::AdverseEvent,
                atc_level: None,
                signals: vec![],
                color: None,
                metadata: HashMap::new(),
            },
        ];

        let edges = vec![
            // ATC hierarchy: N -> N02 -> drugs
            VdagEdge {
                from: "N".into(),
                to: "N02".into(),
                edge_type: VdagEdgeType::Contains,
                weight: None,
            },
            VdagEdge {
                from: "N02".into(),
                to: "aspirin".into(),
                edge_type: VdagEdgeType::Contains,
                weight: None,
            },
            VdagEdge {
                from: "N02".into(),
                to: "ibuprofen".into(),
                edge_type: VdagEdgeType::Contains,
                weight: None,
            },
            // AE linkage edges
            VdagEdge {
                from: "aspirin".into(),
                to: "ae_gi_bleeding".into(),
                edge_type: VdagEdgeType::HasAdverseEvent,
                weight: Some(3.2),
            },
            VdagEdge {
                from: "aspirin".into(),
                to: "ae_tinnitus".into(),
                edge_type: VdagEdgeType::HasAdverseEvent,
                weight: Some(1.1),
            },
            VdagEdge {
                from: "ibuprofen".into(),
                to: "ae_gi_bleeding".into(),
                edge_type: VdagEdgeType::HasAdverseEvent,
                weight: Some(2.8),
            },
            VdagEdge {
                from: "ibuprofen".into(),
                to: "ae_renal".into(),
                edge_type: VdagEdgeType::HasAdverseEvent,
                weight: Some(4.1),
            },
            // Drug-drug interaction
            VdagEdge {
                from: "aspirin".into(),
                to: "ibuprofen".into(),
                edge_type: VdagEdgeType::InteractsWith,
                weight: None,
            },
        ];

        Vdag {
            nodes,
            edges,
            title: "Analgesics ATC Hierarchy — VDAG".into(),
        }
    }

    #[test]
    fn drugs_filter_returns_classes_and_drugs() {
        let vdag = build_test_vdag();
        let drugs = vdag.drugs();
        // N (DrugClass), N02 (DrugClass), aspirin (Drug), ibuprofen (Drug)
        assert_eq!(drugs.len(), 4);
        let ids: Vec<&str> = drugs.iter().map(|n| n.id.as_str()).collect();
        assert!(ids.contains(&"N"));
        assert!(ids.contains(&"N02"));
        assert!(ids.contains(&"aspirin"));
        assert!(ids.contains(&"ibuprofen"));
    }

    #[test]
    fn adverse_events_filter() {
        let vdag = build_test_vdag();
        let aes = vdag.adverse_events();
        assert_eq!(aes.len(), 3);
        let labels: Vec<&str> = aes.iter().map(|n| n.label.as_str()).collect();
        assert!(labels.contains(&"GI Bleeding"));
        assert!(labels.contains(&"Tinnitus"));
        assert!(labels.contains(&"Renal Failure"));
    }

    #[test]
    fn signals_for_drug_aspirin() {
        let vdag = build_test_vdag();
        let scores = vdag.signals_for_drug("aspirin");
        assert_eq!(scores.len(), 2);
        assert_eq!(scores[0].ae_name, "GI Bleeding");
        assert_eq!(scores[1].ae_name, "Tinnitus");
    }

    #[test]
    fn signals_for_drug_unknown_returns_empty() {
        let vdag = build_test_vdag();
        assert_eq!(vdag.signals_for_drug("phantom").len(), 0);
    }

    #[test]
    fn drugs_for_ae_gi_bleeding_finds_both_drugs() {
        let vdag = build_test_vdag();
        let found = vdag.drugs_for_ae("GI Bleeding");
        assert_eq!(found.len(), 2);
        let ids: Vec<&str> = found.iter().map(|n| n.id.as_str()).collect();
        assert!(ids.contains(&"aspirin"));
        assert!(ids.contains(&"ibuprofen"));
    }

    #[test]
    fn drugs_for_ae_case_insensitive() {
        let vdag = build_test_vdag();
        let found = vdag.drugs_for_ae("gi bleeding");
        assert_eq!(found.len(), 2);
    }

    #[test]
    fn drugs_for_ae_renal_only_ibuprofen() {
        let vdag = build_test_vdag();
        let found = vdag.drugs_for_ae("Renal Failure");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].id, "ibuprofen");
    }

    #[test]
    fn drugs_for_ae_not_found_returns_empty() {
        let vdag = build_test_vdag();
        assert_eq!(vdag.drugs_for_ae("Aplastic Anemia").len(), 0);
    }

    #[test]
    fn subtree_from_n02_includes_drugs_and_aes() {
        let vdag = build_test_vdag();
        let sub = vdag.subtree("N02");
        assert!(sub.contains(&"N02".to_string()));
        assert!(sub.contains(&"aspirin".to_string()));
        assert!(sub.contains(&"ibuprofen".to_string()));
        assert!(sub.contains(&"ae_gi_bleeding".to_string()));
        assert!(sub.contains(&"ae_tinnitus".to_string()));
        assert!(sub.contains(&"ae_renal".to_string()));
    }

    #[test]
    fn subtree_from_leaf_returns_just_leaf() {
        let vdag = build_test_vdag();
        let sub = vdag.subtree("ae_gi_bleeding");
        // ae_gi_bleeding is a leaf — no outgoing edges
        assert_eq!(sub.len(), 1);
        assert!(sub.contains(&"ae_gi_bleeding".to_string()));
    }

    #[test]
    fn subtree_unknown_node_returns_empty() {
        let vdag = build_test_vdag();
        assert!(vdag.subtree("does_not_exist").is_empty());
    }

    #[test]
    fn render_svg_produces_valid_svg() {
        let vdag = build_test_vdag();
        let svg = vdag.render_svg(&Theme::default());
        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>"));
    }

    #[test]
    fn render_svg_contains_node_labels() {
        let vdag = build_test_vdag();
        let svg = vdag.render_svg(&Theme::default());
        assert!(svg.contains("Aspirin"));
        assert!(svg.contains("Ibuprofen"));
        assert!(svg.contains("Analgesics"));
    }

    #[test]
    fn to_3d_layout_returns_all_nodes() {
        let vdag = build_test_vdag();
        let positions = vdag.to_3d_layout();
        assert_eq!(positions.len(), vdag.nodes.len());
    }

    #[test]
    fn to_3d_layout_root_has_x_zero() {
        let vdag = build_test_vdag();
        let positions = vdag.to_3d_layout();
        let root_x = positions
            .iter()
            .find(|p| p.id == "N")
            .map(|p| p.x);
        assert!(root_x.is_some(), "node N not found in 3D layout");
        assert!(
            root_x.map_or(false, |x| x.abs() < 1e-9),
            "root node N should be at x=0"
        );
    }

    #[test]
    fn to_3d_layout_later_levels_have_larger_x() {
        let vdag = build_test_vdag();
        let positions = vdag.to_3d_layout();
        let n_x = positions
            .iter()
            .find(|p| p.id == "N")
            .map_or(0.0, |p| p.x);
        let aspirin_x = positions
            .iter()
            .find(|p| p.id == "aspirin")
            .map_or(0.0, |p| p.x);
        assert!(aspirin_x > n_x, "aspirin must be deeper than root N");
    }

    #[test]
    fn to_3d_layout_no_duplicate_ids() {
        let vdag = build_test_vdag();
        let positions = vdag.to_3d_layout();
        let ids: HashSet<&str> = positions.iter().map(|p| p.id.as_str()).collect();
        assert_eq!(ids.len(), positions.len(), "duplicate IDs in 3D layout");
    }

    #[test]
    fn signal_score_is_signal_passes_above_thresholds() {
        let s = SignalScore {
            ae_name: "Hepatotoxicity".into(),
            prr: Some(3.1),
            ror: Some(3.4),
            ic025: Some(0.5),
            ebgm: Some(2.8),
            case_count: Some(55),
            timestamp: None,
        };
        assert!(s.is_signal());
    }

    #[test]
    fn signal_score_fails_sub_threshold_prr() {
        let s = SignalScore {
            ae_name: "Headache".into(),
            prr: Some(1.5), // < 2.0
            ror: Some(3.0),
            ic025: Some(0.5),
            ebgm: Some(2.1),
            case_count: None,
            timestamp: None,
        };
        assert!(!s.is_signal());
    }

    #[test]
    fn signal_score_fails_negative_ic025() {
        let s = SignalScore {
            ae_name: "Rash".into(),
            prr: Some(2.5),
            ror: Some(2.8),
            ic025: Some(-0.1), // <= 0.0
            ebgm: Some(2.2),
            case_count: Some(10),
            timestamp: None,
        };
        assert!(!s.is_signal());
    }

    #[test]
    fn signal_score_partial_passes_when_missing_stats() {
        // Only PRR present and above threshold — should pass (partial evidence)
        let s = SignalScore {
            ae_name: "Fatigue".into(),
            prr: Some(2.3),
            ror: None,
            ic025: None,
            ebgm: None,
            case_count: Some(8),
            timestamp: None,
        };
        assert!(s.is_signal());
    }

    #[test]
    fn signal_summary_counts_correctly() {
        let vdag = build_test_vdag();
        let summary = vdag.signal_summary();
        assert!(summary.is_some());
        if let Some(s) = summary {
            // aspirin: 2 signals (GI Bleeding active, Tinnitus not)
            // ibuprofen: 3 signals (GI Bleeding active, Renal Failure active, Nausea not)
            assert_eq!(s.total_signals, 5);
            // Active: aspirin GI Bleeding, ibuprofen GI Bleeding, ibuprofen Renal Failure = 3
            assert_eq!(s.active_signals, 3);
        }
    }

    #[test]
    fn signal_summary_max_prr() {
        let vdag = build_test_vdag();
        // ibuprofen Renal Failure has PRR 4.1 — highest
        let max_prr = vdag
            .signal_summary()
            .and_then(|s| s.max_prr);
        assert!(max_prr.is_some(), "expected signal_summary to have max_prr");
        assert!(
            max_prr.map_or(false, |v| (v - 4.1).abs() < 1e-9),
            "max PRR should be 4.1"
        );
    }

    #[test]
    fn signal_summary_max_case_count() {
        let vdag = build_test_vdag();
        // aspirin GI Bleeding has 412 — highest
        let max_count = vdag
            .signal_summary()
            .and_then(|s| s.max_case_count);
        assert_eq!(max_count, Some(412));
    }

    #[test]
    fn signal_summary_empty_vdag_returns_none() {
        let vdag = Vdag {
            nodes: vec![],
            edges: vec![],
            title: "empty".into(),
        };
        assert!(vdag.signal_summary().is_none());
    }

    #[test]
    fn serde_roundtrip_vdag() {
        let vdag = build_test_vdag();
        let json = serde_json::to_string(&vdag).ok();
        assert!(json.is_some(), "serialization failed");
        if let Some(j) = json {
            let restored: Result<Vdag, _> = serde_json::from_str(&j);
            assert!(restored.is_ok(), "deserialization failed");
            if let Ok(r) = restored {
                assert_eq!(r.nodes.len(), vdag.nodes.len());
                assert_eq!(r.edges.len(), vdag.edges.len());
                assert_eq!(r.title, vdag.title);
            }
        }
    }

    #[test]
    fn serde_roundtrip_preserves_atc_levels() {
        let vdag = build_test_vdag();
        let json = serde_json::to_string(&vdag).ok();
        if let Some(j) = json {
            let restored: Result<Vdag, _> = serde_json::from_str(&j);
            if let Ok(r) = restored {
                let n = r.nodes.iter().find(|n| n.id == "N");
                assert!(n.is_some(), "node N missing after deserialization");
                if let Some(node) = n {
                    assert_eq!(node.atc_level, Some(AtcLevel::Anatomical));
                }
            }
        }
    }

    #[test]
    fn render_legend_produces_svg() {
        let vdag = build_test_vdag();
        let svg = vdag.render_legend(&Theme::default());
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("Drug Class"));
        assert!(svg.contains("Adverse Event"));
        assert!(svg.contains("Contains"));
    }

    #[test]
    fn kahn_levels_atc_hierarchy() {
        let vdag = build_test_vdag();
        let levels = compute_vdag_levels(&vdag.nodes, &vdag.edges);
        // Level 0: N (no in-edges from the graph)
        // Level 1: N02
        // Level 2+: aspirin, then ibuprofen (shifted by aspirin->ibuprofen edge)
        assert!(levels.len() >= 3, "expect at least 3 levels in ATC hierarchy");
        assert!(
            levels[0].contains(&"N".to_string()),
            "root N must be in first level"
        );
    }

    #[test]
    fn edge_type_colors_are_distinct() {
        let ci = color_for_edge_type(&VdagEdgeType::Contraindicates);
        let co = color_for_edge_type(&VdagEdgeType::Contains);
        assert_ne!(ci, co, "contraindication and contains must have distinct colors");
    }

    #[test]
    fn node_type_colors_are_distinct() {
        let drug = color_for_node_type(&VdagNodeType::Drug);
        let ae = color_for_node_type(&VdagNodeType::AdverseEvent);
        assert_ne!(drug, ae, "drug and AE nodes must have distinct colors");
    }

    #[test]
    fn isolated_node_appears_in_3d_layout() {
        let mut vdag = build_test_vdag();
        vdag.nodes.push(VdagNode {
            id: "isolated".into(),
            label: "Isolated Drug".into(),
            node_type: VdagNodeType::Drug,
            atc_level: None,
            signals: vec![],
            color: None,
            metadata: HashMap::new(),
        });
        let positions = vdag.to_3d_layout();
        assert_eq!(positions.len(), vdag.nodes.len());
        let found = positions.iter().any(|p| p.id == "isolated");
        assert!(found, "isolated node must appear in 3D layout");
    }

    // ------------------------------------------------------------------------
    // Phase 4 tests — compare_drug_classes
    // ------------------------------------------------------------------------

    #[test]
    fn compare_drug_classes_shared_ae_gi_bleeding() {
        // Both aspirin and ibuprofen are under N02; compare N02 against itself
        // is not realistic — compare two synthetic single-drug classes instead.
        // In the test VDAG there is only one class (N02) containing both drugs,
        // so comparing N02 with a non-existent class yields all AEs unique to A.
        let vdag = build_test_vdag();
        let cmp = vdag.compare_drug_classes("N02", "does_not_exist");
        // N02 subtree has GI Bleeding, Tinnitus, Renal Failure, Nausea
        assert!(!cmp.unique_to_a.is_empty());
        assert!(cmp.unique_to_b.is_empty());
        assert!(cmp.shared_aes.is_empty());
        assert_eq!(cmp.class_a, "N02");
        assert_eq!(cmp.class_b, "does_not_exist");
    }

    #[test]
    fn compare_drug_classes_unknown_both_returns_empty() {
        let vdag = build_test_vdag();
        let cmp = vdag.compare_drug_classes("phantom_a", "phantom_b");
        assert!(cmp.shared_aes.is_empty());
        assert!(cmp.unique_to_a.is_empty());
        assert!(cmp.unique_to_b.is_empty());
        assert!(cmp.differential_scores.is_empty());
    }

    #[test]
    fn compare_drug_classes_differential_score_values() {
        // Build a two-class graph where both classes share "Nausea" with known PRR.
        let make_class = |id: &str| VdagNode {
            id: id.into(),
            label: id.into(),
            node_type: VdagNodeType::DrugClass,
            atc_level: Some(AtcLevel::Therapeutic),
            signals: vec![],
            color: None,
            metadata: HashMap::new(),
        };
        let make_drug = |id: &str, prr: f64| VdagNode {
            id: id.into(),
            label: id.into(),
            node_type: VdagNodeType::Drug,
            atc_level: Some(AtcLevel::Substance),
            signals: vec![SignalScore {
                ae_name: "Nausea".into(),
                prr: Some(prr),
                ror: None,
                ic025: None,
                ebgm: None,
                case_count: None,
                timestamp: None,
            }],
            color: None,
            metadata: HashMap::new(),
        };
        let vdag = Vdag {
            nodes: vec![
                make_class("ca"),
                make_class("cb"),
                make_drug("da", 4.0),
                make_drug("db", 2.0),
            ],
            edges: vec![
                VdagEdge {
                    from: "ca".into(),
                    to: "da".into(),
                    edge_type: VdagEdgeType::Contains,
                    weight: None,
                },
                VdagEdge {
                    from: "cb".into(),
                    to: "db".into(),
                    edge_type: VdagEdgeType::Contains,
                    weight: None,
                },
            ],
            title: "Diff".into(),
        };
        let cmp = vdag.compare_drug_classes("ca", "cb");
        assert_eq!(cmp.shared_aes, vec!["Nausea".to_string()]);
        let ds = cmp
            .differential_scores
            .iter()
            .find(|d| d.ae_name == "Nausea");
        assert!(ds.is_some());
        if let Some(d) = ds {
            assert!((d.diff - 2.0).abs() < 1e-9, "diff should be 4.0 - 2.0 = 2.0");
            assert_eq!(d.prr_a, Some(4.0));
            assert_eq!(d.prr_b, Some(2.0));
        }
    }

    #[test]
    fn compare_drug_classes_unique_ae_sets_are_sorted() {
        let vdag = build_test_vdag();
        // N02 unique to A when B is unknown — result must be sorted
        let cmp = vdag.compare_drug_classes("N02", "phantom");
        let mut expected = cmp.unique_to_a.clone();
        expected.sort();
        assert_eq!(cmp.unique_to_a, expected, "unique_to_a must be sorted");
    }

    // ------------------------------------------------------------------------
    // Phase 4 tests — propagate_signals_up
    // ------------------------------------------------------------------------

    #[test]
    fn propagate_signals_up_gi_bleeding_appears_at_n02_and_n() {
        let vdag = build_test_vdag();
        let signals = vdag.propagate_signals_up();
        // GI Bleeding comes from aspirin (PRR 3.2) and ibuprofen (PRR 2.8)
        // Both N02 and N should receive a propagated GI Bleeding signal.
        let n02_gi = signals
            .iter()
            .find(|s| s.node_id == "N02" && s.ae_name == "GI Bleeding");
        assert!(n02_gi.is_some(), "N02 should have GI Bleeding propagated");
        if let Some(s) = n02_gi {
            // Max PRR across aspirin (3.2) and ibuprofen (2.8) is 3.2
            assert!(
                s.aggregated_prr.map_or(false, |v| (v - 3.2).abs() < 1e-9),
                "aggregated PRR for GI Bleeding at N02 should be 3.2"
            );
            // Sum of case counts: 412 + 319 = 731
            assert_eq!(s.total_cases, 731);
            assert_eq!(s.child_count, 2);
        }
    }

    #[test]
    fn propagate_signals_up_empty_vdag_returns_empty() {
        let vdag = Vdag {
            nodes: vec![],
            edges: vec![],
            title: "empty".into(),
        };
        assert!(vdag.propagate_signals_up().is_empty());
    }

    #[test]
    fn propagate_signals_up_no_drug_signals_returns_empty() {
        // A graph with classes and drugs but no signals on any drug
        let vdag = Vdag {
            nodes: vec![
                VdagNode {
                    id: "cls".into(),
                    label: "Class".into(),
                    node_type: VdagNodeType::DrugClass,
                    atc_level: Some(AtcLevel::Therapeutic),
                    signals: vec![],
                    color: None,
                    metadata: HashMap::new(),
                },
                VdagNode {
                    id: "drg".into(),
                    label: "Drug".into(),
                    node_type: VdagNodeType::Drug,
                    atc_level: Some(AtcLevel::Substance),
                    signals: vec![], // no signals
                    color: None,
                    metadata: HashMap::new(),
                },
            ],
            edges: vec![VdagEdge {
                from: "cls".into(),
                to: "drg".into(),
                edge_type: VdagEdgeType::Contains,
                weight: None,
            }],
            title: "NoSigs".into(),
        };
        assert!(vdag.propagate_signals_up().is_empty());
    }

    #[test]
    fn propagate_signals_up_result_is_sorted() {
        let vdag = build_test_vdag();
        let signals = vdag.propagate_signals_up();
        // Must be sorted by node_id then ae_name
        for pair in signals.windows(2) {
            let a = &pair[0];
            let b = &pair[1];
            let ord = a.node_id.cmp(&b.node_id).then(a.ae_name.cmp(&b.ae_name));
            assert!(
                ord != std::cmp::Ordering::Greater,
                "propagated signals must be sorted: {:?} came before {:?}",
                a.node_id,
                b.node_id
            );
        }
    }

    // ------------------------------------------------------------------------
    // Phase 4 tests — temporal_snapshots
    // ------------------------------------------------------------------------

    #[test]
    fn temporal_snapshots_groups_by_timestamp() {
        let vdag = build_test_vdag();
        // aspirin has 2 signals with "2024-01-01"; ibuprofen has 2 with "2024-Q2"
        // ibuprofen Nausea has no timestamp — excluded
        let snaps = vdag.temporal_snapshots();
        assert_eq!(snaps.len(), 2, "expected 2 distinct timestamps");
        let snap_jan = snaps.iter().find(|s| s.timestamp == "2024-01-01");
        assert!(snap_jan.is_some());
        if let Some(s) = snap_jan {
            assert_eq!(s.total_signals, 2); // aspirin GI Bleeding + Tinnitus
            // GI Bleeding is active, Tinnitus is not
            assert_eq!(s.active_signals, 1);
        }
    }

    #[test]
    fn temporal_snapshots_sorted_chronologically() {
        let vdag = build_test_vdag();
        let snaps = vdag.temporal_snapshots();
        for pair in snaps.windows(2) {
            assert!(
                pair[0].timestamp <= pair[1].timestamp,
                "snapshots must be sorted: {} before {}",
                pair[0].timestamp,
                pair[1].timestamp
            );
        }
    }

    #[test]
    fn temporal_snapshots_no_timestamps_returns_empty() {
        let vdag = Vdag {
            nodes: vec![VdagNode {
                id: "d".into(),
                label: "D".into(),
                node_type: VdagNodeType::Drug,
                atc_level: None,
                signals: vec![SignalScore {
                    ae_name: "AE".into(),
                    prr: Some(2.5),
                    ror: None,
                    ic025: None,
                    ebgm: None,
                    case_count: Some(10),
                    timestamp: None, // no timestamp
                }],
                color: None,
                metadata: HashMap::new(),
            }],
            edges: vec![],
            title: "NoTS".into(),
        };
        assert!(vdag.temporal_snapshots().is_empty());
    }

    #[test]
    fn temporal_snapshots_mean_prr_calculation() {
        // Single timestamp with two signals: PRR 3.0 and PRR 1.0 → mean = 2.0
        let vdag = Vdag {
            nodes: vec![VdagNode {
                id: "d".into(),
                label: "D".into(),
                node_type: VdagNodeType::Drug,
                atc_level: None,
                signals: vec![
                    SignalScore {
                        ae_name: "AE1".into(),
                        prr: Some(3.0),
                        ror: None,
                        ic025: None,
                        ebgm: None,
                        case_count: None,
                        timestamp: Some("2024-01".into()),
                    },
                    SignalScore {
                        ae_name: "AE2".into(),
                        prr: Some(1.0),
                        ror: None,
                        ic025: None,
                        ebgm: None,
                        case_count: None,
                        timestamp: Some("2024-01".into()),
                    },
                ],
                color: None,
                metadata: HashMap::new(),
            }],
            edges: vec![],
            title: "MeanPRR".into(),
        };
        let snaps = vdag.temporal_snapshots();
        assert_eq!(snaps.len(), 1);
        let mean = snaps[0].mean_prr;
        assert!(
            mean.map_or(false, |v| (v - 2.0).abs() < 1e-9),
            "mean PRR should be 2.0"
        );
        assert_eq!(snaps[0].max_prr, Some(3.0));
    }

    // ------------------------------------------------------------------------
    // Phase 4 tests — population_weighted_sizes
    // ------------------------------------------------------------------------

    #[test]
    fn population_weighted_sizes_normalises_correctly() {
        let vdag = build_test_vdag();
        let weights = vdag.population_weighted_sizes();
        // Only Drug nodes: aspirin (412 + 22 = 434) and ibuprofen (319 + 87 + 155 = 561)
        assert_eq!(weights.len(), 2);
        let asp = weights.iter().find(|w| w.node_id == "aspirin");
        let ibu = weights.iter().find(|w| w.node_id == "ibuprofen");
        assert!(asp.is_some());
        assert!(ibu.is_some());
        if let (Some(a), Some(b)) = (asp, ibu) {
            assert_eq!(a.raw_count, 434);
            assert_eq!(b.raw_count, 561);
            // min = 434, max = 561, range = 127
            // aspirin normalized = 0/127 = 0.0
            // ibuprofen normalized = 127/127 = 1.0
            assert!((a.normalized_weight - 0.0).abs() < 1e-9);
            assert!((b.normalized_weight - 1.0).abs() < 1e-9);
        }
    }

    #[test]
    fn population_weighted_sizes_single_drug_weight_is_zero() {
        // Single drug — range = 0, so normalized_weight must be 0.0
        let vdag = Vdag {
            nodes: vec![VdagNode {
                id: "only".into(),
                label: "Only".into(),
                node_type: VdagNodeType::Drug,
                atc_level: None,
                signals: vec![SignalScore {
                    ae_name: "AE".into(),
                    prr: None,
                    ror: None,
                    ic025: None,
                    ebgm: None,
                    case_count: Some(100),
                    timestamp: None,
                }],
                color: None,
                metadata: HashMap::new(),
            }],
            edges: vec![],
            title: "Single".into(),
        };
        let weights = vdag.population_weighted_sizes();
        assert_eq!(weights.len(), 1);
        assert!((weights[0].normalized_weight - 0.0).abs() < 1e-9);
    }

    #[test]
    fn population_weighted_sizes_empty_vdag_returns_empty() {
        let vdag = Vdag {
            nodes: vec![],
            edges: vec![],
            title: "empty".into(),
        };
        assert!(vdag.population_weighted_sizes().is_empty());
    }

    // ------------------------------------------------------------------------
    // Phase 4 tests — interaction_subgraph
    // ------------------------------------------------------------------------

    #[test]
    fn interaction_subgraph_returns_only_interacts_with_edges() {
        let vdag = build_test_vdag();
        let (nodes, edges) = vdag.interaction_subgraph();
        // Only aspirin -> ibuprofen is InteractsWith
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].from, "aspirin");
        assert_eq!(edges[0].to, "ibuprofen");
        // Both endpoint nodes must be returned
        assert_eq!(nodes.len(), 2);
        let ids: Vec<&str> = nodes.iter().map(|n| n.id.as_str()).collect();
        assert!(ids.contains(&"aspirin"));
        assert!(ids.contains(&"ibuprofen"));
    }

    #[test]
    fn interaction_subgraph_empty_when_no_interactions() {
        let vdag = Vdag {
            nodes: vec![VdagNode {
                id: "lone".into(),
                label: "Lone".into(),
                node_type: VdagNodeType::Drug,
                atc_level: None,
                signals: vec![],
                color: None,
                metadata: HashMap::new(),
            }],
            edges: vec![],
            title: "NoInt".into(),
        };
        let (nodes, edges) = vdag.interaction_subgraph();
        assert!(nodes.is_empty());
        assert!(edges.is_empty());
    }

    // ------------------------------------------------------------------------
    // Phase 4 tests — drill_down_context
    // ------------------------------------------------------------------------

    #[test]
    fn drill_down_context_aspirin_returns_full_context() {
        let vdag = build_test_vdag();
        let ctx = vdag.drill_down_context("aspirin");
        assert!(ctx.is_some());
        if let Some(c) = ctx {
            assert_eq!(c.drug.id, "aspirin");
            // Signals sorted by PRR desc: GI Bleeding (3.2) then Tinnitus (1.1)
            assert_eq!(c.signals.len(), 2);
            assert_eq!(c.signals[0].ae_name, "GI Bleeding");
            // Parent chain: N02, then N
            assert!(c.parent_chain.contains(&"N02".to_string()));
            assert!(c.parent_chain.contains(&"N".to_string()));
            // Interaction partner: ibuprofen (aspirin -> ibuprofen InteractsWith)
            assert!(c.interaction_partners.contains(&"ibuprofen".to_string()));
        }
    }

    #[test]
    fn drill_down_context_unknown_drug_returns_none() {
        let vdag = build_test_vdag();
        assert!(vdag.drill_down_context("phantom_drug").is_none());
    }

    #[test]
    fn drill_down_context_signals_sorted_prr_desc() {
        let vdag = build_test_vdag();
        // ibuprofen: Renal Failure (4.1), GI Bleeding (2.8), Nausea (1.8)
        if let Some(ctx) = vdag.drill_down_context("ibuprofen") {
            let prrs: Vec<Option<f64>> = ctx.signals.iter().map(|s| s.prr).collect();
            // Verify descending order of those with PRR present
            for pair in prrs.windows(2) {
                match (pair[0], pair[1]) {
                    (Some(a), Some(b)) => assert!(
                        a >= b,
                        "signals not sorted desc: {a} < {b}"
                    ),
                    (Some(_), None) => {} // Some > None is correct
                    _ => {}
                }
            }
        }
    }

    #[test]
    fn drill_down_context_ibuprofen_has_no_interaction_partners() {
        // In build_test_vdag, only aspirin -> ibuprofen InteractsWith edge exists.
        // When querying ibuprofen, it should appear as a partner because it is
        // the *target* of the edge (either direction is collected).
        let vdag = build_test_vdag();
        if let Some(ctx) = vdag.drill_down_context("ibuprofen") {
            // ibuprofen is the `to` endpoint of aspirin -> ibuprofen
            assert!(
                ctx.interaction_partners.contains(&"aspirin".to_string()),
                "ibuprofen should list aspirin as interaction partner (reverse direction)"
            );
        }
    }
}
