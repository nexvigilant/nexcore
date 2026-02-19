//! PV Signal Detection tools
//!
//! Pharmacovigilance signal detection and causality assessment.

use crate::params::{
    ContingencyTableParams, NaranjoParams, SignalAlgorithmParams, SignalCompleteParams,
    WhoUmcParams,
};
use crate::tooling::attach_forensic_meta;
use nexcore_vigilance::pv::causality::{calculate_naranjo_quick, calculate_who_umc_quick};
use nexcore_vigilance::pv::signals::evaluate_signal_complete;
use nexcore_vigilance::pv::thresholds::SignalCriteria;
use nexcore_vigilance::pv::types::ContingencyTable;
use nexcore_vigilance::pv::{
    calculate_chi_square, calculate_ebgm, calculate_ic, calculate_prr, calculate_ror,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

fn to_table(params: &ContingencyTableParams) -> ContingencyTable {
    ContingencyTable::new(params.a, params.b, params.c, params.d)
}

/// Complete signal analysis with all algorithms
pub fn signal_complete(params: SignalCompleteParams) -> Result<CallToolResult, McpError> {
    let table = to_table(&params.table);
    let mut criteria = SignalCriteria::evans();
    criteria.prr_threshold = params.prr_threshold;
    criteria.min_cases = params.min_n;

    let result = evaluate_signal_complete(&table, &criteria);

    let json = json!({
        "prr": {
            "point_estimate": result.prr.point_estimate,
            "lower_ci": result.prr.lower_ci,
            "upper_ci": result.prr.upper_ci,
            "is_signal": result.prr.is_signal,
        },
        "ror": {
            "point_estimate": result.ror.point_estimate,
            "lower_ci": result.ror.lower_ci,
            "upper_ci": result.ror.upper_ci,
            "is_signal": result.ror.is_signal,
        },
        "ic": {
            "point_estimate": result.ic.point_estimate,
            "lower_ci": result.ic.lower_ci,
            "upper_ci": result.ic.upper_ci,
            "is_signal": result.ic.is_signal,
        },
        "ebgm": {
            "point_estimate": result.ebgm.point_estimate,
            "lower_ci": result.ebgm.lower_ci,
            "upper_ci": result.ebgm.upper_ci,
            "is_signal": result.ebgm.is_signal,
        },
        "chi_square": result.chi_square,
        "n": result.n,
        "any_signal": result.prr.is_signal || result.ror.is_signal || result.ic.is_signal || result.ebgm.is_signal,
    });

    let detecting = [
        result.prr.is_signal,
        result.ror.is_signal,
        result.ic.is_signal,
        result.ebgm.is_signal,
    ]
    .iter()
    .filter(|&&s| s)
    .count();
    let any_signal = detecting > 0;
    let confidence = detecting as f64 / 4.0;
    let mut res = CallToolResult::success(vec![Content::text(json.to_string())]);
    attach_forensic_meta(&mut res, confidence, Some(any_signal), "pv_signal");
    Ok(res)
}

/// Calculate PRR
pub fn signal_prr(params: SignalAlgorithmParams) -> Result<CallToolResult, McpError> {
    let table = to_table(&params.table);
    let criteria = SignalCriteria::evans();
    let result = calculate_prr(&table, &criteria);

    let json = json!({
        "algorithm": "PRR",
        "point_estimate": result.point_estimate,
        "lower_ci": result.lower_ci,
        "upper_ci": result.upper_ci,
        "is_signal": result.is_signal,
    });

    let mut res = CallToolResult::success(vec![Content::text(json.to_string())]);
    attach_forensic_meta(
        &mut res,
        if result.is_signal { 0.95 } else { 0.1 },
        Some(result.is_signal),
        "pv_signal",
    );
    Ok(res)
}

/// Calculate ROR
pub fn signal_ror(params: SignalAlgorithmParams) -> Result<CallToolResult, McpError> {
    let table = to_table(&params.table);
    let criteria = SignalCriteria::evans();
    let result = calculate_ror(&table, &criteria);

    let json = json!({
        "algorithm": "ROR",
        "point_estimate": result.point_estimate,
        "lower_ci": result.lower_ci,
        "upper_ci": result.upper_ci,
        "is_signal": result.is_signal,
    });

    let mut res = CallToolResult::success(vec![Content::text(json.to_string())]);
    attach_forensic_meta(
        &mut res,
        if result.is_signal { 0.95 } else { 0.1 },
        Some(result.is_signal),
        "pv_signal",
    );
    Ok(res)
}

/// Calculate IC
pub fn signal_ic(params: SignalAlgorithmParams) -> Result<CallToolResult, McpError> {
    let table = to_table(&params.table);
    let criteria = SignalCriteria::evans();
    let result = calculate_ic(&table, &criteria);

    let json = json!({
        "algorithm": "IC",
        "point_estimate": result.point_estimate,
        "lower_ci": result.lower_ci,
        "upper_ci": result.upper_ci,
        "is_signal": result.is_signal,
    });

    let mut res = CallToolResult::success(vec![Content::text(json.to_string())]);
    attach_forensic_meta(
        &mut res,
        if result.is_signal { 0.95 } else { 0.1 },
        Some(result.is_signal),
        "pv_signal",
    );
    Ok(res)
}

/// Calculate EBGM
pub fn signal_ebgm(params: SignalAlgorithmParams) -> Result<CallToolResult, McpError> {
    let table = to_table(&params.table);
    let criteria = SignalCriteria::evans();
    let result = calculate_ebgm(&table, &criteria);

    let json = json!({
        "algorithm": "EBGM",
        "point_estimate": result.point_estimate,
        "lower_ci": result.lower_ci,
        "upper_ci": result.upper_ci,
        "is_signal": result.is_signal,
    });

    let mut res = CallToolResult::success(vec![Content::text(json.to_string())]);
    attach_forensic_meta(
        &mut res,
        if result.is_signal { 0.95 } else { 0.1 },
        Some(result.is_signal),
        "pv_signal",
    );
    Ok(res)
}

/// Calculate Chi-square
pub fn chi_square(params: SignalAlgorithmParams) -> Result<CallToolResult, McpError> {
    let table = to_table(&params.table);
    let chi_sq = calculate_chi_square(&table);

    // Chi-square critical values: 3.84 (p=0.05), 6.64 (p=0.01), 10.83 (p=0.001)
    let json = json!({
        "chi_square": chi_sq,
        "significant_0_05": chi_sq >= 3.84,
        "significant_0_01": chi_sq >= 6.64,
        "significant_0_001": chi_sq >= 10.83,
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Naranjo causality assessment
pub fn naranjo_quick(params: NaranjoParams) -> Result<CallToolResult, McpError> {
    // Map standardized API (1=Yes) to algorithm scores
    // Rechallenge: Yes = 2 points
    let rechallenge_score = if params.rechallenge == 1 {
        2
    } else {
        params.rechallenge
    };
    // Alternatives: Yes = -1 point, No = +1 point
    let alternatives_score = if params.alternatives == 1 {
        -1
    } else if params.alternatives == -1 {
        1
    } else {
        0
    };

    let result = calculate_naranjo_quick(
        params.temporal,
        params.dechallenge,
        rechallenge_score,
        alternatives_score,
        params.previous,
    );

    let json = json!({
        "score": result.score,
        "category": result.category.to_string(),
        "question_scores": result.question_scores,
        "interpretation": match result.category {
            nexcore_vigilance::pv::causality::NaranjoCategory::Definite => "Definite causal relationship (score >= 9)",
            nexcore_vigilance::pv::causality::NaranjoCategory::Probable => "Probable causal relationship (score 5-8)",
            nexcore_vigilance::pv::causality::NaranjoCategory::Possible => "Possible causal relationship (score 1-4)",
            nexcore_vigilance::pv::causality::NaranjoCategory::Doubtful => "Doubtful causal relationship (score <= 0)",
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// WHO-UMC causality assessment
pub fn who_umc_quick(params: WhoUmcParams) -> Result<CallToolResult, McpError> {
    let result = calculate_who_umc_quick(
        params.temporal,
        params.dechallenge,
        params.rechallenge,
        params.alternatives,
        params.plausibility,
    );

    let json = json!({
        "category": format!("{:?}", result.category),
        "description": result.description,
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}
