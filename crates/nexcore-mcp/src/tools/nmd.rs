// Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! NMD (Nonsense-Mediated mRNA Decay) surveillance MCP tools.
//!
//! Exposes the full NMD pipeline as MCP tools:
//! - `nmd_check`: Full pipeline (UPF → Thymic → SMG → Adaptive)
//! - `nmd_upf_evaluate`: Raw UPF structural check
//! - `nmd_smg_process`: SMG verdict → actions
//! - `nmd_adaptive_stats`: Per-category degradation statistics
//! - `nmd_thymic_status`: Thymic graduation status
//! - `nmd_status`: Full pipeline health overview
//!
//! ## Primitive Grounding
//!
//! | Tool | T1 Primitives |
//! |------|---------------|
//! | nmd_check | ∂ (boundary) + ν (frequency) + ς (state) + → (causality) |
//! | nmd_upf_evaluate | κ (comparison) + σ (sequence) |
//! | nmd_smg_process | → (causality) + μ (mapping) |
//! | nmd_adaptive_stats | N (quantity) + ν (frequency) |
//! | nmd_thymic_status | ∂ (boundary) + ς (state) |
//! | nmd_status | Σ (sum) + ς (state) |

use crate::params::{
    NmdAdaptiveStatsParams, NmdAnomalyInput, NmdCheckParams, NmdSmgProcessParams,
    NmdThymicStatusParams, NmdUpfEvaluateParams,
};
use nexcore_immunity::adaptive::NmdAdaptiveEngine;
use nexcore_immunity::{
    CheckpointObservation, EjcMarker, SmgComplex, SmgConfig, TaskCategory, ThymicGate, UpfAnomaly,
    UpfChannel, UpfComplex, UpfVerdict,
};
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde_json::json;
use std::sync::{Mutex, OnceLock};

/// Shared NMD state container.
struct NmdState {
    upf: UpfComplex,
    thymic: ThymicGate,
    smg: SmgComplex,
    adaptive: NmdAdaptiveEngine,
}

/// Global NMD state — persists across tool calls within a session.
static NMD_STATE: OnceLock<Mutex<NmdState>> = OnceLock::new();

/// Get or initialize the NMD state.
fn get_nmd_state() -> &'static Mutex<NmdState> {
    NMD_STATE.get_or_init(|| {
        Mutex::new(NmdState {
            upf: UpfComplex::new(),
            thymic: ThymicGate::new(),
            smg: SmgComplex::new(),
            adaptive: NmdAdaptiveEngine::new(),
        })
    })
}

/// Parse a category string into TaskCategory enum.
fn parse_category(s: &str) -> TaskCategory {
    match s.to_lowercase().as_str() {
        "explore" => TaskCategory::Explore,
        "mutate" => TaskCategory::Mutate,
        "compute" => TaskCategory::Compute,
        "orchestrate" => TaskCategory::Orchestrate,
        "verify" => TaskCategory::Verify,
        "browse" => TaskCategory::Browse,
        _ => TaskCategory::Mixed,
    }
}

/// Parse channel string into UpfChannel.
fn parse_channel(s: &str) -> UpfChannel {
    if s.starts_with("UPF1") {
        UpfChannel::Upf1
    } else if s.starts_with("UPF2") {
        UpfChannel::Upf2
    } else {
        UpfChannel::Upf3
    }
}

/// Convert anomaly inputs to UPF anomalies.
fn to_upf_anomalies(inputs: &[NmdAnomalyInput]) -> Vec<UpfAnomaly> {
    inputs
        .iter()
        .map(|a| UpfAnomaly {
            channel: parse_channel(&a.channel),
            description: a.description.clone(),
            severity: a.severity,
        })
        .collect()
}

/// Build a default EjcMarker from observation data.
///
/// When the caller doesn't provide explicit markers, we create one
/// from the observed categories as the expected set (self-referential
/// baseline). UPF2 tool drift will only fire if future observations diverge.
fn build_default_marker(observation: &CheckpointObservation) -> EjcMarker {
    EjcMarker {
        phase_id: observation.phase_id.clone(),
        expected_tool_categories: observation.observed_categories.clone(),
        grounding_confidence_threshold: if observation.total_calls > 0 {
            observation.grounding_signals as f32 / observation.total_calls as f32
        } else {
            0.0
        },
        max_calls_before_checkpoint: observation.total_calls.max(50),
        expected_confidence_range: (0.3, 0.9),
        skippable: false,
    }
}

/// Serialize a verdict to JSON.
fn verdict_to_json(verdict: &UpfVerdict) -> serde_json::Value {
    match verdict {
        UpfVerdict::Continue => json!({"type": "continue"}),
        UpfVerdict::Stall { anomalies } => json!({
            "type": "stall",
            "anomalies": anomalies.iter().map(|a| json!({
                "channel": a.channel.to_string(),
                "description": &a.description,
                "severity": a.severity,
            })).collect::<Vec<_>>(),
        }),
        UpfVerdict::Degrade { anomalies } => json!({
            "type": "degrade",
            "anomalies": anomalies.iter().map(|a| json!({
                "channel": a.channel.to_string(),
                "description": &a.description,
                "severity": a.severity,
            })).collect::<Vec<_>>(),
        }),
    }
}

