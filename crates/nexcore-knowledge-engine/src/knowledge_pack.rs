//! Knowledge pack — the compiled, immutable output format.
//!
//! Versioned, serializable, queryable. Follows Brain's `.resolved.N` pattern.

use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};

use crate::concept_graph::ConceptGraph;
use crate::ingest::KnowledgeFragment;

/// A compiled knowledge pack.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgePack {
    pub id: String,
    pub name: String,
    pub version: u32,
    pub fragments: Vec<KnowledgeFragment>,
    pub concept_graph: ConceptGraph,
    pub stats: PackStats,
    pub created_at: DateTime,
}

/// Index entry for a pack (lightweight, for listing).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackIndex {
    pub name: String,
    pub version: u32,
    pub fragment_count: usize,
    pub concept_count: usize,
    pub avg_score: f64,
    pub created_at: DateTime,
}

/// Statistics for a knowledge pack.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackStats {
    pub fragment_count: usize,
    pub concept_count: usize,
    pub edge_count: usize,
    pub avg_compendious_score: f64,
    pub compression_ratio: f64,
    pub domains: Vec<String>,
    pub total_words: usize,
}

impl KnowledgePack {
    /// Create from fragments and concept graph.
    pub fn new(
        name: String,
        version: u32,
        fragments: Vec<KnowledgeFragment>,
        concept_graph: ConceptGraph,
    ) -> Self {
        let stats = Self::compute_stats(&fragments, &concept_graph);
        Self {
            id: nexcore_id::NexId::v4().to_string(),
            name,
            version,
            fragments,
            concept_graph,
            stats,
            created_at: DateTime::now(),
        }
    }

    fn compute_stats(fragments: &[KnowledgeFragment], graph: &ConceptGraph) -> PackStats {
        let fragment_count = fragments.len();
        let concept_count = graph.node_count();
        let edge_count = graph.edge_count();

        let avg_compendious_score = if fragment_count > 0 {
            fragments
                .iter()
                .map(|f| f.score.compendious_score)
                .sum::<f64>()
                / fragment_count as f64
        } else {
            0.0
        };

        let total_words: usize = fragments.iter().map(|f| f.score.expression_cost).sum();

        let domains: Vec<String> = {
            let mut d: Vec<String> = graph.domains().into_iter().collect();
            d.sort();
            d
        };

        PackStats {
            fragment_count,
            concept_count,
            edge_count,
            avg_compendious_score,
            compression_ratio: 0.0, // Set by compiler after compression
            domains,
            total_words,
        }
    }

    /// Build a PackIndex from this pack.
    pub fn to_index(&self) -> PackIndex {
        PackIndex {
            name: self.name.clone(),
            version: self.version,
            fragment_count: self.stats.fragment_count,
            concept_count: self.stats.concept_count,
            avg_score: self.stats.avg_compendious_score,
            created_at: self.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_pack() {
        let pack = KnowledgePack::new("test".to_string(), 1, vec![], ConceptGraph::new());
        assert_eq!(pack.stats.fragment_count, 0);
        assert_eq!(pack.stats.avg_compendious_score, 0.0);
    }

    #[test]
    fn pack_index() {
        let pack = KnowledgePack::new("test".to_string(), 3, vec![], ConceptGraph::new());
        let idx = pack.to_index();
        assert_eq!(idx.name, "test");
        assert_eq!(idx.version, 3);
    }
}
