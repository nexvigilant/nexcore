//! Bipartite Graph Layout — Drug-AE Network Visualization
//!
//! Implements a two-column bipartite layout for pharmacovigilance drug-adverse
//! event network visualization. Drugs (and drug classes) are placed on the left
//! column; adverse events are placed on the right column. Weighted edges connect
//! drug nodes to the adverse events they signal.
//!
//! ## Layout Algorithm
//!
//! 1. Extract `Drug`/`DrugClass` nodes from a [`Vdag`] as the left partition.
//! 2. Extract `AdverseEvent` nodes as the right partition.
//! 3. Build edges from `HasAdverseEvent` VDAG edges, using the PRR weight.
//! 4. Apply the **barycenter heuristic** to minimize edge crossings:
//!    - Fix left, reorder right by mean position of their left neighbours.
//!    - Fix right, reorder left by mean position of their right neighbours.
//!    - Repeat up to `max_iterations` or until no improvement.
//! 5. Assign normalized `y_position` values and compute SVG geometry.
//! 6. Render edges as cubic Bézier curves whose colour and opacity encode
//!    signal strength.
//!
//! ## Signal Thresholds
//!
//! Standard NexVigilant thresholds: PRR >= 2.0, ROR-LCI > 1.0,
//! IC025 > 0.0, EB05 >= 2.0.
//!
//! ## Example
//!
//! ```rust
//! use nexcore_viz::bipartite::{default_config, from_vdag, layout_bipartite,
//!     render_bipartite_svg, Side};
//! use nexcore_viz::vdag::{Vdag, VdagEdge, VdagEdgeType, VdagNode, VdagNodeType};
//! use nexcore_viz::theme::Theme;
//! use std::collections::HashMap;
//!
//! let nodes = vec![
//!     VdagNode {
//!         id: "aspirin".into(), label: "Aspirin".into(),
//!         node_type: VdagNodeType::Drug, atc_level: None,
//!         signals: vec![], color: None, metadata: HashMap::new(),
//!     },
//!     VdagNode {
//!         id: "ae_nausea".into(), label: "Nausea".into(),
//!         node_type: VdagNodeType::AdverseEvent, atc_level: None,
//!         signals: vec![], color: None, metadata: HashMap::new(),
//!     },
//! ];
//! let edges = vec![VdagEdge {
//!     from: "aspirin".into(), to: "ae_nausea".into(),
//!     edge_type: VdagEdgeType::HasAdverseEvent, weight: Some(2.5),
//! }];
//! let vdag = Vdag { nodes, edges, title: "Example".into() };
//!
//! if let Ok((all_nodes, biedges)) = from_vdag(&vdag) {
//!     let (mut left, mut right): (Vec<_>, Vec<_>) = all_nodes
//!         .into_iter()
//!         .partition(|n| matches!(n.side, Side::Left));
//!     let config = default_config();
//!     let layout = layout_bipartite(&mut left, &mut right, &biedges, &config);
//!     let svg = render_bipartite_svg(&layout, "Drug-AE Network", &Theme::default());
//!     assert!(svg.starts_with("<svg"));
//! }
//! ```

use crate::svg::{self, SvgDoc, palette, rect_stroke, text, text_bold};
use crate::theme::Theme;
use crate::vdag::{Vdag, VdagEdgeType, VdagNodeType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Error type
// ============================================================================

/// Errors that can occur during bipartite graph construction or layout.
#[derive(Debug, Clone, PartialEq)]
pub enum BipartiteError {
    /// The VDAG contained no nodes at all.
    EmptyGraph,
    /// No `HasAdverseEvent` edges were found — nothing to lay out.
    NoEdges,
    /// The partition is inconsistent (e.g., an edge references an unknown node).
    InvalidPartition,
}

impl std::fmt::Display for BipartiteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyGraph => write!(f, "bipartite: graph contains no nodes"),
            Self::NoEdges => write!(f, "bipartite: no HasAdverseEvent edges found"),
            Self::InvalidPartition => {
                write!(f, "bipartite: edge references a node absent from the partition")
            }
        }
    }
}

impl std::error::Error for BipartiteError {}

// ============================================================================
// Core types
// ============================================================================

/// Which column of the bipartite layout a node belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    /// Left column — drugs and drug classes.
    Left,
    /// Right column — adverse events.
    Right,
}

/// A node in the bipartite graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BipartiteNode {
    /// Unique identifier, matching the corresponding `VdagNode.id`.
    pub id: String,
    /// Human-readable display label.
    pub label: String,
    /// Which column this node occupies.
    pub side: Side,
    /// Aggregate weight (e.g., total case count for drugs, connected-drug count
    /// for AEs). Used to scale node width in the rendered SVG.
    pub weight: f64,
    /// Hex fill color for this node's rectangle.
    pub color: String,
    /// Vertical position after layout, normalized to `0.0..=1.0`.
    /// Updated by [`layout_bipartite`] and [`minimize_crossings`].
    pub y_position: f64,
}

/// A weighted edge in the bipartite graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BipartiteEdge {
    /// ID of the left (drug) node.
    pub from: String,
    /// ID of the right (AE) node.
    pub to: String,
    /// Signal weight — typically the PRR value.
    pub weight: f64,
    /// Hex stroke color derived from signal strength.
    pub color: String,
    /// Stroke opacity `0.0..=1.0` proportional to weight.
    pub opacity: f64,
    /// `true` when the PRR exceeds the configured signal threshold.
    pub is_signal: bool,
}

/// Complete bipartite layout result, ready for SVG rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BipartiteLayout {
    /// Left-column nodes in final vertical order.
    pub left_nodes: Vec<BipartiteNode>,
    /// Right-column nodes in final vertical order.
    pub right_nodes: Vec<BipartiteNode>,
    /// All edges with computed colors and opacities.
    pub edges: Vec<BipartiteEdge>,
    /// Number of edge crossings remaining after minimization.
    pub crossing_count: usize,
    /// Total SVG canvas width in pixels.
    pub width: f64,
    /// Total SVG canvas height in pixels.
    pub height: f64,
}

/// Configuration for the bipartite layout algorithm and SVG renderer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BipartiteConfig {
    /// X coordinate of the left (drug) column centre line (default `100.0`).
    pub left_x: f64,
    /// X coordinate of the right (AE) column centre line (default `500.0`).
    pub right_x: f64,
    /// Minimum vertical spacing between node centres in pixels (default `40.0`).
    pub node_spacing: f64,
    /// Top and bottom margin in pixels (default `60.0`).
    pub margin: f64,
    /// Width of node rectangles in pixels before weight scaling (default `140.0`).
    pub node_width: f64,
    /// Height of node rectangles in pixels (default `30.0`).
    pub node_height: f64,
    /// Maximum barycenter-heuristic iterations for crossing minimization
    /// (default `25`).
    pub max_iterations: usize,
    /// PRR threshold above which an edge is classified as a confirmed signal
    /// (default `2.0`).
    pub signal_threshold: f64,
}

impl Default for BipartiteConfig {
    fn default() -> Self {
        default_config()
    }
}

// ============================================================================
// Public functions
// ============================================================================

