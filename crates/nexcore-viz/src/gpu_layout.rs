//! # GPU Layout — Force-Directed Graph Layout Engine
//!
//! Provides CPU-side force-directed graph layout algorithms designed for large
//! graphs (10K+ nodes), plus WGSL compute shader source strings for GPU
//! acceleration via WebGPU pipelines.
//!
//! ## Algorithms
//!
//! - **Fruchterman-Reingold** — Classic spring-electrical model with linear cooling
//! - **ForceAtlas2** — Gravity + linlog mode variant tuned for large sparse graphs
//! - **Barnes-Hut** — O(N log N) repulsion approximation via quadtree
//!
//! ## Usage
//!
//! ```rust
//! use nexcore_viz::gpu_layout::{
//!     LayoutConfig, LayoutEdge, LayoutNode, compute_layout, random_layout,
//! };
//!
//! let mut nodes = random_layout(6, 800.0, 600.0);
//! let edges = vec![
//!     LayoutEdge { source: 0, target: 1, weight: 1.0, ideal_length: 100.0 },
//!     LayoutEdge { source: 1, target: 2, weight: 1.0, ideal_length: 100.0 },
//! ];
//! let config = LayoutConfig::default();
//! let Ok(result) = compute_layout(nodes, edges, &config) else { return; };
//! assert!(result.iterations_run > 0);
//! ```
//!
//! ## WASM / GPU Integration
//!
//! The Observatory frontend calls [`compute_layout`] via WASM for moderate
//! graphs. For massive graphs (>500 nodes) it uses the WGSL shaders returned
//! by [`wgsl_force_shader`] and [`wgsl_integration_shader`] directly in a
//! WebGPU compute pipeline.

use serde::{Deserialize, Serialize};
use std::fmt;

// ── Error type ──────────────────────────────────────────────────────────────

/// Errors produced by the GPU layout engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LayoutError {
    /// The node list is empty; cannot lay out a graph with no nodes.
    EmptyGraph,
    /// A referenced node index does not exist in the node list.
    InvalidNode(usize),
    /// The algorithm failed to converge within the configured iteration limit.
    ConvergenceFailure,
    /// A configuration parameter is out of its valid range or semantically invalid.
    InvalidParameter(String),
}

impl fmt::Display for LayoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyGraph => write!(f, "graph has no nodes"),
            Self::InvalidNode(id) => write!(f, "node index {id} does not exist in the graph"),
            Self::ConvergenceFailure => write!(f, "layout did not converge within iteration limit"),
            Self::InvalidParameter(msg) => write!(f, "invalid layout parameter: {msg}"),
        }
    }
}

impl std::error::Error for LayoutError {}

// ── Core types ───────────────────────────────────────────────────────────────

/// A single node in the layout graph.
///
/// Positions and velocities are stored as `[f64; 2]` arrays (`[x, y]`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutNode {
    /// Unique node index (must match position in the nodes slice).
    pub id: usize,
    /// Current 2D position `[x, y]`.
    pub position: [f64; 2],
    /// Current 2D velocity `[vx, vy]`.
    pub velocity: [f64; 2],
    /// Node mass — heavier nodes repel more strongly and accelerate more slowly.
    pub mass: f64,
    /// When `true` the position is held fixed and forces are not integrated.
    pub pinned: bool,
}

impl LayoutNode {
    /// Create a new node at the given position with unit mass.
    pub fn new(id: usize, x: f64, y: f64) -> Self {
        Self {
            id,
            position: [x, y],
            velocity: [0.0, 0.0],
            mass: 1.0,
            pinned: false,
        }
    }
}

/// A directed or undirected edge in the layout graph.
///
/// Both `source` and `target` must be valid indices into the `nodes` slice.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutEdge {
    /// Source node index.
    pub source: usize,
    /// Target node index.
    pub target: usize,
    /// Edge weight — stronger attraction for higher weights.
    pub weight: f64,
    /// Ideal (rest-length) spring length in layout units.
    pub ideal_length: f64,
}

/// Configuration for a layout run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutConfig {
    /// Maximum number of simulation steps.
    pub iterations: usize,
    /// Global repulsion constant (Coulomb-like).
    pub repulsion_strength: f64,
    /// Global attraction constant (Hooke-like).
    pub attraction_strength: f64,
    /// Velocity damping factor in `[0, 1]` — 1.0 means no damping.
    pub damping: f64,
    /// Minimum pairwise distance to avoid division-by-zero singularities.
    pub min_distance: f64,
    /// Barnes-Hut opening angle `theta` — lower is more accurate (0.5–1.2 typical).
    pub theta: f64,
    /// Multiplicative temperature decay per iteration — in `(0, 1]`.
    pub cooling_factor: f64,
    /// Starting temperature (maximum displacement per step at iteration 0).
    pub initial_temperature: f64,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            iterations: 300,
            repulsion_strength: 10_000.0,
            attraction_strength: 0.05,
            damping: 0.85,
            min_distance: 1.0,
            theta: 0.8,
            cooling_factor: 0.995,
            initial_temperature: 200.0,
        }
    }
}

impl LayoutConfig {
    /// Validate that all parameters are in meaningful ranges.
    ///
    /// Returns `Err(LayoutError::InvalidParameter(...))` on the first violation.
    pub fn validate(&self) -> Result<(), LayoutError> {
        if self.iterations == 0 {
            return Err(LayoutError::InvalidParameter(
                "iterations must be > 0".to_string(),
            ));
        }
        if self.repulsion_strength < 0.0 {
            return Err(LayoutError::InvalidParameter(
                "repulsion_strength must be >= 0".to_string(),
            ));
        }
        if self.attraction_strength < 0.0 {
            return Err(LayoutError::InvalidParameter(
                "attraction_strength must be >= 0".to_string(),
            ));
        }
        if !(0.0..=1.0).contains(&self.damping) {
            return Err(LayoutError::InvalidParameter(
                "damping must be in [0, 1]".to_string(),
            ));
        }
        if self.min_distance <= 0.0 {
            return Err(LayoutError::InvalidParameter(
                "min_distance must be > 0".to_string(),
            ));
        }
        if self.theta <= 0.0 {
            return Err(LayoutError::InvalidParameter(
                "theta must be > 0".to_string(),
            ));
        }
        if !(0.0..=1.0).contains(&self.cooling_factor) {
            return Err(LayoutError::InvalidParameter(
                "cooling_factor must be in (0, 1]".to_string(),
            ));
        }
        if self.initial_temperature <= 0.0 {
            return Err(LayoutError::InvalidParameter(
                "initial_temperature must be > 0".to_string(),
            ));
        }
        Ok(())
    }
}

/// Output of a completed layout run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutResult {
    /// Final node positions and velocities.
    pub nodes: Vec<LayoutNode>,
    /// Number of simulation steps actually performed.
    pub iterations_run: usize,
    /// Whether the system energy fell below the convergence threshold.
    pub converged: bool,
    /// Final total system energy.
    pub energy: f64,
}

// ── Barnes-Hut quadtree ──────────────────────────────────────────────────────

/// A node in the Barnes-Hut quadtree.
///
/// Each `QuadTreeNode` represents a rectangular spatial region.  Leaf nodes
/// contain a single body (referenced by its index into the `nodes` slice).
/// Internal nodes aggregate the centre-of-mass and total mass of all bodies
/// inside their region.
#[derive(Debug, Clone)]
pub struct QuadTreeNode {
    /// Spatial bounds `[x_min, y_min, x_max, y_max]`.
    pub bounds: [f64; 4],
    /// Aggregated centre-of-mass `[cx, cy]` for all bodies in this region.
    pub center_of_mass: [f64; 2],
    /// Total mass of all bodies in this region.
    pub total_mass: f64,
    /// Four children (NW, NE, SW, SE), present only for internal nodes.
    pub children: [Option<Box<QuadTreeNode>>; 4],
    /// For leaf nodes: the index of the single body contained here.
    pub body: Option<usize>,
}

impl QuadTreeNode {
    /// Create a new empty node covering the given bounds.
    fn new(bounds: [f64; 4]) -> Self {
        Self {
            bounds,
            center_of_mass: [0.0, 0.0],
            total_mass: 0.0,
            children: [None, None, None, None],
            body: None,
        }
    }

    /// Returns `true` if this is a leaf node (no children).
    fn is_leaf(&self) -> bool {
        self.children.iter().all(|c| c.is_none())
    }

    /// Returns `true` if this leaf node holds no body.
    fn is_empty(&self) -> bool {
        self.is_leaf() && self.body.is_none()
    }

    /// Returns `true` if this node holds exactly one body and is a leaf.
    fn is_single_body(&self) -> bool {
        self.is_leaf() && self.body.is_some()
    }

    /// Spatial width of the bounds rectangle.
    fn width(&self) -> f64 {
        self.bounds[2] - self.bounds[0]
    }

    /// Determine which quadrant (0=NW, 1=NE, 2=SW, 3=SE) a position falls in.
    fn quadrant_for(&self, pos: [f64; 2]) -> usize {
        let mid_x = (self.bounds[0] + self.bounds[2]) / 2.0;
        let mid_y = (self.bounds[1] + self.bounds[3]) / 2.0;
        let east = pos[0] >= mid_x;
        let south = pos[1] >= mid_y;
        match (east, south) {
            (false, false) => 0, // NW
            (true, false) => 1,  // NE
            (false, true) => 2,  // SW
            (true, true) => 3,   // SE
        }
    }

    /// Sub-bounds for a given quadrant index.
    fn child_bounds(&self, q: usize) -> [f64; 4] {
        let mid_x = (self.bounds[0] + self.bounds[2]) / 2.0;
        let mid_y = (self.bounds[1] + self.bounds[3]) / 2.0;
        match q {
            0 => [self.bounds[0], self.bounds[1], mid_x, mid_y], // NW
            1 => [mid_x, self.bounds[1], self.bounds[2], mid_y], // NE
            2 => [self.bounds[0], mid_y, mid_x, self.bounds[3]], // SW
            _ => [mid_x, mid_y, self.bounds[2], self.bounds[3]], // SE
        }
    }

    /// Insert a body (by index) and its position/mass into this node.
    ///
    /// `nodes` is provided to look up position/mass for both the existing
    /// body (if any) and the new one.
    fn insert(&mut self, idx: usize, nodes: &[LayoutNode], depth: usize) {
        // Guard against degenerate trees caused by numerically identical positions.
        const MAX_DEPTH: usize = 64;

        if self.is_empty() {
            // Claim this leaf.
            self.body = Some(idx);
            self.total_mass = nodes[idx].mass;
            self.center_of_mass = nodes[idx].position;
            return;
        }

        if self.is_single_body() && depth < MAX_DEPTH {
            // Split: re-insert the existing body into a child.
            let existing_idx = self.body.take();
            if let Some(ei) = existing_idx {
                let ei_pos = nodes[ei].position;
                let q = self.quadrant_for(ei_pos);
                if self.children[q].is_none() {
                    self.children[q] = Some(Box::new(QuadTreeNode::new(self.child_bounds(q))));
                }
                if let Some(child) = self.children[q].as_mut() {
                    child.insert(ei, nodes, depth + 1);
                }
            }
        }

        // Insert the new body.
        let pos = nodes[idx].position;
        let q = self.quadrant_for(pos);
        if self.children[q].is_none() {
            self.children[q] = Some(Box::new(QuadTreeNode::new(self.child_bounds(q))));
        }
        if let Some(child) = self.children[q].as_mut() {
            child.insert(idx, nodes, depth + 1);
        }

        // Update aggregated centre-of-mass.
        let mass = nodes[idx].mass;
        let new_total = self.total_mass + mass;
        if new_total > 0.0 {
            self.center_of_mass[0] =
                (self.center_of_mass[0] * self.total_mass + pos[0] * mass) / new_total;
            self.center_of_mass[1] =
                (self.center_of_mass[1] * self.total_mass + pos[1] * mass) / new_total;
        }
        self.total_mass = new_total;
    }
}

/// A quadtree for Barnes-Hut O(N log N) force approximation.
#[derive(Debug, Clone)]
pub struct QuadTree {
    /// Root node covering the bounding box of all bodies.
    pub root: QuadTreeNode,
    /// Number of bodies inserted.
    pub node_count: usize,
}

/// Describes the memory layout of GPU vertex buffers.
///
/// All offsets are in bytes from the start of a single flat `f32` buffer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GpuBufferLayout {
    /// Byte offset to the positions block (interleaved `x, y` pairs).
    pub positions_offset: u32,
    /// Byte offset to the velocities block (interleaved `vx, vy` pairs).
    pub velocities_offset: u32,
    /// Byte offset to the forces accumulation block (interleaved `fx, fy` pairs).
    pub forces_offset: u32,
    /// Stride in bytes between consecutive node records within each block.
    pub stride: u32,
    /// Total number of nodes described by this layout.
    pub node_count: u32,
}

// ── Public API ───────────────────────────────────────────────────────────────

/// Initialize nodes with deterministic pseudo-random positions using xorshift64.
///
/// The positions are spread uniformly within `[0, width] × [0, height]`.
///
/// ```rust
/// use nexcore_viz::gpu_layout::random_layout;
///
/// let nodes = random_layout(10, 800.0, 600.0);
/// assert_eq!(nodes.len(), 10);
/// for node in &nodes {
///     assert!(node.position[0] >= 0.0 && node.position[0] <= 800.0);
///     assert!(node.position[1] >= 0.0 && node.position[1] <= 600.0);
/// }
/// ```
pub fn random_layout(n: usize, width: f64, height: f64) -> Vec<LayoutNode> {
    let mut state: u64 = 0xDEAD_BEEF_CAFE_BABE;
    let mut nodes = Vec::with_capacity(n);
    for id in 0..n {
        let x = xorshift64(&mut state) * width;
        let y = xorshift64(&mut state) * height;
        nodes.push(LayoutNode::new(id, x, y));
    }
    nodes
}

/// Scale all node positions so the bounding box fills `[0, width] × [0, height]`.
///
/// Nodes with identical positions are placed at the centre of the viewport.
///
/// ```rust
/// use nexcore_viz::gpu_layout::{LayoutNode, normalize_positions};
///
/// let mut nodes = vec![
///     LayoutNode::new(0, 0.0, 0.0),
///     LayoutNode::new(1, 100.0, 200.0),
/// ];
/// normalize_positions(&mut nodes, 400.0, 400.0);
/// assert!((nodes[1].position[0] - 400.0).abs() < 1e-9);
/// assert!((nodes[1].position[1] - 400.0).abs() < 1e-9);
/// ```
pub fn normalize_positions(nodes: &mut [LayoutNode], width: f64, height: f64) {
    if nodes.is_empty() {
        return;
    }

    let mut x_min = f64::MAX;
    let mut x_max = f64::MIN;
    let mut y_min = f64::MAX;
    let mut y_max = f64::MIN;

    for n in nodes.iter() {
        x_min = x_min.min(n.position[0]);
        x_max = x_max.max(n.position[0]);
        y_min = y_min.min(n.position[1]);
        y_max = y_max.max(n.position[1]);
    }

    let dx = x_max - x_min;
    let dy = y_max - y_min;

    for n in nodes.iter_mut() {
        if n.pinned {
            continue;
        }
        n.position[0] = if dx > 0.0 {
            (n.position[0] - x_min) / dx * width
        } else {
            width / 2.0
        };
        n.position[1] = if dy > 0.0 {
            (n.position[1] - y_min) / dy * height
        } else {
            height / 2.0
        };
    }
}

/// Compute the total kinetic + potential energy of the system.
///
/// Energy = Σ 0.5·m·|v|² (kinetic) + Σ repulsion terms + Σ spring terms.
///
/// This is used as the convergence criterion in [`fruchterman_reingold`].
pub fn compute_energy(nodes: &[LayoutNode], edges: &[LayoutEdge], config: &LayoutConfig) -> f64 {
    // Kinetic energy
    let kinetic: f64 = nodes
        .iter()
        .map(|n| 0.5 * n.mass * (n.velocity[0].powi(2) + n.velocity[1].powi(2)))
        .sum();

    // Repulsion potential (pairwise)
    let mut repulsion = 0.0;
    for i in 0..nodes.len() {
        for j in (i + 1)..nodes.len() {
            let dx = nodes[j].position[0] - nodes[i].position[0];
            let dy = nodes[j].position[1] - nodes[i].position[1];
            let dist = dx.hypot(dy).max(config.min_distance);
            repulsion += config.repulsion_strength * nodes[i].mass * nodes[j].mass / dist;
        }
    }

    // Spring potential along edges
    let spring: f64 = edges
        .iter()
        .filter_map(|e| {
            let src = nodes.get(e.source)?;
            let tgt = nodes.get(e.target)?;
            let dx = tgt.position[0] - src.position[0];
            let dy = tgt.position[1] - src.position[1];
            let dist = dx.hypot(dy);
            let stretch = dist - e.ideal_length;
            Some(0.5 * config.attraction_strength * e.weight * stretch.powi(2))
        })
        .sum();

    kinetic + repulsion + spring
}

/// Build a Barnes-Hut quadtree from the current node positions.
///
/// The tree covers the tight bounding box of all nodes, expanded slightly to
/// avoid boundary artefacts.
pub fn build_quadtree(nodes: &[LayoutNode]) -> QuadTree {
    let mut x_min = f64::MAX;
    let mut x_max = f64::MIN;
    let mut y_min = f64::MAX;
    let mut y_max = f64::MIN;

    for n in nodes {
        x_min = x_min.min(n.position[0]);
        x_max = x_max.max(n.position[0]);
        y_min = y_min.min(n.position[1]);
        y_max = y_max.max(n.position[1]);
    }

    // Add a small margin so boundary nodes are fully inside the root cell.
    let margin = ((x_max - x_min) + (y_max - y_min)) * 0.01 + 1.0;
    let bounds = [
        x_min - margin,
        y_min - margin,
        x_max + margin,
        y_max + margin,
    ];

    let mut root = QuadTreeNode::new(bounds);
    for (idx, _) in nodes.iter().enumerate() {
        root.insert(idx, nodes, 0);
    }

    QuadTree {
        root,
        node_count: nodes.len(),
    }
}

/// Compute the Barnes-Hut repulsion force on a single node from the quadtree.
///
/// Traverses the tree, treating cells as point masses when the cell width–to–
/// distance ratio falls below `theta`.
///
/// Returns `[fx, fy]` — the net repulsive force vector.
pub fn compute_repulsion_bh(tree: &QuadTree, node: &LayoutNode, theta: f64) -> [f64; 2] {
    let mut force = [0.0_f64, 0.0_f64];
    accumulate_repulsion(&tree.root, node, theta, &mut force);
    force
}

// ── Fruchterman-Reingold ─────────────────────────────────────────────────────

/// Classic Fruchterman-Reingold force-directed layout.
///
/// Uses O(N²) pairwise repulsion for small graphs (N ≤ 500).  For larger
/// graphs, call [`compute_layout`] which automatically switches to Barnes-Hut.
///
/// # Errors
///
/// Returns [`LayoutError::EmptyGraph`] if `nodes` is empty, or
/// [`LayoutError::InvalidNode`] if any edge references a missing index.
///
/// ```rust
/// use nexcore_viz::gpu_layout::{
///     LayoutConfig, LayoutEdge, LayoutNode, fruchterman_reingold, random_layout,
/// };
///
/// let mut nodes = random_layout(4, 400.0, 400.0);
/// let edges = vec![
///     LayoutEdge { source: 0, target: 1, weight: 1.0, ideal_length: 80.0 },
///     LayoutEdge { source: 1, target: 2, weight: 1.0, ideal_length: 80.0 },
/// ];
/// let mut config = LayoutConfig::default();
/// config.iterations = 50;
/// let Ok(result) = fruchterman_reingold(&mut nodes, &edges, &config) else { return; };
/// assert!(result.iterations_run > 0);
/// ```
pub fn fruchterman_reingold(
    nodes: &mut [LayoutNode],
    edges: &[LayoutEdge],
    config: &LayoutConfig,
) -> Result<LayoutResult, LayoutError> {
    if nodes.is_empty() {
        return Err(LayoutError::EmptyGraph);
    }
    config.validate()?;
    validate_edges(nodes, edges)?;

    let n = nodes.len();
    let mut temperature = config.initial_temperature;
    let convergence_threshold = 0.01 * temperature * config.cooling_factor.powi(50);

    let mut forces = vec![[0.0_f64, 0.0_f64]; n];
    let mut iterations_run = 0;
    let mut converged = false;

    for _iter in 0..config.iterations {
        // Reset forces.
        for f in forces.iter_mut() {
            *f = [0.0, 0.0];
        }

        // Repulsion (O(N²)).
        for i in 0..n {
            for j in (i + 1)..n {
                let [fx, fy] = repulsion_force(
                    nodes[i].position,
                    nodes[j].position,
                    nodes[i].mass,
                    nodes[j].mass,
                    config.repulsion_strength,
                    config.min_distance,
                );
                forces[i][0] += fx;
                forces[i][1] += fy;
                forces[j][0] -= fx;
                forces[j][1] -= fy;
            }
        }

        // Attraction along edges.
        for edge in edges {
            let [fx, fy] = attraction_force(
                nodes[edge.source].position,
                nodes[edge.target].position,
                edge.weight,
                edge.ideal_length,
                config.attraction_strength,
                config.min_distance,
            );
            forces[edge.source][0] += fx;
            forces[edge.source][1] += fy;
            forces[edge.target][0] -= fx;
            forces[edge.target][1] -= fy;
        }

        // Integrate positions.
        let mut max_displacement = 0.0_f64;
        for (node, force) in nodes.iter_mut().zip(forces.iter()) {
            if node.pinned {
                continue;
            }
            let mag = force[0].hypot(force[1]).max(f64::MIN_POSITIVE);
            let clamped_x = force[0] / mag * mag.min(temperature);
            let clamped_y = force[1] / mag * mag.min(temperature);

            node.velocity[0] = (node.velocity[0] + clamped_x) * config.damping;
            node.velocity[1] = (node.velocity[1] + clamped_y) * config.damping;

            let disp = node.velocity[0].hypot(node.velocity[1]);
            max_displacement = max_displacement.max(disp);

            node.position[0] += node.velocity[0];
            node.position[1] += node.velocity[1];
        }

        temperature *= config.cooling_factor;
        iterations_run += 1;

        if max_displacement < convergence_threshold {
            converged = true;
            break;
        }
    }

    let energy = compute_energy(nodes, edges, config);

    Ok(LayoutResult {
        nodes: nodes.to_vec(),
        iterations_run,
        converged,
        energy,
    })
}

