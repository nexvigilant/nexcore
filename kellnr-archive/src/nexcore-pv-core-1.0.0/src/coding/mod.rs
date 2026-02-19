//! # Medical Coding Module
//!
//! MedDRA hierarchy, SMQs, fuzzy search, ICH glossary, and MESH terminology.

use serde::{Deserialize, Serialize};

pub mod crossref;
pub mod glossary;
pub mod meddra;
pub mod mesh;

// Re-export MedDRA (P3)
pub use meddra::{MedDRAHierarchy, MedDRATerm, SMQ, SMQRegistry};

// Re-export glossary (P3)
pub use glossary::{
    GlossaryEntry, ICHGlossary, OutcomeCode as GlossaryOutcomeCode, SeriousnessCriteria,
};

pub use crate::foundation_compat::algorithms::levenshtein::{fuzzy_search, levenshtein};

/// MedDRA hierarchy level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HierarchyLevel {
    /// Lowest Level Term
    LLT,
    /// Preferred Term
    PT,
    /// High Level Term
    HLT,
    /// High Level Group Term
    HLGT,
    /// System Organ Class
    SOC,
}

impl std::fmt::Display for HierarchyLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LLT => write!(f, "LLT"),
            Self::PT => write!(f, "PT"),
            Self::HLT => write!(f, "HLT"),
            Self::HLGT => write!(f, "HLGT"),
            Self::SOC => write!(f, "SOC"),
        }
    }
}

/// Result of a MedDRA encoding operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodeResult {
    /// Matched term
    pub term: String,
    /// MedDRA code
    pub code: u32,
    /// Hierarchy level
    pub level: HierarchyLevel,
    /// Match score (0-1)
    pub score: f64,
    /// Edit distance
    pub distance: usize,
}

/// Jaro similarity between two strings
#[must_use]
pub fn jaro(a: &str, b: &str) -> f64 {
    if a == b {
        return 1.0;
    }

    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a_len == 0 || b_len == 0 {
        return 0.0;
    }

    let match_distance = (a_len.max(b_len) / 2).saturating_sub(1);
    let mut a_matches = vec![false; a_len];
    let mut b_matches = vec![false; b_len];
    let mut matches = 0.0;
    let mut transpositions = 0.0;

    for i in 0..a_len {
        let start = i.saturating_sub(match_distance);
        let end = (i + match_distance + 1).min(b_len);

        for j in start..end {
            if b_matches[j] || a_chars[i] != b_chars[j] {
                continue;
            }
            a_matches[i] = true;
            b_matches[j] = true;
            matches += 1.0;
            break;
        }
    }

    if matches == 0.0 {
        return 0.0;
    }

    let mut k = 0;
    for i in 0..a_len {
        if !a_matches[i] {
            continue;
        }
        while !b_matches[k] {
            k += 1;
        }
        if a_chars[i] != b_chars[k] {
            transpositions += 1.0;
        }
        k += 1;
    }

    (matches / a_len as f64 + matches / b_len as f64 + (matches - transpositions / 2.0) / matches)
        / 3.0
}

/// Jaro-Winkler similarity
#[must_use]
pub fn jaro_winkler(a: &str, b: &str, prefix_scale: f64) -> f64 {
    let jaro_sim = jaro(a, b);

    // Find common prefix length (up to 4 chars)
    let prefix_len = a
        .chars()
        .zip(b.chars())
        .take(4)
        .take_while(|(ca, cb)| ca == cb)
        .count();

    jaro_sim + (prefix_len as f64 * prefix_scale * (1.0 - jaro_sim))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jaro() {
        assert!((jaro("hello", "hello") - 1.0).abs() < 0.001);
        assert!(jaro("hello", "hallo") > 0.8);
    }

    #[test]
    fn test_jaro_winkler() {
        let sim = jaro_winkler("hello", "hallo", 0.1);
        assert!(sim > jaro("hello", "hallo"));
    }
}
