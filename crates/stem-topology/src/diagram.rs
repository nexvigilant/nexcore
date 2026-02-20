//! Persistence diagram types.

use serde::{Deserialize, Serialize};

/// A point in a persistence diagram: a (birth, death) pair at a given dimension.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersistencePoint {
    /// Filtration value at which this topological feature is born.
    pub birth: f64,
    /// Filtration value at which this topological feature dies.
    pub death: f64,
    /// Homological dimension (0 = connected components, 1 = loops, 2 = voids).
    pub dimension: usize,
}

impl PersistencePoint {
    /// Create a new persistence point.
    pub fn new(birth: f64, death: f64, dimension: usize) -> Self {
        Self {
            birth,
            death,
            dimension,
        }
    }

    /// Persistence = death - birth. Longer persistence = more stable feature.
    pub fn persistence(&self) -> f64 {
        self.death - self.birth
    }

    /// Persistence ratio relative to total filtration range.
    pub fn persistence_ratio(&self, max_filtration: f64) -> f64 {
        if max_filtration <= 0.0 {
            return 0.0;
        }
        self.persistence() / max_filtration
    }

    /// Returns true if this is a stable signal: persistence > `min_persistence`
    /// AND ratio > `min_ratio`.
    pub fn is_stable(&self, min_persistence: f64, min_ratio: f64, max_filtration: f64) -> bool {
        self.persistence() > min_persistence
            && self.persistence_ratio(max_filtration) > min_ratio
    }

    /// Returns true if this feature persists to infinity (essential class).
    pub fn is_infinite(&self) -> bool {
        self.death.is_infinite()
    }
}

/// A persistence diagram: collection of birth-death pairs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PersistenceDiagram {
    /// All (birth, death, dimension) points in this diagram.
    pub points: Vec<PersistencePoint>,
}

impl PersistenceDiagram {
    /// Create an empty persistence diagram.
    pub fn new() -> Self {
        Self { points: Vec::new() }
    }

    /// Add a persistence point to this diagram.
    pub fn add_point(&mut self, point: PersistencePoint) {
        self.points.push(point);
    }

    /// Get all points at a specific homological dimension.
    pub fn points_of_dim(&self, dim: usize) -> Vec<&PersistencePoint> {
        self.points.iter().filter(|p| p.dimension == dim).collect()
    }

    /// Filter to stable signals (high persistence).
    pub fn stable_signals(
        &self,
        min_persistence: f64,
        min_ratio: f64,
        max_filtration: f64,
    ) -> Vec<&PersistencePoint> {
        self.points
            .iter()
            .filter(|p| p.is_stable(min_persistence, min_ratio, max_filtration))
            .collect()
    }

    /// Count points per dimension. Returns `(dimension, count)` pairs.
    pub fn summary(&self) -> Vec<(usize, usize)> {
        let max_dim = self.points.iter().map(|p| p.dimension).max().unwrap_or(0);
        (0..=max_dim)
            .map(|d| (d, self.points_of_dim(d).len()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persistence_point_persistence() {
        let p = PersistencePoint::new(1.0, 3.0, 0);
        assert!((p.persistence() - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn persistence_point_ratio() {
        let p = PersistencePoint::new(0.0, 2.0, 0);
        assert!((p.persistence_ratio(4.0) - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn persistence_point_ratio_zero_max() {
        let p = PersistencePoint::new(0.0, 1.0, 0);
        assert_eq!(p.persistence_ratio(0.0), 0.0);
    }

    #[test]
    fn persistence_point_is_stable() {
        let p = PersistencePoint::new(0.0, 3.0, 0);
        assert!(p.is_stable(1.0, 0.2, 10.0));
        assert!(!p.is_stable(5.0, 0.0, 10.0)); // persistence too small
    }

    #[test]
    fn persistence_point_is_infinite() {
        let p = PersistencePoint::new(0.0, f64::INFINITY, 0);
        assert!(p.is_infinite());
        let q = PersistencePoint::new(0.0, 2.0, 0);
        assert!(!q.is_infinite());
    }

    #[test]
    fn diagram_points_of_dim() {
        let mut d = PersistenceDiagram::new();
        d.add_point(PersistencePoint::new(0.0, 1.0, 0));
        d.add_point(PersistencePoint::new(0.0, 2.0, 0));
        d.add_point(PersistencePoint::new(0.5, 1.5, 1));
        assert_eq!(d.points_of_dim(0).len(), 2);
        assert_eq!(d.points_of_dim(1).len(), 1);
        assert_eq!(d.points_of_dim(2).len(), 0);
    }

    #[test]
    fn diagram_summary() {
        let mut d = PersistenceDiagram::new();
        d.add_point(PersistencePoint::new(0.0, 1.0, 0));
        d.add_point(PersistencePoint::new(0.5, 1.5, 1));
        let s = d.summary();
        assert_eq!(s.len(), 2);
        assert_eq!(s[0], (0, 1));
        assert_eq!(s[1], (1, 1));
    }

    #[test]
    fn diagram_stable_signals() {
        let mut d = PersistenceDiagram::new();
        d.add_point(PersistencePoint::new(0.0, 5.0, 0)); // stable
        d.add_point(PersistencePoint::new(0.0, 0.1, 0)); // ephemeral
        let stable = d.stable_signals(0.5, 0.1, 10.0);
        assert_eq!(stable.len(), 1);
    }
}
