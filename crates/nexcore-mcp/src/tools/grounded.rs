//! Grounded MCP tools: Epistemological substrate for all tool outputs.
//!
//! Provides uncertainty tracking, evidence chain propagation, and confidence
//! gating as universal primitives for grounded reasoning.
//!
//! ## T1 Primitive Grounding
//!
//! | Concept | Primitive | Symbol |
//! |---------|-----------|--------|
//! | Uncertain<T> | Product × Quantity × Boundary | ×, N, ∂ |
//! | Confidence | Quantity × Boundary | N, ∂ |
//! | ConfidenceBand | Sum | Σ |
//! | EvidenceChain | Sequence × Causality × Quantity | σ, →, N |
//! | Require gate | Boundary | ∂ |
//! | Compose | Mapping | μ |

use crate::params::grounded::{
    GroundedComposeParams, GroundedEvidenceGetParams, GroundedEvidenceNewParams,
    GroundedEvidenceStepParams, GroundedRequireParams, GroundedSkillAssessParams,
    GroundedUncertainParams,
};
use grounded::skill::SkillContext;
use grounded::{Confidence, EvidenceChain};
use parking_lot::Mutex;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::collections::HashMap;
use std::sync::LazyLock;

// Thread-safe evidence chain store (keyed by chain_id)
static EVIDENCE_CHAINS: LazyLock<Mutex<HashMap<String, EvidenceChain>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

// ============================================================================
// Tool Implementations
// ============================================================================

/// Wrap a value with confidence and get its band classification.
pub fn grounded_uncertain(params: GroundedUncertainParams) -> Result<CallToolResult, McpError> {
    let confidence = Confidence::new(params.confidence)
        .map_err(|e| McpError::invalid_params(format!("Invalid confidence: {e}"), None))?;

    let band = confidence.band();

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "value": params.value,
            "confidence": confidence.value(),
            "confidence_pct": format!("{confidence}"),
            "band": format!("{band}"),
            "provenance": params.provenance,
            "action_guidance": match band {
                grounded::ConfidenceBand::High => "Safe to act on directly",
                grounded::ConfidenceBand::Medium => "Act with fallback prepared",
                grounded::ConfidenceBand::Low => "Gather more evidence before acting",
                grounded::ConfidenceBand::Negligible => "Insufficient basis — do not act",
            }
        })
        .to_string(),
    )]))
}

/// Gate a value on minimum confidence — returns Ok or rejects with explanation.
pub fn grounded_require(params: GroundedRequireParams) -> Result<CallToolResult, McpError> {
    let confidence = Confidence::new(params.confidence)
        .map_err(|e| McpError::invalid_params(format!("Invalid confidence: {e}"), None))?;
    let min = Confidence::new(params.min_confidence)
        .map_err(|e| McpError::invalid_params(format!("Invalid min_confidence: {e}"), None))?;

    let passed = confidence >= min;

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "passed": passed,
            "value": if passed { Some(&params.value) } else { None },
            "confidence": confidence.value(),
            "min_confidence": min.value(),
            "band": format!("{}", confidence.band()),
            "gap": if passed { 0.0 } else { min.value() - confidence.value() },
            "verdict": if passed {
                "PROCEED — confidence meets threshold"
            } else {
                "BLOCKED — gather more evidence to close the gap"
            }
        })
        .to_string(),
    )]))
}

/// Compose two confidence values multiplicatively (P(A∧B) = P(A) × P(B)).
pub fn grounded_compose(params: GroundedComposeParams) -> Result<CallToolResult, McpError> {
    let a = Confidence::new(params.confidence_a)
        .map_err(|e| McpError::invalid_params(format!("Invalid confidence_a: {e}"), None))?;
    let b = Confidence::new(params.confidence_b)
        .map_err(|e| McpError::invalid_params(format!("Invalid confidence_b: {e}"), None))?;

    let composed = a.compose(b);
    let degradation = params.confidence_a.min(params.confidence_b) - composed.value();

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "confidence_a": a.value(),
            "confidence_b": b.value(),
            "composed": composed.value(),
            "composed_pct": format!("{composed}"),
            "band": format!("{}", composed.band()),
            "degradation": degradation,
            "label": params.label,
            "interpretation": format!(
                "{:.1}% × {:.1}% = {:.1}% — each hop costs confidence",
                a.value() * 100.0, b.value() * 100.0, composed.value() * 100.0
            )
        })
        .to_string(),
    )]))
}

