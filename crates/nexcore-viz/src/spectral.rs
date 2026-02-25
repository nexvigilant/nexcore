//! Spectral Graph Analysis
//!
//! Pure `f64` spectral analysis of graphs without external linear-algebra
//! dependencies.  All matrix representations use `Vec<Vec<f64>>` (row-major,
//! n×n).
//!
//! # Key Operations
//!
//! | Function | Returns | Description |
//! |----------|---------|-------------|
//! | [`adjacency_matrix`] | `Vec<Vec<f64>>` | Symmetric 0/1 adjacency |
//! | [`degree_matrix`] | `Vec<f64>` | Diagonal degree vector |
//! | [`laplacian_matrix`] | `Vec<Vec<f64>>` | D − A |
//! | [`power_iteration`] | `(f64, Vec<f64>)` | Dominant eigenvalue + eigenvector |
//! | [`algebraic_connectivity`] | `f64` | Fiedler value (λ₂) |
//!
//! # Example
//!
//! ```rust
//! use nexcore_viz::spectral::{GraphSpec, algebraic_connectivity};
//!
//! let g = GraphSpec {
//!     node_ids: vec!["a".into(), "b".into(), "c".into()],
//!     edges: vec![
//!         ("a".into(), "b".into()),
//!         ("b".into(), "c".into()),
//!     ],
//! };
//! let lambda2 = algebraic_connectivity(&g);
//! assert!(lambda2 > 0.0, "connected graph has positive Fiedler value");
//! ```

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── Public Types ────────────────────────────────────────────────────────────

/// A simple undirected graph described by string node IDs and (from, to) edges.
///
/// Edges are treated as **undirected**: an edge `(u, v)` also implies `(v, u)`
/// for all spectral and centrality calculations.  Self-loops are ignored.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphSpec {
    /// Ordered list of unique node identifiers.
    pub node_ids: Vec<String>,
    /// Edge list.  Each tuple is `(source_id, target_id)`.
    pub edges: Vec<(String, String)>,
}

impl GraphSpec {
    /// Returns the number of nodes.
    #[must_use]
    pub fn n(&self) -> usize {
        self.node_ids.len()
    }

    /// Builds a map from node ID to its 0-based index in [`Self::node_ids`].
    #[must_use]
    pub fn index_map(&self) -> HashMap<&str, usize> {
        self.node_ids
            .iter()
            .enumerate()
            .map(|(i, id)| (id.as_str(), i))
            .collect()
    }
}

// ─── Matrix Construction ─────────────────────────────────────────────────────

/// Builds the **symmetric adjacency matrix** A (n×n).
///
/// `A[i][j] = 1.0` if there is an edge between node `i` and node `j`; `0.0`
/// otherwise.  Self-loops are ignored.  Unknown node IDs in edges are skipped
/// silently.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::spectral::{GraphSpec, adjacency_matrix};
///
/// let g = GraphSpec {
///     node_ids: vec!["x".into(), "y".into()],
///     edges: vec![("x".into(), "y".into())],
/// };
/// let a = adjacency_matrix(&g);
/// assert_eq!(a[0][1], 1.0);
/// assert_eq!(a[1][0], 1.0);
/// assert_eq!(a[0][0], 0.0);
/// ```
#[must_use]
pub fn adjacency_matrix(graph: &GraphSpec) -> Vec<Vec<f64>> {
    let n = graph.n();
    let idx = graph.index_map();
    let mut a = vec![vec![0.0_f64; n]; n];

    for (u, v) in &graph.edges {
        let Some(&i) = idx.get(u.as_str()) else {
            continue;
        };
        let Some(&j) = idx.get(v.as_str()) else {
            continue;
        };
        if i == j {
            continue; // ignore self-loops
        }
        a[i][j] = 1.0;
        a[j][i] = 1.0;
    }
    a
}

