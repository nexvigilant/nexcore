//! # NexVigilant Core — edit-distance
//!
//! Generic edit distance framework with pluggable operations, cost models, and solvers.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]
//!
//! ## Configuration Space
//!
//! ```text
//! edit_distance(ops, costs, solver) → {
//!   ops={ins,del,sub}, costs=1, solver=DP        → Levenshtein
//!   ops={ins,del,sub,trans}, costs=1, solver=DP   → Damerau-Levenshtein
//!   ops={ins,del,sub}, costs=matrix, solver=DP    → Needleman-Wunsch
//!   ops={ins,del}, costs=1, solver=DP             → LCS distance
//! }
//! ```
//!
//! ## Quick Start
//!
//! ```rust
//! use nexcore_edit_distance::prelude::*;
//!
//! // Classic Levenshtein
//! let d = levenshtein("kitten", "sitting");
//! assert!((d - 3.0).abs() < f64::EPSILON);
//!
//! // Indel-only (LCS distance)
//! let d = lcs_distance("kitten", "sitting");
//! assert!(d > 3.0); // No substitution allowed, so more operations needed
//! ```

pub mod adapter;
pub mod classic;
pub mod cost;
pub mod grounding;
pub mod metric;
pub mod ops;
pub mod solver;
pub mod spatial_bridge;
pub mod transfer;

use cost::{CostModel, UniformCost};
use ops::{DamerauOps, IndelOps, OperationSet, StdOps};
use solver::{FullMatrixDp, SolveResult, Solver, TwoRowDp};

/// Convenience prelude for common imports.
pub mod prelude {
    pub use crate::cost::{CostModel, MatrixCost, UniformCost, WeightedCost};
    pub use crate::metric::{
        DamerauLev, EditMetric, EditMetricBuilder, Lcs, Levenshtein, LevenshteinTraceback,
    };
    pub use crate::ops::{DamerauOps, EditOp, IndelOps, OperationSet, StdOps};
    pub use crate::solver::{BandedDp, FullMatrixDp, SolveResult, Solver, TwoRowDp};
    pub use crate::{compute, damerau_levenshtein, lcs_distance, levenshtein};
}

/// Compute classic Levenshtein distance (unit cost, ins/del/sub, two-row DP).
#[must_use]
pub fn levenshtein(source: &str, target: &str) -> f64 {
    let src: Vec<char> = source.chars().collect();
    let tgt: Vec<char> = target.chars().collect();
    TwoRowDp.solve(&src, &tgt, &StdOps, &UniformCost).distance
}

/// Compute Damerau-Levenshtein distance (adds transposition).
#[must_use]
pub fn damerau_levenshtein(source: &str, target: &str) -> f64 {
    let src: Vec<char> = source.chars().collect();
    let tgt: Vec<char> = target.chars().collect();
    FullMatrixDp
        .solve(&src, &tgt, &DamerauOps, &UniformCost)
        .distance
}

/// Compute LCS (indel-only) distance.
#[must_use]
pub fn lcs_distance(source: &str, target: &str) -> f64 {
    let src: Vec<char> = source.chars().collect();
    let tgt: Vec<char> = target.chars().collect();
    TwoRowDp.solve(&src, &tgt, &IndelOps, &UniformCost).distance
}

/// Compute edit distance with full control over ops, cost, and solver.
///
/// This is the generic entry point — the reified configuration space.
pub fn compute<O, C, S>(
    source: &str,
    target: &str,
    ops: &O,
    cost: &C,
    solver: &S,
) -> SolveResult<char>
where
    O: OperationSet,
    C: CostModel<char>,
    S: Solver<char, C>,
{
    let src: Vec<char> = source.chars().collect();
    let tgt: Vec<char> = target.chars().collect();
    solver.solve(&src, &tgt, ops, cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classic_levenshtein() {
        assert!((levenshtein("kitten", "sitting") - 3.0).abs() < f64::EPSILON);
        assert!((levenshtein("", "") - 0.0).abs() < f64::EPSILON);
        assert!((levenshtein("abc", "") - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn damerau_transposition() {
        // "ab" -> "ba" is 1 transposition in Damerau, 2 ops in standard
        assert!((damerau_levenshtein("ab", "ba") - 1.0).abs() < f64::EPSILON);
        assert!((levenshtein("ab", "ba") - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn lcs_no_substitution() {
        let d = lcs_distance("ab", "ba");
        assert!((d - 2.0).abs() < f64::EPSILON);
        assert!((lcs_distance("abc", "abc") - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn lcs_ge_levenshtein() {
        let cases = [
            ("kitten", "sitting"),
            ("abc", "def"),
            ("hello", "hallo"),
            ("flaw", "lawn"),
        ];
        for (a, b) in cases {
            let lev = levenshtein(a, b);
            let lcs = lcs_distance(a, b);
            assert!(
                lcs >= lev - f64::EPSILON,
                "LCS should >= Levenshtein for ({a:?}, {b:?}): lcs={lcs}, lev={lev}"
            );
        }
    }

    #[test]
    fn custom_solver_via_generic() {
        use solver::BandedDp;
        let result = compute(
            "kitten",
            "sitting",
            &StdOps,
            &UniformCost,
            &BandedDp::new(5),
        );
        assert!((result.distance - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn unicode_support() {
        assert!((levenshtein("こんにちは", "こんばんは") - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn symmetry_property() {
        let cases = [("abc", "def"), ("kitten", "sitting"), ("foo", "bar")];
        for (a, b) in cases {
            let d1 = levenshtein(a, b);
            let d2 = levenshtein(b, a);
            assert!(
                (d1 - d2).abs() < f64::EPSILON,
                "symmetry broken for ({a:?}, {b:?})"
            );
        }
    }
}
