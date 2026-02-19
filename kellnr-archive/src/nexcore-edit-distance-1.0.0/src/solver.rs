//! # Solvers (T2-C Layer)
//!
//! The "how" of edit distance computation, separated from the "what" (metric definition).
//! Solvers are the contingent bond — swapping one changes performance, not correctness.
//!
//! Built-in solvers:
//! - `FullMatrixDp` — O(mn) space, supports traceback/alignment
//! - `TwoRowDp` — O(min(m,n)) space, distance only (no traceback)
//! - `BandedDp` — O(kn) for bounded distance k, early termination

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::cost::CostModel;
use crate::ops::{EditOp, OperationSet};

/// Result of solving an edit distance problem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolveResult<E: Clone + Eq> {
    /// The computed edit distance
    pub distance: f64,
    /// Optional sequence of operations (traceback), if solver supports it
    #[serde(skip)]
    pub operations: Option<Vec<EditOp<E>>>,
    /// Number of DP cells computed (for benchmarking)
    pub cells_computed: usize,
}

/// Computes edit distance between two sequences.
///
/// Generic over element type `E` and cost model `C`.
/// The operation set is passed as a generic to `solve` for monomorphization.
pub trait Solver<E: Clone + Eq, C: CostModel<E>>: Clone + Send + Sync + fmt::Debug {
    /// Compute edit distance between `source` and `target`.
    fn solve<O: OperationSet>(
        &self,
        source: &[E],
        target: &[E],
        ops: &O,
        cost: &C,
    ) -> SolveResult<E>;

    /// Whether this solver supports traceback (operation sequence reconstruction).
    fn supports_traceback(&self) -> bool;

    /// Human-readable name
    fn name(&self) -> &str;
}

// ---------------------------------------------------------------------------
// FullMatrixDp: O(mn) space, supports traceback
// ---------------------------------------------------------------------------

/// Classic Wagner-Fischer with full matrix. O(mn) time and space.
/// Supports traceback to reconstruct the edit operation sequence.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct FullMatrixDp;

impl<E: Clone + Eq, C: CostModel<E>> Solver<E, C> for FullMatrixDp {
    fn solve<O: OperationSet>(
        &self,
        source: &[E],
        target: &[E],
        ops: &O,
        cost: &C,
    ) -> SolveResult<E> {
        let m = source.len();
        let n = target.len();

        // Build full (m+1) x (n+1) matrix
        let mut matrix = vec![vec![0.0_f64; n + 1]; m + 1];

        for i in 1..=m {
            matrix[i][0] = matrix[i - 1][0] + cost.delete_cost(&source[i - 1]);
        }
        for j in 1..=n {
            matrix[0][j] = matrix[0][j - 1] + cost.insert_cost(&target[j - 1]);
        }

        let mut cells = m + n;

        for i in 1..=m {
            for j in 1..=n {
                cells += 1;

                let del = if ops.allows_delete() {
                    matrix[i - 1][j] + cost.delete_cost(&source[i - 1])
                } else {
                    f64::INFINITY
                };

                let ins = if ops.allows_insert() {
                    matrix[i][j - 1] + cost.insert_cost(&target[j - 1])
                } else {
                    f64::INFINITY
                };

                let sub = if ops.allows_substitute() || source[i - 1] == target[j - 1] {
                    matrix[i - 1][j - 1] + cost.substitute_cost(&source[i - 1], &target[j - 1])
                } else {
                    f64::INFINITY
                };

                let mut best = del.min(ins).min(sub);

                // Damerau transposition
                if ops.allows_transpose()
                    && i > 1
                    && j > 1
                    && source[i - 1] == target[j - 2]
                    && source[i - 2] == target[j - 1]
                {
                    let trans =
                        matrix[i - 2][j - 2] + cost.transpose_cost(&source[i - 2], &source[i - 1]);
                    best = best.min(trans);
                }

                matrix[i][j] = best;
            }
        }

        let operations = traceback(&matrix, source, target, ops, cost);

        SolveResult {
            distance: matrix[m][n],
            operations: Some(operations),
            cells_computed: cells,
        }
    }

    fn supports_traceback(&self) -> bool {
        true
    }

    fn name(&self) -> &str {
        "full-matrix-dp"
    }
}

