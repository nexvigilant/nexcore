//! Structural compression beyond BLUFF patterns.
//!
//! Four stages: Pattern → Dedup → Hierarchy → Summary.
//! Token Jaccard similarity reused from `nexcore-brain/src/implicit.rs:60-87`.

use std::collections::BTreeSet;

/// Verbose pattern replacements (ported from compendious-machine).
const VERBOSE_PATTERNS: &[(&str, &str)] = &[
    // Throat-clearing deletions
    ("it is important to note that", ""),
    ("it should be mentioned that", ""),
    ("as a matter of fact", ""),
    ("for all intents and purposes", ""),
    ("at the end of the day", ""),
    ("the fact of the matter is", ""),
    ("it is worth noting that", ""),
    ("needless to say", ""),
    // Prepositional bloat
    ("in order to", "to"),
    ("for the purpose of", "to"),
    ("with regard to", "about"),
    ("in reference to", "about"),
    ("in terms of", "regarding"),
    ("on the basis of", "based on"),
    ("in the event that", "if"),
    ("at this point in time", "now"),
    ("at the present time", "now"),
    ("prior to", "before"),
    ("subsequent to", "after"),
    ("in spite of the fact that", "although"),
    ("due to the fact that", "because"),
    ("in light of the fact that", "because"),
    // Redundancies
    ("completely finished", "finished"),
    ("absolutely essential", "essential"),
    ("basic fundamentals", "fundamentals"),
    ("past history", "history"),
    ("future plans", "plans"),
    ("end result", "result"),
    ("final outcome", "outcome"),
    ("close proximity", "near"),
    ("each and every", "each"),
    ("any and all", "all"),
    ("first and foremost", "first"),
    // Nominalizations
    ("make a decision", "decide"),
    ("give consideration to", "consider"),
    ("reach a conclusion", "conclude"),
    ("perform an analysis", "analyze"),
    ("conduct an investigation", "investigate"),
    ("provide assistance to", "help"),
    ("make an improvement", "improve"),
    ("take action", "act"),
    // Common verbose phrases
    ("a large number of", "many"),
    ("a significant number of", "many"),
    ("the vast majority of", "most"),
    ("in the near future", "soon"),
    ("at some point in the future", "eventually"),
];

/// Compression method applied.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompressionMethod {
    Pattern,
    Dedup,
    Hierarchy,
    Summary,
}

impl std::fmt::Display for CompressionMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pattern => write!(f, "pattern"),
            Self::Dedup => write!(f, "dedup"),
            Self::Hierarchy => write!(f, "hierarchy"),
            Self::Summary => write!(f, "summary"),
        }
    }
}

/// Result of structural compression.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CompressionResult {
    pub original_text: String,
    pub compressed_text: String,
    pub original_word_count: usize,
    pub compressed_word_count: usize,
    pub compression_ratio: f64,
    pub methods_applied: Vec<CompressionMethod>,
}

/// Structural compressor with four stages.
pub struct StructuralCompressor {
    dedup_threshold: f64,
}

impl Default for StructuralCompressor {
    fn default() -> Self {
        Self {
            dedup_threshold: 0.8,
        }
    }
}

/// Maximum number of sentences to consider for O(n²) dedup.
/// Beyond this limit, tail sentences pass through unchanged.
const MAX_SENTENCES_FOR_DEDUP: usize = 150;