/// Return the default [`BipartiteConfig`] with sensible production values.
///
/// | Field | Default |
/// |-------|---------|
/// | `left_x` | `100.0` |
/// | `right_x` | `500.0` |
/// | `node_spacing` | `40.0` |
/// | `margin` | `60.0` |
/// | `node_width` | `140.0` |
/// | `node_height` | `30.0` |
/// | `max_iterations` | `25` |
/// | `signal_threshold` | `2.0` |
///
/// # Example
///
/// ```rust
/// use nexcore_viz::bipartite::default_config;
///
/// let cfg = default_config();
/// assert_eq!(cfg.left_x, 100.0);
/// assert_eq!(cfg.signal_threshold, 2.0);
/// ```
#[must_use]
pub fn default_config() -> BipartiteConfig {
    BipartiteConfig {
        left_x: 100.0,
        right_x: 500.0,
        node_spacing: 40.0,
        margin: 60.0,
        node_width: 140.0,
        node_height: 30.0,
        max_iterations: 25,
        signal_threshold: 2.0,
    }
}

/// Extract bipartite nodes and edges from a [`Vdag`].
///
/// - `Drug` and `DrugClass` nodes → [`Side::Left`], colored teal.
/// - `AdverseEvent` nodes → [`Side::Right`], colored red.
/// - `HasAdverseEvent` edges only; weight taken from `VdagEdge.weight` if
///   present, otherwise `1.0`.
///
/// Left-node weight = sum of `case_count` from all attached signals (fallback
/// `1.0` per signal, `0.0` for no signals).
/// Right-node weight = number of distinct drug nodes connected to it.
///
/// # Errors
///
/// - [`BipartiteError::EmptyGraph`] when the VDAG has no nodes.
/// - [`BipartiteError::NoEdges`] when no `HasAdverseEvent` edges exist.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::bipartite::from_vdag;
/// use nexcore_viz::vdag::{Vdag, VdagEdge, VdagEdgeType, VdagNode, VdagNodeType};
/// use std::collections::HashMap;
///
/// let nodes = vec![
///     VdagNode { id: "drug_a".into(), label: "Drug A".into(),
///         node_type: VdagNodeType::Drug, atc_level: None,
///         signals: vec![], color: None, metadata: HashMap::new() },
///     VdagNode { id: "ae_x".into(), label: "AE X".into(),
///         node_type: VdagNodeType::AdverseEvent, atc_level: None,
///         signals: vec![], color: None, metadata: HashMap::new() },
/// ];
/// let edges = vec![VdagEdge {
///     from: "drug_a".into(), to: "ae_x".into(),
///     edge_type: VdagEdgeType::HasAdverseEvent, weight: Some(3.0),
/// }];
/// let vdag = Vdag { nodes, edges, title: "T".into() };
/// if let Ok((nodes_out, edges_out)) = from_vdag(&vdag) {
///     assert_eq!(nodes_out.len(), 2);
///     assert_eq!(edges_out.len(), 1);
/// }
/// ```
pub fn from_vdag(
    vdag: &Vdag,
) -> Result<(Vec<BipartiteNode>, Vec<BipartiteEdge>), BipartiteError> {
    if vdag.nodes.is_empty() {
        return Err(BipartiteError::EmptyGraph);
    }

    // Collect HasAdverseEvent edges first to count AE connectivity
    let hae_edges: Vec<&crate::vdag::VdagEdge> = vdag
        .edges
        .iter()
        .filter(|e| e.edge_type == VdagEdgeType::HasAdverseEvent)
        .collect();

    if hae_edges.is_empty() {
        return Err(BipartiteError::NoEdges);
    }

    // Count how many distinct drug nodes connect to each AE node
    let mut ae_drug_count: HashMap<&str, usize> = HashMap::new();
    for edge in &hae_edges {
        *ae_drug_count.entry(edge.to.as_str()).or_insert(0) += 1;
    }

    // Build left nodes (Drug / DrugClass)
    let left_nodes: Vec<BipartiteNode> = vdag
        .nodes
        .iter()
        .filter(|n| matches!(n.node_type, VdagNodeType::Drug | VdagNodeType::DrugClass))
        .map(|n| {
            let weight: f64 = if n.signals.is_empty() {
                0.0
            } else {
                n.signals
                    .iter()
                    .map(|s| s.case_count.map_or(1.0, |c| f64::from(c)))
                    .sum()
            };
            BipartiteNode {
                id: n.id.clone(),
                label: n.label.clone(),
                side: Side::Left,
                weight,
                color: palette::SCIENCE.to_string(),
                y_position: 0.0,
            }
        })
        .collect();

    // Build right nodes (AdverseEvent)
    let right_nodes: Vec<BipartiteNode> = vdag
        .nodes
        .iter()
        .filter(|n| n.node_type == VdagNodeType::AdverseEvent)
        .map(|n| {
            let weight = *ae_drug_count.get(n.id.as_str()).unwrap_or(&0) as f64;
            BipartiteNode {
                id: n.id.clone(),
                label: n.label.clone(),
                side: Side::Right,
                weight,
                color: palette::RED.to_string(),
                y_position: 0.0,
            }
        })
        .collect();

    // Build bipartite edges from HasAdverseEvent VDAG edges.
    // Determine max weight for opacity normalization.
    let max_weight = hae_edges
        .iter()
        .filter_map(|e| e.weight)
        .fold(0.0_f64, f64::max)
        .max(1.0);

    // Standard NexVigilant PRR threshold
    let threshold = 2.0_f64;
    let edges: Vec<BipartiteEdge> = hae_edges
        .iter()
        .map(|e| {
            let weight = e.weight.unwrap_or(1.0);
            let color = weight_to_color(weight, threshold);
            let opacity = weight_to_opacity(weight, max_weight);
            let is_signal = weight >= threshold;
            BipartiteEdge {
                from: e.from.clone(),
                to: e.to.clone(),
                weight,
                color,
                opacity,
                is_signal,
            }
        })
        .collect();

    let mut all_nodes = left_nodes;
    all_nodes.extend(right_nodes);
    Ok((all_nodes, edges))
}