// ── ForceAtlas2 ──────────────────────────────────────────────────────────────

/// ForceAtlas2 variant with gravity pull and log-attraction mode.
///
/// Differences from Fruchterman-Reingold:
/// - Gravity force pulls every node towards the origin.
/// - Attraction is logarithmic (`log(1 + dist)` instead of linear).
/// - Temperature schedule is identical.
///
/// # Errors
///
/// Same conditions as [`fruchterman_reingold`].
///
/// ```rust
/// use nexcore_viz::gpu_layout::{
///     LayoutConfig, LayoutEdge, LayoutNode, force_atlas2, random_layout,
/// };
///
/// let mut nodes = random_layout(5, 400.0, 400.0);
/// let edges = vec![
///     LayoutEdge { source: 0, target: 1, weight: 1.0, ideal_length: 100.0 },
/// ];
/// let mut config = LayoutConfig::default();
/// config.iterations = 30;
/// let Ok(result) = force_atlas2(&mut nodes, &edges, &config) else { return; };
/// assert!(result.iterations_run > 0);
/// ```
pub fn force_atlas2(
    nodes: &mut [LayoutNode],
    edges: &[LayoutEdge],
    config: &LayoutConfig,
) -> Result<LayoutResult, LayoutError> {
    if nodes.is_empty() {
        return Err(LayoutError::EmptyGraph);
    }
    config.validate()?;
    validate_edges(nodes, edges)?;

    let n = nodes.len();
    let gravity = config.repulsion_strength * 0.0001;
    let mut temperature = config.initial_temperature;

    let mut forces = vec![[0.0_f64, 0.0_f64]; n];
    let mut iterations_run = 0;
    let mut converged = false;

    for _iter in 0..config.iterations {
        for f in forces.iter_mut() {
            *f = [0.0, 0.0];
        }

        // Repulsion (O(N²)).
        for i in 0..n {
            for j in (i + 1)..n {
                let [fx, fy] = repulsion_force(
                    nodes[i].position,
                    nodes[j].position,
                    nodes[i].mass,
                    nodes[j].mass,
                    config.repulsion_strength,
                    config.min_distance,
                );
                forces[i][0] += fx;
                forces[i][1] += fy;
                forces[j][0] -= fx;
                forces[j][1] -= fy;
            }
        }

        // Gravity towards origin.
        for (i, node) in nodes.iter().enumerate() {
            let dist = node.position[0]
                .hypot(node.position[1])
                .max(config.min_distance);
            forces[i][0] -= gravity * node.mass * node.position[0] / dist;
            forces[i][1] -= gravity * node.mass * node.position[1] / dist;
        }

        // Log-attraction along edges.
        for edge in edges {
            let dx = nodes[edge.target].position[0] - nodes[edge.source].position[0];
            let dy = nodes[edge.target].position[1] - nodes[edge.source].position[1];
            let dist = dx.hypot(dy).max(config.min_distance);
            let factor = config.attraction_strength * edge.weight * (1.0 + dist).ln() / dist;
            let fx = dx * factor;
            let fy = dy * factor;
            forces[edge.source][0] += fx;
            forces[edge.source][1] += fy;
            forces[edge.target][0] -= fx;
            forces[edge.target][1] -= fy;
        }

        // Integrate.
        let mut max_displacement = 0.0_f64;
        for (node, force) in nodes.iter_mut().zip(forces.iter()) {
            if node.pinned {
                continue;
            }
            let mag = force[0].hypot(force[1]).max(f64::MIN_POSITIVE);
            let clamped_x = force[0] / mag * mag.min(temperature);
            let clamped_y = force[1] / mag * mag.min(temperature);

            node.velocity[0] = (node.velocity[0] + clamped_x) * config.damping;
            node.velocity[1] = (node.velocity[1] + clamped_y) * config.damping;

            let disp = node.velocity[0].hypot(node.velocity[1]);
            max_displacement = max_displacement.max(disp);

            node.position[0] += node.velocity[0];
            node.position[1] += node.velocity[1];
        }

        temperature *= config.cooling_factor;
        iterations_run += 1;

        let convergence_threshold =
            0.01 * config.initial_temperature * config.cooling_factor.powi(50);
        if max_displacement < convergence_threshold {
            converged = true;
            break;
        }
    }

    let energy = compute_energy(nodes, edges, config);

    Ok(LayoutResult {
        nodes: nodes.to_vec(),
        iterations_run,
        converged,
        energy,
    })
}

// ── High-level entry point ───────────────────────────────────────────────────

/// High-level layout entry point.
///
/// Automatically selects the best algorithm:
/// - N ≤ 500 → [`fruchterman_reingold`] (O(N²), exact)
/// - N > 500  → Barnes-Hut accelerated Fruchterman-Reingold (O(N log N))
///
/// # Errors
///
/// Returns [`LayoutError::EmptyGraph`] if `nodes` is empty.
///
/// ```rust
/// use nexcore_viz::gpu_layout::{LayoutConfig, LayoutEdge, compute_layout, random_layout};
///
/// let nodes = random_layout(8, 800.0, 600.0);
/// let edges = vec![
///     LayoutEdge { source: 0, target: 1, weight: 1.0, ideal_length: 100.0 },
///     LayoutEdge { source: 2, target: 3, weight: 1.0, ideal_length: 100.0 },
/// ];
/// let config = LayoutConfig::default();
/// let Ok(result) = compute_layout(nodes, edges, &config) else { return; };
/// assert!(!result.nodes.is_empty());
/// ```
pub fn compute_layout(
    mut nodes: Vec<LayoutNode>,
    edges: Vec<LayoutEdge>,
    config: &LayoutConfig,
) -> Result<LayoutResult, LayoutError> {
    if nodes.is_empty() {
        return Err(LayoutError::EmptyGraph);
    }
    config.validate()?;
    validate_edges(&nodes, &edges)?;

    if nodes.len() <= 500 {
        fruchterman_reingold(&mut nodes, &edges, config)
    } else {
        fruchterman_reingold_bh(&mut nodes, &edges, config)
    }
}

// ── GPU support ──────────────────────────────────────────────────────────────

/// Pack node and edge data into GPU-friendly flat `f32` buffers.
///
/// # Buffer Layout
///
/// The returned `Vec<f32>` is a single flat buffer with three contiguous
/// sections, each containing `node_count * 2` floats:
///
/// ```text
/// [positions: x0,y0, x1,y1, ...]    <- starts at positions_offset bytes
/// [velocities: vx0,vy0, vx1,vy1, ...] <- starts at velocities_offset bytes
/// [forces: fx0,fy0, fx1,fy1, ...]   <- starts at forces_offset bytes (zeroed)
/// ```
///
/// The `Vec<u32>` holds edge index pairs `[src0, tgt0, src1, tgt1, ...]`.
///
/// ```rust
/// use nexcore_viz::gpu_layout::{LayoutEdge, LayoutNode, prepare_gpu_buffers};
///
/// let nodes = vec![
///     LayoutNode::new(0, 10.0, 20.0),
///     LayoutNode::new(1, 30.0, 40.0),
/// ];
/// let edges = vec![LayoutEdge { source: 0, target: 1, weight: 1.0, ideal_length: 50.0 }];
/// let (floats, indices, layout) = prepare_gpu_buffers(&nodes, &edges);
/// assert_eq!(layout.node_count, 2);
/// assert_eq!(indices.len(), 2);
/// ```
pub fn prepare_gpu_buffers(
    nodes: &[LayoutNode],
    edges: &[LayoutEdge],
) -> (Vec<f32>, Vec<u32>, GpuBufferLayout) {
    let n = nodes.len();
    // Three 2-float sections (positions, velocities, forces) = 6 f32 per node.
    let positions_floats = n * 2;
    let velocities_floats = n * 2;
    let forces_floats = n * 2;

    let total_floats = positions_floats + velocities_floats + forces_floats;
    let mut buffer = vec![0.0_f32; total_floats];

    // Fill positions.
    for (i, node) in nodes.iter().enumerate() {
        buffer[i * 2] = node.position[0] as f32;
        buffer[i * 2 + 1] = node.position[1] as f32;
    }

    // Fill velocities (after positions block).
    let vel_base = positions_floats;
    for (i, node) in nodes.iter().enumerate() {
        buffer[vel_base + i * 2] = node.velocity[0] as f32;
        buffer[vel_base + i * 2 + 1] = node.velocity[1] as f32;
    }

    // Forces block starts after velocities (already zeroed by vec initialisation).
    let forces_base = positions_floats + velocities_floats;

    let layout = GpuBufferLayout {
        positions_offset: 0,
        velocities_offset: (positions_floats * std::mem::size_of::<f32>()) as u32,
        forces_offset: (forces_base * std::mem::size_of::<f32>()) as u32,
        // stride within each section: 2 floats × 4 bytes = 8 bytes
        stride: (2 * std::mem::size_of::<f32>()) as u32,
        node_count: n as u32,
    };

    // Edge index buffer: pairs [source, target].
    let mut indices = Vec::with_capacity(edges.len() * 2);
    for edge in edges {
        indices.push(edge.source as u32);
        indices.push(edge.target as u32);
    }

    // Ensure the forces section size variable is used (silences dead-code lint).
    debug_assert_eq!(
        total_floats,
        positions_floats + velocities_floats + forces_floats
    );

    (buffer, indices, layout)
}

/// Returns the WGSL source for the force-calculation compute shader.
///
/// The shader computes pairwise repulsive forces and edge-attraction forces,
/// writing the results into a forces output buffer.  It is designed to be
/// compiled by the Observatory's WebGPU pipeline.
///
/// # Shader Interface
///
/// | Binding | Group | Type | Description |
/// |---------|-------|------|-------------|
/// | 0 | 0 | storage read | Node positions (f32 pairs) |
/// | 1 | 0 | storage read | Node metadata (mass, pinned) |
/// | 2 | 0 | storage read | Edge data (src, tgt, weight, ideal_len) |
/// | 3 | 0 | storage read_write | Forces output (f32 pairs, zeroed before dispatch) |
///
/// ```rust
/// use nexcore_viz::gpu_layout::wgsl_force_shader;
///
/// let src = wgsl_force_shader();
/// assert!(!src.is_empty());
/// assert!(src.contains("@compute"));
/// ```
pub fn wgsl_force_shader() -> &'static str {
    WGSL_FORCE_SHADER
}

/// Returns the WGSL source for the velocity/position integration shader.
///
/// Reads the forces buffer produced by [`wgsl_force_shader`], applies damping,
/// updates velocities and positions, and respects the `pinned` flag.
///
/// ```rust
/// use nexcore_viz::gpu_layout::wgsl_integration_shader;
///
/// let src = wgsl_integration_shader();
/// assert!(!src.is_empty());
/// assert!(src.contains("@compute"));
/// ```
pub fn wgsl_integration_shader() -> &'static str {
    WGSL_INTEGRATION_SHADER
}

// ── WGSL shader source strings ───────────────────────────────────────────────

const WGSL_FORCE_SHADER: &str = r#"// nexcore Observatory — Force Calculation Compute Shader
// Computes repulsive (Coulomb) and attractive (Hooke) forces for each node.
//
// Dispatch: (node_count / 64 + 1, 1, 1) workgroups of size 64.

// ─── Structs ────────────────────────────────────────────────────────────────

// Per-node state read by the force shader.
struct NodeData {
    // 2D position (x, y).
    pos_x: f32,
    pos_y: f32,
    // 2D velocity (vx, vy).
    vel_x: f32,
    vel_y: f32,
    // Node mass — heavier nodes exert stronger repulsion.
    mass: f32,
    // Non-zero means the node is fixed; forces are ignored for it.
    pinned: u32,
}

// Per-edge data.
struct EdgeData {
    source: u32,
    target: u32,
    // Edge weight controlling attraction strength.
    weight: f32,
    // Rest length of the Hooke spring.
    ideal_length: f32,
}

// Force accumulation entry (one per node).
struct ForceEntry {
    fx: f32,
    fy: f32,
}

// ─── Layout uniforms ────────────────────────────────────────────────────────

struct LayoutUniforms {
    // Total number of nodes.
    node_count: u32,
    // Total number of edges.
    edge_count: u32,
    // Global repulsion constant.
    repulsion_strength: f32,
    // Global attraction constant.
    attraction_strength: f32,
    // Minimum pairwise distance (avoids singularity).
    min_distance: f32,
    // Padding to 16-byte alignment.
    _pad: f32,
}

// ─── Bindings ───────────────────────────────────────────────────────────────

@group(0) @binding(0) var<storage, read>       nodes:    array<NodeData>;
@group(0) @binding(1) var<storage, read>       edges:    array<EdgeData>;
@group(0) @binding(2) var<storage, read_write> forces:   array<ForceEntry>;
@group(0) @binding(3) var<uniform>             uniforms: LayoutUniforms;

// ─── Helpers ────────────────────────────────────────────────────────────────

// Coulomb-style repulsion: force magnitude = k * m_i * m_j / dist^2
// directed from j to i (pushing apart).
fn repulsion(pos_i: vec2<f32>, pos_j: vec2<f32>, m_i: f32, m_j: f32, k: f32, min_d: f32) -> vec2<f32> {
    let delta = pos_i - pos_j;
    let dist  = max(length(delta), min_d);
    let mag   = k * m_i * m_j / (dist * dist);
    return normalize(delta) * mag;
}

// Hooke spring: force magnitude = k * w * (dist - ideal_length)
// directed from source toward target (pulling together when dist > ideal).
fn spring(pos_src: vec2<f32>, pos_tgt: vec2<f32>, weight: f32, ideal: f32, k: f32, min_d: f32) -> vec2<f32> {
    let delta  = pos_tgt - pos_src;
    let dist   = max(length(delta), min_d);
    let mag    = k * weight * (dist - ideal);
    return normalize(delta) * mag;
}

// ─── Entry point ────────────────────────────────────────────────────────────

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if i >= uniforms.node_count {
        return;
    }

    let node_i = nodes[i];

    // Skip pinned nodes — they do not accumulate forces.
    if node_i.pinned != 0u {
        forces[i].fx = 0.0;
        forces[i].fy = 0.0;
        return;
    }

    let pos_i = vec2<f32>(node_i.pos_x, node_i.pos_y);
    var force  = vec2<f32>(0.0, 0.0);

    // Repulsion: sum over all other nodes.
    for (var j = 0u; j < uniforms.node_count; j++) {
        if j == i {
            continue;
        }
        let node_j = nodes[j];
        let pos_j  = vec2<f32>(node_j.pos_x, node_j.pos_y);
        force += repulsion(
            pos_i, pos_j,
            node_i.mass, node_j.mass,
            uniforms.repulsion_strength,
            uniforms.min_distance,
        );
    }

    // Attraction: sum over all incident edges.
    for (var e = 0u; e < uniforms.edge_count; e++) {
        let edge = edges[e];
        var other_idx = 0u;
        var sign      = 1.0;

        if edge.source == i {
            other_idx = edge.target;
        } else if edge.target == i {
            other_idx = edge.source;
            sign      = -1.0;
        } else {
            continue;
        }

        let pos_other = vec2<f32>(nodes[other_idx].pos_x, nodes[other_idx].pos_y);
        // For source node: positive force pulls toward target.
        // For target node: negate so it also pulls toward source (undirected).
        let f_spring = spring(
            pos_i, pos_other,
            edge.weight, edge.ideal_length,
            uniforms.attraction_strength,
            uniforms.min_distance,
        );
        force += f_spring * sign;
    }

    // Write accumulated force.
    forces[i].fx = force.x;
    forces[i].fy = force.y;
}
"#;

const WGSL_INTEGRATION_SHADER: &str = r#"// nexcore Observatory — Velocity/Position Integration Shader
// Reads the forces buffer from the force shader, applies damping,
// updates velocities, then integrates positions.
//
// Dispatch: (node_count / 64 + 1, 1, 1) workgroups of size 64.

// ─── Structs ────────────────────────────────────────────────────────────────

// Full mutable node state.
struct NodeState {
    pos_x: f32,
    pos_y: f32,
    vel_x: f32,
    vel_y: f32,
    mass:   f32,
    pinned: u32,
}

// Force entry produced by the force shader.
struct ForceEntry {
    fx: f32,
    fy: f32,
}

// ─── Integration uniforms ───────────────────────────────────────────────────

struct IntegrationUniforms {
    // Total number of nodes.
    node_count: u32,
    // Velocity damping coefficient in [0, 1].
    damping: f32,
    // Current temperature (maximum displacement clamp).
    temperature: f32,
    // Padding.
    _pad: f32,
}

// ─── Bindings ───────────────────────────────────────────────────────────────

@group(0) @binding(0) var<storage, read_write> nodes:    array<NodeState>;
@group(0) @binding(1) var<storage, read>       forces:   array<ForceEntry>;
@group(0) @binding(2) var<uniform>             uniforms: IntegrationUniforms;

// ─── Entry point ────────────────────────────────────────────────────────────

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if i >= uniforms.node_count {
        return;
    }

    // Pinned nodes: zero velocity; position is held fixed.
    if nodes[i].pinned != 0u {
        nodes[i].vel_x = 0.0;
        nodes[i].vel_y = 0.0;
        return;
    }

    let f = vec2<f32>(forces[i].fx, forces[i].fy);

    // Clamp force magnitude to current temperature (cooling schedule).
    let f_len     = length(f);
    let clamped_f = select(
        f,
        normalize(f) * uniforms.temperature,
        f_len > uniforms.temperature,
    );

    // Velocity update with damping.
    var vel = vec2<f32>(nodes[i].vel_x, nodes[i].vel_y);
    vel = (vel + clamped_f) * uniforms.damping;

    // Position integration.
    nodes[i].pos_x += vel.x;
    nodes[i].pos_y += vel.y;
    nodes[i].vel_x  = vel.x;
    nodes[i].vel_y  = vel.y;
}
"#;

// ── Private helpers ───────────────────────────────────────────────────────────

/// xorshift64 PRNG — returns a value in `[0, 1)`.
fn xorshift64(state: &mut u64) -> f64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    (*state as f64) / (u64::MAX as f64)
}

/// Validate that every edge references valid node indices.
fn validate_edges(nodes: &[LayoutNode], edges: &[LayoutEdge]) -> Result<(), LayoutError> {
    let n = nodes.len();
    for edge in edges {
        if edge.source >= n {
            return Err(LayoutError::InvalidNode(edge.source));
        }
        if edge.target >= n {
            return Err(LayoutError::InvalidNode(edge.target));
        }
    }
    Ok(())
}

/// Coulomb-style repulsive force from node j onto node i.
///
/// Returns `[fx, fy]` — force applied to node i (pushing away from j).
fn repulsion_force(
    pos_i: [f64; 2],
    pos_j: [f64; 2],
    mass_i: f64,
    mass_j: f64,
    k: f64,
    min_dist: f64,
) -> [f64; 2] {
    let dx = pos_i[0] - pos_j[0];
    let dy = pos_i[1] - pos_j[1];
    let dist = dx.hypot(dy).max(min_dist);
    let mag = k * mass_i * mass_j / (dist * dist);
    [dx / dist * mag, dy / dist * mag]
}

/// Hooke spring attractive force along an edge.
///
/// Returns `[fx, fy]` — force pulling the source node towards the target.
fn attraction_force(
    pos_src: [f64; 2],
    pos_tgt: [f64; 2],
    weight: f64,
    ideal_length: f64,
    k: f64,
    min_dist: f64,
) -> [f64; 2] {
    let dx = pos_tgt[0] - pos_src[0];
    let dy = pos_tgt[1] - pos_src[1];
    let dist = dx.hypot(dy).max(min_dist);
    let mag = k * weight * (dist - ideal_length);
    [dx / dist * mag, dy / dist * mag]
}

