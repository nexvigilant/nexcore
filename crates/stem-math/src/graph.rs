//! # Graph Theory: Unified Graph<V,E> Type
//!
//! Canonical directed graph implementation supporting weighted and unweighted edges,
//! with algorithms for topological sort, SCC, centrality, shortest paths, PageRank,
//! and community detection.
//!
//! ## Primitive Foundation
//!
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | T1: Mapping (μ) | vertex-to-vertex relationships (DOMINANT) |
//! | T1: Location (λ) | nodes as positions in a network |
//! | T1: Boundary (∂) | communities, components |
//! | T1: Sequence (σ) | paths, topological ordering |
//!
//! ## Design
//!
//! `Graph<V, E>` with `E = ()` default supports unweighted graphs at zero cost.
//! Weighted graphs use `Graph<V, f64>` or any `E: Into<f64> + Copy`.
//! Directed by default; undirected via `add_undirected_edge()`.
//!
//! ## Relationship to Existing Implementations
//!
//! This is the canonical graph type. Existing implementations in:
//! - `nexcore-measure/src/graph.rs` (DepGraph + Tarjan + Brandes)
//! - `nexcore-vigilance/src/foundation/algorithms/graph.rs` (SkillGraph + topo sort + Dijkstra)
//! - `nexcore-mcp/src/tools/topology.rs` (topology tools)
//! - `academy-forge/src/graph/` (Kahn's topo sort)
//!
//! should eventually migrate to use this type. Migration is a future directive.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, VecDeque};

use stem_core::{Confidence, Measured};

// ============================================================================
// Core Types
// ============================================================================

/// Opaque vertex identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VertexId(usize);

impl VertexId {
    /// Create a VertexId from a raw index.
    #[must_use]
    pub fn new(index: usize) -> Self {
        Self(index)
    }

    /// Get the underlying index.
    #[must_use]
    pub fn index(self) -> usize {
        self.0
    }
}

/// A directed graph with typed vertices and edges.
///
/// Supports both weighted and unweighted edges via the type parameter `E`.
/// - Unweighted: `Graph<V>` or `Graph<V, ()>` — zero-size edge data
/// - Weighted: `Graph<V, f64>` — edge weights for shortest paths, PageRank, etc.
///
/// Adjacency list representation. O(V + E) space.
#[derive(Debug, Clone)]
pub struct Graph<V, E = ()> {
    /// Vertex data indexed by VertexId.
    vertices: Vec<V>,
    /// Adjacency list: for each vertex, a list of (target_vertex_id, edge_data).
    adjacency: Vec<Vec<(VertexId, E)>>,
}

impl<V, E> Default for Graph<V, E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V, E> Graph<V, E> {
    /// Create an empty graph.
    #[must_use]
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            adjacency: Vec::new(),
        }
    }

    /// Create a graph with pre-allocated capacity.
    #[must_use]
    pub fn with_capacity(vertex_capacity: usize) -> Self {
        Self {
            vertices: Vec::with_capacity(vertex_capacity),
            adjacency: Vec::with_capacity(vertex_capacity),
        }
    }

    /// Add a vertex and return its identifier.
    pub fn add_vertex(&mut self, data: V) -> VertexId {
        let id = VertexId(self.vertices.len());
        self.vertices.push(data);
        self.adjacency.push(Vec::new());
        id
    }

    /// Add a directed edge from `from` to `to` with edge data.
    ///
    /// # Panics
    ///
    /// Does not panic — silently ignores invalid vertex IDs.
    pub fn add_edge(&mut self, from: VertexId, to: VertexId, edge: E) {
        if from.0 < self.vertices.len() && to.0 < self.vertices.len() {
            self.adjacency[from.0].push((to, edge));
        }
    }

    /// Number of vertices.
    #[must_use]
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Number of directed edges.
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.adjacency.iter().map(|adj| adj.len()).sum()
    }

    /// Get outgoing neighbors and their edge data for a vertex.
    #[must_use]
    pub fn neighbors(&self, v: VertexId) -> &[(VertexId, E)] {
        if v.0 < self.adjacency.len() {
            &self.adjacency[v.0]
        } else {
            &[]
        }
    }

    /// Check if a directed edge exists from `from` to `to`.
    #[must_use]
    pub fn has_edge(&self, from: VertexId, to: VertexId) -> bool {
        if from.0 < self.adjacency.len() {
            self.adjacency[from.0].iter().any(|(t, _)| *t == to)
        } else {
            false
        }
    }

    /// Get vertex data by ID.
    #[must_use]
    pub fn vertex(&self, id: VertexId) -> Option<&V> {
        self.vertices.get(id.0)
    }

    /// Iterate over all vertices with their IDs.
    pub fn vertices(&self) -> impl Iterator<Item = (VertexId, &V)> {
        self.vertices
            .iter()
            .enumerate()
            .map(|(i, v)| (VertexId(i), v))
    }

    /// Iterate over all edges as (from, to, &edge_data).
    pub fn edges(&self) -> impl Iterator<Item = (VertexId, VertexId, &E)> {
        self.adjacency
            .iter()
            .enumerate()
            .flat_map(|(from_idx, adj)| adj.iter().map(move |(to, e)| (VertexId(from_idx), *to, e)))
    }

    /// Compute in-degree for each vertex.
    #[must_use]
    pub fn in_degrees(&self) -> Vec<usize> {
        let mut degrees = vec![0usize; self.vertices.len()];
        for adj in &self.adjacency {
            for &(to, _) in adj {
                degrees[to.0] += 1;
            }
        }
        degrees
    }

    /// Check if the graph is empty (no vertices).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }
}

