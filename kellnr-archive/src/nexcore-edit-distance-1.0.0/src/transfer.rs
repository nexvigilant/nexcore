//! # Transfer Confidence (Cross-Domain Mapping)
//!
//! Formalizes how edit distance concepts transfer between domains.
//!
//! ```text
//! confidence = structural × 0.4 + functional × 0.4 + contextual × 0.2
//! ```
//!
//! Pre-computed maps for known domain pairs (strings ↔ bioinformatics, etc.)
//! plus a builder for custom transfer assessments.

use serde::{Deserialize, Serialize};

/// Three-dimensional transfer confidence between two domains.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferMap {
    /// Source domain name
    pub source_domain: String,
    /// Target domain name
    pub target_domain: String,
    /// Equivalent concept name in target domain
    pub target_equivalent: String,
    /// Overlap in structural components (Jaccard-like)
    pub structural: f64,
    /// Similarity of purpose/function
    pub functional: f64,
    /// Context compatibility
    pub contextual: f64,
    /// Auto-generated caveat from limiting factor
    pub caveat: String,
}

impl TransferMap {
    /// Composite confidence score.
    /// `confidence = structural × 0.4 + functional × 0.4 + contextual × 0.2`
    #[must_use]
    pub fn confidence(&self) -> f64 {
        self.structural * 0.4 + self.functional * 0.4 + self.contextual * 0.2
    }

    /// Which dimension limits the transfer most.
    #[must_use]
    pub fn limiting_factor(&self) -> &str {
        if self.structural <= self.functional && self.structural <= self.contextual {
            "structural"
        } else if self.functional <= self.contextual {
            "functional"
        } else {
            "contextual"
        }
    }
}

/// Builder for `TransferMap`.
pub struct TransferMapBuilder {
    source_domain: String,
    target_domain: String,
    target_equivalent: String,
    structural: f64,
    functional: f64,
    contextual: f64,
    caveat: String,
}

impl TransferMapBuilder {
    /// Start building a transfer map between two domains.
    #[must_use]
    pub fn new(source: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            source_domain: source.into(),
            target_domain: target.into(),
            target_equivalent: String::new(),
            structural: 0.0,
            functional: 0.0,
            contextual: 0.0,
            caveat: String::new(),
        }
    }

    /// Set the equivalent concept in the target domain.
    #[must_use]
    pub fn equivalent(mut self, name: impl Into<String>) -> Self {
        self.target_equivalent = name.into();
        self
    }

    /// Set structural similarity [0.0, 1.0].
    #[must_use]
    pub fn structural(mut self, score: f64) -> Self {
        self.structural = score.clamp(0.0, 1.0);
        self
    }

    /// Set functional similarity [0.0, 1.0].
    #[must_use]
    pub fn functional(mut self, score: f64) -> Self {
        self.functional = score.clamp(0.0, 1.0);
        self
    }

    /// Set contextual similarity [0.0, 1.0].
    #[must_use]
    pub fn contextual(mut self, score: f64) -> Self {
        self.contextual = score.clamp(0.0, 1.0);
        self
    }

    /// Set explicit caveat.
    #[must_use]
    pub fn caveat(mut self, caveat: impl Into<String>) -> Self {
        self.caveat = caveat.into();
        self
    }

    /// Build the `TransferMap`. Auto-generates caveat if not set.
    #[must_use]
    pub fn build(mut self) -> TransferMap {
        if self.caveat.is_empty() {
            let map = TransferMap {
                source_domain: String::new(),
                target_domain: self.target_domain.clone(),
                target_equivalent: String::new(),
                structural: self.structural,
                functional: self.functional,
                contextual: self.contextual,
                caveat: String::new(),
            };
            self.caveat = format!(
                "Transfer limited by {} dimension ({:.2})",
                map.limiting_factor(),
                match map.limiting_factor() {
                    "structural" => self.structural,
                    "functional" => self.functional,
                    _ => self.contextual,
                }
            );
        }
        TransferMap {
            source_domain: self.source_domain,
            target_domain: self.target_domain,
            target_equivalent: self.target_equivalent,
            structural: self.structural,
            functional: self.functional,
            contextual: self.contextual,
            caveat: self.caveat,
        }
    }
}

// ---------------------------------------------------------------------------
// Pre-computed transfer maps for edit distance domains
// ---------------------------------------------------------------------------

/// Registry of known cross-domain transfer mappings.
#[derive(Debug, Clone, Default)]
pub struct TransferRegistry {
    maps: Vec<TransferMap>,
}