/// Count the number of edge crossings for the current node ordering.
///
/// Two edges `e1` and `e2` cross when:
/// - `left_pos(e1.from) < left_pos(e2.from)` AND `right_pos(e1.to) > right_pos(e2.to)`
/// - OR the symmetric mirror case.
///
/// Positions are taken from each node's index within its respective slice.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::bipartite::{BipartiteEdge, BipartiteNode, Side, count_crossings};
///
/// // Two left nodes (top→bottom: A, B) and two right nodes (top→bottom: X, Y).
/// // A–Y, B–X should cross once.
/// let left = vec![
///     BipartiteNode { id: "A".into(), label: "A".into(), side: Side::Left,
///         weight: 1.0, color: "#fff".into(), y_position: 0.0 },
///     BipartiteNode { id: "B".into(), label: "B".into(), side: Side::Left,
///         weight: 1.0, color: "#fff".into(), y_position: 1.0 },
/// ];
/// let right = vec![
///     BipartiteNode { id: "X".into(), label: "X".into(), side: Side::Right,
///         weight: 1.0, color: "#fff".into(), y_position: 0.0 },
///     BipartiteNode { id: "Y".into(), label: "Y".into(), side: Side::Right,
///         weight: 1.0, color: "#fff".into(), y_position: 1.0 },
/// ];
/// let edges = vec![
///     BipartiteEdge { from: "A".into(), to: "Y".into(), weight: 1.0,
///         color: "#f00".into(), opacity: 1.0, is_signal: false },
///     BipartiteEdge { from: "B".into(), to: "X".into(), weight: 1.0,
///         color: "#f00".into(), opacity: 1.0, is_signal: false },
/// ];
/// assert_eq!(count_crossings(&left, &right, &edges), 1);
/// ```
#[must_use]
pub fn count_crossings(
    left: &[BipartiteNode],
    right: &[BipartiteNode],
    edges: &[BipartiteEdge],
) -> usize {
    // Build position maps: node_id → index in slice
    let left_pos: HashMap<&str, usize> = left
        .iter()
        .enumerate()
        .map(|(i, n)| (n.id.as_str(), i))
        .collect();
    let right_pos: HashMap<&str, usize> = right
        .iter()
        .enumerate()
        .map(|(i, n)| (n.id.as_str(), i))
        .collect();

    let mut crossings = 0usize;
    let n = edges.len();
    for i in 0..n {
        for j in (i + 1)..n {
            let e1 = &edges[i];
            let e2 = &edges[j];

            let Some(&lp1) = left_pos.get(e1.from.as_str()) else {
                continue;
            };
            let Some(&rp1) = right_pos.get(e1.to.as_str()) else {
                continue;
            };
            let Some(&lp2) = left_pos.get(e2.from.as_str()) else {
                continue;
            };
            let Some(&rp2) = right_pos.get(e2.to.as_str()) else {
                continue;
            };

            // Edges cross when left order and right order disagree
            if (lp1 < lp2 && rp1 > rp2) || (lp1 > lp2 && rp1 < rp2) {
                crossings += 1;
            }
        }
    }
    crossings
}

/// Reduce edge crossings using the barycenter heuristic.
///
/// Alternately fixes one side and reorders the other by the mean positional
/// index of its connected nodes on the fixed side. Repeats until no
/// improvement is observed or `max_iterations` is reached.
///
/// Returns the final crossing count after minimization.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::bipartite::{BipartiteEdge, BipartiteNode, Side, minimize_crossings};
///
/// let mut left = vec![
///     BipartiteNode { id: "A".into(), label: "A".into(), side: Side::Left,
///         weight: 1.0, color: "#fff".into(), y_position: 0.0 },
///     BipartiteNode { id: "B".into(), label: "B".into(), side: Side::Left,
///         weight: 1.0, color: "#fff".into(), y_position: 0.5 },
/// ];
/// let mut right = vec![
///     BipartiteNode { id: "X".into(), label: "X".into(), side: Side::Right,
///         weight: 1.0, color: "#fff".into(), y_position: 0.0 },
///     BipartiteNode { id: "Y".into(), label: "Y".into(), side: Side::Right,
///         weight: 1.0, color: "#fff".into(), y_position: 0.5 },
/// ];
/// // A-Y, B-X cross once initially
/// let edges = vec![
///     BipartiteEdge { from: "A".into(), to: "Y".into(), weight: 2.0,
///         color: "#f00".into(), opacity: 0.8, is_signal: true },
///     BipartiteEdge { from: "B".into(), to: "X".into(), weight: 2.0,
///         color: "#f00".into(), opacity: 0.8, is_signal: true },
/// ];
/// let final_crossings = minimize_crossings(&mut left, &mut right, &edges, 10);
/// assert_eq!(final_crossings, 0);
/// ```
pub fn minimize_crossings(
    left: &mut [BipartiteNode],
    right: &mut [BipartiteNode],
    edges: &[BipartiteEdge],
    max_iterations: usize,
) -> usize {
    let mut best = count_crossings(left, right, edges);

    for _ in 0..max_iterations {
        if best == 0 {
            break;
        }

        // Phase A: fix left, reorder right by barycenter
        barycenter_sort_right(left, right, edges);

        // Phase B: fix right, reorder left by barycenter
        barycenter_sort_left(left, right, edges);

        let after = count_crossings(left, right, edges);
        if after >= best {
            // No improvement this round; stop early
            best = after;
            break;
        }
        best = after;
    }

    best
}

/// Compute the final [`BipartiteLayout`] from left/right node slices and edges.
///
/// Steps performed:
/// 1. Run [`minimize_crossings`] using `config.max_iterations`.
/// 2. Assign `y_position` values (`0.0..=1.0`) based on final ordering.
/// 3. Recompute edge colors and opacities using `config.signal_threshold`.
/// 4. Compute canvas `width` and `height`.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::bipartite::{BipartiteEdge, BipartiteNode, Side, default_config, layout_bipartite};
///
/// let mut left = vec![BipartiteNode {
///     id: "D".into(), label: "Drug".into(), side: Side::Left,
///     weight: 100.0, color: "#2dd4bf".into(), y_position: 0.0,
/// }];
/// let mut right = vec![BipartiteNode {
///     id: "E".into(), label: "AE".into(), side: Side::Right,
///     weight: 1.0, color: "#ef4444".into(), y_position: 0.0,
/// }];
/// let edges = vec![BipartiteEdge {
///     from: "D".into(), to: "E".into(), weight: 3.5,
///     color: "#ef4444".into(), opacity: 0.9, is_signal: true,
/// }];
/// let cfg = default_config();
/// let layout = layout_bipartite(&mut left, &mut right, &edges, &cfg);
/// assert_eq!(layout.left_nodes.len(), 1);
/// assert_eq!(layout.right_nodes.len(), 1);
/// assert_eq!(layout.edges.len(), 1);
/// assert!(layout.width > 0.0);
/// assert!(layout.height > 0.0);
/// ```
#[must_use]
pub fn layout_bipartite(
    left: &mut Vec<BipartiteNode>,
    right: &mut Vec<BipartiteNode>,
    edges: &[BipartiteEdge],
    config: &BipartiteConfig,
) -> BipartiteLayout {
    let crossing_count = minimize_crossings(left, right, edges, config.max_iterations);

    // Assign normalized y_position values based on final ordering
    assign_positions(left);
    assign_positions(right);

    // Determine canvas dimensions
    let left_count = left.len().max(1);
    let right_count = right.len().max(1);
    let max_count = left_count.max(right_count);
    let height = config.margin * 2.0 + (max_count as f64 - 1.0).max(0.0) * config.node_spacing;
    let width = config.right_x + config.node_width + config.margin;

    // Recompute edge colors / opacities with the configured threshold
    let max_weight = edges
        .iter()
        .map(|e| e.weight)
        .fold(0.0_f64, f64::max)
        .max(1.0);

    let updated_edges: Vec<BipartiteEdge> = edges
        .iter()
        .map(|e| BipartiteEdge {
            from: e.from.clone(),
            to: e.to.clone(),
            weight: e.weight,
            color: weight_to_color(e.weight, config.signal_threshold),
            opacity: weight_to_opacity(e.weight, max_weight),
            is_signal: e.weight >= config.signal_threshold,
        })
        .collect();

    BipartiteLayout {
        left_nodes: left.clone(),
        right_nodes: right.clone(),
        edges: updated_edges,
        crossing_count,
        width,
        height,
    }
}