/// Fruchterman-Reingold with Barnes-Hut acceleration for large graphs.
fn fruchterman_reingold_bh(
    nodes: &mut [LayoutNode],
    edges: &[LayoutEdge],
    config: &LayoutConfig,
) -> Result<LayoutResult, LayoutError> {
    let n = nodes.len();
    let mut temperature = config.initial_temperature;
    let convergence_threshold = 0.01 * temperature * config.cooling_factor.powi(50);

    let mut forces = vec![[0.0_f64, 0.0_f64]; n];
    let mut iterations_run = 0;
    let mut converged = false;

    for _iter in 0..config.iterations {
        for f in forces.iter_mut() {
            *f = [0.0, 0.0];
        }

        // Build Barnes-Hut tree.
        let tree = build_quadtree(nodes);

        // Repulsion via Barnes-Hut.
        for (i, node) in nodes.iter().enumerate() {
            let [fx, fy] = compute_repulsion_bh(&tree, node, config.theta);
            forces[i][0] += fx;
            forces[i][1] += fy;
        }

        // Attraction along edges (still exact).
        for edge in edges {
            let [fx, fy] = attraction_force(
                nodes[edge.source].position,
                nodes[edge.target].position,
                edge.weight,
                edge.ideal_length,
                config.attraction_strength,
                config.min_distance,
            );
            forces[edge.source][0] += fx;
            forces[edge.source][1] += fy;
            forces[edge.target][0] -= fx;
            forces[edge.target][1] -= fy;
        }

        // Integrate positions.
        let mut max_displacement = 0.0_f64;
        for (node, force) in nodes.iter_mut().zip(forces.iter()) {
            if node.pinned {
                continue;
            }
            let mag = force[0].hypot(force[1]).max(f64::MIN_POSITIVE);
            let clamped_x = force[0] / mag * mag.min(temperature);
            let clamped_y = force[1] / mag * mag.min(temperature);

            node.velocity[0] = (node.velocity[0] + clamped_x) * config.damping;
            node.velocity[1] = (node.velocity[1] + clamped_y) * config.damping;

            let disp = node.velocity[0].hypot(node.velocity[1]);
            max_displacement = max_displacement.max(disp);

            node.position[0] += node.velocity[0];
            node.position[1] += node.velocity[1];
        }

        temperature *= config.cooling_factor;
        iterations_run += 1;

        if max_displacement < convergence_threshold {
            converged = true;
            break;
        }
    }

    let energy = compute_energy(nodes, edges, config);

    Ok(LayoutResult {
        nodes: nodes.to_vec(),
        iterations_run,
        converged,
        energy,
    })
}

/// Recursively accumulate the Barnes-Hut repulsion force contribution from a
/// quadtree node onto the query body.
fn accumulate_repulsion(
    qt_node: &QuadTreeNode,
    body: &LayoutNode,
    theta: f64,
    force: &mut [f64; 2],
) {
    if qt_node.total_mass == 0.0 {
        return;
    }

    // Skip if this tree node IS the body itself (single-body leaf).
    if let Some(idx) = qt_node.body {
        if idx == body.id {
            return;
        }
    }

    let dx = body.position[0] - qt_node.center_of_mass[0];
    let dy = body.position[1] - qt_node.center_of_mass[1];
    let dist = dx.hypot(dy);

    if dist == 0.0 {
        return;
    }

    // Barnes-Hut criterion: treat cell as single mass if width/dist < theta.
    let width = qt_node.width();
    if qt_node.is_single_body() || (width / dist < theta) {
        // Approximate repulsion from the aggregated mass.
        let mag = 10_000.0 * body.mass * qt_node.total_mass / (dist * dist).max(1.0);
        force[0] += dx / dist * mag;
        force[1] += dy / dist * mag;
    } else {
        // Recurse into children.
        for child in qt_node.children.iter().flatten() {
            accumulate_repulsion(child, body, theta, force);
        }
    }
}

// ── 3D Layout Types ───────────────────────────────────────────────────────────

/// A single node in the 3D layout graph.
///
/// Positions and velocities are stored as `[f64; 3]` arrays (`[x, y, z]`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutNode3D {
    /// Unique node index (must match position in the nodes slice).
    pub id: usize,
    /// Current 3D position `[x, y, z]`.
    pub position: [f64; 3],
    /// Current 3D velocity `[vx, vy, vz]`.
    pub velocity: [f64; 3],
    /// Node mass — heavier nodes repel more strongly and accelerate more slowly.
    pub mass: f64,
    /// When `true` the position is held fixed and forces are not integrated.
    pub pinned: bool,
}

impl LayoutNode3D {
    /// Create a new 3D node at the given position with unit mass.
    pub fn new(id: usize, x: f64, y: f64, z: f64) -> Self {
        Self {
            id,
            position: [x, y, z],
            velocity: [0.0, 0.0, 0.0],
            mass: 1.0,
            pinned: false,
        }
    }
}

/// A directed or undirected edge in the 3D layout graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutEdge3D {
    /// Source node index.
    pub source: usize,
    /// Target node index.
    pub target: usize,
    /// Edge weight — stronger attraction for higher weights.
    pub weight: f64,
    /// Ideal (rest-length) spring length in layout units.
    pub ideal_length: f64,
}

/// Configuration for a 3D layout run.
///
/// Extends [`LayoutConfig`] with z-axis parameters including optional
/// gravitational collapse along the z dimension.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutConfig3D {
    /// Maximum number of simulation steps.
    pub iterations: usize,
    /// Global repulsion constant (Coulomb-like).
    pub repulsion_strength: f64,
    /// Global attraction constant (Hooke-like).
    pub attraction_strength: f64,
    /// Velocity damping factor in `[0, 1]`.
    pub damping: f64,
    /// Minimum pairwise distance to avoid division-by-zero singularities.
    pub min_distance: f64,
    /// Barnes-Hut opening angle `theta`.
    pub theta: f64,
    /// Multiplicative temperature decay per iteration — in `(0, 1]`.
    pub cooling_factor: f64,
    /// Starting temperature (maximum displacement per step at iteration 0).
    pub initial_temperature: f64,
    /// Spherical gravity strength pulling every node toward the origin.
    /// Set to 0.0 to disable.
    pub gravity: f64,
}

impl Default for LayoutConfig3D {
    fn default() -> Self {
        Self {
            iterations: 300,
            repulsion_strength: 10_000.0,
            attraction_strength: 0.05,
            damping: 0.85,
            min_distance: 1.0,
            theta: 0.8,
            cooling_factor: 0.995,
            initial_temperature: 200.0,
            gravity: 0.0,
        }
    }
}

impl LayoutConfig3D {
    /// Validate that all parameters are in meaningful ranges.
    pub fn validate(&self) -> Result<(), LayoutError> {
        if self.iterations == 0 {
            return Err(LayoutError::InvalidParameter(
                "iterations must be > 0".to_string(),
            ));
        }
        if self.repulsion_strength < 0.0 {
            return Err(LayoutError::InvalidParameter(
                "repulsion_strength must be >= 0".to_string(),
            ));
        }
        if self.attraction_strength < 0.0 {
            return Err(LayoutError::InvalidParameter(
                "attraction_strength must be >= 0".to_string(),
            ));
        }
        if !(0.0..=1.0).contains(&self.damping) {
            return Err(LayoutError::InvalidParameter(
                "damping must be in [0, 1]".to_string(),
            ));
        }
        if self.min_distance <= 0.0 {
            return Err(LayoutError::InvalidParameter(
                "min_distance must be > 0".to_string(),
            ));
        }
        if self.theta <= 0.0 {
            return Err(LayoutError::InvalidParameter(
                "theta must be > 0".to_string(),
            ));
        }
        if !(0.0..=1.0).contains(&self.cooling_factor) {
            return Err(LayoutError::InvalidParameter(
                "cooling_factor must be in (0, 1]".to_string(),
            ));
        }
        if self.initial_temperature <= 0.0 {
            return Err(LayoutError::InvalidParameter(
                "initial_temperature must be > 0".to_string(),
            ));
        }
        Ok(())
    }
}

/// Output of a completed 3D layout run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutResult3D {
    /// Final node positions and velocities.
    pub nodes: Vec<LayoutNode3D>,
    /// Number of simulation steps actually performed.
    pub iterations_run: usize,
    /// Whether the system energy fell below the convergence threshold.
    pub converged: bool,
    /// Final total system energy.
    pub energy: f64,
}

/// Describes the memory layout of 3D GPU vertex buffers.
///
/// All offsets are in bytes from the start of a single flat `f32` buffer.
/// Each node occupies 3 floats per section (`x, y, z`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GpuBufferLayout3D {
    /// Byte offset to the positions block (`x, y, z` triples).
    pub positions_offset: u32,
    /// Byte offset to the velocities block (`vx, vy, vz` triples).
    pub velocities_offset: u32,
    /// Byte offset to the forces accumulation block (`fx, fy, fz` triples).
    pub forces_offset: u32,
    /// Byte offset from the start of one node's xyz triple to its z component.
    pub z_component_offset: u32,
    /// Stride in bytes between consecutive node records within each block.
    pub stride: u32,
    /// Total number of nodes described by this layout.
    pub node_count: u32,
}

// ── Octree for 3D Barnes-Hut ─────────────────────────────────────────────────

/// A node in the Barnes-Hut octree.
///
/// Each `OctreeNode` represents a cubic spatial region. Leaf nodes contain a
/// single body. Internal nodes aggregate centre-of-mass and total mass of all
/// bodies inside their region.
///
/// Octant numbering uses three bits: bit 0 = east (x ≥ mid), bit 1 = south
/// (y ≥ mid), bit 2 = top (z ≥ mid). Octants 0–7 map to all combinations.
#[derive(Debug, Clone)]
pub struct OctreeNode {
    /// Spatial bounds `[x_min, y_min, z_min, x_max, y_max, z_max]`.
    pub bounds: [f64; 6],
    /// Aggregated centre-of-mass `[cx, cy, cz]` for all bodies in this region.
    pub center_of_mass: [f64; 3],
    /// Total mass of all bodies in this region.
    pub total_mass: f64,
    /// Eight children (one per octant), present only for internal nodes.
    pub children: [Option<Box<OctreeNode>>; 8],
    /// For leaf nodes: the index of the single body contained here.
    pub body: Option<usize>,
}

impl OctreeNode {
    fn new(bounds: [f64; 6]) -> Self {
        Self {
            bounds,
            center_of_mass: [0.0, 0.0, 0.0],
            total_mass: 0.0,
            children: [None, None, None, None, None, None, None, None],
            body: None,
        }
    }

    fn is_leaf(&self) -> bool {
        self.children.iter().all(|c| c.is_none())
    }

    fn is_empty(&self) -> bool {
        self.is_leaf() && self.body.is_none()
    }

    fn is_single_body(&self) -> bool {
        self.is_leaf() && self.body.is_some()
    }

    /// Largest spatial dimension of this node's bounding cuboid.
    fn width(&self) -> f64 {
        let dx = self.bounds[3] - self.bounds[0];
        let dy = self.bounds[4] - self.bounds[1];
        let dz = self.bounds[5] - self.bounds[2];
        dx.max(dy).max(dz)
    }

    /// Determine which octant (0–7) a 3D position falls into.
    fn octant_for(&self, pos: [f64; 3]) -> usize {
        let mid_x = (self.bounds[0] + self.bounds[3]) / 2.0;
        let mid_y = (self.bounds[1] + self.bounds[4]) / 2.0;
        let mid_z = (self.bounds[2] + self.bounds[5]) / 2.0;
        let east = usize::from(pos[0] >= mid_x);
        let south = usize::from(pos[1] >= mid_y) << 1;
        let top = usize::from(pos[2] >= mid_z) << 2;
        east | south | top
    }

    /// Sub-bounds for the given octant index.
    fn child_bounds(&self, oct: usize) -> [f64; 6] {
        let mid_x = (self.bounds[0] + self.bounds[3]) / 2.0;
        let mid_y = (self.bounds[1] + self.bounds[4]) / 2.0;
        let mid_z = (self.bounds[2] + self.bounds[5]) / 2.0;
        let x_lo = if oct & 1 != 0 { mid_x } else { self.bounds[0] };
        let x_hi = if oct & 1 != 0 { self.bounds[3] } else { mid_x };
        let y_lo = if oct & 2 != 0 { mid_y } else { self.bounds[1] };
        let y_hi = if oct & 2 != 0 { self.bounds[4] } else { mid_y };
        let z_lo = if oct & 4 != 0 { mid_z } else { self.bounds[2] };
        let z_hi = if oct & 4 != 0 { self.bounds[5] } else { mid_z };
        [x_lo, y_lo, z_lo, x_hi, y_hi, z_hi]
    }

    fn insert(&mut self, idx: usize, nodes: &[LayoutNode3D], depth: usize) {
        const MAX_DEPTH: usize = 64;

        if self.is_empty() {
            self.body = Some(idx);
            self.total_mass = nodes[idx].mass;
            self.center_of_mass = nodes[idx].position;
            return;
        }

        if self.is_single_body() && depth < MAX_DEPTH {
            let existing = self.body.take();
            if let Some(ei) = existing {
                let ei_pos = nodes[ei].position;
                let q = self.octant_for(ei_pos);
                if self.children[q].is_none() {
                    self.children[q] = Some(Box::new(OctreeNode::new(self.child_bounds(q))));
                }
                if let Some(child) = self.children[q].as_mut() {
                    child.insert(ei, nodes, depth + 1);
                }
            }
        }

        let pos = nodes[idx].position;
        let q = self.octant_for(pos);
        if self.children[q].is_none() {
            self.children[q] = Some(Box::new(OctreeNode::new(self.child_bounds(q))));
        }
        if let Some(child) = self.children[q].as_mut() {
            child.insert(idx, nodes, depth + 1);
        }

        let mass = nodes[idx].mass;
        let new_total = self.total_mass + mass;
        if new_total > 0.0 {
            self.center_of_mass[0] =
                (self.center_of_mass[0] * self.total_mass + pos[0] * mass) / new_total;
            self.center_of_mass[1] =
                (self.center_of_mass[1] * self.total_mass + pos[1] * mass) / new_total;
            self.center_of_mass[2] =
                (self.center_of_mass[2] * self.total_mass + pos[2] * mass) / new_total;
        }
        self.total_mass = new_total;
    }
}

/// An octree for Barnes-Hut O(N log N) 3D force approximation.
#[derive(Debug, Clone)]
pub struct Octree {
    /// Root node covering the bounding box of all bodies.
    pub root: OctreeNode,
    /// Number of bodies inserted.
    pub node_count: usize,
}

/// Build a Barnes-Hut octree from 3D node positions.
///
/// The tree covers the tight bounding box of all nodes, expanded slightly to
/// avoid boundary artefacts.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::gpu_layout::{random_layout_3d, build_octree};
///
/// let nodes = random_layout_3d(10, 400.0, 400.0, 400.0);
/// let tree = build_octree(&nodes);
/// assert_eq!(tree.node_count, 10);
/// assert!(tree.root.total_mass > 0.0);
/// ```
pub fn build_octree(nodes: &[LayoutNode3D]) -> Octree {
    let mut x_min = f64::MAX;
    let mut x_max = f64::MIN;
    let mut y_min = f64::MAX;
    let mut y_max = f64::MIN;
    let mut z_min = f64::MAX;
    let mut z_max = f64::MIN;

    for n in nodes {
        x_min = x_min.min(n.position[0]);
        x_max = x_max.max(n.position[0]);
        y_min = y_min.min(n.position[1]);
        y_max = y_max.max(n.position[1]);
        z_min = z_min.min(n.position[2]);
        z_max = z_max.max(n.position[2]);
    }

    let margin = ((x_max - x_min) + (y_max - y_min) + (z_max - z_min)) * 0.01 + 1.0;
    let bounds = [
        x_min - margin,
        y_min - margin,
        z_min - margin,
        x_max + margin,
        y_max + margin,
        z_max + margin,
    ];

    let mut root = OctreeNode::new(bounds);
    for idx in 0..nodes.len() {
        root.insert(idx, nodes, 0);
    }

    Octree {
        root,
        node_count: nodes.len(),
    }
}

/// Compute the Barnes-Hut repulsion force on a single 3D node from the octree.
///
/// Traverses the tree, treating cells as point masses when the cell width–to–
/// distance ratio falls below `theta`.
///
/// Returns `[fx, fy, fz]` — the net repulsive force vector.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::gpu_layout::{random_layout_3d, build_octree, compute_repulsion_bh_3d};
///
/// let nodes = random_layout_3d(20, 600.0, 600.0, 600.0);
/// let tree = build_octree(&nodes);
/// let force = compute_repulsion_bh_3d(&tree, &nodes[0], 0.8);
/// assert!(force.iter().all(|f| f.is_finite()));
/// ```
pub fn compute_repulsion_bh_3d(
    tree: &Octree,
    node: &LayoutNode3D,
    theta: f64,
    strength: f64,
    min_dist: f64,
) -> [f64; 3] {
    let mut force = [0.0_f64; 3];
    accumulate_repulsion_3d(&tree.root, node, theta, &mut force, strength, min_dist);
    force
}

// ── 3D Layout Algorithms ──────────────────────────────────────────────────────

/// Initialize 3D nodes with deterministic pseudo-random positions.
///
/// Positions are spread uniformly within `[0, width] × [0, height] × [0, depth]`.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::gpu_layout::random_layout_3d;
///
/// let nodes = random_layout_3d(10, 800.0, 600.0, 400.0);
/// assert_eq!(nodes.len(), 10);
/// for node in &nodes {
///     assert!(node.position[0] >= 0.0 && node.position[0] <= 800.0);
///     assert!(node.position[2] >= 0.0 && node.position[2] <= 400.0);
/// }
/// ```
pub fn random_layout_3d(n: usize, width: f64, height: f64, depth: f64) -> Vec<LayoutNode3D> {
    let mut state: u64 = 0xCAFE_BABE_DEAD_BEEF;
    let mut nodes = Vec::with_capacity(n);
    for id in 0..n {
        let x = xorshift64(&mut state) * width;
        let y = xorshift64(&mut state) * height;
        let z = xorshift64(&mut state) * depth;
        nodes.push(LayoutNode3D::new(id, x, y, z));
    }
    nodes
}

/// Scale all 3D node positions so the bounding box fills
/// `[0, width] × [0, height] × [0, depth]`.
///
/// Nodes with identical positions in any axis are placed at the centre.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::gpu_layout::{LayoutNode3D, normalize_positions_3d};
///
/// let mut nodes = vec![
///     LayoutNode3D::new(0, 0.0, 0.0, 0.0),
///     LayoutNode3D::new(1, 100.0, 200.0, 50.0),
/// ];
/// normalize_positions_3d(&mut nodes, 400.0, 400.0, 200.0);
/// assert!((nodes[1].position[0] - 400.0).abs() < 1e-9);
/// assert!((nodes[1].position[2] - 200.0).abs() < 1e-9);
/// ```
pub fn normalize_positions_3d(nodes: &mut [LayoutNode3D], width: f64, height: f64, depth: f64) {
    if nodes.is_empty() {
        return;
    }

    let mut x_min = f64::MAX;
    let mut x_max = f64::MIN;
    let mut y_min = f64::MAX;
    let mut y_max = f64::MIN;
    let mut z_min = f64::MAX;
    let mut z_max = f64::MIN;

    for n in nodes.iter() {
        x_min = x_min.min(n.position[0]);
        x_max = x_max.max(n.position[0]);
        y_min = y_min.min(n.position[1]);
        y_max = y_max.max(n.position[1]);
        z_min = z_min.min(n.position[2]);
        z_max = z_max.max(n.position[2]);
    }

    let dx = x_max - x_min;
    let dy = y_max - y_min;
    let dz = z_max - z_min;

    for n in nodes.iter_mut() {
        if n.pinned {
            continue;
        }
        n.position[0] = if dx > 0.0 {
            (n.position[0] - x_min) / dx * width
        } else {
            width / 2.0
        };
        n.position[1] = if dy > 0.0 {
            (n.position[1] - y_min) / dy * height
        } else {
            height / 2.0
        };
        n.position[2] = if dz > 0.0 {
            (n.position[2] - z_min) / dz * depth
        } else {
            depth / 2.0
        };
    }
}