/// Reconstruct edit operations by walking the DP matrix backward.
fn traceback<E: Clone + Eq, O: OperationSet, C: CostModel<E>>(
    matrix: &[Vec<f64>],
    source: &[E],
    target: &[E],
    ops: &O,
    cost: &C,
) -> Vec<EditOp<E>> {
    let mut result = Vec::new();
    let mut i = source.len();
    let mut j = target.len();

    while i > 0 || j > 0 {
        if i > 0
            && j > 0
            && (ops.allows_substitute() || source[i - 1] == target[j - 1])
            && (matrix[i][j]
                - (matrix[i - 1][j - 1] + cost.substitute_cost(&source[i - 1], &target[j - 1])))
            .abs()
                < f64::EPSILON
        {
            if source[i - 1] != target[j - 1] {
                result.push(EditOp::Substitute {
                    pos: i - 1,
                    from: source[i - 1].clone(),
                    to: target[j - 1].clone(),
                });
            }
            i -= 1;
            j -= 1;
        } else if i > 0
            && ops.allows_delete()
            && (matrix[i][j] - (matrix[i - 1][j] + cost.delete_cost(&source[i - 1]))).abs()
                < f64::EPSILON
        {
            result.push(EditOp::Delete {
                pos: i - 1,
                elem: source[i - 1].clone(),
            });
            i -= 1;
        } else if j > 0
            && ops.allows_insert()
            && (matrix[i][j] - (matrix[i][j - 1] + cost.insert_cost(&target[j - 1]))).abs()
                < f64::EPSILON
        {
            result.push(EditOp::Insert {
                pos: j - 1,
                elem: target[j - 1].clone(),
            });
            j -= 1;
        } else {
            // Match or fallback
            if i > 0 && j > 0 {
                i -= 1;
                j -= 1;
            } else if i > 0 {
                i -= 1;
            } else {
                j -= 1;
            }
        }
    }

    result.reverse();
    result
}

// ---------------------------------------------------------------------------
// TwoRowDp: O(min(m,n)) space, no traceback
// ---------------------------------------------------------------------------

/// Two-row Wagner-Fischer. O(mn) time, O(min(m,n)) space.
/// Does **not** support traceback — use `FullMatrixDp` if you need operations.
///
/// This is the solver used by the existing `foundation_levenshtein` MCP tool.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct TwoRowDp;

impl<E: Clone + Eq, C: CostModel<E>> Solver<E, C> for TwoRowDp {
    fn solve<O: OperationSet>(
        &self,
        source: &[E],
        target: &[E],
        ops: &O,
        cost: &C,
    ) -> SolveResult<E> {
        let m = source.len();
        let n = target.len();

        if m == 0 {
            let total: f64 = target.iter().map(|e| cost.insert_cost(e)).sum();
            return SolveResult {
                distance: total,
                operations: None,
                cells_computed: n,
            };
        }
        if n == 0 {
            let total: f64 = source.iter().map(|e| cost.delete_cost(e)).sum();
            return SolveResult {
                distance: total,
                operations: None,
                cells_computed: m,
            };
        }

        // Swap to iterate over longer dimension (shorter in memory)
        let (short, long, short_len, long_len, swapped) = if m <= n {
            (source, target, m, n, false)
        } else {
            (target, source, n, m, true)
        };

        // Initialize first row
        let mut prev_row: Vec<f64> = Vec::with_capacity(short_len + 1);
        prev_row.push(0.0);
        for j in 1..=short_len {
            let gap = if swapped {
                cost.delete_cost(&short[j - 1])
            } else {
                cost.insert_cost(&short[j - 1])
            };
            prev_row.push(prev_row[j - 1] + gap);
        }
        let mut curr_row: Vec<f64> = vec![0.0; short_len + 1];
        let mut cells = short_len;

        for i in 1..=long_len {
            let row_gap = if swapped {
                cost.insert_cost(&long[i - 1])
            } else {
                cost.delete_cost(&long[i - 1])
            };
            curr_row[0] = prev_row[0] + row_gap;

            for j in 1..=short_len {
                cells += 1;

                let (src_elem, tgt_elem) = if swapped {
                    (&short[j - 1], &long[i - 1])
                } else {
                    (&long[i - 1], &short[j - 1])
                };

                let del = if ops.allows_delete() {
                    prev_row[j] + cost.delete_cost(src_elem)
                } else {
                    f64::INFINITY
                };

                let ins = if ops.allows_insert() {
                    curr_row[j - 1] + cost.insert_cost(tgt_elem)
                } else {
                    f64::INFINITY
                };

                let sub = if ops.allows_substitute() || src_elem == tgt_elem {
                    prev_row[j - 1] + cost.substitute_cost(src_elem, tgt_elem)
                } else {
                    f64::INFINITY
                };

                curr_row[j] = del.min(ins).min(sub);
            }

            std::mem::swap(&mut prev_row, &mut curr_row);
        }

        SolveResult {
            distance: prev_row[short_len],
            operations: None,
            cells_computed: cells,
        }
    }

