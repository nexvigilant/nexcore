//! Disney Loop MCP tools
//!
//! Forward-only compound discovery pipeline: ρ(t) → ∂(¬σ⁻¹) → ∃(ν) → ρ(t+1)
//!
//! Each stage maps to a T1 primitive:
//! - ρ(t): State assessment (ρ = Recursion)
//! - ∂(¬σ⁻¹): Anti-regression gate (∂ = Boundary, σ = Sequence, ¬ = negation)
//! - ∃(ν): Curiosity search (∃ = Existence, ν = Frequency)
//! - ρ(t+1): New state (ρ = Recursion, forward)

use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use std::collections::HashMap;

use crate::params::disney_loop::{
    DisneyAntiRegressionParams, DisneyCuriositySearchParams, DisneyLoopRunParams, DisneyRecord,
    DisneyStateAssessParams,
};

/// Run the full Disney Loop pipeline: ingest → filter → aggregate → output.
pub fn run(params: DisneyLoopRunParams) -> Result<CallToolResult, McpError> {
    let total = params.records.len();

    // Stage 1: ρ(t) — State Assessment
    let records = params.records;

    // Stage 2: ∂(¬σ⁻¹) — Anti-Regression Gate
    let forward: Vec<&DisneyRecord> = records
        .iter()
        .filter(|r| r.direction != "backward")
        .collect();
    let rejected = total - forward.len();

    // Stage 3: ∃(ν) — Curiosity Search (aggregate by domain)
    let mut domains: HashMap<&str, (i64, Vec<&str>)> = HashMap::new();
    for r in &forward {
        let entry = domains.entry(&r.domain).or_insert((0, Vec::new()));
        entry.0 += r.novelty_score;
        if r.discovery != "none" {
            entry.1.push(&r.discovery);
        }
    }

    let aggregated: Vec<serde_json::Value> = domains
        .iter()
        .map(|(domain, (novelty, discoveries))| {
            serde_json::json!({
                "domain": domain,
                "total_novelty": novelty,
                "discoveries": discoveries.len(),
                "discovery_labels": discoveries,
            })
        })
        .collect();

    let result = serde_json::json!({
        "pipeline": "ρ(t) → ∂(¬σ⁻¹) → ∃(ν) → ρ(t+1)",
        "stages": {
            "state_assessment": { "records_ingested": total },
            "anti_regression_gate": { "rejected": rejected, "passed": forward.len() },
            "curiosity_search": { "domains_found": domains.len() },
        },
        "output": aggregated,
        "t1_primitives": {
            "ρ": "Recursion — state feeds back into next iteration",
            "∂": "Boundary — rejects backward regression",
            "σ⁻¹": "Inverse Sequence — backward direction (negated)",
            "∃": "Existence — novel discoveries instantiated",
            "ν": "Frequency — novelty score aggregated",
        }
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Stage 2: ∂(¬σ⁻¹) — Anti-Regression Gate. Filters out backward records.
pub fn anti_regression(params: DisneyAntiRegressionParams) -> Result<CallToolResult, McpError> {
    let total = params.records.len();
    let forward: Vec<&DisneyRecord> = params
        .records
        .iter()
        .filter(|r| r.direction != "backward")
        .collect();
    let rejected = total - forward.len();

    let kept: Vec<serde_json::Value> = forward
        .iter()
        .map(|r| {
            serde_json::json!({
                "domain": r.domain,
                "direction": r.direction,
                "novelty_score": r.novelty_score,
                "discovery": r.discovery,
            })
        })
        .collect();

    let result = serde_json::json!({
        "stage": "∂(¬σ⁻¹) — Anti-Regression Gate",
        "total_input": total,
        "rejected_backward": rejected,
        "passed_forward": forward.len(),
        "records": kept,
        "t1_grounding": "BOUNDARY (∂) — only forward-moving state survives",
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Stage 3: ∃(ν) — Curiosity Search. Aggregates novelty by domain.
pub fn curiosity_search(params: DisneyCuriositySearchParams) -> Result<CallToolResult, McpError> {
    let mut domains: HashMap<&str, (i64, Vec<&str>)> = HashMap::new();
    for r in &params.records {
        let entry = domains.entry(&r.domain).or_insert((0, Vec::new()));
        entry.0 += r.novelty_score;
        if r.discovery != "none" {
            entry.1.push(&r.discovery);
        }
    }

    let aggregated: Vec<serde_json::Value> = domains
        .iter()
        .map(|(domain, (novelty, discoveries))| {
            serde_json::json!({
                "domain": domain,
                "total_novelty": novelty,
                "discovery_count": discoveries.len(),
                "discoveries": discoveries,
            })
        })
        .collect();

    let total_novelty: i64 = domains.values().map(|(n, _)| n).sum();

    let result = serde_json::json!({
        "stage": "∃(ν) — Curiosity Search",
        "domains_found": domains.len(),
        "total_novelty_across_domains": total_novelty,
        "aggregated": aggregated,
        "t1_grounding": "EXISTENCE (∃) + FREQUENCY (ν) — novel discoveries counted by domain",
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Stage 1: ρ(t) — State Assessment. Analyze records without transforming.
pub fn state_assess(params: DisneyStateAssessParams) -> Result<CallToolResult, McpError> {
    let total = params.records.len();
    let forward_count = params
        .records
        .iter()
        .filter(|r| r.direction == "forward")
        .count();
    let backward_count = total - forward_count;

    let mut domain_counts: HashMap<&str, usize> = HashMap::new();
    let mut total_novelty: i64 = 0;
    let mut discoveries: Vec<&str> = Vec::new();

    for r in &params.records {
        *domain_counts.entry(&r.domain).or_insert(0) += 1;
        total_novelty += r.novelty_score;
        if r.discovery != "none" {
            discoveries.push(&r.discovery);
        }
    }

    let avg_novelty = if total > 0 {
        total_novelty as f64 / total as f64
    } else {
        0.0
    };

    let forward_ratio = if total > 0 {
        forward_count as f64 / total as f64
    } else {
        0.0
    };

    let result = serde_json::json!({
        "stage": "ρ(t) — State Assessment",
        "total_records": total,
        "forward_count": forward_count,
        "backward_count": backward_count,
        "forward_ratio": forward_ratio,
        "health": if forward_ratio >= 0.8 {
            "healthy — strong forward momentum"
        } else if forward_ratio >= 0.5 {
            "mixed — some regression present"
        } else {
            "regressing — majority backward"
        },
        "unique_domains": domain_counts.len(),
        "domain_distribution": domain_counts,
        "total_novelty": total_novelty,
        "avg_novelty_per_record": avg_novelty,
        "unique_discoveries": discoveries.len(),
        "t1_grounding": "RECURSION (ρ) — assess current state before next iteration",
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}
