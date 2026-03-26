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
    // Fragment-weighted average — not average of averages.
    // A pack with 100 fragments contributes 100x more than a pack with 1.
    let avg_score = if total_fragments > 0 {
        packs
            .iter()
            .map(|p| p.avg_score * p.fragment_count as f64)
            .sum::<f64>()
            / total_fragments as f64
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::{CompileOptions, KnowledgeCompiler};
    use crate::ingest::{KnowledgeSource, RawKnowledge};
    use nexcore_chrono::DateTime;

    fn make_store_with_pack(pack_name: &str) -> KnowledgeStore {
        let store = KnowledgeStore::temp().unwrap();
        let compiler = KnowledgeCompiler::new(store.clone());
        let options = CompileOptions {
            name: pack_name.to_string(),
            include_distillations: false,
            include_artifacts: false,
            include_implicit: false,
            include_staged: false,
            sources: vec![RawKnowledge {
                text: "Signal detection uses PRR for pharmacovigilance safety analysis."
                    .to_string(),
                source: KnowledgeSource::FreeText,
                domain: Some("pv".to_string()),
                timestamp: DateTime::now(),
            }],
        };
        compiler.compile(options).unwrap();
        store
    }

    #[test]
    fn compute_stats_empty_store() {
        let store = KnowledgeStore::temp().unwrap();
        let stats = compute_stats(&store).unwrap();
        assert_eq!(stats.total_packs, 0);
        assert_eq!(stats.total_fragments, 0);
        assert_eq!(stats.avg_score, 0.0);
    }

    #[test]
    fn compute_stats_with_pack() {
        let store = make_store_with_pack("stats-test");
        let stats = compute_stats(&store).unwrap();
        assert_eq!(stats.total_packs, 1);
        // The pack contains 1 fragment (one source text)
        assert_eq!(stats.total_fragments, 1);
        // avg_score must be positive — the fragment has actual content
        assert!(
            stats.avg_score > 0.0,
            "avg_score should be positive, got {}",
            stats.avg_score
        );
        assert_eq!(stats.packs.len(), 1);
        assert_eq!(stats.packs[0].name, "stats-test");
    }

    #[test]
    fn pack_stats_specific_pack() {
        let store = make_store_with_pack("specific-pack");
        let stats = pack_stats(&store, "specific-pack").unwrap();
        assert_eq!(stats.total_packs, 1);
        assert_eq!(stats.total_fragments, 1);
        assert_eq!(stats.packs[0].name, "specific-pack");
    }

    #[test]
    fn pack_stats_nonexistent_returns_error() {
        let store = KnowledgeStore::temp().unwrap();
        assert!(pack_stats(&store, "does-not-exist").is_err());
    }
}
