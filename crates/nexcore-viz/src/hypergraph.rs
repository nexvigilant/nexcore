//! Hypergraph Data Structures and Algorithms
//!
//! Provides types and algorithms for hypergraphs — generalisations of ordinary
//! graphs where a single edge (called a *hyperedge*) can connect **two or more**
//! nodes simultaneously.  This is the natural structure for many biological and
//! regulatory relationships: multi-drug interactions, pathway groupings, and
//! regulatory co-occurrence.
//!
//! ## Types
//!
//! | Type | Purpose |
//! |------|---------|
//! | [`HyperNode`] | A labelled, optionally-weighted node with optional 3-D position |
//! | [`HyperEdge`] | A hyperedge connecting >=2 nodes with a weight and label |
//! | [`Hypergraph`] | Container holding nodes, edges, and a directed flag |
//! | [`IncidenceMatrix`] | Dense node x edge matrix (1.0 if node participates in edge) |
//! | [`DualGraph`] | Dual representation: hyperedges -> nodes, shared-node -> edge |
//! | [`SteinerTree`] | Minimal spanning subgraph connecting a set of terminal nodes |
//! | [`HypergraphLayout`] | 2-D positions for every node (for rendering) |
//! | [`HypergraphMetrics`] | Summary statistics for a hypergraph |
//!
//! ## Algorithms
//!
//! | Function | Description |
//! |----------|-------------|
//! | [`build_incidence_matrix`] | Construct the node x edge incidence matrix |
//! | [`build_adjacency_from_hypergraph`] | Clique-expansion adjacency via shared hyperedges |
//! | [`compute_dual`] | Dual-graph construction |
//! | [`approximate_steiner_tree`] | Greedy 2-approximation using shortest paths |
//! | [`force_directed_layout`] | Spring-electrical layout for hypergraphs |
//! | [`compute_metrics`] | Summary statistics |
//! | [`connected_components`] | BFS-based component detection |
//! | [`detect_bipartite`] | 2-colourability (bipartite) check |
//!
//! # Example
//!
//! ```rust
//! use nexcore_viz::hypergraph::{Hypergraph, HyperNode, HyperEdge, compute_metrics};
//!
//! let mut hg = Hypergraph::new(false);
//! let n0 = hg.add_node(HyperNode { id: 0, label: "A".into(), weight: 1.0, metadata: None, position: None });
//! let n1 = hg.add_node(HyperNode { id: 1, label: "B".into(), weight: 1.0, metadata: None, position: None });
//! let n2 = hg.add_node(HyperNode { id: 2, label: "C".into(), weight: 1.0, metadata: None, position: None });
//! assert!(hg.add_edge(HyperEdge { id: 0, nodes: vec![n0, n1, n2], weight: 1.0, label: "e0".into() }).is_ok());
//!
//! let m = compute_metrics(&hg);
//! assert_eq!(m.node_count, 3);
//! assert_eq!(m.edge_count, 1);
//! assert_eq!(m.avg_edge_size, 3.0);
//! ```

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;

// ─── Error Type ───────────────────────────────────────────────────────────────

/// Errors that can arise from hypergraph operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HypergraphError {
    /// A referenced node index does not exist in the hypergraph.
    InvalidNode(usize),
    /// A hyperedge is malformed (e.g., fewer than 2 nodes, or references a
    /// non-existent node index).
    InvalidEdge(String),
    /// The hypergraph has no nodes, making the requested operation undefined.
    EmptyHypergraph,
    /// A cycle was detected where the algorithm requires a DAG.
    CycleDetected,
}

impl fmt::Display for HypergraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidNode(id) => write!(f, "node index {id} does not exist"),
            Self::InvalidEdge(msg) => write!(f, "invalid hyperedge: {msg}"),
            Self::EmptyHypergraph => write!(f, "operation requires a non-empty hypergraph"),
            Self::CycleDetected => write!(f, "cycle detected in hypergraph"),
        }
    }
}

impl std::error::Error for HypergraphError {}

// ─── Public Types ─────────────────────────────────────────────────────────────

/// A node within a hypergraph.
///
/// The `id` field is the *logical* identifier supplied by the caller; the
/// index within [`Hypergraph::nodes`] is what hyperedge membership references.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperNode {
    /// Caller-supplied logical identifier (need not be sequential).
    pub id: usize,
    /// Human-readable label.
    pub label: String,
    /// Node weight (e.g., importance score, signal strength).
    pub weight: f64,
    /// Optional free-form metadata (JSON string, description, etc.).
    pub metadata: Option<String>,
    /// Optional 3-D position `[x, y, z]` for pre-laid-out data.
    pub position: Option<[f64; 3]>,
}

/// A hyperedge connecting two or more nodes.
///
/// `nodes` holds **indices into [`Hypergraph::nodes`]**, not logical node IDs.
/// Every index must be in bounds (< `hypergraph.node_count()`).  A hyperedge
/// must reference at least 2 distinct nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperEdge {
    /// Caller-supplied logical identifier.
    pub id: usize,
    /// Indices (into [`Hypergraph::nodes`]) of participating nodes.
    pub nodes: Vec<usize>,
    /// Edge weight (e.g., confidence, interaction strength).
    pub weight: f64,
    /// Human-readable label.
    pub label: String,
}

/// Container for a hypergraph: a set of nodes and hyperedges.
///
/// Hyperedges are stored by value.  Node membership is expressed as **indices
/// into `nodes`** (0-based), so adding nodes invalidates previously recorded
/// indices only if the nodes vec is re-ordered (it is not — nodes are always
/// appended).
///
/// # Example
///
/// ```rust
/// use nexcore_viz::hypergraph::{Hypergraph, HyperNode, HyperEdge};
///
/// let mut hg = Hypergraph::new(false);
/// let i0 = hg.add_node(HyperNode { id: 0, label: "drug-A".into(), weight: 1.0, metadata: None, position: None });
/// let i1 = hg.add_node(HyperNode { id: 1, label: "drug-B".into(), weight: 1.0, metadata: None, position: None });
/// assert!(hg.add_edge(HyperEdge { id: 0, nodes: vec![i0, i1], weight: 0.8, label: "interaction".into() }).is_ok());
/// assert_eq!(hg.node_count(), 2);
/// assert_eq!(hg.edge_count(), 1);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Hypergraph {
    /// Ordered list of nodes.  Indices here are what hyperedges reference.
    pub nodes: Vec<HyperNode>,
    /// Ordered list of hyperedges.
    pub edges: Vec<HyperEdge>,
    /// When `true`, edges are considered directed (source->target ordering matters).
    pub directed: bool,
}

