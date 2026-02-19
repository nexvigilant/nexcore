//! # ICH Glossary - Pharmacovigilance Regulatory Terminology
//!
//! High-performance O(1) lookup for 894+ ICH/CIOMS terms from regulatory guidelines.
//!
//! ## Modules
//!
//! - `types` - Core data structures (Term, Definition, Source)
//! - `guidelines` - ICH guideline metadata and lookup
//! - `categories` - Category classification (Q/S/E/M)
//! - `terms` - Term definitions with PHF lookup tables
//! - `search` - Search and query functionality
//!
//! ## Usage
//!
//! ```rust
//! use nexcore_vigilance::pv::regulatory::ich_glossary::{lookup_term, search_terms};
//!
//! // O(1) term lookup
//! if let Some(term) = lookup_term("adverse drug reaction") {
//!     println!("Definition: {}", term.definition);
//! }
//!
//! // Fuzzy search
//! let results = search_terms("signal detection");
//! ```

pub mod categories;
pub mod guidelines;
pub mod search;
pub mod terms;
pub mod types;

// Re-export commonly used types
pub use categories::CategoryInfo;
pub use search::SearchStats;
pub use types::{
    AlternativeDefinition, GlossaryStats, Guideline, GuidelineStatus, IchCategory, MatchType,
    SearchResult, Source, Term, TermQuery,
};

// Re-export commonly used functions - Categories
pub use categories::{all_categories, lookup_category, parse_category};

// Re-export commonly used functions - Guidelines
pub use guidelines::{
    all_guidelines, guideline_count_by_category, guidelines_by_category, lookup_guideline,
};

// Re-export commonly used functions - Search
pub use search::{
    autocomplete, get_related_terms, get_search_stats, search_by_guideline_series, search_terms,
    search_with_query,
};

// Re-export commonly used functions - Terms
pub use terms::{
    all_terms, lookup_term, new_terms, terms_by_category, terms_by_guideline,
    terms_with_abbreviations, terms_with_alternatives,
};

/// Total number of terms in the glossary (from build report)
pub const TOTAL_TERM_COUNT: usize = 904;

/// Total number of ICH guidelines referenced
pub const TOTAL_GUIDELINE_COUNT: usize = 127;

/// ICH Glossary metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GlossaryMetadata {
    pub version: String,
    pub release_date: String,
    pub source: String,
    pub source_url: String,
    pub total_terms: usize,
    pub total_guidelines: usize,
}

/// Get ICH glossary metadata
#[must_use]
pub fn glossary_metadata() -> GlossaryMetadata {
    GlossaryMetadata {
        version: "9".to_string(),
        release_date: "2025-12-09".to_string(),
        source: "CIOMS".to_string(),
        source_url: "https://doi.org/10.56759/eftb6868".to_string(),
        total_terms: TOTAL_TERM_COUNT,
        total_guidelines: TOTAL_GUIDELINE_COUNT,
    }
}

// Python bindings not needed in NexCore (standalone library only)
// #[cfg(feature = "python")]
// pub mod python_bindings;
