// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Quality Scoring Engine
//!
//! Deterministic quality validation for KSB research content.
//! Migrated from `studio/backend/app/services/ksb_quality_validator.py`.
//!
//! ## Primitive Grounding
//!
//! | Component | Primitive | Meaning |
//! |-----------|-----------|---------|
//! | Score thresholds | ∂ | Boundaries that separate pass/warn/fail |
//! | Source credibility | κ | Comparison of source quality |
//! | Bloom's levels | σ | Ordered cognitive sequence |
//! | Overall score | N | Quantified quality measurement |

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Constants (from ksb_quality_validator.py)
// ---------------------------------------------------------------------------

/// Minimum character count for valid research content.
pub const MIN_CHARACTERS: usize = 15_000;

/// Recommended character count for high-quality content.
pub const RECOMMENDED_CHARACTERS: usize = 18_000;

/// Minimum citation count for valid research content.
pub const MIN_CITATIONS: usize = 40;

/// Recommended citation count for high-quality content.
pub const RECOMMENDED_CITATIONS: usize = 50;

// ---------------------------------------------------------------------------
// Source Credibility
// ---------------------------------------------------------------------------

/// Score a URL source by its domain trustworthiness.
///
/// Returns a credibility score from 0–100. Higher-specificity matches
/// take priority (e.g., `nih.gov` matches before `.gov`).
///
/// Source: NDC Learning Pipeline quality standards.
pub fn source_credibility_score(url: &str) -> u8 {
    let lower = url.to_lowercase();

    // Specific domains first (higher specificity wins)
    let specific: &[(&str, u8)] = &[
        ("nih.gov", 95),
        ("fda.gov", 95),
        ("ncbi.nlm.nih.gov", 95),
        ("nejm.org", 95),
        ("pubmed", 95),
        ("nature.com", 90),
        ("science.org", 90),
        ("doi.org", 90),
        ("jama", 90),
        ("thelancet", 90),
        ("scholar.google", 85),
        ("springer", 85),
        ("wiley", 85),
        ("elsevier", 85),
        ("researchgate", 80),
        ("arxiv", 80),
        ("wikipedia", 55),
        ("medium", 50),
        ("blog", 40),
    ];

    for &(pattern, score) in specific {
        if lower.contains(pattern) {
            return score;
        }
    }

    // Generic TLD fallbacks
    if lower.contains(".gov") {
        return 95;
    }
    if lower.contains(".edu") {
        return 90;
    }
    if lower.contains(".org") {
        return 85;
    }
    if lower.contains(".com") {
        return 60;
    }

    50 // Unknown source
}

/// Compute average credibility score across multiple source URLs.
pub fn average_credibility(sources: &[&str]) -> f64 {
    if sources.is_empty() {
        return 0.0;
    }
    let total: u32 = sources
        .iter()
        .map(|s| u32::from(source_credibility_score(s)))
        .sum();
    f64::from(total) / sources.len() as f64
}

// ---------------------------------------------------------------------------
// Bloom's Taxonomy
// ---------------------------------------------------------------------------

/// The six levels of Bloom's Taxonomy (cognitive domain).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BloomLevel {
    /// Recall facts and basic concepts.
    Remember,
    /// Explain ideas or concepts.
    Understand,
    /// Use information in new situations.
    Apply,
    /// Draw connections among ideas.
    Analyze,
    /// Justify a decision or course of action.
    Evaluate,
    /// Produce new or original work.
    Create,
}

impl BloomLevel {
    /// Keywords that indicate this cognitive level in content.
    pub fn keywords(&self) -> &'static [&'static str] {
        match self {
            Self::Remember => &["define", "identify", "list", "name", "recall", "recognize"],
            Self::Understand => &["explain", "describe", "summarize", "interpret", "clarify"],
            Self::Apply => &[
                "demonstrate",
                "implement",
                "use",
                "execute",
                "apply",
                "solve",
            ],
            Self::Analyze => &["compare", "contrast", "examine", "categorize", "analyze"],
            Self::Evaluate => &[
                "assess",
                "judge",
                "critique",
                "justify",
                "evaluate",
                "recommend",
            ],
            Self::Create => &[
                "design",
                "develop",
                "formulate",
                "construct",
                "create",
                "generate",
            ],
        }
    }

    /// All six levels in cognitive order.
    pub const ALL: [BloomLevel; 6] = [
        Self::Remember,
        Self::Understand,
        Self::Apply,
        Self::Analyze,
        Self::Evaluate,
        Self::Create,
    ];
}

