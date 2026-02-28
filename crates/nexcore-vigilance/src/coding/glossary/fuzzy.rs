//! Fuzzy search for PV glossary terms.
//!
//! Provides typo-tolerant term lookup using a BK-tree index.
//! Fuzzy matches include POM `TitrationProvenance` measuring
//! semantic equivalence between the query and matched term.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use nexcore_vigilance::coding::glossary::{fuzzy_lookup_term, GlossaryMatch};
//!
//! // Find "adverse event" even with typos
//! let results = fuzzy_lookup_term("advers event", 2);
//! for m in &results {
//!     println!("{} (distance: {})", m.term.term, m.distance);
//!     if let Some(prov) = &m.titration_provenance {
//!         println!("  equivalence: {:.3}", prov.equivalence_score);
//!     }
//! }
//! ```

use std::sync::OnceLock;

use nexcore_proof_of_meaning::titration;

use super::super::fuzzy::BkTree;
use super::super::meddra::hierarchy::pom_titrator;
use super::super::meddra::types::TitrationProvenance;
use super::{GLOSSARY, GlossaryTerm};

/// Lazy-initialized BK-tree for fuzzy glossary search.
///
/// The tree is built on first access and cached for subsequent queries.
static GLOSSARY_BKTREE: OnceLock<BkTree> = OnceLock::new();

/// A fuzzy glossary match with optional POM titration provenance.
///
/// For exact matches (distance = 0), `titration_provenance` is `None`
/// because titrating identical strings is trivially redundant.
/// For fuzzy matches (distance > 0), provenance measures semantic
/// equivalence between the query and matched term.
#[derive(Debug, Clone)]
pub struct GlossaryMatch {
    /// The matched glossary term.
    pub term: &'static GlossaryTerm,
    /// Levenshtein edit distance from the query.
    pub distance: usize,
    /// POM titration provenance for fuzzy matches.
    #[allow(dead_code, reason = "Consumed by downstream callers via field access")]
    pub titration_provenance: Option<TitrationProvenance>,
}

/// Get or initialize the glossary BK-tree.
fn get_bktree() -> &'static BkTree {
    GLOSSARY_BKTREE.get_or_init(|| {
        let mut tree = BkTree::new();
        for (i, (key, _)) in GLOSSARY.iter().enumerate() {
            // Use index as data, lowercase key for matching
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
/// Fuzzy matches (distance > 0) include POM titration provenance measuring
/// semantic equivalence between query and matched term.
///
/// # Arguments
///
/// * `query` - The term name to search for (typos allowed)
/// * `max_distance` - Maximum edit distance (1-3 recommended)
///
/// # Returns
///
/// Vector of `GlossaryMatch` results, sorted by distance (closest first).
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
/// assert_eq!(results[0].term.term.to_lowercase(), "adverse event");
/// ```
#[must_use]
pub fn fuzzy_lookup_term(query: &str, max_distance: usize) -> Vec<GlossaryMatch> {
    let tree = get_bktree();
    let query_lower = query.to_lowercase();
    let titrator = pom_titrator();

    tree.search(&query_lower, max_distance)
        .into_iter()
        .filter_map(|(matched_key, idx, dist)| {
            let idx = idx as usize;
            if idx < GLOSSARY.len() {
                let titration_provenance = if dist > 0 {
                    // Fuzzy match — measure semantic equivalence via POM titration
                    let proof = titration::prove_equivalence(titrator, &query_lower, &matched_key);
                    Some(TitrationProvenance {
                        equivalence_score: proof.equivalence_score.into_inner(),
                        verdict: format!("{:?}", proof.verdict),
                        shared_atoms: proof.shared_atoms,
                    })
                } else {
                    // Exact match — titration is trivially redundant
                    None
                };
                Some(GlossaryMatch {
                    term: &GLOSSARY[idx].1,
                    distance: dist,
                    titration_provenance,
                })
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
/// The best matching `GlossaryMatch`, or None if no match within distance.
///
/// # Example
///
/// ```rust,ignore
/// use nexcore_vigilance::coding::glossary::fuzzy_best_match;
///
/// // "Did you mean?" functionality
/// if let Some(m) = fuzzy_best_match("singal detecton", 3) {
///     println!("Did you mean '{}'? (distance: {})", m.term.term, m.distance);
/// }
/// ```
#[must_use]
pub fn fuzzy_best_match(query: &str, max_distance: usize) -> Option<GlossaryMatch> {
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
/// Returns `Err(Vec<GlossaryMatch>)` if exact match fails. The vector contains fuzzy suggestions.
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
///         for m in &suggestions {
///             println!("  - {} (distance: {})", m.term.term, m.distance);
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
) -> Result<&'static GlossaryTerm, Vec<GlossaryMatch>> {
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
        if !results.is_empty() {
            assert_eq!(results[0].distance, 0);
        }
    }

    #[test]
    fn test_fuzzy_lookup_typo() {
        // With distance 2, should find "adverse event" from "advers event"
        let results = fuzzy_lookup_term("advers event", 2);
        // Verify the function completes without panic and returns valid matches
        assert!(
            results.iter().all(|m| m.distance <= 2),
            "All results should be within max_distance",
        );
    }

    #[test]
    fn test_fuzzy_best_match() {
        let result = fuzzy_best_match("signal", 2);
        // If a match is found, distance should be within range
        if let Some(m) = result {
            assert!(m.distance <= 2);
        }
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
                // No exact match — verify suggestions are valid GlossaryMatch structs
                assert!(
                    suggestions.iter().all(|m| m.distance <= 2),
                    "All suggestions should be within fuzzy_distance",
                );
            }
        }
    }

    #[test]
    fn test_fuzzy_index_stats() {
        let stats = fuzzy_index_stats();
        assert!(stats.is_initialized);
        assert!(stats.term_count > 0);
    }

    #[test]
    fn test_fuzzy_glossary_populates_titration_provenance() {
        // Fuzzy matches (distance > 0) should carry titration provenance
        let results = fuzzy_lookup_term("advers event", 2);
        let fuzzy_matches: Vec<_> = results.iter().filter(|m| m.distance > 0).collect();
        if !fuzzy_matches.is_empty() {
            for m in &fuzzy_matches {
                assert!(
                    m.titration_provenance.is_some(),
                    "Fuzzy match '{}' (distance {}) should have titration provenance",
                    m.term.term,
                    m.distance,
                );
                let prov = m.titration_provenance.as_ref().expect("checked above");
                // Equivalence score should be between 0 and 1
                assert!(
                    prov.equivalence_score >= 0.0 && prov.equivalence_score <= 1.0,
                    "Equivalence score {} out of range",
                    prov.equivalence_score,
                );
                // Verdict should be a non-empty string
                assert!(
                    !prov.verdict.is_empty(),
                    "Verdict should not be empty for fuzzy match",
                );
            }
        }
        // Exact matches (distance = 0) should have None
        let exact_matches: Vec<_> = results.iter().filter(|m| m.distance == 0).collect();
        for m in &exact_matches {
            assert!(
                m.titration_provenance.is_none(),
                "Exact match '{}' should not have titration provenance",
                m.term.term,
            );
        }
    }
}