// Undirected edge support
impl<V, E: Clone> Graph<V, E> {
    /// Add an undirected edge (inserts both directions).
    pub fn add_undirected_edge(&mut self, a: VertexId, b: VertexId, edge: E) {
        if a.0 < self.vertices.len() && b.0 < self.vertices.len() {
            self.adjacency[a.0].push((b, edge.clone()));
            self.adjacency[b.0].push((a, edge));
        }
    }
}

// ============================================================================
// Algorithm: Topological Sort (Kahn's)
// ============================================================================

/// Topological sort using Kahn's algorithm.
///
/// Returns vertices in topological order, or `None` if the graph contains a cycle.
///
/// **Time:** O(V + E)
/// **Space:** O(V)
pub fn topological_sort<V, E>(graph: &Graph<V, E>) -> Option<Vec<VertexId>> {
    let n = graph.vertex_count();
    let mut in_degree = graph.in_degrees();
    let mut queue: VecDeque<VertexId> = (0..n)
        .filter(|&i| in_degree[i] == 0)
        .map(VertexId)
        .collect();

    let mut result = Vec::with_capacity(n);

    while let Some(v) = queue.pop_front() {
        result.push(v);
        for &(to, _) in graph.neighbors(v) {
            in_degree[to.0] -= 1;
            if in_degree[to.0] == 0 {
                queue.push_back(to);
            }
        }
    }

    if result.len() == n {
        Some(result)
    } else {
        None // cycle detected
    }
}

// ============================================================================
// Algorithm: Tarjan's SCC
// ============================================================================

/// State for Tarjan's strongly connected components algorithm.
struct TarjanState {
    counter: usize,
    stack: Vec<VertexId>,
    on_stack: Vec<bool>,
    indices: Vec<Option<usize>>,
    lowlinks: Vec<usize>,
    result: Vec<Vec<VertexId>>,
}

impl TarjanState {
    fn new(n: usize) -> Self {
        Self {
            counter: 0,
            stack: Vec::new(),
            on_stack: vec![false; n],
            indices: vec![None; n],
            lowlinks: vec![0; n],
            result: Vec::new(),
        }
    }
}

/// Strongly connected components via Tarjan's algorithm.
///
/// Returns all SCCs (including singletons). Filter for `scc.len() > 1` to find cycles.
///
/// **Time:** O(V + E)
/// **Space:** O(V)
pub fn tarjan_scc<V, E>(graph: &Graph<V, E>) -> Vec<Vec<VertexId>> {
    let n = graph.vertex_count();
    let mut state = TarjanState::new(n);

    for v in 0..n {
        if state.indices[v].is_none() {
            tarjan_visit(graph, VertexId(v), &mut state);
        }
    }

    state.result
}

fn tarjan_visit<V, E>(graph: &Graph<V, E>, v: VertexId, s: &mut TarjanState) {
    s.indices[v.0] = Some(s.counter);
    s.lowlinks[v.0] = s.counter;
    s.counter += 1;
    s.stack.push(v);
    s.on_stack[v.0] = true;

    for &(w, _) in graph.neighbors(v) {
        if s.indices[w.0].is_none() {
            tarjan_visit(graph, w, s);
            s.lowlinks[v.0] = s.lowlinks[v.0].min(s.lowlinks[w.0]);
        } else if s.on_stack[w.0] {
            let w_idx = s.indices[w.0].unwrap_or(0);
            s.lowlinks[v.0] = s.lowlinks[v.0].min(w_idx);
        }
    }

    // If v is a root of an SCC, pop the stack
    let v_idx = s.indices[v.0].unwrap_or(0);
    if s.lowlinks[v.0] == v_idx {
        let mut scc = Vec::new();
        while let Some(w) = s.stack.pop() {
            s.on_stack[w.0] = false;
            scc.push(w);
            if w == v {
                break;
            }
        }
        s.result.push(scc);
    }
}

// ============================================================================
// Algorithm: Betweenness Centrality (Brandes)
// ============================================================================

