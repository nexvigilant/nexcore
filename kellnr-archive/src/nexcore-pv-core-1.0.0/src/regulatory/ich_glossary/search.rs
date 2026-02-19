//! Search Module
//!
//! Provides search and query functionality for ICH glossary terms,
//! including exact match, prefix search, contains, and fuzzy matching.

use crate::regulatory::ich_glossary::terms::{TERMS, all_terms};
use crate::regulatory::ich_glossary::types::{
    IchCategory, MatchType, SearchResult, Term, TermQuery,
};

// ============================================================================
// Search Functions
// ============================================================================

/// Search terms by query string
/// Returns results sorted by relevance score (highest first)
pub fn search_terms(query: &str) -> Vec<SearchResult> {
    search_with_query(TermQuery::new(query))
}

/// Search terms with full query parameters
pub fn search_with_query(query: TermQuery) -> Vec<SearchResult> {
    let query_lower = query.query.to_lowercase();
    let query_words: Vec<&str> = query_lower.split_whitespace().collect();

    let mut results: Vec<SearchResult> = Vec::new();

    for term in TERMS.values() {
        // Apply category filter
        if let Some(cat) = query.category {
            if term.source.category() != Some(cat) {
                continue;
            }
        }

        // Apply guideline filter
        if let Some(ref guideline) = query.guideline {
            let guideline_lower = guideline.to_lowercase();
            if !term
                .source
                .guideline_id
                .to_lowercase()
                .contains(&guideline_lower)
            {
                continue;
            }
        }

        // Calculate match
        if let Some(result) =
            calculate_match(term, &query_lower, &query_words, query.search_definitions)
        {
            // Apply minimum score filter
            if let Some(min_score) = query.min_score {
                if result.score < min_score {
                    continue;
                }
            }
            results.push(result);
        }
    }

    // Sort by score descending
    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Apply limit
    if let Some(limit) = query.limit {
        results.truncate(limit);
    }

    results
}

/// Calculate match score for a term against a query
fn calculate_match(
    term: &'static Term,
    query_lower: &str,
    query_words: &[&str],
    search_definitions: bool,
) -> Option<SearchResult> {
    let term_name_lower = term.name.to_lowercase();
    let term_key_lower = term.key.to_lowercase();

    // Check for exact match
    if term_name_lower == query_lower || term_key_lower == query_lower.replace(' ', "_") {
        return Some(SearchResult {
            term: term.clone(),
            score: 1.0,
            match_type: MatchType::Exact,
        });
    }

    // Check for abbreviation match
    if let Some(abbrev) = term.abbreviation {
        if abbrev.to_lowercase() == query_lower {
            return Some(SearchResult {
                term: term.clone(),
                score: 0.95,
                match_type: MatchType::Exact,
            });
        }
    }

    // Check for prefix match
    if term_name_lower.starts_with(query_lower)
        || term_key_lower.starts_with(&query_lower.replace(' ', "_"))
    {
        let coverage = query_lower.len() as f64 / term_name_lower.len() as f64;
        return Some(SearchResult {
            term: term.clone(),
            score: 0.8 + (coverage * 0.1),
            match_type: MatchType::Prefix,
        });
    }

    // Check for contains match (all query words in term name)
    let name_contains_all = query_words.iter().all(|w| term_name_lower.contains(w));
    if name_contains_all {
        let coverage = query_words.iter().map(|w| w.len()).sum::<usize>() as f64
            / term_name_lower.len() as f64;
        return Some(SearchResult {
            term: term.clone(),
            score: 0.6 + (coverage * 0.1).min(0.1),
            match_type: MatchType::Contains,
        });
    }

    // Check for partial contains (some words)
    let words_found = query_words
        .iter()
        .filter(|w| term_name_lower.contains(*w))
        .count();
    if words_found > 0 && words_found < query_words.len() {
        let partial_score = words_found as f64 / query_words.len() as f64;
        return Some(SearchResult {
            term: term.clone(),
            score: 0.4 + (partial_score * 0.2),
            match_type: MatchType::Contains,
        });
    }

    // Search definitions if enabled
    if search_definitions {
        let def_lower = term.definition.to_lowercase();
        let def_contains_all = query_words.iter().all(|w| def_lower.contains(w));
        if def_contains_all {
            return Some(SearchResult {
                term: term.clone(),
                score: 0.5,
                match_type: MatchType::DefinitionMatch,
            });
        }

        let def_words_found = query_words
            .iter()
            .filter(|w| def_lower.contains(*w))
            .count();
        if def_words_found > query_words.len() / 2 {
            let partial_score = def_words_found as f64 / query_words.len() as f64;
            return Some(SearchResult {
                term: term.clone(),
                score: 0.3 + (partial_score * 0.2),
                match_type: MatchType::DefinitionMatch,
            });
        }
    }

    // Fuzzy match using Levenshtein-like scoring
    let fuzzy_score = calculate_fuzzy_score(&term_name_lower, query_lower);
    if fuzzy_score > 0.5 {
        return Some(SearchResult {
            term: term.clone(),
            score: fuzzy_score * 0.6,
            match_type: MatchType::Fuzzy,
        });
    }

    None
}

