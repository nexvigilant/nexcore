//! MCP tools for The Foundry assembly-line architecture.
//!
//! Exposes foundry types as callable tools: artifact validation, cascade
//! checking, intelligence report rendering, VDAG ordering, and causal
//! inference.

use crate::params::foundry::{
    FoundryCascadeValidateParams, FoundryInferParams, FoundryRenderIntelligenceParams,
    FoundryValidateArtifactParams, FoundryVdagOrderParams,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

use nexcore_foundry::analyst::{IntelligenceReport, RiskLevel};
use nexcore_foundry::artifact::{ShippableArtifact, ValidatedDeliverable};
use nexcore_foundry::governance::CascadeValidation;
use nexcore_foundry::report::{render_intelligence_report, risk_level_label};
use nexcore_foundry::station::PipelineOrder;
use nexcore_reason::dag::{CausalDag, CausalLink, CausalNode, NodeId, NodeType};
use nexcore_reason::inference::InferenceEngine;

// ---------------------------------------------------------------------------
// foundry_validate_artifact
// ---------------------------------------------------------------------------

/// Validate a deliverable against The Foundry's B3 quality gate.
///
/// Returns the gate verdict (green/red), the full validation report, and
/// a shippable-artifact readiness flag.
pub fn foundry_validate_artifact(
    params: FoundryValidateArtifactParams,
) -> Result<CallToolResult, McpError> {
    let validated = ValidatedDeliverable {
        build_pass: params.build_pass,
        test_count: params.test_count,
        tests_passed: params.tests_passed,
        lint_pass: params.lint_pass,
        coverage_percent: params.coverage_percent,
        failures: params.failures,
    };

    let green = validated.is_green();
    let shippable = ShippableArtifact::from_validated(validated);

    let result = json!({
        "success": true,
        "gate_verdict": if green { "GREEN" } else { "RED" },
        "ready_to_ship": shippable.ready,
        "details": {
            "build_pass": params.build_pass,
            "tests": format!("{}/{}", params.tests_passed, params.test_count),
            "lint_pass": params.lint_pass,
            "coverage_percent": params.coverage_percent,
        }
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

// ---------------------------------------------------------------------------
// foundry_cascade_validate
// ---------------------------------------------------------------------------

/// Validate alignment of a SMART goal cascade.
///
/// Computes alignment percentage and reports whether the cascade is fully
/// aligned (all operational goals trace to strategic level).
pub fn foundry_cascade_validate(
    params: FoundryCascadeValidateParams,
) -> Result<CallToolResult, McpError> {
    if params.total_operational == 0 {
        return Err(McpError::invalid_params(
            "total_operational must be > 0".to_string(),
            None,
        ));
    }

    #[allow(clippy::cast_precision_loss)]
    let alignment_percent =
        (params.traced_to_strategic as f64 / params.total_operational as f64) * 100.0;

    let validation = CascadeValidation {
        total_operational_goals: params.total_operational,
        traced_to_team: params.traced_to_team,
        traced_to_strategic: params.traced_to_strategic,
        alignment_percent,
    };

    let result = json!({
        "success": true,
        "fully_aligned": validation.is_fully_aligned(),
        "alignment_percent": (alignment_percent * 100.0).round() / 100.0,
        "total_operational": params.total_operational,
        "traced_to_team": params.traced_to_team,
        "traced_to_strategic": params.traced_to_strategic,
        "gaps": {
            "untraced_to_team": params.total_operational.saturating_sub(params.traced_to_team),
            "untraced_to_strategic": params.total_operational.saturating_sub(params.traced_to_strategic),
        }
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

// ---------------------------------------------------------------------------
// foundry_render_intelligence
// ---------------------------------------------------------------------------

/// Render an A3 intelligence report as markdown.
pub fn foundry_render_intelligence(
    params: FoundryRenderIntelligenceParams,
) -> Result<CallToolResult, McpError> {
    let risk_level = match params.risk_level.to_lowercase().as_str() {
        "low" => RiskLevel::Low,
        "moderate" => RiskLevel::Moderate,
        "high" => RiskLevel::High,
        "critical" => RiskLevel::Critical,
        other => {
            return Err(McpError::invalid_params(
                format!("Invalid risk_level '{other}'. Expected: low, moderate, high, critical"),
                None,
            ));
        }
    };

    if !(0.0..=1.0).contains(&params.confidence) {
        return Err(McpError::invalid_params(
            format!(
                "confidence must be in [0.0, 1.0], got {}",
                params.confidence
            ),
            None,
        ));
    }

    let report = IntelligenceReport {
        findings: params.findings,
        recommendations: params.recommendations,
        risk_level,
        confidence: params.confidence,
    };

    let markdown = render_intelligence_report(&report);

    let result = json!({
        "success": true,
        "risk_level": risk_level_label(&report.risk_level),
        "confidence_pct": format!("{:.0}%", report.confidence * 100.0),
        "findings_count": report.findings.len(),
        "recommendations_count": report.recommendations.len(),
        "markdown": markdown,
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

// ---------------------------------------------------------------------------
// foundry_vdag_order
// ---------------------------------------------------------------------------

/// Return the VDAG pipeline ordering for a given variant.
pub fn foundry_vdag_order(params: FoundryVdagOrderParams) -> Result<CallToolResult, McpError> {
    let order = match params.variant.to_lowercase().as_str() {
        "full" => PipelineOrder::vdag_full(),
        "builder" => PipelineOrder::builder_sequence(),
        "analyst" => PipelineOrder::analyst_sequence(),
        other => {
            return Err(McpError::invalid_params(
                format!("Invalid variant '{other}'. Expected: full, builder, analyst"),
                None,
            ));
        }
    };

    let stage_names: Vec<String> = order.stages.iter().map(|s| format!("{s:?}")).collect();

    let result = json!({
        "success": true,
        "variant": params.variant,
        "stage_count": order.len(),
        "stages": stage_names,
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

// ---------------------------------------------------------------------------
// foundry_infer
// ---------------------------------------------------------------------------

/// Run causal inference over a user-provided DAG.
///
/// Builds a [`CausalDag`] from the input nodes and links, runs the
/// [`InferenceEngine`], and returns the resulting intelligence report.
pub fn foundry_infer(params: FoundryInferParams) -> Result<CallToolResult, McpError> {
    if params.nodes.is_empty() {
        return Err(McpError::invalid_params(
            "nodes must not be empty".to_string(),
            None,
        ));
    }

    let mut dag = CausalDag::new();

    for node in &params.nodes {
        let node_type = match node.node_type.to_lowercase().as_str() {
            "metric" => NodeType::Metric,
            "pattern" => NodeType::Pattern,
            "module" => NodeType::Module,
            "risk" => NodeType::Risk,
            "recommendation" => NodeType::Recommendation,
            other => {
                return Err(McpError::invalid_params(
                    format!(
                        "Invalid node_type '{other}' for node '{}'. Expected: metric, pattern, module, risk, recommendation",
                        node.id
                    ),
                    None,
                ));
            }
        };

        dag.add_node(CausalNode {
            id: NodeId::new(&node.id),
            label: node.label.clone(),
            node_type,
        });
    }

    for link in &params.links {
        if !(0.0..=1.0).contains(&link.strength) {
            return Err(McpError::invalid_params(
                format!(
                    "Link strength must be in [0.0, 1.0], got {} for {} -> {}",
                    link.strength, link.from, link.to
                ),
                None,
            ));
        }

        dag.add_link(CausalLink {
            from: NodeId::new(&link.from),
            to: NodeId::new(&link.to),
            strength: link.strength,
            evidence: link.evidence.clone(),
        })
        .map_err(|e| McpError::invalid_params(format!("Cycle detected: {e}"), None))?;
    }

    let engine = InferenceEngine::new(dag);
    let report = engine
        .infer()
        .map_err(|e| McpError::invalid_params(format!("Inference failed: {e}"), None))?;

    let graph = engine.to_causal_graph();

    let result = json!({
        "success": true,
        "risk_level": risk_level_label(&report.risk_level),
        "confidence": (report.confidence * 10000.0).round() / 10000.0,
        "findings": report.findings,
        "recommendations": report.recommendations,
        "causal_edges": graph.edges.len(),
        "markdown": render_intelligence_report(&report),
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}
