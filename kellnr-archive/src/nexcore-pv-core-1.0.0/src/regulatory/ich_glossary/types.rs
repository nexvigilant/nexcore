//! Core Types Module
//!
//! Defines the fundamental data structures for ICH glossary terms,
//! definitions, sources, and related metadata.

use serde::{Deserialize, Serialize, Serializer};

/// Helper to serialize static string slices as JSON arrays
pub fn serialize_static_slice<S>(
    slice: &&'static [&'static str],
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.collect_seq(*slice)
}

// ============================================================================
// ICH Categories
// ============================================================================

/// ICH guideline category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IchCategory {
    /// Quality guidelines (Q1-Q14)
    Quality,
    /// Safety guidelines (S1-S12)
    Safety,
    /// Efficacy guidelines (E1-E22)
    Efficacy,
    /// Multidisciplinary guidelines (M1-M15)
    Multidisciplinary,
}

impl IchCategory {
    /// Get the category prefix letter
    pub fn prefix(&self) -> &'static str {
        match self {
            IchCategory::Quality => "Q",
            IchCategory::Safety => "S",
            IchCategory::Efficacy => "E",
            IchCategory::Multidisciplinary => "M",
        }
    }

    /// Get full category name
    pub fn name(&self) -> &'static str {
        match self {
            IchCategory::Quality => "Quality",
            IchCategory::Safety => "Safety",
            IchCategory::Efficacy => "Efficacy",
            IchCategory::Multidisciplinary => "Multidisciplinary",
        }
    }

    /// Get category description
    pub fn description(&self) -> &'static str {
        match self {
            IchCategory::Quality => {
                "Guidelines on pharmaceutical quality, stability, specifications, and manufacturing"
            }
            IchCategory::Safety => {
                "Guidelines on nonclinical safety assessment, toxicology, and pharmacology"
            }
            IchCategory::Efficacy => {
                "Guidelines on clinical trial design, safety reporting, and biostatistics"
            }
            IchCategory::Multidisciplinary => {
                "Cross-cutting guidelines covering terminology, CTD format, and general topics"
            }
        }
    }

    /// Parse category from guideline ID prefix
    pub fn from_prefix(s: &str) -> Option<Self> {
        let first = s.chars().next()?;
        match first.to_ascii_uppercase() {
            'Q' => Some(IchCategory::Quality),
            'S' => Some(IchCategory::Safety),
            'E' => Some(IchCategory::Efficacy),
            'M' => Some(IchCategory::Multidisciplinary),
            _ => None,
        }
    }
}

impl std::fmt::Display for IchCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

// ============================================================================
// Guideline Status
// ============================================================================

/// ICH guideline development status (Step process)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GuidelineStatus {
    /// Step 2: Draft for public consultation
    Step2Draft,
    /// Step 4: Final adopted guideline
    Step4Final,
    /// Work in progress (Expert Working Group)
    WorkInProgress,
    /// Guideline has been withdrawn
    Withdrawn,
    /// Not subject to formal ICH Step process
    NotFormal,
}

impl GuidelineStatus {
    /// Human-readable status string
    pub fn as_str(&self) -> &'static str {
        match self {
            GuidelineStatus::Step2Draft => "Step 2 (draft)",
            GuidelineStatus::Step4Final => "Step 4 (final)",
            GuidelineStatus::WorkInProgress => "Work in progress",
            GuidelineStatus::Withdrawn => "Withdrawn",
            GuidelineStatus::NotFormal => "Not formal ICH process",
        }
    }

    /// Whether this is an active (non-withdrawn) guideline
    pub fn is_active(&self) -> bool {
        !matches!(self, GuidelineStatus::Withdrawn)
    }
}

impl std::fmt::Display for GuidelineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ============================================================================
// Source Reference
// ============================================================================

/// Source reference for a term definition
#[derive(Debug, Clone, Serialize)]
pub struct Source {
    /// ICH guideline ID (e.g., "E2A", "Q9(R1)")
    pub guideline_id: &'static str,
    /// Full guideline title
    pub guideline_title: &'static str,
    /// Guideline status (Step 2/4, etc.)
    pub status: GuidelineStatus,
    /// Publication/adoption date
    pub date: &'static str,
    /// Section reference within the guideline
    pub section: &'static str,
    /// URL to the guideline PDF (if available)
    pub url: Option<&'static str>,
}

impl Source {
    /// Get the ICH category for this source
    pub fn category(&self) -> Option<IchCategory> {
        IchCategory::from_prefix(self.guideline_id)
    }
}

// ============================================================================
// Term Definition
// ============================================================================

/// A glossary term with its definition(s)
#[derive(Debug, Clone, Serialize)]
pub struct Term {
    /// Canonical term name
    pub name: &'static str,
    /// Normalized key for lookup (lowercase, no special chars)
    pub key: &'static str,
    /// Primary definition text
    pub definition: &'static str,
    /// Primary source reference
    pub source: Source,
    /// Alternative definitions from other guidelines
    #[serde(skip_serializing_if = "<[AlternativeDefinition]>::is_empty")]
    pub alternative_definitions: &'static [AlternativeDefinition],
    /// Related terms ("See also" references)
    #[serde(serialize_with = "serialize_static_slice")]
    pub see_also: &'static [&'static str],
    /// Abbreviation(s) if applicable
    pub abbreviation: Option<&'static str>,
    /// CIOMS clarifications (text in curly brackets)
    pub clarification: Option<&'static str>,
    /// Whether this is a new entry in the current glossary version
    pub is_new: bool,
}

