//! # Seven Universal Languages
//!
//! Classification of domain patterns into seven universal languages.
//! Every domain pattern can be expressed through one or more of these languages.
//!
//! ## The Seven Languages
//!
//! | Language | Core Concept | Cross-Domain Examples |
//! |----------|--------------|----------------------|
//! | Risk | Probability, hazard | Insurance, medicine, finance |
//! | Optimization | Objective, constraint | Logistics, ML, economics |
//! | Network | Node, edge, flow | Social, neural, infrastructure |
//! | Information | Signal, entropy | Communication, ML, biology |
//! | Resource | Capacity, allocation | Cloud, manufacturing, ecology |
//! | Emergence | Hierarchy, feedback | Biology, society, AI |
//! | Adaptation | Learning, evolution | ML, biology, markets |

use serde::{Deserialize, Serialize};

/// One of seven universal languages for domain pattern classification.
///
/// These languages form a complete basis for expressing domain concepts.
/// Every domain pattern maps to one or more languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DomainLanguage {
    /// Risk: Probability, uncertainty, hazard, exposure.
    /// Rust: `Result<T, E>`, probability types, error handling.
    Risk,

    /// Optimization: Objective function, constraints, feasibility.
    /// Rust: Iterators with `min`/`max`, constraint types.
    Optimization,

    /// Network: Nodes, edges, flows, connectivity.
    /// Rust: Graph types, adjacency structures.
    Network,

    /// Information: Signal, noise, entropy, encoding.
    /// Rust: `Vec<u8>`, channels, serialization.
    Information,

    /// Resource: Capacity, allocation, throughput, contention.
    /// Rust: Pools, `Arc`, semaphores, rate limiters.
    Resource,

    /// Emergence: Hierarchy, levels, phase transitions, feedback.
    /// Rust: Nested enums, recursive types.
    Emergence,

    /// Adaptation: Learning, evolution, fitness, mutation.
    /// Rust: State machines, strategy patterns.
    Adaptation,
}

impl DomainLanguage {
    /// All seven languages.
    pub const ALL: [DomainLanguage; 7] = [
        DomainLanguage::Risk,
        DomainLanguage::Optimization,
        DomainLanguage::Network,
        DomainLanguage::Information,
        DomainLanguage::Resource,
        DomainLanguage::Emergence,
        DomainLanguage::Adaptation,
    ];

