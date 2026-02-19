//! # Spatial Bridge: nexcore-vigilance algorithms → stem-math
//!
//! Wraps the vigilance foundation algorithms (levenshtein, fuzzy_search)
//! as formal `Metric` implementations and expresses fuzzy thresholds as
//! `Neighborhood` containment checks.
//!
//! ## Primitive Foundation
//!
//! String similarity is fundamentally a Metric space:
//! - Levenshtein distance satisfies all 4 metric axioms
//! - Fuzzy search threshold = Neighborhood radius
//! - Adaptive tightening = dynamic Neighborhood shrinking
//!
//! ## Architecture Decision
//!
//! Uses the crate-local `levenshtein_distance()` function (63× faster than Python)
//! rather than depending on `nexcore-edit-distance` to avoid circular deps.

use stem_math::spatial::{Distance, Metric, Neighborhood};

use super::levenshtein::{levenshtein, levenshtein_distance};

// ============================================================================
// StringMetric: Levenshtein as formal Metric
// ============================================================================

/// Spatial metric over strings using the vigilance Levenshtein implementation.
///
/// This wraps `levenshtein_distance()` (63× faster than Python) into a
/// formal `Metric` implementation for use with the spatial type system.
///
/// Tier: T2-C (N Quantity + κ Comparison + μ Mapping)
pub struct StringMetric;

impl Metric for StringMetric {
    type Element = String;

    fn distance(&self, a: &String, b: &String) -> Distance {
        Distance::new(levenshtein_distance(a, b) as f64)
    }
}

impl StringMetric {
    /// Compute distance between string slices (convenience).
    pub fn str_distance(&self, a: &str, b: &str) -> Distance {
        Distance::new(levenshtein_distance(a, b) as f64)
    }

    /// Compute normalized similarity (0.0 to 1.0).
    pub fn str_similarity(&self, a: &str, b: &str) -> f64 {
        levenshtein(a, b).similarity
    }

    /// Check if two strings are within a neighborhood.
    pub fn str_within(&self, a: &str, b: &str, neighborhood: &Neighborhood) -> bool {
        neighborhood.contains(self.str_distance(a, b))
    }
}

// ============================================================================
// Neighborhood constructors for fuzzy matching
// ============================================================================

/// Create a closed neighborhood for exact edit distance matching.
///
/// A string is "similar" if its edit distance is within `max_edits`.
pub fn fuzzy_neighborhood(max_edits: usize) -> Neighborhood {
    Neighborhood::closed(Distance::new(max_edits as f64))
}

/// Create an adaptive fuzzy neighborhood based on string length and similarity.
///
/// This mirrors the adaptive bound logic used in `fuzzy_search`:
/// `max_distance = ceil(max_len * (1.0 - min_similarity))`
///
/// Longer strings get wider neighborhoods (absolute distance allowance),
/// maintaining the same relative similarity requirement.
pub fn adaptive_fuzzy_neighborhood(query_len: usize, min_similarity: f64) -> Neighborhood {
    let max_dist = (query_len as f64 * (1.0 - min_similarity.clamp(0.0, 1.0))).ceil();
    Neighborhood::closed(Distance::new(max_dist))
}

