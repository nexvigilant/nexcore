//! Affix analysis for vocabulary terms.
//!
//! Identifies prefixes and suffixes with their meanings to aid
//! vocabulary learning through morphological awareness.

use phf::phf_map;
use serde::{Deserialize, Serialize};

/// Analysis of word affixes (prefixes and suffixes).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AffixAnalysis {
    /// The analyzed word.
    pub word: String,
    /// Detected prefix (e.g., "re-").
    pub prefix: Option<AffixInfo>,
    /// Detected suffix (e.g., "-tion").
    pub suffix: Option<AffixInfo>,
    /// Inferred root word.
    pub root: Option<String>,
}

/// Information about a detected affix.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AffixInfo {
    /// The affix text.
    pub text: String,
    /// Meaning of the affix.
    pub meaning: String,
}

impl AffixAnalysis {
    /// Analyze a word for affixes. O(n)
    #[must_use]
    pub fn analyze(word: &str) -> Self {
        let lower = word.to_lowercase();
        let prefix = detect_prefix(&lower);
        let suffix = detect_suffix(&lower);
        let root = extract_root(&lower, prefix.as_ref(), suffix.as_ref());

        Self {
            word: word.to_string(),
            prefix,
            suffix,
            root,
        }
    }

    /// Check if any affixes were detected. O(1)
    #[must_use]
    pub fn has_affixes(&self) -> bool {
        self.prefix.is_some() || self.suffix.is_some()
    }
}

// ============================================================================
// PREFIX DEFINITIONS
// ============================================================================

/// Common prefixes with meanings.
pub static PREFIXES: phf::Map<&'static str, &'static str> = phf_map! {
    // Negation
    "un" => "not, opposite of",
    "in" => "not, without",
    "im" => "not, without",
    "il" => "not, without",
    "ir" => "not, without",
    "dis" => "not, opposite of",
    "non" => "not",
    "anti" => "against, opposite",

    // Position/Direction
    "pre" => "before",
    "post" => "after",
    "sub" => "under, below",
    "super" => "above, beyond",
    "trans" => "across, through",
    "inter" => "between, among",
    "intra" => "within",
    "extra" => "beyond, outside",
    "ex" => "out of, former",

    // Quantity
    "mono" => "one, single",
    "uni" => "one",
    "bi" => "two",
    "tri" => "three",
    "multi" => "many",
    "poly" => "many",
    "semi" => "half, partial",

    // Repetition/Change
    "re" => "again, back",
    "de" => "remove, reverse",
    "mis" => "wrongly, badly",
    "over" => "excessive, above",
    "under" => "insufficient, below",

    // Size/Degree
    "micro" => "small",
    "macro" => "large",
    "mega" => "very large",
    "mini" => "small",
    "hyper" => "excessive, over",
    "hypo" => "under, below normal",

    // Technical
    "auto" => "self",
    "co" => "together, with",
    "para" => "beside, alongside",
    "pseudo" => "false, fake",
    "proto" => "first, original",
    "meta" => "about, beyond",
};

// ============================================================================
// SUFFIX DEFINITIONS
// ============================================================================

/// Common suffixes with meanings.
pub static SUFFIXES: phf::Map<&'static str, &'static str> = phf_map! {
    // Noun-forming
    "tion" => "action, state of",
    "sion" => "action, state of",
    "ment" => "action, result of",
    "ness" => "state, quality of",
    "ity" => "state, quality of",
    "ance" => "state, action of",
    "ence" => "state, action of",
    "er" => "one who does",
    "or" => "one who does",
    "ist" => "one who practices",
    "ism" => "belief, practice",
    "ology" => "study of",
    "dom" => "state, realm of",
    "ship" => "state, condition",
    "hood" => "state, condition",

    // Verb-forming
    "ize" => "to make, become",
    "ise" => "to make, become",
    "ify" => "to make, cause",
    "ate" => "to make, act upon",
    "en" => "to make, become",

    // Adjective-forming
    "able" => "capable of being",
    "ible" => "capable of being",
    "ful" => "full of, having",
    "less" => "without",
    "ous" => "having quality of",
    "ive" => "having nature of",
    "al" => "relating to",
    "ical" => "relating to",
    "ic" => "relating to",
    "ary" => "relating to",
    "ory" => "relating to",

    // Adverb-forming
    "ly" => "in manner of",
    "ward" => "in direction of",
    "wise" => "in manner of",
};

