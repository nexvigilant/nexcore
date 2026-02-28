//! # Guardian Coding
//!
//! Medical coding engine for `MedDRA` and `WHODrug` dictionaries.
//!
//! ## Features
//!
//! - **`MedDRA` Coding**: Load and query `MedDRA` terminology with fuzzy search
//! - **Hierarchy Traversal**: Navigate LLT -> PT -> HLT -> HLGT -> SOC
//! - **Fuzzy Matching**: BK-tree indexed Levenshtein distance search
//! - **High Performance**: O(1) code lookups, O(n^(1-1/d)) fuzzy search
//! - **PV Glossary**: 1100+ ICH/CIOMS pharmacovigilance terms with fuzzy search
//!
//! ## State Management
//!
//! The `MeddraDictionary` is the primary stateful component:
//!
//! - **Thread-safe reads**: Multiple concurrent queries are safe
//! - **Immutable after loading**: Dictionary is built once, then read-only
//! - **No external state**: No database, network, or filesystem during queries
//!
//! ## Example
//!
//! ```rust,ignore
//! use nexcore_vigilance::coding::meddra::{MeddraDictionary, MeddraVersion, loader};
//!
//! // Create dictionary
//! let mut dict = MeddraDictionary::new(MeddraVersion::new(26, 1, "English"));
//!
//! // Load MedDRA ASCII files
//! dict.load_llts(loader::parse_llt(&llt_content)?);
//! dict.load_pts(loader::parse_pt(&pt_content)?);
//!
//! // Encode a term (finds best match)
//! let result = dict.encode("headache")?;
//! assert_eq!(result.code, 10019231);
//!
//! // Get full hierarchy path
//! let path = dict.get_hierarchy_path(10019211)?;
//! println!("SOC: {}", path.soc_name);
//! ```
//!
//! ## Performance
//!
//! | Operation | Complexity | Notes |
//! |-----------|------------|-------|
//! | Code lookup | O(1) | HashMap |
//! | Exact name match | O(1) | HashMap |
//! | Fuzzy search | O(n^(1-1/d)) | BK-tree, d=max_distance |
//! | Hierarchy path | O(1) | 5 HashMap lookups |
//! | Batch encode | O(n * search) | Parallelized with rayon |

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
// Pre-existing style - format! used for regex patterns with escapes
#![allow(clippy::useless_format)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::double_must_use)]

pub mod error;
pub mod fuzzy;
pub mod glossary;
pub mod meddra;

// Re-export main types for convenience
pub use error::CodingError;
pub use fuzzy::{BkTree, jaro, jaro_winkler, levenshtein, levenshtein_similarity};
pub use meddra::{
    DictionaryStats, HierarchyLevel, HierarchyPath, Hlgt, Hlt, Llt, MeddraDictionary,
    MeddraVersion, Pt, SearchResult, Soc, TitrationProvenance,
};
// Re-export glossary for PV terminology lookup
pub use glossary::{
    GlossaryTerm, fuzzy_best_match, fuzzy_lookup_term, lookup_term, search_terms, smart_lookup,
    term_count,
};