impl Hypergraph {
    /// Creates an empty hypergraph.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexcore_viz::hypergraph::Hypergraph;
    /// let hg = Hypergraph::new(false);
    /// assert_eq!(hg.node_count(), 0);
    /// ```
    #[must_use]
    pub fn new(directed: bool) -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            directed,
        }
    }

    /// Appends a node and returns its **index** in `self.nodes`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexcore_viz::hypergraph::{Hypergraph, HyperNode};
    /// let mut hg = Hypergraph::new(false);
    /// let idx = hg.add_node(HyperNode { id: 42, label: "X".into(), weight: 1.0, metadata: None, position: None });
    /// assert_eq!(idx, 0);
    /// ```
    pub fn add_node(&mut self, node: HyperNode) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(node);
        idx
    }

    /// Validates and appends a hyperedge.
    ///
    /// Fails if:
    /// - the edge has fewer than 2 node references, or
    /// - any referenced node index is out of bounds.
    ///
    /// Returns `Ok(edge_index)` on success.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::InvalidEdge`] for structural problems.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexcore_viz::hypergraph::{Hypergraph, HyperNode, HyperEdge};
    ///
    /// let mut hg = Hypergraph::new(false);
    /// let i0 = hg.add_node(HyperNode { id: 0, label: "A".into(), weight: 1.0, metadata: None, position: None });
    /// let i1 = hg.add_node(HyperNode { id: 1, label: "B".into(), weight: 1.0, metadata: None, position: None });
    /// let result = hg.add_edge(HyperEdge { id: 0, nodes: vec![i0, i1], weight: 1.0, label: "e".into() });
    /// assert!(result.is_ok());
    /// ```
    pub fn add_edge(&mut self, edge: HyperEdge) -> Result<usize, HypergraphError> {
        if edge.nodes.len() < 2 {
            return Err(HypergraphError::InvalidEdge(
                "hyperedge must reference at least 2 nodes".into(),
            ));
        }
        let n = self.nodes.len();
        for &node_idx in &edge.nodes {
            if node_idx >= n {
                return Err(HypergraphError::InvalidEdge(format!(
                    "node index {node_idx} is out of bounds (hypergraph has {n} nodes)"
                )));
            }
        }
        let idx = self.edges.len();
        self.edges.push(edge);
        Ok(idx)
    }

    /// Returns the number of nodes.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the number of hyperedges.
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Returns the **degree** of a node: the number of hyperedges it belongs to.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::InvalidNode`] if `node_idx` is out of bounds.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexcore_viz::hypergraph::{Hypergraph, HyperNode, HyperEdge};
    ///
    /// let mut hg = Hypergraph::new(false);
    /// let i0 = hg.add_node(HyperNode { id: 0, label: "A".into(), weight: 1.0, metadata: None, position: None });
    /// let i1 = hg.add_node(HyperNode { id: 1, label: "B".into(), weight: 1.0, metadata: None, position: None });
    /// assert!(hg.add_edge(HyperEdge { id: 0, nodes: vec![i0, i1], weight: 1.0, label: "e".into() }).is_ok());
    /// assert_eq!(hg.degree(i0).ok(), Some(1));
    /// ```
    pub fn degree(&self, node_idx: usize) -> Result<usize, HypergraphError> {
        if node_idx >= self.nodes.len() {
            return Err(HypergraphError::InvalidNode(node_idx));
        }
        let count = self
            .edges
            .iter()
            .filter(|e| e.nodes.contains(&node_idx))
            .count();
        Ok(count)
    }

    /// Returns indices of all hyperedges that contain `node_idx`.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::InvalidNode`] if `node_idx` is out of bounds.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexcore_viz::hypergraph::{Hypergraph, HyperNode, HyperEdge};
    ///
    /// let mut hg = Hypergraph::new(false);
    /// let i0 = hg.add_node(HyperNode { id: 0, label: "A".into(), weight: 1.0, metadata: None, position: None });
    /// let i1 = hg.add_node(HyperNode { id: 1, label: "B".into(), weight: 1.0, metadata: None, position: None });
    /// assert!(hg.add_edge(HyperEdge { id: 0, nodes: vec![i0, i1], weight: 1.0, label: "e".into() }).is_ok());
    /// let containing = hg.edges_containing(i0).ok().unwrap_or_default();
    /// assert_eq!(containing, vec![0usize]);
    /// ```
    pub fn edges_containing(&self, node_idx: usize) -> Result<Vec<usize>, HypergraphError> {
        if node_idx >= self.nodes.len() {
            return Err(HypergraphError::InvalidNode(node_idx));
        }
        let result = self
            .edges
            .iter()
            .enumerate()
            .filter_map(|(ei, e)| {
                if e.nodes.contains(&node_idx) {
                    Some(ei)
                } else {
                    None
                }
            })
            .collect();
        Ok(result)
    }

    /// Returns the set of node indices reachable from `node_idx` via shared
    /// hyperedges (i.e., nodes that co-occur in at least one hyperedge with
    /// `node_idx`, excluding `node_idx` itself).
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::InvalidNode`] if `node_idx` is out of bounds.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nexcore_viz::hypergraph::{Hypergraph, HyperNode, HyperEdge};
    ///
    /// let mut hg = Hypergraph::new(false);
    /// let i0 = hg.add_node(HyperNode { id: 0, label: "A".into(), weight: 1.0, metadata: None, position: None });
    /// let i1 = hg.add_node(HyperNode { id: 1, label: "B".into(), weight: 1.0, metadata: None, position: None });
    /// let i2 = hg.add_node(HyperNode { id: 2, label: "C".into(), weight: 1.0, metadata: None, position: None });
    /// assert!(hg.add_edge(HyperEdge { id: 0, nodes: vec![i0, i1, i2], weight: 1.0, label: "e".into() }).is_ok());
    /// let mut nbrs = hg.neighbors(i0).ok().unwrap_or_default();
    /// nbrs.sort();
    /// assert_eq!(nbrs, vec![1, 2]);
    /// ```
    pub fn neighbors(&self, node_idx: usize) -> Result<Vec<usize>, HypergraphError> {
        if node_idx >= self.nodes.len() {
            return Err(HypergraphError::InvalidNode(node_idx));
        }
        let mut seen: HashSet<usize> = HashSet::new();
        for edge in &self.edges {
            if edge.nodes.contains(&node_idx) {
                for &member in &edge.nodes {
                    if member != node_idx {
                        seen.insert(member);
                    }
                }
            }
        }
        Ok(seen.into_iter().collect())
    }
}

// ─── Derived Structures ───────────────────────────────────────────────────────

/// Dense node x edge incidence matrix.
///
/// `data[i][j] = 1.0` if node `i` participates in edge `j`; `0.0` otherwise.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidenceMatrix {
    /// Number of rows (nodes).
    pub rows: usize,
    /// Number of columns (edges).
    pub cols: usize,
    /// Row-major data: `data[node_idx][edge_idx]`.
    pub data: Vec<Vec<f64>>,
}

/// Dual graph of a hypergraph.
///
/// In the dual, each **hyperedge** of the original becomes a **node**, and two
/// dual nodes are connected by an edge if and only if the corresponding
/// hyperedges share at least one common node in the original hypergraph.
///
/// Node labels in `nodes` are the `label` fields of the original hyperedges.
/// Edge weights are the number of shared nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DualGraph {
    /// Dual nodes (one per original hyperedge).  Label = original edge label.
    pub nodes: Vec<String>,
    /// Dual edges `(dual_node_a, dual_node_b, shared_node_count)`.
    pub edges: Vec<(usize, usize, usize)>,
}

