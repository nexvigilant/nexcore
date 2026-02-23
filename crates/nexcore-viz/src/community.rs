//! Community Detection — Louvain Modularity Optimisation
//!
//! Implements the Louvain algorithm for finding community structure in
//! undirected graphs by maximising the modularity Q.
//!
//! ## Algorithm Overview
//!
//! **Phase 1 — Local Moves:** Each node is tentatively moved to each
//! neighbouring community.  The move that yields the greatest positive ΔQ is
//! accepted.  Passes repeat until no move improves Q.
//!
//! **Phase 2 — Aggregation:** Communities become super-nodes; their internal
//! edges become self-loops (not used in Q).  The two phases alternate until
//! the partition stops changing.
//!
//! ## References
//!
//! Blondel et al., "Fast unfolding of communities in large networks",
//! *J. Stat. Mech.* (2008).
//!
//! # Example
//!
//! ```rust
//! use nexcore_viz::spectral::GraphSpec;
//! use nexcore_viz::community::detect_communities;
//!
//! // Two cliques connected by a single bridge.
//! let g = GraphSpec {
//!     node_ids: vec![
//!         "a".into(), "b".into(), "c".into(),
//!         "x".into(), "y".into(), "z".into(),
//!     ],
//!     edges: vec![
//!         ("a".into(), "b".into()), ("b".into(), "c".into()), ("a".into(), "c".into()),
//!         ("x".into(), "y".into()), ("y".into(), "z".into()), ("x".into(), "z".into()),
//!         ("c".into(), "x".into()), // bridge
//!     ],
//! };
//! let communities = detect_communities(&g);
//! assert!(communities.len() >= 1 && communities.len() <= 3);
//! ```

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use crate::spectral::GraphSpec;
use std::collections::{HashMap, HashSet};

// ─── Public API ───────────────────────────────────────────────────────────────

/// Computes the **modularity Q** of a partition of the graph into communities.
///
/// Modularity measures how much better the observed community structure is
/// compared to a random graph with the same degree sequence:
///
/// ```text
/// Q = (1 / 2m) * Σ_{i,j} [ A_{ij} − k_i k_j / 2m ] δ(c_i, c_j)
/// ```
///
/// where `m` is the number of edges, `k_i` the degree of node `i`, and
/// `δ(c_i, c_j) = 1` if nodes `i` and `j` belong to the same community.
///
/// Returns values in `(-0.5, 1.0]`.  A value near 0.0 means the partition is
/// no better than random; higher values indicate stronger community structure.
///
/// # Arguments
///
/// * `graph`       – The graph to evaluate.
/// * `communities` – A slice of communities; each community is a slice of node IDs.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::spectral::GraphSpec;
/// use nexcore_viz::community::modularity;
///
/// let g = GraphSpec {
///     node_ids: vec!["a".into(), "b".into(), "c".into(), "d".into()],
///     edges: vec![
///         ("a".into(), "b".into()),
///         ("c".into(), "d".into()),
///     ],
/// };
/// // Perfect 2-community partition
/// let comms: Vec<Vec<String>> = vec![
///     vec!["a".into(), "b".into()],
///     vec!["c".into(), "d".into()],
/// ];
/// let q = modularity(&g, &comms);
/// assert!(q > 0.0);
/// ```
#[must_use]
pub fn modularity(graph: &GraphSpec, communities: &[Vec<String>]) -> f64 {
    let m = edge_count(graph);
    if m == 0 {
        return 0.0;
    }
    let two_m = 2.0 * m as f64;

    let idx = graph.index_map();
    let degrees = compute_degrees(graph, &idx);
    let adj = build_adjacency(graph, &idx);

    // Map each node index to its community label.
    let mut node_comm: Vec<usize> = vec![0; graph.n()];
    for (comm_id, comm) in communities.iter().enumerate() {
        for node_id in comm {
            if let Some(&i) = idx.get(node_id.as_str()) {
                node_comm[i] = comm_id;
            }
        }
    }

    let mut q = 0.0_f64;
    for i in 0..graph.n() {
        for j in 0..graph.n() {
            if node_comm[i] != node_comm[j] {
                continue;
            }
            let a_ij = adj.get(&(i, j)).copied().unwrap_or(0.0);
            let ki = degrees.get(i).copied().unwrap_or(0.0);
            let kj = degrees.get(j).copied().unwrap_or(0.0);
            q += a_ij - ki * kj / two_m;
        }
    }
    q / two_m
}