impl TransferRegistry {
    /// Create registry pre-loaded with known edit distance domain transfers.
    #[must_use]
    pub fn with_defaults() -> Self {
        let mut reg = Self::default();

        // Strings ↔ Bioinformatics
        reg.add(
            TransferMapBuilder::new("text/unicode", "bioinformatics/dna")
                .equivalent("Needleman-Wunsch (global alignment)")
                .structural(0.90)
                .functional(0.85)
                .contextual(0.55)
                .caveat("Bio uses variable substitution costs (BLOSUM/PAM matrices)")
                .build(),
        );

        // Strings ↔ Spell checking
        reg.add(
            TransferMapBuilder::new("text/unicode", "spell-checking")
                .equivalent("edit distance with keyboard-weighted costs")
                .structural(0.95)
                .functional(0.90)
                .contextual(0.65)
                .caveat("Spell checkers often add transposition (Damerau-Levenshtein)")
                .build(),
        );

        // Strings ↔ NLP (WER)
        reg.add(
            TransferMapBuilder::new("text/unicode", "nlp/tokens")
                .equivalent("word error rate (WER)")
                .structural(0.80)
                .functional(0.75)
                .contextual(0.45)
                .caveat("WER operates on word tokens, not characters")
                .build(),
        );

        // Bio ↔ Music (sequence alignment)
        reg.add(
            TransferMapBuilder::new("bioinformatics/dna", "music/melody")
                .equivalent("melodic sequence alignment")
                .structural(0.75)
                .functional(0.60)
                .contextual(0.40)
                .caveat("Music uses pitch intervals, not discrete alphabet")
                .build(),
        );

        // Strings ↔ PV signal detection
        reg.add(
            TransferMapBuilder::new("text/unicode", "pharmacovigilance")
                .equivalent("drug name fuzzy matching (FAERS)")
                .structural(0.95)
                .functional(0.85)
                .contextual(0.70)
                .caveat("PV requires high recall; false negatives are safety-critical")
                .build(),
        );

        reg
    }

    /// Add a transfer map to the registry.
    pub fn add(&mut self, map: TransferMap) {
        self.maps.push(map);
    }

    /// Look up transfer between two domains. Returns all matches.
    #[must_use]
    pub fn lookup(&self, source: &str, target: &str) -> Vec<&TransferMap> {
        self.maps
            .iter()
            .filter(|m| {
                (m.source_domain == source && m.target_domain == target)
                    || (m.source_domain == target && m.target_domain == source)
            })
            .collect()
    }

    /// All registered maps.
    #[must_use]
    pub fn all(&self) -> &[TransferMap] {
        &self.maps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confidence_formula() {
        let map = TransferMapBuilder::new("a", "b")
            .equivalent("B-equivalent")
            .structural(0.90)
            .functional(0.80)
            .contextual(0.55)
            .build();
        // 0.90*0.4 + 0.80*0.4 + 0.55*0.2 = 0.36 + 0.32 + 0.11 = 0.79
        assert!((map.confidence() - 0.79).abs() < 0.001);
    }

    #[test]
    fn limiting_factor_detection() {
        let map = TransferMapBuilder::new("a", "b")
            .structural(0.90)
            .functional(0.80)
            .contextual(0.55)
            .build();
        assert_eq!(map.limiting_factor(), "contextual");
    }

    #[test]
    fn auto_caveat_generation() {
        let map = TransferMapBuilder::new("a", "b")
            .structural(0.90)
            .functional(0.80)
            .contextual(0.55)
            .build();
        assert!(map.caveat.contains("contextual"));
    }

    #[test]
    fn explicit_caveat_preserved() {
        let map = TransferMapBuilder::new("a", "b")
            .caveat("Custom warning")
            .build();
        assert_eq!(map.caveat, "Custom warning");
    }

    #[test]
    fn registry_defaults() {
        let reg = TransferRegistry::with_defaults();
        assert!(reg.all().len() >= 4);
    }

    #[test]
    fn registry_lookup() {
        let reg = TransferRegistry::with_defaults();
        let maps = reg.lookup("text/unicode", "bioinformatics/dna");
        assert_eq!(maps.len(), 1);
        assert!(maps[0].confidence() > 0.7);
    }

    #[test]
    fn registry_bidirectional_lookup() {
        let reg = TransferRegistry::with_defaults();
        let fwd = reg.lookup("text/unicode", "nlp/tokens");
        let rev = reg.lookup("nlp/tokens", "text/unicode");
        assert_eq!(fwd.len(), rev.len());
    }

    #[test]
    fn serialize_roundtrip() {
        let map = TransferMapBuilder::new("a", "b")
            .equivalent("B-thing")
            .structural(0.9)
            .functional(0.8)
            .contextual(0.5)
            .build();
        let json = serde_json::to_string(&map).expect("serialize");
        let m2: TransferMap = serde_json::from_str(&json).expect("deserialize");
        assert!((map.confidence() - m2.confidence()).abs() < f64::EPSILON);
    }
}