/// Returns the **diagonal degree vector** D (length n).
///
/// `D[i]` is the degree of node `i` — the number of distinct neighbours
/// (self-loops excluded).
///
/// # Example
///
/// ```rust
/// use nexcore_viz::spectral::{GraphSpec, degree_matrix};
///
/// let g = GraphSpec {
///     node_ids: vec!["a".into(), "b".into(), "c".into()],
///     edges: vec![("a".into(), "b".into()), ("a".into(), "c".into())],
/// };
/// let d = degree_matrix(&g);
/// assert_eq!(d[0], 2.0); // "a" has degree 2
/// assert_eq!(d[1], 1.0);
/// assert_eq!(d[2], 1.0);
/// ```
#[must_use]
pub fn degree_matrix(graph: &GraphSpec) -> Vec<f64> {
    let a = adjacency_matrix(graph);
    a.iter().map(|row| row.iter().sum()).collect()
}

/// Builds the **graph Laplacian** L = D − A (n×n).
///
/// The Laplacian is positive semi-definite.  Its smallest eigenvalue is always
/// 0 (for a connected graph, with multiplicity 1).  The **second-smallest**
/// eigenvalue is the *Fiedler value* returned by [`algebraic_connectivity`].
///
/// # Example
///
/// ```rust
/// use nexcore_viz::spectral::{GraphSpec, laplacian_matrix};
///
/// let g = GraphSpec {
///     node_ids: vec!["p".into(), "q".into()],
///     edges: vec![("p".into(), "q".into())],
/// };
/// let l = laplacian_matrix(&g);
/// // L = [[1,-1],[-1,1]] for a single edge
/// assert_eq!(l[0][0],  1.0);
/// assert_eq!(l[0][1], -1.0);
/// assert_eq!(l[1][0], -1.0);
/// assert_eq!(l[1][1],  1.0);
/// ```
#[must_use]
pub fn laplacian_matrix(graph: &GraphSpec) -> Vec<Vec<f64>> {
    let n = graph.n();
    let a = adjacency_matrix(graph);
    let d = degree_matrix(graph);

    let mut l = vec![vec![0.0_f64; n]; n];
    for i in 0..n {
        for j in 0..n {
            l[i][j] = if i == j { d[i] } else { -a[i][j] };
        }
    }
    l
}

// ─── Eigenvalue Computation ───────────────────────────────────────────────────

/// Computes the **dominant eigenvalue and eigenvector** of `matrix` using the
/// power iteration method.
///
/// Convergence is declared when the Euclidean norm of the change in the
/// eigenvector estimate falls below `tolerance`, or after `max_iter` steps,
/// whichever comes first.
///
/// Returns `(eigenvalue, eigenvector)`.  Returns `(0.0, zeros)` for an empty
/// or all-zero matrix.
///
/// # Arguments
///
/// * `matrix`   – Square n×n matrix (row-major).
/// * `max_iter` – Maximum number of iterations before early return.
/// * `tolerance` – Convergence threshold on the eigenvector change norm.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::spectral::{GraphSpec, adjacency_matrix, power_iteration};
///
/// let g = GraphSpec {
///     node_ids: vec!["a".into(), "b".into(), "c".into()],
///     edges: vec![("a".into(), "b".into()), ("b".into(), "c".into()),
///                 ("a".into(), "c".into())],
/// };
/// let a = adjacency_matrix(&g);
/// let (lambda, _v) = power_iteration(&a, 1000, 1e-9);
/// // K₃ has dominant eigenvalue 2.0
/// assert!((lambda - 2.0).abs() < 1e-6);
/// ```
#[must_use]
pub fn power_iteration(matrix: &[Vec<f64>], max_iter: usize, tolerance: f64) -> (f64, Vec<f64>) {
    let n = matrix.len();
    if n == 0 {
        return (0.0, vec![]);
    }

    // Initialise to uniform vector.
    let init = 1.0 / (n as f64).sqrt();
    let mut v: Vec<f64> = vec![init; n];

    let mut eigenvalue = 0.0_f64;

    for _ in 0..max_iter {
        // w = A * v
        let mut w = vec![0.0_f64; n];
        for i in 0..n {
            for (j, &vj) in v.iter().enumerate() {
                w[i] += matrix[i].get(j).copied().unwrap_or(0.0) * vj;
            }
        }

        // Rayleigh quotient: λ ≈ v·w
        let new_lambda: f64 = v.iter().zip(w.iter()).map(|(vi, wi)| vi * wi).sum();

        // Normalise w → new eigenvector estimate.
        let norm: f64 = w.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm < f64::EPSILON {
            break;
        }
        let w_norm: Vec<f64> = w.iter().map(|x| x / norm).collect();

        // Check convergence.
        let delta: f64 = v
            .iter()
            .zip(w_norm.iter())
            .map(|(a, b)| (a - b) * (a - b))
            .sum::<f64>()
            .sqrt();

        v = w_norm;
        eigenvalue = new_lambda;

        if delta < tolerance {
            break;
        }
    }

    (eigenvalue, v)
}