    fn supports_traceback(&self) -> bool {
        false
    }

    fn name(&self) -> &str {
        "two-row-dp"
    }
}

// ---------------------------------------------------------------------------
// BandedDp: O(kn) for bounded distance k
// ---------------------------------------------------------------------------

/// Banded Wagner-Fischer for bounded edit distance.
///
/// Only computes cells within `band_width` of the diagonal.
/// Returns `f64::INFINITY` if true distance exceeds the band.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BandedDp {
    /// Maximum distance to consider (band half-width)
    pub max_distance: usize,
}

impl BandedDp {
    /// Create a banded solver with given maximum distance.
    #[must_use]
    pub fn new(max_distance: usize) -> Self {
        Self { max_distance }
    }
}

impl<E: Clone + Eq, C: CostModel<E>> Solver<E, C> for BandedDp {
    fn solve<O: OperationSet>(
        &self,
        source: &[E],
        target: &[E],
        ops: &O,
        cost: &C,
    ) -> SolveResult<E> {
        let m = source.len();
        let n = target.len();
        let k = self.max_distance;

        // Length difference alone exceeds band
        if m.abs_diff(n) > k {
            return SolveResult {
                distance: f64::INFINITY,
                operations: None,
                cells_computed: 0,
            };
        }

        if m == 0 {
            let total: f64 = target.iter().map(|e| cost.insert_cost(e)).sum();
            return if total <= k as f64 {
                SolveResult {
                    distance: total,
                    operations: None,
                    cells_computed: n,
                }
            } else {
                SolveResult {
                    distance: f64::INFINITY,
                    operations: None,
                    cells_computed: n,
                }
            };
        }

        if n == 0 {
            let total: f64 = source.iter().map(|e| cost.delete_cost(e)).sum();
            return if total <= k as f64 {
                SolveResult {
                    distance: total,
                    operations: None,
                    cells_computed: m,
                }
            } else {
                SolveResult {
                    distance: f64::INFINITY,
                    operations: None,
                    cells_computed: m,
                }
            };
        }

        let band = k;
        let mut prev_row: Vec<f64> = vec![f64::INFINITY; n + 1];
        let mut curr_row: Vec<f64> = vec![f64::INFINITY; n + 1];

        prev_row[0] = 0.0;
        for j in 1..=band.min(n) {
            prev_row[j] = prev_row[j - 1] + cost.insert_cost(&target[j - 1]);
        }

        let mut cells = band.min(n);

        for i in 1..=m {
            curr_row.fill(f64::INFINITY);

            let j_start = if i > band { i - band } else { 1 };
            let j_end = (i + band).min(n);

            if i <= band {
                let mut acc = 0.0;
                for idx in 0..i {
                    acc += cost.delete_cost(&source[idx]);
                }
                curr_row[0] = acc;
            }

            for j in j_start..=j_end {
                cells += 1;

                let del = if ops.allows_delete() {
                    prev_row[j] + cost.delete_cost(&source[i - 1])
                } else {
                    f64::INFINITY
                };

                let ins = if ops.allows_insert() && j > 0 {
                    curr_row[j - 1] + cost.insert_cost(&target[j - 1])
                } else {
                    f64::INFINITY
                };

                let sub = if (ops.allows_substitute() || source[i - 1] == target[j - 1])
                    && prev_row[j - 1] < f64::INFINITY
                {
                    prev_row[j - 1] + cost.substitute_cost(&source[i - 1], &target[j - 1])
                } else {
                    f64::INFINITY
                };

                curr_row[j] = del.min(ins).min(sub);
            }

            // Early termination
            let min_in_band = curr_row[j_start..=j_end]
                .iter()
                .copied()
                .fold(f64::INFINITY, f64::min);
            if min_in_band > k as f64 {
                return SolveResult {
                    distance: f64::INFINITY,
                    operations: None,
                    cells_computed: cells,
                };
            }

            std::mem::swap(&mut prev_row, &mut curr_row);
        }

        let distance = prev_row[n];
        SolveResult {
            distance: if distance > k as f64 {
                f64::INFINITY
            } else {
                distance
            },
            operations: None,
            cells_computed: cells,
        }
    }

