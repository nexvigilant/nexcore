//! Synapse MCP tools - Amplitude growth learning management
//!
//! Provides tools for managing synapses (amplitude-based learning units) that
//! implement the unified growth equation:
//!
//! ```text
//! α(t+1) = α(t)·e^(-λt) + η·confidence·relevance
//! ```
//!
//! ## T1 Primitive Grounding
//!
//! - ν (Frequency) → observation_count
//! - Σ (Sum) → amplitude accumulation
//! - π (Persistence) → file-backed storage
//! - ∂ (Boundary) → consolidation threshold
//! - ∝ (Irreversibility) → temporal decay

use nexcore_brain::{PersistentSynapseBank, SynapseInfo};
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;

use crate::params::{
    SynapseGetOrCreateParams, SynapseGetParams, SynapseListParams, SynapseObserveParams,
};

// ============================================================================
// Synapse Creation & Retrieval
// ============================================================================

/// Create or get a synapse by ID
pub fn synapse_get_or_create(params: SynapseGetOrCreateParams) -> Result<CallToolResult, McpError> {
    let mut bank =
        PersistentSynapseBank::load().map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let synapse = match params.synapse_type.as_str() {
        "pattern" => bank.get_or_create_for_pattern(&params.id),
        "preference" => bank.get_or_create_for_preference(&params.id),
        "belief" => bank.get_or_create_for_belief(&params.id),
        _ => bank.get_or_create_for_pattern(&params.id),
    };

    let info = SynapseInfo {
        id: synapse.id.clone(),
        amplitude: synapse.current_amplitude().value(),
        observation_count: synapse.observation_count(),
        status: synapse.status(),
        is_persistent: synapse.is_persistent(),
    };

    bank.save()
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let result = serde_json::json!({
        "success": true,
        "synapse": info,
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?,
    )]))
}

/// Get synapse information by ID
pub fn synapse_get(params: SynapseGetParams) -> Result<CallToolResult, McpError> {
    let bank =
        PersistentSynapseBank::load().map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let synapse = bank.get(&params.id);

    let result = match synapse {
        Some(s) => {
            let info = SynapseInfo {
                id: s.id.clone(),
                amplitude: s.current_amplitude().value(),
                observation_count: s.observation_count(),
                status: s.status(),
                is_persistent: s.is_persistent(),
            };
            serde_json::json!({
                "found": true,
                "synapse": info,
                "peak_amplitude": s.peak_amplitude().value(),
                "time_to_decay": s.time_to_decay().map(|d| d.as_secs()),
            })
        }
        None => {
            serde_json::json!({
                "found": false,
                "id": params.id,
            })
        }
    };

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?,
    )]))
}

// ============================================================================
// Synapse Observation (Learning)
// ============================================================================

/// Observe a learning signal on a synapse
pub fn synapse_observe(params: SynapseObserveParams) -> Result<CallToolResult, McpError> {
    let mut bank =
        PersistentSynapseBank::load().map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let amplitude = bank.observe(&params.id, params.confidence, params.relevance);

    let result = match amplitude {
        Some(amp) => {
            bank.save()
                .map_err(|e| McpError::internal_error(e.to_string(), None))?;

            let synapse = bank.get(&params.id);
            serde_json::json!({
                "success": true,
                "id": params.id,
                "amplitude": amp.value(),
                "observation_count": synapse.map(|s| s.observation_count()).unwrap_or(0),
                "status": synapse.map(|s| format!("{}", s.status())).unwrap_or_default(),
            })
        }
        None => {
            serde_json::json!({
                "success": false,
                "error": format!("Synapse '{}' not found. Create it first with synapse_get_or_create.", params.id),
            })
        }
    };

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?,
    )]))
}

// ============================================================================
// Synapse Listing & Statistics
// ============================================================================

/// List all synapses with optional filtering
pub fn synapse_list(params: SynapseListParams) -> Result<CallToolResult, McpError> {
    let bank =
        PersistentSynapseBank::load().map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let mut synapses = bank.list();

    // Apply filters
    if let Some(ref filter_type) = params.filter_type {
        let prefix = format!("{filter_type}:");
        synapses.retain(|s| s.id.starts_with(&prefix));
    }

    if params.consolidated_only {
        synapses.retain(|s| matches!(s.status, nexcore_synapse::ConsolidationStatus::Consolidated));
    }

    // Sort by amplitude descending
    synapses.sort_by(|a, b| {
        b.amplitude
            .partial_cmp(&a.amplitude)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let result = serde_json::json!({
        "count": synapses.len(),
        "synapses": synapses,
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?,
    )]))
}

/// Get synapse bank statistics
pub fn synapse_stats() -> Result<CallToolResult, McpError> {
    let bank =
        PersistentSynapseBank::load().map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let stats = bank.stats();

    let result = serde_json::json!({
        "total_synapses": stats.total_synapses,
        "consolidated_count": stats.consolidated_count,
        "accumulating_count": stats.accumulating_count,
        "pattern_synapses": stats.pattern_synapses,
        "preference_synapses": stats.preference_synapses,
        "belief_synapses": stats.belief_synapses,
        "average_amplitude": stats.average_amplitude,
        "peak_amplitude": stats.peak_amplitude,
        "last_saved": bank.last_saved,
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?,
    )]))
}

/// Prune decayed synapses (amplitude near zero)
pub fn synapse_prune() -> Result<CallToolResult, McpError> {
    let mut bank =
        PersistentSynapseBank::load().map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let before = bank.len();
    let pruned = bank.prune_decayed();

    bank.save()
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let result = serde_json::json!({
        "pruned_count": pruned,
        "before_count": before,
        "after_count": bank.len(),
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?,
    )]))
}