/// Full NMD pipeline check: UPF → Thymic → SMG → Adaptive.
///
/// The primary tool — runs the entire surveillance chain and returns
/// the filtered verdict, actions, and learning events.
pub fn nmd_check(params: NmdCheckParams) -> Result<CallToolResult, McpError> {
    let state = get_nmd_state();
    let mut nmd = state
        .lock()
        .map_err(|e| McpError::internal_error(format!("NMD lock poisoned: {e}"), None))?;

    // Build observation
    let observation = CheckpointObservation {
        phase_id: params.phase_id.clone(),
        observed_categories: params
            .observed_categories
            .iter()
            .map(|c| parse_category(c))
            .collect(),
        grounding_signals: params.grounding_signals,
        total_calls: params.total_calls,
        checkpoint_index: params.checkpoint_index,
    };

    // Build default marker for this observation
    let markers = vec![build_default_marker(&observation)];

    // Stage 1: UPF evaluation
    let raw_verdict = nmd.upf.scan_checkpoint(&observation, &markers);

    // Stage 2: Thymic filter
    let filtered_verdict = nmd
        .thymic
        .filter_verdict(&params.category, raw_verdict.clone());

    // Stage 3: SMG process
    let actions = nmd.smg.process_verdict(&filtered_verdict);

    // Stage 4: Adaptive record
    let mut learning_events = Vec::new();
    let is_degradation = matches!(filtered_verdict, UpfVerdict::Degrade { .. });
    if is_degradation {
        for action in &actions {
            let events = nmd.adaptive.process_adaptive_action(action);
            learning_events.extend(events);
        }
    } else {
        nmd.adaptive.record_success(&params.category);
    }

    let graduated = nmd.thymic.is_graduated(&params.category);
    let degrade_rate = nmd.adaptive.degradation_rate(&params.category);

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        json!({
            "category": &params.category,
            "phase_id": &params.phase_id,
            "raw_verdict": verdict_to_json(&raw_verdict),
            "filtered_verdict": verdict_to_json(&filtered_verdict),
            "thymic_graduated": graduated,
            "actions": actions.iter().map(|a| format!("{a:?}")).collect::<Vec<_>>(),
            "learning_events": learning_events.len(),
            "degradation_rate": degrade_rate,
        })
        .to_string(),
    )]))
}

/// Raw UPF structural check only (no thymic/SMG/adaptive).
pub fn nmd_upf_evaluate(params: NmdUpfEvaluateParams) -> Result<CallToolResult, McpError> {
    let state = get_nmd_state();
    let nmd = state
        .lock()
        .map_err(|e| McpError::internal_error(format!("NMD lock poisoned: {e}"), None))?;

    let observation = CheckpointObservation {
        phase_id: params.phase_id,
        observed_categories: params
            .observed_categories
            .iter()
            .map(|c| parse_category(c))
            .collect(),
        grounding_signals: params.grounding_signals,
        total_calls: params.total_calls,
        checkpoint_index: params.checkpoint_index,
    };

    let markers = vec![build_default_marker(&observation)];
    let verdict = nmd.upf.scan_checkpoint(&observation, &markers);

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        verdict_to_json(&verdict).to_string(),
    )]))
}

/// Process a verdict through the SMG degradation complex.
pub fn nmd_smg_process(params: NmdSmgProcessParams) -> Result<CallToolResult, McpError> {
    let state = get_nmd_state();
    let nmd = state
        .lock()
        .map_err(|e| McpError::internal_error(format!("NMD lock poisoned: {e}"), None))?;

    let verdict = match params.verdict.to_lowercase().as_str() {
        "continue" => UpfVerdict::Continue,
        "stall" => UpfVerdict::Stall {
            anomalies: to_upf_anomalies(&params.anomalies),
        },
        "degrade" => UpfVerdict::Degrade {
            anomalies: to_upf_anomalies(&params.anomalies),
        },
        other => {
            return Err(McpError::invalid_params(
                format!("Unknown verdict type: {other}. Expected: continue, stall, degrade"),
                None,
            ));
        }
    };

    let actions = nmd.smg.process_verdict(&verdict);

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        json!({
            "verdict": &params.verdict,
            "actions": actions.iter().map(|a| format!("{a:?}")).collect::<Vec<_>>(),
            "action_count": actions.len(),
        })
        .to_string(),
    )]))
}