    /// Returns the language name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Risk => "Risk",
            Self::Optimization => "Optimization",
            Self::Network => "Network",
            Self::Information => "Information",
            Self::Resource => "Resource",
            Self::Emergence => "Emergence",
            Self::Adaptation => "Adaptation",
        }
    }

    /// Returns core concepts for this language.
    #[must_use]
    pub const fn core_concepts(&self) -> &'static [&'static str] {
        match self {
            Self::Risk => &[
                "probability",
                "uncertainty",
                "hazard",
                "exposure",
                "mitigation",
            ],
            Self::Optimization => &[
                "objective",
                "constraint",
                "feasibility",
                "optimum",
                "trade-off",
            ],
            Self::Network => &["node", "edge", "flow", "connectivity", "path"],
            Self::Information => &["signal", "noise", "entropy", "encoding", "channel"],
            Self::Resource => &["capacity", "allocation", "throughput", "contention", "pool"],
            Self::Emergence => &["hierarchy", "level", "phase", "feedback", "emergence"],
            Self::Adaptation => &["learning", "evolution", "fitness", "mutation", "selection"],
        }
    }

    /// Returns typical Rust types for this language.
    #[must_use]
    pub const fn rust_types(&self) -> &'static [&'static str] {
        match self {
            Self::Risk => &[
                "Result<T, E>",
                "Option<T>",
                "f64 probability",
                "Error types",
            ],
            Self::Optimization => &["Iterator", "min/max", "Ord", "constraint structs"],
            Self::Network => &["Graph<N, E>", "HashMap adjacency", "petgraph types"],
            Self::Information => &["Vec<u8>", "Channel<T>", "Serialize/Deserialize"],
            Self::Resource => &["Arc<T>", "Pool<T>", "Semaphore", "RateLimiter"],
            Self::Emergence => &["enum { Variant(Box<Self>) }", "nested types"],
            Self::Adaptation => &["StateMachine", "Strategy trait", "Fn mutation"],
        }
    }

    /// Returns keywords that suggest this language.
    #[must_use]
    pub const fn keywords(&self) -> &'static [&'static str] {
        match self {
            Self::Risk => &[
                "risk",
                "probability",
                "hazard",
                "danger",
                "exposure",
                "likelihood",
                "uncertainty",
                "volatile",
                "adverse",
                "harm",
                "threat",
            ],
            Self::Optimization => &[
                "optimize",
                "minimize",
                "maximize",
                "constraint",
                "objective",
                "feasible",
                "optimal",
                "trade-off",
                "pareto",
                "efficient",
            ],
            Self::Network => &[
                "network",
                "graph",
                "node",
                "edge",
                "connection",
                "flow",
                "path",
                "route",
                "topology",
                "link",
                "vertex",
            ],
            Self::Information => &[
                "signal",
                "data",
                "message",
                "encode",
                "decode",
                "entropy",
                "channel",
                "transmit",
                "receive",
                "bandwidth",
                "compression",
            ],
            Self::Resource => &[
                "resource",
                "capacity",
                "allocate",
                "pool",
                "limit",
                "quota",
                "throughput",
                "budget",
                "consume",
                "reserve",
                "contention",
            ],
            Self::Emergence => &[
                "emerge",
                "hierarchy",
                "level",
                "layer",
                "phase",
                "transition",
                "feedback",
                "cascade",
                "aggregate",
                "collective",
                "system",
            ],
            Self::Adaptation => &[
                "adapt",
                "learn",
                "evolve",
                "mutate",
                "fitness",
                "selection",
                "train",
                "update",
                "improve",
                "genetic",
                "neural",
            ],
        }
    }
}

impl std::fmt::Display for DomainLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Classification result for a domain pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageClassification {
    /// Primary language (highest confidence).
    pub primary: DomainLanguage,

    /// Primary language confidence [0.0, 1.0].
    pub primary_confidence: f64,

    /// Secondary languages with confidence scores.
    #[serde(default)]
    pub secondary: Vec<(DomainLanguage, f64)>,

    /// Matched keywords that led to classification.
    #[serde(default)]
    pub matched_keywords: Vec<String>,

    /// Classification rationale.
    #[serde(default)]
    pub rationale: Option<String>,
}

impl LanguageClassification {
    /// Creates a single-language classification.
    #[must_use]
    pub fn single(language: DomainLanguage, confidence: f64, rationale: &str) -> Self {
        Self {
            primary: language,
            primary_confidence: confidence.clamp(0.0, 1.0),
            secondary: Vec::new(),
            matched_keywords: Vec::new(),
            rationale: Some(rationale.to_string()),
        }
    }

    /// Creates a multi-language classification.
    #[must_use]
    pub fn multi(
        primary: DomainLanguage,
        confidence: f64,
        secondary: Vec<(DomainLanguage, f64)>,
    ) -> Self {
        Self {
            primary,
            primary_confidence: confidence.clamp(0.0, 1.0),
            secondary,
            matched_keywords: Vec::new(),
            rationale: None,
        }
    }

    /// Adds matched keywords.
    #[must_use]
    pub fn with_keywords(mut self, keywords: Vec<String>) -> Self {
        self.matched_keywords = keywords;
        self
    }

    /// Returns all languages with their confidences.
    pub fn all_languages(&self) -> impl Iterator<Item = (DomainLanguage, f64)> + '_ {
        std::iter::once((self.primary, self.primary_confidence))
            .chain(self.secondary.iter().copied())
    }

    /// Returns the total confidence (sum, capped at 1.0).
    #[must_use]
    pub fn total_confidence(&self) -> f64 {
        let sum: f64 = self.all_languages().map(|(_, c)| c).sum();
        sum.min(1.0)
    }
}

