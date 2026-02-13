//! Lexicon — Word Mining, Refining & Processing via DNA Math.
//!
//! Three-phase pipeline following the biological metaphor:
//!
//! 1. **Mine** — encode words as DNA strands, extract mathematical properties
//!    (Shannon entropy, GC content, nucleotide distribution, codon fingerprint)
//! 2. **Refine** — compare words via edit distance, similarity, LCS
//!    (Levenshtein, normalized similarity, longest common subsequence)
//! 3. **Process** — build vocabulary, compute pairwise matrices, find nearest neighbors
//!    (insert, search, nearest-k, similarity matrix)
//!
//! All algorithms implemented from scratch. Zero external dependencies.

use crate::ops;
use crate::storage;
use crate::types::{Nucleotide, Strand};
use std::fmt;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Raw mined properties of a word encoded as DNA.
///
/// Tier: T2-C (N Quantity + σ Sequence + μ Mapping + κ Comparison)
/// Dominant: N Quantity (word mining extracts numerical properties)
pub struct WordOre {
    /// Original text.
    pub word: String,
    /// DNA encoding via `storage::encode_str`.
    pub strand: Strand,
    /// Shannon entropy (bits) over byte frequencies.
    pub entropy: f64,
    /// Fraction of G/C bases in the strand.
    pub gc_content: f64,
    /// Strand nucleotide count.
    pub length: usize,
    /// Nucleotide frequency counts: [A, T, G, C].
    pub nuc_dist: [usize; 4],
}

/// Refined comparison between two words.
///
/// Tier: T2-P (κ Comparison + N Quantity)
/// Dominant: κ Comparison (measures distance between words)
pub struct Affinity {
    /// Levenshtein edit distance.
    pub distance: usize,
    /// Normalized similarity: `1.0 - distance / max(len_a, len_b)`.
    pub similarity: f64,
    /// Longest common subsequence length.
    pub lcs_length: usize,
    /// DNA Hamming distance (only if strand lengths match).
    pub hamming: Option<usize>,
}

/// Vocabulary index of processed words.
///
/// Tier: T3 (σ + μ + κ + N + ∂ + ∃ + π)
/// Dominant: μ Mapping (maps words to mathematical properties)
pub struct Lexicon {
    entries: Vec<WordOre>,
}

// ---------------------------------------------------------------------------
// Mining Functions
// ---------------------------------------------------------------------------

/// Shannon entropy over byte frequencies: `H = -Σ p(x) log₂(p(x))`.
///
/// Returns 0.0 for empty input.
#[must_use]
pub fn entropy(word: &str) -> f64 {
    if word.is_empty() {
        return 0.0;
    }

    let bytes = word.as_bytes();
    let len = bytes.len() as f64;

    // Count byte frequencies
    let mut freq = [0u32; 256];
    for &b in bytes {
        freq[b as usize] += 1;
    }

    let mut h = 0.0_f64;
    for &count in &freq {
        if count > 0 {
            let p = count as f64 / len;
            h -= p * p.log2();
        }
    }

    h
}

/// Count nucleotide occurrences in a strand: [A, T, G, C].
#[must_use]
pub fn nuc_distribution(strand: &Strand) -> [usize; 4] {
    let mut dist = [0usize; 4];
    for &base in &strand.bases {
        match base {
            Nucleotide::A => dist[0] += 1,
            Nucleotide::T => dist[1] += 1,
            Nucleotide::G => dist[2] += 1,
            Nucleotide::C => dist[3] += 1,
        }
    }
    dist
}

/// Mine a word: encode to DNA, compute entropy, GC content, distribution.
#[must_use]
pub fn mine(word: &str) -> WordOre {
    let strand = storage::encode_str(word);
    let ent = entropy(word);
    let gc = ops::gc_content(&strand);
    let length = strand.len();
    let nuc_dist = nuc_distribution(&strand);

    WordOre {
        word: word.to_string(),
        strand,
        entropy: ent,
        gc_content: gc,
        length,
        nuc_dist,
    }
}

// ---------------------------------------------------------------------------
// Refining Functions (Edit Distance + LCS)
// ---------------------------------------------------------------------------

