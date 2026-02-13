//! `MedDRA` medical coding module.
//!
//! Provides functionality for working with the `MedDRA` (Medical Dictionary
//! for Regulatory Activities) terminology:
//!
//! - Loading `MedDRA` ASCII distribution files
//! - Hierarchy traversal (LLT -> PT -> HLT -> HLGT -> SOC)
//! - Fuzzy term matching with BK-tree indexing
//! - Encoding verbatim terms to `MedDRA` codes
//!
//! # Example
//!
//! ```rust,ignore
//! use nexcore_vigilance::coding::meddra::{MeddraDictionary, loader};
//!
//! // Load MedDRA files
//! let llts = loader::parse_llt(&llt_content)?;
//! let pts = loader::parse_pt(&pt_content)?;
//!
//! // Create dictionary
//! let mut dict = MeddraDictionary::new(MeddraVersion::new(26, 1, "English"));
//! dict.load_llts(llts);
//! dict.load_pts(pts);
//!
//! // Encode a term
//! let result = dict.encode("headache")?;
//! println!("Code: {}, Level: {}", result.code, result.level);
//!
//! // Get full hierarchy
//! let path = dict.get_hierarchy_path(result.code)?;
//! println!("SOC: {}", path.soc_name);
//! ```

pub mod hierarchy;
pub mod loader;
pub mod types;

// Re-export main types
pub use hierarchy::{DictionaryStats, MeddraDictionary};
pub use types::{
    HierarchyLevel, HierarchyPath, Hlgt, Hlt, Llt, MeddraVersion, Pt, SearchResult, Soc,
};