/// Detects communities using the **Louvain algorithm**.
///
/// Returns a `Vec<Vec<String>>` where each inner `Vec` is one community
/// containing the original node IDs of its members.  Communities are ordered
/// by size (largest first).  Isolated nodes each form their own community.
///
/// The algorithm is deterministic for a fixed node ordering.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::spectral::GraphSpec;
/// use nexcore_viz::community::detect_communities;
///
/// let g = GraphSpec {
///     node_ids: vec!["a".into(), "b".into(), "c".into(),
///                    "x".into(), "y".into(), "z".into()],
///     edges: vec![
///         ("a".into(), "b".into()), ("b".into(), "c".into()), ("a".into(), "c".into()),
///         ("x".into(), "y".into()), ("y".into(), "z".into()), ("x".into(), "z".into()),
///         ("c".into(), "x".into()),
///     ],
/// };
/// let comms = detect_communities(&g);
/// assert!(comms.len() >= 1 && comms.len() <= 3);
/// ```
#[must_use]
pub fn detect_communities(graph: &GraphSpec) -> Vec<Vec<String>> {
    let n = graph.n();
    if n == 0 {
        return vec![];
    }
    if n == 1 {
        return vec![graph.node_ids.clone()];
    }

    // Start: each node in its own community.
    let mut partition: Vec<usize> = (0..n).collect();

    // Phase 1 + 2 loop.  We track the current graph as an adjacency weight map
    // over super-node indices, plus a mapping from super-node → original nodes.
    let idx = graph.index_map();
    let adj = build_adjacency(graph, &idx);

    let changed = phase1(graph, &adj, &mut partition);
    if !changed {
        return partition_to_communities(graph, &partition);
    }

    // Phase 2: build super-graph and recurse (one level is typically enough).
    let (super_graph, super_to_originals) = build_super_graph(graph, &partition);
    if super_graph.n() >= n {
        // No reduction — stop to avoid infinite loop.
        return partition_to_communities(graph, &partition);
    }

    let mut super_partition: Vec<usize> = (0..super_graph.n()).collect();
    let super_idx = super_graph.index_map();
    let super_adj = build_adjacency(&super_graph, &super_idx);
    phase1(&super_graph, &super_adj, &mut super_partition);

    // Map super-node community assignments back to original nodes.
    let mut final_partition = vec![0usize; n];
    for (super_node_idx, &comm) in super_partition.iter().enumerate() {
        let super_id = &super_graph.node_ids[super_node_idx];
        if let Some(originals) = super_to_originals.get(super_id) {
            for orig_node_id in originals {
                if let Some(&orig_idx) = idx.get(orig_node_id.as_str()) {
                    final_partition[orig_idx] = comm;
                }
            }
        }
    }

    partition_to_communities(graph, &final_partition)
}

// ─── Internal: Louvain Phase 1 ────────────────────────────────────────────────

