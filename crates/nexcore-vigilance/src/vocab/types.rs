//! Core domain types for vocabulary intelligence.
//!
//! This module defines the fundamental data structures used throughout
//! the vocabulary analysis system.

use serde::{Deserialize, Serialize};

use super::domain::VocabDomain;
use super::tier::VocabTier;

/// Part of speech classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PartOfSpeech {
    /// Noun (person, place, thing, idea).
    #[default]
    Noun,
    /// Verb (action or state).
    Verb,
    /// Adjective (describes noun).
    Adjective,
    /// Adverb (modifies verb/adjective/adverb).
    Adverb,
    /// Multi-word phrase.
    Phrase,
    /// Compound word.
    Compound,
}

impl PartOfSpeech {
    /// Infer part of speech from word ending.
    #[must_use]
    pub fn infer(word: &str) -> Self {
        let lower = word.to_lowercase();

        // Multi-word → Phrase
        if word.contains(' ') {
            return Self::Phrase;
        }

        // Verb patterns
        if lower.ends_with("ing")
            || lower.ends_with("ed")
            || lower.ends_with("ize")
            || lower.ends_with("ise")
            || lower.ends_with("ate")
            || lower.ends_with("ify")
        {
            return Self::Verb;
        }

        // Noun patterns
        if lower.ends_with("tion")
            || lower.ends_with("sion")
            || lower.ends_with("ment")
            || lower.ends_with("ness")
            || lower.ends_with("ity")
            || lower.ends_with("er")
            || lower.ends_with("or")
        {
            return Self::Noun;
        }

        // Adjective patterns
        if lower.ends_with("able")
            || lower.ends_with("ible")
            || lower.ends_with("ful")
            || lower.ends_with("less")
            || lower.ends_with("ive")
            || lower.ends_with("ous")
        {
            return Self::Adjective;
        }

        // Adverb pattern
        if lower.ends_with("ly") {
            return Self::Adverb;
        }

        Self::Noun // Default
    }

    /// Short label for this POS.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Noun => "NN",
            Self::Verb => "VB",
            Self::Adjective => "JJ",
            Self::Adverb => "RB",
            Self::Phrase => "PH",
            Self::Compound => "CP",
        }
    }
}

impl std::fmt::Display for PartOfSpeech {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Noun => "noun",
            Self::Verb => "verb",
            Self::Adjective => "adjective",
            Self::Adverb => "adverb",
            Self::Phrase => "phrase",
            Self::Compound => "compound",
        };
        write!(f, "{name}")
    }
}

/// A classified vocabulary token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Token {
    /// Original text.
    pub text: String,
    /// Lowercase form.
    pub lower: String,
    /// Part of speech.
    pub pos: PartOfSpeech,
    /// Vocabulary tier.
    pub tier: VocabTier,
    /// Detected domain (if Tier 3).
    pub domain: Option<VocabDomain>,
}

impl Token {
    /// Create a new token.
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        let lower = text.to_lowercase();
        let pos = PartOfSpeech::infer(&text);
        Self {
            text,
            lower,
            pos,
            tier: VocabTier::Basic,
            domain: None,
        }
    }

    /// Set the tier for this token.
    #[must_use]
    pub fn with_tier(mut self, tier: VocabTier) -> Self {
        self.tier = tier;
        self
    }

    /// Set the domain for this token.
    #[must_use]
    pub fn with_domain(mut self, domain: VocabDomain) -> Self {
        self.domain = Some(domain);
        self.tier = VocabTier::DomainSpecific;
        self
    }
}

/// Analysis of word affixes (prefixes and suffixes).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AffixAnalysis {
    /// The analyzed word.
    pub word: String,
    /// Detected prefix (e.g., "re-").
    pub prefix: Option<String>,
    /// Meaning of the prefix.
    pub prefix_meaning: Option<String>,
    /// Detected suffix (e.g., "-tion").
    pub suffix: Option<String>,
    /// Meaning of the suffix.
    pub suffix_meaning: Option<String>,
    /// Inferred root word.
    pub root: Option<String>,
}

impl AffixAnalysis {
    /// Create an empty analysis for a word.
    #[must_use]
    pub fn new(word: impl Into<String>) -> Self {
        Self {
            word: word.into(),
            prefix: None,
            prefix_meaning: None,
            suffix: None,
            suffix_meaning: None,
            root: None,
        }
    }