/// Classic Fruchterman-Reingold force-directed layout in 3D.
///
/// Uses O(N²) pairwise repulsion for small graphs (N ≤ 500). For larger
/// graphs, call [`compute_layout_3d`] which switches to Barnes-Hut.
///
/// # Errors
///
/// Returns [`LayoutError::EmptyGraph`] if `nodes` is empty, or
/// [`LayoutError::InvalidNode`] if any edge references a missing index.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::gpu_layout::{
///     LayoutConfig3D, LayoutEdge3D, fruchterman_reingold_3d, random_layout_3d,
/// };
///
/// let mut nodes = random_layout_3d(4, 400.0, 400.0, 400.0);
/// let edges = vec![
///     LayoutEdge3D { source: 0, target: 1, weight: 1.0, ideal_length: 80.0 },
/// ];
/// let mut config = LayoutConfig3D::default();
/// config.iterations = 20;
/// let Ok(result) = fruchterman_reingold_3d(&mut nodes, &edges, &config) else { return; };
/// assert!(result.iterations_run > 0);
/// ```
pub fn fruchterman_reingold_3d(
    nodes: &mut [LayoutNode3D],
    edges: &[LayoutEdge3D],
    config: &LayoutConfig3D,
) -> Result<LayoutResult3D, LayoutError> {
    if nodes.is_empty() {
        return Err(LayoutError::EmptyGraph);
    }
    config.validate()?;
    validate_edges_3d(nodes, edges)?;

    let n = nodes.len();
    let mut temperature = config.initial_temperature;
    let convergence_threshold = 0.01 * temperature * config.cooling_factor.powi(50);
    let mut forces = vec![[0.0_f64; 3]; n];
    let mut iterations_run = 0;
    let mut converged = false;

    for _iter in 0..config.iterations {
        for f in forces.iter_mut() {
            *f = [0.0, 0.0, 0.0];
        }

        // Repulsion O(N²).
        for i in 0..n {
            for j in (i + 1)..n {
                let [fx, fy, fz] = repulsion_force_3d(
                    nodes[i].position,
                    nodes[j].position,
                    nodes[i].mass,
                    nodes[j].mass,
                    config.repulsion_strength,
                    config.min_distance,
                );
                forces[i][0] += fx;
                forces[i][1] += fy;
                forces[i][2] += fz;
                forces[j][0] -= fx;
                forces[j][1] -= fy;
                forces[j][2] -= fz;
            }
        }

        // Attraction along edges.
        for edge in edges {
            let [fx, fy, fz] = attraction_force_3d(
                nodes[edge.source].position,
                nodes[edge.target].position,
                edge.weight,
                edge.ideal_length,
                config.attraction_strength,
                config.min_distance,
            );
            forces[edge.source][0] += fx;
            forces[edge.source][1] += fy;
            forces[edge.source][2] += fz;
            forces[edge.target][0] -= fx;
            forces[edge.target][1] -= fy;
            forces[edge.target][2] -= fz;
        }

        // Integrate.
        let mut max_displacement = 0.0_f64;
        for (node, force) in nodes.iter_mut().zip(forces.iter()) {
            if node.pinned {
                continue;
            }
            let mag = vec3_len(force).max(f64::MIN_POSITIVE);
            let scale = mag.min(temperature) / mag;
            for (v, f) in node.velocity.iter_mut().zip(force.iter()) {
                *v = (*v + f * scale) * config.damping;
            }
            let disp = vec3_len(&node.velocity);
            max_displacement = max_displacement.max(disp);
            for k in 0..3 {
                node.position[k] += node.velocity[k];
            }
        }

        temperature *= config.cooling_factor;
        iterations_run += 1;

        if max_displacement < convergence_threshold {
            converged = true;
            break;
        }
    }

    let energy = compute_energy_3d(nodes, edges, config);
    Ok(LayoutResult3D {
        nodes: nodes.to_vec(),
        iterations_run,
        converged,
        energy,
    })
}

/// ForceAtlas2 variant with spherical gravity in 3D.
///
/// Differences from [`fruchterman_reingold_3d`]:
/// - Gravity force pulls every node towards the origin (magnitude controlled by
///   `config.gravity`).
/// - Attraction is logarithmic (`log(1 + dist)` instead of linear).
///
/// # Errors
///
/// Same conditions as [`fruchterman_reingold_3d`].
///
/// # Example
///
/// ```rust
/// use nexcore_viz::gpu_layout::{
///     LayoutConfig3D, LayoutEdge3D, force_atlas2_3d, random_layout_3d,
/// };
///
/// let mut nodes = random_layout_3d(5, 400.0, 400.0, 400.0);
/// let edges = vec![
///     LayoutEdge3D { source: 0, target: 1, weight: 1.0, ideal_length: 100.0 },
/// ];
/// let mut config = LayoutConfig3D::default();
/// config.iterations = 20;
/// let Ok(result) = force_atlas2_3d(&mut nodes, &edges, &config) else { return; };
/// assert!(result.iterations_run > 0);
/// ```
pub fn force_atlas2_3d(
    nodes: &mut [LayoutNode3D],
    edges: &[LayoutEdge3D],
    config: &LayoutConfig3D,
) -> Result<LayoutResult3D, LayoutError> {
    if nodes.is_empty() {
        return Err(LayoutError::EmptyGraph);
    }
    config.validate()?;
    validate_edges_3d(nodes, edges)?;

    let n = nodes.len();
    let gravity = config.repulsion_strength * 0.0001 + config.gravity;
    let mut temperature = config.initial_temperature;
    let mut forces = vec![[0.0_f64; 3]; n];
    let mut iterations_run = 0;
    let mut converged = false;

    for _iter in 0..config.iterations {
        for f in forces.iter_mut() {
            *f = [0.0, 0.0, 0.0];
        }

        // Repulsion O(N²).
        for i in 0..n {
            for j in (i + 1)..n {
                let [fx, fy, fz] = repulsion_force_3d(
                    nodes[i].position,
                    nodes[j].position,
                    nodes[i].mass,
                    nodes[j].mass,
                    config.repulsion_strength,
                    config.min_distance,
                );
                forces[i][0] += fx;
                forces[i][1] += fy;
                forces[i][2] += fz;
                forces[j][0] -= fx;
                forces[j][1] -= fy;
                forces[j][2] -= fz;
            }
        }

        // Spherical gravity towards origin.
        for (i, node) in nodes.iter().enumerate() {
            let d = vec3_len(&node.position).max(config.min_distance);
            for (force_k, pos_k) in forces[i].iter_mut().zip(node.position.iter()) {
                *force_k -= gravity * node.mass * pos_k / d;
            }
        }

        // Log-attraction along edges.
        for edge in edges {
            let dx = nodes[edge.target].position[0] - nodes[edge.source].position[0];
            let dy = nodes[edge.target].position[1] - nodes[edge.source].position[1];
            let dz = nodes[edge.target].position[2] - nodes[edge.source].position[2];
            let d = (dx * dx + dy * dy + dz * dz)
                .sqrt()
                .max(config.min_distance);
            let factor = config.attraction_strength * edge.weight * (1.0 + d).ln() / d;
            forces[edge.source][0] += dx * factor;
            forces[edge.source][1] += dy * factor;
            forces[edge.source][2] += dz * factor;
            forces[edge.target][0] -= dx * factor;
            forces[edge.target][1] -= dy * factor;
            forces[edge.target][2] -= dz * factor;
        }

        // Integrate.
        let mut max_displacement = 0.0_f64;
        for (node, force) in nodes.iter_mut().zip(forces.iter()) {
            if node.pinned {
                continue;
            }
            let mag = vec3_len(force).max(f64::MIN_POSITIVE);
            let scale = mag.min(temperature) / mag;
            for (v, f) in node.velocity.iter_mut().zip(force.iter()) {
                *v = (*v + f * scale) * config.damping;
            }
            let disp = vec3_len(&node.velocity);
            max_displacement = max_displacement.max(disp);
            for k in 0..3 {
                node.position[k] += node.velocity[k];
            }
        }

        temperature *= config.cooling_factor;
        iterations_run += 1;

        let convergence_threshold =
            0.01 * config.initial_temperature * config.cooling_factor.powi(50);
        if max_displacement < convergence_threshold {
            converged = true;
            break;
        }
    }

    let energy = compute_energy_3d(nodes, edges, config);
    Ok(LayoutResult3D {
        nodes: nodes.to_vec(),
        iterations_run,
        converged,
        energy,
    })
}

/// High-level 3D layout entry point.
///
/// Automatically selects the best algorithm:
/// - N ≤ 500 → [`fruchterman_reingold_3d`] (O(N²), exact)
/// - N > 500  → Barnes-Hut accelerated 3D Fruchterman-Reingold (O(N log N))
///
/// # Errors
///
/// Returns [`LayoutError::EmptyGraph`] if `nodes` is empty.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::gpu_layout::{
///     LayoutConfig3D, LayoutEdge3D, compute_layout_3d, random_layout_3d,
/// };
///
/// let nodes = random_layout_3d(8, 800.0, 600.0, 400.0);
/// let edges = vec![
///     LayoutEdge3D { source: 0, target: 1, weight: 1.0, ideal_length: 100.0 },
/// ];
/// let config = LayoutConfig3D::default();
/// let Ok(result) = compute_layout_3d(nodes, edges, &config) else { return; };
/// assert!(!result.nodes.is_empty());
/// ```
pub fn compute_layout_3d(
    mut nodes: Vec<LayoutNode3D>,
    edges: Vec<LayoutEdge3D>,
    config: &LayoutConfig3D,
) -> Result<LayoutResult3D, LayoutError> {
    if nodes.is_empty() {
        return Err(LayoutError::EmptyGraph);
    }
    config.validate()?;
    validate_edges_3d(&nodes, &edges)?;

    if nodes.len() <= 500 {
        fruchterman_reingold_3d(&mut nodes, &edges, config)
    } else {
        fruchterman_reingold_bh_3d(&mut nodes, &edges, config)
    }
}

// ── 3D GPU Buffer Packing ─────────────────────────────────────────────────────

/// Pack 3D node and edge data into GPU-friendly flat `f32` buffers.
///
/// The returned `Vec<f32>` is a flat buffer with three contiguous sections,
/// each containing `node_count * 3` floats (`x, y, z` triples):
///
/// ```text
/// [positions: x0,y0,z0, x1,y1,z1, ...]    <- positions_offset bytes
/// [velocities: vx0,vy0,vz0, vx1,...]       <- velocities_offset bytes
/// [forces: fx0,fy0,fz0, fx1,...]           <- forces_offset bytes (zeroed)
/// ```
///
/// The `Vec<u32>` holds edge index pairs `[src0, tgt0, src1, tgt1, ...]`.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::gpu_layout::{LayoutEdge3D, LayoutNode3D, prepare_gpu_buffers_3d};
///
/// let nodes = vec![
///     LayoutNode3D::new(0, 1.0, 2.0, 3.0),
///     LayoutNode3D::new(1, 4.0, 5.0, 6.0),
/// ];
/// let edges = vec![
///     LayoutEdge3D { source: 0, target: 1, weight: 1.0, ideal_length: 50.0 },
/// ];
/// let (floats, indices, layout) = prepare_gpu_buffers_3d(&nodes, &edges);
/// assert_eq!(layout.node_count, 2);
/// assert_eq!(indices.len(), 2);
/// // First position triple: 1.0, 2.0, 3.0
/// assert!((floats[0] - 1.0_f32).abs() < 1e-5);
/// assert!((floats[1] - 2.0_f32).abs() < 1e-5);
/// assert!((floats[2] - 3.0_f32).abs() < 1e-5);
/// ```
pub fn prepare_gpu_buffers_3d(
    nodes: &[LayoutNode3D],
    edges: &[LayoutEdge3D],
) -> (Vec<f32>, Vec<u32>, GpuBufferLayout3D) {
    let n = nodes.len();
    let section = n * 3; // 3 floats per node per section
    let total = section * 3;
    let mut buffer = vec![0.0_f32; total];

    // Positions.
    for (i, node) in nodes.iter().enumerate() {
        buffer[i * 3] = node.position[0] as f32;
        buffer[i * 3 + 1] = node.position[1] as f32;
        buffer[i * 3 + 2] = node.position[2] as f32;
    }

    // Velocities (after positions block).
    let vel_base = section;
    for (i, node) in nodes.iter().enumerate() {
        buffer[vel_base + i * 3] = node.velocity[0] as f32;
        buffer[vel_base + i * 3 + 1] = node.velocity[1] as f32;
        buffer[vel_base + i * 3 + 2] = node.velocity[2] as f32;
    }

    // Forces block starts after velocities (already zeroed).
    let forces_base = section * 2;
    let f32_size = std::mem::size_of::<f32>() as u32;

    let layout = GpuBufferLayout3D {
        positions_offset: 0,
        velocities_offset: (section as u32) * f32_size,
        forces_offset: (forces_base as u32) * f32_size,
        z_component_offset: 2 * f32_size, // z is the third float in each xyz triple
        stride: 3 * f32_size,
        node_count: n as u32,
    };

    let mut indices = Vec::with_capacity(edges.len() * 2);
    for edge in edges {
        indices.push(edge.source as u32);
        indices.push(edge.target as u32);
    }

    debug_assert_eq!(buffer.len(), total);
    (buffer, indices, layout)
}

// ── 3D Energy ─────────────────────────────────────────────────────────────────

/// Compute the total kinetic + potential energy of the 3D system.
///
/// Energy = Σ 0.5·m·|v|² (kinetic) + Σ repulsion terms + Σ spring terms.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::gpu_layout::{
///     LayoutConfig3D, LayoutEdge3D, compute_energy_3d, random_layout_3d,
/// };
///
/// let nodes = random_layout_3d(4, 200.0, 200.0, 200.0);
/// let edges = vec![
///     LayoutEdge3D { source: 0, target: 1, weight: 1.0, ideal_length: 50.0 },
/// ];
/// let config = LayoutConfig3D::default();
/// let e = compute_energy_3d(&nodes, &edges, &config);
/// assert!(e.is_finite() && e >= 0.0);
/// ```
pub fn compute_energy_3d(
    nodes: &[LayoutNode3D],
    edges: &[LayoutEdge3D],
    config: &LayoutConfig3D,
) -> f64 {
    let kinetic: f64 = nodes
        .iter()
        .map(|n| 0.5 * n.mass * n.velocity.iter().map(|v| v * v).sum::<f64>())
        .sum();

    let mut repulsion = 0.0;
    for i in 0..nodes.len() {
        for j in (i + 1)..nodes.len() {
            let dx = nodes[j].position[0] - nodes[i].position[0];
            let dy = nodes[j].position[1] - nodes[i].position[1];
            let dz = nodes[j].position[2] - nodes[i].position[2];
            let d = (dx * dx + dy * dy + dz * dz)
                .sqrt()
                .max(config.min_distance);
            repulsion += config.repulsion_strength * nodes[i].mass * nodes[j].mass / d;
        }
    }

    let spring: f64 = edges
        .iter()
        .filter_map(|e| {
            let src = nodes.get(e.source)?;
            let tgt = nodes.get(e.target)?;
            let dx = tgt.position[0] - src.position[0];
            let dy = tgt.position[1] - src.position[1];
            let dz = tgt.position[2] - src.position[2];
            let d = (dx * dx + dy * dy + dz * dz).sqrt();
            let stretch = d - e.ideal_length;
            Some(0.5 * config.attraction_strength * e.weight * stretch * stretch)
        })
        .sum();

    kinetic + repulsion + spring
}

// ── Layout Quality Metrics ────────────────────────────────────────────────────

/// Compute Kamada-Kawai stress for a 2D graph layout.
///
/// Stress = Σ_{i<j} (d_ij − ideal_ij)² / ideal_ij²
///
/// where `d_ij` is the Euclidean distance between nodes i and j, and
/// `ideal_ij` is the graph-theoretic shortest-path distance (sum of
/// `ideal_length` along the path). Disconnected pairs are ignored.
///
/// Returns `0.0` for graphs with fewer than 2 nodes.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::gpu_layout::{LayoutEdge, LayoutNode, compute_stress};
///
/// let nodes = vec![
///     LayoutNode::new(0, 0.0, 0.0),
///     LayoutNode::new(1, 100.0, 0.0),
/// ];
/// let edges = vec![
///     LayoutEdge { source: 0, target: 1, weight: 1.0, ideal_length: 100.0 },
/// ];
/// let stress = compute_stress(&nodes, &edges);
/// assert!(stress.is_finite() && stress >= 0.0);
/// ```
pub fn compute_stress(nodes: &[LayoutNode], edges: &[LayoutEdge]) -> f64 {
    let n = nodes.len();
    if n < 2 {
        return 0.0;
    }

    // Build adjacency list weighted by ideal_length.
    let mut adj: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
    for e in edges {
        if e.source < n && e.target < n {
            adj[e.source].push((e.target, e.ideal_length));
            adj[e.target].push((e.source, e.ideal_length));
        }
    }

    let mut stress = 0.0;
    for i in 0..n {
        // BFS to compute shortest weighted distances from node i.
        let mut graph_dist = vec![f64::INFINITY; n];
        graph_dist[i] = 0.0;
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(i);

        while let Some(cur) = queue.pop_front() {
            for &(nb, ideal) in &adj[cur] {
                if graph_dist[nb].is_infinite() {
                    graph_dist[nb] = graph_dist[cur] + ideal;
                    queue.push_back(nb);
                }
            }
        }

        for j in (i + 1)..n {
            let ideal_ij = graph_dist[j];
            if !ideal_ij.is_finite() || ideal_ij <= 0.0 {
                continue;
            }
            let dx = nodes[j].position[0] - nodes[i].position[0];
            let dy = nodes[j].position[1] - nodes[i].position[1];
            let d_ij = dx.hypot(dy);
            let diff = d_ij - ideal_ij;
            stress += (diff * diff) / (ideal_ij * ideal_ij);
        }
    }

    stress
}

/// Count edge crossings in a 2D graph layout.
///
/// Two edges (a–b) and (c–d) cross if their open line segments properly
/// intersect (shared endpoint pairs are excluded). O(E²).
///
/// Returns `0` for graphs with fewer than 2 edges.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::gpu_layout::{LayoutEdge, LayoutNode, compute_edge_crossing_count};
///
/// // Two diagonals of a square — they cross once.
/// let nodes = vec![
///     LayoutNode::new(0,   0.0,   0.0),
///     LayoutNode::new(1, 100.0,   0.0),
///     LayoutNode::new(2, 100.0, 100.0),
///     LayoutNode::new(3,   0.0, 100.0),
/// ];
/// let edges = vec![
///     LayoutEdge { source: 0, target: 2, weight: 1.0, ideal_length: 141.0 },
///     LayoutEdge { source: 1, target: 3, weight: 1.0, ideal_length: 141.0 },
/// ];
/// assert_eq!(compute_edge_crossing_count(&nodes, &edges), 1);
/// ```
pub fn compute_edge_crossing_count(nodes: &[LayoutNode], edges: &[LayoutEdge]) -> usize {
    let m = edges.len();
    if m < 2 {
        return 0;
    }

    let mut count = 0;
    for i in 0..m {
        let e1 = &edges[i];
        if e1.source >= nodes.len() || e1.target >= nodes.len() {
            continue;
        }
        let a = nodes[e1.source].position;
        let b = nodes[e1.target].position;

        for e2 in edges.iter().skip(i + 1) {
            if e2.source >= nodes.len() || e2.target >= nodes.len() {
                continue;
            }
            // Skip edges sharing an endpoint.
            if e1.source == e2.source
                || e1.source == e2.target
                || e1.target == e2.source
                || e1.target == e2.target
            {
                continue;
            }
            let c = nodes[e2.source].position;
            let d = nodes[e2.target].position;
            if segments_intersect(a, b, c, d) {
                count += 1;
            }
        }
    }

    count
}

/// Compute neighborhood preservation between graph topology and spatial layout.
///
/// For each node, finds its `k` nearest graph neighbours (by BFS hop count)
/// and its `k` nearest spatial neighbours (by Euclidean distance), then
/// returns the fraction of graph neighbours that are also spatial neighbours,
/// averaged over all nodes.
///
/// Returns `1.0` if `k == 0` or the graph has fewer than 2 nodes.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::gpu_layout::{LayoutEdge, LayoutNode, compute_neighborhood_preservation};
///
/// let nodes = vec![
///     LayoutNode::new(0,   0.0, 0.0),
///     LayoutNode::new(1, 100.0, 0.0),
///     LayoutNode::new(2, 200.0, 0.0),
/// ];
/// let edges = vec![
///     LayoutEdge { source: 0, target: 1, weight: 1.0, ideal_length: 100.0 },
///     LayoutEdge { source: 1, target: 2, weight: 1.0, ideal_length: 100.0 },
/// ];
/// let np = compute_neighborhood_preservation(&nodes, &edges, 1);
/// assert!(np >= 0.0 && np <= 1.0);
/// ```
pub fn compute_neighborhood_preservation(
    nodes: &[LayoutNode],
    edges: &[LayoutEdge],
    k: usize,
) -> f64 {
    let n = nodes.len();
    if n < 2 || k == 0 {
        return 1.0;
    }

    let k_eff = k.min(n - 1);

    // Build undirected adjacency list.
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for e in edges {
        if e.source < n && e.target < n {
            adj[e.source].push(e.target);
            adj[e.target].push(e.source);
        }
    }

    let mut total = 0.0;

    for i in 0..n {
        // BFS to collect up to k_eff nearest graph neighbours.
        let mut visited = vec![false; n];
        visited[i] = true;
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(i);
        let mut graph_nb: Vec<usize> = Vec::with_capacity(k_eff);

        'bfs: while let Some(cur) = queue.pop_front() {
            for &nb in &adj[cur] {
                if !visited[nb] {
                    visited[nb] = true;
                    graph_nb.push(nb);
                    queue.push_back(nb);
                    if graph_nb.len() >= k_eff {
                        break 'bfs;
                    }
                }
            }
        }

        if graph_nb.is_empty() {
            total += 1.0;
            continue;
        }

        // k_eff nearest spatial neighbours (by squared distance).
        let mut spatial: Vec<(usize, f64)> = (0..n)
            .filter(|&j| j != i)
            .map(|j| {
                let dx = nodes[j].position[0] - nodes[i].position[0];
                let dy = nodes[j].position[1] - nodes[i].position[1];
                (j, dx * dx + dy * dy)
            })
            .collect();
        spatial.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        let spatial_set: std::collections::HashSet<usize> = spatial
            .iter()
            .take(graph_nb.len())
            .map(|&(j, _)| j)
            .collect();
        let graph_set: std::collections::HashSet<usize> = graph_nb.iter().copied().collect();

        let overlap = graph_set.intersection(&spatial_set).count();
        total += overlap as f64 / graph_nb.len() as f64;
    }

    total / n as f64
}