/// Computes the **Fiedler value** (algebraic connectivity, λ₂) — the
/// second-smallest eigenvalue of the graph Laplacian.
///
/// Uses inverse power iteration on a shifted Laplacian `(L + σI)` to isolate
/// λ₂ after removing the trivial zero eigenvalue.
///
/// A higher value means a more robustly connected graph.  Returns `0.0` for
/// graphs with fewer than two nodes or a disconnected graph.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::spectral::{GraphSpec, algebraic_connectivity};
///
/// // Path graph: 1 — 2 — 3 — 4 — 5
/// let g = GraphSpec {
///     node_ids: (1..=5).map(|i| i.to_string()).collect(),
///     edges: vec![
///         ("1".into(), "2".into()), ("2".into(), "3".into()),
///         ("3".into(), "4".into()), ("4".into(), "5".into()),
///     ],
/// };
/// let ac = algebraic_connectivity(&g);
/// assert!(ac > 0.0);
/// assert!(ac < 1.0); // path graph has low connectivity
/// ```
#[must_use]
pub fn algebraic_connectivity(graph: &GraphSpec) -> f64 {
    let n = graph.n();
    if n < 2 {
        return 0.0;
    }

    let l = laplacian_matrix(graph);

    // The Laplacian always has 0 as its smallest eigenvalue.
    // We find the dominant eigenvalue of (σI - L) where σ is slightly above
    // the spectral radius, so that the *largest* eigenvalue of (σI - L)
    // corresponds to the *second smallest* eigenvalue of L.
    //
    // Spectral radius bound: max row sum (Gershgorin).
    let sigma = l
        .iter()
        .map(|row| row.iter().map(|x| x.abs()).sum::<f64>())
        .fold(0.0_f64, f64::max)
        + 1.0;

    // Build shifted matrix M = σI - L.
    let mut m = vec![vec![0.0_f64; n]; n];
    for i in 0..n {
        for j in 0..n {
            m[i][j] = if i == j { sigma - l[i][j] } else { -l[i][j] };
        }
    }

    // Project out the all-ones eigenvector (λ=0 of L → λ=σ of M).
    // We do this by deflating: run power iteration but subtract the
    // component along 1/√n after each matrix-vector product.
    let inv_sqrt_n = 1.0 / (n as f64).sqrt();
    let max_iter = 2000;
    let tol = 1e-10_f64;

    let init = 1.0 / (n as f64).sqrt();
    let mut v: Vec<f64> = (0..n)
        .map(|i| if i == 0 { init + 0.001 } else { init })
        .collect();
    orthogonalise_ones(&mut v, inv_sqrt_n);
    normalise_vec(&mut v);

    let mut lambda_m = 0.0_f64;

    for _ in 0..max_iter {
        // w = M * v
        let mut w = vec![0.0_f64; n];
        for i in 0..n {
            for j in 0..n {
                w[i] += m[i][j] * v[j];
            }
        }

        // Deflate: remove component along the all-ones vector.
        orthogonalise_ones(&mut w, inv_sqrt_n);

        let new_lambda: f64 = v.iter().zip(w.iter()).map(|(vi, wi)| vi * wi).sum();

        let norm: f64 = w.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm < f64::EPSILON {
            break;
        }
        let w_norm: Vec<f64> = w.iter().map(|x| x / norm).collect();

        let delta: f64 = v
            .iter()
            .zip(w_norm.iter())
            .map(|(a, b)| (a - b) * (a - b))
            .sum::<f64>()
            .sqrt();

        v = w_norm;
        lambda_m = new_lambda;

        if delta < tol {
            break;
        }
    }

    // λ₂(L) = σ - λ_dominant(M)
    let lambda2 = sigma - lambda_m;
    // Clamp to 0 to avoid tiny negative artefacts from floating-point drift.
    lambda2.max(0.0)
}

