//! Fuzzy string matching algorithms.
//!
//! Implements Levenshtein distance and Jaro-Winkler similarity for
//! typo-tolerant medical term matching.
//!
//! # Complexity
//!
//! - Levenshtein: O(n*m) time, O(min(n,m)) space (optimized single-row)
//! - Jaro-Winkler: O(n*m) time, O(n+m) space

use std::cmp::min;

/// Calculate Levenshtein (edit) distance between two strings.
///
/// Uses Wagner-Fischer algorithm with single-row space optimization.
///
/// # Complexity
///
/// - TIME: O(n * m) where n, m are string lengths
/// - SPACE: O(min(n, m)) using single-row optimization
///
/// # Example
///
/// ```
/// use nexcore_vigilance::coding::fuzzy::levenshtein;
///
/// assert_eq!(levenshtein("headache", "headache"), 0);
/// assert_eq!(levenshtein("headache", "headahce"), 2); // transposition
/// assert_eq!(levenshtein("nausea", "nausia"), 1);
/// ```
#[must_use]
pub fn levenshtein(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    let n = a_chars.len();
    let m = b_chars.len();

    // Edge cases
    if n == 0 {
        return m;
    }
    if m == 0 {
        return n;
    }

    // Ensure we use the shorter string for the row to minimize space
    let (shorter, longer, s_len, _l_len) = if n <= m {
        (&a_chars, &b_chars, n, m)
    } else {
        (&b_chars, &a_chars, m, n)
    };

    // Single row optimization: O(min(n,m)) space
    let mut prev_row: Vec<usize> = (0..=s_len).collect();
    let mut curr_row: Vec<usize> = vec![0; s_len + 1];

    for (i, longer_char) in longer.iter().enumerate() {
        curr_row[0] = i + 1;

        for (j, shorter_char) in shorter.iter().enumerate() {
            let cost = usize::from(longer_char != shorter_char);

            curr_row[j + 1] = min(
                min(
                    prev_row[j + 1] + 1, // deletion
                    curr_row[j] + 1,     // insertion
                ),
                prev_row[j] + cost, // substitution
            );
        }

        std::mem::swap(&mut prev_row, &mut curr_row);
    }

    prev_row[s_len]
}

/// Calculate normalized Levenshtein similarity (0.0 to 1.0).
///
/// Returns 1.0 for identical strings, 0.0 for completely different strings.
///
/// # Example
///
/// ```
/// use nexcore_vigilance::coding::fuzzy::levenshtein_similarity;
///
/// assert!((levenshtein_similarity("headache", "headache") - 1.0).abs() < f64::EPSILON);
/// assert!(levenshtein_similarity("headache", "xyz") < 0.5);
/// ```
#[must_use]
pub fn levenshtein_similarity(a: &str, b: &str) -> f64 {
    let max_len = a.chars().count().max(b.chars().count());
    if max_len == 0 {
        return 1.0;
    }
    let distance = levenshtein(a, b);
    #[allow(clippy::cast_precision_loss)]
    let distance_f = distance as f64;
    #[allow(clippy::cast_precision_loss)]
    let max_len_f = max_len as f64;
    1.0 - (distance_f / max_len_f)
}

/// Calculate Jaro similarity between two strings.
///
/// # Complexity
///
/// - TIME: O(n * m)
/// - SPACE: O(n + m)
///
/// # Example
///
/// ```
/// use nexcore_vigilance::coding::fuzzy::jaro;
///
/// let sim = jaro("headache", "headahce");
/// assert!(sim > 0.9); // High similarity for transposition
/// ```
#[must_use]
pub fn jaro(a: &str, b: &str) -> f64 {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a_len == 0 && b_len == 0 {
        return 1.0;
    }
    if a_len == 0 || b_len == 0 {
        return 0.0;
    }

    // Match window
    let match_distance = (a_len.max(b_len) / 2).saturating_sub(1);

    let mut a_matches = vec![false; a_len];
    let mut b_matches = vec![false; b_len];

    let mut matches = 0usize;
    let mut transpositions = 0usize;

    // Find matches
    for (i, a_char) in a_chars.iter().enumerate() {
        let start = i.saturating_sub(match_distance);
        let end = min(i + match_distance + 1, b_len);

        for j in start..end {
            if b_matches[j] || a_char != &b_chars[j] {
                continue;
            }
            a_matches[i] = true;
            b_matches[j] = true;
            matches += 1;
            break;
        }
    }

    if matches == 0 {
        return 0.0;
    }

    // Count transpositions
    let mut k = 0;
    for (i, matched) in a_matches.iter().enumerate() {
        if !matched {
            continue;
        }
        while !b_matches[k] {
            k += 1;
        }
        if a_chars[i] != b_chars[k] {
            transpositions += 1;
        }
        k += 1;
    }

    #[allow(clippy::cast_precision_loss)]
    let matches_f = matches as f64;
    #[allow(clippy::cast_precision_loss)]
    let a_len_f = a_len as f64;
    #[allow(clippy::cast_precision_loss)]
    let b_len_f = b_len as f64;
    #[allow(clippy::cast_precision_loss)]
    let trans_f = (transpositions / 2) as f64;

    (matches_f / a_len_f + matches_f / b_len_f + (matches_f - trans_f) / matches_f) / 3.0
}