/// Get adaptive engine statistics.
pub fn nmd_adaptive_stats(params: NmdAdaptiveStatsParams) -> Result<CallToolResult, McpError> {
    let state = get_nmd_state();
    let nmd = state
        .lock()
        .map_err(|e| McpError::internal_error(format!("NMD lock poisoned: {e}"), None))?;

    if let Some(ref category) = params.category {
        let rate = nmd.adaptive.degradation_rate(category);
        let stats = nmd.adaptive.category_stats(category);
        let stats_json = stats
            .map(|s| {
                json!({
                    "total_runs": s.total_runs,
                    "degradation_count": s.degradation_count,
                    "avg_severity": s.avg_severity,
                    "max_severity": s.max_severity,
                    "channel_hits": s.channel_hits,
                })
            })
            .unwrap_or_else(|| json!(null));

        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            json!({
                "category": category,
                "degradation_rate": rate,
                "stats": stats_json,
                "total_events": nmd.adaptive.total_events(),
            })
            .to_string(),
        )]))
    } else {
        let all = nmd.adaptive.all_stats();
        let categories: serde_json::Value = all
            .iter()
            .map(|(k, s)| {
                (
                    k.clone(),
                    json!({
                        "total_runs": s.total_runs,
                        "degradation_count": s.degradation_count,
                        "avg_severity": s.avg_severity,
                        "max_severity": s.max_severity,
                        "degradation_rate": nmd.adaptive.degradation_rate(k),
                    }),
                )
            })
            .collect::<serde_json::Map<String, serde_json::Value>>()
            .into();

        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            json!({
                "categories": categories,
                "total_events": nmd.adaptive.total_events(),
            })
            .to_string(),
        )]))
    }
}

/// Get thymic graduation status.
pub fn nmd_thymic_status(params: NmdThymicStatusParams) -> Result<CallToolResult, McpError> {
    let state = get_nmd_state();
    let nmd = state
        .lock()
        .map_err(|e| McpError::internal_error(format!("NMD lock poisoned: {e}"), None))?;

    if let Some(ref category) = params.category {
        let graduated = nmd.thymic.is_graduated(category);
        let obs = nmd.thymic.observation(category);
        let obs_json = obs
            .map(|o| {
                json!({
                    "runs_observed": o.runs_observed,
                    "suppressed_degrades": o.suppressed_degrades,
                    "stalls_observed": o.stalls_observed,
                    "graduated": o.graduated,
                })
            })
            .unwrap_or_else(|| json!({"graduated": false, "runs_observed": 0}));

        let suppress_rate = nmd.thymic.suppressed_degrade_rate(category);

        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            json!({
                "category": category,
                "graduated": graduated,
                "observation": obs_json,
                "suppressed_degrade_rate": suppress_rate,
            })
            .to_string(),
        )]))
    } else {
        let all = nmd.thymic.all_observations();
        let categories: serde_json::Value = all
            .iter()
            .map(|(k, o)| {
                (
                    k.clone(),
                    json!({
                        "runs_observed": o.runs_observed,
                        "suppressed_degrades": o.suppressed_degrades,
                        "stalls_observed": o.stalls_observed,
                        "graduated": o.graduated,
                        "suppressed_degrade_rate": nmd.thymic.suppressed_degrade_rate(k),
                    }),
                )
            })
            .collect::<serde_json::Map<String, serde_json::Value>>()
            .into();

        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            json!({
                "categories": categories,
            })
            .to_string(),
        )]))
    }
}

/// Full NMD pipeline health overview.
pub fn nmd_status() -> Result<CallToolResult, McpError> {
    let state = get_nmd_state();
    let nmd = state
        .lock()
        .map_err(|e| McpError::internal_error(format!("NMD lock poisoned: {e}"), None))?;

    let thymic_obs = nmd.thymic.all_observations();
    let graduated_count = thymic_obs.values().filter(|o| o.graduated).count();
    let total_categories = thymic_obs.len();

    let adaptive_stats = nmd.adaptive.all_stats();
    let total_runs: u32 = adaptive_stats.values().map(|s| s.total_runs).sum();
    let total_degradations: u32 = adaptive_stats.values().map(|s| s.degradation_count).sum();

    let overall_degrade_rate = if total_runs == 0 {
        0.0
    } else {
        total_degradations as f64 / total_runs as f64
    };

    let health = if overall_degrade_rate < 0.1 {
        "healthy"
    } else if overall_degrade_rate < 0.3 {
        "elevated"
    } else {
        "critical"
    };

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        json!({
            "nmd_pipeline": {
                "upf_channels": ["UPF1-PhaseOrder", "UPF2-ToolDrift", "UPF3-Grounding"],
                "thymic_observation_window": 20,
                "smg_actions": ["AbortPipeline", "FlagSource", "AdaptiveUpdate"],
            },
            "thymic": {
                "total_categories": total_categories,
                "graduated": graduated_count,
                "in_observation": total_categories - graduated_count,
            },
            "adaptive": {
                "total_events": nmd.adaptive.total_events(),
                "total_runs": total_runs,
                "total_degradations": total_degradations,
                "overall_degradation_rate": overall_degrade_rate,
                "categories_tracked": adaptive_stats.len(),
            },
            "health": health,
        })
        .to_string(),
    )]))
}
