//! Graph Centrality Metrics
//!
//! Computes four standard node-level centrality measures for undirected
//! graphs.  All functions accept a [`GraphSpec`] and return a
//! `HashMap<String, f64>` mapping each node ID to its centrality score.
//!
//! ## Available Metrics
//!
//! | Function | Algorithm | Complexity |
//! |----------|-----------|------------|
//! | [`degree_centrality`] | Degree / (n−1) | O(E) |
//! | [`betweenness_centrality`] | Brandes (2001) | O(VE) |
//! | [`closeness_centrality`] | BFS shortest paths | O(V·(V+E)) |
//! | [`eigenvector_centrality`] | Power iteration | O(iter·E) |
//!
//! ## Normalisation
//!
//! - **Degree**: normalised by `n − 1` so values are in `[0, 1]`.
//! - **Betweenness**: normalised by `(n−1)(n−2)` (undirected convention).
//! - **Closeness**: uses the harmonic variant so it handles disconnected nodes
//!   gracefully — disconnected node pairs contribute 0.
//! - **Eigenvector**: the vector is normalised so its maximum entry is 1.0.
//!
//! # Example
//!
//! ```rust
//! use nexcore_viz::spectral::GraphSpec;
//! use nexcore_viz::centrality::{degree_centrality, betweenness_centrality};
//!
//! let g = GraphSpec {
//!     node_ids: vec!["hub".into(), "a".into(), "b".into(), "c".into()],
//!     edges: vec![
//!         ("hub".into(), "a".into()),
//!         ("hub".into(), "b".into()),
//!         ("hub".into(), "c".into()),
//!     ],
//! };
//! let dc = degree_centrality(&g);
//! assert_eq!(dc["hub"], 1.0); // hub is connected to all others
//!
//! let bc = betweenness_centrality(&g);
//! assert!(bc["hub"] > bc["a"]);
//! ```

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

use crate::spectral::GraphSpec;
use std::collections::{HashMap, HashSet, VecDeque};

// ─── Public API ───────────────────────────────────────────────────────────────

/// Computes **degree centrality** for every node.
///
/// `DC(v) = deg(v) / (n − 1)`
///
/// Values are in `[0.0, 1.0]`.  A value of `1.0` means the node is connected
/// to every other node.  For a single-node graph every node gets `0.0`.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::spectral::GraphSpec;
/// use nexcore_viz::centrality::degree_centrality;
///
/// let g = GraphSpec {
///     node_ids: vec!["a".into(), "b".into(), "c".into()],
///     edges: vec![("a".into(), "b".into()), ("a".into(), "c".into())],
/// };
/// let dc = degree_centrality(&g);
/// assert_eq!(dc["a"], 1.0);  // connected to both others
/// assert_eq!(dc["b"], 0.5);
/// ```
#[must_use]
pub fn degree_centrality(graph: &GraphSpec) -> HashMap<String, f64> {
    let n = graph.n();
    let denom = if n <= 1 { 1.0 } else { (n - 1) as f64 };

    let idx = graph.index_map();
    let mut deg = vec![0usize; n];

    for (u, v) in &graph.edges {
        let Some(&i) = idx.get(u.as_str()) else {
            continue;
        };
        let Some(&j) = idx.get(v.as_str()) else {
            continue;
        };
        if i == j {
            continue;
        }
        deg[i] += 1;
        deg[j] += 1;
    }

    graph
        .node_ids
        .iter()
        .enumerate()
        .map(|(i, id)| {
            let d = deg.get(i).copied().unwrap_or(0);
            (id.clone(), d as f64 / denom)
        })
        .collect()
}