/// Render a [`BipartiteLayout`] as a self-contained SVG string.
///
/// The SVG contains:
/// - A title at the top.
/// - Column headers ("Drugs" left, "Adverse Events" right).
/// - Nodes as rounded rectangles with centered labels.
/// - Edges as cubic Bézier curves whose stroke color and opacity encode
///   the signal strength.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::bipartite::{BipartiteEdge, BipartiteNode, BipartiteLayout, Side,
///     render_bipartite_svg};
/// use nexcore_viz::theme::Theme;
///
/// let layout = BipartiteLayout {
///     left_nodes: vec![BipartiteNode {
///         id: "d".into(), label: "Aspirin".into(), side: Side::Left,
///         weight: 200.0, color: "#2dd4bf".into(), y_position: 0.5,
///     }],
///     right_nodes: vec![BipartiteNode {
///         id: "ae".into(), label: "Nausea".into(), side: Side::Right,
///         weight: 1.0, color: "#ef4444".into(), y_position: 0.5,
///     }],
///     edges: vec![BipartiteEdge {
///         from: "d".into(), to: "ae".into(), weight: 2.5,
///         color: "#ef4444".into(), opacity: 0.8, is_signal: true,
///     }],
///     crossing_count: 0,
///     width: 680.0,
///     height: 200.0,
/// };
/// let svg = render_bipartite_svg(&layout, "Test", &Theme::default());
/// assert!(svg.starts_with("<svg"));
/// assert!(svg.contains("Aspirin"));
/// assert!(svg.contains("Nausea"));
/// ```
#[must_use]
pub fn render_bipartite_svg(layout: &BipartiteLayout, title: &str, theme: &Theme) -> String {
    let cfg = default_config();
    let mut doc = SvgDoc::new_with_theme(layout.width, layout.height, theme.clone());

    // Background
    doc.add(svg::rect(
        0.0,
        0.0,
        layout.width,
        layout.height,
        theme.background,
        0.0,
    ));

    // Title
    doc.add(text_bold(
        layout.width / 2.0,
        cfg.margin / 3.0,
        title,
        14.0,
        theme.text,
        "middle",
    ));

    // Column headers
    doc.add(text_bold(
        cfg.left_x + cfg.node_width / 2.0,
        cfg.margin * 0.65,
        "Drugs",
        11.0,
        palette::SCIENCE,
        "middle",
    ));
    doc.add(text_bold(
        cfg.right_x + cfg.node_width / 2.0,
        cfg.margin * 0.65,
        "Adverse Events",
        11.0,
        palette::RED,
        "middle",
    ));

    // Draw edges first (behind nodes)
    for edge in &layout.edges {
        let Some(left_node) = layout.left_nodes.iter().find(|n| n.id == edge.from) else {
            continue;
        };
        let Some(right_node) = layout.right_nodes.iter().find(|n| n.id == edge.to) else {
            continue;
        };

        let ly = node_y(left_node.y_position, layout.height, &cfg);
        let ry = node_y(right_node.y_position, layout.height, &cfg);

        // Right edge of left node rectangle, vertical center
        let x1 = cfg.left_x + cfg.node_width;
        let y1 = ly;
        // Left edge of right node rectangle, vertical center
        let x2 = cfg.right_x;
        let y2 = ry;

        // Cubic Bézier: control points at 1/3 and 2/3 along the x-axis
        let ctrl_x1 = x1 + (x2 - x1) / 3.0;
        let ctrl_x2 = x1 + 2.0 * (x2 - x1) / 3.0;
        let bezier_d = format!(
            "M {x1:.1} {y1:.1} C {ctrl_x1:.1} {y1:.1} {ctrl_x2:.1} {y2:.1} {x2:.1} {y2:.1}"
        );

        let stroke_w = if edge.is_signal { 2.0 } else { 1.0 };
        let path_el = format!(
            r#"<path d="{bezier_d}" fill="none" stroke="{}" stroke-width="{stroke_w:.1}" opacity="{:.2}"/>"#,
            edge.color, edge.opacity
        );
        doc.add(path_el);
    }

    // Draw left nodes
    for node in &layout.left_nodes {
        let ny = node_y(node.y_position, layout.height, &cfg);
        let rect_y = ny - cfg.node_height / 2.0;

        doc.add(rect_stroke(
            cfg.left_x,
            rect_y,
            cfg.node_width,
            cfg.node_height,
            theme.card_bg,
            &node.color,
            2.0,
            cfg.node_height / 4.0,
        ));
        doc.add(text(
            cfg.left_x + cfg.node_width / 2.0,
            ny,
            &truncate_label(&node.label, 18),
            9.5,
            theme.text,
            "middle",
        ));
    }

    // Draw right nodes
    for node in &layout.right_nodes {
        let ny = node_y(node.y_position, layout.height, &cfg);
        let rect_y = ny - cfg.node_height / 2.0;

        doc.add(rect_stroke(
            cfg.right_x,
            rect_y,
            cfg.node_width,
            cfg.node_height,
            theme.card_bg,
            &node.color,
            2.0,
            cfg.node_height / 4.0,
        ));
        doc.add(text(
            cfg.right_x + cfg.node_width / 2.0,
            ny,
            &truncate_label(&node.label, 18),
            9.5,
            theme.text,
            "middle",
        ));
    }

    doc.render()
}

// ============================================================================
// Private helpers
// ============================================================================

/// Map a signal weight to a hex color string.
///
/// - Below threshold → slate (neutral / sub-signal).
/// - At or above threshold but below 4.0 → amber (moderate signal).
/// - 4.0 and above → red (strong signal).
fn weight_to_color(weight: f64, threshold: f64) -> String {
    if weight < threshold {
        palette::SLATE.to_string()
    } else if weight < 4.0 {
        palette::AMBER.to_string()
    } else {
        palette::RED.to_string()
    }
}

/// Normalize a weight to an opacity in `0.3..=1.0`.
///
/// Returns `0.3` when `max_weight <= 0.0` to avoid division by zero.
fn weight_to_opacity(weight: f64, max_weight: f64) -> f64 {
    if max_weight <= 0.0 {
        return 0.3;
    }
    let ratio = (weight / max_weight).clamp(0.0, 1.0);
    // Map [0, 1] → [0.3, 1.0]
    0.3 + ratio * 0.7
}

/// Assign `y_position` values (0.0..=1.0) to a node slice based on its current order.
fn assign_positions(nodes: &mut [BipartiteNode]) {
    let n = nodes.len();
    if n == 0 {
        return;
    }
    if n == 1 {
        nodes[0].y_position = 0.5;
        return;
    }
    for (i, node) in nodes.iter_mut().enumerate() {
        node.y_position = i as f64 / (n - 1) as f64;
    }
}