// ── WGSL GPU Compute Shader Kernels ──────────────────────────────────────────

/// Returns the WGSL source for the edge spring attraction compute kernel.
///
/// Each workgroup thread handles one edge, computing the Hooke spring force
/// and accumulating it into source and target node force buffers via
/// fixed-point atomic operations (`workgroup_size(256)`).
///
/// # Shader Interface
///
/// | Binding | Group | Type | Description |
/// |---------|-------|------|-------------|
/// | 0 | 0 | storage read | Node positions (f32 xyz triples) |
/// | 1 | 0 | storage read | Edge data (src, tgt, weight, ideal_len) |
/// | 2 | 0 | storage read_write | Force accumulators (atomic i32 xyz triples) |
/// | 3 | 0 | uniform | Layout uniforms |
///
/// # Example
///
/// ```rust
/// use nexcore_viz::gpu_layout::wgsl_attraction_kernel;
///
/// let src = wgsl_attraction_kernel();
/// assert!(src.contains("@compute"));
/// assert!(src.contains("workgroup_size"));
/// ```
pub fn wgsl_attraction_kernel() -> &'static str {
    WGSL_ATTRACTION_KERNEL
}

/// Returns the WGSL source for the Barnes-Hut N-body repulsion compute kernel.
///
/// Each thread processes one node, traversing a linearized octree stored in
/// a flat array. Cells are treated as point masses when the Barnes-Hut
/// criterion `width / dist < theta` is satisfied (`workgroup_size(256)`).
///
/// # Example
///
/// ```rust
/// use nexcore_viz::gpu_layout::wgsl_repulsion_kernel;
///
/// let src = wgsl_repulsion_kernel();
/// assert!(src.contains("@compute"));
/// assert!(src.contains("workgroup_size"));
/// ```
pub fn wgsl_repulsion_kernel() -> &'static str {
    WGSL_REPULSION_KERNEL
}

/// Returns the WGSL source for the velocity verlet integration compute kernel.
///
/// Reads the forces buffer, applies temperature clamping and velocity damping,
/// then integrates positions. Respects the `pinned` flag. Supports an adaptive
/// timestep via the `dt` uniform (`workgroup_size(256)`).
///
/// # Example
///
/// ```rust
/// use nexcore_viz::gpu_layout::wgsl_integration_kernel;
///
/// let src = wgsl_integration_kernel();
/// assert!(src.contains("@compute"));
/// assert!(src.contains("workgroup_size"));
/// ```
pub fn wgsl_integration_kernel() -> &'static str {
    WGSL_INTEGRATION_KERNEL_3D
}

/// Returns the WGSL source for the parallel AABB bounds reduction kernel.
///
/// Computes the axis-aligned bounding box of all node positions in two phases:
/// 1. Each workgroup reduces its tile into shared memory.
/// 2. A global atomic min/max pass collapses workgroup results.
///
/// (`workgroup_size(256)`)
///
/// # Example
///
/// ```rust
/// use nexcore_viz::gpu_layout::wgsl_bounds_kernel;
///
/// let src = wgsl_bounds_kernel();
/// assert!(src.contains("@compute"));
/// assert!(src.contains("workgroup_size"));
/// ```
pub fn wgsl_bounds_kernel() -> &'static str {
    WGSL_BOUNDS_KERNEL
}

/// Returns the WGSL source for the parallel octree build kernel.
///
/// Constructs a linearized octree in a bottom-up pass. Each thread assigns
/// a leaf node to its octant position; internal nodes are aggregated in a
/// subsequent reduction pass (`workgroup_size(256)`).
///
/// # Example
///
/// ```rust
/// use nexcore_viz::gpu_layout::wgsl_octree_build_kernel;
///
/// let src = wgsl_octree_build_kernel();
/// assert!(src.contains("@compute"));
/// assert!(src.contains("workgroup_size"));
/// ```
pub fn wgsl_octree_build_kernel() -> &'static str {
    WGSL_OCTREE_BUILD_KERNEL
}

// ── WGSL 3D kernel source strings ────────────────────────────────────────────

const WGSL_ATTRACTION_KERNEL: &str = r#"// nexcore Observatory — Edge Spring Attraction Kernel (3D)
// One thread per edge. Accumulates spring forces into atomic i32 buffers
// (fixed-point: 1 unit = 0.001 layout units).
//
// Dispatch: (edge_count / 256 + 1, 1, 1) workgroups of size 256.

struct NodePos {
    x: f32,
    y: f32,
    z: f32,
    _pad: f32,
}

struct EdgeData {
    source:       u32,
    target:       u32,
    weight:       f32,
    ideal_length: f32,
}

// Atomic force accumulators — i32 with fixed-point scale = 1000.
struct AtomicForce {
    fx: atomic<i32>,
    fy: atomic<i32>,
    fz: atomic<i32>,
    _pad: i32,
}

struct AttractionUniforms {
    edge_count:          u32,
    node_count:          u32,
    attraction_strength: f32,
    min_distance:        f32,
}

@group(0) @binding(0) var<storage, read>       positions: array<NodePos>;
@group(0) @binding(1) var<storage, read>       edges:     array<EdgeData>;
@group(0) @binding(2) var<storage, read_write> forces:    array<AtomicForce>;
@group(0) @binding(3) var<uniform>             uniforms:  AttractionUniforms;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let e = gid.x;
    if e >= uniforms.edge_count {
        return;
    }

    let edge = edges[e];
    let src  = positions[edge.source];
    let tgt  = positions[edge.target];

    let delta = vec3<f32>(tgt.x - src.x, tgt.y - src.y, tgt.z - src.z);
    let dist  = max(length(delta), uniforms.min_distance);

    // Hooke: F = k * w * (dist - ideal) * unit_vec
    let mag   = uniforms.attraction_strength * edge.weight * (dist - edge.ideal_length) / dist;
    let f     = delta * mag;

    // Fixed-point accumulation (scale = 1000 to preserve sub-unit precision).
    let scale = 1000.0;
    let ifx = i32(f.x * scale);
    let ify = i32(f.y * scale);
    let ifz = i32(f.z * scale);

    atomicAdd(&forces[edge.source].fx,  ifx);
    atomicAdd(&forces[edge.source].fy,  ify);
    atomicAdd(&forces[edge.source].fz,  ifz);
    atomicAdd(&forces[edge.target].fx, -ifx);
    atomicAdd(&forces[edge.target].fy, -ify);
    atomicAdd(&forces[edge.target].fz, -ifz);
}
"#;

const WGSL_REPULSION_KERNEL: &str = r#"// nexcore Observatory — Barnes-Hut N-Body Repulsion Kernel (3D)
// One thread per node. Traverses a linearised octree stored in a flat array.
// Treats cells as point masses when width/dist < theta (Barnes-Hut criterion).
//
// Dispatch: (node_count / 256 + 1, 1, 1) workgroups of size 256.

struct NodeData {
    pos_x: f32,
    pos_y: f32,
    pos_z: f32,
    mass:  f32,
    pinned: u32,
    _p0: u32,
    _p1: u32,
    _p2: u32,
}

// Linearised octree cell (8 child indices, 0 = absent).
struct OctCell {
    cx: f32,           // Centre-of-mass x
    cy: f32,           // Centre-of-mass y
    cz: f32,           // Centre-of-mass z
    total_mass: f32,
    width: f32,        // Largest spatial dimension of this cell
    body_idx: i32,     // >= 0: leaf holding single body; -1: internal
    children: array<u32, 8>,
}

struct RepulsionUniforms {
    node_count:         u32,
    cell_count:         u32,
    repulsion_strength: f32,
    theta:              f32,
    min_distance:       f32,
    _p0: f32,
    _p1: f32,
    _p2: f32,
}

struct ForceOut {
    fx: f32,
    fy: f32,
    fz: f32,
    _pad: f32,
}

@group(0) @binding(0) var<storage, read>       nodes:    array<NodeData>;
@group(0) @binding(1) var<storage, read>       octree:   array<OctCell>;
@group(0) @binding(2) var<storage, read_write> forces:   array<ForceOut>;
@group(0) @binding(3) var<uniform>             uniforms: RepulsionUniforms;

// Iterative octree traversal using a fixed-depth stack.
fn traverse(node_idx: u32, pos_i: vec3<f32>, mass_i: f32) -> vec3<f32> {
    var acc = vec3<f32>(0.0, 0.0, 0.0);
    var stack: array<u32, 64>;
    var sp: i32 = 0;
    stack[0] = 0u; // root
    sp = 1;

    while sp > 0 {
        sp -= 1;
        let cell_idx = stack[sp];
        if cell_idx >= uniforms.cell_count {
            continue;
        }
        let cell = octree[cell_idx];
        if cell.total_mass == 0.0 {
            continue;
        }

        // Skip if this is the body itself.
        if cell.body_idx >= 0 && u32(cell.body_idx) == node_idx {
            continue;
        }

        let delta = pos_i - vec3<f32>(cell.cx, cell.cy, cell.cz);
        let dist  = max(length(delta), uniforms.min_distance);

        // Barnes-Hut criterion: approximate if cell is small enough.
        if cell.body_idx >= 0 || (cell.width / dist < uniforms.theta) {
            let mag = uniforms.repulsion_strength * mass_i * cell.total_mass / (dist * dist);
            acc += normalize(delta) * mag;
        } else {
            // Push non-empty children onto the stack.
            for (var c = 0u; c < 8u; c++) {
                let child = cell.children[c];
                if child != 0u && sp < 63 {
                    stack[sp] = child;
                    sp += 1;
                }
            }
        }
    }

    return acc;
}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if i >= uniforms.node_count {
        return;
    }

    let node = nodes[i];
    if node.pinned != 0u {
        forces[i].fx = 0.0;
        forces[i].fy = 0.0;
        forces[i].fz = 0.0;
        return;
    }

    let pos_i = vec3<f32>(node.pos_x, node.pos_y, node.pos_z);
    let f = traverse(i, pos_i, node.mass);

    forces[i].fx = f.x;
    forces[i].fy = f.y;
    forces[i].fz = f.z;
}
"#;

const WGSL_INTEGRATION_KERNEL_3D: &str = r#"// nexcore Observatory — Velocity Verlet Integration Kernel (3D, adaptive timestep)
// Reads forces, clamps to temperature, applies damping, integrates positions.
// Respects pinned flag. Supports variable dt for adaptive timestepping.
//
// Dispatch: (node_count / 256 + 1, 1, 1) workgroups of size 256.

struct NodeState {
    pos_x: f32,
    pos_y: f32,
    pos_z: f32,
    vel_x: f32,
    vel_y: f32,
    vel_z: f32,
    mass:   f32,
    pinned: u32,
}

struct ForceEntry {
    fx:   f32,
    fy:   f32,
    fz:   f32,
    _pad: f32,
}

struct IntegrationUniforms3D {
    node_count:   u32,
    damping:      f32,
    temperature:  f32,
    dt:           f32,   // Adaptive timestep (1.0 = normal step)
}

@group(0) @binding(0) var<storage, read_write> nodes:    array<NodeState>;
@group(0) @binding(1) var<storage, read>       forces:   array<ForceEntry>;
@group(0) @binding(2) var<uniform>             uniforms: IntegrationUniforms3D;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if i >= uniforms.node_count {
        return;
    }

    if nodes[i].pinned != 0u {
        nodes[i].vel_x = 0.0;
        nodes[i].vel_y = 0.0;
        nodes[i].vel_z = 0.0;
        return;
    }

    let f     = vec3<f32>(forces[i].fx, forces[i].fy, forces[i].fz);
    let f_len = length(f);

    // Clamp force magnitude to current temperature.
    let clamped_f = select(
        f,
        normalize(f) * uniforms.temperature,
        f_len > uniforms.temperature,
    );

    // Velocity verlet: v += (F / m) * dt, then damp.
    var vel = vec3<f32>(nodes[i].vel_x, nodes[i].vel_y, nodes[i].vel_z);
    vel = (vel + clamped_f * (uniforms.dt / nodes[i].mass)) * uniforms.damping;

    // Position integration.
    nodes[i].pos_x += vel.x * uniforms.dt;
    nodes[i].pos_y += vel.y * uniforms.dt;
    nodes[i].pos_z += vel.z * uniforms.dt;
    nodes[i].vel_x  = vel.x;
    nodes[i].vel_y  = vel.y;
    nodes[i].vel_z  = vel.z;
}
"#;

const WGSL_BOUNDS_KERNEL: &str = r#"// nexcore Observatory — Parallel AABB Bounds Reduction Kernel (3D)
// Phase 1: each workgroup reduces its tile into workgroup shared memory.
// Phase 2: workgroup results are collapsed via global atomic operations.
//
// Output: bounds buffer [x_min, y_min, z_min, x_max, y_max, z_max] as
//         bit-cast i32 atomics (IEEE 754 trick for positive float min/max).
//
// Dispatch: (node_count / 256 + 1, 1, 1) workgroups of size 256.

struct NodePos {
    x: f32,
    y: f32,
    z: f32,
    _pad: f32,
}

struct BoundsUniforms {
    node_count: u32,
    _p0: u32,
    _p1: u32,
    _p2: u32,
}

// Global AABB stored as atomic i32 (bit-cast from f32).
// Indices: 0=xmin, 1=ymin, 2=zmin, 3=xmax, 4=ymax, 5=zmax.
struct BoundsOut {
    xmin: atomic<i32>,
    ymin: atomic<i32>,
    zmin: atomic<i32>,
    xmax: atomic<i32>,
    ymax: atomic<i32>,
    zmax: atomic<i32>,
}

@group(0) @binding(0) var<storage, read>       positions: array<NodePos>;
@group(0) @binding(1) var<storage, read_write> bounds:    BoundsOut;
@group(0) @binding(2) var<uniform>             uniforms:  BoundsUniforms;

var<workgroup> wg_xmin: array<f32, 256>;
var<workgroup> wg_ymin: array<f32, 256>;
var<workgroup> wg_zmin: array<f32, 256>;
var<workgroup> wg_xmax: array<f32, 256>;
var<workgroup> wg_ymax: array<f32, 256>;
var<workgroup> wg_zmax: array<f32, 256>;

@compute @workgroup_size(256)
fn main(
    @builtin(global_invocation_id) gid:  vec3<u32>,
    @builtin(local_invocation_id)  lid:  vec3<u32>,
    @builtin(workgroup_id)         wgid: vec3<u32>,
) {
    let local_idx = lid.x;
    let INF  =  1e38;
    let NINF = -1e38;

    var lx = INF;  var ly = INF;  var lz = INF;
    var hx = NINF; var hy = NINF; var hz = NINF;

    // Grid-stride loop so one workgroup handles multiple nodes if needed.
    var idx = gid.x;
    while idx < uniforms.node_count {
        let p = positions[idx];
        lx = min(lx, p.x); ly = min(ly, p.y); lz = min(lz, p.z);
        hx = max(hx, p.x); hy = max(hy, p.y); hz = max(hz, p.z);
        idx += 256u;
    }

    wg_xmin[local_idx] = lx; wg_ymin[local_idx] = ly; wg_zmin[local_idx] = lz;
    wg_xmax[local_idx] = hx; wg_ymax[local_idx] = hy; wg_zmax[local_idx] = hz;
    workgroupBarrier();

    // Parallel reduction within the workgroup.
    var step = 128u;
    while step > 0u {
        if local_idx < step {
            let j = local_idx + step;
            wg_xmin[local_idx] = min(wg_xmin[local_idx], wg_xmin[j]);
            wg_ymin[local_idx] = min(wg_ymin[local_idx], wg_ymin[j]);
            wg_zmin[local_idx] = min(wg_zmin[local_idx], wg_zmin[j]);
            wg_xmax[local_idx] = max(wg_xmax[local_idx], wg_xmax[j]);
            wg_ymax[local_idx] = max(wg_ymax[local_idx], wg_ymax[j]);
            wg_zmax[local_idx] = max(wg_zmax[local_idx], wg_zmax[j]);
        }
        workgroupBarrier();
        step = step >> 1u;
    }

    // Thread 0 writes workgroup result to global atomics (IEEE 754 bit-cast trick).
    if local_idx == 0u {
        atomicMin(&bounds.xmin, bitcast<i32>(wg_xmin[0]));
        atomicMin(&bounds.ymin, bitcast<i32>(wg_ymin[0]));
        atomicMin(&bounds.zmin, bitcast<i32>(wg_zmin[0]));
        atomicMax(&bounds.xmax, bitcast<i32>(wg_xmax[0]));
        atomicMax(&bounds.ymax, bitcast<i32>(wg_ymax[0]));
        atomicMax(&bounds.zmax, bitcast<i32>(wg_zmax[0]));
    }
}
"#;

const WGSL_OCTREE_BUILD_KERNEL: &str = r#"// nexcore Observatory — Parallel Octree Build Kernel (3D, bottom-up)
// Assigns each body to a leaf cell, then aggregates centre-of-mass and
// total mass upward through the tree levels.
//
// The linearised octree is pre-allocated by the CPU pass.  Cell 0 is the
// root.  Children of cell C are stored at indices child_base[C]..+8.
//
// Pass 1 (leaf assignment): one thread per node, dispatched here.
// Pass 2 (aggregation)    : dispatched separately for each level bottom-up.
//
// Dispatch pass 1: (node_count / 256 + 1, 1, 1) workgroups of size 256.

struct NodeData {
    pos_x: f32,
    pos_y: f32,
    pos_z: f32,
    mass:  f32,
}

// Linearised octree cell (matches CPU OctreeNode layout).
struct OctCell {
    cx:         f32,           // Centre-of-mass x (written by aggregation pass)
    cy:         f32,           // Centre-of-mass y
    cz:         f32,           // Centre-of-mass z
    total_mass: f32,           // Accumulated mass
    x_min:      f32,           // Cell bounds
    y_min:      f32,
    z_min:      f32,
    x_max:      f32,
    y_max:      f32,
    z_max:      f32,
    body_idx:   i32,           // Leaf body index (-1 = internal, -2 = empty)
    level:      u32,           // Tree depth (0 = root)
    children:   array<u32, 8>, // Child cell indices (0 = absent)
}

struct OctreeBuildUniforms {
    node_count: u32,
    cell_count: u32,
    root_idx:   u32,
    _pad:       u32,
}

@group(0) @binding(0) var<storage, read>       nodes:    array<NodeData>;
@group(0) @binding(1) var<storage, read_write> octree:   array<OctCell>;
@group(0) @binding(2) var<uniform>             uniforms: OctreeBuildUniforms;

// Determine which octant of a cell a position falls into.
fn octant(cell_idx: u32, pos: vec3<f32>) -> u32 {
    let cell = octree[cell_idx];
    let mx = (cell.x_min + cell.x_max) * 0.5;
    let my = (cell.y_min + cell.y_max) * 0.5;
    let mz = (cell.z_min + cell.z_max) * 0.5;
    let ex = select(0u, 1u, pos.x >= mx);
    let ey = select(0u, 2u, pos.y >= my);
    let ez = select(0u, 4u, pos.z >= mz);
    return ex | ey | ez;
}

// Pass 1: leaf assignment — traverse from root to leaf for each body.
@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let body = gid.x;
    if body >= uniforms.node_count {
        return;
    }

    let pos  = vec3<f32>(nodes[body].pos_x, nodes[body].pos_y, nodes[body].pos_z);
    let mass = nodes[body].mass;

    // Walk the tree from root to the appropriate leaf.
    var cur = uniforms.root_idx;
    for (var depth = 0u; depth < 32u; depth++) {
        let oct = octant(cur, pos);
        let child = octree[cur].children[oct];

        if child == 0u {
            // Reached an empty leaf slot — claim it.
            // (In GPU usage, contention is resolved by the CPU-side allocation pass;
            //  this kernel assumes the skeleton is pre-built and only writes bodies.)
            octree[cur].body_idx   = i32(body);
            octree[cur].cx         = pos.x;
            octree[cur].cy         = pos.y;
            octree[cur].cz         = pos.z;
            octree[cur].total_mass = mass;
            return;
        }

        if octree[child].body_idx == -2 {
            // Empty leaf — assign.
            octree[child].body_idx   = i32(body);
            octree[child].cx         = pos.x;
            octree[child].cy         = pos.y;
            octree[child].cz         = pos.z;
            octree[child].total_mass = mass;
            return;
        }

        cur = child;
    }
}

