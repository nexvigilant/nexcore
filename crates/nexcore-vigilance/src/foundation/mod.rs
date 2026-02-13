//! # nexcore Foundation
//!
//! Foundation layer providing algorithms, state management, execution engine,
//! and data processing capabilities for the nexcore unified kernel.
//!
//! ## Module Organization
//!
//! - **algorithms** - Core algorithms (levenshtein, graph, crypto, math, compression, scheduling)
//! - **data** - Data format processing (YAML, JSON, TOML)
//! - **text** - Text processing and SKILL.md parsing
//! - **decisions** - Decision tree engine and skill chaining
//! - **execution** - DAG-based execution engine with checkpoints
//! - **state** - State management and persistence
//! - **config** - Environment-aware configuration management
//! - **traits** - Common traits (Executable, Calculable, Validatable)
//! - **sop** - SMART goals and Standard Operating Procedures
//!
//! ## Performance
//!
//! | Operation | Python | Rust | Speedup |
//! |-----------|--------|------|---------|
//! | Levenshtein | 63ms | 1ms | 63x |
//! | SHA-256 | 10ms | 0.5ms | 20x |
//! | YAML Parse | 35ms | 5ms | 7x |
//! | Graph Topsort | 50ms | 5ms | 10x |
//!
//! ## Example
//!
//! ```rust,ignore
//! use nexcore_vigilance::foundation::algorithms::{levenshtein, sha256_hash};
//! use nexcore_vigilance::foundation::algorithms::graph::SkillGraph;
//!
//! // Fuzzy string matching (63x faster than Python)
//! let result = levenshtein("kitten", "sitting");
//! assert_eq!(result.distance, 3);
//!
//! // SHA-256 hashing
//! let hash = sha256_hash("hello");
//! assert_eq!(hash.hex.len(), 64);
//! ```

pub mod algorithms;
pub mod config;
pub mod data;
pub mod decisions;
pub mod error;
pub mod execution;
pub mod sop;
pub mod state;
pub mod text;
pub mod traits;

// Re-export commonly used types at crate root
pub use algorithms::concept_grep::{ConceptExpansion, expand_concept};
pub use algorithms::crypto::{HashResult, sha256_bytes, sha256_hash, sha256_verify};
pub use algorithms::graph::{SkillGraph, SkillNode};
pub use algorithms::levenshtein::{
    FuzzyMatch, LevenshteinResult, fuzzy_search, levenshtein, levenshtein_bounded,
    levenshtein_distance,
};
pub use algorithms::scheduling::{Card, CardState, FsrsScheduler, Rating, ReviewResult};
pub use config::{Environment, GcpSettings, NexCoreConfig};
pub use data::mapper::{DataMapper, MappingResult};
pub use data::yaml::{ParseResult, parse_config, parse_toml, parse_yaml};
pub use error::{FoundationError, FoundationResult};
pub use sop::{ImpactCategory, SmartGoal, SmartSop};
pub use traits::{Calculable, Validatable};

/// Returns the version of nexcore-foundation
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