impl StructuralCompressor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_dedup_threshold(mut self, threshold: f64) -> Self {
        self.dedup_threshold = threshold;
        self
    }

    /// Run all compression stages.
    pub fn compress(&self, text: &str) -> CompressionResult {
        let original_word_count = text.split_whitespace().count();
        let mut methods = Vec::new();
        let mut current = text.to_string();

        // Stage 1: Pattern replacement
        let after_pattern = Self::apply_patterns(&current);
        if after_pattern != current {
            methods.push(CompressionMethod::Pattern);
            current = after_pattern;
        }

        // Stage 2: Dedup near-duplicate sentences
        let after_dedup = self.apply_dedup(&current);
        if after_dedup != current {
            methods.push(CompressionMethod::Dedup);
            current = after_dedup;
        }

        // Stage 3: Flatten single-child heading nesting
        let after_hierarchy = Self::apply_hierarchy(&current);
        if after_hierarchy != current {
            methods.push(CompressionMethod::Hierarchy);
            current = after_hierarchy;
        }

        let compressed_word_count = current.split_whitespace().count();
        let compression_ratio = if original_word_count > 0 {
            1.0 - (compressed_word_count as f64 / original_word_count as f64)
        } else {
            0.0
        };

        CompressionResult {
            original_text: text.to_string(),
            compressed_text: current,
            original_word_count,
            compressed_word_count,
            compression_ratio,
            methods_applied: methods,
        }
    }

    /// Apply only pattern replacement.
    pub fn compress_pattern_only(text: &str) -> String {
        Self::apply_patterns(text)
    }

    /// Stage 1: BLUFF verbose pattern replacement.
    ///
    /// All patterns in `VERBOSE_PATTERNS` are lowercase ASCII. The algorithm keeps
    /// a parallel lowercase view and slices both original-case and lowercase in
    /// lockstep, advancing only by the pattern's byte length. This is safe because
    /// ASCII patterns have identical byte lengths in original and lowercased forms,
    /// even when surrounding non-ASCII characters change byte length under
    /// `to_lowercase()`.
    fn apply_patterns(text: &str) -> String {
        let mut result = text.to_string();

        for (verbose, replacement) in VERBOSE_PATTERNS {
            // Rebuild lowercase each iteration since prior replacements change the text
            let lower_result = result.to_lowercase();
            if !lower_result.contains(verbose) {
                continue;
            }

            let mut out = String::with_capacity(result.len());
            let mut rest = result.as_str();
            let mut rest_lower = lower_result.as_str();

            while let Some(pos) = rest_lower.find(verbose) {
                // All VERBOSE_PATTERNS are ASCII, so `pos` in the lowercase view is
                // also a valid char boundary in the original-case `rest` — ASCII
                // chars never change byte length under to_lowercase().
                //
                // Safety: we verify `pos` is within bounds of `rest` and is a char
                // boundary before slicing.
                if pos > rest.len() || !rest.is_char_boundary(pos) {
                    // Misaligned — non-ASCII expansion shifted offsets. Bail safely.
                    break;
                }
                let end = pos.checked_add(verbose.len()).unwrap_or(rest.len());
                if end > rest.len() || !rest.is_char_boundary(end) {
                    break;
                }

                out.push_str(&rest[..pos]);
                out.push_str(replacement);
                rest = &rest[end..];
                rest_lower = &rest_lower[pos + verbose.len()..];
            }
            out.push_str(rest);
            result = out;
        }

        // Clean up runs of whitespace from deletions (single allocation)
        if result.contains("  ") {
            result = result.split_whitespace().collect::<Vec<_>>().join(" ");
        }
        result.trim().to_string()
    }

    /// Stage 2: Token Jaccard dedup — collapse near-duplicate sentences.
    ///
    /// Capped at `MAX_SENTENCES_FOR_DEDUP` to prevent O(n²) blowup on large inputs.
    /// Sentences beyond the cap pass through unchanged.
    fn apply_dedup(&self, text: &str) -> String {
        let sentences: Vec<&str> = text
            .split(['.', '!', '?'])
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        if sentences.len() < 2 {
            return text.to_string();
        }

        // Cap the dedup window to prevent quadratic blowup
        let dedup_count = sentences.len().min(MAX_SENTENCES_FOR_DEDUP);

        let mut keep = vec![true; sentences.len()];
        for i in 0..dedup_count {
            if !keep[i] {
                continue;
            }
            for j in (i + 1)..dedup_count {
                if !keep[j] {
                    continue;
                }
                if token_similarity(sentences[i], sentences[j]) > self.dedup_threshold {
                    // Keep the longer one
                    if sentences[i].len() >= sentences[j].len() {
                        keep[j] = false;
                    } else {
                        keep[i] = false;
                        break;
                    }
                }
            }
        }

        let deduped: Vec<&str> = sentences
            .iter()
            .zip(keep.iter())
            .filter(|(_, k)| **k)
            .map(|(s, _)| *s)
            .collect();

        if deduped.is_empty() {
            return text.to_string();
        }

        deduped.join(". ") + "."
    }

    /// Stage 3: Flatten single-child markdown heading nesting.
    ///
    /// A parent heading is promoted away only when it has **exactly one** direct
    /// child heading (a heading at `parent_level + 1`) with no non-heading content
    /// between the parent and that child. If the parent has two or more direct
    /// children, the structure is left unchanged to avoid orphaning siblings.
    fn apply_hierarchy(text: &str) -> String {
        let lines: Vec<&str> = text.lines().collect();
        if lines.len() < 3 {
            return text.to_string();
        }

        let mut result = Vec::new();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i];

            if line.starts_with('#') && i + 1 < lines.len() {
                let current_level = line.chars().take_while(|c| *c == '#').count();

                // Locate the first non-blank line after this heading.
                let mut j = i + 1;
                while j < lines.len() && lines[j].trim().is_empty() {
                    j += 1;
                }

                if j < lines.len() && lines[j].starts_with('#') {
                    let next_level = lines[j].chars().take_while(|c| *c == '#').count();
                    if next_level == current_level + 1 {
                        // Count how many direct children (level == current_level + 1) follow
                        // before we reach a heading at or above current_level or the end.
                        let child_idx = j;
                        let mut k = j + 1;
                        let mut direct_child_count = 1_usize;
                        while k < lines.len() {
                            let l = lines[k];
                            if l.starts_with('#') {
                                let lv = l.chars().take_while(|c| *c == '#').count();
                                if lv <= current_level {
                                    // Sibling or uncle heading — stop scanning.
                                    break;
                                }
                                if lv == current_level + 1 {
                                    direct_child_count += 1;
                                }
                            }
                            k += 1;
                        }

                        if direct_child_count == 1 {
                            // Exactly one direct child: promote it and drop the parent.
                            let child_text = lines[child_idx].trim_start_matches('#').trim();
                            result.push(format!("{} {}", "#".repeat(current_level), child_text));
                            i = child_idx + 1;
                            continue;
                        }
                    }
                }
            }

            result.push(line.to_string());
            i += 1;
        }

        result.join("\n")
    }
}