/// Calculate Jaro-Winkler similarity between two strings.
///
/// Gives more weight to strings that match from the beginning.
/// Returns value between 0.0 (no similarity) and 1.0 (identical).
///
/// # Parameters
///
/// - `a`, `b`: Strings to compare
/// - `prefix_scale`: Scaling factor for common prefix bonus (default 0.1, max 0.25)
///
/// # Example
///
/// ```
/// use nexcore_vigilance::coding::fuzzy::jaro_winkler;
///
/// let sim = jaro_winkler("headache", "headahce", 0.1);
/// assert!(sim > 0.95); // Very high similarity
/// ```
#[must_use]
pub fn jaro_winkler(a: &str, b: &str, prefix_scale: f64) -> f64 {
    let jaro_sim = jaro(a, b);

    // Common prefix (max 4 characters)
    let a_chars: Vec<char> = a.chars().take(4).collect();
    let b_chars: Vec<char> = b.chars().take(4).collect();

    let prefix_len = a_chars
        .iter()
        .zip(b_chars.iter())
        .take_while(|(a, b)| a == b)
        .count();

    // Winkler modification: boost for common prefix
    // prefix_scale should be <= 0.25 to keep result <= 1.0
    let scale = prefix_scale.min(0.25);

    #[allow(clippy::cast_precision_loss)]
    let prefix_len_f = prefix_len as f64;

    // jaro_sim + (prefix_len * scale * (1.0 - jaro_sim))
    (prefix_len_f * scale).mul_add(1.0 - jaro_sim, jaro_sim)
}

/// BK-tree node for efficient fuzzy search.
///
/// BK-trees partition strings by edit distance, enabling
/// O(n^(1-1/d)) average-case search complexity where d is
/// the maximum allowed distance.
#[derive(Debug, Clone)]
pub struct BkTree {
    root: Option<BkNode>,
    size: usize,
}

#[derive(Debug, Clone)]
struct BkNode {
    term: String,
    data: u32,                    // Associated data (e.g., MedDRA code)
    children: Vec<(usize, Self)>, // (distance, child)
}

