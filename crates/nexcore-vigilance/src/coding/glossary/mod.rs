//! # PV Glossary Module
//!
//! Pharmacovigilance terminology from ICH and CIOMS sources.
//!
//! ## Features
//!
//! - **O(log n) exact lookup** via binary search on sorted terms
//! - **Fuzzy search** for typo-tolerant lookups using BK-tree
//! - **Substring search** for finding related terms
//! - **Source filtering** by ICH, CIOMS, etc.
//! - **Zero allocation lookups** - all data is static
//!
//! ## Usage
//!
//! ```rust,ignore
//! use nexcore_vigilance::coding::glossary::{lookup_term, search_terms, term_count};
//! use nexcore_vigilance::coding::glossary::{fuzzy_lookup_term, smart_lookup};
//!
//! // Exact lookup (case-insensitive)
//! if let Some(term) = lookup_term("adverse drug reaction") {
//!     println!("Definition: {}", term.definition);
//!     if let Some(guideline) = term.guideline {
//!         println!("Source: {}", guideline);
//!     }
//! }
//!
//! // Fuzzy lookup (typo-tolerant)
//! let results = fuzzy_lookup_term("advers event", 2);
//! for (term, distance) in results {
//!     println!("{} (distance: {})", term.term, distance);
//! }
//!
//! // Smart lookup (exact first, then fuzzy)
//! match smart_lookup("singal", 2) {
//!     Ok(term) => println!("Found: {}", term.definition),
//!     Err(suggestions) => {
//!         for (term, _) in suggestions.iter().take(3) {
//!             println!("Did you mean: {}?", term.term);
//!         }
//!     }
//! }
//!
//! // Search by keyword
//! let signal_terms = search_terms("signal", 10);
//! for term in signal_terms {
//!     println!("- {}", term.term);
//! }
//!
//! // Get total count
//! println!("Glossary contains {} terms", term_count());
//! ```
//!
//! ## Regenerating
//!
//! The terms are auto-generated from CIOMS/ICH source files:
//!
//! ```bash
//! cd crates/guardian-coding
//! ./scripts/build_glossary.sh /path/to/glossary/files
//! ```

mod fuzzy;
mod terms;

pub use fuzzy::{
    FuzzyIndexStats, fuzzy_best_match, fuzzy_index_stats, fuzzy_lookup_term, smart_lookup,
};
pub use terms::{
    GLOSSARY, GlossaryTerm, all_terms, lookup_term, search_by_prefix, search_terms, term_count,
    terms_by_source,
};
