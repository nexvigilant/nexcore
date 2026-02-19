//! SQI (Skill Quality Index) v2 tools.
//!
//! Exposes per-skill and ecosystem-level chemometrics scoring.

use crate::params::{SqiEcosystemParams, SqiScoreParams};
use nexcore_vigilance::skills::sqi::{compute_ecosystem_sqi, compute_sqi, sensitivity_analysis};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Score a single skill's SQI from its SKILL.md content.
pub fn sqi_score(params: SqiScoreParams) -> Result<CallToolResult, McpError> {
    match compute_sqi(&params.content) {
        Ok(result) => {
            let sens = sensitivity_analysis(&result, 0.01);
            let sensitivity: Vec<_> = sens
                .iter()
                .map(|(dim, delta)| {
                    json!({
                        "dimension": format!("{dim}"),
                        "delta_sqi": delta,
                    })
                })
                .collect();

            let dimensions: Vec<_> = result
                .dimensions
                .iter()
                .map(|d| {
                    json!({
                        "dimension": format!("{}", d.dimension),
                        "score": d.score,
                        "weight": d.weight,
                        "weighted": d.weighted,
                        "rationale": d.rationale,
                    })
                })
                .collect();

            let output = json!({
                "sqi": result.sqi,
                "grade": format!("{}", result.grade),
                "dimensions": dimensions,
                "limiting_dimension": format!("{}", result.limiting_dimension),
                "recommendations": result.recommendations,
                "sensitivity": sensitivity,
            });

            Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_string()),
            )]))
        }
        Err(e) => {
            let output = json!({ "error": format!("{e}") });
            Ok(CallToolResult::success(vec![Content::text(
                output.to_string(),
            )]))
        }
    }
}

/// Score ecosystem-level SQI across multiple skills/servers.
pub fn sqi_ecosystem(params: SqiEcosystemParams) -> Result<CallToolResult, McpError> {
    // Score individual skills if contents provided
    let skill_results: Vec<_> = params
        .skill_contents
        .iter()
        .filter_map(|content| compute_sqi(content).ok())
        .collect();

    let eco = compute_ecosystem_sqi(&skill_results, &params.tool_counts);

    let skills_json: Vec<_> = eco
        .skill_results
        .iter()
        .map(|r| {
            json!({
                "sqi": r.sqi,
                "grade": format!("{}", r.grade),
                "limiting_dimension": format!("{}", r.limiting_dimension),
            })
        })
        .collect();

    let output = json!({
        "mean_sqi_unweighted": eco.mean_sqi_unweighted,
        "mean_sqi_weighted": eco.mean_sqi_weighted,
        "distribution_entropy": eco.distribution_entropy,
        "concentration_risk": eco.concentration_risk,
        "grade": format!("{}", eco.grade),
        "tool_counts": params.tool_counts,
        "skills": skills_json,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_string()),
    )]))
}
