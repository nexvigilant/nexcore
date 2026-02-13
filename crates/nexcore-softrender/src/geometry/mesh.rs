//! Mesh: triangle list — the universal rendering primitive
//!
//! Everything is triangles. Rects are 2 triangles. Circles are N triangles.
//! The GPU doesn't know what a "rectangle" is. It only knows triangles.

use super::vertex::Vertex;

#[derive(Debug, Clone)]
pub struct Triangle {
    pub v0: Vertex,
    pub v1: Vertex,
    pub v2: Vertex,
}

impl Triangle {
    pub fn new(v0: Vertex, v1: Vertex, v2: Vertex) -> Self {
        Self { v0, v1, v2 }
    }
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub triangles: Vec<Triangle>,
}

impl Mesh {
    pub fn new() -> Self {
        Self {
            triangles: Vec::new(),
        }
    }

    pub fn with_capacity(n: usize) -> Self {
        Self {
            triangles: Vec::with_capacity(n),
        }
    }

    pub fn push(&mut self, tri: Triangle) {
        self.triangles.push(tri);
    }

    pub fn triangle_count(&self) -> usize {
        self.triangles.len()
    }

    pub fn vertex_count(&self) -> usize {
        self.triangles.len() * 3
    }

    /// Merge another mesh into this one
    pub fn extend(&mut self, other: &Mesh) {
        self.triangles.extend_from_slice(&other.triangles);
    }
}

impl Default for Mesh {
    fn default() -> Self {
        Self::new()
    }
}

impl FromIterator<Triangle> for Mesh {
    fn from_iter<I: IntoIterator<Item = Triangle>>(iter: I) -> Self {
        Self {
            triangles: iter.into_iter().collect(),
        }
    }
}
