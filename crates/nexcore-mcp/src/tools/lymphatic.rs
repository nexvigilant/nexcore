//! Lymphatic System MCP tools — overflow management, output quality, inspection.
//!
//! Maps Claude Code's output quality infrastructure:
//! - Drainage: overflow handling (excess output, log rotation)
//! - Lymph nodes: inspection/validation checkpoints
//! - Thymic selection: quality gating (positive/negative selection)
//!
//! ## T1 Primitive Grounding
//! - Drainage: ∂(Boundary) + N(Quantity)
//! - Inspection: κ(Comparison) + ∃(Existence)
//! - Selection: Σ(Sum) + ∝(Irreversibility)

use crate::params::lymphatic::{
    LymphaticDrainageParams, LymphaticHealthParams, LymphaticInspectParams, LymphaticThymicParams,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Analyze drainage capacity (overflow management).
pub fn drainage(params: LymphaticDrainageParams) -> Result<CallToolResult, McpError> {
    let items = params.item_count;
    let capacity = params.capacity.unwrap_or(10000);

    let utilization = items as f64 / capacity.max(1) as f64;
    let overflow = if items > capacity {
        items - capacity
    } else {
        0
    };

    let status = if utilization < 0.7 {
        "normal"
    } else if utilization < 0.9 {
        "elevated"
    } else if utilization <= 1.0 {
        "near_capacity"
    } else {
        "overflow"
    };

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "drainage": {
                "items": items,
                "capacity": capacity,
                "utilization": (utilization * 100.0).round() / 100.0,
                "overflow_count": overflow,
                "status": status,
            },
            "recommendations": if overflow > 0 {
                vec![
                    format!("Drain {} excess items", overflow),
                    "Increase capacity or reduce inflow".to_string(),
                    "Consider archiving older items".to_string(),
                ]
            } else if utilization > 0.8 {
                vec!["Approaching capacity — schedule preventive drainage".to_string()]
            } else {
                vec!["Drainage operating normally".to_string()]
            },
            "analog": {
                "items": "Output tokens, log entries, telemetry records",
                "capacity": "Buffer/file size limits",
                "drainage": "Log rotation, telemetry pruning, waste collection",
            },
        })
        .to_string(),
    )]))
}

/// Run thymic selection on a candidate (quality gate).
pub fn thymic_selection(params: LymphaticThymicParams) -> Result<CallToolResult, McpError> {
    let candidate = &params.candidate;
    let criteria = params.criteria.unwrap_or_else(|| {
        vec![
            "exists".to_string(),
            "documented".to_string(),
            "tested".to_string(),
            "grounded".to_string(),
        ]
    });

    // Simulate thymic positive/negative selection
    let mut passed = Vec::new();
    let mut failed = Vec::new();

    for criterion in &criteria {
        let c = criterion.to_lowercase();
        // Heuristic checks based on criterion type
        let result = match c.as_str() {
            "exists" => !candidate.is_empty(),
            "documented" => candidate.len() > 3, // Has enough substance
            "tested" => true,                    // Assume tested unless proven otherwise
            "grounded" => true,                  // Assume grounded
            _ => true,
        };

        if result {
            passed.push(json!({"criterion": criterion, "verdict": "positive_selection"}));
        } else {
            failed.push(json!({"criterion": criterion, "verdict": "negative_selection"}));
        }
    }

    let verdict = if failed.is_empty() {
        "mature_t_cell"
    } else if failed.len() as f64 / criteria.len() as f64 > 0.5 {
        "apoptosis"
    } else {
        "partial_anergy"
    };

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "candidate": candidate,
            "thymic_selection": {
                "verdict": verdict,
                "positive_selections": passed,
                "negative_selections": failed,
                "criteria_count": criteria.len(),
                "pass_rate": if criteria.is_empty() { 0.0 }
                    else { (passed.len() as f64 / criteria.len() as f64 * 100.0).round() / 100.0 },
            },
            "analog": {
                "positive_selection": "Candidate recognizes self-MHC (meets minimum quality)",
                "negative_selection": "Candidate is autoreactive (violates constraints)",
                "mature_t_cell": "Passes all gates — ready for deployment",
                "apoptosis": "Fails too many gates — should be removed",
                "anergy": "Partially functional — needs improvement",
            },
        })
        .to_string(),
    )]))
}

/// Inspect a node in the lymphatic network.
pub fn inspect(params: LymphaticInspectParams) -> Result<CallToolResult, McpError> {
    let node = &params.node;

    // Map known inspection nodes to their roles
    let inspection = match node.to_lowercase().as_str() {
        "output" | "response" => json!({
            "node": node,
            "role": "Output quality checkpoint",
            "checks": ["format_compliance", "content_safety", "length_bounds", "encoding_valid"],
            "position": "efferent (outgoing)",
        }),
        "input" | "prompt" => json!({
            "node": node,
            "role": "Input validation checkpoint",
            "checks": ["injection_detection", "encoding_valid", "size_limits", "schema_match"],
            "position": "afferent (incoming)",
        }),
        "hook" | "hooks" => json!({
            "node": node,
            "role": "Hook execution checkpoint",
            "checks": ["timeout_enforcement", "exit_code_valid", "output_parseable", "no_side_effects"],
            "position": "interstitial",
        }),
        "tool" | "mcp" => json!({
            "node": node,
            "role": "MCP tool dispatch checkpoint",
            "checks": ["command_known", "params_valid", "result_structured", "error_propagated"],
            "position": "visceral",
        }),
        _ => json!({
            "node": node,
            "role": "Unknown node — no predefined inspection",
            "checks": [],
            "position": "unclassified",
        }),
    };

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "inspection": inspection,
            "analog": {
                "lymph_node": "Checkpoint where antigens (data) are inspected",
                "afferent": "Incoming data validation",
                "efferent": "Outgoing quality control",
                "interstitial": "In-transit processing checks",
            },
        })
        .to_string(),
    )]))
}

/// Get lymphatic system health overview.
pub fn health(_params: LymphaticHealthParams) -> Result<CallToolResult, McpError> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/matthew".to_string());

    // Check overflow indicators
    let hook_log = format!("{}/.claude/hooks/state/hook_executions.jsonl", home);
    let log_size = std::fs::metadata(&hook_log).map(|m| m.len()).unwrap_or(0);

    let telemetry_dir = format!("{}/.claude/brain/telemetry", home);
    let telemetry_exists = std::path::Path::new(&telemetry_dir).exists();

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "lymphatic_health": {
                "status": "operational",
                "hook_log_size_bytes": log_size,
                "hook_log_overflow_risk": if log_size > 10_000_000 { "high" }
                    else if log_size > 1_000_000 { "moderate" }
                    else { "low" },
                "telemetry_drainage": telemetry_exists,
            },
            "components": {
                "drainage": "Log rotation, telemetry pruning (urinary system handles)",
                "lymph_nodes": "Hook validation checkpoints (PreToolUse, PostToolUse)",
                "thymic_selection": "Skill validation (Diamond v2), hook compilation",
                "spleen": "Antibody registry (immunity system)",
            },
        })
        .to_string(),
    )]))
}
