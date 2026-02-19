//! Vocabulary tier definitions and classification.
//!
//! The Three Tiers framework categorizes vocabulary by utility and context:
//! - **Tier 1**: Basic, high-frequency words (the, is, run, file)
//! - **Tier 2**: High-utility, cross-domain academic words (validate, configure, aggregate)
//! - **Tier 3**: Domain-specific technical terms (FAERS, MedDRA, pharmacovigilance)

use phf::phf_set;
use serde::{Deserialize, Serialize};

/// Vocabulary tier classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum VocabTier {
    /// Tier 1: Basic, high-frequency words.
    /// Universal vocabulary known by most native speakers.
    #[default]
    Basic = 1,

    /// Tier 2: Cross-domain academic/technical words.
    /// High-utility terms that transfer across professional contexts.
    CrossDomain = 2,

    /// Tier 3: Domain-specific terminology.
    /// Specialized vocabulary requiring explicit instruction.
    DomainSpecific = 3,
}

impl VocabTier {
    /// Create a tier from a numeric value.
    #[must_use]
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::Basic),
            2 => Some(Self::CrossDomain),
            3 => Some(Self::DomainSpecific),
            _ => None,
        }
    }

    /// Get the numeric value of this tier.
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    /// Human-readable name for this tier.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Basic => "Basic",
            Self::CrossDomain => "Cross-Domain",
            Self::DomainSpecific => "Domain-Specific",
        }
    }

    /// Description of this tier.
    #[must_use]
    pub const fn description(self) -> &'static str {
        match self {
            Self::Basic => "High-frequency words known by most speakers",
            Self::CrossDomain => "Academic/technical words useful across domains",
            Self::DomainSpecific => "Specialized terminology requiring explicit instruction",
        }
    }
}

impl std::fmt::Display for VocabTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tier {} ({})", self.as_u8(), self.name())
    }
}

// ============================================================================
// TIER 1: Basic/High-Frequency Words
// ============================================================================

/// Top most common English words (Tier 1 candidates).
pub static TIER_1_WORDS: phf::Set<&'static str> = phf_set! {
    // Function words
    "the", "be", "to", "of", "and", "a", "in", "that", "have", "i",
    "it", "for", "not", "on", "with", "he", "as", "you", "do", "at",
    "this", "but", "his", "by", "from", "they", "we", "say", "her", "she",
    "or", "an", "will", "my", "one", "all", "would", "there", "their", "what",
    "so", "up", "out", "if", "about", "who", "get", "which", "go", "me",
    "when", "make", "can", "like", "time", "no", "just", "him", "know", "take",
    "people", "into", "year", "your", "good", "some", "could", "them", "see", "other",
    "than", "then", "now", "look", "only", "come", "its", "over", "think", "also",
    "back", "after", "use", "two", "how", "our", "work", "first", "well", "way",
    "even", "new", "want", "because", "any", "these", "give", "day", "most", "us",
    // Common forms of 'to be'
    "is", "are", "was", "were", "been", "being", "has", "had", "did", "does",
    // Common technical verbs (basic level)
    "file", "data", "run", "check", "set", "put", "add", "show", "find", "name",
    // Adverbs and quantifiers
    "very", "much", "each", "many", "such", "more", "here", "where", "why", "may",
};

// ============================================================================
// TIER 2: Academic/Cross-Domain Words
// ============================================================================

/// Academic Word List - Tier 2 candidates (adapted from Coxhead's AWL).
pub static TIER_2_WORDS: phf::Set<&'static str> = phf_set! {
    // Technical verbs
    "analyze", "analyse", "validate", "configure", "deploy", "implement",
    "synthesize", "aggregate", "normalize", "transform", "process", "generate",
    "integrate", "orchestrate", "provision", "instantiate", "refactor", "optimize",
    "execute", "initialize", "terminate", "authenticate", "authorize", "serialize",
    "deserialize", "encode", "decode", "parse", "render", "compile", "interpret",
    // Architectural nouns
    "framework", "protocol", "module", "component", "service", "layer",
    "endpoint", "interface", "schema", "manifest", "pipeline", "workflow",
    "architecture", "infrastructure", "repository", "registry", "configuration",
    "dependency", "parameter", "attribute", "property", "method", "function",
    // Process nouns
    "verification", "validation", "assessment", "evaluation", "analysis",
    "implementation", "execution", "initialization", "termination", "iteration",
    "migration", "integration", "transformation", "conversion", "compilation",
    // Descriptive adjectives
    "comprehensive", "systematic", "automated", "manual", "incremental",
    "sequential", "parallel", "synchronous", "asynchronous", "recursive",
    "iterative", "modular", "scalable", "robust", "resilient", "optimal",
};