/// Convert a normalized `y_position` (0.0..=1.0) to a pixel Y coordinate
/// within the canvas.
fn node_y(y_position: f64, canvas_height: f64, cfg: &BipartiteConfig) -> f64 {
    let usable = canvas_height - 2.0 * cfg.margin;
    cfg.margin + y_position * usable
}

/// Reorder the right-side nodes by the barycenter of their left-neighbour positions.
///
/// Left positions are fixed (given by index in the left slice).
fn barycenter_sort_right(
    left: &[BipartiteNode],
    right: &mut [BipartiteNode],
    edges: &[BipartiteEdge],
) {
    let left_pos: HashMap<&str, f64> = left
        .iter()
        .enumerate()
        .map(|(i, n)| (n.id.as_str(), i as f64))
        .collect();

    let mut bary: Vec<(f64, usize)> = right
        .iter()
        .enumerate()
        .map(|(i, rn)| {
            let positions: Vec<f64> = edges
                .iter()
                .filter(|e| e.to == rn.id)
                .filter_map(|e| left_pos.get(e.from.as_str()).copied())
                .collect();
            let center = if positions.is_empty() {
                i as f64
            } else {
                positions.iter().sum::<f64>() / positions.len() as f64
            };
            (center, i)
        })
        .collect();

    bary.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let original: Vec<BipartiteNode> = right.to_vec();
    for (new_pos, &(_, old_pos)) in bary.iter().enumerate() {
        right[new_pos] = original[old_pos].clone();
    }
}

/// Reorder the left-side nodes by the barycenter of their right-neighbour positions.
///
/// Right positions are fixed (given by index in the right slice).
fn barycenter_sort_left(
    left: &mut [BipartiteNode],
    right: &[BipartiteNode],
    edges: &[BipartiteEdge],
) {
    let right_pos: HashMap<&str, f64> = right
        .iter()
        .enumerate()
        .map(|(i, n)| (n.id.as_str(), i as f64))
        .collect();

    let mut bary: Vec<(f64, usize)> = left
        .iter()
        .enumerate()
        .map(|(i, ln)| {
            let positions: Vec<f64> = edges
                .iter()
                .filter(|e| e.from == ln.id)
                .filter_map(|e| right_pos.get(e.to.as_str()).copied())
                .collect();
            let center = if positions.is_empty() {
                i as f64
            } else {
                positions.iter().sum::<f64>() / positions.len() as f64
            };
            (center, i)
        })
        .collect();

    bary.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let original: Vec<BipartiteNode> = left.to_vec();
    for (new_pos, &(_, old_pos)) in bary.iter().enumerate() {
        left[new_pos] = original[old_pos].clone();
    }
}