impl BkTree {
    /// Create a new empty BK-tree.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            root: None,
            size: 0,
        }
    }

    /// Insert a term with associated data into the tree.
    ///
    /// # Complexity
    ///
    /// - TIME: O(h) where h is tree height (typically O(log n))
    /// - SPACE: O(1)
    ///
    /// # Panics
    ///
    /// Panics if tree state is inconsistent (root is None but size > 0).
    pub fn insert(&mut self, term: impl Into<String>, data: u32) {
        let term = term.into();

        if self.root.is_none() {
            self.root = Some(BkNode {
                term,
                data,
                children: Vec::new(),
            });
            self.size = 1;
            return;
        }

        #[allow(clippy::expect_used)]
        let mut current = self.root.as_mut().expect("root is Some");
        loop {
            let dist = levenshtein(&current.term, &term);
            if dist == 0 {
                // Duplicate term, update data
                current.data = data;
                return;
            }

            if let Some(pos) = current.children.iter().position(|(d, _)| *d == dist) {
                current = &mut current.children[pos].1;
            } else {
                current.children.push((
                    dist,
                    BkNode {
                        term,
                        data,
                        children: Vec::new(),
                    },
                ));
                self.size += 1;
                return;
            }
        }
    }

    /// Find all terms within `max_distance` of the query.
    ///
    /// # Complexity
    ///
    /// - TIME: O(n^(1-1/d)) average case, O(n) worst case
    /// - SPACE: O(k) where k is number of results
    ///
    /// # Example
    ///
    /// ```
    /// use nexcore_vigilance::coding::fuzzy::BkTree;
    ///
    /// let mut tree = BkTree::new();
    /// tree.insert("headache", 10000001);
    /// tree.insert("headahce", 10000002); // typo
    /// tree.insert("nausea", 10000003);
    ///
    /// let results = tree.search("headache", 2);
    /// assert!(results.iter().any(|(_, code, _)| *code == 10000001));
    /// assert!(results.iter().any(|(_, code, _)| *code == 10000002));
    /// ```
    #[must_use]
    pub fn search(&self, query: &str, max_distance: usize) -> Vec<(String, u32, usize)> {
        let mut results = Vec::new();

        if let Some(root) = &self.root {
            Self::search_recursive(root, query, max_distance, &mut results);
        }

        // Sort by distance (closest first)
        results.sort_by_key(|(_, _, dist)| *dist);
        results
    }

    fn search_recursive(
        node: &BkNode,
        query: &str,
        max_distance: usize,
        results: &mut Vec<(String, u32, usize)>,
    ) {
        let dist = levenshtein(&node.term, query);

        if dist <= max_distance {
            results.push((node.term.clone(), node.data, dist));
        }

        // Only search children within the distance range [dist - max_distance, dist + max_distance]
        let min_dist = dist.saturating_sub(max_distance);
        let max_dist = dist + max_distance;

        for (child_dist, child) in &node.children {
            if *child_dist >= min_dist && *child_dist <= max_dist {
                Self::search_recursive(child, query, max_distance, results);
            }
        }
    }

    /// Return the number of terms in the tree.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.size
    }

    /// Check if the tree is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.size == 0
    }
}

impl Default for BkTree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_identical() {
        assert_eq!(levenshtein("headache", "headache"), 0);
        assert_eq!(levenshtein("", ""), 0);
    }

    #[test]
    fn test_levenshtein_empty() {
        assert_eq!(levenshtein("", "abc"), 3);
        assert_eq!(levenshtein("abc", ""), 3);
    }

    #[test]
    fn test_levenshtein_single_edit() {
        assert_eq!(levenshtein("cat", "bat"), 1); // substitution
        assert_eq!(levenshtein("cat", "cats"), 1); // insertion
        assert_eq!(levenshtein("cats", "cat"), 1); // deletion
    }

    #[test]
    fn test_levenshtein_medical_terms() {
        assert_eq!(levenshtein("nausea", "nausia"), 1);
        assert_eq!(levenshtein("headache", "headahce"), 2); // transposition
        assert_eq!(levenshtein("rhabdomyolysis", "rhabdomyolisis"), 1);
    }

    #[test]
    fn test_jaro_identical() {
        let sim = jaro("headache", "headache");
        assert!((sim - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_jaro_transposition() {
        let sim = jaro("headache", "headahce");
        assert!(sim > 0.9);
    }

    #[test]
    fn test_jaro_winkler_prefix_boost() {
        let jaro_sim = jaro("prefix_abc", "prefix_xyz");
        let jw_sim = jaro_winkler("prefix_abc", "prefix_xyz", 0.1);
        // Jaro-Winkler should be higher due to common prefix
        assert!(jw_sim >= jaro_sim);
    }

    #[test]
    fn test_bktree_insert_search() {
        let mut tree = BkTree::new();
        tree.insert("headache", 10000001);
        tree.insert("nausea", 10000002);
        tree.insert("dizziness", 10000003);

        assert_eq!(tree.len(), 3);

        let results = tree.search("headache", 0);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1, 10000001);
    }

    #[test]
    fn test_bktree_fuzzy_search() {
        let mut tree = BkTree::new();
        tree.insert("headache", 1);
        tree.insert("headahce", 2); // typo
        tree.insert("nausea", 3);

        let results = tree.search("headache", 2);
        assert_eq!(results.len(), 2);

        // Should find exact match first
        assert_eq!(results[0].0, "headache");
        assert_eq!(results[0].2, 0); // distance 0
    }

    #[test]
    fn test_levenshtein_similarity() {
        assert!((levenshtein_similarity("abc", "abc") - 1.0).abs() < f64::EPSILON);
        assert!(levenshtein_similarity("abc", "xyz") < 0.5);
        assert!(levenshtein_similarity("abc", "ab") > 0.5);
    }
}