/// Technical naming pattern for Tier 2 detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TechnicalPattern {
    /// ALL CAPS acronym (e.g., FAERS, API)
    Acronym,
    /// camelCase (e.g., getUserData)
    CamelCase,
    /// snake_case (e.g., get_user_data)
    SnakeCase,
    /// kebab-case (e.g., get-user-data)
    KebabCase,
    /// Version number (e.g., v1.0, 2.5.1)
    Version,
}

impl TechnicalPattern {
    /// Check if text matches this pattern.
    #[must_use]
    pub fn matches(&self, text: &str) -> bool {
        match self {
            Self::Acronym => {
                text.len() >= 2 && text.len() <= 5 && text.chars().all(|c| c.is_ascii_uppercase())
            }
            Self::CamelCase => {
                let chars: Vec<char> = text.chars().collect();
                if chars.is_empty() || !chars[0].is_lowercase() {
                    return false;
                }
                chars.iter().any(|c| c.is_uppercase()) && chars.iter().all(|c| c.is_alphanumeric())
            }
            Self::SnakeCase => {
                text.contains('_')
                    && text.chars().all(|c| c.is_lowercase() || c == '_')
                    && !text.starts_with('_')
                    && !text.ends_with('_')
            }
            Self::KebabCase => {
                text.contains('-')
                    && text.chars().all(|c| c.is_lowercase() || c == '-')
                    && !text.starts_with('-')
                    && !text.ends_with('-')
            }
            Self::Version => {
                let s = text.strip_prefix('v').unwrap_or(text);
                s.contains('.') && s.chars().all(|c| c.is_ascii_digit() || c == '.')
            }
        }
    }

    /// Try to detect which pattern a string matches.
    #[must_use]
    pub fn detect(text: &str) -> Option<Self> {
        if Self::Acronym.matches(text) {
            Some(Self::Acronym)
        } else if Self::CamelCase.matches(text) {
            Some(Self::CamelCase)
        } else if Self::SnakeCase.matches(text) {
            Some(Self::SnakeCase)
        } else if Self::KebabCase.matches(text) {
            Some(Self::KebabCase)
        } else if Self::Version.matches(text) {
            Some(Self::Version)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_roundtrip() {
        for tier in [
            VocabTier::Basic,
            VocabTier::CrossDomain,
            VocabTier::DomainSpecific,
        ] {
            let value = tier.as_u8();
            let restored = VocabTier::from_u8(value);
            assert_eq!(restored, Some(tier));
        }
    }

    #[test]
    fn test_tier_1_words() {
        assert!(TIER_1_WORDS.contains("the"));
        assert!(TIER_1_WORDS.contains("is"));
        assert!(TIER_1_WORDS.contains("file"));
        assert!(!TIER_1_WORDS.contains("pharmacovigilance"));
    }

    #[test]
    fn test_tier_2_words() {
        assert!(TIER_2_WORDS.contains("validate"));
        assert!(TIER_2_WORDS.contains("framework"));
        assert!(TIER_2_WORDS.contains("infrastructure"));
        assert!(!TIER_2_WORDS.contains("the"));
    }

    #[test]
    fn test_technical_patterns() {
        assert!(TechnicalPattern::Acronym.matches("API"));
        assert!(TechnicalPattern::Acronym.matches("FAERS"));
        assert!(!TechnicalPattern::Acronym.matches("api"));

        assert!(TechnicalPattern::CamelCase.matches("getUserData"));
        assert!(!TechnicalPattern::CamelCase.matches("GetUserData"));

        assert!(TechnicalPattern::SnakeCase.matches("get_user_data"));
        assert!(!TechnicalPattern::SnakeCase.matches("getUserData"));

        assert!(TechnicalPattern::KebabCase.matches("get-user-data"));
        assert!(!TechnicalPattern::KebabCase.matches("get_user_data"));

        assert!(TechnicalPattern::Version.matches("v1.0"));
        assert!(TechnicalPattern::Version.matches("2.5.1"));
    }

    #[test]
    fn test_pattern_detection() {
        assert_eq!(
            TechnicalPattern::detect("API"),
            Some(TechnicalPattern::Acronym)
        );
        assert_eq!(
            TechnicalPattern::detect("getUserData"),
            Some(TechnicalPattern::CamelCase)
        );
        assert_eq!(
            TechnicalPattern::detect("get_user_data"),
            Some(TechnicalPattern::SnakeCase)
        );
    }
}