/// Calculate fuzzy similarity score (0.0 - 1.0)
fn calculate_fuzzy_score(s1: &str, s2: &str) -> f64 {
    if s1.is_empty() || s2.is_empty() {
        return 0.0;
    }

    // Simple character overlap scoring
    let s1_chars: std::collections::HashSet<char> = s1.chars().collect();
    let s2_chars: std::collections::HashSet<char> = s2.chars().collect();

    let intersection = s1_chars.intersection(&s2_chars).count();
    let union = s1_chars.union(&s2_chars).count();

    if union == 0 {
        return 0.0;
    }

    // Jaccard similarity
    let jaccard = intersection as f64 / union as f64;

    // Bonus for substring containment
    let containment_bonus = if s1.contains(s2) || s2.contains(s1) {
        0.2
    } else {
        0.0
    };

    // Bonus for same starting characters
    let common_prefix = s1
        .chars()
        .zip(s2.chars())
        .take_while(|(a, b)| a == b)
        .count();
    let prefix_bonus = (common_prefix as f64 / s1.len().min(s2.len()) as f64) * 0.2;

    (jaccard + containment_bonus + prefix_bonus).min(1.0)
}

// ============================================================================
// Specialized Search Functions
// ============================================================================

/// Search for terms related to safety/pharmacovigilance
pub fn search_safety_terms(query: &str) -> Vec<SearchResult> {
    search_with_query(
        TermQuery::new(query)
            .with_category(IchCategory::Efficacy)
            .search_definitions(),
    )
}

/// Search for terms by guideline prefix (e.g., "E2" for all E2 series)
pub fn search_by_guideline_series(prefix: &str) -> Vec<&'static Term> {
    let prefix_lower = prefix.to_lowercase();
    all_terms()
        .into_iter()
        .filter(|t| {
            t.source
                .guideline_id
                .to_lowercase()
                .starts_with(&prefix_lower)
        })
        .collect()
}

/// Get related terms via "see also" references
pub fn get_related_terms(term_key: &str) -> Vec<&'static Term> {
    let key_normalized = term_key.to_lowercase().replace(' ', "_");

    // Find the source term
    let source_term = TERMS.get(&key_normalized);

    if let Some(term) = source_term {
        term.see_also
            .iter()
            .filter_map(|related_name| {
                let related_key = related_name.to_lowercase().replace(' ', "_");
                TERMS.get(&related_key)
            })
            .collect()
    } else {
        vec![]
    }
}

/// Autocomplete suggestions for partial input
pub fn autocomplete(partial: &str, limit: usize) -> Vec<&'static str> {
    let partial_lower = partial.to_lowercase();

    let mut suggestions: Vec<(&'static str, f64)> = TERMS
        .values()
        .filter_map(|term| {
            let name_lower = term.name.to_lowercase();
            if name_lower.starts_with(&partial_lower) {
                // Exact prefix match - high score
                Some((term.name, 1.0 - (name_lower.len() as f64 / 100.0)))
            } else if name_lower.contains(&partial_lower) {
                // Contains match - lower score
                Some((term.name, 0.5 - (name_lower.len() as f64 / 100.0)))
            } else {
                None
            }
        })
        .collect();

    suggestions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    suggestions.truncate(limit);
    suggestions.into_iter().map(|(name, _)| name).collect()
}

