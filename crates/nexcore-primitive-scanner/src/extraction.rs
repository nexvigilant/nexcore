//! Extraction context and results.

use crate::types::Primitive;
use serde::{Deserialize, Serialize};

/// Extraction context.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ExtractionContext {
    /// Domain name.
    pub domain: String,
    /// Source mode.
    pub source_mode: SourceMode,
}

/// Source mode for extraction.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SourceMode {
    /// Full corpus provided.
    Full,
    /// Partial corpus.
    Partial,
    /// Expert generation (no corpus).
    #[default]
    Expert,
    /// Hybrid with web fetch.
    Hybrid,
}

/// Extraction result.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ExtractionResult {
    /// Domain extracted from.
    pub domain: String,
    /// Extracted primitives.
    pub primitives: Vec<Primitive>,
    /// Source mode used.
    pub source_mode: SourceMode,
}
