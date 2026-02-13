//! Scanner module - orchestrates primitive extraction.

use crate::extraction::ExtractionResult;
#[allow(unused_imports)]
use crate::types::{Primitive, PrimitiveTier, TermDefinition};

/// Primitive scanner for automated extraction.
#[derive(Debug, Default)]
pub struct Scanner {
    /// Minimum confidence threshold.
    pub min_confidence: f64,
}

impl Scanner {
    /// Create a new scanner.
    #[must_use]
    pub fn new() -> Self {
        Self {
            min_confidence: 0.5,
        }
    }

    /// Scan sources for primitives.
    pub fn scan(&self, _domain: &str, _sources: &[String]) -> ExtractionResult {
        ExtractionResult::default()
    }
}
