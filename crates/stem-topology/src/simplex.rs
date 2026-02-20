//! Simplicial complex types for TDA.
//!
//! ## T1 Grounding
//! - Simplex → N (quantity of vertices)
//! - SimplicialComplex → Σ (sum/collection)

use serde::{Deserialize, Serialize};

/// A simplex: a set of vertices with a filtration value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Simplex {
    /// Sorted vertex indices forming this simplex.
    pub vertices: Vec<usize>,
    /// The filtration parameter at which this simplex appears.
    pub filtration_value: f64,
}

impl Simplex {
    /// Create a new simplex, sorting vertices for canonical form.
    pub fn new(vertices: Vec<usize>, filtration_value: f64) -> Self {
        let mut v = vertices;
        v.sort_unstable();
        Self {
            vertices: v,
            filtration_value,
        }
    }

    /// Dimension = |vertices| - 1. A point is dimension 0, edge is 1, triangle is 2.
    pub fn dimension(&self) -> usize {
        if self.vertices.is_empty() {
            0
        } else {
            self.vertices.len() - 1
        }
    }

    /// Returns true if this simplex is a face of `other` (every vertex of self
    /// appears in other).
    pub fn is_face_of(&self, other: &Simplex) -> bool {
        self.vertices.iter().all(|v| other.vertices.contains(v))
    }
}

/// A simplicial complex: a collection of simplices closed under taking faces.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SimplicialComplex {
    /// All simplices in this complex.
    pub simplices: Vec<Simplex>,
}

impl SimplicialComplex {
    /// Create an empty simplicial complex.
    pub fn new() -> Self {
        Self {
            simplices: Vec::new(),
        }
    }

    /// Add a simplex to this complex.
    pub fn add_simplex(&mut self, simplex: Simplex) {
        self.simplices.push(simplex);
    }

    /// Sort simplices by filtration value (required before persistence computation).
    pub fn sort_by_filtration(&mut self) {
        self.simplices.sort_by(|a, b| {
            a.filtration_value
                .partial_cmp(&b.filtration_value)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /// Maximum dimension of any simplex in this complex.
    pub fn dimension(&self) -> usize {
        self.simplices
            .iter()
            .map(|s| s.dimension())
            .max()
            .unwrap_or(0)
    }

    /// Total number of simplices.
    pub fn simplex_count(&self) -> usize {
        self.simplices.len()
    }

    /// Get simplices of a specific dimension.
    pub fn simplices_of_dim(&self, dim: usize) -> Vec<&Simplex> {
        self.simplices
            .iter()
            .filter(|s| s.dimension() == dim)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simplex_dimension_vertex() {
        let s = Simplex::new(vec![0], 0.0);
        assert_eq!(s.dimension(), 0);
    }

    #[test]
    fn simplex_dimension_edge() {
        let s = Simplex::new(vec![0, 1], 1.0);
        assert_eq!(s.dimension(), 1);
    }

    #[test]
    fn simplex_dimension_triangle() {
        let s = Simplex::new(vec![0, 1, 2], 2.0);
        assert_eq!(s.dimension(), 2);
    }

    #[test]
    fn simplex_dimension_empty_returns_zero() {
        let s = Simplex::new(vec![], 0.0);
        assert_eq!(s.dimension(), 0);
    }

    #[test]
    fn simplex_vertices_sorted() {
        let s = Simplex::new(vec![3, 1, 2], 0.0);
        assert_eq!(s.vertices, vec![1, 2, 3]);
    }

    #[test]
    fn simplex_is_face_of_edge() {
        let vertex = Simplex::new(vec![0], 0.0);
        let edge = Simplex::new(vec![0, 1], 1.0);
        assert!(vertex.is_face_of(&edge));
    }

    #[test]
    fn simplex_is_face_of_self() {
        let s = Simplex::new(vec![0, 1], 1.0);
        assert!(s.is_face_of(&s.clone()));
    }

    #[test]
    fn simplex_not_face_of_disjoint() {
        let a = Simplex::new(vec![2, 3], 1.0);
        let b = Simplex::new(vec![0, 1], 1.0);
        assert!(!a.is_face_of(&b));
    }

    #[test]
    fn complex_sort_by_filtration() {
        let mut c = SimplicialComplex::new();
        c.add_simplex(Simplex::new(vec![0, 1], 2.0));
        c.add_simplex(Simplex::new(vec![0], 0.0));
        c.add_simplex(Simplex::new(vec![1], 0.0));
        c.sort_by_filtration();
        assert_eq!(c.simplices[0].filtration_value, 0.0);
        assert_eq!(c.simplices[2].filtration_value, 2.0);
    }

    #[test]
    fn complex_simplices_of_dim() {
        let mut c = SimplicialComplex::new();
        c.add_simplex(Simplex::new(vec![0], 0.0));
        c.add_simplex(Simplex::new(vec![1], 0.0));
        c.add_simplex(Simplex::new(vec![0, 1], 1.0));
        assert_eq!(c.simplices_of_dim(0).len(), 2);
        assert_eq!(c.simplices_of_dim(1).len(), 1);
        assert_eq!(c.simplices_of_dim(2).len(), 0);
    }

    #[test]
    fn complex_dimension() {
        let mut c = SimplicialComplex::new();
        c.add_simplex(Simplex::new(vec![0, 1, 2], 1.0));
        assert_eq!(c.dimension(), 2);
    }
}
