//! Vietoris-Rips filtration construction.

use crate::simplex::{Simplex, SimplicialComplex};
use serde::{Deserialize, Serialize};

/// Distance matrix for building Vietoris-Rips complex.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistanceMatrix {
    /// Number of points.
    pub size: usize,
    /// Pairwise distances (size × size).
    pub distances: Vec<Vec<f64>>,
    /// Optional point labels.
    pub labels: Option<Vec<String>>,
}

impl DistanceMatrix {
    /// Create a distance matrix from a square matrix of pairwise distances.
    pub fn new(distances: Vec<Vec<f64>>) -> Self {
        let size = distances.len();
        Self {
            size,
            distances,
            labels: None,
        }
    }

    /// Attach labels to the points.
    pub fn with_labels(mut self, labels: Vec<String>) -> Self {
        self.labels = Some(labels);
        self
    }

    /// Get the distance between points `i` and `j`.
    pub fn distance(&self, i: usize, j: usize) -> f64 {
        self.distances[i][j]
    }
}

/// Build a Vietoris-Rips complex from a distance matrix.
///
/// A simplex {v0, v1, ..., vk} is included at filtration value
/// `max(d(vi, vj))` for all pairs i, j.
///
/// # Arguments
/// - `dm`: pairwise distance matrix
/// - `max_dim`: maximum simplex dimension to include (1 = edges only, 2 = triangles, …)
/// - `max_filtration`: upper bound on filtration parameter
pub fn vietoris_rips(dm: &DistanceMatrix, max_dim: usize, max_filtration: f64) -> SimplicialComplex {
    let n = dm.size;
    let mut complex = SimplicialComplex::new();

    // 0-simplices (vertices) appear at filtration 0
    for i in 0..n {
        complex.add_simplex(Simplex::new(vec![i], 0.0));
    }

    // 1-simplices (edges)
    for i in 0..n {
        for j in (i + 1)..n {
            let d = dm.distance(i, j);
            if d <= max_filtration {
                complex.add_simplex(Simplex::new(vec![i, j], d));
            }
        }
    }

    // 2-simplices (triangles) and higher
    if max_dim >= 2 {
        for i in 0..n {
            for j in (i + 1)..n {
                for k in (j + 1)..n {
                    let d_ij = dm.distance(i, j);
                    let d_ik = dm.distance(i, k);
                    let d_jk = dm.distance(j, k);
                    let max_d = d_ij.max(d_ik).max(d_jk);
                    if max_d <= max_filtration {
                        complex.add_simplex(Simplex::new(vec![i, j, k], max_d));
                    }
                }
            }
        }
    }

    complex.sort_by_filtration();
    complex
}

#[cfg(test)]
mod tests {
    use super::*;

    fn three_point_dm() -> DistanceMatrix {
        // Equilateral triangle with side 1.0
        DistanceMatrix::new(vec![
            vec![0.0, 1.0, 1.0],
            vec![1.0, 0.0, 1.0],
            vec![1.0, 1.0, 0.0],
        ])
    }

    #[test]
    fn vietoris_rips_vertices() {
        let dm = three_point_dm();
        let c = vietoris_rips(&dm, 0, 2.0);
        assert_eq!(c.simplices_of_dim(0).len(), 3);
        assert_eq!(c.simplices_of_dim(1).len(), 0);
    }

    #[test]
    fn vietoris_rips_edges_at_threshold_1() {
        let dm = three_point_dm();
        let c = vietoris_rips(&dm, 1, 1.0);
        assert_eq!(c.simplices_of_dim(0).len(), 3);
        assert_eq!(c.simplices_of_dim(1).len(), 3);
        assert_eq!(c.simplices_of_dim(2).len(), 0);
    }

    #[test]
    fn vietoris_rips_triangle_at_max_dim_2() {
        let dm = three_point_dm();
        let c = vietoris_rips(&dm, 2, 1.0);
        // Should include 3 vertices + 3 edges + 1 triangle
        assert_eq!(c.simplex_count(), 7);
        assert_eq!(c.simplices_of_dim(2).len(), 1);
    }

    #[test]
    fn vietoris_rips_sorted_by_filtration() {
        let dm = three_point_dm();
        let c = vietoris_rips(&dm, 2, 2.0);
        for w in c.simplices.windows(2) {
            assert!(
                w[0].filtration_value <= w[1].filtration_value,
                "not sorted: {} > {}",
                w[0].filtration_value,
                w[1].filtration_value
            );
        }
    }

    #[test]
    fn vietoris_rips_with_labels() {
        let dm = three_point_dm()
            .with_labels(vec!["A".to_string(), "B".to_string(), "C".to_string()]);
        assert!(dm.labels.is_some());
        let c = vietoris_rips(&dm, 1, 2.0);
        assert_eq!(c.simplices_of_dim(0).len(), 3);
    }
}
