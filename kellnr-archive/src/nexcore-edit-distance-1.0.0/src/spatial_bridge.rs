//! # Spatial Bridge: Edit Distance → stem-math
//!
//! Implements `stem_math::spatial::Metric` for the edit distance framework,
//! making implicit distance computations explicit via formal spatial types.
//!
//! ## Primitive Foundation
//!
//! Edit distance is fundamentally a Metric (N Quantity + kappa Comparison + mu Mapping):
//! - Non-negative distance between string elements
//! - Identity: d("abc", "abc") = 0
//! - Symmetry: d(a,b) = d(b,a)
//! - Triangle inequality: d(a,c) <= d(a,b) + d(b,c)
//!
//! ## Architecture Decision
//!
//! Implements `Metric` for `EditMetric<char, O, C, S>` (string-specialized metrics).
//! Also provides convenience wrapper `StringEditMetric` for direct &str usage.

use stem_math::spatial::{Distance, Metric, Neighborhood};

use crate::cost::{CostModel, UniformCost};
use crate::metric::{DamerauLev, EditMetric, Lcs, Levenshtein};
use crate::ops::{DamerauOps, IndelOps, OperationSet, StdOps};
use crate::solver::{Solver, TwoRowDp};

// ============================================================================
// StringEditMetric: Metric impl for string-level operations
// ============================================================================

/// A spatial metric adapter wrapping any `EditMetric<char, O, C, S>`.
///
/// Converts the internal `EditMetric::distance()` → `stem_math::spatial::Distance`.
///
/// Tier: T2-C (N Quantity + kappa Comparison + mu Mapping)
pub struct StringEditMetric<O, C, S>
where
    O: OperationSet,
    C: CostModel<char>,
    S: Solver<char, C>,
{
    inner: EditMetric<char, O, C, S>,
}

impl<O, C, S> StringEditMetric<O, C, S>
where
    O: OperationSet,
    C: CostModel<char>,
    S: Solver<char, C>,
{
    /// Create a spatial metric from an existing EditMetric.
    pub fn new(inner: EditMetric<char, O, C, S>) -> Self {
        Self { inner }
    }

    /// Compute distance between two strings directly.
    pub fn str_distance(&self, source: &str, target: &str) -> Distance {
        Distance::new(self.inner.str_distance(source, target))
    }

    /// Check if two strings are within a neighborhood.
    pub fn str_within(&self, source: &str, target: &str, neighborhood: &Neighborhood) -> bool {
        neighborhood.contains(self.str_distance(source, target))
    }
}

/// Convenience: Levenshtein as a spatial Metric over char slices.
pub type LevenshteinSpatialMetric = StringEditMetric<StdOps, UniformCost, TwoRowDp>;

/// Convenience: Damerau-Levenshtein as a spatial Metric.
pub type DamerauSpatialMetric =
    StringEditMetric<DamerauOps, UniformCost, crate::solver::FullMatrixDp>;

/// Convenience: LCS distance as a spatial Metric.
pub type LcsSpatialMetric = StringEditMetric<IndelOps, UniformCost, TwoRowDp>;

impl LevenshteinSpatialMetric {
    /// Create a default Levenshtein spatial metric.
    pub fn levenshtein() -> Self {
        Self::new(Levenshtein::default())
    }
}

impl DamerauSpatialMetric {
    /// Create a default Damerau-Levenshtein spatial metric.
    pub fn damerau() -> Self {
        Self::new(DamerauLev::default())
    }
}

impl LcsSpatialMetric {
    /// Create a default LCS spatial metric.
    pub fn lcs() -> Self {
        Self::new(Lcs::default())
    }
}

// ============================================================================
// Metric trait impl for char slices
// ============================================================================

impl<O, C, S> Metric for StringEditMetric<O, C, S>
where
    O: OperationSet,
    C: CostModel<char>,
    S: Solver<char, C>,
{
    type Element = Vec<char>;

    fn distance(&self, a: &Vec<char>, b: &Vec<char>) -> Distance {
        Distance::new(self.inner.distance(a, b))
    }
}

// ============================================================================
// Neighborhood constructors for common edit-distance thresholds
// ============================================================================

/// Create a closed neighborhood for fuzzy matching within N edits.
///
/// A string is "similar" if its edit distance is within `max_edits`.
pub fn fuzzy_neighborhood(max_edits: usize) -> Neighborhood {
    Neighborhood::closed(Distance::new(max_edits as f64))
}

