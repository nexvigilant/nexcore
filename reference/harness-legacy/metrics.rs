//! Coverage metrics calculations for test harness
//!
//! Provides set operations and similarity metrics for measuring test coverage.
//! Uses `HashSet` for O(1) membership testing and efficient set operations.

use std::collections::HashSet;
use std::hash::Hash;

/// Coverage report containing primitive and test function data
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CoverageReport {
    /// Functions/classes in source code
    pub source_primitives: Vec<String>,
    /// Test functions found
    pub test_functions: Vec<String>,
    /// Primitives that appear to have test coverage
    pub covered_primitives: Vec<String>,
    /// Primitives without apparent test coverage
    pub uncovered_primitives: Vec<String>,
    /// Jaccard similarity score (0.0 - 1.0)
    pub jaccard_score: f64,
    /// Simple coverage ratio (covered / total)
    pub coverage_ratio: f64,
}

/// Calculate Jaccard similarity coefficient between two sets
///
/// J(A,B) = |A ∩ B| / |A ∪ B|
///
/// Returns 1.0 if both sets are empty (vacuously similar)
///
/// # Examples
/// ```
/// use rust_skills::harness::metrics::jaccard_similarity;
/// let a = vec!["foo", "bar", "baz"];
/// let b = vec!["bar", "baz", "qux"];
/// let score = jaccard_similarity(&a, &b);
/// assert!((score - 0.5).abs() < 0.001); // 2 common / 4 total = 0.5
/// ```
pub fn jaccard_similarity<T: Eq + Hash>(set_a: &[T], set_b: &[T]) -> f64 {
    let a: HashSet<&T> = set_a.iter().collect();
    let b: HashSet<&T> = set_b.iter().collect();

    let intersection = a.intersection(&b).count();
    let union = a.union(&b).count();

    if union == 0 {
        1.0 // Both empty sets are vacuously similar
    } else {
        intersection as f64 / union as f64
    }
}

/// Calculate set intersection
pub fn set_intersection<T: Eq + Hash + Clone>(set_a: &[T], set_b: &[T]) -> Vec<T> {
    let a: HashSet<&T> = set_a.iter().collect();
    let b: HashSet<&T> = set_b.iter().collect();

    a.intersection(&b).map(|&x| x.clone()).collect()
}

/// Calculate set difference (A - B)
pub fn set_difference<T: Eq + Hash + Clone>(set_a: &[T], set_b: &[T]) -> Vec<T> {
    let a: HashSet<&T> = set_a.iter().collect();
    let b: HashSet<&T> = set_b.iter().collect();

    a.difference(&b).map(|&x| x.clone()).collect()
}

/// Normalize a function name for matching
///
/// Converts test function names to their likely source function:
/// - `test_foo_bar` -> `foo_bar`
/// - `test_FooBar` -> `FooBar`
/// - `TestFooBar` -> `FooBar`
/// - `foo_test` -> `foo`
pub fn normalize_test_name(name: &str) -> String {
    let mut normalized = name.to_string();

    // Remove common test prefixes
    for prefix in ["test_", "Test", "test"] {
        if normalized.starts_with(prefix) {
            normalized = normalized[prefix.len()..].to_string();
            break;
        }
    }

    // Remove common test suffixes
    for suffix in ["_test", "Test", "_spec", "Spec"] {
        if normalized.ends_with(suffix) {
            normalized = normalized[..normalized.len() - suffix.len()].to_string();
            break;
        }
    }

    normalized
}

/// Calculate coverage by matching test functions to source primitives
///
/// Uses fuzzy matching: a source primitive is "covered" if any test function
/// contains its name (after normalization).
pub fn calculate_coverage(
    source_primitives: &[String],
    test_functions: &[String],
) -> CoverageReport {
    // Normalize test names for matching
    let normalized_tests: Vec<String> = test_functions
        .iter()
        .map(|t| normalize_test_name(t).to_lowercase())
        .collect();

    let mut covered = Vec::new();
    let mut uncovered = Vec::new();

    for primitive in source_primitives {
        let primitive_lower = primitive.to_lowercase();
        let is_covered = normalized_tests
            .iter()
            .any(|test| test.contains(&primitive_lower) || primitive_lower.contains(test));

        if is_covered {
            covered.push(primitive.clone());
        } else {
            uncovered.push(primitive.clone());
        }
    }

    let coverage_ratio = if source_primitives.is_empty() {
        1.0
    } else {
        covered.len() as f64 / source_primitives.len() as f64
    };

    // Jaccard between covered primitives and all test functions (normalized)
    let jaccard_score = jaccard_similarity(&covered, &normalized_tests);

    CoverageReport {
        source_primitives: source_primitives.to_vec(),
        test_functions: test_functions.to_vec(),
        covered_primitives: covered,
        uncovered_primitives: uncovered,
        jaccard_score,
        coverage_ratio,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jaccard_identical_sets() {
        let a = vec!["foo", "bar", "baz"];
        let b = vec!["foo", "bar", "baz"];
        assert!((jaccard_similarity(&a, &b) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_jaccard_disjoint_sets() {
        let a = vec!["foo", "bar"];
        let b = vec!["baz", "qux"];
        assert!(jaccard_similarity(&a, &b).abs() < f64::EPSILON);
    }

    #[test]
    fn test_jaccard_partial_overlap() {
        let a = vec!["foo", "bar", "baz"];
        let b = vec!["bar", "baz", "qux"];
        // Intersection: {bar, baz} = 2
        // Union: {foo, bar, baz, qux} = 4
        // J = 2/4 = 0.5
        assert!((jaccard_similarity(&a, &b) - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_jaccard_empty_sets() {
        let a: Vec<&str> = vec![];
        let b: Vec<&str> = vec![];
        assert!((jaccard_similarity(&a, &b) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_set_intersection() {
        let a = vec!["foo", "bar", "baz"];
        let b = vec!["bar", "baz", "qux"];
        let result = set_intersection(&a, &b);
        assert!(result.contains(&"bar"));
        assert!(result.contains(&"baz"));
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_set_difference() {
        let a = vec!["foo", "bar", "baz"];
        let b = vec!["bar", "baz", "qux"];
        let result = set_difference(&a, &b);
        assert_eq!(result, vec!["foo"]);
    }

    #[test]
    fn test_normalize_test_name() {
        assert_eq!(normalize_test_name("test_foo_bar"), "foo_bar");
        assert_eq!(normalize_test_name("TestFooBar"), "FooBar");
        assert_eq!(normalize_test_name("foo_test"), "foo");
        assert_eq!(normalize_test_name("FooSpec"), "Foo");
        assert_eq!(normalize_test_name("regular_function"), "regular_function");
    }

    #[test]
    fn test_calculate_coverage() {
        let source = vec![
            "foo".to_string(),
            "bar".to_string(),
            "baz".to_string(),
        ];
        let tests = vec![
            "test_foo".to_string(),
            "test_bar".to_string(),
        ];

        let report = calculate_coverage(&source, &tests);

        assert_eq!(report.covered_primitives.len(), 2);
        assert_eq!(report.uncovered_primitives.len(), 1);
        assert!(report.uncovered_primitives.contains(&"baz".to_string()));
        assert!((report.coverage_ratio - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_coverage_empty_source() {
        let source: Vec<String> = vec![];
        let tests = vec!["test_foo".to_string()];

        let report = calculate_coverage(&source, &tests);
        assert!((report.coverage_ratio - 1.0).abs() < f64::EPSILON);
    }
}