/// Computes **betweenness centrality** using the Brandes algorithm.
///
/// `BC(v) = Σ_{s≠v≠t} σ(s,t|v) / σ(s,t)` normalised by `(n−1)(n−2)` for
/// undirected graphs.
///
/// Uses unweighted BFS shortest paths.  Nodes that lie on no shortest path
/// receive `0.0`.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::spectral::GraphSpec;
/// use nexcore_viz::centrality::betweenness_centrality;
///
/// // Path 1-2-3: node 2 is the only bridge.
/// let g = GraphSpec {
///     node_ids: vec!["1".into(), "2".into(), "3".into()],
///     edges: vec![("1".into(), "2".into()), ("2".into(), "3".into())],
/// };
/// let bc = betweenness_centrality(&g);
/// assert!(bc["2"] > bc["1"]);
/// assert!(bc["2"] > bc["3"]);
/// ```
#[must_use]
pub fn betweenness_centrality(graph: &GraphSpec) -> HashMap<String, f64> {
    let n = graph.n();
    let idx = graph.index_map();
    let neighbours = build_neighbour_index(graph, &idx, n);

    let mut centrality = vec![0.0_f64; n];

    // Brandes algorithm: for each source s run BFS, accumulate pair
    // dependencies.
    for s in 0..n {
        // Stack of nodes in order of non-decreasing distance.
        let mut stack: Vec<usize> = Vec::with_capacity(n);
        // Predecessors on shortest paths from s.
        let mut pred: Vec<Vec<usize>> = vec![vec![]; n];
        // Number of shortest paths from s to each node.
        let mut sigma = vec![0.0_f64; n];
        sigma[s] = 1.0;
        // Distance from s.
        let mut dist = vec![-1_i64; n];
        dist[s] = 0;

        let mut queue: VecDeque<usize> = VecDeque::new();
        queue.push_back(s);

        // BFS
        while let Some(v) = queue.pop_front() {
            stack.push(v);
            let d_v = dist[v];
            for &w in neighbours.get(v).map(Vec::as_slice).unwrap_or(&[]) {
                // First visit?
                if dist[w] < 0 {
                    dist[w] = d_v + 1;
                    queue.push_back(w);
                }
                // Shortest path via v?
                if dist[w] == d_v + 1 {
                    sigma[w] += sigma[v];
                    pred[w].push(v);
                }
            }
        }

        // Accumulation — walk back through stack in reverse.
        let mut delta = vec![0.0_f64; n];
        while let Some(w) = stack.pop() {
            for &v in &pred[w] {
                let ratio = if sigma[w] > 0.0 {
                    sigma[v] / sigma[w]
                } else {
                    0.0
                };
                delta[v] += ratio * (1.0 + delta[w]);
            }
            if w != s {
                centrality[w] += delta[w];
            }
        }
    }

    // Normalise: each pair (s,t) counted once per direction in undirected BFS,
    // so divide by 2, then by (n-1)(n-2).
    let norm = if n > 2 {
        ((n - 1) as f64) * ((n - 2) as f64)
    } else {
        1.0
    };

    graph
        .node_ids
        .iter()
        .enumerate()
        .map(|(i, id)| {
            let raw = centrality.get(i).copied().unwrap_or(0.0);
            (id.clone(), raw / norm)
        })
        .collect()
}

/// Computes **closeness centrality** using the harmonic mean of shortest-path
/// distances (harmonic closeness).
///
/// `CC(v) = (1 / (n−1)) * Σ_{u≠v} 1 / d(v,u)`
///
/// The harmonic variant is well-defined even for disconnected graphs: pairs
/// with no path simply contribute 0 (as if `d = ∞`).  Normalised by `n − 1`.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::spectral::GraphSpec;
/// use nexcore_viz::centrality::closeness_centrality;
///
/// // Star: centre should have highest closeness.
/// let g = GraphSpec {
///     node_ids: vec!["c".into(), "a".into(), "b".into(), "d".into()],
///     edges: vec![
///         ("c".into(), "a".into()),
///         ("c".into(), "b".into()),
///         ("c".into(), "d".into()),
///     ],
/// };
/// let cc = closeness_centrality(&g);
/// assert!(cc["c"] > cc["a"]);
/// ```
#[must_use]
pub fn closeness_centrality(graph: &GraphSpec) -> HashMap<String, f64> {
    let n = graph.n();
    let idx = graph.index_map();
    let neighbours = build_neighbour_index(graph, &idx, n);
    let denom = if n <= 1 { 1.0 } else { (n - 1) as f64 };

    graph
        .node_ids
        .iter()
        .enumerate()
        .map(|(s, id)| {
            let harmonic_sum = bfs_harmonic_sum(s, n, &neighbours);
            (id.clone(), harmonic_sum / denom)
        })
        .collect()
}

