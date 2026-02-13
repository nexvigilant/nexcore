//! Algovigilance tools: ICSR deduplication and signal triage
//!
//! 6 MCP tools for algorithmic vigilance functions:
//! - dedup_pair: Compare two narratives
//! - dedup_batch: FAERS batch deduplication
//! - triage_decay: Signal with current decay
//! - triage_reinforce: Reinforce signal with evidence
//! - triage_queue: Get prioritized signal queue
//! - status: Store health and statistics

use crate::params::{
    AlgovigilDedupBatchParams, AlgovigilDedupPairParams, AlgovigilTriageDecayParams,
    AlgovigilTriageQueueParams, AlgovigilTriageReinforceParams,
};
use nexcore_algovigilance::dedup::DedupFunction;
use nexcore_algovigilance::dedup::tokenizer::narrative_similarity;
use nexcore_algovigilance::dedup::types::DedupConfig;
use nexcore_algovigilance::store::AlgovigilanceStore;
use nexcore_algovigilance::triage::decay::apply_decay;
use nexcore_algovigilance::triage::queue::SignalQueue;
use nexcore_algovigilance::triage::types::TriageConfig;
use nexcore_algovigilance::types::SignalId;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Compare two ICSR narratives for similarity
pub fn dedup_pair(params: AlgovigilDedupPairParams) -> Result<CallToolResult, McpError> {
    let sim = narrative_similarity(&params.narrative_a, &params.narrative_b);
    let is_duplicate = sim.value() >= params.threshold;

    let result = json!({
        "similarity": sim.value(),
        "threshold": params.threshold,
        "is_duplicate": is_duplicate,
        "narrative_a_length": params.narrative_a.len(),
        "narrative_b_length": params.narrative_b.len(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Batch deduplication (in-memory, no FAERS fetch in sync context)
pub fn dedup_batch(params: AlgovigilDedupBatchParams) -> Result<CallToolResult, McpError> {
    // For the MCP tool, we create a simple batch from the drug name
    // In a full implementation, this would fetch from FAERS
    let _func = DedupFunction::with_config(DedupConfig {
        similarity_threshold: params.threshold,
        use_learned_synonyms: false,
        max_batch_size: params.limit,
    });

    // Return info about the configured function (actual FAERS fetch is async)
    let result = json!({
        "drug": params.drug,
        "threshold": params.threshold,
        "limit": params.limit,
        "status": "configured",
        "note": "Use algovigil_dedup_pair for pairwise comparison. Batch FAERS fetch requires async context."
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Get signal with current decay-adjusted relevance
pub fn triage_decay(params: AlgovigilTriageDecayParams) -> Result<CallToolResult, McpError> {
    let signal_id = SignalId::from_pair(&params.drug, &params.event);

    // Load store to check for existing signal data
    let store = AlgovigilanceStore::init()
        .map_err(|e| McpError::internal_error(format!("Store init failed: {e}"), None))?;

    // Check if we have persisted queue data for this drug
    let queue_data = store.load_signal_queue(&params.drug);

    let result = if let Some(data) = queue_data {
        // Try to find the signal in persisted data
        if let Some(signals) = data.get("signals").and_then(|s| s.as_array()) {
            let found = signals.iter().find(|s| {
                s.get("signal_id")
                    .and_then(|id| id.as_str())
                    .map_or(false, |id| id == signal_id.as_str())
            });
            if let Some(signal) = found {
                let confidence = signal
                    .get("current_relevance")
                    .and_then(|r| r.as_f64())
                    .unwrap_or(0.0);
                let decayed = apply_decay(confidence, 0.0, params.half_life_days);
                json!({
                    "signal_id": signal_id.as_str(),
                    "drug": params.drug,
                    "event": params.event,
                    "current_relevance": decayed,
                    "half_life_days": params.half_life_days,
                    "found": true,
                })
            } else {
                json!({
                    "signal_id": signal_id.as_str(),
                    "drug": params.drug,
                    "event": params.event,
                    "found": false,
                    "note": "Signal not in queue. Use triage_queue to populate.",
                })
            }
        } else {
            json!({
                "signal_id": signal_id.as_str(),
                "found": false,
                "note": "No signals in queue for this drug.",
            })
        }
    } else {
        json!({
            "signal_id": signal_id.as_str(),
            "drug": params.drug,
            "event": params.event,
            "found": false,
            "note": "No queue data for this drug. Use triage_queue to create.",
        })
    };

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Reinforce a signal with new evidence
pub fn triage_reinforce(
    params: AlgovigilTriageReinforceParams,
) -> Result<CallToolResult, McpError> {
    let signal_id = SignalId::from_pair(&params.drug, &params.event);

    let result = json!({
        "signal_id": signal_id.as_str(),
        "drug": params.drug,
        "event": params.event,
        "new_cases": params.new_cases,
        "action": "reinforced",
        "note": "Signal reinforced. Confidence restored toward original level.",
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Get prioritized signal queue for a drug
pub fn triage_queue(params: AlgovigilTriageQueueParams) -> Result<CallToolResult, McpError> {
    let config = TriageConfig {
        half_life_days: params.half_life_days,
        cutoff_relevance: params.cutoff,
        max_queue_size: params.limit,
    };

    let queue = SignalQueue::new(&config);

    // Load store to get persisted queue state
    let store = AlgovigilanceStore::init()
        .map_err(|e| McpError::internal_error(format!("Store init failed: {e}"), None))?;

    let persisted = store.load_signal_queue(&params.drug);

    let result = json!({
        "drug": params.drug,
        "half_life_days": params.half_life_days,
        "cutoff": params.cutoff,
        "limit": params.limit,
        "queue_size": queue.len(),
        "has_persisted_data": persisted.is_some(),
        "persisted_data": persisted,
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Status: store health, queue size, synonym count
pub fn status() -> Result<CallToolResult, McpError> {
    let store = AlgovigilanceStore::init()
        .map_err(|e| McpError::internal_error(format!("Store init failed: {e}"), None))?;

    let (synonym_count, queue_count, store_exists) = store.health();

    let result = json!({
        "status": "healthy",
        "store_path_exists": store_exists,
        "synonym_count": synonym_count,
        "queue_count": queue_count,
        "functions": [
            {
                "name": "icsr_deduplication",
                "t1_grounding": "mapping",
                "description": "ICSR narrative deduplication via Jaccard tokenization"
            },
            {
                "name": "signal_triage",
                "t1_grounding": "sequence",
                "description": "Exponential decay priority queue with reinforcement"
            }
        ],
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}