// ─── Internal Helpers ─────────────────────────────────────────────────────────

/// Subtracts the projection of `v` onto the all-ones unit vector (1/√n · 1).
fn orthogonalise_ones(v: &mut [f64], inv_sqrt_n: f64) {
    let dot: f64 = v.iter().map(|x| x * inv_sqrt_n).sum::<f64>();
    for x in v.iter_mut() {
        *x -= dot * inv_sqrt_n;
    }
}

/// Normalises `v` in-place to unit Euclidean norm.  No-op if the norm is zero.
fn normalise_vec(v: &mut [f64]) {
    let norm: f64 = v.iter().map(|x| x * x).sum::<f64>().sqrt();
    if norm > f64::EPSILON {
        for x in v.iter_mut() {
            *x /= norm;
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Complete graph K₃ (triangle).
    fn triangle() -> GraphSpec {
        GraphSpec {
            node_ids: vec!["a".into(), "b".into(), "c".into()],
            edges: vec![
                ("a".into(), "b".into()),
                ("b".into(), "c".into()),
                ("a".into(), "c".into()),
            ],
        }
    }

    /// Path graph 1 — 2 — 3 — 4 — 5.
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

    /// Star graph: centre connected to 4 leaves.
    fn star() -> GraphSpec {
        GraphSpec {
            node_ids: vec![
                "c".into(),
                "l1".into(),
                "l2".into(),
                "l3".into(),
                "l4".into(),
            ],
            edges: vec![
                ("c".into(), "l1".into()),
                ("c".into(), "l2".into()),
                ("c".into(), "l3".into()),
                ("c".into(), "l4".into()),
            ],
        }
    }

    // ── adjacency_matrix ──────────────────────────────────────────────────

    #[test]
    fn adjacency_matrix_triangle_symmetric() {
        let a = adjacency_matrix(&triangle());
        assert_eq!(a.len(), 3);
        // Diagonal is 0
        for i in 0..3 {
            assert_eq!(a[i][i], 0.0);
        }
        // All off-diagonals are 1 (K₃)
        for i in 0..3 {
            for j in 0..3 {
                if i != j {
                    assert_eq!(a[i][j], 1.0, "K₃ should be fully connected");
                }
            }
        }
        // Symmetry
        for i in 0..3 {
            for j in 0..3 {
                assert_eq!(a[i][j], a[j][i]);
            }
        }
    }

    #[test]
    fn adjacency_matrix_ignores_self_loops() {
        let g = GraphSpec {
            node_ids: vec!["x".into()],
            edges: vec![("x".into(), "x".into())],
        };
        let a = adjacency_matrix(&g);
        assert_eq!(a[0][0], 0.0);
    }

    #[test]
    fn adjacency_matrix_ignores_unknown_ids() {
        let g = GraphSpec {
            node_ids: vec!["a".into(), "b".into()],
            edges: vec![("a".into(), "UNKNOWN".into())],
        };
        let a = adjacency_matrix(&g);
        // No edge should have been recorded.
        assert_eq!(a[0][1], 0.0);
        assert_eq!(a[1][0], 0.0);
    }

    // ── degree_matrix ─────────────────────────────────────────────────────

    #[test]
    fn degree_matrix_path5() {
        let d = degree_matrix(&path5());
        // Endpoints have degree 1, interior nodes have degree 2.
        assert_eq!(d[0], 1.0);
        assert_eq!(d[1], 2.0);
        assert_eq!(d[2], 2.0);
        assert_eq!(d[3], 2.0);
        assert_eq!(d[4], 1.0);
    }

    #[test]
    fn degree_matrix_star_centre() {
        let d = degree_matrix(&star());
        // Centre has degree 4, leaves degree 1.
        assert_eq!(d[0], 4.0);
        for i in 1..5 {
            assert_eq!(d[i], 1.0);
        }
    }

    // ── laplacian_matrix ──────────────────────────────────────────────────

    #[test]
    fn laplacian_row_sums_are_zero() {
        let l = laplacian_matrix(&path5());
        for row in &l {
            let sum: f64 = row.iter().sum();
            assert!(sum.abs() < 1e-12, "L row sum should be 0, got {sum}");
        }
    }

    #[test]
    fn laplacian_is_symmetric() {
        let l = laplacian_matrix(&triangle());
        let n = l.len();
        for i in 0..n {
            for j in 0..n {
                assert!(
                    (l[i][j] - l[j][i]).abs() < 1e-12,
                    "L should be symmetric at [{i}][{j}]"
                );
            }
        }
    }

    #[test]
    fn laplacian_single_edge() {
        let g = GraphSpec {
            node_ids: vec!["p".into(), "q".into()],
            edges: vec![("p".into(), "q".into())],
        };
        let l = laplacian_matrix(&g);
        assert_eq!(l[0][0], 1.0);
        assert_eq!(l[0][1], -1.0);
        assert_eq!(l[1][0], -1.0);
        assert_eq!(l[1][1], 1.0);
    }

    // ── power_iteration ───────────────────────────────────────────────────

    #[test]
    fn power_iteration_k3_dominant_eigenvalue() {
        // K₃ adjacency has dominant eigenvalue 2.0.
        let a = adjacency_matrix(&triangle());
        let (lambda, v) = power_iteration(&a, 1000, 1e-9);
        assert!(
            (lambda - 2.0).abs() < 1e-5,
            "K₃ dominant eigenvalue should be 2.0, got {lambda}"
        );
        // Eigenvector should be unit-normalised.
        let norm: f64 = v.iter().map(|x| x * x).sum::<f64>().sqrt();
        assert!((norm - 1.0).abs() < 1e-6, "eigenvector should be unit-norm");
    }

    #[test]
    fn power_iteration_empty_matrix_returns_zero() {
        let (lambda, v) = power_iteration(&[], 100, 1e-9);
        assert_eq!(lambda, 0.0);
        assert!(v.is_empty());
    }

    // ── algebraic_connectivity ────────────────────────────────────────────

    #[test]
    fn algebraic_connectivity_triangle_positive() {
        let ac = algebraic_connectivity(&triangle());
        assert!(ac > 0.0, "connected graph should have positive λ₂");
    }

    #[test]
    fn algebraic_connectivity_k3_is_three() {
        // K₃ Laplacian eigenvalues are 0, 3, 3.  λ₂ = 3.0.
        let ac = algebraic_connectivity(&triangle());
        assert!(
            (ac - 3.0).abs() < 0.05,
            "K₃ algebraic connectivity should be ~3.0, got {ac}"
        );
    }

    #[test]
    fn algebraic_connectivity_path_lower_than_complete() {
        let ac_path = algebraic_connectivity(&path5());
        let k5 = GraphSpec {
            node_ids: (1..=5).map(|i| i.to_string()).collect(),
            edges: vec![
                ("1".into(), "2".into()),
                ("1".into(), "3".into()),
                ("1".into(), "4".into()),
                ("1".into(), "5".into()),
                ("2".into(), "3".into()),
                ("2".into(), "4".into()),
                ("2".into(), "5".into()),
                ("3".into(), "4".into()),
                ("3".into(), "5".into()),
                ("4".into(), "5".into()),
            ],
        };
        let ac_complete = algebraic_connectivity(&k5);
        assert!(
            ac_path < ac_complete,
            "path graph should have lower connectivity than K₅: {ac_path} < {ac_complete}"
        );
    }

    #[test]
    fn algebraic_connectivity_single_node_is_zero() {
        let g = GraphSpec {
            node_ids: vec!["solo".into()],
            edges: vec![],
        };
        assert_eq!(algebraic_connectivity(&g), 0.0);
    }

    #[test]
    fn algebraic_connectivity_non_negative() {
        // Should never return a negative number due to floating-point noise.
        let g = GraphSpec {
            node_ids: vec!["a".into(), "b".into()],
            edges: vec![],
        };
        assert!(algebraic_connectivity(&g) >= 0.0);
    }
}
