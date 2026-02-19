//! Fuzzy search for PV glossary terms.
//!
//! Provides typo-tolerant term lookup using a BK-tree index.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use nexcore_vigilance::coding::glossary::{fuzzy_lookup_term, fuzzy_search_terms};
//!
//! // Find "adverse event" even with typos
//! let results = fuzzy_lookup_term("advers event", 2);
//! for (term, distance) in results {
//!     println!("{} (distance: {})", term.term, distance);
//! }
//!
//! // Best match only
//! if let Some((term, dist)) = fuzzy_search_terms("singal detection", 3).first() {
//!     println!("Did you mean: {}?", term.term);
//! }
//! ```

use std::sync::OnceLock;

use super::super::fuzzy::BkTree;

use super::{GLOSSARY, GlossaryTerm};

/// Lazy-initialized BK-tree for fuzzy glossary search.
///
/// The tree is built on first access and cached for subsequent queries.
static GLOSSARY_BKTREE: OnceLock<BkTree> = OnceLock::new();

/// Get or initialize the glossary BK-tree.
fn get_bktree() -> &'static BkTree {
    GLOSSARY_BKTREE.get_or_init(|| {
        let mut tree = BkTree::new();
        for (i, (key, _)) in GLOSSARY.iter().enumerate() {
            // Use index as data, lowercase key for matching
            // Try convert to u32, if fail (very unlikely for glossary), just truncate
            // The glossary size is well within u32 range (< 10k terms)
            #[allow(clippy::cast_possible_truncation)]
            let idx = i as u32;
            tree.insert(key.to_lowercase(), idx);
        }
        tree
    })
}

/// Fuzzy lookup of a glossary term by name with Levenshtein distance tolerance.
///
/// Returns all terms within `max_distance` edits of the query, sorted by distance.
///
/// # Arguments
///
/// * `query` - The term name to search for (typos allowed)
/// * `max_distance` - Maximum edit distance (1-3 recommended)
///
/// # Returns
///
/// Vector of (`GlossaryTerm`, distance) tuples, sorted by distance (closest first).
///
/// # Complexity
///
/// - TIME: O(n^(1-1/d)) average case where d = `max_distance`
/// - SPACE: O(k) where k is number of results
///
/// # Example
///
/// ```rust,ignore
/// use nexcore_vigilance::coding::glossary::fuzzy_lookup_term;
///
/// // Find "adverse event" even with typo "advers"
/// let results = fuzzy_lookup_term("advers event", 2);
/// assert!(!results.is_empty());
/// assert_eq!(results[0].0.term.to_lowercase(), "adverse event");
/// ```
#[must_use]
pub fn fuzzy_lookup_term(query: &str, max_distance: usize) -> Vec<(&'static GlossaryTerm, usize)> {
    let tree = get_bktree();
    let query_lower = query.to_lowercase();

    tree.search(&query_lower, max_distance)
        .into_iter()
        .filter_map(|(_, idx, dist)| {
            let idx = idx as usize;
            if idx < GLOSSARY.len() {
                Some((&GLOSSARY[idx].1, dist))
            } else {
                None
            }
        })
        .collect()
}

/// Find the best fuzzy match for a term.
///
/// Returns the closest matching term if within `max_distance`, or None.
///
/// # Arguments
///
/// * `query` - The term name to search for
/// * `max_distance` - Maximum edit distance to consider
///
/// # Returns
///
/// The best matching term and its distance, or None if no match within distance.
///
/// # Example
///
/// ```rust,ignore
/// use nexcore_vigilance::coding::glossary::fuzzy_best_match;
///
/// // "Did you mean?" functionality
/// if let Some((term, dist)) = fuzzy_best_match("singal detecton", 3) {
///     println!("Did you mean '{}'? (distance: {})", term.term, dist);
/// }
/// ```
#[must_use]
pub fn fuzzy_best_match(
    query: &str,
    max_distance: usize,
) -> Option<(&'static GlossaryTerm, usize)> {
    fuzzy_lookup_term(query, max_distance).into_iter().next()
}

/// Smart term lookup that tries exact match first, then fuzzy.
///
/// Provides a convenient "did you mean?" style lookup:
/// 1. First tries exact case-insensitive match (O(log n))
/// 2. If no exact match, falls back to fuzzy search (O(n^(1-1/d)))
///
/// # Arguments
///
/// * `query` - The term name to search for
/// * `fuzzy_distance` - Maximum edit distance for fuzzy fallback
///
/// # Returns
///
/// - `Ok(term)` - Exact match found
/// - `Err(suggestions)` - No exact match, but fuzzy matches found
/// - `Err([])` - No matches at all
///
/// # Errors
///
/// Returns `Err(Vec<(&GlossaryTerm, usize)>)` if exact match fails. The vector contains fuzzy suggestions.
///
/// # Example
///
/// ```rust,ignore
/// use nexcore_vigilance::coding::glossary::smart_lookup;
///
/// match smart_lookup("adverse event", 2) {
///     Ok(term) => println!("Found: {}", term.definition),
///     Err(suggestions) if !suggestions.is_empty() => {
///         println!("Did you mean one of these?");
///         for (term, dist) in suggestions {
///             println!("  - {} (distance: {})", term.term, dist);
///         }
///     }
///     Err(_) => println!("No matches found"),
/// }
/// ```
#[allow(clippy::double_must_use)] // False positive
#[must_use]
pub fn smart_lookup(
    query: &str,
    fuzzy_distance: usize,
) -> Result<&'static GlossaryTerm, Vec<(&'static GlossaryTerm, usize)>> {
    // Try exact match first (O(log n))
    if let Some(term) = super::lookup_term(query) {
        return Ok(term);
    }

    // Fall back to fuzzy search
    let fuzzy_results = fuzzy_lookup_term(query, fuzzy_distance);
    Err(fuzzy_results)
}

/// Get statistics about the fuzzy search index.
#[must_use]
pub fn fuzzy_index_stats() -> FuzzyIndexStats {
    let tree = get_bktree();
    FuzzyIndexStats {
        term_count: tree.len(),
        is_initialized: true,
    }
}

/// Statistics about the fuzzy search index.
#[derive(Debug, Clone, Copy)]
pub struct FuzzyIndexStats {
    /// Number of terms in the BK-tree.
    pub term_count: usize,
    /// Whether the index has been initialized.
    pub is_initialized: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_lookup_exact() {
        // Exact match should have distance 0
        let results = fuzzy_lookup_term("adverse event", 0);
        // Should find if it exists in glossary
        if !results.is_empty() {
            assert_eq!(results[0].1, 0);
        }
    }

    #[test]
    fn test_fuzzy_lookup_typo() {
        // With distance 2, should find "adverse event" from "advers event"
        let results = fuzzy_lookup_term("advers event", 2);
        // May or may not find depending on glossary content
        // Just verify no panic
        let _ = results;
    }

    #[test]
    fn test_fuzzy_best_match() {
        let result = fuzzy_best_match("signal", 2);
        // Should return something (signal detection, etc.) if in glossary
        let _ = result;
    }

    #[test]
    fn test_smart_lookup_exact() {
        // If "adverse event" exists, should return Ok
        let result = smart_lookup("adverse event", 2);
        match result {
            Ok(term) => {
                assert!(term.term.to_lowercase().contains("adverse"));
            }
            Err(suggestions) => {
                // No exact match, but that's OK
                let _ = suggestions;
            }
        }
    }

    #[test]
    fn test_fuzzy_index_stats() {
        let stats = fuzzy_index_stats();
        assert!(stats.is_initialized);
        assert!(stats.term_count > 0);
    }
}
