//! Polymer MCP tools — hook pipeline composition with stoichiometry.
//!
//! Composes hook sequences into linear or cyclic pipelines (polymers).
//! Validates event compatibility, computes theoretical properties.
//!
//! ## T1 Primitive Grounding
//! - Pipeline: σ(Sequence) + →(Causality)
//! - Stoichiometry: N(Quantity) + ∂(Boundary)
//! - Validation: κ(Comparison) + ×(Product)

use crate::params::polymer::{PolymerAnalyzeParams, PolymerComposeParams, PolymerValidateParams};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::path::Path;

/// Compose a hook pipeline from named hooks.
pub fn compose(params: PolymerComposeParams) -> Result<CallToolResult, McpError> {
    let topology = params.topology.as_deref().unwrap_or("linear");
    let stoichiometry = params.stoichiometry.unwrap_or(1.0);

    if params.hooks.is_empty() {
        return Err(McpError::invalid_params(
            "At least one hook required".to_string(),
            None,
        ));
    }

    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/matthew".to_string());
    let hooks_dir = format!("{}/.claude/hooks/core-hooks/src/bin", home);

    // Resolve hooks to their binary paths and validate existence
    let mut stages = Vec::new();
    let mut missing = Vec::new();

    for (i, hook_name) in params.hooks.iter().enumerate() {
        let binary_path = format!(
            "{}/.claude/hooks/core-hooks/target/release/{}",
            home, hook_name
        );
        let shell_path = format!("{}/.claude/hooks/{}.sh", home, hook_name);
        let src_path = format!("{}/{}.rs", hooks_dir, hook_name);

        let exists = Path::new(&binary_path).exists()
            || Path::new(&shell_path).exists()
            || Path::new(&src_path).exists();

        if !exists {
            missing.push(hook_name.clone());
        }

        stages.push(json!({
            "index": i,
            "hook": hook_name,
            "exists": exists,
            "binary_path": binary_path,
        }));
    }

    let pipeline_valid = missing.is_empty();
    let chain_length = params.hooks.len();

    // Compute theoretical latency (sum of stage latencies, estimated)
    let est_latency_ms = chain_length as f64 * 15.0; // ~15ms per hook average

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "pipeline": {
                "topology": topology,
                "stoichiometry": stoichiometry,
                "stages": stages,
                "chain_length": chain_length,
            },
            "validation": {
                "valid": pipeline_valid,
                "missing_hooks": missing,
            },
            "properties": {
                "estimated_latency_ms": est_latency_ms,
                "fail_fast": stoichiometry >= 1.0,
                "is_cyclic": topology == "cyclic",
            },
            "execution_order": params.hooks,
        })
        .to_string(),
    )]))
}

/// Validate a pipeline definition.
pub fn validate(params: PolymerValidateParams) -> Result<CallToolResult, McpError> {
    let check_events = params.check_events.unwrap_or(true);
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/matthew".to_string());

    let mut issues = Vec::new();

    // Check for duplicates
    let mut seen = std::collections::HashSet::new();
    for hook in &params.hooks {
        if !seen.insert(hook.as_str()) {
            issues.push(json!({"type": "duplicate", "hook": hook, "severity": "warning"}));
        }
    }

    // Check existence
    for hook in &params.hooks {
        let binary = format!("{}/.claude/hooks/core-hooks/target/release/{}", home, hook);
        let shell = format!("{}/.claude/hooks/{}.sh", home, hook);
        if !Path::new(&binary).exists() && !Path::new(&shell).exists() {
            issues.push(json!({"type": "missing", "hook": hook, "severity": "error"}));
        }
    }

    // Check for known incompatible sequences
    if check_events {
        for window in params.hooks.windows(2) {
            let a = &window[0];
            let b = &window[1];
            // Known incompatibility: blocking hooks before async hooks in same event
            if a.contains("blocker") && b.contains("async") {
                issues.push(json!({
                    "type": "event_incompatibility",
                    "hooks": [a, b],
                    "reason": "Blocking hook before async hook may cause timeout",
                    "severity": "warning",
                }));
            }
        }
    }

    let valid = !issues.iter().any(|i| i["severity"] == "error");

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "valid": valid,
            "hook_count": params.hooks.len(),
            "issues": issues,
            "issue_count": issues.len(),
        })
        .to_string(),
    )]))
}

/// Analyze a pipeline's theoretical properties.
pub fn analyze(params: PolymerAnalyzeParams) -> Result<CallToolResult, McpError> {
    let chain_len = params.hooks.len();

    // Reliability: each stage has ~0.98 reliability, chain = 0.98^n
    let per_stage_reliability = 0.98f64;
    let chain_reliability = per_stage_reliability.powi(chain_len as i32);

    // Latency: additive across stages
    let est_latency_ms = chain_len as f64 * 15.0;

    // Information flow: each stage can add/filter/transform
    let information_throughput = 1.0 / (1.0 + chain_len as f64 * 0.05); // Slight loss per stage

    // Identify potential bottlenecks
    let bottlenecks: Vec<serde_json::Value> = params.hooks.iter()
        .enumerate()
        .filter(|(_, name)| {
            name.contains("compile") || name.contains("build") || name.contains("test")
        })
        .map(|(i, name)| json!({"stage": i, "hook": name, "reason": "Potentially slow (compile/build/test)"}))
        .collect();

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "chain_length": chain_len,
            "hooks": params.hooks,
            "properties": {
                "chain_reliability": (chain_reliability * 10000.0).round() / 10000.0,
                "per_stage_reliability": per_stage_reliability,
                "estimated_latency_ms": est_latency_ms,
                "information_throughput": (information_throughput * 100.0).round() / 100.0,
            },
            "bottlenecks": bottlenecks,
            "recommendations": if chain_len > 5 {
                vec!["Consider splitting into sub-pipelines for reliability"]
            } else {
                vec!["Pipeline length is manageable"]
            },
        })
        .to_string(),
    )]))
}