/// Trait for classifying domain patterns into languages.
pub trait LanguageClassifier {
    /// Classifies a text pattern into domain languages.
    fn classify(&self, text: &str) -> LanguageClassification;

    /// Classifies with a specific context hint.
    fn classify_with_context(&self, text: &str, _context: &str) -> LanguageClassification {
        // Default: ignore context
        self.classify(text)
    }
}

/// Simple keyword-based language classifier.
#[derive(Debug, Default)]
pub struct KeywordClassifier {
    /// Minimum keyword matches for classification.
    pub min_matches: usize,
    /// Confidence boost per additional match.
    pub match_boost: f64,
}

impl KeywordClassifier {
    /// Creates a new keyword classifier.
    #[must_use]
    pub fn new() -> Self {
        Self {
            min_matches: 1,
            match_boost: 0.05,
        }
    }

    /// Counts keyword matches for a language.
    fn count_matches(&self, text: &str, language: DomainLanguage) -> (usize, Vec<String>) {
        let text_lower = text.to_lowercase();
        let mut matches = Vec::new();

        for keyword in language.keywords() {
            if text_lower.contains(keyword) {
                matches.push((*keyword).to_string());
            }
        }

        (matches.len(), matches)
    }
}

impl LanguageClassifier for KeywordClassifier {
    fn classify(&self, text: &str) -> LanguageClassification {
        let mut scores: Vec<(DomainLanguage, usize, Vec<String>)> = DomainLanguage::ALL
            .iter()
            .map(|&lang| {
                let (count, keywords) = self.count_matches(text, lang);
                (lang, count, keywords)
            })
            .filter(|(_, count, _)| *count >= self.min_matches)
            .collect();

        // Sort by match count descending
        scores.sort_by(|a, b| b.1.cmp(&a.1));

        if scores.is_empty() {
            // Default to Information if no matches
            return LanguageClassification::single(
                DomainLanguage::Information,
                0.3,
                "No keyword matches; defaulting to Information",
            );
        }

        let (primary, primary_count, primary_keywords) = scores.remove(0);
        let base_confidence = 0.5;
        let primary_confidence =
            (base_confidence + (primary_count as f64 * self.match_boost)).min(0.95);

        let secondary: Vec<(DomainLanguage, f64)> = scores
            .into_iter()
            .take(3)
            .map(|(lang, count, _)| {
                let conf = (0.3 + (count as f64 * self.match_boost)).min(0.7);
                (lang, conf)
            })
            .collect();

        LanguageClassification {
            primary,
            primary_confidence,
            secondary,
            matched_keywords: primary_keywords,
            rationale: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_language_all() {
        assert_eq!(DomainLanguage::ALL.len(), 7);
    }

    #[test]
    fn test_domain_language_names() {
        assert_eq!(DomainLanguage::Risk.name(), "Risk");
        assert_eq!(DomainLanguage::Emergence.name(), "Emergence");
    }

    #[test]
    fn test_language_classification_single() {
        let c =
            LanguageClassification::single(DomainLanguage::Risk, 0.9, "High probability keywords");
        assert_eq!(c.primary, DomainLanguage::Risk);
        assert!((c.primary_confidence - 0.9).abs() < 0.001);
    }

    #[test]
    fn test_keyword_classifier_risk() {
        let classifier = KeywordClassifier::new();
        let result = classifier.classify("Calculate the probability of an adverse event");

        assert_eq!(result.primary, DomainLanguage::Risk);
        assert!(result.primary_confidence > 0.5);
        assert!(result.matched_keywords.contains(&"probability".to_string()));
    }

    #[test]
    fn test_keyword_classifier_network() {
        let classifier = KeywordClassifier::new();
        let result = classifier.classify("Add a node to the graph and connect edges");

        assert_eq!(result.primary, DomainLanguage::Network);
        assert!(result.matched_keywords.len() >= 2);
    }

    #[test]
    fn test_keyword_classifier_default() {
        let classifier = KeywordClassifier::new();
        let result = classifier.classify("Some random text without keywords");

        // Should default to Information
        assert_eq!(result.primary, DomainLanguage::Information);
        assert!(result.primary_confidence < 0.5);
    }
}