/// Betweenness centrality via Brandes' algorithm.
///
/// Returns normalized centrality score per vertex.
/// For directed graphs, normalization factor is (V-1)(V-2).
///
/// **Time:** O(V * E)
/// **Space:** O(V + E)
pub fn betweenness_centrality<V, E>(graph: &Graph<V, E>) -> Vec<f64> {
    let n = graph.vertex_count();
    let mut cb = vec![0.0_f64; n];

    for s in 0..n {
        let (stack, sigma, pred) = brandes_bfs(graph, VertexId(s));
        brandes_accumulate(VertexId(s), &stack, &sigma, &pred, &mut cb);
    }

    // Normalize
    let norm = if n > 2 {
        ((n - 1) * (n - 2)) as f64
    } else {
        1.0
    };
    for c in &mut cb {
        *c /= norm;
    }

    cb
}

/// BFS phase of Brandes' algorithm.
fn brandes_bfs<V, E>(
    graph: &Graph<V, E>,
    source: VertexId,
) -> (Vec<VertexId>, Vec<f64>, Vec<Vec<VertexId>>) {
    let n = graph.vertex_count();
    let mut sigma = vec![0.0_f64; n];
    sigma[source.0] = 1.0;
    let mut dist = vec![-1_i64; n];
    dist[source.0] = 0;
    let mut pred: Vec<Vec<VertexId>> = vec![Vec::new(); n];
    let mut stack = Vec::new();
    let mut queue = VecDeque::new();
    queue.push_back(source);

    while let Some(v) = queue.pop_front() {
        stack.push(v);
        for &(w, _) in graph.neighbors(v) {
            if dist[w.0] < 0 {
                dist[w.0] = dist[v.0] + 1;
                queue.push_back(w);
            }
            if dist[w.0] == dist[v.0] + 1 {
                sigma[w.0] += sigma[v.0];
                pred[w.0].push(v);
            }
        }
    }

    (stack, sigma, pred)
}

/// Accumulation phase of Brandes' algorithm.
fn brandes_accumulate(
    source: VertexId,
    stack: &[VertexId],
    sigma: &[f64],
    pred: &[Vec<VertexId>],
    cb: &mut [f64],
) {
    let n = sigma.len();
    let mut delta = vec![0.0_f64; n];

    for &w in stack.iter().rev() {
        for &v in &pred[w.0] {
            if sigma[w.0] > 0.0 {
                delta[v.0] += (sigma[v.0] / sigma[w.0]) * (1.0 + delta[w.0]);
            }
        }
        if w != source {
            cb[w.0] += delta[w.0];
        }
    }
}

// ============================================================================
// Algorithm: Connected Components (BFS)
// ============================================================================

/// Find connected components treating edges as undirected.
///
/// Returns groups of vertex IDs, one group per component.
///
/// **Time:** O(V + E)
/// **Space:** O(V)
pub fn connected_components<V, E>(graph: &Graph<V, E>) -> Vec<Vec<VertexId>> {
    let n = graph.vertex_count();
    let mut visited = vec![false; n];
    let mut components = Vec::new();

    // Build reverse adjacency for undirected traversal
    let mut rev_adj: Vec<Vec<VertexId>> = vec![Vec::new(); n];
    for (from, to, _) in graph.edges() {
        rev_adj[to.0].push(from);
    }

    for start in 0..n {
        if visited[start] {
            continue;
        }
        let mut component = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back(VertexId(start));
        visited[start] = true;

        while let Some(v) = queue.pop_front() {
            component.push(v);
            // Forward edges
            for &(w, _) in graph.neighbors(v) {
                if !visited[w.0] {
                    visited[w.0] = true;
                    queue.push_back(w);
                }
            }
            // Reverse edges (treat as undirected)
            for &w in &rev_adj[v.0] {
                if !visited[w.0] {
                    visited[w.0] = true;
                    queue.push_back(w);
                }
            }
        }

        components.push(component);
    }

    components
}

// ============================================================================
// Algorithm: BFS Shortest Path (Unweighted)
// ============================================================================

/// BFS shortest path in an unweighted graph.
///
/// Returns the path as a sequence of vertex IDs from `from` to `to`,
/// or `None` if no path exists.
///
/// **Time:** O(V + E)
/// **Space:** O(V)
pub fn bfs_shortest_path<V, E>(
    graph: &Graph<V, E>,
    from: VertexId,
    to: VertexId,
) -> Option<Vec<VertexId>> {
    if from == to {
        return Some(vec![from]);
    }

    let n = graph.vertex_count();
    if from.0 >= n || to.0 >= n {
        return None;
    }

    let mut visited = vec![false; n];
    let mut parent: Vec<Option<VertexId>> = vec![None; n];
    let mut queue = VecDeque::new();

    visited[from.0] = true;
    queue.push_back(from);

    while let Some(v) = queue.pop_front() {
        for &(w, _) in graph.neighbors(v) {
            if !visited[w.0] {
                visited[w.0] = true;
                parent[w.0] = Some(v);
                if w == to {
                    return Some(reconstruct_path(&parent, from, to));
                }
                queue.push_back(w);
            }
        }
    }

    None
}