    /// Check if any affixes were detected.
    #[must_use]
    pub fn has_affixes(&self) -> bool {
        self.prefix.is_some() || self.suffix.is_some()
    }
}

/// Type of compound word formation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompoundType {
    /// Hyphenated compound (e.g., "well-known").
    Hyphenated,
    /// Closed/solid compound (e.g., "database").
    Closed,
    /// Open compound with space (e.g., "ice cream").
    Open,
}

impl std::fmt::Display for CompoundType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Hyphenated => "hyphenated",
            Self::Closed => "closed",
            Self::Open => "open",
        };
        write!(f, "{name}")
    }
}

/// A compound word or phrase.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Compound {
    /// Full compound text.
    pub text: String,
    /// Component parts.
    pub parts: Vec<String>,
    /// Type of compound.
    pub compound_type: CompoundType,
}

impl Compound {
    /// Create a new hyphenated compound.
    #[must_use]
    pub fn hyphenated(text: impl Into<String>) -> Self {
        let text = text.into();
        let parts = text.split('-').map(String::from).collect();
        Self {
            text,
            parts,
            compound_type: CompoundType::Hyphenated,
        }
    }

    /// Create a new open compound.
    #[must_use]
    pub fn open(text: impl Into<String>) -> Self {
        let text = text.into();
        let parts = text.split_whitespace().map(String::from).collect();
        Self {
            text,
            parts,
            compound_type: CompoundType::Open,
        }
    }
}

/// A word collocation (frequently co-occurring words).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Collocation {
    /// Pattern description (e.g., "VB + NN").
    pub pattern: String,
    /// The collocated words.
    pub words: Vec<String>,
    /// Frequency of occurrence.
    pub frequency: u32,
}

impl Collocation {
    /// Create a new collocation.
    #[must_use]
    pub fn new(words: Vec<String>, frequency: u32) -> Self {
        let pattern = words
            .iter()
            .map(|w| PartOfSpeech::infer(w).label())
            .collect::<Vec<_>>()
            .join(" + ");
        Self {
            pattern,
            words,
            frequency,
        }
    }
}

/// A technical idiom with its meaning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Idiom {
    /// The idiom phrase.
    pub phrase: String,
    /// The meaning/definition.
    pub meaning: String,
}

impl Idiom {
    /// Create a new idiom.
    #[must_use]
    pub fn new(phrase: impl Into<String>, meaning: impl Into<String>) -> Self {
        Self {
            phrase: phrase.into(),
            meaning: meaning.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pos_inference() {
        assert_eq!(PartOfSpeech::infer("running"), PartOfSpeech::Verb);
        assert_eq!(PartOfSpeech::infer("validation"), PartOfSpeech::Noun);
        assert_eq!(PartOfSpeech::infer("scalable"), PartOfSpeech::Adjective);
        assert_eq!(PartOfSpeech::infer("quickly"), PartOfSpeech::Adverb);
        assert_eq!(
            PartOfSpeech::infer("signal detection"),
            PartOfSpeech::Phrase
        );
    }

    #[test]
    fn test_token_creation() {
        let token = Token::new("Validation").with_tier(VocabTier::CrossDomain);
        assert_eq!(token.text, "Validation");
        assert_eq!(token.lower, "validation");
        assert_eq!(token.pos, PartOfSpeech::Noun);
        assert_eq!(token.tier, VocabTier::CrossDomain);
    }

    #[test]
    fn test_compound() {
        let compound = Compound::hyphenated("cross-domain");
        assert_eq!(compound.parts, vec!["cross", "domain"]);
        assert_eq!(compound.compound_type, CompoundType::Hyphenated);

        let open = Compound::open("signal detection");
        assert_eq!(open.parts, vec!["signal", "detection"]);
        assert_eq!(open.compound_type, CompoundType::Open);
    }

    #[test]
    fn test_collocation() {
        let collocation =
            Collocation::new(vec!["validate".to_string(), "configuration".to_string()], 5);
        assert_eq!(collocation.pattern, "VB + NN");
        assert_eq!(collocation.frequency, 5);
    }
}
