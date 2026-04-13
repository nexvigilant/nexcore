//! # Perception Core: World Model & Arbitration
//!
//! Models and arbitrates uncertain information from multiple sources.

use serde::{Deserialize, Serialize};

/// Confidence weight used for source-level reliability.
pub type Confidence = f32;

/// TTL-stamped entity with uncertainty tagging.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity<T> {
    pub value: T,
    pub confidence: Confidence,
    pub last_updated_ms: u64,
    pub is_uncertain: bool,
}

/// Interface for arbitrating conflicts between sources.
pub trait ConflictArbitrator<T> {
    /// Merges multiple sources into a single entity, applying arbitration rules.
    fn arbitrate(&self, sources: &[Entity<T>]) -> Entity<T>;
}

/// Source Registry for confidence weight management (YAML microgram equivalent).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceRegistry {
    /// Confidence weights for registered data sources.
    pub source_weights: std::collections::HashMap<String, Confidence>,
}

impl SourceRegistry {
    /// Returns the registered weight or a conservative default.
    pub fn get_weight(&self, source_id: &str) -> Confidence {
        *self.source_weights.get(source_id).unwrap_or(&0.5)
    }
}
