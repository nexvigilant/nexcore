//! Nash equilibrium solver — 2×2 payoff matrix with mixed strategy computation.
//!
//! Merged from ferro-forge-engine. T1 Primitives: N(Quantity) + κ(Comparison) + μ(Mapping) + Σ(Sum)

use serde::{Deserialize, Serialize};

/// A 2×2 payoff matrix with row/column labels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoffMatrix {
    /// Row labels (player strategies)
    pub row_labels: [String; 2],
    /// Column labels (opponent strategies)
    pub col_labels: [String; 2],
    /// Payoff values: [[a, b], [c, d]]
    pub values: [[f64; 2]; 2],
}

impl PayoffMatrix {
    /// Create a new payoff matrix.
    pub fn new(row_labels: [&str; 2], col_labels: [&str; 2], values: [[f64; 2]; 2]) -> Self {
        Self {
            row_labels: [row_labels[0].to_string(), row_labels[1].to_string()],
            col_labels: [col_labels[0].to_string(), col_labels[1].to_string()],
            values,
        }
    }

    /// Expected payoff for each row strategy (uniform opponent).
    pub fn row_expected_payoffs(&self) -> [f64; 2] {
        [
            (self.values[0][0] + self.values[0][1]) / 2.0,
            (self.values[1][0] + self.values[1][1]) / 2.0,
        ]
    }

    /// Best response for each column (which row maximizes payoff).
    pub fn col_best_responses(&self) -> [usize; 2] {
        [
            if self.values[0][0] >= self.values[1][0] {
                0
            } else {
                1
            },
            if self.values[0][1] >= self.values[1][1] {
                0
            } else {
                1
            },
        ]
    }

    /// Dominant row strategy (if one exists).
    pub fn dominant_row(&self) -> Option<usize> {
        if self.values[0][0] > self.values[1][0] && self.values[0][1] > self.values[1][1] {
            Some(0)
        } else if self.values[1][0] > self.values[0][0] && self.values[1][1] > self.values[0][1] {
            Some(1)
        } else {
            None
        }
    }

    /// Minimax value (maximum of row minimums).
    pub fn maximin(&self) -> (usize, f64) {
        let row_mins = [
            self.values[0][0].min(self.values[0][1]),
            self.values[1][0].min(self.values[1][1]),
        ];
        if row_mins[0] >= row_mins[1] {
            (0, row_mins[0])
        } else {
            (1, row_mins[1])
        }
    }
}

/// Mixed strategy Nash equilibrium solver for 2×2 games.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NashSolver {
    /// Row player mixed strategy [p(row0), p(row1)]
    pub row_strategy: [f64; 2],
    /// Column player mixed strategy [p(col0), p(col1)]
    pub col_strategy: [f64; 2],
    /// Expected payoff at equilibrium
    pub expected_payoff: f64,
    /// Whether a pure strategy equilibrium exists
    pub pure_equilibrium: bool,
}

impl NashSolver {
    /// Solve 2×2 game via analytical mixed strategy computation.
    ///
    /// For matrix [[a,b],[c,d]], row player mixes with:
    /// p = (d - c) / (a - b - c + d)
    pub fn solve(matrix: &PayoffMatrix) -> Self {
        let a = matrix.values[0][0];
        let b = matrix.values[0][1];
        let c = matrix.values[1][0];
        let d = matrix.values[1][1];

        let denom = a - b - c + d;

        if denom.abs() < 1e-10 {
            return Self {
                row_strategy: [0.5, 0.5],
                col_strategy: [0.5, 0.5],
                expected_payoff: (a + b + c + d) / 4.0,
                pure_equilibrium: false,
            };
        }

        let p = ((d - c) / denom).clamp(0.0, 1.0);
        let q = ((d - b) / denom).clamp(0.0, 1.0);

        let pure_equilibrium = (p - 0.0).abs() < 1e-10 || (p - 1.0).abs() < 1e-10;

        let expected_payoff = p * (q * a + (1.0 - q) * b) + (1.0 - p) * (q * c + (1.0 - q) * d);

        Self {
            row_strategy: [p, 1.0 - p],
            col_strategy: [q, 1.0 - q],
            expected_payoff,
            pure_equilibrium,
        }
    }

    /// Which row strategy has higher weight?
    pub fn preferred_row(&self) -> usize {
        if self.row_strategy[0] >= self.row_strategy[1] {
            0
        } else {
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payoff_matrix_expected() {
        let m = PayoffMatrix::new(
            ["Monolith", "Modular"],
            ["Speed", "Quality"],
            [[7.0, 3.0], [4.0, 8.0]],
        );
        let expected = m.row_expected_payoffs();
        assert!((expected[0] - 5.0).abs() < 1e-10);
        assert!((expected[1] - 6.0).abs() < 1e-10);
    }

    #[test]
    fn test_no_dominant_strategy() {
        let m = PayoffMatrix::new(["A", "B"], ["X", "Y"], [[7.0, 3.0], [4.0, 8.0]]);
        assert!(m.dominant_row().is_none());
    }

    #[test]
    fn test_dominant_strategy() {
        let m = PayoffMatrix::new(["A", "B"], ["X", "Y"], [[7.0, 6.0], [4.0, 3.0]]);
        assert_eq!(m.dominant_row(), Some(0));
    }

    #[test]
    fn test_nash_solve_mixed() {
        let m = PayoffMatrix::new(
            ["Monolith", "Modular"],
            ["Speed", "Quality"],
            [[7.0, 3.0], [4.0, 8.0]],
        );
        let nash = NashSolver::solve(&m);
        assert!((nash.row_strategy[0] - 0.5).abs() < 0.01);
        assert!(!nash.pure_equilibrium);
    }

    #[test]
    fn test_nash_expected_payoff() {
        let m = PayoffMatrix::new(["A", "B"], ["X", "Y"], [[7.0, 3.0], [4.0, 8.0]]);
        let nash = NashSolver::solve(&m);
        assert!(nash.expected_payoff > 4.0);
        assert!(nash.expected_payoff < 8.0);
    }
}