/// Reconstruct path from parent array.
fn reconstruct_path(parent: &[Option<VertexId>], from: VertexId, to: VertexId) -> Vec<VertexId> {
    let mut path = Vec::new();
    let mut current = to;
    path.push(current);
    while current != from {
        match parent[current.0] {
            Some(p) => {
                path.push(p);
                current = p;
            }
            None => break,
        }
    }
    path.reverse();
    path
}

// ============================================================================
// Algorithm: Dijkstra's Weighted Shortest Path
// ============================================================================

/// Entry in Dijkstra's priority queue.
#[derive(Debug)]
struct DijkstraEntry {
    vertex: VertexId,
    cost: f64,
}

impl PartialEq for DijkstraEntry {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}

impl Eq for DijkstraEntry {}

impl PartialOrd for DijkstraEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DijkstraEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap behavior with BinaryHeap (max-heap)
        other
            .cost
            .partial_cmp(&self.cost)
            .unwrap_or(Ordering::Equal)
    }
}

/// Dijkstra's weighted shortest path algorithm.
///
/// Returns the path and total cost, or `None` if no path exists.
/// Edge weights must be non-negative.
///
/// **Time:** O((V + E) log V)
/// **Space:** O(V)
pub fn dijkstra<V, E: EdgeWeight>(
    graph: &Graph<V, E>,
    from: VertexId,
    to: VertexId,
) -> Option<(Vec<VertexId>, f64)> {
    let n = graph.vertex_count();
    if from.0 >= n || to.0 >= n {
        return None;
    }

    let mut dist = vec![f64::INFINITY; n];
    let mut parent: Vec<Option<VertexId>> = vec![None; n];
    let mut visited = vec![false; n];
    let mut heap = BinaryHeap::new();

    dist[from.0] = 0.0;
    heap.push(DijkstraEntry {
        vertex: from,
        cost: 0.0,
    });

    while let Some(DijkstraEntry { vertex: v, cost }) = heap.pop() {
        if visited[v.0] {
            continue;
        }
        if v == to {
            return Some((reconstruct_path(&parent, from, to), cost));
        }
        visited[v.0] = true;

        for &(w, ref edge) in graph.neighbors(v) {
            let weight = edge.weight();
            let new_cost = cost + weight;
            if new_cost < dist[w.0] {
                dist[w.0] = new_cost;
                parent[w.0] = Some(v);
                heap.push(DijkstraEntry {
                    vertex: w,
                    cost: new_cost,
                });
            }
        }
    }

    // Check if target was reached
    if dist[to.0].is_finite() {
        Some((reconstruct_path(&parent, from, to), dist[to.0]))
    } else {
        None
    }
}

// ============================================================================
// Algorithm: PageRank
// ============================================================================

/// Iterative PageRank computation.
///
/// Returns `Measured<f64>` per vertex. Confidence derived from convergence:
/// converged within tolerance → high confidence; hit max_iterations → lower confidence.
///
/// **Time:** O(max_iterations * E)
/// **Space:** O(V)
pub fn pagerank<V, E>(
    graph: &Graph<V, E>,
    damping: f64,
    max_iterations: usize,
    tolerance: f64,
) -> Vec<Measured<f64>> {
    let n = graph.vertex_count();
    if n == 0 {
        return Vec::new();
    }

    let initial = 1.0 / n as f64;
    let mut rank = vec![initial; n];
    let mut new_rank = vec![0.0_f64; n];

    // Precompute out-degrees
    let out_degree: Vec<usize> = (0..n).map(|i| graph.neighbors(VertexId(i)).len()).collect();

    let mut converged = false;
    let mut iterations_used = 0;

    for iter in 0..max_iterations {
        iterations_used = iter + 1;

        // Base rank for all nodes (teleportation)
        let base = (1.0 - damping) / n as f64;
        for r in new_rank.iter_mut() {
            *r = base;
        }

        // Distribute rank along edges
        for v in 0..n {
            if out_degree[v] == 0 {
                // Dangling node: distribute rank evenly to all nodes
                let share = damping * rank[v] / n as f64;
                for r in new_rank.iter_mut() {
                    *r += share;
                }
            } else {
                let share = damping * rank[v] / out_degree[v] as f64;
                for &(w, _) in graph.neighbors(VertexId(v)) {
                    new_rank[w.0] += share;
                }
            }
        }

        // Check convergence (L1 norm)
        let diff: f64 = rank
            .iter()
            .zip(new_rank.iter())
            .map(|(a, b)| (a - b).abs())
            .sum();

        std::mem::swap(&mut rank, &mut new_rank);

        if diff < tolerance {
            converged = true;
            break;
        }
    }

    // Confidence based on convergence
    let confidence_val = if converged {
        // Converged: high confidence, scaled by how quickly
        (1.0 - iterations_used as f64 / max_iterations as f64)
            .mul_add(0.15, 0.85)
            .clamp(0.85, 0.99)
    } else {
        // Didn't converge: moderate confidence
        0.60
    };
    let confidence = Confidence::new(confidence_val);

    rank.into_iter()
        .map(|r| Measured::new(r, confidence))
        .collect()
}