/// Start a new evidence chain for a claim.
pub fn grounded_evidence_new(
    params: GroundedEvidenceNewParams,
) -> Result<CallToolResult, McpError> {
    let initial = Confidence::new(params.initial_confidence)
        .map_err(|e| McpError::invalid_params(format!("Invalid initial_confidence: {e}"), None))?;

    let chain = EvidenceChain::new(&params.claim, initial);
    let chain_id = format!("ev-{}", nexcore_chrono::DateTime::now().timestamp_millis());

    let mut chains = EVIDENCE_CHAINS.lock();
    chains.insert(chain_id.clone(), chain);

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "chain_id": chain_id,
            "claim": params.claim,
            "initial_confidence": initial.value(),
            "band": format!("{}", initial.band()),
            "steps": 1,
            "usage": "Call grounded_evidence_step to add supporting or opposing evidence"
        })
        .to_string(),
    )]))
}

/// Add a step to an existing evidence chain (strengthen or weaken).
pub fn grounded_evidence_step(
    params: GroundedEvidenceStepParams,
) -> Result<CallToolResult, McpError> {
    let factor = Confidence::new(params.factor)
        .map_err(|e| McpError::invalid_params(format!("Invalid factor: {e}"), None))?;

    let mut chains = EVIDENCE_CHAINS.lock();
    let chain = chains.get_mut(&params.chain_id).ok_or_else(|| {
        McpError::invalid_params(
            format!(
                "Unknown chain_id '{}'. Call grounded_evidence_new first.",
                params.chain_id
            ),
            None,
        )
    })?;

    let before = chain.confidence().value();

    match params.direction.to_lowercase().as_str() {
        "strengthen" | "support" | "+" => chain.strengthen(&params.description, factor),
        "weaken" | "oppose" | "-" => chain.weaken(&params.description, factor),
        other => {
            return Err(McpError::invalid_params(
                format!("Invalid direction '{other}'. Use 'strengthen' or 'weaken'."),
                None,
            ));
        }
    }

    let after = chain.confidence().value();

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "chain_id": params.chain_id,
            "claim": chain.claim,
            "step_added": params.description,
            "direction": params.direction,
            "factor": factor.value(),
            "confidence_before": before,
            "confidence_after": after,
            "delta": after - before,
            "band": format!("{}", chain.confidence().band()),
            "total_steps": chain.len(),
            "net_evidence": chain.net_evidence(),
            "total_support": chain.total_support(),
            "total_opposition": chain.total_opposition()
        })
        .to_string(),
    )]))
}

/// Get the full evidence chain with all steps and current confidence.
pub fn grounded_evidence_get(
    params: GroundedEvidenceGetParams,
) -> Result<CallToolResult, McpError> {
    let chains = EVIDENCE_CHAINS.lock();
    let chain = chains.get(&params.chain_id).ok_or_else(|| {
        McpError::invalid_params(
            format!(
                "Unknown chain_id '{}'. Call grounded_evidence_new first.",
                params.chain_id
            ),
            None,
        )
    })?;

    let steps: Vec<serde_json::Value> = chain
        .steps()
        .iter()
        .enumerate()
        .map(|(i, step)| {
            json!({
                "index": i,
                "description": step.description,
                "confidence": step.confidence.value(),
                "delta": step.delta,
                "recorded_at": step.recorded_at.to_rfc3339()
            })
        })
        .collect();

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "chain_id": params.chain_id,
            "claim": chain.claim,
            "current_confidence": chain.confidence().value(),
            "band": format!("{}", chain.confidence().band()),
            "steps": steps,
            "total_steps": chain.len(),
            "net_evidence": chain.net_evidence(),
            "total_support": chain.total_support(),
            "total_opposition": chain.total_opposition(),
            "formatted": format!("{chain}")
        })
        .to_string(),
    )]))
}

/// Run a grounded skill assessment (Bronze→Diamond compliance).
pub fn grounded_skill_assess(
    params: GroundedSkillAssessParams,
) -> Result<CallToolResult, McpError> {
    let ctx = SkillContext::new(&params.skill_path);
    let summary = ctx.summary();

    let checks_total = summary.checks_passed.len() + summary.checks_failed.len();
    let score_pct = if checks_total > 0 {
        (summary.checks_passed.len() as f64 / checks_total as f64) * 100.0
    } else {
        0.0
    };

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "skill_name": summary.skill_name,
            "skill_path": params.skill_path,
            "tier": summary.tier,
            "compliance_score": summary.compliance_score,
            "score_pct": format!("{score_pct:.0}%"),
            "checks_passed": summary.checks_passed,
            "checks_failed": summary.checks_failed,
            "total_checks": checks_total,
            "formatted": format!("{summary}")
        })
        .to_string(),
    )]))
}
