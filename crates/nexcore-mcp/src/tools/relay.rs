//! # Relay Fidelity MCP Tools
//!
//! Expose relay chain creation, verification, and fidelity computation
//! via MCP for real-time pipeline health monitoring.

use nexcore_primitives::relay::{Fidelity, RelayChain, RelayHop};
use signal::relay as pipeline_relay;
use rmcp::model::{CallToolResult, Content};
use rmcp::ErrorData as McpError;

use crate::params::relay::{RelayChainComputeParams, RelayFidelityComposeParams};

/// Build a relay chain from user-provided hops, run A1-A5 verification.
pub fn relay_chain_verify(params: RelayChainComputeParams) -> Result<CallToolResult, McpError> {
    let mut chain = RelayChain::new(params.f_min);

    for hop in &params.hops {
        if hop.activated {
            chain.add_hop(RelayHop::new(
                &hop.stage,
                Fidelity::new(hop.fidelity),
                hop.threshold,
            ));
        } else {
            chain.add_hop(RelayHop::inactive(&hop.stage, hop.threshold));
        }
    }

    let verification = chain.verify();
    let weakest = chain
        .weakest_hop()
        .map(|h| {
            serde_json::json!({
                "stage": h.stage,
                "fidelity": h.fidelity.value(),
            })
        })
        .unwrap_or(serde_json::json!(null));

    let json = serde_json::json!({
        "total_fidelity": chain.total_fidelity().value(),
        "signal_loss_pct": chain.signal_loss() * 100.0,
        "active_hops": chain.active_hop_count(),
        "total_hops": chain.hop_count(),
        "f_min": params.f_min,
        "axioms": {
            "a1_directionality": verification.a1_directionality,
            "a2_mediation": verification.a2_mediation,
            "a3_preservation": verification.a3_preservation,
            "a4_threshold": verification.a4_threshold,
            "a5_boundedness": verification.a5_boundedness,
        },
        "axioms_passing": verification.axioms_passing(),
        "is_valid": verification.is_valid(),
        "weakest_hop": weakest,
        "hops": params.hops.iter().map(|h| serde_json::json!({
            "stage": h.stage,
            "fidelity": h.fidelity,
            "threshold": h.threshold,
            "activated": h.activated,
        })).collect::<Vec<_>>(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&json).unwrap_or_default(),
    )]))
}

/// Get the pre-configured PV signal pipeline relay chain with verification.
pub fn relay_pv_pipeline() -> Result<CallToolResult, McpError> {
    let chain = pipeline_relay::pv_pipeline_chain();
    let verification = chain.verify();

    let hops: Vec<_> = chain
        .hops()
        .iter()
        .map(|h| {
            serde_json::json!({
                "stage": h.stage,
                "fidelity": h.fidelity.value(),
                "threshold": h.threshold,
                "activated": h.activated,
            })
        })
        .collect();

    let json = serde_json::json!({
        "pipeline": "pv_signal_detection",
        "description": "Full 7-stage PV signal pipeline: ingest → normalize → detect → threshold → store → alert → report",
        "total_fidelity": chain.total_fidelity().value(),
        "signal_loss_pct": chain.signal_loss() * 100.0,
        "f_min": 0.80,
        "passes_safety_critical": chain.verify_preservation(),
        "axioms": {
            "a1_directionality": verification.a1_directionality,
            "a2_mediation": verification.a2_mediation,
            "a3_preservation": verification.a3_preservation,
            "a4_threshold": verification.a4_threshold,
            "a5_boundedness": verification.a5_boundedness,
        },
        "is_valid": verification.is_valid(),
        "weakest_stage": chain.weakest_hop().map(|h| h.stage.as_str()).unwrap_or("none"),
        "hops": hops,
        "recommendation": if chain.verify_preservation() {
            "Pipeline meets safety-critical fidelity threshold."
        } else {
            "WARNING: Pipeline fails safety-critical fidelity. Consider reducing hops or improving weakest stage."
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&json).unwrap_or_default(),
    )]))
}

/// Get the core 4-hop detection chain.
pub fn relay_core_detection() -> Result<CallToolResult, McpError> {
    let chain = pipeline_relay::core_detection_chain();
    let verification = chain.verify();

    let hops: Vec<_> = chain
        .hops()
        .iter()
        .map(|h| {
            serde_json::json!({
                "stage": h.stage,
                "fidelity": h.fidelity.value(),
                "threshold": h.threshold,
                "activated": h.activated,
            })
        })
        .collect();

    let json = serde_json::json!({
        "pipeline": "core_detection",
        "description": "Core 4-stage detection: ingest → detect → threshold → alert",
        "total_fidelity": chain.total_fidelity().value(),
        "signal_loss_pct": chain.signal_loss() * 100.0,
        "f_min": 0.80,
        "passes_safety_critical": chain.verify_preservation(),
        "is_valid": verification.is_valid(),
        "weakest_stage": chain.weakest_hop().map(|h| h.stage.as_str()).unwrap_or("none"),
        "hops": hops,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&json).unwrap_or_default(),
    )]))
}

/// Compose fidelity values multiplicatively.
pub fn relay_fidelity_compose(
    params: RelayFidelityComposeParams,
) -> Result<CallToolResult, McpError> {
    let mut composed = Fidelity::PERFECT;
    for &v in &params.values {
        composed = composed.compose(Fidelity::new(v));
    }

    let json = serde_json::json!({
        "input_values": params.values,
        "composed_fidelity": composed.value(),
        "signal_loss_pct": composed.loss() * 100.0,
        "hop_count": params.values.len(),
        "meets_safety_critical": composed.meets_minimum(0.80),
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&json).unwrap_or_default(),
    )]))
}