/// Computes **eigenvector centrality** via power iteration.
///
/// A node's score is proportional to the sum of its neighbours' scores.
/// Converges to the dominant eigenvector of the adjacency matrix.
///
/// The returned scores are normalised so the maximum is `1.0`.  Isolated
/// nodes receive `0.0`.
///
/// # Arguments
///
/// * `graph`    – The graph to analyse.
/// * `max_iter` – Maximum power-iteration steps (100–1000 is usually enough).
///
/// # Example
///
/// ```rust
/// use nexcore_viz::spectral::GraphSpec;
/// use nexcore_viz::centrality::eigenvector_centrality;
///
/// let g = GraphSpec {
///     node_ids: vec!["hub".into(), "a".into(), "b".into(), "c".into()],
///     edges: vec![
///         ("hub".into(), "a".into()),
///         ("hub".into(), "b".into()),
///         ("hub".into(), "c".into()),
///         ("a".into(),   "b".into()),
///     ],
/// };
/// let ec = eigenvector_centrality(&g, 200);
/// // hub is most central
/// assert!(ec["hub"] >= ec["a"]);
/// assert!(ec["hub"] >= ec["c"]);
/// ```
#[must_use]
pub fn eigenvector_centrality(graph: &GraphSpec, max_iter: usize) -> HashMap<String, f64> {
    let n = graph.n();
    if n == 0 {
        return HashMap::new();
    }

    let idx = graph.index_map();
    let neighbours = build_neighbour_index(graph, &idx, n);

    // Initialise to uniform.
    let mut scores = vec![1.0_f64; n];
    let tol = 1e-8_f64;

    for _ in 0..max_iter {
        let mut new_scores = vec![0.0_f64; n];
        for (v, score) in new_scores.iter_mut().enumerate() {
            for &u in neighbours.get(v).map(Vec::as_slice).unwrap_or(&[]) {
                *score += scores.get(u).copied().unwrap_or(0.0);
            }
        }

        // Normalise by max so we don't overflow.
        let max_val = new_scores.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        if max_val > f64::EPSILON {
            for s in new_scores.iter_mut() {
                *s /= max_val;
            }
        }

        // Check convergence.
        let delta: f64 = scores
            .iter()
            .zip(new_scores.iter())
            .map(|(a, b)| (a - b).abs())
            .fold(0.0_f64, f64::max);

        scores = new_scores;
        if delta < tol {
            break;
        }
    }

    // Final max-normalise.
    let max_val = scores.iter().copied().fold(0.0_f64, f64::max);

    graph
        .node_ids
        .iter()
        .enumerate()
        .map(|(i, id)| {
            let raw = scores.get(i).copied().unwrap_or(0.0);
            let normalised = if max_val > f64::EPSILON {
                raw / max_val
            } else {
                0.0
            };
            (id.clone(), normalised)
        })
        .collect()
}

// ─── Internal Helpers ─────────────────────────────────────────────────────────

/// Builds an adjacency list indexed by node index: `neighbours[i]` = list of
/// neighbour indices.
fn build_neighbour_index(
    graph: &GraphSpec,
    idx: &HashMap<&str, usize>,
    n: usize,
) -> Vec<Vec<usize>> {
    let mut adj: Vec<HashSet<usize>> = vec![HashSet::new(); n];
    for (u, v) in &graph.edges {
        let Some(&i) = idx.get(u.as_str()) else {
            continue;
        };
        let Some(&j) = idx.get(v.as_str()) else {
            continue;
        };
        if i == j {
            continue;
        }
        adj[i].insert(j);
        adj[j].insert(i);
    }
    adj.into_iter().map(|s| s.into_iter().collect()).collect()
}

