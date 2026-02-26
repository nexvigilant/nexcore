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
    pub id: crate::KnowledgeId,
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PackStats {
    pub fragment_count: usize,
    pub concept_count: usize,
    pub edge_count: usize,
    pub avg_compendious_score: f64,
    pub compression_ratio: f64,
    pub domains: Vec<String>,
    pub total_words: usize,
    /// Number of raw sources attempted during compilation.
    #[serde(default)]
    pub sources_attempted: usize,
    /// Number of sources that failed ingestion (e.g., compressed to empty).
    #[serde(default)]
    pub sources_failed: usize,
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
            sources_attempted: 0, // Set by compiler
            sources_failed: 0,    // Set by compiler
        }
    }

    /// Look up a fragment by ID within this pack.
    ///
    /// Returns [`KnowledgeEngineError::FragmentNotFound`] if no match.
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use nexcore_knowledge_engine::knowledge_pack::KnowledgePack;
    /// use nexcore_knowledge_engine::concept_graph::ConceptGraph;
    ///
    /// let pack = KnowledgePack::new("test".to_string(), 1, vec![], ConceptGraph::new());
    /// assert!(pack.get_fragment("nonexistent").is_err());
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_fragment(&self, id: &str) -> crate::error::Result<&KnowledgeFragment> {
        self.fragments
            .iter()
            .find(|f| f.id == id)
            .ok_or_else(|| crate::error::KnowledgeEngineError::FragmentNotFound(id.to_string()))
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

    #[test]
    fn get_fragment_found() {
        use crate::ingest::{KnowledgeSource, RawKnowledge};
        use nexcore_chrono::DateTime;

        let raw = RawKnowledge {
            text: "Signal detection uses PRR for analysis.".to_string(),
            source: KnowledgeSource::FreeText,
            domain: Some("pv".to_string()),
            timestamp: DateTime::now(),
        };
        let frag = crate::ingest::ingest(raw).unwrap();
        let frag_id = frag.id.clone();
        let pack = KnowledgePack::new("test".to_string(), 1, vec![frag], ConceptGraph::new());

        assert!(pack.get_fragment(&frag_id).is_ok());
    }

    #[test]
    fn get_fragment_not_found() {
        let pack = KnowledgePack::new("test".to_string(), 1, vec![], ConceptGraph::new());
        let err = pack.get_fragment("nonexistent");
        assert!(err.is_err());
        assert!(matches!(
            err.unwrap_err(),
            crate::error::KnowledgeEngineError::FragmentNotFound(_)
        ));
    }
}