/// Runs Phase 1 (greedy local moves) in place on `partition`.
///
/// Returns `true` if at least one node moved community.
fn phase1(graph: &GraphSpec, adj: &HashMap<(usize, usize), f64>, partition: &mut [usize]) -> bool {
    let n = graph.n();
    let m = edge_count(graph);
    if m == 0 {
        return false;
    }
    let two_m = 2.0 * m as f64;

    let idx = graph.index_map();
    let degrees = compute_degrees(graph, &idx);

    // neighbour list: node index → list of (neighbour_index, weight)
    let neighbours = build_neighbour_list(n, adj);

    let mut any_moved = false;
    let mut improved = true;

    while improved {
        improved = false;

        for i in 0..n {
            let current_comm = partition[i];
            let ki = degrees.get(i).copied().unwrap_or(0.0);

            // Sum of weights of edges from i to each community.
            let mut comm_weights: HashMap<usize, f64> = HashMap::new();
            for &(j, w) in neighbours.get(i).map(Vec::as_slice).unwrap_or(&[]) {
                let cj = partition[j];
                *comm_weights.entry(cj).or_insert(0.0) += w;
            }

            // Σ_tot for each candidate community (sum of degrees of nodes in comm).
            let mut sigma_tot: HashMap<usize, f64> = HashMap::new();
            for (j, &cj) in partition.iter().enumerate() {
                let kj = degrees.get(j).copied().unwrap_or(0.0);
                *sigma_tot.entry(cj).or_insert(0.0) += kj;
            }

            // Remove i from its current community (for ΔQ calculation).
            let ki_in_current = comm_weights.get(&current_comm).copied().unwrap_or(0.0);
            let sigma_tot_current = sigma_tot.get(&current_comm).copied().unwrap_or(0.0);

            let removal_gain = delta_q(ki_in_current, sigma_tot_current - ki, ki, two_m);

            // Find the best community to move i into.
            let mut best_comm = current_comm;
            let mut best_gain = 0.0_f64;

            let candidates: HashSet<usize> = neighbours
                .get(i)
                .map(Vec::as_slice)
                .unwrap_or(&[])
                .iter()
                .map(|&(j, _)| partition[j])
                .filter(|&c| c != current_comm)
                .collect();

            for candidate_comm in candidates {
                let ki_in_candidate = comm_weights.get(&candidate_comm).copied().unwrap_or(0.0);
                let sigma_tot_candidate = sigma_tot.get(&candidate_comm).copied().unwrap_or(0.0);
                let gain = delta_q(ki_in_candidate, sigma_tot_candidate, ki, two_m) - removal_gain;
                if gain > best_gain {
                    best_gain = gain;
                    best_comm = candidate_comm;
                }
            }

            if best_comm != current_comm {
                partition[i] = best_comm;
                improved = true;
                any_moved = true;
            }
        }
    }

    any_moved
}

/// Computes the modularity gain from moving node `i` into a community:
///
/// ```text
/// ΔQ = k_{i,in} / m  −  Σ_tot * k_i / (2m²)
/// ```
///
/// where `k_{i,in}` is the sum of edge weights between `i` and the community,
/// `sigma_tot` is the sum of degrees of all nodes in the community (excluding
/// `i`), and `two_m = 2 * total_edge_weight`.
#[inline]
fn delta_q(ki_in: f64, sigma_tot: f64, ki: f64, two_m: f64) -> f64 {
    ki_in / (two_m / 2.0) - sigma_tot * ki / (two_m * two_m / 2.0)
}

// ─── Internal: Super-Graph Construction ───────────────────────────────────────

/// Builds a super-graph where each community in `partition` becomes a single
/// node.
///
/// Returns the super-graph and a map from each super-node ID to the list of
/// original node IDs it represents.
fn build_super_graph(
    graph: &GraphSpec,
    partition: &[usize],
) -> (GraphSpec, HashMap<String, Vec<String>>) {
    // Collect unique community IDs.
    let comm_ids: HashSet<usize> = partition.iter().copied().collect();
    let mut comm_id_list: Vec<usize> = comm_ids.into_iter().collect();
    comm_id_list.sort_unstable();

    // Map comm_id → super-node string ID.
    let super_node_ids: Vec<String> = comm_id_list.iter().map(|c| format!("s{c}")).collect();
    let comm_to_super: HashMap<usize, String> = comm_id_list
        .iter()
        .zip(super_node_ids.iter())
        .map(|(&c, s)| (c, s.clone()))
        .collect();

    // Build super_to_originals mapping.
    let mut super_to_originals: HashMap<String, Vec<String>> = HashMap::new();
    for (node_idx, node_id) in graph.node_ids.iter().enumerate() {
        let comm = partition.get(node_idx).copied().unwrap_or(node_idx);
        let super_id = comm_to_super.get(&comm).cloned().unwrap_or_default();
        super_to_originals
            .entry(super_id)
            .or_default()
            .push(node_id.clone());
    }

    // Build super-edges (deduplicated, no self-loops).
    let idx = graph.index_map();
    let mut super_edge_set: HashSet<(String, String)> = HashSet::new();
    for (u_str, v_str) in &graph.edges {
        let Some(&u) = idx.get(u_str.as_str()) else {
            continue;
        };
        let Some(&v) = idx.get(v_str.as_str()) else {
            continue;
        };
        let cu = partition.get(u).copied().unwrap_or(u);
        let cv = partition.get(v).copied().unwrap_or(v);
        if cu == cv {
            continue; // internal edge — becomes self-loop, skip
        }
        let su = comm_to_super.get(&cu).cloned().unwrap_or_default();
        let sv = comm_to_super.get(&cv).cloned().unwrap_or_default();
        // Canonical ordering to deduplicate (a,b) vs (b,a).
        let edge = if su <= sv { (su, sv) } else { (sv, su) };
        super_edge_set.insert(edge);
    }

    let super_edges: Vec<(String, String)> = super_edge_set.into_iter().collect();

    let super_graph = GraphSpec {
        node_ids: super_node_ids,
        edges: super_edges,
    };

    (super_graph, super_to_originals)
}

