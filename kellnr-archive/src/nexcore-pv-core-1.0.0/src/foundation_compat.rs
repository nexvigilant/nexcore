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
/// Uses Wagner-Fischer algorithm with O(min(m,n)) space optimization.
#[must_use]
pub fn levenshtein_distance(source: &str, target: &str) -> usize {
    let source_chars: Vec<char> = source.chars().collect();
    let target_chars: Vec<char> = target.chars().collect();

    let m = source_chars.len();
    let n = target_chars.len();

    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }

    let (shorter, longer, short_len, long_len) = if m <= n {
        (&source_chars, &target_chars, m, n)
    } else {
        (&target_chars, &source_chars, n, m)
    };

    let mut prev_row: Vec<usize> = (0..=short_len).collect();
    let mut curr_row: Vec<usize> = vec![0; short_len + 1];

    for i in 1..=long_len {
        curr_row[0] = i;
        for j in 1..=short_len {
            let cost = usize::from(longer[i - 1] != shorter[j - 1]);
            curr_row[j] = (prev_row[j] + 1)
                .min(curr_row[j - 1] + 1)
                .min(prev_row[j - 1] + cost);
        }
        std::mem::swap(&mut prev_row, &mut curr_row);
    }

    prev_row[short_len]
}

/// Compute Levenshtein distance with full result including similarity ratio.
#[must_use]
pub fn levenshtein(source: &str, target: &str) -> LevenshteinResult {
    let distance = levenshtein_distance(source, target);
    let max_len = source.chars().count().max(target.chars().count());
    let similarity = if max_len == 0 {
        1.0
    } else {
        1.0 - (distance as f64 / max_len as f64)
    };

    LevenshteinResult {
        distance,
        similarity: (similarity * 10000.0).round() / 10000.0,
        source_len: source.chars().count(),
        target_len: target.chars().count(),
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
#[must_use]
pub fn levenshtein_bounded(source: &str, target: &str, max_distance: usize) -> Option<usize> {
    let source_chars: Vec<char> = source.chars().collect();
    let target_chars: Vec<char> = target.chars().collect();

    let m = source_chars.len();
    let n = target_chars.len();

    if m.abs_diff(n) > max_distance {
        return None;
    }
    if m == 0 {
        return if n <= max_distance { Some(n) } else { None };
    }
    if n == 0 {
        return if m <= max_distance { Some(m) } else { None };
    }

    let (shorter, longer, short_len, long_len) = if m <= n {
        (&source_chars, &target_chars, m, n)
    } else {
        (&target_chars, &source_chars, n, m)
    };

    let mut prev_row: Vec<usize> = (0..=short_len).collect();
    let mut curr_row: Vec<usize> = vec![0; short_len + 1];

    for i in 1..=long_len {
        curr_row[0] = i;
        for j in 1..=short_len {
            let cost = usize::from(longer[i - 1] != shorter[j - 1]);
            curr_row[j] = (prev_row[j] + 1)
                .min(curr_row[j - 1] + 1)
                .min(prev_row[j - 1] + cost);
        }
        if curr_row.iter().all(|&v| v > max_distance) {
            return None;
        }
        std::mem::swap(&mut prev_row, &mut curr_row);
    }

    let distance = prev_row[short_len];
    if distance <= max_distance {
        Some(distance)
    } else {
        None
    }
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