/// Levenshtein edit distance via classic DP with single-row optimization.
///
/// O(n*m) time, O(min(n,m)) space.
#[must_use]
pub fn levenshtein(a: &str, b: &str) -> usize {
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();

    let n = a_bytes.len();
    let m = b_bytes.len();

    // Optimize: ensure we iterate over the shorter dimension
    if n > m {
        return levenshtein(b, a);
    }

    // n <= m now
    let mut prev: Vec<usize> = (0..=n).collect();
    let mut curr = vec![0usize; n + 1];

    for j in 1..=m {
        curr[0] = j;
        for i in 1..=n {
            let cost = if a_bytes[i - 1] == b_bytes[j - 1] {
                0
            } else {
                1
            };
            curr[i] = (prev[i] + 1) // deletion
                .min(curr[i - 1] + 1) // insertion
                .min(prev[i - 1] + cost); // substitution
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[n]
}

/// Normalized similarity: `1.0 - distance / max(len_a, len_b)`.
///
/// Returns 1.0 for identical strings, 0.0 for completely different.
#[must_use]
pub fn similarity(a: &str, b: &str) -> f64 {
    let max_len = a.len().max(b.len());
    if max_len == 0 {
        return 1.0; // both empty
    }
    let dist = levenshtein(a, b) as f64;
    1.0 - dist / max_len as f64
}

/// Longest common subsequence length via DP.
///
/// O(n*m) time, O(min(n,m)) space.
#[must_use]
pub fn lcs_length(a: &str, b: &str) -> usize {
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();

    let n = a_bytes.len();
    let m = b_bytes.len();

    // Optimize: shorter as columns
    if n > m {
        return lcs_length(b, a);
    }

    // n <= m now
    let mut prev = vec![0usize; n + 1];
    let mut curr = vec![0usize; n + 1];

    for j in 1..=m {
        for i in 1..=n {
            if a_bytes[i - 1] == b_bytes[j - 1] {
                curr[i] = prev[i - 1] + 1;
            } else {
                curr[i] = prev[i].max(curr[i - 1]);
            }
        }
        std::mem::swap(&mut prev, &mut curr);
        // Reset curr for next row
        curr.fill(0);
    }

    prev[n]
}

/// Compare two words: compute all metrics at once.
///
/// Returns an `Affinity` with distance, similarity, LCS, and DNA Hamming
/// (if strand lengths match).
#[must_use]
pub fn compare(a: &str, b: &str) -> Affinity {
    let dist = levenshtein(a, b);
    let sim = similarity(a, b);
    let lcs = lcs_length(a, b);

    let strand_a = storage::encode_str(a);
    let strand_b = storage::encode_str(b);

    let hamming = if strand_a.len() == strand_b.len() {
        ops::hamming_distance(&strand_a, &strand_b).ok()
    } else {
        None
    };

    Affinity {
        distance: dist,
        similarity: sim,
        lcs_length: lcs,
        hamming,
    }
}

// ---------------------------------------------------------------------------
// Processing Functions (Lexicon)
// ---------------------------------------------------------------------------

impl Lexicon {
    /// Create an empty vocabulary.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Mine a word and add to the index. Skips duplicates.
    pub fn insert(&mut self, word: &str) {
        if self.entries.iter().any(|e| e.word == word) {
            return;
        }
        self.entries.push(mine(word));
    }

    /// Number of entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the lexicon is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Lookup a word by exact match.
    #[must_use]
    pub fn get(&self, word: &str) -> Option<&WordOre> {
        self.entries.iter().find(|e| e.word == word)
    }

    /// Find k nearest neighbors by Levenshtein distance.
    ///
    /// Returns entries sorted by ascending distance. If `k > len()`,
    /// returns all entries.
    #[must_use]
    pub fn nearest(&self, word: &str, k: usize) -> Vec<(&WordOre, Affinity)> {
        let mut scored: Vec<(&WordOre, Affinity)> = self
            .entries
            .iter()
            .map(|ore| {
                let aff = compare(word, &ore.word);
                (ore, aff)
            })
            .collect();

        scored.sort_by_key(|(_, aff)| aff.distance);
        scored.truncate(k);
        scored
    }

    /// Pairwise similarity matrix for all entries.
    ///
    /// `matrix[i][j]` = similarity between entry i and entry j.
    #[must_use]
    #[allow(clippy::needless_range_loop)]
    pub fn similarity_matrix(&self) -> Vec<Vec<f64>> {
        let n = self.entries.len();
        let mut matrix = vec![vec![0.0_f64; n]; n];

        // Set diagonal
        for (i, row) in matrix.iter_mut().enumerate().take(n) {
            row[i] = 1.0;
        }

        // Fill upper triangle + mirror to lower (cross-row mutation requires indexing)
        let words: Vec<&str> = self.entries.iter().map(|e| e.word.as_str()).collect();
        for i in 0..n {
            for j in (i + 1)..n {
                let sim = similarity(words[i], words[j]);
                matrix[i][j] = sim;
                matrix[j][i] = sim;
            }
        }

        matrix
    }

    /// Entry with the highest Shannon entropy.
    #[must_use]
    pub fn most_entropic(&self) -> Option<&WordOre> {
        self.entries.iter().max_by(|a, b| {
            a.entropy
                .partial_cmp(&b.entropy)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Entry with the lowest Shannon entropy.
    #[must_use]
    pub fn least_entropic(&self) -> Option<&WordOre> {
        self.entries.iter().min_by(|a, b| {
            a.entropy
                .partial_cmp(&b.entropy)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Iterate over all entries.
    pub fn entries(&self) -> &[WordOre] {
        &self.entries
    }
}

impl Default for Lexicon {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Display Implementations
// ---------------------------------------------------------------------------

impl fmt::Display for WordOre {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\"{}\" -> {} nuc, H={:.2}, GC={:.2}, [A:{},T:{},G:{},C:{}]",
            self.word,
            self.length,
            self.entropy,
            self.gc_content,
            self.nuc_dist[0],
            self.nuc_dist[1],
            self.nuc_dist[2],
            self.nuc_dist[3],
        )
    }
}

impl fmt::Display for Affinity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "d={}, sim={:.2}, lcs={}",
            self.distance, self.similarity, self.lcs_length,
        )?;
        if let Some(ham) = self.hamming {
            write!(f, ", ham={ham}")?;
        }
        Ok(())
    }
}

impl fmt::Display for Lexicon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.entries.is_empty() {
            write!(f, "Lexicon(empty)")
        } else {
            let avg_h: f64 =
                self.entries.iter().map(|e| e.entropy).sum::<f64>() / self.entries.len() as f64;
            write!(
                f,
                "Lexicon({} words, avg H={:.2})",
                self.entries.len(),
                avg_h
            )
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Mining tests
    // -----------------------------------------------------------------------

    #[test]
    fn mine_empty_string() {
        let ore = mine("");
        assert_eq!(ore.word, "");
        assert!((ore.entropy - 0.0).abs() < f64::EPSILON);
        assert_eq!(ore.nuc_dist, nuc_distribution(&ore.strand));
    }

    #[test]
    fn mine_single_char() {
        let ore = mine("A");
        assert_eq!(ore.word, "A");
        assert!(ore.length > 0);
        assert!(ore.gc_content >= 0.0 && ore.gc_content <= 1.0);
    }

    #[test]
    fn mine_word_properties() {
        let ore = mine("hello");
        assert_eq!(ore.word, "hello");
        assert!(ore.length > 0);
        assert!(ore.entropy > 0.0);
        assert!(ore.gc_content >= 0.0 && ore.gc_content <= 1.0);
        let dist_sum: usize = ore.nuc_dist.iter().sum();
        assert_eq!(dist_sum, ore.length);
    }

    #[test]
    fn entropy_uniform_distribution() {
        // "abcd" — 4 unique bytes, each p=0.25 → H = 2.0
        let h = entropy("abcd");
        assert!((h - 2.0).abs() < 0.01);
    }

    #[test]
    fn entropy_single_byte_repeated() {
        // "aaaa" — 1 unique byte, p=1.0 → H = 0.0
        let h = entropy("aaaa");
        assert!((h - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn entropy_aaaa_less_than_abcd() {
        let h_uniform = entropy("abcd");
        let h_mono = entropy("aaaa");
        assert!(h_uniform > h_mono);
    }

    #[test]
    fn nuc_distribution_counts() {
        let strand = Strand::parse("AATGCC");
        assert!(strand.is_ok());
        if let Some(s) = strand.ok() {
            let dist = nuc_distribution(&s);
            assert_eq!(dist[0], 2); // A
            assert_eq!(dist[1], 1); // T
            assert_eq!(dist[2], 1); // G
            assert_eq!(dist[3], 2); // C
        }
    }

    #[test]
    fn gc_content_cross_check() {
        let ore = mine("test");
        let expected_gc = ops::gc_content(&ore.strand);
        assert!((ore.gc_content - expected_gc).abs() < f64::EPSILON);
    }

    // -----------------------------------------------------------------------
    // Refining tests — Levenshtein
    // -----------------------------------------------------------------------

    #[test]
    fn levenshtein_empty_strings() {
        assert_eq!(levenshtein("", ""), 0);
    }

    #[test]
    fn levenshtein_empty_vs_nonempty() {
        assert_eq!(levenshtein("", "abc"), 3);
        assert_eq!(levenshtein("abc", ""), 3);
    }

    #[test]
    fn levenshtein_identical() {
        assert_eq!(levenshtein("kitten", "kitten"), 0);
    }

    #[test]
    fn levenshtein_single_char() {
        assert_eq!(levenshtein("a", "b"), 1);
        assert_eq!(levenshtein("a", "a"), 0);
    }

    #[test]
    fn levenshtein_insertions() {
        assert_eq!(levenshtein("abc", "abcd"), 1);
    }

    #[test]
    fn levenshtein_deletions() {
        assert_eq!(levenshtein("abcd", "abc"), 1);
    }

    #[test]
    fn levenshtein_substitutions() {
        assert_eq!(levenshtein("abc", "axc"), 1);
    }

    #[test]
    fn levenshtein_classic_kitten_sitting() {
        assert_eq!(levenshtein("kitten", "sitting"), 3);
    }

    #[test]
    fn levenshtein_symmetric() {
        let d1 = levenshtein("abc", "xyz");
        let d2 = levenshtein("xyz", "abc");
        assert_eq!(d1, d2);
    }

    // -----------------------------------------------------------------------
    // Refining tests — Similarity
    // -----------------------------------------------------------------------

    #[test]
    fn similarity_identical() {
        let s = similarity("hello", "hello");
        assert!((s - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn similarity_empty_strings() {
        let s = similarity("", "");
        assert!((s - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn similarity_completely_different() {
        // "abc" vs "xyz" — distance=3, max_len=3 → sim=0.0
        let s = similarity("abc", "xyz");
        assert!((s - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn similarity_range() {
        let s = similarity("kitten", "sitting");
        assert!(s >= 0.0 && s <= 1.0);
    }

    // -----------------------------------------------------------------------
    // Refining tests — LCS
    // -----------------------------------------------------------------------

    #[test]
    fn lcs_empty() {
        assert_eq!(lcs_length("", ""), 0);
        assert_eq!(lcs_length("abc", ""), 0);
        assert_eq!(lcs_length("", "abc"), 0);
    }

    #[test]
    fn lcs_identical() {
        assert_eq!(lcs_length("abcde", "abcde"), 5);
    }

    #[test]
    fn lcs_subsequence() {
        assert_eq!(lcs_length("abcde", "ace"), 3);
    }

    #[test]
    fn lcs_no_common() {
        assert_eq!(lcs_length("abc", "xyz"), 0);
    }

    #[test]
    fn lcs_symmetric() {
        let l1 = lcs_length("abc", "aec");
        let l2 = lcs_length("aec", "abc");
        assert_eq!(l1, l2);
    }

    // -----------------------------------------------------------------------
    // Refining tests — Compare (Affinity)
    // -----------------------------------------------------------------------

    #[test]
    fn compare_identical_words() {
        let aff = compare("hello", "hello");
        assert_eq!(aff.distance, 0);
        assert!((aff.similarity - 1.0).abs() < f64::EPSILON);
        assert_eq!(aff.lcs_length, 5);
        // Same word → same strand length → hamming should be Some(0)
        assert_eq!(aff.hamming, Some(0));
    }

    #[test]
    fn compare_different_words() {
        let aff = compare("kitten", "sitting");
        assert_eq!(aff.distance, 3);
        assert!(aff.lcs_length > 0);
    }

    #[test]
    fn compare_dna_hamming_integration() {
        // Same-length words get hamming distance
        let aff = compare("cat", "dog");
        // "cat" and "dog" both have 3 bytes → same strand length (3*4+16=28 each)
        assert!(aff.hamming.is_some());
    }

    #[test]
    fn compare_different_length_no_hamming() {
        let aff = compare("hi", "hello");
        // Different byte lengths → different strand lengths → no hamming
        assert!(aff.hamming.is_none());
    }

    // -----------------------------------------------------------------------
    // Processing tests — Lexicon
    // -----------------------------------------------------------------------

    #[test]
    fn lexicon_new_empty() {
        let lex = Lexicon::new();
        assert!(lex.is_empty());
        assert_eq!(lex.len(), 0);
    }

    #[test]
    fn lexicon_default() {
        let lex = Lexicon::default();
        assert!(lex.is_empty());
    }

    #[test]
    fn lexicon_insert_and_get() {
        let mut lex = Lexicon::new();
        lex.insert("hello");
        assert_eq!(lex.len(), 1);
        let ore = lex.get("hello");
        assert!(ore.is_some());
        if let Some(o) = ore {
            assert_eq!(o.word, "hello");
        }
    }

    #[test]
    fn lexicon_insert_duplicate_skipped() {
        let mut lex = Lexicon::new();
        lex.insert("hello");
        lex.insert("hello");
        assert_eq!(lex.len(), 1);
    }

    #[test]
    fn lexicon_get_missing() {
        let lex = Lexicon::new();
        assert!(lex.get("nope").is_none());
    }

    #[test]
    fn lexicon_nearest_basic() {
        let mut lex = Lexicon::new();
        lex.insert("kitten");
        lex.insert("sitting");
        lex.insert("apple");

        let neighbors = lex.nearest("kitten", 2);
        assert_eq!(neighbors.len(), 2);
        // First neighbor should be "kitten" itself (distance 0)
        assert_eq!(neighbors[0].0.word, "kitten");
        assert_eq!(neighbors[0].1.distance, 0);
    }

    #[test]
    fn lexicon_nearest_k_exceeds_len() {
        let mut lex = Lexicon::new();
        lex.insert("a");
        lex.insert("b");

        let neighbors = lex.nearest("a", 10);
        assert_eq!(neighbors.len(), 2);
    }

    #[test]
    fn lexicon_similarity_matrix_dimensions() {
        let mut lex = Lexicon::new();
        lex.insert("one");
        lex.insert("two");
        lex.insert("three");

        let matrix = lex.similarity_matrix();
        assert_eq!(matrix.len(), 3);
        for row in &matrix {
            assert_eq!(row.len(), 3);
        }
    }

    #[test]
    fn lexicon_similarity_matrix_symmetry() {
        let mut lex = Lexicon::new();
        lex.insert("rust");
        lex.insert("ruby");
        lex.insert("java");

        let matrix = lex.similarity_matrix();
        for i in 0..3 {
            for j in 0..3 {
                assert!(
                    (matrix[i][j] - matrix[j][i]).abs() < f64::EPSILON,
                    "matrix not symmetric at [{i}][{j}]"
                );
            }
        }
    }

    #[test]
    fn lexicon_similarity_matrix_diagonal() {
        let mut lex = Lexicon::new();
        lex.insert("alpha");
        lex.insert("beta");

        let matrix = lex.similarity_matrix();
        for i in 0..2 {
            assert!(
                (matrix[i][i] - 1.0).abs() < f64::EPSILON,
                "diagonal [{i}][{i}] should be 1.0"
            );
        }
    }

    #[test]
    fn lexicon_most_least_entropic() {
        let mut lex = Lexicon::new();
        lex.insert("aaaa"); // low entropy (0.0)
        lex.insert("abcd"); // higher entropy

        let most = lex.most_entropic();
        assert!(most.is_some());
        if let Some(m) = most {
            assert_eq!(m.word, "abcd");
        }

        let least = lex.least_entropic();
        assert!(least.is_some());
        if let Some(l) = least {
            assert_eq!(l.word, "aaaa");
        }
    }

    #[test]
    fn lexicon_most_entropic_empty() {
        let lex = Lexicon::new();
        assert!(lex.most_entropic().is_none());
        assert!(lex.least_entropic().is_none());
    }

    #[test]
    fn lexicon_insert_multiple_and_search() {
        let mut lex = Lexicon::new();
        for word in &["rust", "ruby", "java", "python", "go"] {
            lex.insert(word);
        }
        assert_eq!(lex.len(), 5);

        // nearest to "rust" should include "rust" first
        let neighbors = lex.nearest("rust", 3);
        assert_eq!(neighbors.len(), 3);
        assert_eq!(neighbors[0].0.word, "rust");
    }

    // -----------------------------------------------------------------------
    // Display tests
    // -----------------------------------------------------------------------

    #[test]
    fn display_wordore() {
        let ore = mine("hi");
        let s = format!("{ore}");
        assert!(s.contains("\"hi\""));
        assert!(s.contains("H="));
        assert!(s.contains("GC="));
    }

    #[test]
    fn display_affinity() {
        let aff = compare("cat", "bat");
        let s = format!("{aff}");
        assert!(s.contains("d="));
        assert!(s.contains("sim="));
        assert!(s.contains("lcs="));
    }

    #[test]
    fn display_lexicon_empty() {
        let lex = Lexicon::new();
        let s = format!("{lex}");
        assert!(s.contains("empty"));
    }

    #[test]
    fn display_lexicon_with_entries() {
        let mut lex = Lexicon::new();
        lex.insert("alpha");
        lex.insert("beta");
        let s = format!("{lex}");
        assert!(s.contains("2 words"));
        assert!(s.contains("avg H="));
    }
}