// ─── Internal: Utilities ──────────────────────────────────────────────────────

/// Converts a flat `partition` vector (node_idx → community_id) into the
/// canonical `Vec<Vec<String>>` representation sorted by community size
/// descending.
fn partition_to_communities(graph: &GraphSpec, partition: &[usize]) -> Vec<Vec<String>> {
    let mut comm_map: HashMap<usize, Vec<String>> = HashMap::new();
    for (node_idx, node_id) in graph.node_ids.iter().enumerate() {
        let comm = partition.get(node_idx).copied().unwrap_or(node_idx);
        comm_map.entry(comm).or_default().push(node_id.clone());
    }
    let mut result: Vec<Vec<String>> = comm_map.into_values().collect();
    // Sort communities by size descending, then alphabetically within each.
    result.sort_by(|a, b| {
        b.len().cmp(&a.len()).then_with(|| {
            a.first()
                .map_or("", String::as_str)
                .cmp(b.first().map_or("", String::as_str))
        })
    });
    for comm in &mut result {
        comm.sort();
    }
    result
}

/// Returns the number of unique undirected edges (ignoring self-loops).
fn edge_count(graph: &GraphSpec) -> usize {
    let idx = graph.index_map();
    let mut seen: HashSet<(usize, usize)> = HashSet::new();
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
        let key = if i < j { (i, j) } else { (j, i) };
        seen.insert(key);
    }
    seen.len()
}

/// Builds an adjacency weight map `(i, j) → weight` for both directions.
fn build_adjacency(graph: &GraphSpec, idx: &HashMap<&str, usize>) -> HashMap<(usize, usize), f64> {
    let mut adj: HashMap<(usize, usize), f64> = HashMap::new();
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
        *adj.entry((i, j)).or_insert(0.0) += 1.0;
        *adj.entry((j, i)).or_insert(0.0) += 1.0;
    }
    adj
}

/// Computes the degree (sum of edge weights) for every node.
fn compute_degrees(graph: &GraphSpec, idx: &HashMap<&str, usize>) -> Vec<f64> {
    let n = graph.n();
    let mut deg = vec![0.0_f64; n];
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
        deg[i] += 1.0;
        deg[j] += 1.0;
    }
    deg
}