/// Create an adaptive fuzzy neighborhood based on string length and minimum similarity.
///
/// This mirrors the adaptive bound logic used in `fuzzy_search`:
/// max_distance = len * (1.0 - min_similarity)
pub fn adaptive_fuzzy_neighborhood(query_len: usize, min_similarity: f64) -> Neighborhood {
    let max_dist = query_len as f64 * (1.0 - min_similarity.clamp(0.0, 1.0));
    Neighborhood::closed(Distance::new(max_dist))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use stem_math::spatial::Distance;

    // ===== Metric Axiom Tests =====

    #[test]
    fn axiom_non_negativity() {
        let m = LevenshteinSpatialMetric::levenshtein();
        let cases: Vec<(&str, &str)> = vec![
            ("kitten", "sitting"),
            ("abc", ""),
            ("", ""),
            ("hello", "hello"),
        ];
        for (a, b) in cases {
            let ac: Vec<char> = a.chars().collect();
            let bc: Vec<char> = b.chars().collect();
            assert!(
                m.distance(&ac, &bc).value() >= 0.0,
                "non-negativity failed for ({a:?}, {b:?})"
            );
        }
    }

    #[test]
    fn axiom_identity_of_indiscernibles() {
        let m = LevenshteinSpatialMetric::levenshtein();
        let cases = ["", "hello", "world", "abc"];
        for s in cases {
            let sc: Vec<char> = s.chars().collect();
            assert!(
                m.distance(&sc, &sc)
                    .approx_eq(&Distance::ZERO, f64::EPSILON),
                "identity failed for {s:?}"
            );
        }
    }

    #[test]
    fn axiom_symmetry() {
        let m = LevenshteinSpatialMetric::levenshtein();
        let cases = [("kitten", "sitting"), ("abc", "def"), ("", "hello")];
        for (a, b) in cases {
            let ac: Vec<char> = a.chars().collect();
            let bc: Vec<char> = b.chars().collect();
            assert!(
                m.is_symmetric(&ac, &bc, f64::EPSILON),
                "symmetry failed for ({a:?}, {b:?})"
            );
        }
    }

    #[test]
    fn axiom_triangle_inequality() {
        let m = LevenshteinSpatialMetric::levenshtein();
        let triples = [
            ("abc", "axc", "axy"),
            ("kitten", "sitting", "bitten"),
            ("", "a", "ab"),
        ];
        for (a, b, c) in triples {
            let ac: Vec<char> = a.chars().collect();
            let bc: Vec<char> = b.chars().collect();
            let cc: Vec<char> = c.chars().collect();
            let d_ab = m.distance(&ac, &bc);
            let d_bc = m.distance(&bc, &cc);
            let d_ac = m.distance(&ac, &cc);
            assert!(
                Distance::triangle_valid(d_ab, d_bc, d_ac),
                "triangle inequality failed for ({a:?}, {b:?}, {c:?}): d(a,c)={} > d(a,b)+d(b,c)={}",
                d_ac.value(),
                d_ab.value() + d_bc.value()
            );
        }
    }

    // ===== Damerau metric axioms =====

    #[test]
    fn damerau_symmetry() {
        let m = DamerauSpatialMetric::damerau();
        let ac: Vec<char> = "ab".chars().collect();
        let bc: Vec<char> = "ba".chars().collect();
        assert!(m.is_symmetric(&ac, &bc, f64::EPSILON));
    }

    #[test]
    fn damerau_transposition_distance() {
        let m = DamerauSpatialMetric::damerau();
        let ac: Vec<char> = "ab".chars().collect();
        let bc: Vec<char> = "ba".chars().collect();
        let d = m.distance(&ac, &bc);
        assert!((d.value() - 1.0).abs() < f64::EPSILON);
    }

    // ===== LCS metric axioms =====

    #[test]
    fn lcs_identity() {
        let m = LcsSpatialMetric::lcs();
        let sc: Vec<char> = "abc".chars().collect();
        assert!(
            m.distance(&sc, &sc)
                .approx_eq(&Distance::ZERO, f64::EPSILON)
        );
    }

    // ===== Neighborhood tests =====

    #[test]
    fn fuzzy_neighborhood_containment() {
        let m = LevenshteinSpatialMetric::levenshtein();
        let n = fuzzy_neighborhood(3);
        let kitten: Vec<char> = "kitten".chars().collect();
        let sitting: Vec<char> = "sitting".chars().collect();
        let hello: Vec<char> = "hello".chars().collect();

        assert!(m.within(&kitten, &sitting, &n)); // d=3, within 3
        assert!(!m.within(&kitten, &hello, &n)); // d=5, outside 3
    }

    #[test]
    fn adaptive_neighborhood_scales_with_length() {
        let short = adaptive_fuzzy_neighborhood(5, 0.8);
        let long = adaptive_fuzzy_neighborhood(20, 0.8);

        // Longer strings get wider neighborhoods
        assert!(short.radius.value() < long.radius.value());
        // 5 * (1.0 - 0.8) ≈ 1.0, 20 * (1.0 - 0.8) ≈ 4.0
        // Use tolerance for IEEE 754 subtraction: 1.0 - 0.8 = 0.19999999999999996
        assert!((short.radius.value() - 1.0).abs() < 1e-10);
        assert!((long.radius.value() - 4.0).abs() < 1e-10);
    }

    // ===== String convenience tests =====

    #[test]
    fn str_distance_convenience() {
        let m = LevenshteinSpatialMetric::levenshtein();
        let d = m.str_distance("kitten", "sitting");
        assert!((d.value() - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn str_within_convenience() {
        let m = LevenshteinSpatialMetric::levenshtein();
        let n = fuzzy_neighborhood(3);
        assert!(m.str_within("kitten", "sitting", &n)); // d=3, within closed [0,3]
        assert!(!m.str_within("abc", "uvwxyz", &n)); // d=6, outside closed [0,3]
    }
}
