//! # Levenshtein Edit Distance
//!
//! High-performance string similarity algorithms achieving 63x speedup over Python.
//!
//! **Canonical implementation:** [`nexcore_edit_distance::classic`].
//! This module re-exports all public API for backward compatibility.

pub use nexcore_edit_distance::classic::{
    FuzzyMatch, LevenshteinResult, fuzzy_search, levenshtein, levenshtein_bounded,
    levenshtein_distance, levenshtein_similarity,
};