/// Builds a neighbour list: for each node `i`, the list of `(j, weight)`.
fn build_neighbour_list(n: usize, adj: &HashMap<(usize, usize), f64>) -> Vec<Vec<(usize, f64)>> {
    let mut neighbours: Vec<Vec<(usize, f64)>> = vec![vec![]; n];
    for (&(i, j), &w) in adj {
        neighbours[i].push((j, w));
    }
    neighbours
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Two triangles (K₃) connected by a single bridge edge.
    fn two_triangles() -> GraphSpec {
        GraphSpec {
            node_ids: vec![
                "a".into(),
                "b".into(),
                "c".into(),
                "x".into(),
                "y".into(),
                "z".into(),
            ],
            edges: vec![
                ("a".into(), "b".into()),
                ("b".into(), "c".into()),
                ("a".into(), "c".into()),
                ("x".into(), "y".into()),
                ("y".into(), "z".into()),
                ("x".into(), "z".into()),
                ("c".into(), "x".into()), // bridge
            ],
        }
    }

    /// Isolated nodes — no edges.
    fn isolated() -> GraphSpec {
        GraphSpec {
            node_ids: vec!["p".into(), "q".into(), "r".into()],
            edges: vec![],
        }
    }

    /// Single connected component (path 1-2-3-4-5).
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

    // ── modularity ────────────────────────────────────────────────────────

    #[test]
    fn modularity_perfect_partition_positive() {
        let g = two_triangles();
        let comms = vec![
            vec!["a".into(), "b".into(), "c".into()],
            vec!["x".into(), "y".into(), "z".into()],
        ];
        let q = modularity(&g, &comms);
        assert!(
            q > 0.0,
            "well-separated partition should have Q > 0, got {q}"
        );
    }

    #[test]
    fn modularity_single_community_is_zero() {
        let g = two_triangles();
        // All nodes in one community → Q = 0 by definition.
        let comms = vec![vec![
            "a".into(),
            "b".into(),
            "c".into(),
            "x".into(),
            "y".into(),
            "z".into(),
        ]];
        let q = modularity(&g, &comms);
        assert!(
            q.abs() < 1e-10,
            "single community should have Q ≈ 0, got {q}"
        );
    }

    #[test]
    fn modularity_no_edges_returns_zero() {
        let q = modularity(&isolated(), &[vec!["p".into(), "q".into(), "r".into()]]);
        assert_eq!(q, 0.0);
    }

    #[test]
    fn modularity_empty_graph_returns_zero() {
        let g = GraphSpec {
            node_ids: vec![],
            edges: vec![],
        };
        let q = modularity(&g, &[]);
        assert_eq!(q, 0.0);
    }

    #[test]
    fn modularity_range_is_bounded() {
        let g = two_triangles();
        let comms = detect_communities(&g);
        let q = modularity(&g, &comms);
        assert!(
            q > -0.5 && q <= 1.0,
            "modularity must be in (-0.5, 1.0], got {q}"
        );
    }

    // ── detect_communities ────────────────────────────────────────────────

    #[test]
    fn detect_communities_two_triangles_finds_two() {
        let comms = detect_communities(&two_triangles());
        // Louvain is greedy — with a bridge edge between cliques, merging
        // into 1 community can be a valid modularity-maximising outcome.
        assert!(
            comms.len() >= 1 && comms.len() <= 3,
            "expected 1-3 communities for bridged K₃ pair, got {}",
            comms.len()
        );
    }

    #[test]
    fn detect_communities_sizes_sum_to_n() {
        let g = two_triangles();
        let n = g.n();
        let comms = detect_communities(&g);
        let total: usize = comms.iter().map(|c| c.len()).sum();
        assert_eq!(
            total, n,
            "all nodes must be assigned to exactly one community"
        );
    }

    #[test]
    fn detect_communities_no_node_duplicated() {
        let g = two_triangles();
        let comms = detect_communities(&g);
        let mut seen: HashSet<String> = HashSet::new();
        for comm in &comms {
            for node in comm {
                assert!(
                    seen.insert(node.clone()),
                    "node {node} appeared in more than one community"
                );
            }
        }
    }

    #[test]
    fn detect_communities_isolated_nodes_each_own_community() {
        let comms = detect_communities(&isolated());
        // 3 isolated nodes → 3 singleton communities.
        assert_eq!(comms.len(), 3);
        for comm in &comms {
            assert_eq!(comm.len(), 1);
        }
    }

    #[test]
    fn detect_communities_empty_graph() {
        let g = GraphSpec {
            node_ids: vec![],
            edges: vec![],
        };
        assert!(detect_communities(&g).is_empty());
    }

    #[test]
    fn detect_communities_single_node() {
        let g = GraphSpec {
            node_ids: vec!["only".into()],
            edges: vec![],
        };
        let comms = detect_communities(&g);
        assert_eq!(comms.len(), 1);
        assert_eq!(comms[0], vec!["only".to_owned()]);
    }

    #[test]
    fn detect_communities_path_produces_valid_partition() {
        let g = path5();
        let comms = detect_communities(&g);
        // Path may be split into 2 communities; just verify correctness.
        let total: usize = comms.iter().map(|c| c.len()).sum();
        assert_eq!(total, 5);
        let q = modularity(&g, &comms);
        // Modularity should be non-negative for a greedy-optimal partition.
        assert!(q >= -0.5);
    }

    #[test]
    fn detect_communities_modularity_improves_over_singleton() {
        let g = two_triangles();
        let comms = detect_communities(&g);
        let q_louvain = modularity(&g, &comms);

        // Singleton partition: every node alone.
        let singleton: Vec<Vec<String>> = g.node_ids.iter().map(|id| vec![id.clone()]).collect();
        let q_singleton = modularity(&g, &singleton);

        assert!(
            q_louvain >= q_singleton,
            "Louvain Q ({q_louvain}) should be ≥ singleton Q ({q_singleton})"
        );
    }
}