// ============================================================================
// DETECTION FUNCTIONS
// ============================================================================

/// Try to get prefix at given length. O(1)
#[inline]
fn try_prefix(word: &str, len: usize) -> Option<AffixInfo> {
    if word.len() <= len {
        return None;
    }
    let candidate = &word[..len];
    PREFIXES.get(candidate).map(|meaning| AffixInfo {
        // ALLOC: Required for owned storage in result struct (max 6 chars)
        text: candidate.to_string(),
        // ALLOC: Required for owned storage - static str to String
        meaning: (*meaning).to_string(),
    })
}

/// Detect prefix in a word. O(1) - constant number of PHF lookups.
fn detect_prefix(word: &str) -> Option<AffixInfo> {
    // Try lengths in descending order (longest match wins)
    try_prefix(word, 6)
        .or_else(|| try_prefix(word, 5))
        .or_else(|| try_prefix(word, 4))
        .or_else(|| try_prefix(word, 3))
        .or_else(|| try_prefix(word, 2))
}

/// Try to get suffix at given length. O(1)
#[inline]
fn try_suffix(word: &str, len: usize) -> Option<AffixInfo> {
    if word.len() <= len {
        return None;
    }
    let start = word.len() - len;
    let candidate = &word[start..];
    SUFFIXES.get(candidate).map(|meaning| AffixInfo {
        // ALLOC: Required for owned storage in result struct (max 5 chars)
        text: candidate.to_string(),
        // ALLOC: Required for owned storage - static str to String
        meaning: (*meaning).to_string(),
    })
}

/// Detect suffix in a word. O(1) - constant number of PHF lookups.
fn detect_suffix(word: &str) -> Option<AffixInfo> {
    // Try lengths in descending order (longest match wins)
    try_suffix(word, 5)
        .or_else(|| try_suffix(word, 4))
        .or_else(|| try_suffix(word, 3))
        .or_else(|| try_suffix(word, 2))
}

/// Extract the root word by removing affixes. O(1)
fn extract_root(
    word: &str,
    prefix: Option<&AffixInfo>,
    suffix: Option<&AffixInfo>,
) -> Option<String> {
    let start = prefix.map_or(0, |p| p.text.len());
    let end = suffix.map_or(word.len(), |s| word.len().saturating_sub(s.text.len()));

    if start < end {
        let root = &word[start..end];
        if root.len() >= 2 {
            // ALLOC: Required for owned storage in result struct
            return Some(root.to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefix_detection() {
        let analysis = AffixAnalysis::analyze("unvalidated");
        assert!(analysis.prefix.is_some());
        if let Some(prefix) = &analysis.prefix {
            assert_eq!(prefix.text, "un");
            assert!(prefix.meaning.contains("not"));
        }
    }

    #[test]
    fn test_suffix_detection() {
        let analysis = AffixAnalysis::analyze("validation");
        assert!(analysis.suffix.is_some());
        if let Some(suffix) = &analysis.suffix {
            assert_eq!(suffix.text, "tion");
            assert!(suffix.meaning.contains("action"));
        }
    }

    #[test]
    fn test_root_extraction() {
        let analysis = AffixAnalysis::analyze("revalidation");
        assert_eq!(analysis.root, Some("valida".to_string()));
    }

    #[test]
    fn test_no_affixes() {
        let analysis = AffixAnalysis::analyze("cat");
        assert!(!analysis.has_affixes());
    }

    #[test]
    fn test_both_affixes() {
        let analysis = AffixAnalysis::analyze("unacceptable");
        assert!(analysis.prefix.is_some());
        assert!(analysis.suffix.is_some());
    }
}