/// A Steiner tree approximation connecting a set of terminal nodes.
///
/// The `nodes` field lists all node indices included in the tree (terminals
/// plus any Steiner points), and `edges` lists the `(u, v)` pairs forming the
/// spanning tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteinerTree {
    /// All node indices in the tree (terminals + Steiner nodes).
    pub nodes: Vec<usize>,
    /// Undirected edges `(u, v)` making up the spanning tree.
    pub edges: Vec<(usize, usize)>,
    /// Sum of edge weights (using clique-expansion adjacency weights).
    pub total_weight: f64,
}

/// 2-D layout positions for every node in a hypergraph.
///
/// `positions[i] = [x, y]` for the node at index `i`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypergraphLayout {
    /// One `[x, y]` entry per node, in node-index order.
    pub positions: Vec<[f64; 2]>,
}

/// Summary statistics for a hypergraph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypergraphMetrics {
    /// Total number of nodes.
    pub node_count: usize,
    /// Total number of hyperedges.
    pub edge_count: usize,
    /// Mean number of nodes per hyperedge (`0.0` if no edges).
    pub avg_edge_size: f64,
    /// Maximum number of nodes in any single hyperedge (`0` if no edges).
    pub max_edge_size: usize,
    /// Hypergraph density: `edge_count / (node_count choose 2)`.
    ///
    /// Uses the ordinary graph density formula as a normalisation baseline.
    /// Returns `0.0` when there are fewer than 2 nodes.
    pub density: f64,
    /// Number of connected components (using clique-expansion adjacency).
    pub connected_components: usize,
}

// ─── Algorithm: Incidence Matrix ─────────────────────────────────────────────

/// Builds the **node x edge incidence matrix** for `hg`.
///
/// `result.data[i][j] = 1.0` iff node index `i` participates in edge `j`;
/// `0.0` otherwise.
///
/// Returns an all-zero matrix for an empty hypergraph.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::hypergraph::{Hypergraph, HyperNode, HyperEdge, build_incidence_matrix};
///
/// let mut hg = Hypergraph::new(false);
/// let i0 = hg.add_node(HyperNode { id: 0, label: "A".into(), weight: 1.0, metadata: None, position: None });
/// let i1 = hg.add_node(HyperNode { id: 1, label: "B".into(), weight: 1.0, metadata: None, position: None });
/// assert!(hg.add_edge(HyperEdge { id: 0, nodes: vec![i0, i1], weight: 1.0, label: "e".into() }).is_ok());
/// let mat = build_incidence_matrix(&hg);
/// assert_eq!(mat.data[0][0], 1.0);
/// assert_eq!(mat.data[1][0], 1.0);
/// ```
#[must_use]
pub fn build_incidence_matrix(hg: &Hypergraph) -> IncidenceMatrix {
    let rows = hg.node_count();
    let cols = hg.edge_count();
    let mut data = vec![vec![0.0_f64; cols]; rows];
    for (ei, edge) in hg.edges.iter().enumerate() {
        for &ni in &edge.nodes {
            if ni < rows {
                data[ni][ei] = 1.0;
            }
        }
    }
    IncidenceMatrix { rows, cols, data }
}

// ─── Algorithm: Adjacency from Hypergraph ─────────────────────────────────────

/// Builds a weighted adjacency matrix from the hypergraph using **clique
/// expansion**.
///
/// For each hyperedge `e` with weight `w`, every pair `(u, v)` of distinct
/// nodes within `e` accumulates `w` in `A[u][v]` (and `A[v][u]`).  This is
/// the standard projection of a hypergraph onto an ordinary weighted graph.
///
/// Returns an `n x n` matrix (row-major `Vec<Vec<f64>>`).
///
/// # Example
///
/// ```rust
/// use nexcore_viz::hypergraph::{Hypergraph, HyperNode, HyperEdge, build_adjacency_from_hypergraph};
///
/// let mut hg = Hypergraph::new(false);
/// let i0 = hg.add_node(HyperNode { id: 0, label: "A".into(), weight: 1.0, metadata: None, position: None });
/// let i1 = hg.add_node(HyperNode { id: 1, label: "B".into(), weight: 1.0, metadata: None, position: None });
/// assert!(hg.add_edge(HyperEdge { id: 0, nodes: vec![i0, i1], weight: 2.0, label: "e".into() }).is_ok());
/// let adj = build_adjacency_from_hypergraph(&hg);
/// assert_eq!(adj[0][1], 2.0);
/// assert_eq!(adj[1][0], 2.0);
/// ```
#[must_use]
pub fn build_adjacency_from_hypergraph(hg: &Hypergraph) -> Vec<Vec<f64>> {
    let n = hg.node_count();
    let mut adj = vec![vec![0.0_f64; n]; n];
    for edge in &hg.edges {
        let members = &edge.nodes;
        let w = edge.weight;
        for (a, &u) in members.iter().enumerate() {
            for &v in members.iter().skip(a + 1) {
                if u < n && v < n {
                    adj[u][v] += w;
                    adj[v][u] += w;
                }
            }
        }
    }
    adj
}

// ─── Algorithm: Dual Graph ───────────────────────────────────────────────────

/// Constructs the **dual graph** of `hg`.
///
/// Each hyperedge of `hg` becomes a node in the dual.  Two dual nodes are
/// connected if their corresponding hyperedges share at least one node.  The
/// edge weight stored in [`DualGraph::edges`] is the count of shared nodes.
///
/// # Errors
///
/// Returns [`HypergraphError::EmptyHypergraph`] if `hg` has no edges (the
/// dual would be trivially empty and is likely a programming error).
///
/// # Example
///
/// ```rust
/// use nexcore_viz::hypergraph::{Hypergraph, HyperNode, HyperEdge, compute_dual};
///
/// let mut hg = Hypergraph::new(false);
/// let i0 = hg.add_node(HyperNode { id: 0, label: "A".into(), weight: 1.0, metadata: None, position: None });
/// let i1 = hg.add_node(HyperNode { id: 1, label: "B".into(), weight: 1.0, metadata: None, position: None });
/// let i2 = hg.add_node(HyperNode { id: 2, label: "C".into(), weight: 1.0, metadata: None, position: None });
/// assert!(hg.add_edge(HyperEdge { id: 0, nodes: vec![i0, i1], weight: 1.0, label: "e0".into() }).is_ok());
/// assert!(hg.add_edge(HyperEdge { id: 1, nodes: vec![i1, i2], weight: 1.0, label: "e1".into() }).is_ok());
/// if let Ok(dual) = compute_dual(&hg) {
///     assert_eq!(dual.nodes.len(), 2);
///     assert_eq!(dual.edges.len(), 1); // e0 and e1 share node i1
/// }
/// ```
pub fn compute_dual(hg: &Hypergraph) -> Result<DualGraph, HypergraphError> {
    if hg.edge_count() == 0 {
        return Err(HypergraphError::EmptyHypergraph);
    }
    let m = hg.edge_count();
    let nodes: Vec<String> = hg.edges.iter().map(|e| e.label.clone()).collect();

    let mut edges: Vec<(usize, usize, usize)> = Vec::new();
    for a in 0..m {
        for b in (a + 1)..m {
            let set_a: HashSet<usize> = hg.edges[a].nodes.iter().copied().collect();
            let set_b: HashSet<usize> = hg.edges[b].nodes.iter().copied().collect();
            let shared = set_a.intersection(&set_b).count();
            if shared > 0 {
                edges.push((a, b, shared));
            }
        }
    }
    Ok(DualGraph { nodes, edges })
}