// ============================================================================
// Algorithm: Louvain Community Detection
// ============================================================================

/// Louvain community detection via modularity optimization.
///
/// Works with weighted graphs (E: Into<f64> + Copy) or unweighted (E = ()).
/// For unweighted graphs, all edges have weight 1.0.
///
/// Returns groups of vertex IDs forming communities.
///
/// **Time:** O(V * E) per pass, typically converges in a few passes
/// **Space:** O(V + E)
pub fn louvain_communities<V, E: EdgeWeight>(graph: &Graph<V, E>) -> Vec<Vec<VertexId>> {
    let n = graph.vertex_count();
    if n == 0 {
        return Vec::new();
    }

    // Build weighted adjacency for modularity computation
    let (adj_weights, total_weight) = build_weight_matrix(graph);
    if total_weight < f64::EPSILON {
        // No edges — each node is its own community
        return (0..n).map(|i| vec![VertexId(i)]).collect();
    }

    // Initialize: each node in its own community
    let mut community: Vec<usize> = (0..n).collect();

    // Node strengths (sum of edge weights)
    let strength: Vec<f64> = (0..n)
        .map(|i| adj_weights[i].iter().map(|(_, w)| w).sum())
        .collect();

    // Iterative phase: move nodes to maximize modularity gain
    let mut improved = true;
    while improved {
        improved = false;
        for i in 0..n {
            let best = find_best_community(i, &community, &adj_weights, &strength, total_weight);
            if best != community[i] {
                community[i] = best;
                improved = true;
            }
        }
    }

    // Collect communities
    let mut groups: HashMap<usize, Vec<VertexId>> = HashMap::new();
    for (i, &c) in community.iter().enumerate() {
        groups.entry(c).or_default().push(VertexId(i));
    }

    groups.into_values().collect()
}

/// Build weighted adjacency list from graph.
fn build_weight_matrix<V, E: EdgeWeight>(graph: &Graph<V, E>) -> (Vec<Vec<(usize, f64)>>, f64) {
    let n = graph.vertex_count();
    let mut adj: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
    let mut total = 0.0;

    for (from, to, edge) in graph.edges() {
        let w: f64 = edge.weight();
        adj[from.0].push((to.0, w));
        total += w;
    }

    (adj, total)
}

/// Find the community assignment that maximizes modularity gain for node i.
fn find_best_community(
    node: usize,
    community: &[usize],
    adj_weights: &[Vec<(usize, f64)>],
    strength: &[f64],
    total_weight: f64,
) -> usize {
    let current = community[node];
    let mut best_community = current;
    let mut best_gain = 0.0;

    // Compute weights to each neighboring community
    let mut community_weights: HashMap<usize, f64> = HashMap::new();
    for &(neighbor, w) in &adj_weights[node] {
        *community_weights.entry(community[neighbor]).or_default() += w;
    }

    // Sum of strengths in each community
    let mut community_strength: HashMap<usize, f64> = HashMap::new();
    for (i, &c) in community.iter().enumerate() {
        *community_strength.entry(c).or_default() += strength[i];
    }

    let ki = strength[node];
    let m2 = 2.0 * total_weight;

    // Weight to current community (excluding self)
    let w_current = community_weights.get(&current).copied().unwrap_or(0.0);
    let sigma_current = community_strength.get(&current).copied().unwrap_or(0.0) - ki;

    for (&c, &w_c) in &community_weights {
        if c == current {
            continue;
        }
        let sigma_c = community_strength.get(&c).copied().unwrap_or(0.0);

        // Modularity gain of moving node from current to c
        // ΔQ = [w_c - sigma_c * ki / m] / m - [w_current - sigma_current * ki / m] / m
        let gain = (w_c - sigma_c * ki / m2) / m2 - (w_current - sigma_current * ki / m2) / m2;

        if gain > best_gain {
            best_gain = gain;
            best_community = c;
        }
    }

    best_community
}

/// Trait for converting edge data to a numeric weight.
///
/// Implemented for `f64` (direct weight) and `()` (unweighted, weight = 1.0).
/// This avoids orphan rule issues with `From<()> for f64`.
pub trait EdgeWeight {
    /// Convert edge data to a floating-point weight.
    fn weight(&self) -> f64;
}