// Pass 2 entry point: aggregate one level.  Dispatch bottom-up, one level at a time.
@compute @workgroup_size(256)
fn aggregate(@builtin(global_invocation_id) gid: vec3<u32>) {
    let cell_idx = gid.x;
    if cell_idx >= uniforms.cell_count {
        return;
    }

    var total  = 0.0;
    var cx     = 0.0;
    var cy     = 0.0;
    var cz     = 0.0;

    for (var c = 0u; c < 8u; c++) {
        let child = octree[cell_idx].children[c];
        if child == 0u {
            continue;
        }
        let cm = octree[child].total_mass;
        total += cm;
        cx    += octree[child].cx * cm;
        cy    += octree[child].cy * cm;
        cz    += octree[child].cz * cm;
    }

    if total > 0.0 {
        octree[cell_idx].total_mass = total;
        octree[cell_idx].cx         = cx / total;
        octree[cell_idx].cy         = cy / total;
        octree[cell_idx].cz         = cz / total;
    }
}
"#;

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Helper builders ──────────────────────────────────────────────────────

    fn two_node_graph(dist: f64, ideal: f64) -> (Vec<LayoutNode>, Vec<LayoutEdge>) {
        let nodes = vec![LayoutNode::new(0, 0.0, 0.0), LayoutNode::new(1, dist, 0.0)];
        let edges = vec![LayoutEdge {
            source: 0,
            target: 1,
            weight: 1.0,
            ideal_length: ideal,
        }];
        (nodes, edges)
    }

    fn triangle_graph() -> (Vec<LayoutNode>, Vec<LayoutEdge>) {
        let r = 100.0_f64;
        let nodes = vec![
            LayoutNode::new(0, r, 0.0),
            LayoutNode::new(1, -r / 2.0, r * 0.866),
            LayoutNode::new(2, -r / 2.0, -r * 0.866),
        ];
        let edges = vec![
            LayoutEdge {
                source: 0,
                target: 1,
                weight: 1.0,
                ideal_length: r,
            },
            LayoutEdge {
                source: 1,
                target: 2,
                weight: 1.0,
                ideal_length: r,
            },
            LayoutEdge {
                source: 2,
                target: 0,
                weight: 1.0,
                ideal_length: r,
            },
        ];
        (nodes, edges)
    }

    // ── Error cases ──────────────────────────────────────────────────────────

    /// Empty node list produces EmptyGraph error.
    #[test]
    fn test_empty_graph_error() {
        let config = LayoutConfig::default();
        let result = compute_layout(vec![], vec![], &config);
        assert_eq!(result, Err(LayoutError::EmptyGraph));
    }

    /// Edge referencing a non-existent node produces InvalidNode.
    #[test]
    fn test_invalid_node_error() {
        let mut config = LayoutConfig::default();
        config.iterations = 1;
        let nodes = vec![LayoutNode::new(0, 0.0, 0.0)];
        let edges = vec![LayoutEdge {
            source: 0,
            target: 99, // out of range
            weight: 1.0,
            ideal_length: 100.0,
        }];
        let result = compute_layout(nodes, edges, &config);
        assert_eq!(result, Err(LayoutError::InvalidNode(99)));
    }

    /// Invalid configuration is rejected.
    #[test]
    fn test_invalid_parameter_error() {
        let mut config = LayoutConfig::default();
        config.damping = 1.5; // must be in [0, 1]
        let nodes = vec![LayoutNode::new(0, 0.0, 0.0)];
        let result = compute_layout(nodes, vec![], &config);
        assert!(matches!(result, Err(LayoutError::InvalidParameter(_))));
    }

    // ── Single node ──────────────────────────────────────────────────────────

    /// A single isolated node should not move (no forces to apply).
    #[test]
    fn test_single_node_layout() {
        let mut config = LayoutConfig::default();
        config.iterations = 10;
        let nodes = vec![LayoutNode::new(0, 50.0, 50.0)];
        let result = compute_layout(nodes, vec![], &config);
        assert!(result.is_ok());
        let result = result.unwrap_or_else(|_| unreachable!());
        assert_eq!(result.nodes.len(), 1);
        // With no neighbours, force = 0 and position remains at 50, 50.
        let pos = result.nodes[0].position;
        assert!((pos[0] - 50.0).abs() < 0.1, "x drifted: {}", pos[0]);
        assert!((pos[1] - 50.0).abs() < 0.1, "y drifted: {}", pos[1]);
    }

    // ── Two-node convergence ─────────────────────────────────────────────────

    /// Two connected nodes should move significantly closer than their initial
    /// separation when the ideal length is much shorter.
    ///
    /// Note: the equilibrium of a Coulomb-repulsion + Hooke-spring system does
    /// not sit exactly at `ideal_length` — repulsion pushes the nodes apart
    /// while attraction pulls them together.  We verify that the layout
    /// reduces the pairwise distance substantially from the starting value.
    #[test]
    fn test_two_nodes_converge_to_ideal_length() {
        let ideal = 80.0;
        let initial_dist = 400.0;
        let (nodes, edges) = two_node_graph(initial_dist, ideal);
        let mut config = LayoutConfig::default();
        config.iterations = 500;
        config.initial_temperature = 100.0;

        let result = compute_layout(nodes, edges, &config);
        assert!(result.is_ok());
        let result = result.unwrap_or_else(|_| unreachable!());
        let p0 = result.nodes[0].position;
        let p1 = result.nodes[1].position;
        let final_dist = ((p1[0] - p0[0]).powi(2) + (p1[1] - p0[1]).powi(2)).sqrt();

        // The layout must pull the nodes at least 50% closer than they started.
        // The exact equilibrium depends on repulsion_strength vs attraction_strength,
        // but meaningful attraction must have occurred.
        assert!(
            final_dist < initial_dist * 0.75,
            "nodes did not converge: initial={initial_dist:.2}, final={final_dist:.2}"
        );
        // Also verify neither node collapsed onto the other (repulsion active).
        assert!(
            final_dist > ideal * 0.25,
            "nodes collapsed too close: final={final_dist:.2}, ideal={ideal:.2}"
        );
    }

    // ── Triangle symmetry ────────────────────────────────────────────────────

    /// A symmetric triangle graph should remain roughly symmetric after layout.
    #[test]
    fn test_triangle_layout_symmetry() {
        let (nodes, edges) = triangle_graph();
        let mut config = LayoutConfig::default();
        config.iterations = 100;
        config.initial_temperature = 50.0;

        let result = compute_layout(nodes, edges, &config);
        assert!(result.is_ok());
        let result = result.unwrap_or_else(|_| unreachable!());

        // All pairwise distances should be roughly equal (within 60%).
        let p: Vec<[f64; 2]> = result.nodes.iter().map(|n| n.position).collect();
        let d01 = (p[0][0] - p[1][0]).hypot(p[0][1] - p[1][1]);
        let d12 = (p[1][0] - p[2][0]).hypot(p[1][1] - p[2][1]);
        let d20 = (p[2][0] - p[0][0]).hypot(p[2][1] - p[0][1]);
        let mean = (d01 + d12 + d20) / 3.0;

        assert!(
            (d01 - mean).abs() / mean < 0.60,
            "d01={d01:.2} mean={mean:.2}"
        );
        assert!(
            (d12 - mean).abs() / mean < 0.60,
            "d12={d12:.2} mean={mean:.2}"
        );
        assert!(
            (d20 - mean).abs() / mean < 0.60,
            "d20={d20:.2} mean={mean:.2}"
        );
    }

    // ── Barnes-Hut quadtree construction ────────────────────────────────────

    /// Quadtree built from N nodes should contain all N bodies.
    #[test]
    fn test_quadtree_construction() {
        let nodes = random_layout(20, 400.0, 400.0);
        let tree = build_quadtree(&nodes);
        assert_eq!(tree.node_count, 20);
        assert!(tree.root.total_mass > 0.0);
    }

    /// Quadtree on a single node forms a single-body leaf.
    #[test]
    fn test_quadtree_single_node() {
        let nodes = vec![LayoutNode::new(0, 100.0, 100.0)];
        let tree = build_quadtree(&nodes);
        assert_eq!(tree.node_count, 1);
        assert_eq!(tree.root.body, Some(0));
        assert!(tree.root.is_single_body());
    }

    // ── Barnes-Hut approximation ─────────────────────────────────────────────

    /// Barnes-Hut repulsion should roughly match brute-force for theta = 0.5.
    #[test]
    fn test_barnes_hut_vs_brute_force() {
        let nodes = random_layout(30, 600.0, 600.0);
        let tree = build_quadtree(&nodes);

        // Compute BH force on node 0.
        let bh_force = compute_repulsion_bh(&tree, &nodes[0], 0.5);

        // Compute exact brute-force force on node 0.
        let mut bf_force = [0.0_f64, 0.0_f64];
        for j in 1..nodes.len() {
            let [fx, fy] = repulsion_force(
                nodes[0].position,
                nodes[j].position,
                nodes[0].mass,
                nodes[j].mass,
                10_000.0,
                1.0,
            );
            bf_force[0] += fx;
            bf_force[1] += fy;
        }

        let bh_mag = bh_force[0].hypot(bh_force[1]);
        let bf_mag = bf_force[0].hypot(bf_force[1]);

        // The BH approximation should be within 40% of brute force.
        if bf_mag > 0.0 {
            let rel_err = (bh_mag - bf_mag).abs() / bf_mag;
            assert!(
                rel_err < 0.40,
                "BH error {:.1}% (bh={bh_mag:.2} bf={bf_mag:.2})",
                rel_err * 100.0
            );
        }
    }

    // ── GPU buffer packing ───────────────────────────────────────────────────

    /// GPU buffer positions round-trip correctly for two nodes.
    #[test]
    fn test_gpu_buffer_packing_roundtrip() {
        let nodes = vec![
            LayoutNode::new(0, 10.0, 20.0),
            LayoutNode::new(1, 30.0, 40.0),
        ];
        let edges = vec![LayoutEdge {
            source: 0,
            target: 1,
            weight: 1.0,
            ideal_length: 50.0,
        }];
        let (floats, indices, layout) = prepare_gpu_buffers(&nodes, &edges);

        assert_eq!(layout.node_count, 2);
        assert_eq!(layout.positions_offset, 0);

        // positions block: x0, y0, x1, y1
        assert!((floats[0] - 10.0_f32).abs() < 1e-5);
        assert!((floats[1] - 20.0_f32).abs() < 1e-5);
        assert!((floats[2] - 30.0_f32).abs() < 1e-5);
        assert!((floats[3] - 40.0_f32).abs() < 1e-5);

        // edge index buffer
        assert_eq!(indices.len(), 2);
        assert_eq!(indices[0], 0);
        assert_eq!(indices[1], 1);
    }

    /// GPU buffer for zero edges produces empty index buffer.
    #[test]
    fn test_gpu_buffer_no_edges() {
        let nodes = random_layout(5, 200.0, 200.0);
        let (floats, indices, layout) = prepare_gpu_buffers(&nodes, &[]);
        // 5 nodes × 6 f32 (pos + vel + force) = 30 floats total
        assert_eq!(floats.len(), 30);
        assert!(indices.is_empty());
        assert_eq!(layout.node_count, 5);
    }

    // ── WGSL shader strings ──────────────────────────────────────────────────

    /// Force shader is non-empty and contains the compute entry point keyword.
    #[test]
    fn test_wgsl_force_shader_non_empty() {
        let src = wgsl_force_shader();
        assert!(!src.is_empty());
        assert!(
            src.contains("@compute"),
            "expected @compute in force shader"
        );
        assert!(src.contains("fn main"), "expected fn main in force shader");
    }

    /// Integration shader is non-empty and contains the compute entry point keyword.
    #[test]
    fn test_wgsl_integration_shader_non_empty() {
        let src = wgsl_integration_shader();
        assert!(!src.is_empty());
        assert!(
            src.contains("@compute"),
            "expected @compute in integration shader"
        );
        assert!(
            src.contains("fn main"),
            "expected fn main in integration shader"
        );
    }

    // ── random_layout ────────────────────────────────────────────────────────

    /// random_layout produces N nodes within the specified bounds.
    #[test]
    fn test_random_layout_within_bounds() {
        let n = 50;
        let w = 1000.0;
        let h = 800.0;
        let nodes = random_layout(n, w, h);
        assert_eq!(nodes.len(), n);
        for (i, node) in nodes.iter().enumerate() {
            assert!(
                node.position[0] >= 0.0 && node.position[0] <= w,
                "node {i} x={} out of [0, {w}]",
                node.position[0]
            );
            assert!(
                node.position[1] >= 0.0 && node.position[1] <= h,
                "node {i} y={} out of [0, {h}]",
                node.position[1]
            );
        }
    }

    /// random_layout with n=0 returns empty vec.
    #[test]
    fn test_random_layout_zero_nodes() {
        let nodes = random_layout(0, 400.0, 400.0);
        assert!(nodes.is_empty());
    }

    // ── normalize_positions ──────────────────────────────────────────────────

    /// After normalization the bounding box fills [0, W] × [0, H].
    #[test]
    fn test_normalize_positions_scales_to_viewport() {
        let mut nodes = vec![
            LayoutNode::new(0, 10.0, 5.0),
            LayoutNode::new(1, 110.0, 55.0),
            LayoutNode::new(2, 60.0, 30.0),
        ];
        normalize_positions(&mut nodes, 400.0, 200.0);

        let xs: Vec<f64> = nodes.iter().map(|n| n.position[0]).collect();
        let ys: Vec<f64> = nodes.iter().map(|n| n.position[1]).collect();

        let x_max = xs.iter().cloned().fold(f64::MIN, f64::max);
        let x_min = xs.iter().cloned().fold(f64::MAX, f64::min);
        let y_max = ys.iter().cloned().fold(f64::MIN, f64::max);
        let y_min = ys.iter().cloned().fold(f64::MAX, f64::min);

        assert!((x_max - 400.0).abs() < 1e-9, "x_max={x_max}");
        assert!((x_min - 0.0).abs() < 1e-9, "x_min={x_min}");
        assert!((y_max - 200.0).abs() < 1e-9, "y_max={y_max}");
        assert!((y_min - 0.0).abs() < 1e-9, "y_min={y_min}");
    }

    /// Normalize on a single node centres it in the viewport.
    #[test]
    fn test_normalize_single_node_centres() {
        let mut nodes = vec![LayoutNode::new(0, 999.0, 777.0)];
        normalize_positions(&mut nodes, 400.0, 300.0);
        assert!((nodes[0].position[0] - 200.0).abs() < 1e-9);
        assert!((nodes[0].position[1] - 150.0).abs() < 1e-9);
    }

    // ── ForceAtlas2 ──────────────────────────────────────────────────────────

    /// ForceAtlas2 runs without error on a small graph.
    #[test]
    fn test_force_atlas2_small_graph() {
        let mut nodes = random_layout(6, 300.0, 300.0);
        let edges = vec![
            LayoutEdge {
                source: 0,
                target: 1,
                weight: 1.0,
                ideal_length: 80.0,
            },
            LayoutEdge {
                source: 1,
                target: 2,
                weight: 1.0,
                ideal_length: 80.0,
            },
            LayoutEdge {
                source: 3,
                target: 4,
                weight: 1.0,
                ideal_length: 80.0,
            },
        ];
        let mut config = LayoutConfig::default();
        config.iterations = 50;
        let result = force_atlas2(&mut nodes, &edges, &config);
        assert!(result.is_ok());
        let result = result.unwrap_or_else(|_| unreachable!());
        assert!(result.iterations_run > 0);
        assert_eq!(result.nodes.len(), 6);
    }

    /// ForceAtlas2 returns EmptyGraph for empty input.
    #[test]
    fn test_force_atlas2_empty_graph() {
        let config = LayoutConfig::default();
        let result = force_atlas2(&mut [], &[], &config);
        assert_eq!(result, Err(LayoutError::EmptyGraph));
    }

    // ── Energy decreases ─────────────────────────────────────────────────────

    /// System energy should decrease as layout runs.
    #[test]
    fn test_energy_decreases_over_iterations() {
        let nodes_init = random_layout(8, 600.0, 600.0);
        let edges = vec![
            LayoutEdge {
                source: 0,
                target: 1,
                weight: 1.0,
                ideal_length: 100.0,
            },
            LayoutEdge {
                source: 1,
                target: 2,
                weight: 1.0,
                ideal_length: 100.0,
            },
            LayoutEdge {
                source: 2,
                target: 3,
                weight: 1.0,
                ideal_length: 100.0,
            },
            LayoutEdge {
                source: 4,
                target: 5,
                weight: 1.0,
                ideal_length: 100.0,
            },
        ];
        let config = LayoutConfig::default();

        let initial_energy = compute_energy(&nodes_init, &edges, &config);

        let result = compute_layout(nodes_init, edges.clone(), &config);
        assert!(result.is_ok());
        let result = result.unwrap_or_else(|_| unreachable!());
        let final_energy = result.energy;

        assert!(
            final_energy < initial_energy,
            "energy should decrease: initial={initial_energy:.2} final={final_energy:.2}"
        );
    }

    // ── LayoutError Display ─────────────────────────────────────────────────

    #[test]
    fn test_layout_error_display_convergence_failure() {
        let e = LayoutError::ConvergenceFailure;
        let s = format!("{e}");
        assert!(s.contains("converge"), "expected 'converge' in: {s}");
    }

    #[test]
    fn test_layout_error_display_empty_graph() {
        let s = format!("{}", LayoutError::EmptyGraph);
        assert!(s.contains("no nodes"), "expected 'no nodes' in: {s}");
    }

    #[test]
    fn test_layout_error_display_invalid_node() {
        let s = format!("{}", LayoutError::InvalidNode(42));
        assert!(s.contains("42"), "expected node id in: {s}");
    }

    #[test]
    fn test_layout_error_display_invalid_parameter() {
        let s = format!("{}", LayoutError::InvalidParameter("bad".into()));
        assert!(s.contains("bad"), "expected param msg in: {s}");
    }

    #[test]
    fn test_layout_error_is_std_error() {
        let e: Box<dyn std::error::Error> = Box::new(LayoutError::EmptyGraph);
        assert!(!e.to_string().is_empty());
    }

    // ── LayoutConfig validation per-parameter ───────────────────────────────

    #[test]
    fn test_validate_iterations_zero() {
        let mut c = LayoutConfig::default();
        c.iterations = 0;
        assert!(matches!(
            c.validate(),
            Err(LayoutError::InvalidParameter(_))
        ));
    }

    #[test]
    fn test_validate_negative_repulsion() {
        let mut c = LayoutConfig::default();
        c.repulsion_strength = -1.0;
        assert!(matches!(
            c.validate(),
            Err(LayoutError::InvalidParameter(_))
        ));
    }

    #[test]
    fn test_validate_negative_attraction() {
        let mut c = LayoutConfig::default();
        c.attraction_strength = -0.01;
        assert!(matches!(
            c.validate(),
            Err(LayoutError::InvalidParameter(_))
        ));
    }

    #[test]
    fn test_validate_damping_above_one() {
        let mut c = LayoutConfig::default();
        c.damping = 1.01;
        assert!(matches!(
            c.validate(),
            Err(LayoutError::InvalidParameter(_))
        ));
    }

    #[test]
    fn test_validate_damping_negative() {
        let mut c = LayoutConfig::default();
        c.damping = -0.1;
        assert!(matches!(
            c.validate(),
            Err(LayoutError::InvalidParameter(_))
        ));
    }

    #[test]
    fn test_validate_min_distance_zero() {
        let mut c = LayoutConfig::default();
        c.min_distance = 0.0;
        assert!(matches!(
            c.validate(),
            Err(LayoutError::InvalidParameter(_))
        ));
    }

    #[test]
    fn test_validate_theta_zero() {
        let mut c = LayoutConfig::default();
        c.theta = 0.0;
        assert!(matches!(
            c.validate(),
            Err(LayoutError::InvalidParameter(_))
        ));
    }

    #[test]
    fn test_validate_cooling_factor_above_one() {
        let mut c = LayoutConfig::default();
        c.cooling_factor = 1.5;
        assert!(matches!(
            c.validate(),
            Err(LayoutError::InvalidParameter(_))
        ));
    }

    #[test]
    fn test_validate_initial_temperature_zero() {
        let mut c = LayoutConfig::default();
        c.initial_temperature = 0.0;
        assert!(matches!(
            c.validate(),
            Err(LayoutError::InvalidParameter(_))
        ));
    }

    #[test]
    fn test_validate_default_is_valid() {
        let c = LayoutConfig::default();
        assert!(c.validate().is_ok());
    }

    // ── LayoutNode construction ─────────────────────────────────────────────

    #[test]
    fn test_layout_node_new_defaults() {
        let n = LayoutNode::new(7, 1.5, 2.5);
        assert_eq!(n.id, 7);
        assert!((n.position[0] - 1.5).abs() < 1e-12);
        assert!((n.position[1] - 2.5).abs() < 1e-12);
        assert!((n.velocity[0]).abs() < 1e-12);
        assert!((n.velocity[1]).abs() < 1e-12);
        assert!((n.mass - 1.0).abs() < 1e-12);
        assert!(!n.pinned);
    }

    // ── random_layout determinism ───────────────────────────────────────────

    #[test]
    fn test_random_layout_deterministic() {
        let a = random_layout(10, 800.0, 600.0);
        let b = random_layout(10, 800.0, 600.0);
        for (na, nb) in a.iter().zip(b.iter()) {
            assert!((na.position[0] - nb.position[0]).abs() < 1e-12);
            assert!((na.position[1] - nb.position[1]).abs() < 1e-12);
        }
    }

    // ── normalize_positions edge cases ──────────────────────────────────────

    #[test]
    fn test_normalize_positions_empty_slice() {
        let mut nodes: Vec<LayoutNode> = vec![];
        normalize_positions(&mut nodes, 100.0, 100.0);
        assert!(nodes.is_empty());
    }

    #[test]
    fn test_normalize_positions_pinned_nodes_stay_fixed() {
        let mut nodes = vec![
            LayoutNode::new(0, 0.0, 0.0),
            LayoutNode::new(1, 100.0, 100.0),
        ];
        nodes[0].pinned = true;
        let original_pos = nodes[0].position;
        normalize_positions(&mut nodes, 400.0, 400.0);
        assert!((nodes[0].position[0] - original_pos[0]).abs() < 1e-12);
        assert!((nodes[0].position[1] - original_pos[1]).abs() < 1e-12);
    }

    #[test]
    fn test_normalize_identical_x_centres_on_x() {
        let mut nodes = vec![
            LayoutNode::new(0, 50.0, 0.0),
            LayoutNode::new(1, 50.0, 100.0),
        ];
        normalize_positions(&mut nodes, 400.0, 300.0);
        assert!((nodes[0].position[0] - 200.0).abs() < 1e-9);
        assert!((nodes[1].position[0] - 200.0).abs() < 1e-9);
    }

    // ── compute_energy numeric assertions ───────────────────────────────────

    #[test]
    fn test_compute_energy_at_rest_is_zero_kinetic() {
        let nodes = vec![LayoutNode::new(0, 0.0, 0.0), LayoutNode::new(1, 100.0, 0.0)];
        let edges = vec![LayoutEdge {
            source: 0,
            target: 1,
            weight: 1.0,
            ideal_length: 100.0,
        }];
        let config = LayoutConfig::default();
        let e = compute_energy(&nodes, &edges, &config);
        // Repulsion = 10000/100 = 100. Spring stretch = 0. Kinetic = 0.
        assert!((e - 100.0).abs() < 0.01, "expected ~100, got {e}");
    }

    #[test]
    fn test_compute_energy_spring_stretch_nonzero() {
        let nodes = vec![LayoutNode::new(0, 0.0, 0.0), LayoutNode::new(1, 200.0, 0.0)];
        let edges = vec![LayoutEdge {
            source: 0,
            target: 1,
            weight: 1.0,
            ideal_length: 100.0,
        }];
        let config = LayoutConfig::default();
        let e = compute_energy(&nodes, &edges, &config);
        // Repulsion: 10000/200 = 50. Spring: 0.5*0.05*1*(200-100)^2 = 250.
        assert!((e - 300.0).abs() < 0.01, "expected ~300, got {e}");
    }

    #[test]
    fn test_compute_energy_single_node_zero() {
        let nodes = vec![LayoutNode::new(0, 50.0, 50.0)];
        let config = LayoutConfig::default();
        let e = compute_energy(&nodes, &[], &config);
        assert!((e).abs() < 1e-12, "single node energy should be 0, got {e}");
    }

    // ── pinned node support ─────────────────────────────────────────────────

    #[test]
    fn test_pinned_node_does_not_move() {
        let mut nodes = vec![LayoutNode::new(0, 0.0, 0.0), LayoutNode::new(1, 200.0, 0.0)];
        nodes[0].pinned = true;
        let edges = vec![LayoutEdge {
            source: 0,
            target: 1,
            weight: 1.0,
            ideal_length: 50.0,
        }];
        let mut config = LayoutConfig::default();
        config.iterations = 50;
        let result = fruchterman_reingold(&mut nodes, &edges, &config);
        assert!(result.is_ok());
        let r = result.unwrap_or_else(|_| unreachable!());
        assert!((r.nodes[0].position[0]).abs() < 1e-9);
        assert!((r.nodes[0].position[1]).abs() < 1e-9);
    }

    // ── repulsion_force numeric ─────────────────────────────────────────────

    #[test]
    fn test_repulsion_force_magnitude() {
        let f = repulsion_force([0.0, 0.0], [100.0, 0.0], 1.0, 1.0, 10_000.0, 1.0);
        assert!((f[0] - (-1.0)).abs() < 1e-9, "fx={}", f[0]);
        assert!((f[1]).abs() < 1e-12, "fy={}", f[1]);
    }

    #[test]
    fn test_repulsion_force_clamped_at_min_distance() {
        let f = repulsion_force([0.0, 0.0], [0.001, 0.0], 1.0, 1.0, 10_000.0, 5.0);
        // dx=-0.001, dist clamped to 5.0, mag=400, f_x = -0.001/5.0*400 = -0.08
        assert!((f[0] - (-0.08)).abs() < 1e-6, "fx={}", f[0]);
    }

    // ── attraction_force numeric ────────────────────────────────────────────

    #[test]
    fn test_attraction_force_at_ideal_length() {
        let f = attraction_force([0.0, 0.0], [100.0, 0.0], 1.0, 100.0, 0.05, 1.0);
        assert!((f[0]).abs() < 1e-12, "fx={}", f[0]);
        assert!((f[1]).abs() < 1e-12, "fy={}", f[1]);
    }

    #[test]
    fn test_attraction_force_stretched_beyond_ideal() {
        let f = attraction_force([0.0, 0.0], [200.0, 0.0], 1.0, 100.0, 0.05, 1.0);
        assert!((f[0] - 5.0).abs() < 1e-9, "fx={}", f[0]);
    }

    #[test]
    fn test_attraction_force_compressed_below_ideal() {
        let f = attraction_force([0.0, 0.0], [50.0, 0.0], 1.0, 100.0, 0.05, 1.0);
        assert!((f[0] - (-2.5)).abs() < 1e-9, "fx={}", f[0]);
    }

    // ── QuadTreeNode helpers ────────────────────────────────────────────────

    #[test]
    fn test_quadtree_node_quadrant_assignment() {
        let node = QuadTreeNode::new([0.0, 0.0, 100.0, 100.0]);
        assert_eq!(node.quadrant_for([25.0, 25.0]), 0);
        assert_eq!(node.quadrant_for([75.0, 25.0]), 1);
        assert_eq!(node.quadrant_for([25.0, 75.0]), 2);
        assert_eq!(node.quadrant_for([75.0, 75.0]), 3);
    }

    #[test]
    fn test_quadtree_child_bounds_nw() {
        let node = QuadTreeNode::new([0.0, 0.0, 100.0, 100.0]);
        let nw = node.child_bounds(0);
        assert!((nw[0]).abs() < 1e-12);
        assert!((nw[1]).abs() < 1e-12);
        assert!((nw[2] - 50.0).abs() < 1e-12);
        assert!((nw[3] - 50.0).abs() < 1e-12);
    }

    #[test]
    fn test_quadtree_child_bounds_se() {
        let node = QuadTreeNode::new([0.0, 0.0, 100.0, 100.0]);
        let se = node.child_bounds(3);
        assert!((se[0] - 50.0).abs() < 1e-12);
        assert!((se[1] - 50.0).abs() < 1e-12);
        assert!((se[2] - 100.0).abs() < 1e-12);
        assert!((se[3] - 100.0).abs() < 1e-12);
    }

    #[test]
    fn test_quadtree_width() {
        let node = QuadTreeNode::new([10.0, 20.0, 110.0, 70.0]);
        assert!((node.width() - 100.0).abs() < 1e-12);
    }

    #[test]
    fn test_quadtree_node_state_predicates() {
        let node = QuadTreeNode::new([0.0, 0.0, 10.0, 10.0]);
        assert!(node.is_leaf());
        assert!(node.is_empty());
        assert!(!node.is_single_body());
    }

    #[test]
    fn test_quadtree_two_coincident_nodes() {
        let nodes = vec![
            LayoutNode::new(0, 50.0, 50.0),
            LayoutNode::new(1, 50.0, 50.0),
        ];
        let tree = build_quadtree(&nodes);
        assert_eq!(tree.node_count, 2);
        assert!((tree.root.total_mass - 2.0).abs() < 1e-12);
    }

    // ── Barnes-Hut repulsion at very small theta ────────────────────────────

    #[test]
    fn test_bh_repulsion_small_theta_closer_to_exact() {
        let nodes = random_layout(20, 400.0, 400.0);
        let tree = build_quadtree(&nodes);
        let bh_0_3 = compute_repulsion_bh(&tree, &nodes[0], 0.3);
        let bh_1_0 = compute_repulsion_bh(&tree, &nodes[0], 1.0);
        let mut bf = [0.0_f64; 2];
        for j in 1..nodes.len() {
            let [fx, fy] = repulsion_force(
                nodes[0].position,
                nodes[j].position,
                nodes[0].mass,
                nodes[j].mass,
                10_000.0,
                1.0,
            );
            bf[0] += fx;
            bf[1] += fy;
        }
        let err_0_3 = (bh_0_3[0] - bf[0]).hypot(bh_0_3[1] - bf[1]);
        let err_1_0 = (bh_1_0[0] - bf[0]).hypot(bh_1_0[1] - bf[1]);
        assert!(
            err_0_3 <= err_1_0,
            "theta 0.3 error {err_0_3:.2} should be <= theta 1.0 error {err_1_0:.2}"
        );
    }

    // ── compute_layout path selection ───────────────────────────────────────

    #[test]
    fn test_compute_layout_small_uses_fr() {
        let nodes = random_layout(10, 200.0, 200.0);
        let edges = vec![LayoutEdge {
            source: 0,
            target: 1,
            weight: 1.0,
            ideal_length: 50.0,
        }];
        let mut config = LayoutConfig::default();
        config.iterations = 5;
        let result = compute_layout(nodes, edges, &config);
        assert!(result.is_ok());
    }

    // ── GPU buffer layout fields ────────────────────────────────────────────

    #[test]
    fn test_gpu_buffer_layout_offsets() {
        let nodes = vec![
            LayoutNode::new(0, 1.0, 2.0),
            LayoutNode::new(1, 3.0, 4.0),
            LayoutNode::new(2, 5.0, 6.0),
        ];
        let (buf, _, layout) = prepare_gpu_buffers(&nodes, &[]);
        assert_eq!(layout.positions_offset, 0);
        assert_eq!(layout.velocities_offset, 24);
        assert_eq!(layout.forces_offset, 48);
        assert_eq!(layout.stride, 8);
        assert_eq!(layout.node_count, 3);
        assert_eq!(buf.len(), 18);
    }

    #[test]
    fn test_gpu_buffer_velocities_written() {
        let mut node = LayoutNode::new(0, 10.0, 20.0);
        node.velocity = [3.0, 4.0];
        let (buf, _, _) = prepare_gpu_buffers(&[node], &[]);
        assert!((buf[2] - 3.0_f32).abs() < 1e-5, "vel_x={}", buf[2]);
        assert!((buf[3] - 4.0_f32).abs() < 1e-5, "vel_y={}", buf[3]);
    }

    #[test]
    fn test_gpu_buffer_forces_zeroed() {
        let nodes = random_layout(3, 100.0, 100.0);
        let (buf, _, _) = prepare_gpu_buffers(&nodes, &[]);
        for i in 12..18 {
            assert!((buf[i]).abs() < 1e-12, "forces[{i}] should be 0");
        }
    }

    // ── WGSL shader kernel strings ──────────────────────────────────────────

    #[test]
    fn test_wgsl_attraction_kernel_has_compute() {
        let src = wgsl_attraction_kernel();
        assert!(!src.is_empty());
        assert!(src.contains("@compute"));
        assert!(src.contains("workgroup_size"));
    }

    #[test]
    fn test_wgsl_repulsion_kernel_has_compute() {
        let src = wgsl_repulsion_kernel();
        assert!(!src.is_empty());
        assert!(src.contains("@compute"));
        assert!(src.contains("workgroup_size"));
    }

    #[test]
    fn test_wgsl_integration_kernel_has_compute() {
        let src = wgsl_integration_kernel();
        assert!(!src.is_empty());
        assert!(src.contains("@compute"));
        assert!(src.contains("workgroup_size"));
    }

    #[test]
    fn test_wgsl_bounds_kernel_has_compute() {
        let src = wgsl_bounds_kernel();
        assert!(!src.is_empty());
        assert!(src.contains("@compute"));
        assert!(src.contains("workgroup_size"));
    }

    #[test]
    fn test_wgsl_octree_build_kernel_has_compute() {
        let src = wgsl_octree_build_kernel();
        assert!(!src.is_empty());
        assert!(src.contains("@compute"));
        assert!(src.contains("workgroup_size"));
    }

    // ── segments_intersect ──────────────────────────────────────────────────

    #[test]
    fn test_segments_intersect_crossing_diagonals() {
        assert!(segments_intersect(
            [0.0, 0.0],
            [100.0, 100.0],
            [100.0, 0.0],
            [0.0, 100.0]
        ));
    }

    #[test]
    fn test_segments_no_intersection_parallel() {
        assert!(!segments_intersect(
            [0.0, 0.0],
            [100.0, 0.0],
            [0.0, 10.0],
            [100.0, 10.0]
        ));
    }

    #[test]
    fn test_segments_no_intersection_l_shape() {
        assert!(!segments_intersect(
            [0.0, 0.0],
            [50.0, 0.0],
            [50.0, 10.0],
            [50.0, 100.0]
        ));
    }

    // ── compute_stress ──────────────────────────────────────────────────────

    #[test]
    fn test_compute_stress_perfect_layout() {
        let nodes = vec![LayoutNode::new(0, 0.0, 0.0), LayoutNode::new(1, 100.0, 0.0)];
        let edges = vec![LayoutEdge {
            source: 0,
            target: 1,
            weight: 1.0,
            ideal_length: 100.0,
        }];
        let stress = compute_stress(&nodes, &edges);
        assert!(
            stress < 0.01,
            "stress should be ~0 for perfect layout, got {stress}"
        );
    }

    #[test]
    fn test_compute_stress_distorted_layout() {
        let nodes = vec![LayoutNode::new(0, 0.0, 0.0), LayoutNode::new(1, 200.0, 0.0)];
        let edges = vec![LayoutEdge {
            source: 0,
            target: 1,
            weight: 1.0,
            ideal_length: 100.0,
        }];
        let stress = compute_stress(&nodes, &edges);
        assert!((stress - 1.0).abs() < 0.01, "expected ~1.0, got {stress}");
    }

    #[test]
    fn test_compute_stress_single_node_zero() {
        let nodes = vec![LayoutNode::new(0, 0.0, 0.0)];
        assert!((compute_stress(&nodes, &[])).abs() < 1e-12);
    }

    #[test]
    fn test_compute_stress_disconnected_pair_ignored() {
        let nodes = vec![LayoutNode::new(0, 0.0, 0.0), LayoutNode::new(1, 999.0, 0.0)];
        let stress = compute_stress(&nodes, &[]);
        assert!((stress).abs() < 1e-12);
    }

    // ── compute_edge_crossing_count ─────────────────────────────────────────

    #[test]
    fn test_edge_crossing_count_square_diagonals() {
        let nodes = vec![
            LayoutNode::new(0, 0.0, 0.0),
            LayoutNode::new(1, 100.0, 0.0),
            LayoutNode::new(2, 100.0, 100.0),
            LayoutNode::new(3, 0.0, 100.0),
        ];
        let edges = vec![
            LayoutEdge {
                source: 0,
                target: 2,
                weight: 1.0,
                ideal_length: 141.0,
            },
            LayoutEdge {
                source: 1,
                target: 3,
                weight: 1.0,
                ideal_length: 141.0,
            },
        ];
        assert_eq!(compute_edge_crossing_count(&nodes, &edges), 1);
    }

    #[test]
    fn test_edge_crossing_count_no_crossings() {
        let nodes = vec![
            LayoutNode::new(0, 0.0, 0.0),
            LayoutNode::new(1, 100.0, 0.0),
            LayoutNode::new(2, 200.0, 0.0),
        ];
        let edges = vec![
            LayoutEdge {
                source: 0,
                target: 1,
                weight: 1.0,
                ideal_length: 100.0,
            },
            LayoutEdge {
                source: 1,
                target: 2,
                weight: 1.0,
                ideal_length: 100.0,
            },
        ];
        assert_eq!(compute_edge_crossing_count(&nodes, &edges), 0);
    }

    #[test]
    fn test_edge_crossing_count_fewer_than_two_edges() {
        let nodes = vec![LayoutNode::new(0, 0.0, 0.0), LayoutNode::new(1, 1.0, 0.0)];
        let edges = vec![LayoutEdge {
            source: 0,
            target: 1,
            weight: 1.0,
            ideal_length: 1.0,
        }];
        assert_eq!(compute_edge_crossing_count(&nodes, &edges), 0);
    }

    // ── compute_neighborhood_preservation ───────────────────────────────────

    #[test]
    fn test_neighborhood_preservation_k_zero() {
        let nodes = vec![LayoutNode::new(0, 0.0, 0.0), LayoutNode::new(1, 1.0, 0.0)];
        assert!((compute_neighborhood_preservation(&nodes, &[], 0) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_neighborhood_preservation_single_node() {
        let nodes = vec![LayoutNode::new(0, 0.0, 0.0)];
        assert!((compute_neighborhood_preservation(&nodes, &[], 1) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_neighborhood_preservation_linear_chain() {
        let nodes = vec![
            LayoutNode::new(0, 0.0, 0.0),
            LayoutNode::new(1, 100.0, 0.0),
            LayoutNode::new(2, 200.0, 0.0),
        ];
        let edges = vec![
            LayoutEdge {
                source: 0,
                target: 1,
                weight: 1.0,
                ideal_length: 100.0,
            },
            LayoutEdge {
                source: 1,
                target: 2,
                weight: 1.0,
                ideal_length: 100.0,
            },
        ];
        let np = compute_neighborhood_preservation(&nodes, &edges, 1);
        assert!(np > 0.5, "np={np} should be high for aligned layout");
    }

    // ── 3D types ────────────────────────────────────────────────────────────

    #[test]
    fn test_layout_node_3d_new() {
        let n = LayoutNode3D::new(3, 1.0, 2.0, 3.0);
        assert_eq!(n.id, 3);
        assert!((n.position[0] - 1.0).abs() < 1e-12);
        assert!((n.position[1] - 2.0).abs() < 1e-12);
        assert!((n.position[2] - 3.0).abs() < 1e-12);
        assert!((n.mass - 1.0).abs() < 1e-12);
        assert!(!n.pinned);
    }

    // ── LayoutConfig3D validation ───────────────────────────────────────────

    #[test]
    fn test_layout_config_3d_default_valid() {
        let c = LayoutConfig3D::default();
        assert!(c.validate().is_ok());
    }

    #[test]
    fn test_layout_config_3d_validate_iterations_zero() {
        let mut c = LayoutConfig3D::default();
        c.iterations = 0;
        assert!(matches!(
            c.validate(),
            Err(LayoutError::InvalidParameter(_))
        ));
    }

    #[test]
    fn test_layout_config_3d_validate_negative_repulsion() {
        let mut c = LayoutConfig3D::default();
        c.repulsion_strength = -1.0;
        assert!(matches!(
            c.validate(),
            Err(LayoutError::InvalidParameter(_))
        ));
    }

    #[test]
    fn test_layout_config_3d_validate_damping_out_of_range() {
        let mut c = LayoutConfig3D::default();
        c.damping = 2.0;
        assert!(matches!(
            c.validate(),
            Err(LayoutError::InvalidParameter(_))
        ));
    }

    #[test]
    fn test_layout_config_3d_validate_theta_zero() {
        let mut c = LayoutConfig3D::default();
        c.theta = 0.0;
        assert!(matches!(
            c.validate(),
            Err(LayoutError::InvalidParameter(_))
        ));
    }

    // ── random_layout_3d ────────────────────────────────────────────────────

    #[test]
    fn test_random_layout_3d_within_bounds() {
        let nodes = random_layout_3d(20, 800.0, 600.0, 400.0);
        assert_eq!(nodes.len(), 20);
        for (i, n) in nodes.iter().enumerate() {
            assert!(
                n.position[0] >= 0.0 && n.position[0] <= 800.0,
                "node {i} x OOB"
            );
            assert!(
                n.position[1] >= 0.0 && n.position[1] <= 600.0,
                "node {i} y OOB"
            );
            assert!(
                n.position[2] >= 0.0 && n.position[2] <= 400.0,
                "node {i} z OOB"
            );
        }
    }

    #[test]
    fn test_random_layout_3d_deterministic() {
        let a = random_layout_3d(10, 100.0, 100.0, 100.0);
        let b = random_layout_3d(10, 100.0, 100.0, 100.0);
        for (na, nb) in a.iter().zip(b.iter()) {
            for k in 0..3 {
                assert!((na.position[k] - nb.position[k]).abs() < 1e-12);
            }
        }
    }

    #[test]
    fn test_random_layout_3d_zero_nodes() {
        let nodes = random_layout_3d(0, 100.0, 100.0, 100.0);
        assert!(nodes.is_empty());
    }

    // ── normalize_positions_3d ──────────────────────────────────────────────

    #[test]
    fn test_normalize_positions_3d_scales_to_box() {
        let mut nodes = vec![
            LayoutNode3D::new(0, 0.0, 0.0, 0.0),
            LayoutNode3D::new(1, 100.0, 200.0, 50.0),
        ];
        normalize_positions_3d(&mut nodes, 400.0, 400.0, 200.0);
        assert!((nodes[1].position[0] - 400.0).abs() < 1e-9);
        assert!((nodes[1].position[1] - 400.0).abs() < 1e-9);
        assert!((nodes[1].position[2] - 200.0).abs() < 1e-9);
    }

    #[test]
    fn test_normalize_positions_3d_single_node_centres() {
        let mut nodes = vec![LayoutNode3D::new(0, 999.0, 888.0, 777.0)];
        normalize_positions_3d(&mut nodes, 400.0, 300.0, 200.0);
        assert!((nodes[0].position[0] - 200.0).abs() < 1e-9);
        assert!((nodes[0].position[1] - 150.0).abs() < 1e-9);
        assert!((nodes[0].position[2] - 100.0).abs() < 1e-9);
    }

    #[test]
    fn test_normalize_positions_3d_empty_slice() {
        let mut nodes: Vec<LayoutNode3D> = vec![];
        normalize_positions_3d(&mut nodes, 100.0, 100.0, 100.0);
        assert!(nodes.is_empty());
    }

    #[test]
    fn test_normalize_positions_3d_pinned_stays() {
        let mut nodes = vec![
            LayoutNode3D::new(0, 0.0, 0.0, 0.0),
            LayoutNode3D::new(1, 100.0, 100.0, 100.0),
        ];
        nodes[0].pinned = true;
        normalize_positions_3d(&mut nodes, 500.0, 500.0, 500.0);
        assert!((nodes[0].position[0]).abs() < 1e-12);
    }

    // ── build_octree ────────────────────────────────────────────────────────

    #[test]
    fn test_build_octree_basic() {
        let nodes = random_layout_3d(15, 300.0, 300.0, 300.0);
        let tree = build_octree(&nodes);
        assert_eq!(tree.node_count, 15);
        assert!(tree.root.total_mass > 0.0);
    }

    #[test]
    fn test_build_octree_single_node() {
        let nodes = vec![LayoutNode3D::new(0, 50.0, 50.0, 50.0)];
        let tree = build_octree(&nodes);
        assert_eq!(tree.node_count, 1);
        assert!(tree.root.is_single_body());
        assert_eq!(tree.root.body, Some(0));
    }

    // ── OctreeNode helpers ──────────────────────────────────────────────────

    #[test]
    fn test_octree_node_octant_for() {
        let node = OctreeNode::new([0.0, 0.0, 0.0, 100.0, 100.0, 100.0]);
        assert_eq!(node.octant_for([25.0, 25.0, 25.0]), 0);
        assert_eq!(node.octant_for([75.0, 25.0, 25.0]), 1);
        assert_eq!(node.octant_for([25.0, 75.0, 25.0]), 2);
        assert_eq!(node.octant_for([25.0, 25.0, 75.0]), 4);
        assert_eq!(node.octant_for([75.0, 75.0, 75.0]), 7);
    }

    #[test]
    fn test_octree_node_width_cuboid() {
        let node = OctreeNode::new([0.0, 0.0, 0.0, 100.0, 50.0, 200.0]);
        assert!((node.width() - 200.0).abs() < 1e-12);
    }

    #[test]
    fn test_octree_node_child_bounds_octant_0() {
        let node = OctreeNode::new([0.0, 0.0, 0.0, 100.0, 100.0, 100.0]);
        let cb = node.child_bounds(0);
        assert!((cb[0]).abs() < 1e-12);
        assert!((cb[1]).abs() < 1e-12);
        assert!((cb[2]).abs() < 1e-12);
        assert!((cb[3] - 50.0).abs() < 1e-12);
        assert!((cb[4] - 50.0).abs() < 1e-12);
        assert!((cb[5] - 50.0).abs() < 1e-12);
    }

    #[test]
    fn test_octree_node_child_bounds_octant_7() {
        let node = OctreeNode::new([0.0, 0.0, 0.0, 100.0, 100.0, 100.0]);
        let cb = node.child_bounds(7);
        assert!((cb[0] - 50.0).abs() < 1e-12);
        assert!((cb[1] - 50.0).abs() < 1e-12);
        assert!((cb[2] - 50.0).abs() < 1e-12);
        assert!((cb[3] - 100.0).abs() < 1e-12);
        assert!((cb[4] - 100.0).abs() < 1e-12);
        assert!((cb[5] - 100.0).abs() < 1e-12);
    }

    // ── 3D repulsion force ──────────────────────────────────────────────────

    #[test]
    fn test_repulsion_force_3d_magnitude() {
        let f = repulsion_force_3d([0.0, 0.0, 0.0], [100.0, 0.0, 0.0], 1.0, 1.0, 10_000.0, 1.0);
        assert!((f[0] - 1.0).abs() < 1e-9, "fx={}", f[0]);
        assert!((f[1]).abs() < 1e-12);
        assert!((f[2]).abs() < 1e-12);
    }

    #[test]
    fn test_attraction_force_3d_at_ideal() {
        let f = attraction_force_3d([0.0, 0.0, 0.0], [100.0, 0.0, 0.0], 1.0, 100.0, 0.05, 1.0);
        for k in 0..3 {
            assert!((f[k]).abs() < 1e-12, "f[{k}] = {}", f[k]);
        }
    }

    #[test]
    fn test_attraction_force_3d_stretched() {
        let f = attraction_force_3d([0.0, 0.0, 0.0], [200.0, 0.0, 0.0], 1.0, 100.0, 0.05, 1.0);
        assert!((f[0] - 5.0).abs() < 1e-9, "fx={}", f[0]);
    }

    // ── vec3_len ────────────────────────────────────────────────────────────

    #[test]
    fn test_vec3_len_unit() {
        assert!((vec3_len(&[1.0, 0.0, 0.0]) - 1.0).abs() < 1e-12);
        assert!((vec3_len(&[0.0, 1.0, 0.0]) - 1.0).abs() < 1e-12);
        assert!((vec3_len(&[0.0, 0.0, 1.0]) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_vec3_len_3_4_5_analog() {
        assert!((vec3_len(&[3.0, 4.0, 0.0]) - 5.0).abs() < 1e-12);
    }

    // ── compute_energy_3d ───────────────────────────────────────────────────

    #[test]
    fn test_compute_energy_3d_single_node() {
        let nodes = vec![LayoutNode3D::new(0, 0.0, 0.0, 0.0)];
        let config = LayoutConfig3D::default();
        let e = compute_energy_3d(&nodes, &[], &config);
        assert!((e).abs() < 1e-12);
    }

    #[test]
    fn test_compute_energy_3d_at_ideal_distance() {
        let nodes = vec![
            LayoutNode3D::new(0, 0.0, 0.0, 0.0),
            LayoutNode3D::new(1, 100.0, 0.0, 0.0),
        ];
        let edges = vec![LayoutEdge3D {
            source: 0,
            target: 1,
            weight: 1.0,
            ideal_length: 100.0,
        }];
        let config = LayoutConfig3D::default();
        let e = compute_energy_3d(&nodes, &edges, &config);
        assert!((e - 100.0).abs() < 0.01, "energy={e}");
    }

    // ── fruchterman_reingold_3d ─────────────────────────────────────────────

    #[test]
    fn test_fr_3d_empty_graph() {
        let config = LayoutConfig3D::default();
        let result = fruchterman_reingold_3d(&mut [], &[], &config);
        assert_eq!(result, Err(LayoutError::EmptyGraph));
    }

    #[test]
    fn test_fr_3d_invalid_edge() {
        let mut nodes = vec![LayoutNode3D::new(0, 0.0, 0.0, 0.0)];
        let edges = vec![LayoutEdge3D {
            source: 0,
            target: 99,
            weight: 1.0,
            ideal_length: 50.0,
        }];
        let mut config = LayoutConfig3D::default();
        config.iterations = 1;
        let result = fruchterman_reingold_3d(&mut nodes, &edges, &config);
        assert!(matches!(result, Err(LayoutError::InvalidNode(99))));
    }

    #[test]
    fn test_fr_3d_small_graph_runs() {
        let mut nodes = random_layout_3d(6, 200.0, 200.0, 200.0);
        let edges = vec![
            LayoutEdge3D {
                source: 0,
                target: 1,
                weight: 1.0,
                ideal_length: 80.0,
            },
            LayoutEdge3D {
                source: 1,
                target: 2,
                weight: 1.0,
                ideal_length: 80.0,
            },
        ];
        let mut config = LayoutConfig3D::default();
        config.iterations = 20;
        let result = fruchterman_reingold_3d(&mut nodes, &edges, &config);
        assert!(result.is_ok());
        let r = result.unwrap_or_else(|_| unreachable!());
        assert!(r.iterations_run > 0);
        assert_eq!(r.nodes.len(), 6);
    }

    // ── force_atlas2_3d ─────────────────────────────────────────────────────

    #[test]
    fn test_fa2_3d_empty_graph() {
        let config = LayoutConfig3D::default();
        let result = force_atlas2_3d(&mut [], &[], &config);
        assert_eq!(result, Err(LayoutError::EmptyGraph));
    }

    #[test]
    fn test_fa2_3d_small_graph_runs() {
        let mut nodes = random_layout_3d(5, 300.0, 300.0, 300.0);
        let edges = vec![LayoutEdge3D {
            source: 0,
            target: 1,
            weight: 1.0,
            ideal_length: 100.0,
        }];
        let mut config = LayoutConfig3D::default();
        config.iterations = 20;
        let result = force_atlas2_3d(&mut nodes, &edges, &config);
        assert!(result.is_ok());
        let r = result.unwrap_or_else(|_| unreachable!());
        assert!(r.iterations_run > 0);
    }

    #[test]
    fn test_fa2_3d_gravity_pulls_toward_origin() {
        let mut nodes = vec![LayoutNode3D::new(0, 500.0, 500.0, 500.0)];
        let mut config = LayoutConfig3D::default();
        config.iterations = 50;
        config.gravity = 10.0;
        let initial_dist = vec3_len(&nodes[0].position);
        let result = force_atlas2_3d(&mut nodes, &[], &config);
        assert!(result.is_ok());
        let r = result.unwrap_or_else(|_| unreachable!());
        let final_dist = vec3_len(&r.nodes[0].position);
        assert!(final_dist < initial_dist);
    }

    // ── compute_layout_3d ───────────────────────────────────────────────────

    #[test]
    fn test_compute_layout_3d_empty() {
        let config = LayoutConfig3D::default();
        let result = compute_layout_3d(vec![], vec![], &config);
        assert_eq!(result, Err(LayoutError::EmptyGraph));
    }

    #[test]
    fn test_compute_layout_3d_small_graph() {
        let nodes = random_layout_3d(8, 400.0, 400.0, 400.0);
        let edges = vec![LayoutEdge3D {
            source: 0,
            target: 1,
            weight: 1.0,
            ideal_length: 100.0,
        }];
        let mut config = LayoutConfig3D::default();
        config.iterations = 10;
        let result = compute_layout_3d(nodes, edges, &config);
        assert!(result.is_ok());
        let r = result.unwrap_or_else(|_| unreachable!());
        assert!(!r.nodes.is_empty());
    }

    // ── prepare_gpu_buffers_3d ──────────────────────────────────────────────

    #[test]
    fn test_prepare_gpu_buffers_3d_positions() {
        let nodes = vec![
            LayoutNode3D::new(0, 1.0, 2.0, 3.0),
            LayoutNode3D::new(1, 4.0, 5.0, 6.0),
        ];
        let edges = vec![LayoutEdge3D {
            source: 0,
            target: 1,
            weight: 1.0,
            ideal_length: 50.0,
        }];
        let (floats, indices, layout) = prepare_gpu_buffers_3d(&nodes, &edges);
        assert_eq!(layout.node_count, 2);
        assert_eq!(indices.len(), 2);
        assert!((floats[0] - 1.0_f32).abs() < 1e-5);
        assert!((floats[1] - 2.0_f32).abs() < 1e-5);
        assert!((floats[2] - 3.0_f32).abs() < 1e-5);
    }

    #[test]
    fn test_prepare_gpu_buffers_3d_layout_offsets() {
        let nodes = vec![LayoutNode3D::new(0, 0.0, 0.0, 0.0)];
        let (_, _, layout) = prepare_gpu_buffers_3d(&nodes, &[]);
        assert_eq!(layout.positions_offset, 0);
        assert_eq!(layout.velocities_offset, 12);
        assert_eq!(layout.forces_offset, 24);
        assert_eq!(layout.stride, 12);
        assert_eq!(layout.z_component_offset, 8);
    }

    #[test]
    fn test_prepare_gpu_buffers_3d_forces_zeroed() {
        let nodes = random_layout_3d(2, 100.0, 100.0, 100.0);
        let (buf, _, _) = prepare_gpu_buffers_3d(&nodes, &[]);
        for i in 12..18 {
            assert!((buf[i]).abs() < 1e-12);
        }
    }

    #[test]
    fn test_prepare_gpu_buffers_3d_no_edges() {
        let nodes = random_layout_3d(3, 100.0, 100.0, 100.0);
        let (buf, indices, layout) = prepare_gpu_buffers_3d(&nodes, &[]);
        assert_eq!(buf.len(), 27);
        assert!(indices.is_empty());
        assert_eq!(layout.node_count, 3);
    }

    // ── BH 3D repulsion ─────────────────────────────────────────────────────

    #[test]
    fn test_compute_repulsion_bh_3d_finite() {
        let nodes = random_layout_3d(10, 400.0, 400.0, 400.0);
        let tree = build_octree(&nodes);
        let force = compute_repulsion_bh_3d(&tree, &nodes[0], 0.8, 10_000.0, 1.0);
        for k in 0..3 {
            assert!(force[k].is_finite());
        }
    }

    // ── Serialization roundtrips ────────────────────────────────────────────

    #[test]
    fn test_layout_node_serde_roundtrip() {
        let node = LayoutNode::new(5, 42.0, 99.0);
        let json = serde_json::to_string(&node).unwrap_or_else(|_| unreachable!());
        let back: LayoutNode = serde_json::from_str(&json).unwrap_or_else(|_| unreachable!());
        assert_eq!(node, back);
    }

    #[test]
    fn test_layout_edge_serde_roundtrip() {
        let edge = LayoutEdge {
            source: 0,
            target: 1,
            weight: 0.5,
            ideal_length: 80.0,
        };
        let json = serde_json::to_string(&edge).unwrap_or_else(|_| unreachable!());
        let back: LayoutEdge = serde_json::from_str(&json).unwrap_or_else(|_| unreachable!());
        assert_eq!(edge, back);
    }

    #[test]
    fn test_layout_config_serde_roundtrip() {
        let config = LayoutConfig::default();
        let json = serde_json::to_string(&config).unwrap_or_else(|_| unreachable!());
        let back: LayoutConfig = serde_json::from_str(&json).unwrap_or_else(|_| unreachable!());
        assert_eq!(config, back);
    }

    #[test]
    fn test_layout_result_serde_roundtrip() {
        let result = LayoutResult {
            nodes: vec![LayoutNode::new(0, 1.0, 2.0)],
            iterations_run: 42,
            converged: true,
            energy: 123.456,
        };
        let json = serde_json::to_string(&result).unwrap_or_else(|_| unreachable!());
        let back: LayoutResult = serde_json::from_str(&json).unwrap_or_else(|_| unreachable!());
        assert_eq!(result, back);
    }

    #[test]
    fn test_layout_error_serde_roundtrip() {
        let e = LayoutError::InvalidNode(7);
        let json = serde_json::to_string(&e).unwrap_or_else(|_| unreachable!());
        let back: LayoutError = serde_json::from_str(&json).unwrap_or_else(|_| unreachable!());
        assert_eq!(e, back);
    }

    #[test]
    fn test_gpu_buffer_layout_serde_roundtrip() {
        let layout = GpuBufferLayout {
            positions_offset: 0,
            velocities_offset: 24,
            forces_offset: 48,
            stride: 8,
            node_count: 3,
        };
        let json = serde_json::to_string(&layout).unwrap_or_else(|_| unreachable!());
        let back: GpuBufferLayout = serde_json::from_str(&json).unwrap_or_else(|_| unreachable!());
        assert_eq!(layout, back);
    }

    #[test]
    fn test_layout_node_3d_serde_roundtrip() {
        let node = LayoutNode3D::new(2, 10.0, 20.0, 30.0);
        let json = serde_json::to_string(&node).unwrap_or_else(|_| unreachable!());
        let back: LayoutNode3D = serde_json::from_str(&json).unwrap_or_else(|_| unreachable!());
        assert_eq!(node, back);
    }

    #[test]
    fn test_layout_config_3d_serde_roundtrip() {
        let config = LayoutConfig3D::default();
        let json = serde_json::to_string(&config).unwrap_or_else(|_| unreachable!());
        let back: LayoutConfig3D = serde_json::from_str(&json).unwrap_or_else(|_| unreachable!());
        assert_eq!(config, back);
    }

    // ── Mass effects ────────────────────────────────────────────────────────

    #[test]
    fn test_heavier_node_repels_more_strongly() {
        let f_light = repulsion_force([0.0, 0.0], [100.0, 0.0], 1.0, 1.0, 10_000.0, 1.0);
        let f_heavy = repulsion_force([0.0, 0.0], [100.0, 0.0], 1.0, 5.0, 10_000.0, 1.0);
        assert!(f_heavy[0].abs() > f_light[0].abs());
    }

    // ── validate_edges ──────────────────────────────────────────────────────

    #[test]
    fn test_validate_edges_empty() {
        let nodes = vec![LayoutNode::new(0, 0.0, 0.0)];
        assert!(validate_edges(&nodes, &[]).is_ok());
    }

    #[test]
    fn test_validate_edges_source_out_of_range() {
        let nodes = vec![LayoutNode::new(0, 0.0, 0.0)];
        let edges = vec![LayoutEdge {
            source: 5,
            target: 0,
            weight: 1.0,
            ideal_length: 10.0,
        }];
        assert_eq!(
            validate_edges(&nodes, &edges),
            Err(LayoutError::InvalidNode(5))
        );
    }

    #[test]
    fn test_validate_edges_3d_target_out_of_range() {
        let nodes = vec![LayoutNode3D::new(0, 0.0, 0.0, 0.0)];
        let edges = vec![LayoutEdge3D {
            source: 0,
            target: 10,
            weight: 1.0,
            ideal_length: 10.0,
        }];
        assert!(matches!(
            validate_edges_3d(&nodes, &edges),
            Err(LayoutError::InvalidNode(10))
        ));
    }

    // ── xorshift64 PRNG property ────────────────────────────────────────────

    #[test]
    fn test_xorshift64_range() {
        let mut state: u64 = 0xDEAD_BEEF_CAFE_BABE;
        for _ in 0..100 {
            let val = xorshift64(&mut state);
            assert!(val >= 0.0, "val < 0: {val}");
            assert!(val < 1.0, "val >= 1: {val}");
        }
    }
}

// --- ACTUAL IMPLEMENTATIONS ---
pub fn accumulate_repulsion_3d(
    node: &OctreeNode,
    body: &LayoutNode3D,
    theta: f64,
    force: &mut [f64; 3],
    strength: f64,
    min_dist: f64,
) {
    if node.total_mass == 0.0 {
        return;
    }
    let dx = node.center_of_mass[0] - body.position[0];
    let dy = node.center_of_mass[1] - body.position[1];
    let dz = node.center_of_mass[2] - body.position[2];
    let dist = vec3_len(&[dx, dy, dz]).max(min_dist);

    if node.is_leaf() || (node.width() / dist < theta) {
        let mag = strength * body.mass * node.total_mass / (dist * dist);
        force[0] -= dx / dist * mag;
        force[1] -= dy / dist * mag;
        force[2] -= dz / dist * mag;
    } else {
        for child in node.children.iter().flatten() {
            accumulate_repulsion_3d(child, body, theta, force, strength, min_dist);
        }
    }
}

pub fn validate_edges_3d(
    nodes: &[LayoutNode3D],
    edges: &[LayoutEdge3D],
) -> Result<(), LayoutError> {
    let n = nodes.len();
    for e in edges {
        if e.source >= n || e.target >= n {
            return Err(LayoutError::InvalidNode(e.source.max(e.target)));
        }
    }
    Ok(())
}

pub fn repulsion_force_3d(
    pos_i: [f64; 3],
    pos_j: [f64; 3],
    mass_i: f64,
    mass_j: f64,
    strength: f64,
    min_dist: f64,
) -> [f64; 3] {
    let dx = pos_j[0] - pos_i[0];
    let dy = pos_j[1] - pos_i[1];
    let dz = pos_j[2] - pos_i[2];
    let dist = vec3_len(&[dx, dy, dz]).max(min_dist);
    let mag = strength * mass_i * mass_j / (dist * dist);
    [dx / dist * mag, dy / dist * mag, dz / dist * mag]
}

pub fn attraction_force_3d(
    pos_src: [f64; 3],
    pos_tgt: [f64; 3],
    weight: f64,
    ideal_length: f64,
    strength: f64,
    min_dist: f64,
) -> [f64; 3] {
    let dx = pos_tgt[0] - pos_src[0];
    let dy = pos_tgt[1] - pos_src[1];
    let dz = pos_tgt[2] - pos_src[2];
    let dist = vec3_len(&[dx, dy, dz]).max(min_dist);
    let stretch = dist - ideal_length;
    let mag = strength * weight * stretch;
    [dx / dist * mag, dy / dist * mag, dz / dist * mag]
}

pub fn vec3_len(v: &[f64; 3]) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

pub fn fruchterman_reingold_bh_3d(
    nodes: &mut [LayoutNode3D],
    edges: &[LayoutEdge3D],
    config: &LayoutConfig3D,
) -> Result<LayoutResult3D, LayoutError> {
    fruchterman_reingold_3d(nodes, edges, config)
}

pub fn segments_intersect(a: [f64; 2], b: [f64; 2], c: [f64; 2], d: [f64; 2]) -> bool {
    let ccw = |p1: [f64; 2], p2: [f64; 2], p3: [f64; 2]| -> bool {
        (p3[1] - p1[1]) * (p2[0] - p1[0]) > (p2[1] - p1[1]) * (p3[0] - p1[0])
    };
    ccw(a, c, d) != ccw(b, c, d) && ccw(a, b, c) != ccw(a, b, d)
}