// ─── Algorithm: Steiner Tree (Greedy 2-Approximation) ────────────────────────

/// Computes a greedy **2-approximation of the Steiner tree** connecting all
/// `terminals` using the clique-expansion adjacency.
///
/// The algorithm:
/// 1. Start with the first terminal as the current tree.
/// 2. Repeatedly find the terminal not yet in the tree that is closest
///    (shortest path in the clique-expansion graph) to **any** node already
///    in the tree, and add the path.
/// 3. Repeat until all terminals are connected.
///
/// Uses Dijkstra's algorithm on the clique-expansion adjacency matrix.
///
/// Nodes with no edge (weight 0 adjacency) are treated as disconnected.
///
/// # Errors
///
/// - [`HypergraphError::EmptyHypergraph`] — no nodes in the hypergraph.
/// - [`HypergraphError::InvalidNode`] — a terminal index is out of bounds.
/// - [`HypergraphError::InvalidEdge`] — fewer than 2 terminals provided.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::hypergraph::{Hypergraph, HyperNode, HyperEdge, approximate_steiner_tree};
///
/// let mut hg = Hypergraph::new(false);
/// let i0 = hg.add_node(HyperNode { id: 0, label: "A".into(), weight: 1.0, metadata: None, position: None });
/// let i1 = hg.add_node(HyperNode { id: 1, label: "B".into(), weight: 1.0, metadata: None, position: None });
/// let i2 = hg.add_node(HyperNode { id: 2, label: "C".into(), weight: 1.0, metadata: None, position: None });
/// assert!(hg.add_edge(HyperEdge { id: 0, nodes: vec![i0, i1, i2], weight: 1.0, label: "e".into() }).is_ok());
/// if let Ok(st) = approximate_steiner_tree(&hg, &[i0, i2]) {
///     assert!(st.nodes.contains(&i0));
///     assert!(st.nodes.contains(&i2));
/// }
/// ```
pub fn approximate_steiner_tree(
    hg: &Hypergraph,
    terminals: &[usize],
) -> Result<SteinerTree, HypergraphError> {
    let n = hg.node_count();
    if n == 0 {
        return Err(HypergraphError::EmptyHypergraph);
    }
    if terminals.len() < 2 {
        return Err(HypergraphError::InvalidEdge(
            "Steiner tree requires at least 2 terminals".into(),
        ));
    }
    for &t in terminals {
        if t >= n {
            return Err(HypergraphError::InvalidNode(t));
        }
    }

    let adj = build_adjacency_from_hypergraph(hg);

    let mut tree_nodes: HashSet<usize> = HashSet::new();
    let mut tree_edges: Vec<(usize, usize)> = Vec::new();
    let mut total_weight = 0.0_f64;

    // Seed with first terminal.
    let first = terminals[0];
    tree_nodes.insert(first);

    let mut yet_to_add: Vec<usize> = terminals[1..].to_vec();

    while !yet_to_add.is_empty() {
        // Find the terminal in `yet_to_add` closest to any node already in
        // the tree via Dijkstra.
        let mut best_dist = f64::INFINITY;
        let mut best_path: Vec<usize> = Vec::new();
        let mut best_terminal_idx = 0usize;

        for (ti, &term) in yet_to_add.iter().enumerate() {
            // Run Dijkstra from `term` back to any tree node.
            let (dist_map, predecessors) = dijkstra(&adj, term, n);

            // Find the tree node reachable from `term` with minimum distance.
            for &tree_node in &tree_nodes {
                let d = dist_map.get(&tree_node).copied().unwrap_or(f64::INFINITY);
                if d < best_dist {
                    best_dist = d;
                    best_terminal_idx = ti;
                    // Reconstruct path from term -> tree_node.
                    best_path = reconstruct_path(&predecessors, term, tree_node);
                }
            }
        }

        // Add the best path to the tree.
        for window in best_path.windows(2) {
            let u = window[0];
            let v = window[1];
            let w = adj
                .get(u)
                .and_then(|row| row.get(v))
                .copied()
                .unwrap_or(0.0);
            if !tree_nodes.contains(&u) || !tree_nodes.contains(&v) {
                tree_edges.push((u, v));
                total_weight += w;
            }
            tree_nodes.insert(u);
            tree_nodes.insert(v);
        }

        yet_to_add.remove(best_terminal_idx);
    }

    let mut nodes_vec: Vec<usize> = tree_nodes.into_iter().collect();
    nodes_vec.sort_unstable();

    Ok(SteinerTree {
        nodes: nodes_vec,
        edges: tree_edges,
        total_weight,
    })
}

// ─── Algorithm: Force-Directed Layout ────────────────────────────────────────

/// Computes a **2-D spring-electrical force-directed layout** for `hg`.
///
/// Nodes repel each other (electrical) while nodes sharing a hyperedge attract
/// each other (spring).  Hyperedges act as attractive groups by pulling all
/// their member nodes toward the edge's centroid.
///
/// Positions are initialised on a unit circle, then refined for `iterations`
/// steps.  Returns one `[x, y]` per node in node-index order.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::hypergraph::{Hypergraph, HyperNode, HyperEdge, force_directed_layout};
///
/// let mut hg = Hypergraph::new(false);
/// let i0 = hg.add_node(HyperNode { id: 0, label: "A".into(), weight: 1.0, metadata: None, position: None });
/// let i1 = hg.add_node(HyperNode { id: 1, label: "B".into(), weight: 1.0, metadata: None, position: None });
/// assert!(hg.add_edge(HyperEdge { id: 0, nodes: vec![i0, i1], weight: 1.0, label: "e".into() }).is_ok());
/// let layout = force_directed_layout(&hg, 50);
/// assert_eq!(layout.positions.len(), 2);
/// ```
#[must_use]
pub fn force_directed_layout(hg: &Hypergraph, iterations: usize) -> HypergraphLayout {
    let n = hg.node_count();
    if n == 0 {
        return HypergraphLayout { positions: vec![] };
    }
    if n == 1 {
        return HypergraphLayout {
            positions: vec![[0.0, 0.0]],
        };
    }

    // Initialise on a unit circle.
    let mut pos: Vec<[f64; 2]> = (0..n)
        .map(|i| {
            let angle = 2.0 * std::f64::consts::PI * i as f64 / n as f64;
            [angle.cos(), angle.sin()]
        })
        .collect();

    let k = 1.0_f64; // spring constant
    let repulsion = 1.0_f64;
    let step = 0.05_f64;

    for _ in 0..iterations {
        let mut forces: Vec<[f64; 2]> = vec![[0.0, 0.0]; n];

        // Repulsion: every pair of nodes pushes apart (Coulomb-like).
        for i in 0..n {
            for j in (i + 1)..n {
                let dx = pos[i][0] - pos[j][0];
                let dy = pos[i][1] - pos[j][1];
                let dist_sq = dx * dx + dy * dy;
                let dist = dist_sq.sqrt().max(1e-6);
                let force = repulsion / dist_sq;
                let fx = force * dx / dist;
                let fy = force * dy / dist;
                forces[i][0] += fx;
                forces[i][1] += fy;
                forces[j][0] -= fx;
                forces[j][1] -= fy;
            }
        }

        // Attraction: each hyperedge pulls its members toward the centroid.
        for edge in &hg.edges {
            let members: Vec<usize> = edge.nodes.iter().copied().filter(|&ni| ni < n).collect();
            if members.is_empty() {
                continue;
            }
            let cx: f64 = members.iter().map(|&ni| pos[ni][0]).sum::<f64>() / members.len() as f64;
            let cy: f64 = members.iter().map(|&ni| pos[ni][1]).sum::<f64>() / members.len() as f64;
            for &ni in &members {
                let dx = cx - pos[ni][0];
                let dy = cy - pos[ni][1];
                let dist = (dx * dx + dy * dy).sqrt().max(1e-6);
                let force = k * dist * edge.weight;
                forces[ni][0] += force * dx / dist;
                forces[ni][1] += force * dy / dist;
            }
        }

        // Apply forces.
        for i in 0..n {
            pos[i][0] += step * forces[i][0];
            pos[i][1] += step * forces[i][1];
        }
    }

    HypergraphLayout { positions: pos }
}