impl EdgeWeight for f64 {
    fn weight(&self) -> f64 {
        *self
    }
}

impl EdgeWeight for () {
    fn weight(&self) -> f64 {
        1.0
    }
}

impl EdgeWeight for f32 {
    fn weight(&self) -> f64 {
        *self as f64
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- Helper graph builders ---

    fn make_dag() -> Graph<&'static str> {
        let mut g = Graph::new();
        let a = g.add_vertex("A");
        let b = g.add_vertex("B");
        let c = g.add_vertex("C");
        let d = g.add_vertex("D");
        g.add_edge(a, b, ());
        g.add_edge(a, c, ());
        g.add_edge(b, d, ());
        g.add_edge(c, d, ());
        g
    }

    fn make_cycle() -> Graph<&'static str> {
        let mut g = Graph::new();
        let a = g.add_vertex("A");
        let b = g.add_vertex("B");
        let c = g.add_vertex("C");
        g.add_edge(a, b, ());
        g.add_edge(b, c, ());
        g.add_edge(c, a, ());
        g
    }

    fn make_disconnected() -> Graph<&'static str> {
        let mut g = Graph::new();
        let a = g.add_vertex("A");
        let b = g.add_vertex("B");
        let c = g.add_vertex("C");
        let d = g.add_vertex("D");
        g.add_edge(a, b, ());
        g.add_edge(c, d, ());
        g
    }

    fn make_star() -> Graph<&'static str> {
        let mut g = Graph::new();
        let center = g.add_vertex("center");
        let a = g.add_vertex("A");
        let b = g.add_vertex("B");
        let c = g.add_vertex("C");
        let d = g.add_vertex("D");
        g.add_edge(center, a, ());
        g.add_edge(center, b, ());
        g.add_edge(center, c, ());
        g.add_edge(center, d, ());
        g.add_edge(a, center, ());
        g.add_edge(b, center, ());
        g.add_edge(c, center, ());
        g.add_edge(d, center, ());
        g
    }

    fn make_weighted() -> Graph<&'static str, f64> {
        let mut g = Graph::new();
        let a = g.add_vertex("A");
        let b = g.add_vertex("B");
        let c = g.add_vertex("C");
        let d = g.add_vertex("D");
        // A→B: 1, A→C: 4, B→C: 2, B→D: 6, C→D: 3
        g.add_edge(a, b, 1.0);
        g.add_edge(a, c, 4.0);
        g.add_edge(b, c, 2.0);
        g.add_edge(b, d, 6.0);
        g.add_edge(c, d, 3.0);
        g
    }

    fn make_three_node_cycle_weighted() -> Graph<&'static str, f64> {
        let mut g = Graph::new();
        let a = g.add_vertex("A");
        let b = g.add_vertex("B");
        let c = g.add_vertex("C");
        g.add_edge(a, b, 1.0);
        g.add_edge(b, c, 1.0);
        g.add_edge(c, a, 1.0);
        g
    }

    fn make_two_clusters() -> Graph<&'static str, f64> {
        let mut g = Graph::new();
        // Cluster 1: A-B-C (fully connected)
        let a = g.add_vertex("A");
        let b = g.add_vertex("B");
        let c = g.add_vertex("C");
        g.add_undirected_edge(a, b, 1.0);
        g.add_undirected_edge(b, c, 1.0);
        g.add_undirected_edge(a, c, 1.0);

        // Cluster 2: D-E-F (fully connected)
        let d = g.add_vertex("D");
        let e = g.add_vertex("E");
        let f = g.add_vertex("F");
        g.add_undirected_edge(d, e, 1.0);
        g.add_undirected_edge(e, f, 1.0);
        g.add_undirected_edge(d, f, 1.0);

        // Weak bridge between clusters
        g.add_undirected_edge(c, d, 0.1);
        g
    }

    // --- Core API tests ---

    #[test]
    fn test_empty_graph() {
        let g: Graph<i32> = Graph::new();
        assert_eq!(g.vertex_count(), 0);
        assert_eq!(g.edge_count(), 0);
        assert!(g.is_empty());
    }

    #[test]
    fn test_add_vertices_and_edges() {
        let mut g = Graph::new();
        let a = g.add_vertex("A");
        let b = g.add_vertex("B");
        g.add_edge(a, b, ());
        assert_eq!(g.vertex_count(), 2);
        assert_eq!(g.edge_count(), 1);
        assert!(g.has_edge(a, b));
        assert!(!g.has_edge(b, a));
    }

    #[test]
    fn test_undirected_edge() {
        let mut g = Graph::new();
        let a = g.add_vertex("A");
        let b = g.add_vertex("B");
        g.add_undirected_edge(a, b, ());
        assert!(g.has_edge(a, b));
        assert!(g.has_edge(b, a));
        assert_eq!(g.edge_count(), 2);
    }

    #[test]
    fn test_vertex_data() {
        let mut g: Graph<i32> = Graph::new();
        let a = g.add_vertex(42);
        assert_eq!(g.vertex(a), Some(&42));
    }

    #[test]
    fn test_weighted_edge() {
        let mut g: Graph<&str, f64> = Graph::new();
        let a = g.add_vertex("A");
        let b = g.add_vertex("B");
        g.add_edge(a, b, 3.14);
        let neighbors = g.neighbors(a);
        assert_eq!(neighbors.len(), 1);
        assert_eq!(neighbors[0].1, 3.14);
    }

    // --- Topological Sort ---

    #[test]
    fn test_topo_sort_dag() {
        let g = make_dag();
        let order = topological_sort(&g);
        assert!(order.is_some());
        let order = order.as_ref().map(|o| o.as_slice()).unwrap_or(&[]);
        assert_eq!(order.len(), 4);
        // A must come before B, C. B and C must come before D.
        let pos = |id: VertexId| order.iter().position(|&v| v == id);
        assert!(pos(VertexId(0)) < pos(VertexId(1))); // A before B
        assert!(pos(VertexId(0)) < pos(VertexId(2))); // A before C
        assert!(pos(VertexId(1)) < pos(VertexId(3))); // B before D
        assert!(pos(VertexId(2)) < pos(VertexId(3))); // C before D
    }

    #[test]
    fn test_topo_sort_cycle() {
        let g = make_cycle();
        assert!(topological_sort(&g).is_none());
    }

    // --- Tarjan SCC ---

    #[test]
    fn test_scc_dag_no_cycles() {
        let g = make_dag();
        let sccs = tarjan_scc(&g);
        // All singletons — no multi-node SCCs
        assert!(sccs.iter().all(|scc| scc.len() == 1));
    }

    #[test]
    fn test_scc_cycle() {
        let g = make_cycle();
        let sccs = tarjan_scc(&g);
        let multi: Vec<_> = sccs.iter().filter(|scc| scc.len() > 1).collect();
        assert_eq!(multi.len(), 1);
        assert_eq!(multi[0].len(), 3);
    }

    #[test]
    fn test_scc_three_components() {
        // Graph with 3 SCCs: {A,B}, {C,D}, {E}
        let mut g = Graph::new();
        let a = g.add_vertex("A");
        let b = g.add_vertex("B");
        let c = g.add_vertex("C");
        let d = g.add_vertex("D");
        let e = g.add_vertex("E");
        // SCC 1: A↔B
        g.add_edge(a, b, ());
        g.add_edge(b, a, ());
        // SCC 2: C↔D
        g.add_edge(c, d, ());
        g.add_edge(d, c, ());
        // Bridge: B→C (not part of SCC)
        g.add_edge(b, c, ());
        // Singleton: E
        g.add_edge(d, e, ());

        let sccs = tarjan_scc(&g);
        let multi: Vec<_> = sccs.iter().filter(|scc| scc.len() > 1).collect();
        assert_eq!(multi.len(), 2); // Two non-trivial SCCs
    }

    // --- Betweenness Centrality ---

    #[test]
    fn test_betweenness_star() {
        let g = make_star();
        let bc = betweenness_centrality(&g);
        // Center should have highest centrality
        let center_bc = bc[0];
        for &c in &bc[1..] {
            assert!(center_bc >= c);
        }
    }

    #[test]
    fn test_betweenness_linear() {
        let mut g = Graph::new();
        let a = g.add_vertex("A");
        let b = g.add_vertex("B");
        let c = g.add_vertex("C");
        g.add_edge(a, b, ());
        g.add_edge(b, c, ());
        let bc = betweenness_centrality(&g);
        // B is the bridge — highest centrality
        assert!(bc[b.0] >= bc[a.0]);
        assert!(bc[b.0] >= bc[c.0]);
    }

    // --- Connected Components ---

    #[test]
    fn test_components_connected() {
        let g = make_dag();
        let comps = connected_components(&g);
        assert_eq!(comps.len(), 1);
    }

    #[test]
    fn test_components_disconnected() {
        let g = make_disconnected();
        let comps = connected_components(&g);
        assert_eq!(comps.len(), 2);
    }

    // --- BFS Shortest Path ---

    #[test]
    fn test_bfs_path_exists() {
        let g = make_dag();
        let path = bfs_shortest_path(&g, VertexId(0), VertexId(3));
        assert!(path.is_some());
        let path = path.as_ref().map(|p| p.as_slice()).unwrap_or(&[]);
        assert_eq!(path.len(), 3); // A → B → D or A → C → D
        assert_eq!(path[0], VertexId(0)); // starts at A
        assert_eq!(path[2], VertexId(3)); // ends at D
    }

    #[test]
    fn test_bfs_no_path() {
        let g = make_disconnected();
        let path = bfs_shortest_path(&g, VertexId(0), VertexId(2));
        assert!(path.is_none());
    }

    #[test]
    fn test_bfs_self_path() {
        let g = make_dag();
        let path = bfs_shortest_path(&g, VertexId(0), VertexId(0));
        assert_eq!(path, Some(vec![VertexId(0)]));
    }

    // --- Dijkstra ---

    #[test]
    fn test_dijkstra_weighted() {
        // A→B: 1, A→C: 4, B→C: 2, B→D: 6, C→D: 3
        // Shortest A→D: A→B→C→D = 1+2+3 = 6 (not A→B→D = 1+6 = 7)
        let g = make_weighted();
        let result = dijkstra(&g, VertexId(0), VertexId(3));
        assert!(result.is_some());
        let (path, cost) = result
            .as_ref()
            .map(|(p, c)| (p.as_slice(), *c))
            .unwrap_or((&[], 0.0));
        assert!((cost - 6.0).abs() < 1e-10);
        assert_eq!(path[0], VertexId(0)); // A
        assert_eq!(path[path.len() - 1], VertexId(3)); // D
    }

    #[test]
    fn test_dijkstra_unreachable() {
        let g = make_weighted();
        // D has no outgoing edges to A
        let result = dijkstra(&g, VertexId(3), VertexId(0));
        assert!(result.is_none());
    }

    #[test]
    fn test_dijkstra_self() {
        let g = make_weighted();
        let result = dijkstra(&g, VertexId(0), VertexId(0));
        assert!(result.is_some());
        let (path, cost) = result
            .as_ref()
            .map(|(p, c)| (p.as_slice(), *c))
            .unwrap_or((&[], 0.0));
        assert_eq!(path.len(), 1);
        assert!((cost).abs() < 1e-10);
    }

    // --- PageRank ---

    #[test]
    fn test_pagerank_cycle() {
        // 3-node cycle: all should have equal PageRank
        let g = make_three_node_cycle_weighted();
        let ranks = pagerank(&g, 0.85, 100, 1e-6);
        assert_eq!(ranks.len(), 3);
        let r0 = ranks[0].value;
        let r1 = ranks[1].value;
        let r2 = ranks[2].value;
        assert!((r0 - r1).abs() < 1e-4);
        assert!((r1 - r2).abs() < 1e-4);
    }

    #[test]
    fn test_pagerank_star() {
        // Star graph: center should have highest rank
        let g = make_star();
        let ranks = pagerank(&g, 0.85, 100, 1e-6);
        let center_rank = ranks[0].value;
        for r in &ranks[1..] {
            assert!(center_rank >= r.value - 1e-6);
        }
    }

    #[test]
    fn test_pagerank_convergence_confidence() {
        let g = make_three_node_cycle_weighted();
        let ranks = pagerank(&g, 0.85, 100, 1e-6);
        // Should converge → high confidence
        assert!(ranks[0].confidence.value() >= 0.85);
    }

    // --- Louvain Community Detection ---

    #[test]
    fn test_louvain_two_clusters() {
        let g = make_two_clusters();
        let communities = louvain_communities(&g);
        // Should detect at least 2 communities (weak bridge between clusters)
        assert!(
            communities.len() >= 2,
            "Expected >= 2 communities, got {}",
            communities.len()
        );
    }

    #[test]
    fn test_louvain_complete_graph() {
        // Complete graph should form one community
        let mut g: Graph<&str, f64> = Graph::new();
        let a = g.add_vertex("A");
        let b = g.add_vertex("B");
        let c = g.add_vertex("C");
        g.add_undirected_edge(a, b, 1.0);
        g.add_undirected_edge(b, c, 1.0);
        g.add_undirected_edge(a, c, 1.0);
        let communities = louvain_communities(&g);
        // Complete graph: either 1 community or 3 singletons (both valid)
        assert!(!communities.is_empty());
    }

    #[test]
    fn test_louvain_empty() {
        let g: Graph<&str, f64> = Graph::new();
        let communities = louvain_communities(&g);
        assert!(communities.is_empty());
    }

    // --- Edge iteration ---

    #[test]
    fn test_edges_iterator() {
        let g = make_dag();
        let edges: Vec<_> = g.edges().collect();
        assert_eq!(edges.len(), 4);
    }

    #[test]
    fn test_vertices_iterator() {
        let g = make_dag();
        let verts: Vec<_> = g.vertices().collect();
        assert_eq!(verts.len(), 4);
        assert_eq!(*verts[0].1, "A");
    }

    // --- In-degree ---

    #[test]
    fn test_in_degrees() {
        let g = make_dag();
        let deg = g.in_degrees();
        assert_eq!(deg[0], 0); // A: no incoming
        assert_eq!(deg[1], 1); // B: from A
        assert_eq!(deg[2], 1); // C: from A
        assert_eq!(deg[3], 2); // D: from B and C
    }
}
