//! # Foundation Compatibility Layer
//!
//! Inlined foundation utilities from `nexcore-vigilance::foundation` that
//! `nexcore-pv-core` modules depend on. This avoids a circular dependency
//! back to vigilance.

use serde::{Deserialize, Serialize};

// =============================================================================
// Levenshtein distance (from foundation::algorithms::levenshtein)
// =============================================================================

/// Result of a Levenshtein distance calculation
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LevenshteinResult {
    /// Edit distance (number of insertions, deletions, substitutions)
    pub distance: usize,
    /// Normalized similarity (0.0 to 1.0, where 1.0 = identical)
    pub similarity: f64,
    /// Length of source string in characters
    pub source_len: usize,
    /// Length of target string in characters
    pub target_len: usize,
}

/// Compute Levenshtein edit distance between two strings.
///
/// Delegates to the canonical `nexcore-edit-distance` implementation.
#[must_use]
pub fn levenshtein_distance(source: &str, target: &str) -> usize {
    nexcore_edit_distance::classic::levenshtein_distance(source, target)
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
        source_len: result.source_len,
        target_len: result.target_len,
    }
}

/// Result of a fuzzy match operation
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FuzzyMatch {
    /// The matched candidate string
    pub candidate: String,
    /// Edit distance from query
    pub distance: usize,
    /// Normalized similarity (0.0 to 1.0)
    pub similarity: f64,
}

/// Compute Levenshtein distance with early termination when distance exceeds threshold.
///
/// Delegates to the canonical `nexcore-edit-distance` implementation.
#[must_use]
pub fn levenshtein_bounded(source: &str, target: &str, max_distance: usize) -> Option<usize> {
    nexcore_edit_distance::classic::levenshtein_bounded(source, target, max_distance)
}

/// Batch fuzzy search: find best matches for a query against candidates.
#[must_use]
pub fn fuzzy_search(query: &str, candidates: &[String], limit: usize) -> Vec<FuzzyMatch> {
    if candidates.is_empty() || limit == 0 {
        return Vec::new();
    }

    let query_len = query.chars().count();
    let mut top_k: Vec<FuzzyMatch> = Vec::with_capacity(limit + 1);
    let mut worst_sim: f64 = -1.0;

    for c in candidates {
        let c_len = c.chars().count();
        let max_len = query_len.max(c_len);

        let max_dist = if top_k.len() < limit {
            max_len
        } else {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let d = (max_len as f64 * (1.0 - worst_sim)).ceil() as usize;
            d.min(max_len)
        };

        let Some(distance) = levenshtein_bounded(query, c, max_dist) else {
            continue;
        };

        let similarity = if max_len == 0 {
            1.0
        } else {
            ((1.0 - (distance as f64 / max_len as f64)) * 10000.0).round() / 10000.0
        };

        if top_k.len() < limit || similarity > worst_sim {
            top_k.push(FuzzyMatch {
                candidate: c.clone(),
                distance,
                similarity,
            });

            if top_k.len() > limit {
                top_k.sort_unstable_by(|a, b| {
                    b.similarity
                        .partial_cmp(&a.similarity)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                top_k.truncate(limit);
            }

            if top_k.len() >= limit {
                worst_sim = top_k
                    .iter()
                    .map(|m| m.similarity)
                    .fold(f64::INFINITY, f64::min);
            }
        }
    }

    top_k.sort_by(|a, b| {
        b.similarity
            .partial_cmp(&a.similarity)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.candidate.cmp(&b.candidate))
    });

    top_k
}

// =============================================================================
// Safety traits (from foundation::traits)
// =============================================================================

/// A result that includes a mandatory safety assessment.
pub struct VigilantResult<T> {
    /// The raw computational result
    pub data: T,
    /// The associated safety margin (d(s))
    pub safety_margin: f32,
    /// The epistemic trust score (0.0-1.0)
    pub trust_score: f64,
}

/// A trait for calculations that must be performed within safety axioms.
pub trait SafeCalculable {
    /// The input type for the calculation.
    type Input;
    /// The output type for the calculation.
    type Output;

    /// Calculate the result and automatically compute the safety manifold distance.
    fn calculate_safe(&self, input: Self::Input) -> VigilantResult<Self::Output>;
}

// Convenience aliases matching the original vigilance::foundation paths
pub mod algorithms {
    pub mod levenshtein {
        pub use crate::foundation_compat::{
            FuzzyMatch, LevenshteinResult, fuzzy_search, levenshtein, levenshtein_bounded,
            levenshtein_distance,
        };
    }
}

pub mod traits {
    pub use crate::foundation_compat::{SafeCalculable, VigilantResult};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_basic() {
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("", ""), 0);
        assert_eq!(levenshtein_distance("abc", ""), 3);
    }

    #[test]
    fn test_fuzzy_search_basic() {
        let candidates = vec![
            "commit".to_string(),
            "comment".to_string(),
            "comet".to_string(),
        ];
        let results = fuzzy_search("comit", &candidates, 3);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].candidate, "commit");
    }
}