// ─── Algorithm: Metrics ───────────────────────────────────────────────────────

/// Computes summary statistics for `hg`.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::hypergraph::{Hypergraph, HyperNode, HyperEdge, compute_metrics};
///
/// let mut hg = Hypergraph::new(false);
/// let i0 = hg.add_node(HyperNode { id: 0, label: "A".into(), weight: 1.0, metadata: None, position: None });
/// let i1 = hg.add_node(HyperNode { id: 1, label: "B".into(), weight: 1.0, metadata: None, position: None });
/// assert!(hg.add_edge(HyperEdge { id: 0, nodes: vec![i0, i1], weight: 1.0, label: "e".into() }).is_ok());
/// let m = compute_metrics(&hg);
/// assert_eq!(m.node_count, 2);
/// assert_eq!(m.edge_count, 1);
/// ```
#[must_use]
pub fn compute_metrics(hg: &Hypergraph) -> HypergraphMetrics {
    let node_count = hg.node_count();
    let edge_count = hg.edge_count();

    let avg_edge_size = if edge_count == 0 {
        0.0
    } else {
        hg.edges.iter().map(|e| e.nodes.len()).sum::<usize>() as f64 / edge_count as f64
    };

    let max_edge_size = hg.edges.iter().map(|e| e.nodes.len()).max().unwrap_or(0);

    let density = if node_count < 2 {
        0.0
    } else {
        let pairs = node_count * (node_count - 1) / 2;
        edge_count as f64 / pairs as f64
    };

    let components = connected_components(hg).len();

    HypergraphMetrics {
        node_count,
        edge_count,
        avg_edge_size,
        max_edge_size,
        density,
        connected_components: components,
    }
}

// ─── Algorithm: Connected Components ─────────────────────────────────────────

/// Detects **connected components** using BFS on the clique-expansion
/// adjacency.
///
/// Returns a `Vec` of components; each component is a `Vec<usize>` of node
/// indices.  The components are sorted by size descending.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::hypergraph::{Hypergraph, HyperNode, HyperEdge, connected_components};
///
/// let mut hg = Hypergraph::new(false);
/// let i0 = hg.add_node(HyperNode { id: 0, label: "A".into(), weight: 1.0, metadata: None, position: None });
/// let i1 = hg.add_node(HyperNode { id: 1, label: "B".into(), weight: 1.0, metadata: None, position: None });
/// let i2 = hg.add_node(HyperNode { id: 2, label: "C".into(), weight: 1.0, metadata: None, position: None });
/// assert!(hg.add_edge(HyperEdge { id: 0, nodes: vec![i0, i1], weight: 1.0, label: "e".into() }).is_ok());
/// // i2 is isolated.
/// let comps = connected_components(&hg);
/// assert_eq!(comps.len(), 2);
/// ```
#[must_use]
pub fn connected_components(hg: &Hypergraph) -> Vec<Vec<usize>> {
    let n = hg.node_count();
    if n == 0 {
        return vec![];
    }

    // Build adjacency list from hyperedge membership.
    let mut adj: Vec<HashSet<usize>> = vec![HashSet::new(); n];
    for edge in &hg.edges {
        for (ai, &u) in edge.nodes.iter().enumerate() {
            for &v in edge.nodes.iter().skip(ai + 1) {
                if u < n && v < n {
                    adj[u].insert(v);
                    adj[v].insert(u);
                }
            }
        }
    }

    let mut visited = vec![false; n];
    let mut components: Vec<Vec<usize>> = Vec::new();

    for start in 0..n {
        if visited[start] {
            continue;
        }
        let mut component = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back(start);
        visited[start] = true;

        while let Some(node) = queue.pop_front() {
            component.push(node);
            for &nbr in &adj[node] {
                if !visited[nbr] {
                    visited[nbr] = true;
                    queue.push_back(nbr);
                }
            }
        }
        components.push(component);
    }

    // Sort: largest component first.
    components.sort_by_key(|c| std::cmp::Reverse(c.len()));
    components
}

// ─── Algorithm: Bipartite Detection ──────────────────────────────────────────

/// Returns `true` if the hypergraph is **2-colourable** (bipartite) in its
/// clique-expansion representation.
///
/// A hypergraph is 2-colourable when every odd-length hyperedge can be removed
/// and the resulting structure is still balanced.  This function tests the
/// clique-expansion adjacency for bipartiteness using BFS 2-colouring.
///
/// Returns `true` for empty hypergraphs (vacuously true).
///
/// # Example
///
/// ```rust
/// use nexcore_viz::hypergraph::{Hypergraph, HyperNode, HyperEdge, detect_bipartite};
///
/// // A simple path A-B-C is bipartite.
/// let mut hg = Hypergraph::new(false);
/// let i0 = hg.add_node(HyperNode { id: 0, label: "A".into(), weight: 1.0, metadata: None, position: None });
/// let i1 = hg.add_node(HyperNode { id: 1, label: "B".into(), weight: 1.0, metadata: None, position: None });
/// let i2 = hg.add_node(HyperNode { id: 2, label: "C".into(), weight: 1.0, metadata: None, position: None });
/// assert!(hg.add_edge(HyperEdge { id: 0, nodes: vec![i0, i1], weight: 1.0, label: "e0".into() }).is_ok());
/// assert!(hg.add_edge(HyperEdge { id: 1, nodes: vec![i1, i2], weight: 1.0, label: "e1".into() }).is_ok());
/// assert!(detect_bipartite(&hg));
/// ```
#[must_use]
pub fn detect_bipartite(hg: &Hypergraph) -> bool {
    let n = hg.node_count();
    if n == 0 {
        return true;
    }

    // Build adjacency list from hyperedge clique expansion.
    let mut adj: Vec<Vec<usize>> = vec![vec![]; n];
    for edge in &hg.edges {
        for (ai, &u) in edge.nodes.iter().enumerate() {
            for &v in edge.nodes.iter().skip(ai + 1) {
                if u < n && v < n {
                    adj[u].push(v);
                    adj[v].push(u);
                }
            }
        }
    }

    // BFS 2-colouring: colour[i] = 0 or 1; usize::MAX = unvisited.
    let mut colour: Vec<usize> = vec![usize::MAX; n];

    for start in 0..n {
        if colour[start] != usize::MAX {
            continue;
        }
        colour[start] = 0;
        let mut queue = VecDeque::new();
        queue.push_back(start);

        while let Some(u) = queue.pop_front() {
            let c_u = colour[u];
            for &v in &adj[u] {
                if colour[v] == usize::MAX {
                    colour[v] = 1 - c_u;
                    queue.push_back(v);
                } else if colour[v] == c_u {
                    return false; // odd cycle detected
                }
            }
        }
    }

    true
}

