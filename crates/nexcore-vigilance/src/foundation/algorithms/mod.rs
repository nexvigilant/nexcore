//! # Algorithms Module
//!
//! Core algorithms for nexcore foundation layer.
//!
//! ## Submodules
//!
//! - **levenshtein** - Edit distance and fuzzy string matching (63x faster than Python)
//! - **graph** - DAG operations: topological sort, shortest path, level parallelization
//! - **crypto** - SHA-256 hashing and verification (20x faster than Python)
//! - **math** - Statistical calculations and metrics
//! - **compression** - Gzip compression utilities
//! - **scheduling** - FSRS spaced repetition algorithm for optimal learning
//! - **concept_grep** - Deterministic concept expansion for multi-variant search

pub mod compression;
pub mod concept_grep;
pub mod crypto;
pub mod graph;
pub mod hamming;
pub mod jaro;
pub mod levenshtein;
pub mod math;
pub mod scheduling;
pub mod spatial_bridge;

// Re-export key types
pub use compression::{compress_gzip, decompress_gzip};
pub use crypto::{HashResult, sha256_bytes, sha256_hash, sha256_verify};
pub use graph::{SkillGraph, SkillNode};
pub use hamming::{hamming, hamming_distance};
pub use jaro::{jaro_similarity, jaro_winkler_similarity};
pub use levenshtein::{
    FuzzyMatch, LevenshteinResult, fuzzy_search, levenshtein, levenshtein_bounded,
    levenshtein_distance,
};
pub use math::{StatisticalSummary, calculate_variance};
pub use scheduling::{Card, CardState, DEFAULT_PARAMETERS, FsrsScheduler, Rating, ReviewResult};