    fn supports_traceback(&self) -> bool {
        false
    }

    fn name(&self) -> &str {
        "banded-dp"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cost::UniformCost;
    use crate::ops::StdOps;

    fn distance_with<S: Solver<char, UniformCost>>(solver: &S, source: &str, target: &str) -> f64 {
        let src: Vec<char> = source.chars().collect();
        let tgt: Vec<char> = target.chars().collect();
        solver.solve(&src, &tgt, &StdOps, &UniformCost).distance
    }

    #[test]
    fn full_matrix_kitten_sitting() {
        let d = distance_with(&FullMatrixDp, "kitten", "sitting");
        assert!((d - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn full_matrix_empty_strings() {
        assert!((distance_with(&FullMatrixDp, "", "") - 0.0).abs() < f64::EPSILON);
        assert!((distance_with(&FullMatrixDp, "abc", "") - 3.0).abs() < f64::EPSILON);
        assert!((distance_with(&FullMatrixDp, "", "xyz") - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn full_matrix_traceback() {
        let src: Vec<char> = "kitten".chars().collect();
        let tgt: Vec<char> = "sitting".chars().collect();
        let result = FullMatrixDp.solve(&src, &tgt, &StdOps, &UniformCost);
        let ops = result.operations.expect("full matrix supports traceback");
        assert!(!ops.is_empty());
        assert!(ops.len() <= 4);
    }

    #[test]
    fn full_matrix_symmetry() {
        let d1 = distance_with(&FullMatrixDp, "abc", "def");
        let d2 = distance_with(&FullMatrixDp, "def", "abc");
        assert!((d1 - d2).abs() < f64::EPSILON);
    }

    #[test]
    fn two_row_kitten_sitting() {
        let d = distance_with(&TwoRowDp, "kitten", "sitting");
        assert!((d - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn two_row_empty_strings() {
        assert!((distance_with(&TwoRowDp, "", "") - 0.0).abs() < f64::EPSILON);
        assert!((distance_with(&TwoRowDp, "abc", "") - 3.0).abs() < f64::EPSILON);
        assert!((distance_with(&TwoRowDp, "", "xyz") - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn two_row_no_traceback() {
        let src: Vec<char> = "abc".chars().collect();
        let tgt: Vec<char> = "xyz".chars().collect();
        let result = TwoRowDp.solve(&src, &tgt, &StdOps, &UniformCost);
        assert!(result.operations.is_none());
        assert!(!<TwoRowDp as Solver<char, UniformCost>>::supports_traceback(&TwoRowDp));
    }

    #[test]
    fn two_row_agrees_with_full_matrix() {
        let cases = [
            ("kitten", "sitting"),
            ("", "abc"),
            ("hello", "hello"),
            ("saturday", "sunday"),
            ("flaw", "lawn"),
        ];
        for (a, b) in cases {
            let d1 = distance_with(&FullMatrixDp, a, b);
            let d2 = distance_with(&TwoRowDp, a, b);
            assert!(
                (d1 - d2).abs() < f64::EPSILON,
                "solver disagreement for ({a:?}, {b:?}): full={d1}, two_row={d2}"
            );
        }
    }

    #[test]
    fn banded_within_band() {
        let d = distance_with(&BandedDp::new(5), "kitten", "sitting");
        assert!((d - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn banded_exceeds_band() {
        let d = distance_with(&BandedDp::new(2), "kitten", "sitting");
        assert!(d.is_infinite());
    }

    #[test]
    fn banded_agrees_with_full_when_wide() {
        let cases = [
            ("kitten", "sitting"),
            ("abc", "def"),
            ("hello", "hallo"),
            ("saturday", "sunday"),
        ];
        for (a, b) in cases {
            let d1 = distance_with(&FullMatrixDp, a, b);
            let d2 = distance_with(&BandedDp::new(100), a, b);
            assert!(
                (d1 - d2).abs() < f64::EPSILON,
                "banded(100) should agree with full for ({a:?}, {b:?})"
            );
        }
    }

    #[test]
    fn banded_empty_strings() {
        assert!((distance_with(&BandedDp::new(5), "", "") - 0.0).abs() < f64::EPSILON);
        assert!((distance_with(&BandedDp::new(5), "abc", "") - 3.0).abs() < f64::EPSILON);
    }
}
