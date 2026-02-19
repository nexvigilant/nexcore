//! Engine-level statistics across all packs.

use serde::{Deserialize, Serialize};

use crate::knowledge_pack::PackIndex;
use crate::store::KnowledgeStore;

/// Overall engine statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineStats {
    pub packs: Vec<PackIndex>,
    pub total_packs: usize,
    pub total_fragments: usize,
    pub total_concepts: usize,
    pub avg_score: f64,
}

/// Compute engine-wide stats.
pub fn compute_stats(store: &KnowledgeStore) -> crate::error::Result<EngineStats> {
    let packs = store.list_packs()?;

    let total_packs = packs.len();
    let total_fragments: usize = packs.iter().map(|p| p.fragment_count).sum();
    let total_concepts: usize = packs.iter().map(|p| p.concept_count).sum();
    let avg_score = if total_packs > 0 {
        packs.iter().map(|p| p.avg_score).sum::<f64>() / total_packs as f64
    } else {
        0.0
    };

    Ok(EngineStats {
        packs,
        total_packs,
        total_fragments,
        total_concepts,
        avg_score,
    })
}

/// Compute stats for a single pack.
pub fn pack_stats(store: &KnowledgeStore, name: &str) -> crate::error::Result<EngineStats> {
    let pack = store.load_latest(name)?;
    let idx = pack.to_index();

    Ok(EngineStats {
        total_packs: 1,
        total_fragments: idx.fragment_count,
        total_concepts: idx.concept_count,
        avg_score: idx.avg_score,
        packs: vec![idx],
    })
}
