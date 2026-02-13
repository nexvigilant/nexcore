//! Vigilance tools: safety margin, risk scoring, ToV, harm types
//!
//! Theory of Vigilance (ToV) implementation.

use crate::params::{MapToTovParams, RiskScoreParams, SafetyMarginParams};
use crate::tooling::attach_forensic_meta;
use nexcore_vigilance::guardian::{OriginatorType, RiskContext, calculate_risk_score};
use nexcore_vigilance::pv::hierarchy::{LEVEL_METADATA, SafetyLevel, map_to_tov_level};
use nexcore_vigilance::tov::{HarmType, SafetyMargin};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Calculate safety margin d(s)
pub fn safety_margin(params: SafetyMarginParams) -> Result<CallToolResult, McpError> {
    let result = SafetyMargin::calculate(
        params.prr,
        params.ror_lower,
        params.ic025,
        params.eb05,
        params.n,
    );

    let json = json!({
        "distance": result.distance,
        "interpretation": result.interpretation,
        "action": result.action,
        "inputs": {
            "prr": params.prr,
            "ror_lower": params.ror_lower,
            "ic025": params.ic025,
            "eb05": params.eb05,
            "n": params.n,
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Guardian-AV risk scoring
pub fn risk_score(params: RiskScoreParams) -> Result<CallToolResult, McpError> {
    let context = RiskContext {
        drug: params.drug.clone(),
        event: params.event.clone(),
        prr: params.prr,
        ror_lower: params.ror_lower,
        ic025: params.ic025,
        eb05: params.eb05,
        n: params.n,
        originator: OriginatorType::default(),
    };

    let result = calculate_risk_score(&context);

    let json = json!({
        "score": result.score,
        "level": result.level,
        "factors": result.factors,
        "drug": params.drug,
        "event": params.event,
    });

    let mut res = CallToolResult::success(vec![Content::text(json.to_string())]);
    attach_forensic_meta(&mut res, result.score.value / 10.0, None, "risk_score");
    Ok(res)
}

/// List all 8 harm types
pub fn harm_types() -> Result<CallToolResult, McpError> {
    let types: Vec<_> = HarmType::all()
        .iter()
        .map(|h| {
            json!({
                "letter": h.letter().to_string(),
                "name": format!("{:?}", h),
                "conservation_law": h.conservation_law(),
                "hierarchy_levels": h.hierarchy_levels(),
            })
        })
        .collect();

    let json = json!({
        "harm_types": types,
        "count": types.len(),
        "note": "A-H derived combinatorially from 3 binary attributes (2³ = 8): Temporal, Scope, Mechanism",
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Map SafetyLevel to ToV level
pub fn map_to_tov(params: MapToTovParams) -> Result<CallToolResult, McpError> {
    let safety_level = match params.level {
        1 => SafetyLevel::Molecular,
        2 => SafetyLevel::Cellular,
        3 => SafetyLevel::Tissue,
        4 => SafetyLevel::Organ,
        5 => SafetyLevel::System,
        6 => SafetyLevel::Clinical,
        7 => SafetyLevel::Epidemiological,
        8 => SafetyLevel::Regulatory,
        _ => {
            let json = json!({
                "error": "Invalid level. Must be 1-8 (Molecular to Regulatory).",
            });
            return Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]));
        }
    };

    let tov_level = map_to_tov_level(safety_level);

    // Get metadata if available
    let metadata = LEVEL_METADATA.get(&safety_level);

    let json = json!({
        "safety_level": params.level,
        "safety_level_name": format!("{:?}", safety_level),
        "tov_level": tov_level as u8,
        "tov_level_name": format!("{:?}", tov_level),
        "metadata": metadata.map(|m| json!({
            "scope": m.scope,
            "time_scale": format!("{} - {}", m.time_scale_min, m.time_scale_max),
            "system_units": format!("{} - {}", m.system_units_min, m.system_units_max),
            "example_phenomena": m.example_phenomena,
        })),
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}