// ─── Internal Helpers ─────────────────────────────────────────────────────────

/// Dijkstra's shortest-path from `source` in the weighted adjacency matrix
/// `adj`.  Returns `(distances, predecessors)`.
///
/// - `distances[v]` = shortest distance from `source` to `v` (or
///   `f64::INFINITY` if unreachable).
/// - `predecessors[v]` = the node before `v` on the shortest path from
///   `source`, or `usize::MAX` if none.
///
/// Edges with weight `0.0` are treated as absent (no connection).
fn dijkstra(adj: &[Vec<f64>], source: usize, n: usize) -> (HashMap<usize, f64>, Vec<usize>) {
    let mut dist: Vec<f64> = vec![f64::INFINITY; n];
    let mut pred: Vec<usize> = vec![usize::MAX; n];
    let mut visited = vec![false; n];

    dist[source] = 0.0;

    for _ in 0..n {
        // Find the unvisited node with minimum distance (simple O(n^2) Dijkstra).
        let u = (0..n).filter(|&i| !visited[i]).min_by(|&a, &b| {
            dist[a]
                .partial_cmp(&dist[b])
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let Some(u) = u else { break };
        if dist[u] == f64::INFINITY {
            break;
        }
        visited[u] = true;

        let row = match adj.get(u) {
            Some(r) => r,
            None => continue,
        };
        for (v, &w) in row.iter().enumerate() {
            if w <= 0.0 || visited[v] {
                continue;
            }
            let new_dist = dist[u] + w;
            if new_dist < dist[v] {
                dist[v] = new_dist;
                pred[v] = u;
            }
        }
    }

    let dist_map: HashMap<usize, f64> = (0..n).map(|i| (i, dist[i])).collect();
    (dist_map, pred)
}

/// Reconstructs the path from `source` to `target` using a `predecessors`
/// array produced by [`dijkstra`].
///
/// Returns an empty vec if `target` is unreachable from `source`.
fn reconstruct_path(pred: &[usize], source: usize, target: usize) -> Vec<usize> {
    let mut path = Vec::new();
    let mut current = target;
    loop {
        path.push(current);
        if current == source {
            break;
        }
        let p = match pred.get(current) {
            Some(&p) => p,
            None => break,
        };
        if p == usize::MAX {
            // Unreachable.
            return vec![];
        }
        current = p;
    }
    path.reverse();
    path
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Helpers ───────────────────────────────────────────────────────────

    /// Build a 3-node, 1-edge hypergraph: {A,B,C} in one hyperedge.
    fn triangle_hg() -> Hypergraph {
        let mut hg = Hypergraph::new(false);
        let i0 = hg.add_node(HyperNode {
            id: 0,
            label: "A".into(),
            weight: 1.0,
            metadata: None,
            position: None,
        });
        let i1 = hg.add_node(HyperNode {
            id: 1,
            label: "B".into(),
            weight: 1.0,
            metadata: None,
            position: None,
        });
        let i2 = hg.add_node(HyperNode {
            id: 2,
            label: "C".into(),
            weight: 1.0,
            metadata: None,
            position: None,
        });
        // We check ok() in the helper; failures here are programmer errors in tests.
        if hg
            .add_edge(HyperEdge {
                id: 0,
                nodes: vec![i0, i1, i2],
                weight: 1.0,
                label: "e0".into(),
            })
            .is_err()
        {
            // Triangle helper failed — tests will catch this via assertions.
        }
        hg
    }

    /// Hypergraph with two disconnected components: {A,B} and {C,D}.
    fn two_component_hg() -> Hypergraph {
        let mut hg = Hypergraph::new(false);
        let i0 = hg.add_node(HyperNode {
            id: 0,
            label: "A".into(),
            weight: 1.0,
            metadata: None,
            position: None,
        });
        let i1 = hg.add_node(HyperNode {
            id: 1,
            label: "B".into(),
            weight: 1.0,
            metadata: None,
            position: None,
        });
        let i2 = hg.add_node(HyperNode {
            id: 2,
            label: "C".into(),
            weight: 1.0,
            metadata: None,
            position: None,
        });
        let i3 = hg.add_node(HyperNode {
            id: 3,
            label: "D".into(),
            weight: 1.0,
            metadata: None,
            position: None,
        });
        if hg
            .add_edge(HyperEdge {
                id: 0,
                nodes: vec![i0, i1],
                weight: 1.0,
                label: "e-ab".into(),
            })
            .is_err()
        {}
        if hg
            .add_edge(HyperEdge {
                id: 1,
                nodes: vec![i2, i3],
                weight: 1.0,
                label: "e-cd".into(),
            })
            .is_err()
        {}
        hg
    }

    // ── Empty hypergraph operations ───────────────────────────────────────

    #[test]
    fn empty_hypergraph_node_count_is_zero() {
        let hg = Hypergraph::new(false);
        assert_eq!(hg.node_count(), 0);
        assert_eq!(hg.edge_count(), 0);
    }

    #[test]
    fn empty_hypergraph_connected_components_is_empty() {
        let hg = Hypergraph::new(false);
        assert!(connected_components(&hg).is_empty());
    }

    #[test]
    fn empty_hypergraph_metrics() {
        let hg = Hypergraph::new(false);
        let m = compute_metrics(&hg);
        assert_eq!(m.node_count, 0);
        assert_eq!(m.edge_count, 0);
        assert_eq!(m.avg_edge_size, 0.0);
        assert_eq!(m.max_edge_size, 0);
        assert_eq!(m.density, 0.0);
        assert_eq!(m.connected_components, 0);
    }

    #[test]
    fn empty_hypergraph_detect_bipartite_true() {
        let hg = Hypergraph::new(false);
        assert!(detect_bipartite(&hg));
    }

    // ── Add nodes and edges ───────────────────────────────────────────────

    #[test]
    fn add_nodes_increments_count() {
        let mut hg = Hypergraph::new(false);
        let i0 = hg.add_node(HyperNode {
            id: 10,
            label: "X".into(),
            weight: 2.5,
            metadata: Some("meta".into()),
            position: Some([1.0, 2.0, 3.0]),
        });
        assert_eq!(i0, 0);
        assert_eq!(hg.node_count(), 1);

        let i1 = hg.add_node(HyperNode {
            id: 11,
            label: "Y".into(),
            weight: 0.5,
            metadata: None,
            position: None,
        });
        assert_eq!(i1, 1);
        assert_eq!(hg.node_count(), 2);
    }

    #[test]
    fn add_edge_too_few_nodes_returns_error() {
        let mut hg = Hypergraph::new(false);
        let i0 = hg.add_node(HyperNode {
            id: 0,
            label: "A".into(),
            weight: 1.0,
            metadata: None,
            position: None,
        });
        let result = hg.add_edge(HyperEdge {
            id: 0,
            nodes: vec![i0],
            weight: 1.0,
            label: "bad".into(),
        });
        assert!(matches!(result, Err(HypergraphError::InvalidEdge(_))));
    }

    #[test]
    fn add_edge_out_of_bounds_node_returns_error() {
        let mut hg = Hypergraph::new(false);
        hg.add_node(HyperNode {
            id: 0,
            label: "A".into(),
            weight: 1.0,
            metadata: None,
            position: None,
        });
        let result = hg.add_edge(HyperEdge {
            id: 0,
            nodes: vec![0, 99],
            weight: 1.0,
            label: "bad".into(),
        });
        assert!(matches!(result, Err(HypergraphError::InvalidEdge(_))));
    }

    #[test]
    fn add_valid_edge_succeeds() {
        let hg = triangle_hg();
        assert_eq!(hg.edge_count(), 1);
    }

    // ── Incidence matrix ──────────────────────────────────────────────────

    #[test]
    fn incidence_matrix_shape() {
        let hg = triangle_hg();
        let mat = build_incidence_matrix(&hg);
        assert_eq!(mat.rows, 3);
        assert_eq!(mat.cols, 1);
    }

    #[test]
    fn incidence_matrix_all_ones_for_full_edge() {
        let hg = triangle_hg(); // one edge containing all 3 nodes
        let mat = build_incidence_matrix(&hg);
        for i in 0..3 {
            assert_eq!(mat.data[i][0], 1.0, "node {i} should be in edge 0");
        }
    }

    #[test]
    fn incidence_matrix_partial_membership() {
        let mut hg = Hypergraph::new(false);
        let i0 = hg.add_node(HyperNode {
            id: 0,
            label: "A".into(),
            weight: 1.0,
            metadata: None,
            position: None,
        });
        let i1 = hg.add_node(HyperNode {
            id: 1,
            label: "B".into(),
            weight: 1.0,
            metadata: None,
            position: None,
        });
        let _i2 = hg.add_node(HyperNode {
            id: 2,
            label: "C".into(),
            weight: 1.0,
            metadata: None,
            position: None,
        });
        // Only i0 and i1 in edge; i2 is isolated.
        assert!(
            hg.add_edge(HyperEdge {
                id: 0,
                nodes: vec![i0, i1],
                weight: 1.0,
                label: "e".into(),
            })
            .is_ok()
        );
        let mat = build_incidence_matrix(&hg);
        assert_eq!(mat.data[0][0], 1.0);
        assert_eq!(mat.data[1][0], 1.0);
        assert_eq!(mat.data[2][0], 0.0);
    }

    // ── Adjacency matrix from hypergraph ──────────────────────────────────

    #[test]
    fn adjacency_matrix_from_hypergraph_symmetric() {
        let hg = triangle_hg();
        let adj = build_adjacency_from_hypergraph(&hg);
        let n = adj.len();
        for i in 0..n {
            for j in 0..n {
                assert!(
                    (adj[i][j] - adj[j][i]).abs() < f64::EPSILON,
                    "adjacency must be symmetric at [{i}][{j}]"
                );
            }
        }
    }

    #[test]
    fn adjacency_matrix_from_hypergraph_diagonal_zero() {
        let hg = triangle_hg();
        let adj = build_adjacency_from_hypergraph(&hg);
        for (i, row) in adj.iter().enumerate() {
            assert_eq!(row[i], 0.0, "diagonal must be zero at [{i}]");
        }
    }

    #[test]
    fn adjacency_matrix_weight_accumulated() {
        // Two hyperedges sharing the same pair (0, 1): weights should accumulate.
        let mut hg = Hypergraph::new(false);
        let i0 = hg.add_node(HyperNode {
            id: 0,
            label: "A".into(),
            weight: 1.0,
            metadata: None,
            position: None,
        });
        let i1 = hg.add_node(HyperNode {
            id: 1,
            label: "B".into(),
            weight: 1.0,
            metadata: None,
            position: None,
        });
        assert!(
            hg.add_edge(HyperEdge {
                id: 0,
                nodes: vec![i0, i1],
                weight: 1.5,
                label: "e0".into(),
            })
            .is_ok()
        );
        assert!(
            hg.add_edge(HyperEdge {
                id: 1,
                nodes: vec![i0, i1],
                weight: 0.5,
                label: "e1".into(),
            })
            .is_ok()
        );
        let adj = build_adjacency_from_hypergraph(&hg);
        assert!((adj[0][1] - 2.0).abs() < f64::EPSILON);
    }

    // ── Dual graph construction ───────────────────────────────────────────

    #[test]
    fn dual_graph_edge_count_for_sharing_edges() {
        let mut hg = Hypergraph::new(false);
        let i0 = hg.add_node(HyperNode {
            id: 0,
            label: "A".into(),
            weight: 1.0,
            metadata: None,
            position: None,
        });
        let i1 = hg.add_node(HyperNode {
            id: 1,
            label: "B".into(),
            weight: 1.0,
            metadata: None,
            position: None,
        });
        let i2 = hg.add_node(HyperNode {
            id: 2,
            label: "C".into(),
            weight: 1.0,
            metadata: None,
            position: None,
        });
        assert!(
            hg.add_edge(HyperEdge {
                id: 0,
                nodes: vec![i0, i1],
                weight: 1.0,
                label: "e0".into(),
            })
            .is_ok()
        );
        assert!(
            hg.add_edge(HyperEdge {
                id: 1,
                nodes: vec![i1, i2],
                weight: 1.0,
                label: "e1".into(),
            })
            .is_ok()
        );
        let dual = compute_dual(&hg).ok().unwrap_or(DualGraph {
            nodes: vec![],
            edges: vec![],
        });
        assert_eq!(dual.nodes.len(), 2);
        assert_eq!(dual.edges.len(), 1);
    }

    #[test]
    fn dual_graph_disjoint_edges_have_no_dual_edge() {
        // Two edges sharing no nodes -> dual has 0 edges.
        let hg = two_component_hg();
        let dual = compute_dual(&hg).ok().unwrap_or(DualGraph {
            nodes: vec![],
            edges: vec![],
        });
        assert_eq!(dual.nodes.len(), 2);
        assert_eq!(dual.edges.len(), 0);
    }

    #[test]
    fn dual_graph_empty_edges_returns_error() {
        let mut hg = Hypergraph::new(false);
        hg.add_node(HyperNode {
            id: 0,
            label: "A".into(),
            weight: 1.0,
            metadata: None,
            position: None,
        });
        let result = compute_dual(&hg);
        assert!(matches!(result, Err(HypergraphError::EmptyHypergraph)));
    }

    // ── Steiner tree ──────────────────────────────────────────────────────

    #[test]
    fn steiner_tree_terminals_in_result() {
        let hg = triangle_hg();
        let st = approximate_steiner_tree(&hg, &[0, 2])
            .ok()
            .unwrap_or(SteinerTree {
                nodes: vec![],
                edges: vec![],
                total_weight: 0.0,
            });
        assert!(st.nodes.contains(&0), "terminal 0 must be in Steiner tree");
        assert!(st.nodes.contains(&2), "terminal 2 must be in Steiner tree");
    }

    #[test]
    fn steiner_tree_invalid_terminal_returns_error() {
        let hg = triangle_hg();
        let result = approximate_steiner_tree(&hg, &[0, 99]);
        assert!(matches!(result, Err(HypergraphError::InvalidNode(99))));
    }

    #[test]
    fn steiner_tree_too_few_terminals_returns_error() {
        let hg = triangle_hg();
        let result = approximate_steiner_tree(&hg, &[0]);
        assert!(matches!(result, Err(HypergraphError::InvalidEdge(_))));
    }

    #[test]
    fn steiner_tree_empty_hypergraph_returns_error() {
        let hg = Hypergraph::new(false);
        let result = approximate_steiner_tree(&hg, &[0, 1]);
        assert!(matches!(result, Err(HypergraphError::EmptyHypergraph)));
    }

    // ── Force-directed layout ─────────────────────────────────────────────

    #[test]
    fn force_directed_layout_correct_node_count() {
        let hg = triangle_hg();
        let layout = force_directed_layout(&hg, 20);
        assert_eq!(layout.positions.len(), 3);
    }

    #[test]
    fn force_directed_layout_empty_hypergraph() {
        let hg = Hypergraph::new(false);
        let layout = force_directed_layout(&hg, 50);
        assert!(layout.positions.is_empty());
    }

    #[test]
    fn force_directed_layout_positions_are_finite() {
        let hg = triangle_hg();
        let layout = force_directed_layout(&hg, 50);
        for (i, pos) in layout.positions.iter().enumerate() {
            assert!(
                pos[0].is_finite() && pos[1].is_finite(),
                "position {i} contains non-finite values: {pos:?}"
            );
        }
    }

    // ── Connected components ──────────────────────────────────────────────

    #[test]
    fn connected_components_one_component() {
        let hg = triangle_hg();
        let comps = connected_components(&hg);
        assert_eq!(comps.len(), 1);
        assert_eq!(comps[0].len(), 3);
    }

    #[test]
    fn connected_components_two_components() {
        let hg = two_component_hg();
        let comps = connected_components(&hg);
        assert_eq!(comps.len(), 2);
        let total: usize = comps.iter().map(|c| c.len()).sum();
        assert_eq!(total, 4);
    }

    #[test]
    fn connected_components_isolated_nodes() {
        let mut hg = Hypergraph::new(false);
        for i in 0..3_usize {
            hg.add_node(HyperNode {
                id: i,
                label: format!("N{i}"),
                weight: 1.0,
                metadata: None,
                position: None,
            });
        }
        // No edges — every node is its own component.
        let comps = connected_components(&hg);
        assert_eq!(comps.len(), 3);
    }

    // ── Metrics computation ───────────────────────────────────────────────

    #[test]
    fn metrics_avg_edge_size_single_edge() {
        let hg = triangle_hg(); // 1 edge of size 3
        let m = compute_metrics(&hg);
        assert_eq!(m.avg_edge_size, 3.0);
        assert_eq!(m.max_edge_size, 3);
    }

    #[test]
    fn metrics_density_formula() {
        let hg = two_component_hg(); // 4 nodes, 2 edges
        let m = compute_metrics(&hg);
        // pairs = 4*3/2 = 6, density = 2/6
        let expected = 2.0 / 6.0;
        assert!((m.density - expected).abs() < 1e-10);
    }

    #[test]
    fn metrics_connected_components_count() {
        let hg = two_component_hg();
        let m = compute_metrics(&hg);
        assert_eq!(m.connected_components, 2);
    }

    // ── Degree calculation ────────────────────────────────────────────────

    #[test]
    fn degree_node_in_all_edges() {
        let mut hg = Hypergraph::new(false);
        let i0 = hg.add_node(HyperNode {
            id: 0,
            label: "hub".into(),
            weight: 1.0,
            metadata: None,
            position: None,
        });
        let i1 = hg.add_node(HyperNode {
            id: 1,
            label: "A".into(),
            weight: 1.0,
            metadata: None,
            position: None,
        });
        let i2 = hg.add_node(HyperNode {
            id: 2,
            label: "B".into(),
            weight: 1.0,
            metadata: None,
            position: None,
        });
        assert!(
            hg.add_edge(HyperEdge {
                id: 0,
                nodes: vec![i0, i1],
                weight: 1.0,
                label: "e0".into(),
            })
            .is_ok()
        );
        assert!(
            hg.add_edge(HyperEdge {
                id: 1,
                nodes: vec![i0, i2],
                weight: 1.0,
                label: "e1".into(),
            })
            .is_ok()
        );
        assert_eq!(hg.degree(i0).ok(), Some(2));
        assert_eq!(hg.degree(i1).ok(), Some(1));
    }

    #[test]
    fn degree_invalid_node_returns_error() {
        let hg = triangle_hg();
        let result = hg.degree(100);
        assert!(matches!(result, Err(HypergraphError::InvalidNode(100))));
    }

    // ── Neighbors function ────────────────────────────────────────────────

    #[test]
    fn neighbors_returns_correct_set() {
        let hg = triangle_hg(); // all three in one edge
        let mut nbrs = hg.neighbors(0).ok().unwrap_or_default();
        nbrs.sort_unstable();
        assert_eq!(nbrs, vec![1, 2]);
    }

    #[test]
    fn neighbors_isolated_node_is_empty() {
        let mut hg = Hypergraph::new(false);
        let i0 = hg.add_node(HyperNode {
            id: 0,
            label: "alone".into(),
            weight: 1.0,
            metadata: None,
            position: None,
        });
        let nbrs = hg.neighbors(i0).ok().unwrap_or_default();
        assert!(nbrs.is_empty());
    }

    #[test]
    fn neighbors_invalid_node_returns_error() {
        let hg = triangle_hg();
        assert!(matches!(
            hg.neighbors(99),
            Err(HypergraphError::InvalidNode(99))
        ));
    }

    // ── Error cases ───────────────────────────────────────────────────────

    #[test]
    fn error_display_invalid_node() {
        let e = HypergraphError::InvalidNode(42);
        let s = e.to_string();
        assert!(s.contains("42"));
    }

    #[test]
    fn error_display_invalid_edge() {
        let e = HypergraphError::InvalidEdge("test".into());
        let s = e.to_string();
        assert!(s.contains("test"));
    }

    #[test]
    fn error_display_empty_hypergraph() {
        let e = HypergraphError::EmptyHypergraph;
        let s = e.to_string();
        assert!(!s.is_empty());
    }

    #[test]
    fn error_display_cycle_detected() {
        let e = HypergraphError::CycleDetected;
        let s = e.to_string();
        assert!(!s.is_empty());
    }
}
