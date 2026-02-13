//! Term classifier for vocabulary tier assignment.
//!
//! Classifies terms into the Three Tiers framework using:
//! - Word list membership (Tier 1/2 PHF sets)
//! - Domain glossary lookup (Tier 3)
//! - Morphological indicators (academic suffixes)

use super::domain::{VocabDomain, detect_domain};
use super::tier::{TIER_1_WORDS, TIER_2_WORDS, VocabTier};

/// Classification result for a term.
#[derive(Debug, Clone, PartialEq)]
pub struct Classification {
    /// The classified term.
    pub term: String,
    /// Assigned vocabulary tier.
    pub tier: VocabTier,
    /// Detected domain (if Tier 3).
    pub domain: Option<VocabDomain>,
    /// Confidence score (0.0 - 1.0).
    pub confidence: f32,
    /// Reason for classification.
    pub reason: ClassifyReason,
}

/// Reason for tier assignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassifyReason {
    /// Found in Tier 1 word list.
    Tier1Membership,
    /// Found in Tier 2 word list.
    Tier2Membership,
    /// Found in domain glossary.
    DomainGlossary,
    /// Academic suffix detected.
    AcademicIndicator,
    /// Fallback classification.
    Default,
}

impl std::fmt::Display for ClassifyReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Tier1Membership => "Tier 1 word list",
            Self::Tier2Membership => "Tier 2 word list",
            Self::DomainGlossary => "domain glossary",
            Self::AcademicIndicator => "academic indicator",
            Self::Default => "default",
        };
        write!(f, "{s}")
    }
}

/// Classify a term into a vocabulary tier. O(1)
#[must_use]
pub fn classify(term: &str) -> Classification {
    let lower = term.to_lowercase();

    // Check Tier 1 first (most common words)
    if TIER_1_WORDS.contains(lower.as_str()) {
        return Classification {
            term: term.to_string(),
            tier: VocabTier::Basic,
            domain: None,
            confidence: 1.0,
            reason: ClassifyReason::Tier1Membership,
        };
    }

    // Check Tier 2 (academic/cross-domain)
    if TIER_2_WORDS.contains(lower.as_str()) {
        return Classification {
            term: term.to_string(),
            tier: VocabTier::CrossDomain,
            domain: None,
            confidence: 1.0,
            reason: ClassifyReason::Tier2Membership,
        };
    }

    // Check domain glossaries (Tier 3)
    if let Some(domain) = detect_domain(&lower) {
        return Classification {
            term: term.to_string(),
            tier: VocabTier::DomainSpecific,
            domain: Some(domain),
            confidence: 0.95,
            reason: ClassifyReason::DomainGlossary,
        };
    }

    // Check for academic indicators (suffix patterns)
    if has_academic_suffix(&lower) {
        return Classification {
            term: term.to_string(),
            tier: VocabTier::CrossDomain,
            domain: None,
            confidence: 0.7,
            reason: ClassifyReason::AcademicIndicator,
        };
    }

    // Default to Basic
    Classification {
        term: term.to_string(),
        tier: VocabTier::Basic,
        domain: None,
        confidence: 0.5,
        reason: ClassifyReason::Default,
    }
}

/// Classify multiple terms. O(n)
#[must_use]
pub fn classify_batch(terms: &[&str]) -> Vec<Classification> {
    terms.iter().map(|t| classify(t)).collect()
}

/// Check if word has academic suffix patterns. O(1)
fn has_academic_suffix(word: &str) -> bool {
    const ACADEMIC_SUFFIXES: &[&str] = &[
        "tion", "sion", "ment", "ness", "ity", "ance", "ence", "ive", "ous", "ful", "less", "able",
        "ible", "ical",
    ];

    ACADEMIC_SUFFIXES.iter().any(|s| word.ends_with(s))
}

/// Feature vector for ML-based classification (future use).
#[derive(Debug, Clone)]
pub struct TermFeatures {
    /// Word length.
    pub length: usize,
    /// Estimated syllable count.
    pub syllables: usize,
    /// Has academic suffix.
    pub has_academic_suffix: bool,
    /// Contains hyphen.
    pub is_compound: bool,
    /// Contains digits.
    pub has_digits: bool,
    /// All uppercase.
    pub is_acronym: bool,
}

impl TermFeatures {
    /// Extract features from a term. O(n)
    #[must_use]
    pub fn extract(term: &str) -> Self {
        let lower = term.to_lowercase();
        Self {
            length: term.len(),
            syllables: estimate_syllables(&lower),
            has_academic_suffix: has_academic_suffix(&lower),
            is_compound: term.contains('-'),
            has_digits: term.chars().any(|c| c.is_ascii_digit()),
            is_acronym: term.len() >= 2
                && term.len() <= 5
                && term.chars().all(|c| c.is_ascii_uppercase()),
        }
    }
}

/// Estimate syllable count. O(n)
fn estimate_syllables(word: &str) -> usize {
    let vowels = ['a', 'e', 'i', 'o', 'u', 'y'];
    let mut count = 0;
    let mut prev_vowel = false;

    for c in word.chars() {
        let is_vowel = vowels.contains(&c);
        if is_vowel && !prev_vowel {
            count += 1;
        }
        prev_vowel = is_vowel;
    }

    // Silent 'e' adjustment
    if word.ends_with('e') && count > 1 {
        count -= 1;
    }

    count.max(1) // At least one syllable
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier1_classification() {
        let result = classify("the");
        assert_eq!(result.tier, VocabTier::Basic);
        assert_eq!(result.reason, ClassifyReason::Tier1Membership);
        assert!((result.confidence - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_tier2_classification() {
        let result = classify("validate");
        assert_eq!(result.tier, VocabTier::CrossDomain);
        assert_eq!(result.reason, ClassifyReason::Tier2Membership);
    }

    #[test]
    fn test_tier3_classification() {
        let result = classify("faers");
        assert_eq!(result.tier, VocabTier::DomainSpecific);
        assert!(result.domain.is_some());
        assert_eq!(result.reason, ClassifyReason::DomainGlossary);
    }

    #[test]
    fn test_academic_indicator() {
        let result = classify("documentation");
        assert_eq!(result.tier, VocabTier::CrossDomain);
        assert_eq!(result.reason, ClassifyReason::AcademicIndicator);
    }

    #[test]
    fn test_batch_classification() {
        let terms = vec!["the", "validate", "faers"];
        let results = classify_batch(&terms);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].tier, VocabTier::Basic);
        assert_eq!(results[1].tier, VocabTier::CrossDomain);
        assert_eq!(results[2].tier, VocabTier::DomainSpecific);
    }

    #[test]
    fn test_syllable_estimation() {
        assert_eq!(estimate_syllables("cat"), 1);
        assert_eq!(estimate_syllables("validate"), 3);
        assert_eq!(estimate_syllables("pharmacovigilance"), 6);
    }

    #[test]
    fn test_feature_extraction() {
        let features = TermFeatures::extract("validation");
        assert_eq!(features.length, 10);
        assert!(features.has_academic_suffix);
        assert!(!features.is_compound);
        assert!(!features.has_digits);
    }
}
