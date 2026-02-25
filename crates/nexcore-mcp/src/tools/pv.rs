//! PV Signal Detection tools
//!
//! Pharmacovigilance signal detection and causality assessment.

use crate::params::{
    ContingencyTableParams, NaranjoParams, PvSignalCooperativeParams, SignalAlgorithmParams,
    SignalCompleteParams, WhoUmcParams,
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

/// Enrich a PV signal result with exact p-value and significance from stem-math.
///
/// Converts the SignalResult's point_estimate and CI into a statistical evidence block
/// carrying p-value, significance stars, and z-score.
fn enrich_signal(
    result: &nexcore_vigilance::pv::types::SignalResult,
    null_value: f64,
) -> serde_json::Value {
    // Approximate SE from CI width: SE ≈ (upper - lower) / (2 × 1.96) for 95% CI
    let se = if result.upper_ci > result.lower_ci {
        (result.upper_ci - result.lower_ci) / (2.0 * 1.96)
    } else {
        0.0
    };

    let outcome = if se > 0.0 {
        stem_math::statistics::StatisticalOutcome::two_tailed_95(
            result.point_estimate,
            null_value,
            se,
            "PV signal test",
        )
    } else {
        None
    };

    match outcome {
        Some(o) => json!({
            "point_estimate": result.point_estimate,
            "lower_ci": result.lower_ci,
            "upper_ci": result.upper_ci,
            "is_signal": result.is_signal,
            "z_score": o.z_score,
            "p_value": o.p_value,
            "significance": format!("{}", o.significance),
            "stars": o.significance.stars(),
        }),
        None => json!({
            "point_estimate": result.point_estimate,
            "lower_ci": result.lower_ci,
            "upper_ci": result.upper_ci,
            "is_signal": result.is_signal,
        }),
    }
}

/// Complete signal analysis with all algorithms
pub fn signal_complete(params: SignalCompleteParams) -> Result<CallToolResult, McpError> {
    let table = to_table(&params.table);
    let mut criteria = SignalCriteria::evans();
    criteria.prr_threshold = params.prr_threshold;
    criteria.min_cases = params.min_n;

    let result = evaluate_signal_complete(&table, &criteria);

    // Enrich each algorithm result with p-value and significance
    // PRR/ROR: null_value = 1.0 (ratio = 1 means no signal)
    // IC: null_value = 0.0 (IC = 0 means no information)
    // EBGM: null_value = 1.0 (expected ratio)
    let prr_enriched = enrich_signal(&result.prr, 1.0);
    let ror_enriched = enrich_signal(&result.ror, 1.0);
    let ic_enriched = enrich_signal(&result.ic, 0.0);
    let ebgm_enriched = enrich_signal(&result.ebgm, 1.0);

    // Compute exact chi-square p-value from stem-math
    let chi_sq_p = stem_math::statistics::chi_square_p_value(result.chi_square, 1).unwrap_or(1.0);
    let chi_sq_sig = stem_math::statistics::Significance::from_p(chi_sq_p);

    // IC variance transparency (Task A1)
    let (a, b, c, d) = (
        params.table.a as f64,
        params.table.b as f64,
        params.table.c as f64,
        params.table.d as f64,
    );
    let total = a + b + c + d;
    let n_expected = if total > 0.0 {
        (a + b) * (a + c) / total
    } else {
        0.5
    };
    let ic_variance = 1.0 / a.max(0.5) + 1.0 / n_expected.max(0.5);
    let ic_convergence_alpha = 1.0 - (ic_variance / ic_variance.max(1.0)).min(0.95);
    let ic_is_converged = ic_variance < 0.06;

    // Pareto frontier — sensitivity/specificity tradeoff profiles (Task D2)
    let sensitive_criteria = SignalCriteria::sensitive();
    let strict_criteria = SignalCriteria::strict();
    let result_sensitive = evaluate_signal_complete(&table, &sensitive_criteria);
    let result_strict = evaluate_signal_complete(&table, &strict_criteria);

    let build_profile = |r: &nexcore_vigilance::pv::types::CompleteSignalResult,
                         profile: &str,
                         description: &str| {
        let passing: Vec<&str> = [
            if r.prr.is_signal { Some("PRR") } else { None },
            if r.ror.is_signal { Some("ROR") } else { None },
            if r.ic.is_signal { Some("IC") } else { None },
            if r.ebgm.is_signal { Some("EBGM") } else { None },
        ]
        .into_iter()
        .flatten()
        .collect();
        json!({
            "profile": profile,
            "description": description,
            "signal_count": passing.len(),
            "any_signal": !passing.is_empty(),
            "passing_algorithms": passing,
        })
    };

    let pareto_frontier = vec![
        build_profile(
            &result_sensitive,
            "sensitive",
            "Lower thresholds: higher recall, more false positives",
        ),
        build_profile(
            &result,
            "balanced",
            "Evans criteria: standard regulatory thresholds",
        ),
        build_profile(
            &result_strict,
            "strict",
            "Higher thresholds: higher specificity, fewer false positives",
        ),
    ];

    // FDR metadata (Directive 003 Phase B)
    let fdr_metadata = if params.fdr_correction {
        Some(json!({
            "note": "Single-pair query: no cross-pair FDR correction applied. Use pv_core_fdr_adjust for batch correction.",
            "frequentist_methods": {
                "PRR": { "p_value": chi_sq_p, "correctable": true, "explanation": "Frequentist method — requires FDR correction in batch" },
                "ROR": { "p_value": chi_sq_p, "correctable": true, "explanation": "Frequentist method — requires FDR correction in batch" },
            },
            "bayesian_methods": {
                "IC": { "correctable": false, "explanation": "Bayesian method — uses built-in shrinkage, no FDR correction needed" },
                "EBGM": { "correctable": false, "explanation": "Bayesian method — uses built-in MGPS shrinkage, no FDR correction needed" },
            },
        }))
    } else {
        None
    };

    let mut json = json!({
        "prr": prr_enriched,
        "ror": ror_enriched,
        "ic": ic_enriched,
        "ebgm": ebgm_enriched,
        "chi_square": {
            "statistic": result.chi_square,
            "df": 1,
            "p_value": chi_sq_p,
            "significance": format!("{chi_sq_sig}"),
            "stars": chi_sq_sig.stars(),
            "exceeds_threshold": result.chi_square >= 3.841,
        },
        "n": result.n,
        "any_signal": result.prr.is_signal || result.ror.is_signal || result.ic.is_signal || result.ebgm.is_signal,
        "ic_transparency": {
            "ic_variance": ic_variance,
            "ic_convergence_alpha": ic_convergence_alpha,
            "ic_is_converged": ic_is_converged,
            "n_observed": a,
            "n_expected": n_expected,
        },
        "pareto_frontier": pareto_frontier,
    });

    if let Some(fdr) = fdr_metadata {
        json["fdr_correction"] = fdr;
    }

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

/// Cooperative signal strength for co-occurring weak signals.
///
/// When two drug-event signals share patient overlap, their combined evidence
/// follows a sigmoid cooperativity model. Below ~15% overlap, co-occurrence is
/// coincidental. Above ~40%, it's likely mechanistic.
pub fn signal_cooperative(params: PvSignalCooperativeParams) -> Result<CallToolResult, McpError> {
    let overlap = params.patient_overlap.clamp(0.0, 1.0);
    let lambda = 0.5; // empirical coupling constant
    let sigmoid = 1.0 / (1.0 + (-10.0_f64 * (overlap - 0.275)).exp());
    let ic_combined = params.ic_a + params.ic_b + lambda * sigmoid;

    let cooperation_class = if overlap < 0.15 {
        "coincidental"
    } else if overlap < 0.40 {
        "transitional"
    } else {
        "mechanistic"
    };

    let json = json!({
        "ic_a": params.ic_a,
        "ic_b": params.ic_b,
        "patient_overlap": overlap,
        "sigmoid_weight": sigmoid,
        "lambda": lambda,
        "cooperativity_bonus": lambda * sigmoid,
        "ic_combined": ic_combined,
        "cooperation_class": cooperation_class,
        "interpretation": match cooperation_class {
            "coincidental" => "Overlap < 15%: co-occurrence likely coincidental, minimal cooperativity bonus",
            "transitional" => "Overlap 15-40%: sigmoid transition zone, partial cooperativity",
            _ => "Overlap > 40%: likely shared mechanistic pathway, full cooperativity bonus",
        },
    });

    let confidence = if cooperation_class == "mechanistic" {
        0.85
    } else {
        0.5
    };
    let mut res = CallToolResult::success(vec![Content::text(json.to_string())]);
    attach_forensic_meta(&mut res, confidence, Some(ic_combined > 0.0), "pv_signal");
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
