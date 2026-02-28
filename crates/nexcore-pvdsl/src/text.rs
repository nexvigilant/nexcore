//! Inline text utilities for PVDSL.
//!
//! Provides Levenshtein edit distance without external dependencies.

/// Levenshtein edit distance result.
pub struct LevenshteinResult {
    /// Raw edit distance (number of operations).
    pub distance: usize,
    /// Similarity ratio [0.0, 1.0] where 1.0 = identical.
    pub similarity: f64,
}

/// Compute Levenshtein distance with full result including similarity ratio.
///
/// Delegates to the canonical `nexcore-edit-distance` implementation.
#[must_use]
pub fn levenshtein(source: &str, target: &str) -> LevenshteinResult {
    let result = nexcore_edit_distance::classic::levenshtein(source, target);
    LevenshteinResult {
        distance: result.distance,
        similarity: result.similarity,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_basic() {
        let result = levenshtein("kitten", "sitting");
        assert_eq!(result.distance, 3);
    }

    #[test]
    fn test_levenshtein_identical() {
        let result = levenshtein("abc", "abc");
        assert_eq!(result.distance, 0);
        assert!((result.similarity - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_levenshtein_empty() {
        let result = levenshtein("", "abc");
        assert_eq!(result.distance, 3);
        assert!((result.similarity - 0.0).abs() < f64::EPSILON);
    }
}