/// Similarity-based neighborhood: distance <= (1.0 - min_similarity) * max_len.
///
/// Used for normalized similarity thresholds where we want to check
/// if similarity(a, b) >= min_similarity.
pub fn similarity_neighborhood(max_len: usize, min_similarity: f64) -> Neighborhood {
    let max_dist = max_len as f64 * (1.0 - min_similarity.clamp(0.0, 1.0));
    Neighborhood::closed(Distance::new(max_dist))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Metric axiom tests =====

    #[test]
    fn axiom_non_negativity() {
        let m = StringMetric;
        let pairs = [
            ("kitten".to_string(), "sitting".to_string()),
            ("abc".to_string(), String::new()),
            (String::new(), String::new()),
        ];
        for (a, b) in &pairs {
            assert!(m.distance(a, b).value() >= 0.0);
        }
    }

    #[test]
    fn axiom_identity_of_indiscernibles() {
        let m = StringMetric;
        let cases = [String::new(), "hello".to_string(), "world".to_string()];
        for s in &cases {
            assert!(
                m.distance(s, s).approx_eq(&Distance::ZERO, f64::EPSILON),
                "identity failed for {:?}",
                s
            );
        }
    }

    #[test]
    fn axiom_symmetry() {
        let m = StringMetric;
        let pairs = [
            ("kitten".to_string(), "sitting".to_string()),
            ("abc".to_string(), "def".to_string()),
            (String::new(), "hello".to_string()),
        ];
        for (a, b) in &pairs {
            assert!(
                m.is_symmetric(a, b, f64::EPSILON),
                "symmetry failed for ({:?}, {:?})",
                a,
                b
            );
        }
    }

    #[test]
    fn axiom_triangle_inequality() {
        let m = StringMetric;
        let triples = [
            ("abc", "axc", "axy"),
            ("kitten", "sitting", "bitten"),
            ("", "a", "ab"),
        ];
        for (a, b, c) in triples {
            let a = a.to_string();
            let b = b.to_string();
            let c = c.to_string();
            let d_ab = m.distance(&a, &b);
            let d_bc = m.distance(&b, &c);
            let d_ac = m.distance(&a, &c);
            assert!(
                Distance::triangle_valid(d_ab, d_bc, d_ac),
                "triangle failed for ({:?}, {:?}, {:?})",
                a,
                b,
                c
            );
        }
    }

    // ===== Convenience method tests =====

    #[test]
    fn str_distance_matches_metric() {
        let m = StringMetric;
        let d1 = m.str_distance("kitten", "sitting");
        let a = "kitten".to_string();
        let b = "sitting".to_string();
        let d2 = m.distance(&a, &b);
        assert!(d1.approx_eq(&d2, f64::EPSILON));
    }

    #[test]
    fn str_similarity_normalized() {
        let m = StringMetric;
        let sim = m.str_similarity("kitten", "sitting");
        assert!(sim >= 0.0 && sim <= 1.0);
        assert!(m.str_similarity("abc", "abc") > 0.99);
    }

    #[test]
    fn str_within_check() {
        let m = StringMetric;
        let n = fuzzy_neighborhood(3);
        assert!(m.str_within("kitten", "sitting", &n)); // d=3, within [0,3]
        assert!(!m.str_within("kitten", "completely_different", &n)); // d>>3
    }

    // ===== Neighborhood tests =====

    #[test]
    fn fuzzy_neighborhood_containment() {
        let n = fuzzy_neighborhood(2);
        assert!(n.contains(Distance::new(0.0)));
        assert!(n.contains(Distance::new(2.0))); // boundary, closed
        assert!(!n.contains(Distance::new(3.0)));
    }

    #[test]
    fn adaptive_neighborhood_scaling() {
        let short = adaptive_fuzzy_neighborhood(5, 0.8);
        let long = adaptive_fuzzy_neighborhood(20, 0.8);

        // Longer strings get wider neighborhoods
        assert!(short.radius.value() < long.radius.value());
        // ceil(5 * 0.2) = 1, ceil(20 * 0.2) = 4
        assert!((short.radius.value() - 1.0).abs() < f64::EPSILON);
        assert!((long.radius.value() - 4.0).abs() < f64::EPSILON);
    }

    #[test]
    fn similarity_neighborhood_consistency() {
        let n = similarity_neighborhood(7, 0.5);
        // max_dist = 7 * 0.5 = 3.5
        assert!((n.radius.value() - 3.5).abs() < f64::EPSILON);
        // kitten→sitting: d=3, within [0, 3.5]
        assert!(n.contains(Distance::new(3.0)));
    }
}
