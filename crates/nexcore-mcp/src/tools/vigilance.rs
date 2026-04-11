//! Vigilance tools: safety margin, risk scoring, ToV, harm types, SVG charts
//!
//! Theory of Vigilance (ToV) implementation, plus feature-gated viz rendering
//! via `nexcore_vigilance::viz` (requires the `viz` feature on nexcore-vigilance).

use crate::params::{
    MapToTovParams, PvSignalChartParams, PvSignalComparisonParams, RiskScoreGeometricParams,
    RiskScoreParams, SafetyMarginParams,
};
use crate::tooling::attach_forensic_meta;
use nexcore_vigilance::guardian::{OriginatorType, RiskContext, calculate_risk_score};
use nexcore_vigilance::pv::hierarchy::{LEVEL_METADATA, SafetyLevel, map_to_tov_level};
use nexcore_vigilance::pv::{ContingencyTable, SignalCriteria};
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

/// Non-compensatory risk scoring via geometric mean aggregation (ASDF v2.0)
pub fn risk_score_geometric(params: RiskScoreGeometricParams) -> Result<CallToolResult, McpError> {
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

    // Parse optional custom weights
    let custom_weights: Option<[f64; 4]> = params.weights.as_ref().and_then(|w| {
        if w.len() == 4 {
            let arr = [w[0], w[1], w[2], w[3]];
            let sum: f64 = arr.iter().sum();
            if (sum - 1.0).abs() < 0.01 {
                Some(arr)
            } else {
                None // Invalid weights — fall back to default
            }
        } else {
            None
        }
    });
    let weights_ref = custom_weights.as_ref();

    let mode = params.mode.to_lowercase();

    let json = match mode.as_str() {
        "additive" => {
            let result = calculate_risk_score(&context);
            json!({
                "mode": "additive",
                "drug": params.drug,
                "event": params.event,
                "score": result.score,
                "level": result.level,
                "factors": result.factors,
            })
        }
        "geometric" => {
            let result = nexcore_guardian_engine::noncompensatory::calculate_noncompensatory_score(
                &context,
                weights_ref,
            );
            json!({
                "mode": "geometric_noncompensatory",
                "drug": params.drug,
                "event": params.event,
                "composite_score": result.composite,
                "level": result.level,
                "signals_detected": format!("{}/4", result.signals_detected),
                "n_weight": result.n_weight,
                "dimensions": result.dimensions,
                "factors": result.factors,
                "weights_used": weights_ref.unwrap_or(
                    &nexcore_guardian_engine::noncompensatory::DEFAULT_WEIGHTS
                ),
            })
        }
        _ => {
            // "dual" mode — side-by-side comparison with masking detection
            let dual = nexcore_guardian_engine::noncompensatory::calculate_dual_mode(
                &context,
                weights_ref,
            );
            json!({
                "mode": "dual_comparison",
                "drug": params.drug,
                "event": params.event,
                "additive": {
                    "score": dual.additive.score,
                    "level": dual.additive.level,
                    "factors": dual.additive.factors,
                },
                "geometric": {
                    "composite_score": dual.geometric.composite,
                    "level": dual.geometric.level,
                    "signals_detected": format!("{}/4", dual.geometric.signals_detected),
                    "dimensions": dual.geometric.dimensions,
                    "factors": dual.geometric.factors,
                },
                "divergence": dual.divergence,
                "compensatory_masking": dual.compensatory_masking,
                "divergence_explanation": dual.divergence_explanation,
                "weights_used": weights_ref.unwrap_or(
                    &nexcore_guardian_engine::noncompensatory::DEFAULT_WEIGHTS
                ),
            })
        }
    };

    let confidence = match mode.as_str() {
        "geometric" => {
            let r = nexcore_guardian_engine::noncompensatory::calculate_noncompensatory_score(
                &context,
                weights_ref,
            );
            r.composite.value / 100.0
        }
        _ => {
            let r = calculate_risk_score(&context);
            r.score.value / 100.0
        }
    };

    let mut res = CallToolResult::success(vec![Content::text(json.to_string())]);
    attach_forensic_meta(&mut res, confidence, None, "risk_score_geometric");
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

// ============================================================================
// Signal Visualization Tools (feature-gated: nexcore-vigilance/viz)
// ============================================================================

/// Build a `ContingencyTable` from raw cell counts.
fn make_table(a: u64, b: u64, c: u64, d: u64) -> ContingencyTable {
    ContingencyTable::new(a, b, c, d)
}

/// Render a PRR timeline SVG for a set of drug-event pairs.
///
/// Each entry supplies a 2×2 contingency table. The chart draws PRR values
/// as a line series with the Evans threshold (PRR = 2.0) overlaid in red.
/// Returns the raw SVG markup as text content.
///
/// # Errors
/// Returns `McpError::invalid_params` if `entries` is empty.
/// Returns `McpError::internal_error` if the plotters renderer fails.
pub fn pv_signal_chart(params: PvSignalChartParams) -> Result<CallToolResult, McpError> {
    if params.entries.is_empty() {
        return Err(McpError::invalid_params(
            "entries must contain at least one drug-event pair",
            None,
        ));
    }

    let tables: Vec<ContingencyTable> = params
        .entries
        .iter()
        .map(|e| make_table(e.a, e.b, e.c, e.d))
        .collect();

    let data: Vec<(&str, &ContingencyTable)> = params
        .entries
        .iter()
        .zip(tables.iter())
        .map(|(e, t)| (e.label.as_str(), t))
        .collect();

    let criteria = SignalCriteria::evans();

    let svg = nexcore_vigilance::viz::render_prr_timeline_svg(
        &params.title,
        &data,
        &criteria,
        params.width,
        params.height,
    )
    .map_err(|e| McpError::internal_error(format!("Chart rendering failed: {e}"), None))?;

    let n_signals = data
        .iter()
        .filter(|(_, table)| {
            nexcore_vigilance::pv::calculate_prr(table, &criteria).point_estimate >= 2.0
        })
        .count();
    let confidence = if data.is_empty() {
        0.0
    } else {
        n_signals as f64 / data.len() as f64
    };

    let mut result = CallToolResult::success(vec![Content::text(svg)]);
    attach_forensic_meta(
        &mut result,
        confidence,
        Some(n_signals > 0),
        "pv_signal_chart",
    );
    Ok(result)
}

/// Render a PRR vs ROR comparison SVG for a set of drug-event pairs.
///
/// Shows both algorithms side-by-side on a single chart so reviewers can
/// confirm algorithm agreement. The Evans threshold line (2.0) is drawn in red.
/// Returns the raw SVG markup as text content.
///
/// # Errors
/// Returns `McpError::invalid_params` if `entries` is empty.
/// Returns `McpError::internal_error` if the plotters renderer fails.
pub fn pv_signal_comparison(params: PvSignalComparisonParams) -> Result<CallToolResult, McpError> {
    if params.entries.is_empty() {
        return Err(McpError::invalid_params(
            "entries must contain at least one drug-event pair",
            None,
        ));
    }

    let tables: Vec<ContingencyTable> = params
        .entries
        .iter()
        .map(|e| make_table(e.a, e.b, e.c, e.d))
        .collect();

    let data: Vec<(&str, &ContingencyTable)> = params
        .entries
        .iter()
        .zip(tables.iter())
        .map(|(e, t)| (e.label.as_str(), t))
        .collect();

    let criteria = SignalCriteria::evans();

    let svg = nexcore_vigilance::viz::render_signal_comparison_svg(
        &params.title,
        &data,
        &criteria,
        params.width,
        params.height,
    )
    .map_err(|e| McpError::internal_error(format!("Chart rendering failed: {e}"), None))?;

    // Confidence: fraction of pairs where both PRR and ROR exceed threshold
    let n_agreement = data
        .iter()
        .filter(|(_, table)| {
            let prr = nexcore_vigilance::pv::calculate_prr(table, &criteria).point_estimate;
            let ror = nexcore_vigilance::pv::calculate_ror(table, &criteria).point_estimate;
            prr >= 2.0 && ror >= 2.0
        })
        .count();
    let confidence = if data.is_empty() {
        0.0
    } else {
        n_agreement as f64 / data.len() as f64
    };

    let mut result = CallToolResult::success(vec![Content::text(svg)]);
    attach_forensic_meta(
        &mut result,
        confidence,
        Some(n_agreement > 0),
        "pv_signal_comparison",
    );
    Ok(result)
}

/// Parse a raw RSK chain output into a typed Verdict struct.
/// This is the driveshaft between the RSK engine (Excrete) and Guardian navigator (Absorb).
pub fn verdict_from_chain(
    params: crate::params::VerdictFromChainParams,
) -> Result<CallToolResult, McpError> {
    use nexcore_vigilance::verdict::Verdict;
    use std::collections::HashMap;

    // Convert JSON object to HashMap<String, Value>
    let map: HashMap<String, serde_json::Value> = match params.chain_output {
        serde_json::Value::Object(obj) => obj.into_iter().collect(),
        _ => {
            return Ok(CallToolResult::error(vec![Content::text(
                json!({
                    "error": "chain_output must be a JSON object",
                    "received_type": "non-object",
                })
                .to_string(),
            )]));
        }
    };

    match Verdict::from_chain_output(&map) {
        Ok(mut verdict) => {
            verdict.drug = params.drug;
            verdict.event = params.event;
            let json = serde_json::to_value(&verdict).unwrap_or_else(|_| json!({}));
            Ok(CallToolResult::success(vec![Content::text(json.to_string())]))
        }
        Err(e) => Ok(CallToolResult::error(vec![Content::text(
            json!({
                "error": format!("Failed to parse chain output: {}", e),
                "hint": "Ensure the chain_output contains signal_detected, causality, regulatory_action, and deadline_days fields",
            })
            .to_string(),
        )])),
    }
}
