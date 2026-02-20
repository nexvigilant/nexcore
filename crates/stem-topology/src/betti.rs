//! Betti number computation from persistence diagrams.

use crate::diagram::PersistenceDiagram;
use serde::{Deserialize, Serialize};

/// Betti numbers at a specific filtration level.
///
/// `β_k` = number of independent k-dimensional holes alive at `filtration_value`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BettiNumbers {
    /// The filtration parameter at which these Betti numbers are evaluated.
    pub filtration_value: f64,
    /// `(dimension, betti_number)` pairs, one per dimension from 0 to max_dim.
    pub numbers: Vec<(usize, usize)>,
}

impl BettiNumbers {
    /// Get the Betti number at a specific dimension, or 0 if not present.
    pub fn at_dim(&self, dim: usize) -> usize {
        self.numbers
            .iter()
            .find(|(d, _)| *d == dim)
            .map(|(_, n)| *n)
            .unwrap_or(0)
    }
}

/// Compute Betti numbers at a given filtration value from a persistence diagram.
///
/// `β_k(t)` = number of points in dimension-k diagram with `birth ≤ t < death`.
pub fn betti_numbers(diagram: &PersistenceDiagram, at_filtration: f64) -> BettiNumbers {
    let max_dim = diagram.points.iter().map(|p| p.dimension).max().unwrap_or(0);
    let numbers: Vec<(usize, usize)> = (0..=max_dim)
        .map(|dim| {
            let count = diagram
                .points
                .iter()
                .filter(|p| {
                    p.dimension == dim && p.birth <= at_filtration && p.death > at_filtration
                })
                .count();
            (dim, count)
        })
        .collect();

    BettiNumbers {
        filtration_value: at_filtration,
        numbers,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagram::PersistencePoint;

    fn sample_diagram() -> PersistenceDiagram {
        let mut d = PersistenceDiagram::new();
        // Betti-0: born at 0, dies at 1
        d.add_point(PersistencePoint::new(0.0, 1.0, 0));
        // Betti-0: born at 0, lives forever (essential component)
        d.add_point(PersistencePoint::new(0.0, f64::INFINITY, 0));
        // Betti-1: loop born at 0.5, dies at 2.0
        d.add_point(PersistencePoint::new(0.5, 2.0, 1));
        d
    }

    #[test]
    fn betti_numbers_at_filtration_zero() {
        let d = sample_diagram();
        let b = betti_numbers(&d, 0.0);
        // Both dim-0 points are alive at t=0 (birth <= 0, death > 0)
        assert_eq!(b.at_dim(0), 2);
        // dim-1 not yet born (birth = 0.5 > 0.0)
        assert_eq!(b.at_dim(1), 0);
    }

    #[test]
    fn betti_numbers_at_filtration_0_5() {
        let d = sample_diagram();
        let b = betti_numbers(&d, 0.5);
        // dim-0 at t=1.5: only the essential one is alive (the other died at 1)
        let b_mid = betti_numbers(&d, 1.5);
        assert_eq!(b_mid.at_dim(0), 1);
        // Betti-1: born at 0.5, alive at 0.5
        assert_eq!(b.at_dim(1), 1);
    }

    #[test]
    fn betti_numbers_at_filtration_after_loop_dies() {
        let d = sample_diagram();
        let b = betti_numbers(&d, 3.0);
        // Loop died at 2.0, so Betti-1 = 0
        assert_eq!(b.at_dim(1), 0);
        // Essential dim-0 still alive
        assert_eq!(b.at_dim(0), 1);
    }

    #[test]
    fn betti_numbers_empty_diagram() {
        let d = PersistenceDiagram::new();
        let b = betti_numbers(&d, 1.0);
        assert!(b.numbers.is_empty());
    }

    #[test]
    fn betti_at_dim_missing_returns_zero() {
        let d = sample_diagram();
        let b = betti_numbers(&d, 1.0);
        assert_eq!(b.at_dim(99), 0);
    }
}