/// BFS from source `s`; returns the harmonic sum Σ_{u≠s} 1/d(s,u).
fn bfs_harmonic_sum(s: usize, n: usize, neighbours: &[Vec<usize>]) -> f64 {
    let mut dist = vec![-1_i64; n];
    dist[s] = 0;
    let mut queue: VecDeque<usize> = VecDeque::new();
    queue.push_back(s);
    let mut harmonic = 0.0_f64;

    while let Some(v) = queue.pop_front() {
        let d_v = dist[v];
        for &w in neighbours.get(v).map(Vec::as_slice).unwrap_or(&[]) {
            if dist[w] < 0 {
                dist[w] = d_v + 1;
                harmonic += 1.0 / dist[w] as f64;
                queue.push_back(w);
            }
        }
    }
    harmonic
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Star graph: one hub connected to 4 leaves.
    fn star() -> GraphSpec {
        GraphSpec {
            node_ids: vec!["hub".into(), "a".into(), "b".into(), "c".into(), "d".into()],
            edges: vec![
                ("hub".into(), "a".into()),
                ("hub".into(), "b".into()),
                ("hub".into(), "c".into()),
                ("hub".into(), "d".into()),
            ],
        }
    }

    /// Path 1 — 2 — 3 — 4 — 5.
    fn path5() -> GraphSpec {
        GraphSpec {
            node_ids: (1..=5).map(|i| i.to_string()).collect(),
            edges: vec![
                ("1".into(), "2".into()),
                ("2".into(), "3".into()),
                ("3".into(), "4".into()),
                ("4".into(), "5".into()),
            ],
        }
    }

    /// Complete graph K₄.
    fn k4() -> GraphSpec {
        GraphSpec {
            node_ids: vec!["a".into(), "b".into(), "c".into(), "d".into()],
            edges: vec![
                ("a".into(), "b".into()),
                ("a".into(), "c".into()),
                ("a".into(), "d".into()),
                ("b".into(), "c".into()),
                ("b".into(), "d".into()),
                ("c".into(), "d".into()),
            ],
        }
    }

    // ── degree_centrality ─────────────────────────────────────────────────

    #[test]
    fn degree_centrality_star_hub_is_one() {
        let dc = degree_centrality(&star());
        assert_eq!(dc["hub"], 1.0);
    }

    #[test]
    fn degree_centrality_star_leaves_are_quarter() {
        let dc = degree_centrality(&star());
        for leaf in ["a", "b", "c", "d"] {
            assert_eq!(dc[leaf], 0.25, "leaf {leaf} should have DC 0.25");
        }
    }

    #[test]
    fn degree_centrality_k4_all_equal() {
        let dc = degree_centrality(&k4());
        for &id in &["a", "b", "c", "d"] {
            assert!(
                (dc[id] - 1.0).abs() < 1e-10,
                "K₄ node {id} should have DC 1.0"
            );
        }
    }

    #[test]
    fn degree_centrality_empty_graph() {
        let g = GraphSpec {
            node_ids: vec![],
            edges: vec![],
        };
        assert!(degree_centrality(&g).is_empty());
    }

    #[test]
    fn degree_centrality_all_values_in_unit_interval() {
        let dc = degree_centrality(&path5());
        for (_, v) in &dc {
            assert!(*v >= 0.0 && *v <= 1.0, "DC value {v} out of [0, 1]");
        }
    }

    // ── betweenness_centrality ────────────────────────────────────────────

    #[test]
    fn betweenness_centrality_path_middle_is_highest() {
        let bc = betweenness_centrality(&path5());
        // Node "3" (middle) should have the highest BC in a path graph.
        let center = bc["3"];
        assert!(
            center > bc["1"],
            "middle node should beat endpoint: {center} > {}",
            bc["1"]
        );
        assert!(
            center > bc["5"],
            "middle node should beat endpoint: {center} > {}",
            bc["5"]
        );
    }

    #[test]
    fn betweenness_centrality_star_hub_dominates() {
        let bc = betweenness_centrality(&star());
        let hub = bc["hub"];
        for leaf in ["a", "b", "c", "d"] {
            assert!(
                hub > bc[leaf],
                "hub BC {hub} should exceed leaf {leaf} BC {}",
                bc[leaf]
            );
        }
    }

    #[test]
    fn betweenness_centrality_k4_all_equal() {
        let bc = betweenness_centrality(&k4());
        // In K₄ every node is symmetric, so all BC values are equal.
        let values: Vec<f64> = bc.values().copied().collect();
        let first = values.first().copied().unwrap_or(0.0);
        for v in &values {
            assert!(
                (v - first).abs() < 1e-10,
                "K₄ BC values should all be equal: {first} vs {v}"
            );
        }
    }

    #[test]
    fn betweenness_centrality_non_negative() {
        let bc = betweenness_centrality(&path5());
        for (_, v) in &bc {
            assert!(*v >= 0.0, "betweenness must be non-negative, got {v}");
        }
    }

    #[test]
    fn betweenness_centrality_all_nodes_present() {
        let g = path5();
        let bc = betweenness_centrality(&g);
        for id in &g.node_ids {
            assert!(
                bc.contains_key(id),
                "node {id} missing from betweenness result"
            );
        }
    }

    // ── closeness_centrality ──────────────────────────────────────────────

    #[test]
    fn closeness_centrality_star_hub_is_highest() {
        let cc = closeness_centrality(&star());
        let hub = cc["hub"];
        for leaf in ["a", "b", "c", "d"] {
            assert!(
                hub > cc[leaf],
                "hub closeness {hub} should exceed leaf {leaf} closeness {}",
                cc[leaf]
            );
        }
    }

    #[test]
    fn closeness_centrality_path_endpoints_lowest() {
        let cc = closeness_centrality(&path5());
        let center = cc["3"];
        assert!(
            center > cc["1"],
            "center closeness should exceed endpoint: {center} > {}",
            cc["1"]
        );
        assert!(
            center > cc["5"],
            "center closeness should exceed endpoint: {center} > {}",
            cc["5"]
        );
    }

    #[test]
    fn closeness_centrality_k4_all_equal() {
        let cc = closeness_centrality(&k4());
        let values: Vec<f64> = cc.values().copied().collect();
        let first = values.first().copied().unwrap_or(0.0);
        for v in &values {
            assert!(
                (v - first).abs() < 1e-10,
                "K₄ closeness values should all be equal"
            );
        }
    }

    #[test]
    fn closeness_centrality_disconnected_node_is_zero() {
        let g = GraphSpec {
            node_ids: vec!["a".into(), "b".into(), "isolated".into()],
            edges: vec![("a".into(), "b".into())],
        };
        let cc = closeness_centrality(&g);
        assert_eq!(cc["isolated"], 0.0);
    }

    // ── eigenvector_centrality ────────────────────────────────────────────

    #[test]
    fn eigenvector_centrality_star_hub_scores_one() {
        let ec = eigenvector_centrality(&star(), 500);
        assert!(
            (ec["hub"] - 1.0).abs() < 0.01,
            "hub eigenvector centrality should be ≈1.0, got {}",
            ec["hub"]
        );
    }

    #[test]
    fn eigenvector_centrality_k4_all_equal() {
        let ec = eigenvector_centrality(&k4(), 500);
        let values: Vec<f64> = ec.values().copied().collect();
        let first = values.first().copied().unwrap_or(0.0);
        for v in &values {
            assert!(
                (v - first).abs() < 1e-6,
                "K₄ eigenvector centrality should all be equal"
            );
        }
    }

    #[test]
    fn eigenvector_centrality_max_is_one() {
        let ec = eigenvector_centrality(&path5(), 500);
        let max_val = ec.values().copied().fold(f64::NEG_INFINITY, f64::max);
        assert!(
            (max_val - 1.0).abs() < 1e-6,
            "maximum eigenvector centrality should be 1.0, got {max_val}"
        );
    }

    #[test]
    fn eigenvector_centrality_non_negative() {
        let ec = eigenvector_centrality(&star(), 200);
        for (_, v) in &ec {
            assert!(
                *v >= 0.0,
                "eigenvector centrality must be non-negative, got {v}"
            );
        }
    }

    #[test]
    fn eigenvector_centrality_empty_graph() {
        let g = GraphSpec {
            node_ids: vec![],
            edges: vec![],
        };
        assert!(eigenvector_centrality(&g, 100).is_empty());
    }

    #[test]
    fn eigenvector_centrality_hub_beats_spoke() {
        // Hub connects to all; well-connected neighbour boosts score.
        let g = GraphSpec {
            node_ids: vec!["hub".into(), "a".into(), "b".into(), "c".into()],
            edges: vec![
                ("hub".into(), "a".into()),
                ("hub".into(), "b".into()),
                ("hub".into(), "c".into()),
                ("a".into(), "b".into()),
            ],
        };
        let ec = eigenvector_centrality(&g, 500);
        assert!(
            ec["hub"] >= ec["c"],
            "hub EC {} should be ≥ leaf EC {}",
            ec["hub"],
            ec["c"]
        );
    }
}