/// Truncate a label to at most `max_chars` characters, appending '…' if cut.
fn truncate_label(label: &str, max_chars: usize) -> String {
    let chars: Vec<char> = label.chars().collect();
    if chars.len() <= max_chars {
        label.to_string()
    } else {
        let truncated: String = chars[..max_chars.saturating_sub(1)].iter().collect();
        format!("{truncated}\u{2026}")
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vdag::{SignalScore, Vdag, VdagEdge, VdagNode, VdagNodeType};
    use std::collections::HashMap;

    // -------------------------------------------------------------------------
    // Test helpers
    // -------------------------------------------------------------------------

    fn make_drug(id: &str, label: &str, case_count: u32, prr: f64) -> VdagNode {
        VdagNode {
            id: id.into(),
            label: label.into(),
            node_type: VdagNodeType::Drug,
            atc_level: None,
            signals: vec![SignalScore {
                ae_name: "test_ae".into(),
                prr: Some(prr),
                ror: None,
                ic025: None,
                ebgm: None,
                case_count: Some(case_count),
                timestamp: None,
            }],
            color: None,
            metadata: HashMap::new(),
        }
    }

    fn make_ae(id: &str, label: &str) -> VdagNode {
        VdagNode {
            id: id.into(),
            label: label.into(),
            node_type: VdagNodeType::AdverseEvent,
            atc_level: None,
            signals: vec![],
            color: None,
            metadata: HashMap::new(),
        }
    }

    fn make_hae_edge(from: &str, to: &str, weight: f64) -> VdagEdge {
        VdagEdge {
            from: from.into(),
            to: to.into(),
            edge_type: VdagEdgeType::HasAdverseEvent,
            weight: Some(weight),
        }
    }

    /// Build a small VDAG: 2 drugs x 3 AEs, 4 `HasAdverseEvent` edges.
    ///
    /// ```text
    /// drug_a ──(PRR 3.2)──► ae_nausea
    /// drug_a ──(PRR 1.1)──► ae_fatigue
    /// drug_b ──(PRR 2.8)──► ae_nausea
    /// drug_b ──(PRR 4.1)──► ae_hepato
    /// ```
    fn build_test_vdag() -> Vdag {
        let nodes = vec![
            make_drug("drug_a", "Drug Alpha", 412, 3.2),
            make_drug("drug_b", "Drug Beta", 319, 2.8),
            make_ae("ae_nausea", "Nausea"),
            make_ae("ae_fatigue", "Fatigue"),
            make_ae("ae_hepato", "Hepatotoxicity"),
        ];
        let edges = vec![
            make_hae_edge("drug_a", "ae_nausea", 3.2),
            make_hae_edge("drug_a", "ae_fatigue", 1.1),
            make_hae_edge("drug_b", "ae_nausea", 2.8),
            make_hae_edge("drug_b", "ae_hepato", 4.1),
        ];
        Vdag {
            nodes,
            edges,
            title: "Test Drug-AE Network".into(),
        }
    }

    /// Partition `all_nodes` into (left, right) and filter `edges` to only
    /// those whose endpoints appear in the respective sides.
    fn partition_and_filter(
        all_nodes: Vec<BipartiteNode>,
        edges: Vec<BipartiteEdge>,
    ) -> (Vec<BipartiteNode>, Vec<BipartiteNode>, Vec<BipartiteEdge>) {
        let (left, right): (Vec<_>, Vec<_>) =
            all_nodes.into_iter().partition(|n| n.side == Side::Left);
        let left_ids: std::collections::HashSet<&str> =
            left.iter().map(|n| n.id.as_str()).collect();
        let right_ids: std::collections::HashSet<&str> =
            right.iter().map(|n| n.id.as_str()).collect();
        let filtered: Vec<BipartiteEdge> = edges
            .into_iter()
            .filter(|e| left_ids.contains(e.from.as_str()) && right_ids.contains(e.to.as_str()))
            .collect();
        (left, right, filtered)
    }

    // =========================================================================
    // from_vdag tests
    // =========================================================================

    #[test]
    fn from_vdag_basic_produces_correct_partition() -> Result<(), BipartiteError> {
        let vdag = build_test_vdag();
        let (nodes, edges) = from_vdag(&vdag)?;
        let left: Vec<_> = nodes.iter().filter(|n| n.side == Side::Left).collect();
        let right: Vec<_> = nodes.iter().filter(|n| n.side == Side::Right).collect();
        assert_eq!(left.len(), 2, "expected 2 drug nodes on the left");
        assert_eq!(right.len(), 3, "expected 3 AE nodes on the right");
        assert_eq!(edges.len(), 4, "expected 4 HasAdverseEvent edges");
        Ok(())
    }

    #[test]
    fn from_vdag_drug_color_is_teal() -> Result<(), BipartiteError> {
        let vdag = build_test_vdag();
        let (nodes, _) = from_vdag(&vdag)?;
        for n in nodes.iter().filter(|n| n.side == Side::Left) {
            assert_eq!(n.color, palette::SCIENCE, "drug nodes should be teal");
        }
        Ok(())
    }

    #[test]
    fn from_vdag_ae_color_is_red() -> Result<(), BipartiteError> {
        let vdag = build_test_vdag();
        let (nodes, _) = from_vdag(&vdag)?;
        for n in nodes.iter().filter(|n| n.side == Side::Right) {
            assert_eq!(n.color, palette::RED, "AE nodes should be red");
        }
        Ok(())
    }

    #[test]
    fn from_vdag_drug_weight_is_case_count_sum() -> Result<(), BipartiteError> {
        let vdag = build_test_vdag();
        let (nodes, _) = from_vdag(&vdag)?;
        let drug_a = nodes.iter().find(|n| n.id == "drug_a");
        assert!(drug_a.is_some(), "drug_a should be present");
        if let Some(n) = drug_a {
            assert!(
                (n.weight - 412.0).abs() < 1e-9,
                "drug_a weight should be 412"
            );
        }
        Ok(())
    }

    #[test]
    fn from_vdag_ae_weight_is_connected_drug_count() -> Result<(), BipartiteError> {
        let vdag = build_test_vdag();
        let (nodes, _) = from_vdag(&vdag)?;
        let nausea = nodes.iter().find(|n| n.id == "ae_nausea");
        assert!(nausea.is_some(), "ae_nausea should be present");
        if let Some(n) = nausea {
            assert!(
                (n.weight - 2.0).abs() < 1e-9,
                "ae_nausea should have weight 2 (2 connected drugs)"
            );
        }
        Ok(())
    }

    #[test]
    fn from_vdag_empty_graph_returns_error() {
        let vdag = Vdag {
            nodes: vec![],
            edges: vec![],
            title: "empty".into(),
        };
        assert!(matches!(from_vdag(&vdag), Err(BipartiteError::EmptyGraph)));
    }

    #[test]
    fn from_vdag_no_hae_edges_returns_error() {
        let nodes = vec![make_drug("d", "Drug", 10, 2.0), make_ae("ae", "AE")];
        let edges = vec![VdagEdge {
            from: "d".into(),
            to: "ae".into(),
            edge_type: VdagEdgeType::Contains,
            weight: None,
        }];
        let vdag = Vdag {
            nodes,
            edges,
            title: "no HAE".into(),
        };
        assert!(matches!(from_vdag(&vdag), Err(BipartiteError::NoEdges)));
    }

    #[test]
    fn from_vdag_single_drug_single_ae() -> Result<(), BipartiteError> {
        let nodes = vec![make_drug("d1", "Drug 1", 50, 3.0), make_ae("ae1", "AE 1")];
        let edges = vec![make_hae_edge("d1", "ae1", 3.0)];
        let vdag = Vdag {
            nodes,
            edges,
            title: "single pair".into(),
        };
        let (nodes_out, edges_out) = from_vdag(&vdag)?;
        assert_eq!(nodes_out.len(), 2);
        assert_eq!(edges_out.len(), 1);
        Ok(())
    }

    #[test]
    fn from_vdag_edge_is_signal_above_threshold() -> Result<(), BipartiteError> {
        let vdag = build_test_vdag();
        let (_, edges) = from_vdag(&vdag)?;
        let signal_edges: Vec<_> = edges.iter().filter(|e| e.is_signal).collect();
        assert!(
            signal_edges.len() >= 3,
            "at least 3 edges should exceed PRR threshold 2.0, got {}",
            signal_edges.len()
        );
        Ok(())
    }

    #[test]
    fn from_vdag_sub_threshold_edge_not_signal() -> Result<(), BipartiteError> {
        let vdag = build_test_vdag();
        let (_, edges) = from_vdag(&vdag)?;
        let sub = edges
            .iter()
            .find(|e| e.from == "drug_a" && e.to == "ae_fatigue");
        assert!(sub.is_some(), "drug_a -> ae_fatigue edge should exist");
        if let Some(e) = sub {
            assert!(!e.is_signal, "PRR 1.1 should not be a confirmed signal");
        }
        Ok(())
    }

    // =========================================================================
    // count_crossings tests
    // =========================================================================

    fn make_bi_node(id: &str, side: Side, y: f64) -> BipartiteNode {
        BipartiteNode {
            id: id.into(),
            label: id.into(),
            side,
            weight: 1.0,
            color: "#fff".into(),
            y_position: y,
        }
    }

    fn make_bi_edge(from: &str, to: &str, weight: f64) -> BipartiteEdge {
        BipartiteEdge {
            from: from.into(),
            to: to.into(),
            weight,
            color: "#f00".into(),
            opacity: 1.0,
            is_signal: weight >= 2.0,
        }
    }

    #[test]
    fn count_crossings_zero_for_parallel_edges() {
        let left = vec![
            make_bi_node("A", Side::Left, 0.0),
            make_bi_node("B", Side::Left, 1.0),
        ];
        let right = vec![
            make_bi_node("X", Side::Right, 0.0),
            make_bi_node("Y", Side::Right, 1.0),
        ];
        // A→X and B→Y: same left-right order → 0 crossings
        let edges = vec![make_bi_edge("A", "X", 2.0), make_bi_edge("B", "Y", 2.0)];
        assert_eq!(count_crossings(&left, &right, &edges), 0);
    }

    #[test]
    fn count_crossings_one_for_single_crossing() {
        let left = vec![
            make_bi_node("A", Side::Left, 0.0),
            make_bi_node("B", Side::Left, 1.0),
        ];
        let right = vec![
            make_bi_node("X", Side::Right, 0.0),
            make_bi_node("Y", Side::Right, 1.0),
        ];
        // A→Y and B→X: orders disagree → 1 crossing
        let edges = vec![make_bi_edge("A", "Y", 2.0), make_bi_edge("B", "X", 2.0)];
        assert_eq!(count_crossings(&left, &right, &edges), 1);
    }

    #[test]
    fn count_crossings_empty_edges_zero() {
        let left = vec![make_bi_node("A", Side::Left, 0.0)];
        let right = vec![make_bi_node("X", Side::Right, 0.0)];
        assert_eq!(count_crossings(&left, &right, &[]), 0);
    }

    #[test]
    fn count_crossings_worst_case_four_nodes() {
        // L1→R4, L2→R3, L3→R2, L4→R1 — fully reversed: C(4,2) = 6 crossings
        let left = vec![
            make_bi_node("L1", Side::Left, 0.0),
            make_bi_node("L2", Side::Left, 1.0),
            make_bi_node("L3", Side::Left, 2.0),
            make_bi_node("L4", Side::Left, 3.0),
        ];
        let right = vec![
            make_bi_node("R1", Side::Right, 0.0),
            make_bi_node("R2", Side::Right, 1.0),
            make_bi_node("R3", Side::Right, 2.0),
            make_bi_node("R4", Side::Right, 3.0),
        ];
        let edges = vec![
            make_bi_edge("L1", "R4", 2.0),
            make_bi_edge("L2", "R3", 2.0),
            make_bi_edge("L3", "R2", 2.0),
            make_bi_edge("L4", "R1", 2.0),
        ];
        assert_eq!(count_crossings(&left, &right, &edges), 6);
    }

    // =========================================================================
    // minimize_crossings tests
    // =========================================================================

    #[test]
    fn minimize_crossings_resolves_single_crossing() {
        let mut left = vec![
            make_bi_node("A", Side::Left, 0.0),
            make_bi_node("B", Side::Left, 0.5),
        ];
        let mut right = vec![
            make_bi_node("X", Side::Right, 0.0),
            make_bi_node("Y", Side::Right, 0.5),
        ];
        // A→Y, B→X: 1 crossing
        let edges = vec![make_bi_edge("A", "Y", 2.0), make_bi_edge("B", "X", 2.0)];
        let result = minimize_crossings(&mut left, &mut right, &edges, 10);
        assert_eq!(result, 0, "single crossing should be eliminated");
    }

    #[test]
    fn minimize_crossings_fully_reversed_four_nodes_improves() {
        let mut left = vec![
            make_bi_node("L1", Side::Left, 0.0),
            make_bi_node("L2", Side::Left, 1.0),
            make_bi_node("L3", Side::Left, 2.0),
            make_bi_node("L4", Side::Left, 3.0),
        ];
        let mut right = vec![
            make_bi_node("R1", Side::Right, 0.0),
            make_bi_node("R2", Side::Right, 1.0),
            make_bi_node("R3", Side::Right, 2.0),
            make_bi_node("R4", Side::Right, 3.0),
        ];
        let edges = vec![
            make_bi_edge("L1", "R4", 3.0),
            make_bi_edge("L2", "R3", 3.0),
            make_bi_edge("L3", "R2", 3.0),
            make_bi_edge("L4", "R1", 3.0),
        ];
        let initial = count_crossings(&left, &right, &edges);
        assert_eq!(initial, 6, "should start with 6 crossings");
        let final_cross = minimize_crossings(&mut left, &mut right, &edges, 25);
        assert!(
            final_cross < initial,
            "minimization should improve on {initial} crossings, got {final_cross}"
        );
    }

    #[test]
    fn minimize_crossings_zero_iterations_returns_initial_count() {
        let mut left = vec![
            make_bi_node("A", Side::Left, 0.0),
            make_bi_node("B", Side::Left, 1.0),
        ];
        let mut right = vec![
            make_bi_node("X", Side::Right, 0.0),
            make_bi_node("Y", Side::Right, 1.0),
        ];
        // A→Y, B→X: 1 crossing, no iterations allowed
        let edges = vec![make_bi_edge("A", "Y", 2.0), make_bi_edge("B", "X", 2.0)];
        let result = minimize_crossings(&mut left, &mut right, &edges, 0);
        assert_eq!(result, 1, "zero iterations should leave crossings unchanged");
    }

    // =========================================================================
    // layout_bipartite tests
    // =========================================================================

    #[test]
    fn layout_bipartite_produces_valid_positions() -> Result<(), BipartiteError> {
        let vdag = build_test_vdag();
        let (nodes, edges) = from_vdag(&vdag)?;
        let (mut left, mut right, filtered_edges) = partition_and_filter(nodes, edges);
        let cfg = default_config();
        let layout = layout_bipartite(&mut left, &mut right, &filtered_edges, &cfg);

        for n in &layout.left_nodes {
            assert!(
                n.y_position >= 0.0 && n.y_position <= 1.0,
                "left node y_position out of range: {}",
                n.y_position
            );
        }
        for n in &layout.right_nodes {
            assert!(
                n.y_position >= 0.0 && n.y_position <= 1.0,
                "right node y_position out of range: {}",
                n.y_position
            );
        }
        Ok(())
    }

    #[test]
    fn layout_bipartite_dimensions_positive() -> Result<(), BipartiteError> {
        let vdag = build_test_vdag();
        let (nodes, edges) = from_vdag(&vdag)?;
        let (mut left, mut right, filtered_edges) = partition_and_filter(nodes, edges);
        let cfg = default_config();
        let layout = layout_bipartite(&mut left, &mut right, &filtered_edges, &cfg);
        assert!(layout.width > 0.0, "layout width must be positive");
        assert!(layout.height > 0.0, "layout height must be positive");
        Ok(())
    }

    #[test]
    fn layout_bipartite_single_node_each_side() {
        let mut left = vec![make_bi_node("D", Side::Left, 0.0)];
        let mut right = vec![make_bi_node("E", Side::Right, 0.0)];
        let edges = vec![make_bi_edge("D", "E", 3.0)];
        let cfg = default_config();
        let layout = layout_bipartite(&mut left, &mut right, &edges, &cfg);
        assert_eq!(layout.left_nodes.len(), 1);
        assert_eq!(layout.right_nodes.len(), 1);
        assert!(
            (layout.left_nodes[0].y_position - 0.5).abs() < 1e-9,
            "single left node should be centred at 0.5"
        );
        assert!(
            (layout.right_nodes[0].y_position - 0.5).abs() < 1e-9,
            "single right node should be centred at 0.5"
        );
    }

    #[test]
    fn layout_bipartite_edge_opacity_within_range() -> Result<(), BipartiteError> {
        let vdag = build_test_vdag();
        let (nodes, edges) = from_vdag(&vdag)?;
        let (mut left, mut right, filtered_edges) = partition_and_filter(nodes, edges);
        let cfg = default_config();
        let layout = layout_bipartite(&mut left, &mut right, &filtered_edges, &cfg);
        for e in &layout.edges {
            assert!(
                e.opacity >= 0.3 && e.opacity <= 1.0,
                "edge opacity out of [0.3, 1.0]: {}",
                e.opacity
            );
        }
        Ok(())
    }

    // =========================================================================
    // render_bipartite_svg tests
    // =========================================================================

    #[test]
    fn render_bipartite_svg_produces_valid_svg() -> Result<(), BipartiteError> {
        let vdag = build_test_vdag();
        let (nodes, edges) = from_vdag(&vdag)?;
        let (mut left, mut right, filtered_edges) = partition_and_filter(nodes, edges);
        let cfg = default_config();
        let layout = layout_bipartite(&mut left, &mut right, &filtered_edges, &cfg);
        let svg = render_bipartite_svg(&layout, "Drug Network", &Theme::default());
        assert!(svg.starts_with("<svg"), "output should start with <svg");
        assert!(svg.ends_with("</svg>"), "output should end with </svg>");
        Ok(())
    }

    #[test]
    fn render_bipartite_svg_contains_title() -> Result<(), BipartiteError> {
        let vdag = build_test_vdag();
        let (nodes, edges) = from_vdag(&vdag)?;
        let (mut left, mut right, filtered_edges) = partition_and_filter(nodes, edges);
        let cfg = default_config();
        let layout = layout_bipartite(&mut left, &mut right, &filtered_edges, &cfg);
        let svg = render_bipartite_svg(&layout, "My Custom Title", &Theme::default());
        assert!(
            svg.contains("My Custom Title"),
            "SVG should contain the title"
        );
        Ok(())
    }

    #[test]
    fn render_bipartite_svg_contains_column_headers() -> Result<(), BipartiteError> {
        let vdag = build_test_vdag();
        let (nodes, edges) = from_vdag(&vdag)?;
        let (mut left, mut right, filtered_edges) = partition_and_filter(nodes, edges);
        let cfg = default_config();
        let layout = layout_bipartite(&mut left, &mut right, &filtered_edges, &cfg);
        let svg = render_bipartite_svg(&layout, "T", &Theme::default());
        assert!(svg.contains("Drugs"), "SVG should contain 'Drugs' header");
        assert!(
            svg.contains("Adverse Events"),
            "SVG should contain 'Adverse Events' header"
        );
        Ok(())
    }

    #[test]
    fn render_bipartite_svg_contains_bezier_paths() -> Result<(), BipartiteError> {
        let vdag = build_test_vdag();
        let (nodes, edges) = from_vdag(&vdag)?;
        let (mut left, mut right, filtered_edges) = partition_and_filter(nodes, edges);
        let cfg = default_config();
        let layout = layout_bipartite(&mut left, &mut right, &filtered_edges, &cfg);
        let svg = render_bipartite_svg(&layout, "T", &Theme::default());
        // Cubic Bézier paths use the " C " command
        assert!(
            svg.contains(" C "),
            "SVG should contain cubic Bézier curve paths"
        );
        Ok(())
    }

    // =========================================================================
    // default_config / BipartiteConfig::default tests
    // =========================================================================

    #[test]
    fn default_config_values_are_correct() {
        let cfg = default_config();
        assert_eq!(cfg.left_x, 100.0);
        assert_eq!(cfg.right_x, 500.0);
        assert_eq!(cfg.node_spacing, 40.0);
        assert_eq!(cfg.margin, 60.0);
        assert_eq!(cfg.node_width, 140.0);
        assert_eq!(cfg.node_height, 30.0);
        assert_eq!(cfg.max_iterations, 25);
        assert_eq!(cfg.signal_threshold, 2.0);
    }

    #[test]
    fn bipartite_config_default_matches_default_config() {
        let a = BipartiteConfig::default();
        let b = default_config();
        assert_eq!(a.left_x, b.left_x);
        assert_eq!(a.signal_threshold, b.signal_threshold);
        assert_eq!(a.max_iterations, b.max_iterations);
    }

    // =========================================================================
    // weight_to_color tests
    // =========================================================================

    #[test]
    fn weight_to_color_below_threshold_is_slate() {
        assert_eq!(weight_to_color(1.5, 2.0), palette::SLATE);
    }

    #[test]
    fn weight_to_color_at_threshold_is_amber() {
        assert_eq!(weight_to_color(2.0, 2.0), palette::AMBER);
    }

    #[test]
    fn weight_to_color_moderate_signal_is_amber() {
        assert_eq!(weight_to_color(3.5, 2.0), palette::AMBER);
    }

    #[test]
    fn weight_to_color_strong_signal_is_red() {
        assert_eq!(weight_to_color(4.0, 2.0), palette::RED);
    }

    #[test]
    fn weight_to_color_very_strong_is_red() {
        assert_eq!(weight_to_color(10.0, 2.0), palette::RED);
    }

    // =========================================================================
    // weight_to_opacity tests
    // =========================================================================

    #[test]
    fn weight_to_opacity_zero_weight_gives_min() {
        let op = weight_to_opacity(0.0, 5.0);
        assert!(
            (op - 0.3).abs() < 1e-9,
            "zero weight should give opacity 0.3, got {op}"
        );
    }

    #[test]
    fn weight_to_opacity_max_weight_gives_one() {
        let op = weight_to_opacity(5.0, 5.0);
        assert!(
            (op - 1.0).abs() < 1e-9,
            "max weight should give opacity 1.0, got {op}"
        );
    }

    #[test]
    fn weight_to_opacity_half_weight_midpoint() {
        let op = weight_to_opacity(2.5, 5.0);
        // 0.3 + 0.5 * 0.7 = 0.65
        assert!(
            (op - 0.65).abs() < 1e-9,
            "half weight should give opacity 0.65, got {op}"
        );
    }

    #[test]
    fn weight_to_opacity_zero_max_returns_min() {
        let op = weight_to_opacity(3.0, 0.0);
        assert!(
            (op - 0.3).abs() < 1e-9,
            "zero max_weight should safely return 0.3, got {op}"
        );
    }

    #[test]
    fn weight_to_opacity_is_clamped() {
        let op = weight_to_opacity(100.0, 5.0);
        assert!(op <= 1.0, "opacity should never exceed 1.0, got {op}");
        assert!(op >= 0.3, "opacity should never fall below 0.3, got {op}");
    }

    // =========================================================================
    // Node ordering after minimization
    // =========================================================================

    #[test]
    fn node_ordering_after_minimization_is_consistent() {
        let mut left = vec![
            make_bi_node("L1", Side::Left, 0.0),
            make_bi_node("L2", Side::Left, 1.0),
        ];
        let mut right = vec![
            make_bi_node("R1", Side::Right, 0.0),
            make_bi_node("R2", Side::Right, 1.0),
        ];
        // L1→R2, L2→R1: 1 crossing — minimizer should swap right to R2, R1
        // Barycenter: R2 neighbor L1 at pos 0 → 0.0, R1 neighbor L2 at pos 1 → 1.0
        let edges = vec![
            make_bi_edge("L1", "R2", 3.0),
            make_bi_edge("L2", "R1", 3.0),
        ];
        minimize_crossings(&mut left, &mut right, &edges, 10);
        assert_eq!(
            right[0].id, "R2",
            "R2 should be first after crossing minimization (barycenter=0.0)"
        );
        assert_eq!(
            right[1].id, "R1",
            "R1 should be second after crossing minimization (barycenter=1.0)"
        );
    }

    // =========================================================================
    // Serde roundtrip
    // =========================================================================

    #[test]
    fn bipartite_layout_serde_roundtrip() -> Result<(), serde_json::Error> {
        let mut left = vec![make_bi_node("D", Side::Left, 0.5)];
        let mut right = vec![make_bi_node("E", Side::Right, 0.5)];
        let edges = vec![make_bi_edge("D", "E", 2.5)];
        let cfg = default_config();
        let layout = layout_bipartite(&mut left, &mut right, &edges, &cfg);
        let json = serde_json::to_string(&layout)?;
        let restored = serde_json::from_str::<BipartiteLayout>(&json)?;
        assert_eq!(restored.left_nodes.len(), layout.left_nodes.len());
        assert_eq!(restored.right_nodes.len(), layout.right_nodes.len());
        assert_eq!(restored.edges.len(), layout.edges.len());
        Ok(())
    }
}