/// Count how many Bloom's levels are represented in content.
pub fn bloom_coverage(content: &str) -> Vec<BloomLevel> {
    let lower = content.to_lowercase();
    BloomLevel::ALL
        .iter()
        .filter(|level| level.keywords().iter().any(|kw| lower.contains(kw)))
        .copied()
        .collect()
}

// ---------------------------------------------------------------------------
// Practical Examples Detection
// ---------------------------------------------------------------------------

/// Indicators that content contains practical examples.
const EXAMPLE_INDICATORS: &[&str] = &[
    "star method",
    "star format",
    "case study",
    "scenario",
    "for example",
    "example:",
    "real-world example",
    "resume bullet",
    "interview story",
];

/// Check if content contains practical examples.
pub fn has_practical_examples(content: &str) -> bool {
    let lower = content.to_lowercase();
    EXAMPLE_INDICATORS.iter().any(|ind| lower.contains(ind))
}

/// Check if content references current information (recent years).
pub fn has_current_info(content: &str) -> bool {
    ["2023", "2024", "2025", "2026"]
        .iter()
        .any(|yr| content.contains(yr))
}

// ---------------------------------------------------------------------------
// Quality Validation Result
// ---------------------------------------------------------------------------

/// Severity of a quality issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueSeverity {
    /// Blocks publication.
    Error,
    /// Should be addressed.
    Warning,
}

/// A single quality issue found during validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityIssue {
    /// What dimension this issue relates to.
    pub dimension: String,
    /// Severity level.
    pub severity: IssueSeverity,
    /// Human-readable description.
    pub message: String,
}

/// Result of quality validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityResult {
    /// Whether content passes minimum standards.
    pub passes_standards: bool,
    /// Overall quality score (0–100).
    pub overall_score: f64,
    /// Character count of content.
    pub character_count: usize,
    /// Citation count.
    pub citation_count: usize,
    /// Average source credibility (0–100).
    pub avg_credibility: f64,
    /// Bloom's taxonomy levels found.
    pub bloom_levels_found: usize,
    /// Whether practical examples are present.
    pub has_examples: bool,
    /// Whether current year references exist.
    pub has_current_info: bool,
    /// Individual issues found.
    pub issues: Vec<QualityIssue>,
}

