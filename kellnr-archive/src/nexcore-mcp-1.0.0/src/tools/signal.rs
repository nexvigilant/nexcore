//! Signal Pipeline tools
//!
//! Exposes signal detection via MCP using nexcore-vigilance::pv (canonical path).
//! Provides drug-event pair analysis with SignalStrength classification.

use crate::params::{SignalBatchParams, SignalDetectParams};
use crate::tooling::attach_forensic_meta;
use nexcore_vigilance::pv::signals::evaluate_signal_complete;
use nexcore_vigilance::pv::thresholds::SignalCriteria;
use nexcore_vigilance::pv::types::ContingencyTable;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};

/// Map PRR value to strength label (matches former SignalStrength enum).
fn strength_label(prr: f64) -> &'static str {
    if prr >= 5.0 {
        "Critical"
    } else if prr >= 3.0 {
        "Strong"
    } else if prr >= 2.0 {
        "Moderate"
    } else if prr >= 1.5 {
        "Weak"
    } else {
        "None"
    }
}

/// Single drug-event signal detection.
pub fn signal_detect(params: SignalDetectParams) -> Result<CallToolResult, McpError> {
    let table = ContingencyTable::new(params.a, params.b, params.c, params.d);
    let criteria = SignalCriteria::evans();
    let result = evaluate_signal_complete(&table, &criteria);

    let prr_val = result.prr.point_estimate;
    let strength = strength_label(prr_val);
    let signal = result.prr.is_signal
        || result.ror.is_signal
        || result.ic.is_signal
        || result.ebgm.is_signal;

    let json = serde_json::json!({
        "drug": params.drug,
        "event": params.event,
        "prr": prr_val,
        "ror": result.ror.point_estimate,
        "ic": result.ic.point_estimate,
        "ebgm": result.ebgm.point_estimate,
        "chi_square": result.chi_square,
        "strength": strength,
        "signal": signal,
    });

    let confidence = match strength {
        "Critical" => 1.0,
        "Strong" => 0.8,
        "Moderate" => 0.6,
        "Weak" => 0.3,
        _ => 0.0,
    };
    let mut res = CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&json).unwrap_or_default(),
    )]);
    attach_forensic_meta(&mut res, confidence, Some(signal), "signal_detect");
    Ok(res)
}

/// Batch signal detection for multiple drug-event pairs.
pub fn signal_batch(params: SignalBatchParams) -> Result<CallToolResult, McpError> {
    let criteria = SignalCriteria::evans();

    let results: Vec<serde_json::Value> = params
        .items
        .iter()
        .map(|item| {
            let table = ContingencyTable::new(item.a, item.b, item.c, item.d);
            let result = evaluate_signal_complete(&table, &criteria);

            let prr_val = result.prr.point_estimate;
            let strength = strength_label(prr_val);
            let signal = result.prr.is_signal
                || result.ror.is_signal
                || result.ic.is_signal
                || result.ebgm.is_signal;

            serde_json::json!({
                "drug": item.drug,
                "event": item.event,
                "prr": prr_val,
                "ror": result.ror.point_estimate,
                "ic": result.ic.point_estimate,
                "ebgm": result.ebgm.point_estimate,
                "chi_square": result.chi_square,
                "strength": strength,
                "signal": signal,
            })
        })
        .collect();

    let signals_found = results
        .iter()
        .filter(|r| r["signal"].as_bool() == Some(true))
        .count();

    let json = serde_json::json!({
        "results": results,
        "signals_found": signals_found,
        "total": results.len(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&json).unwrap_or_default(),
    )]))
}

/// Get threshold configurations (Evans, Strict, Sensitive).
pub fn signal_thresholds() -> Result<CallToolResult, McpError> {
    let evans = SignalCriteria::evans();
    let strict = SignalCriteria::strict();
    let sensitive = SignalCriteria::sensitive();

    let json = serde_json::json!({
        "evans": {
            "prr_min": evans.prr_threshold,
            "chi_square_min": evans.chi_square_threshold,
            "case_count_min": evans.min_cases,
            "ror_lower_ci_min": evans.ror_lower_threshold,
            "ic025_min": evans.ic025_threshold,
            "eb05_min": evans.eb05_threshold,
        },
        "strict": {
            "prr_min": strict.prr_threshold,
            "chi_square_min": strict.chi_square_threshold,
            "case_count_min": strict.min_cases,
            "ror_lower_ci_min": strict.ror_lower_threshold,
            "ic025_min": strict.ic025_threshold,
            "eb05_min": strict.eb05_threshold,
        },
        "sensitive": {
            "prr_min": sensitive.prr_threshold,
            "chi_square_min": sensitive.chi_square_threshold,
            "case_count_min": sensitive.min_cases,
            "ror_lower_ci_min": sensitive.ror_lower_threshold,
            "ic025_min": sensitive.ic025_threshold,
            "eb05_min": sensitive.eb05_threshold,
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&json).unwrap_or_default(),
    )]))
}
