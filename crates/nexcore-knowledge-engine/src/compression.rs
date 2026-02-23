//! Structural compression beyond BLUFF patterns.
//!
//! Four stages: Pattern → Dedup → Hierarchy → Summary.
//! Token Jaccard similarity reused from `nexcore-brain/src/implicit.rs:60-87`.

use std::collections::HashSet;

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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
    fn apply_patterns(text: &str) -> String {
        let mut result = text.to_string();
        let lower = text.to_lowercase();

        for (verbose, replacement) in VERBOSE_PATTERNS {
            if lower.contains(verbose) {
                // Case-insensitive replacement
                let mut search_pos = 0;
                let mut new_result = String::new();
                let lower_result = result.to_lowercase();

                while let Some(pos) = lower_result[search_pos..].find(verbose) {
                    let abs_pos = search_pos + pos;
                    new_result.push_str(&result[search_pos..abs_pos]);
                    new_result.push_str(replacement);
                    search_pos = abs_pos + verbose.len();
                }
                new_result.push_str(&result[search_pos..]);
                result = new_result;
            }
        }

        // Clean up double spaces from deletions
        while result.contains("  ") {
            result = result.replace("  ", " ");
        }
        result.trim().to_string()
    }

    /// Stage 2: Token Jaccard dedup — collapse near-duplicate sentences.
    fn apply_dedup(&self, text: &str) -> String {
        let sentences: Vec<&str> = text
            .split(['.', '!', '?'])
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        if sentences.len() < 2 {
            return text.to_string();
        }

        let mut keep = vec![true; sentences.len()];
        for i in 0..sentences.len() {
            if !keep[i] {
                continue;
            }
            for j in (i + 1)..sentences.len() {
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
    fn apply_hierarchy(text: &str) -> String {
        let lines: Vec<&str> = text.lines().collect();
        if lines.len() < 3 {
            return text.to_string();
        }

        let mut result = Vec::new();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i];

            // Check if this is a heading with only one child heading (no content between)
            if line.starts_with('#') && i + 1 < lines.len() {
                let current_level = line.chars().take_while(|c| *c == '#').count();

                // Look ahead: skip empty lines, find next heading
                let mut j = i + 1;
                while j < lines.len() && lines[j].trim().is_empty() {
                    j += 1;
                }

                if j < lines.len() && lines[j].starts_with('#') {
                    let next_level = lines[j].chars().take_while(|c| *c == '#').count();
                    if next_level == current_level + 1 {
                        // Skip the parent heading, promote child
                        let child_text = lines[j].trim_start_matches('#').trim();
                        result.push(format!("{} {}", "#".repeat(current_level), child_text));
                        i = j + 1;
                        continue;
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
/// Reused from `nexcore-brain/src/implicit.rs:60-87`.
pub fn token_similarity(a: &str, b: &str) -> f64 {
    let tokenize = |s: &str| -> HashSet<String> {
        s.to_lowercase()
            .split(|c: char| c.is_whitespace() || c.is_ascii_punctuation())
            .filter(|t| !t.is_empty())
            .map(String::from)
            .collect()
    };

    let set_a = tokenize(a);
    let set_b = tokenize(b);

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
    fn dedup_similar_sentences() {
        let text = "Signal detection uses PRR for analysis. Signal detection uses PRR for signal analysis. Something different here.";
        let compressor = StructuralCompressor::new();
        let result = compressor.compress(text);
        // Should have removed the near-duplicate
        assert!(result.compressed_word_count < result.original_word_count);
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
    fn compression_ratio() {
        let text = "It is important to note that in order to make a decision, we need to give consideration to all the basic fundamentals.";
        let compressor = StructuralCompressor::new();
        let result = compressor.compress(text);
        assert!(result.compression_ratio > 0.0);
    }
}