/// Validate research content quality.
///
/// Applies the NDC Learning Pipeline quality standards:
/// - Character count (25% weight)
/// - Citation count (25% weight)
/// - Source credibility (20% weight)
/// - Bloom's taxonomy coverage (15% weight)
/// - Practical examples (10% weight)
/// - Current information (5% weight)
pub fn validate_quality(content: &str, citation_urls: &[&str]) -> QualityResult {
    let character_count = content.len();
    let citation_count = citation_urls.len();
    let avg_cred = average_credibility(citation_urls);
    let bloom_levels = bloom_coverage(content);
    let bloom_count = bloom_levels.len();
    let examples = has_practical_examples(content);
    let current = has_current_info(content);

    let mut issues = Vec::new();

    // Character count checks
    if character_count < MIN_CHARACTERS {
        issues.push(QualityIssue {
            dimension: "characters".into(),
            severity: IssueSeverity::Error,
            message: format!(
                "Content has {} characters, minimum is {}",
                character_count, MIN_CHARACTERS
            ),
        });
    } else if character_count < RECOMMENDED_CHARACTERS {
        issues.push(QualityIssue {
            dimension: "characters".into(),
            severity: IssueSeverity::Warning,
            message: format!(
                "Content has {} characters, recommended is {}",
                character_count, RECOMMENDED_CHARACTERS
            ),
        });
    }

    // Citation count checks
    if citation_count < MIN_CITATIONS {
        issues.push(QualityIssue {
            dimension: "citations".into(),
            severity: IssueSeverity::Error,
            message: format!(
                "Content has {} citations, minimum is {}",
                citation_count, MIN_CITATIONS
            ),
        });
    } else if citation_count < RECOMMENDED_CITATIONS {
        issues.push(QualityIssue {
            dimension: "citations".into(),
            severity: IssueSeverity::Warning,
            message: format!(
                "Content has {} citations, recommended is {}",
                citation_count, RECOMMENDED_CITATIONS
            ),
        });
    }

    // Source credibility
    if avg_cred < 70.0 {
        issues.push(QualityIssue {
            dimension: "credibility".into(),
            severity: IssueSeverity::Warning,
            message: format!("Average source credibility is {avg_cred:.1}, recommended >= 70.0"),
        });
    }

    // Bloom's coverage
    if bloom_count < 4 {
        issues.push(QualityIssue {
            dimension: "bloom_taxonomy".into(),
            severity: IssueSeverity::Warning,
            message: format!("Only {bloom_count}/6 Bloom's levels represented, recommended >= 4"),
        });
    }

    // Practical examples
    if !examples {
        issues.push(QualityIssue {
            dimension: "examples".into(),
            severity: IssueSeverity::Warning,
            message: "No practical examples detected (STAR method, case studies, scenarios)".into(),
        });
    }

    // Current information
    if !current {
        issues.push(QualityIssue {
            dimension: "currency".into(),
            severity: IssueSeverity::Warning,
            message: "No references to recent years (2023-2026) found".into(),
        });
    }

    // Weighted overall score (capped at 100)
    let char_pct = (character_count as f64 / RECOMMENDED_CHARACTERS as f64).min(1.0) * 100.0;
    let cite_pct = (citation_count as f64 / RECOMMENDED_CITATIONS as f64).min(1.0) * 100.0;
    let bloom_pct = (bloom_count as f64 / 6.0) * 100.0;
    let example_pct = if examples { 100.0 } else { 0.0 };
    let current_pct = if current { 100.0 } else { 0.0 };

    let overall_score = (char_pct * 0.25
        + cite_pct * 0.25
        + avg_cred * 0.20
        + bloom_pct * 0.15
        + example_pct * 0.10
        + current_pct * 0.05)
        .min(100.0);

    let has_errors = issues.iter().any(|i| i.severity == IssueSeverity::Error);

    QualityResult {
        passes_standards: !has_errors,
        overall_score,
        character_count,
        citation_count,
        avg_credibility: avg_cred,
        bloom_levels_found: bloom_count,
        has_examples: examples,
        has_current_info: current,
        issues,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_credibility_specific_domains() {
        assert_eq!(
            source_credibility_score("https://pubmed.ncbi.nlm.nih.gov/123"),
            95
        );
        assert_eq!(source_credibility_score("https://www.fda.gov/drugs"), 95);
        assert_eq!(source_credibility_score("https://arxiv.org/abs/2024.1"), 80);
        assert_eq!(source_credibility_score("https://medium.com/post"), 50);
        assert_eq!(source_credibility_score("https://random-blog.com/x"), 40);
    }

    #[test]
    fn source_credibility_tld_fallback() {
        assert_eq!(source_credibility_score("https://example.gov"), 95);
        assert_eq!(source_credibility_score("https://mit.edu"), 90);
        assert_eq!(source_credibility_score("https://who.org"), 85);
        assert_eq!(source_credibility_score("https://example.com"), 60);
    }

    #[test]
    fn bloom_coverage_detects_levels() {
        let content = "Define the term. Explain the concept. Apply the method. Compare results.";
        let levels = bloom_coverage(content);
        assert_eq!(levels.len(), 4); // Remember, Understand, Apply, Analyze
    }

    #[test]
    fn bloom_coverage_empty_content() {
        let levels = bloom_coverage("no keywords here at all");
        assert!(levels.is_empty());
    }

    #[test]
    fn examples_detection() {
        assert!(has_practical_examples(
            "Use the STAR method to structure your response"
        ));
        assert!(has_practical_examples("Consider this case study:"));
        assert!(!has_practical_examples(
            "This is plain text without examples"
        ));
    }

    #[test]
    fn quality_validation_passes() {
        // Need 15,000+ chars: "define explain apply compare " is 29 chars, 29*520 = 15,080
        let content = "define explain apply compare ".repeat(520);
        let content = format!("{content} for example, in 2024, this case study shows");
        let sources: Vec<&str> = (0..50)
            .map(|_| "https://pubmed.ncbi.nlm.nih.gov/123")
            .collect();
        let result = validate_quality(&content, &sources);
        assert!(result.passes_standards);
        assert!(result.overall_score > 80.0);
    }

    #[test]
    fn quality_validation_fails_on_low_chars() {
        let content = "short content";
        let sources: Vec<&str> = vec![];
        let result = validate_quality(content, &sources);
        assert!(!result.passes_standards);
        assert!(
            result
                .issues
                .iter()
                .any(|i| i.dimension == "characters" && i.severity == IssueSeverity::Error)
        );
    }
}