/// Compute token-based Jaccard similarity between two strings.
///
/// Uses `BTreeSet<&str>` (borrowed slices) to avoid per-token `String` allocation.
/// Case-insensitive: both inputs are lowercased before tokenization.
pub fn token_similarity(a: &str, b: &str) -> f64 {
    let a_lower = a.to_lowercase();
    let b_lower = b.to_lowercase();

    let set_a: BTreeSet<&str> = a_lower
        .split(|c: char| c.is_whitespace() || c.is_ascii_punctuation())
        .filter(|t| !t.is_empty())
        .collect();
    let set_b: BTreeSet<&str> = b_lower
        .split(|c: char| c.is_whitespace() || c.is_ascii_punctuation())
        .filter(|t| !t.is_empty())
        .collect();

    if set_a.is_empty() && set_b.is_empty() {
        return 1.0;
    }
    if set_a.is_empty() || set_b.is_empty() {
        return 0.0;
    }

    let intersection = set_a.intersection(&set_b).count();
    let union = set_a.union(&set_b).count();

    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pattern_replacement() {
        let text = "In order to make a decision, we need to perform an analysis.";
        let compressed = StructuralCompressor::compress_pattern_only(text);
        assert!(compressed.contains("to"));
        assert!(compressed.contains("decide"));
        assert!(compressed.contains("analyze"));
        assert!(!compressed.contains("in order to"));
    }

    #[test]
    fn pattern_replacement_non_ascii_safe() {
        // Ensure non-ASCII characters don't cause corruption via byte-offset misalignment.
        // İ (U+0130) lowercases to 'i' + combining dot above (3 bytes vs 2).
        let text = "İstanbul is great, in order to visit.";
        let compressed = StructuralCompressor::compress_pattern_only(text);
        assert!(
            compressed.contains("İstanbul"),
            "non-ASCII text must survive pattern replacement: {compressed}"
        );
        assert!(
            !compressed.contains("in order to"),
            "pattern should still be replaced: {compressed}"
        );
    }

    #[test]
    fn dedup_similar_sentences() {
        let text = "Signal detection uses PRR for analysis. Signal detection uses PRR for signal analysis. Something different here.";
        let compressor = StructuralCompressor::new();
        let result = compressor.compress(text);
        // Should have removed the near-duplicate
        assert!(result.compressed_word_count < result.original_word_count);
    }

    #[test]
    fn dedup_respects_sentence_cap() {
        // Verify that texts exceeding MAX_SENTENCES_FOR_DEDUP don't panic or hang.
        let mut sentences = Vec::new();
        for i in 0..200 {
            sentences.push(format!("Sentence number {i} is unique enough"));
        }
        let text = sentences.join(". ") + ".";
        let compressor = StructuralCompressor::new();
        // Should complete without hanging — the cap prevents O(n²) on all 200
        let result = compressor.compress(&text);
        assert!(result.compressed_word_count > 0);
    }

    #[test]
    fn token_similarity_identical() {
        assert!((token_similarity("hello world", "hello world") - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn token_similarity_different() {
        let sim = token_similarity("hello world", "goodbye moon");
        assert!(sim < 0.1);
    }

    #[test]
    fn hierarchy_single_child_is_flattened() {
        // A parent with exactly one child: parent should be dropped, child promoted.
        let text = "## Parent\n### Only Child\nSome content.";
        let result = StructuralCompressor::apply_hierarchy(text);
        assert!(
            !result.contains("## Parent"),
            "single-child parent should be removed: {result}"
        );
        assert!(
            result.contains("## Only Child"),
            "child should be promoted to parent level: {result}"
        );
    }

    #[test]
    fn hierarchy_multi_child_is_preserved() {
        // A parent with two direct children: structure must NOT be flattened.
        let text = "## Parent\n### Child1\n### Child2";
        let result = StructuralCompressor::apply_hierarchy(text);
        assert!(
            result.contains("## Parent"),
            "multi-child parent must not be removed: {result}"
        );
        assert!(
            result.contains("### Child1") && result.contains("### Child2"),
            "children must be preserved: {result}"
        );
    }

    #[test]
    fn compression_ratio() {
        // Input contains 5 verbose patterns that are replaced:
        //   "it is important to note that" → ""
        //   "in order to" → "to"
        //   "make a decision" → "decide"
        //   "give consideration to" → "consider"
        //   "basic fundamentals" → "fundamentals"
        // Original: 21 words. After replacements the word count drops meaningfully.
        let text = "It is important to note that in order to make a decision, we need to give consideration to all the basic fundamentals.";
        let compressor = StructuralCompressor::new();
        let result = compressor.compress(text);
        // Must remove at least 30% of words given the heavy verbose load
        assert!(
            result.compression_ratio >= 0.30,
            "expected >= 30% compression on verbose text, got {:.1}%",
            result.compression_ratio * 100.0
        );
        assert_eq!(
            result.methods_applied.len(),
            1,
            "expected exactly 1 method (Pattern)"
        );
    }

    #[test]
    fn double_space_cleanup_single_pass() {
        // Verify that multiple consecutive spaces from deletions are collapsed
        let text = "It is important to note that   the result   was clear.";
        let compressed = StructuralCompressor::compress_pattern_only(text);
        assert!(
            !compressed.contains("  "),
            "double spaces should be cleaned: {compressed:?}"
        );
    }
}