// ============================================================================
// Statistics
// ============================================================================

/// Get search statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchStats {
    pub total_terms: usize,
    pub terms_with_abbreviations: usize,
    pub terms_with_alternatives: usize,
    pub terms_by_category: Vec<(String, usize)>,
    pub new_terms: usize,
}

/// Calculate search statistics
pub fn get_search_stats() -> SearchStats {
    let terms = all_terms();

    SearchStats {
        total_terms: terms.len(),
        terms_with_abbreviations: terms.iter().filter(|t| t.abbreviation.is_some()).count(),
        terms_with_alternatives: terms
            .iter()
            .filter(|t| !t.alternative_definitions.is_empty())
            .count(),
        terms_by_category: vec![
            (
                "Quality".to_string(),
                terms
                    .iter()
                    .filter(|t| t.source.category() == Some(IchCategory::Quality))
                    .count(),
            ),
            (
                "Safety".to_string(),
                terms
                    .iter()
                    .filter(|t| t.source.category() == Some(IchCategory::Safety))
                    .count(),
            ),
            (
                "Efficacy".to_string(),
                terms
                    .iter()
                    .filter(|t| t.source.category() == Some(IchCategory::Efficacy))
                    .count(),
            ),
            (
                "Multidisciplinary".to_string(),
                terms
                    .iter()
                    .filter(|t| t.source.category() == Some(IchCategory::Multidisciplinary))
                    .count(),
            ),
        ],
        new_terms: terms.iter().filter(|t| t.is_new).count(),
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_terms_exact() {
        let results = search_terms("adverse event");
        assert!(!results.is_empty());
        assert_eq!(results[0].match_type, MatchType::Exact);
    }

    #[test]
    fn test_search_terms_prefix() {
        let results = search_terms("adverse");
        assert!(!results.is_empty());
        // Should find multiple adverse-related terms
        assert!(results.len() >= 2);
    }

    #[test]
    fn test_search_terms_abbreviation() {
        let results = search_terms("ADR");
        assert!(!results.is_empty());
        assert!(results[0].term.abbreviation.is_some());
    }

    #[test]
    fn test_search_with_category_filter() {
        let results =
            search_with_query(TermQuery::new("validation").with_category(IchCategory::Quality));
        for result in &results {
            assert_eq!(result.term.source.category(), Some(IchCategory::Quality));
        }
    }

    #[test]
    fn test_search_with_definitions() {
        let results = search_with_query(TermQuery::new("noxious").search_definitions());
        // Should find terms where "noxious" appears in definition
        assert!(!results.is_empty());
    }

    #[test]
    fn test_search_with_limit() {
        let results = search_with_query(TermQuery::new("clinical").limit(3));
        assert!(results.len() <= 3);
    }

    #[test]
    fn test_autocomplete() {
        let suggestions = autocomplete("adv", 5);
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().all(|s| s.to_lowercase().contains("adv")));
    }

    #[test]
    fn test_get_related_terms() {
        let related = get_related_terms("adverse event");
        // Adverse Event should have see_also references
        assert!(!related.is_empty());
    }

    #[test]
    fn test_search_by_guideline_series() {
        let e6_terms = search_by_guideline_series("E6");
        assert!(!e6_terms.is_empty());
        assert!(
            e6_terms
                .iter()
                .all(|t| t.source.guideline_id.starts_with("E6"))
        );
    }

    #[test]
    fn test_fuzzy_score() {
        let score = calculate_fuzzy_score("adverse event", "adverse events");
        assert!(score > 0.8);

        let low_score = calculate_fuzzy_score("adverse event", "clinical trial");
        assert!(low_score < 0.5);
    }

    #[test]
    fn test_get_search_stats() {
        let stats = get_search_stats();
        assert!(stats.total_terms > 0);
        assert_eq!(stats.terms_by_category.len(), 4);
    }
}