impl Term {
    /// Get all ICH categories this term appears in
    pub fn categories(&self) -> Vec<IchCategory> {
        let mut cats = vec![];
        if let Some(cat) = self.source.category() {
            cats.push(cat);
        }
        for alt in self.alternative_definitions {
            if let Some(cat) = alt.source.category() {
                if !cats.contains(&cat) {
                    cats.push(cat);
                }
            }
        }
        cats
    }

    /// Get all source guidelines for this term
    pub fn all_sources(&self) -> Vec<&Source> {
        let mut sources = vec![&self.source];
        for alt in self.alternative_definitions {
            sources.push(&alt.source);
        }
        sources
    }
}

/// Alternative definition from a different guideline
#[derive(Debug, Clone, Serialize)]
pub struct AlternativeDefinition {
    /// Definition text
    pub definition: &'static str,
    /// Source reference
    pub source: Source,
    /// CIOMS clarification if any
    pub clarification: Option<&'static str>,
}

// ============================================================================
// Search Results
// ============================================================================

/// Result of a term search query
#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    /// Matched term
    pub term: Term,
    /// Search relevance score (0.0 - 1.0)
    pub score: f64,
    /// Match type (exact, prefix, contains, fuzzy)
    pub match_type: MatchType,
}

/// Type of match in search results
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchType {
    /// Exact match on term name
    Exact,
    /// Term name starts with query
    Prefix,
    /// Query found within term name
    Contains,
    /// Definition contains query
    DefinitionMatch,
    /// Fuzzy/similarity match
    Fuzzy,
}

impl MatchType {
    /// Base score for this match type
    pub fn base_score(&self) -> f64 {
        match self {
            MatchType::Exact => 1.0,
            MatchType::Prefix => 0.9,
            MatchType::Contains => 0.7,
            MatchType::DefinitionMatch => 0.5,
            MatchType::Fuzzy => 0.3,
        }
    }
}

// ============================================================================
// Guideline Metadata
// ============================================================================

/// ICH Guideline metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guideline {
    /// Guideline ID (e.g., "E2A", "Q9(R1)")
    pub id: &'static str,
    /// Full title
    pub title: &'static str,
    /// ICH category
    pub category: IchCategory,
    /// Current status
    pub status: GuidelineStatus,
    /// Publication date
    pub date: &'static str,
    /// URL to PDF
    pub url: Option<&'static str>,
    /// Number of terms defined in this guideline
    pub term_count: usize,
    /// Brief description
    pub description: &'static str,
}

// ============================================================================
// Query Types
// ============================================================================

/// Query parameters for term search
#[derive(Debug, Clone, Default)]
pub struct TermQuery {
    /// Search text
    pub query: String,
    /// Filter by category
    pub category: Option<IchCategory>,
    /// Filter by guideline
    pub guideline: Option<String>,
    /// Include definitions in search
    pub search_definitions: bool,
    /// Maximum results to return
    pub limit: Option<usize>,
    /// Minimum relevance score
    pub min_score: Option<f64>,
}

impl TermQuery {
    /// Create a new query with just search text
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            ..Default::default()
        }
    }

    /// Filter by category
    pub fn with_category(mut self, category: IchCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Filter by guideline
    pub fn with_guideline(mut self, guideline: impl Into<String>) -> Self {
        self.guideline = Some(guideline.into());
        self
    }

    /// Include definitions in search
    pub fn search_definitions(mut self) -> Self {
        self.search_definitions = true;
        self
    }

    /// Limit results
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
}

// ============================================================================
// Statistics
// ============================================================================

/// Glossary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossaryStats {
    /// Total number of unique terms
    pub total_terms: usize,
    /// Terms by category
    pub terms_by_category: Vec<(IchCategory, usize)>,
    /// Total number of guidelines
    pub total_guidelines: usize,
    /// Guidelines by category
    pub guidelines_by_category: Vec<(IchCategory, usize)>,
    /// Terms with multiple definitions
    pub terms_with_alternatives: usize,
    /// New terms in this version
    pub new_terms: usize,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ich_category_prefix() {
        assert_eq!(IchCategory::Quality.prefix(), "Q");
        assert_eq!(IchCategory::Safety.prefix(), "S");
        assert_eq!(IchCategory::Efficacy.prefix(), "E");
        assert_eq!(IchCategory::Multidisciplinary.prefix(), "M");
    }

    #[test]
    fn test_ich_category_from_prefix() {
        assert_eq!(
            IchCategory::from_prefix("Q9(R1)"),
            Some(IchCategory::Quality)
        );
        assert_eq!(IchCategory::from_prefix("E2A"), Some(IchCategory::Efficacy));
        assert_eq!(IchCategory::from_prefix("S7A"), Some(IchCategory::Safety));
        assert_eq!(
            IchCategory::from_prefix("M14"),
            Some(IchCategory::Multidisciplinary)
        );
        assert_eq!(IchCategory::from_prefix("X1"), None);
    }

    #[test]
    fn test_guideline_status() {
        assert!(GuidelineStatus::Step4Final.is_active());
        assert!(GuidelineStatus::Step2Draft.is_active());
        assert!(!GuidelineStatus::Withdrawn.is_active());
    }

    #[test]
    fn test_match_type_scores() {
        assert!(MatchType::Exact.base_score() > MatchType::Prefix.base_score());
        assert!(MatchType::Prefix.base_score() > MatchType::Contains.base_score());
        assert!(MatchType::Contains.base_score() > MatchType::Fuzzy.base_score());
    }

    #[test]
    fn test_term_query_builder() {
        let query = TermQuery::new("adverse event")
            .with_category(IchCategory::Efficacy)
            .search_definitions()
            .limit(10);

        assert_eq!(query.query, "adverse event");
        assert_eq!(query.category, Some(IchCategory::Efficacy));
        assert!(query.search_definitions);
        assert_eq!(query.limit, Some(10));
    }
}
