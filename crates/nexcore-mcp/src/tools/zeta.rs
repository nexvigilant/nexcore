//! Zeta function and telescope pipeline MCP tools.
//!
//! Exposes the full nexcore-zeta crate via MCP: zero finding, telescope pipeline,
//! batch processing, scaling laws, Cayley transform, and operator hunt.

use nexcore_zeta::{
    self, BatchConfig, ScalingPoint, TelescopeConfig, cayley_anomaly_detect, cayley_transform,
    embedded_riemann_zeros_n, fit_scaling_law, hunt_operators, parse_lmfdb_zeros,
    predict_confidence, reconstruct_cmv, run_telescope, run_telescope_batch_raw,
};
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use stem_complex::Complex;

use crate::params::zeta::{
    ZetaBatchRunParams, ZetaCayleyParams, ZetaComputeParams, ZetaEmbeddedZerosParams,
    ZetaFindZerosParams, ZetaGueCompareParams, ZetaLmfdbParseParams, ZetaOperatorCandidateParams,
    ZetaOperatorHuntParams, ZetaScalingFitParams, ZetaScalingPredictParams, ZetaTelescopeRunParams,
    ZetaVerifyRhParams,
};

// ── Helper ───────────────────────────────────────────────────────────────────

fn ok_json(value: serde_json::Value) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| format!("{value}")),
    )]))
}

fn err_result(msg: &str) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::error(vec![rmcp::model::Content::text(
        msg.to_string(),
    )]))
}

fn find_zeros(t_min: f64, t_max: f64, step: f64) -> Result<Vec<nexcore_zeta::ZetaZero>, String> {
    nexcore_zeta::zeros::find_zeros_bracket(t_min, t_max, step).map_err(|e| format!("{e}"))
}

// ── Tools ────────────────────────────────────────────────────────────────────

/// Compute ζ(s) at a complex point.
pub fn zeta_compute(params: ZetaComputeParams) -> Result<CallToolResult, McpError> {
    let s = Complex::new(params.re, params.im);
    match nexcore_zeta::zeta::zeta(s) {
        Ok(z) => ok_json(serde_json::json!({
            "re": z.re,
            "im": z.im,
            "modulus": (z.re * z.re + z.im * z.im).sqrt(),
            "argument": z.im.atan2(z.re),
        })),
        Err(e) => err_result(&format!("zeta computation failed: {e}")),
    }
}

/// Find zeros of ζ(s) on the critical line in a height range.
pub fn zeta_find_zeros(params: ZetaFindZerosParams) -> Result<CallToolResult, McpError> {
    match find_zeros(params.t_min, params.t_max, params.step) {
        Ok(zeros) => ok_json(serde_json::json!({
            "count": zeros.len(),
            "zeros": zeros.iter().map(|z| serde_json::json!({
                "t": z.t,
                "z_value": z.z_value,
            })).collect::<Vec<_>>(),
        })),
        Err(e) => err_result(&format!("zero finding failed: {e}")),
    }
}

/// Verify the Riemann Hypothesis up to a given height.
pub fn zeta_verify_rh(params: ZetaVerifyRhParams) -> Result<CallToolResult, McpError> {
    match nexcore_zeta::zeros::verify_rh_to_height(params.max_height, params.step) {
        Ok(v) => ok_json(serde_json::json!({
            "all_on_critical_line": v.all_on_critical_line,
            "height": v.height,
            "expected_zeros": v.expected_zeros,
            "found_zeros": v.found_zeros,
            "zeros_count": v.zeros.len(),
        })),
        Err(e) => err_result(&format!("RH verification failed: {e}")),
    }
}

/// Get embedded Riemann zeros (Odlyzko tables, 9+ decimal precision).
pub fn zeta_embedded_zeros(params: ZetaEmbeddedZerosParams) -> Result<CallToolResult, McpError> {
    let zeros = embedded_riemann_zeros_n(params.count);
    ok_json(serde_json::json!({
        "count": zeros.len(),
        "zeros": zeros.iter().map(|z| serde_json::json!({
            "t": z.t,
            "z_value": z.z_value,
        })).collect::<Vec<_>>(),
    }))
}

/// Parse LMFDB zero data from JSON (supports raw array, labeled, or API response).
pub fn zeta_lmfdb_parse(params: ZetaLmfdbParseParams) -> Result<CallToolResult, McpError> {
    match parse_lmfdb_zeros(&params.json) {
        Ok(zset) => ok_json(serde_json::json!({
            "count": zset.zeros.len(),
            "fidelity": zset.fidelity,
            "parse_failures": zset.parse_failures,
            "zeros": zset.zeros.iter().map(|z| serde_json::json!({
                "t": z.t,
                "z_value": z.z_value,
            })).collect::<Vec<_>>(),
        })),
        Err(e) => err_result(&format!("LMFDB parse failed: {e}")),
    }
}

/// Run the full telescope pipeline on zeros in a height range.
pub fn zeta_telescope_run(params: ZetaTelescopeRunParams) -> Result<CallToolResult, McpError> {
    let zeros = match find_zeros(params.t_min, params.t_max, 0.05) {
        Ok(z) => z,
        Err(e) => return err_result(&format!("zero finding failed: {e}")),
    };

    if zeros.len() < 20 {
        return err_result(&format!(
            "need at least 20 zeros, found {} in [{}, {}]",
            zeros.len(),
            params.t_min,
            params.t_max
        ));
    }

    let config = TelescopeConfig {
        n_predict: params.n_predict,
        ..TelescopeConfig::default()
    };

    match run_telescope(&zeros, &config) {
        Ok(r) => ok_json(serde_json::json!({
            "n_zeros": r.n_zeros,
            "height_range": [r.height_range.0, r.height_range.1],
            "predictions": r.predictions,
            "prediction_accuracy": r.prediction_accuracy.as_ref().map(|a| serde_json::json!({
                "mae": a.mae, "rmse": a.rmse, "max_error": a.max_error, "n_compared": a.n_compared,
            })),
            "killip_nenciu": r.killip_nenciu.as_ref().map(|kn| serde_json::json!({
                "ks_statistic": kn.ks_statistic, "ks_pvalue": kn.ks_pvalue,
            })),
            "pair_correlation": r.pair_correlation.as_ref().map(|pc| serde_json::json!({
                "gue_match_score": pc.gue_match_score, "pair_correlation_mae": pc.pair_correlation_mae,
            })),
            "dual_verdict": r.dual_verdict.as_ref().map(|v| serde_json::json!({
                "interpretation": v.interpretation,
            })),
            "fingerprint": r.fingerprint.as_ref().map(|f| serde_json::json!({
                "decay_rate": f.decay_rate,
                "mean_magnitude": f.mean_magnitude,
                "roundtrip_fidelity": f.roundtrip_fidelity,
                "n_zeros": f.n_zeros,
            })),
            "anomaly": r.anomaly.as_ref().map(|a| serde_json::json!({
                "score": a.anomaly_score, "is_anomalous": a.is_anomalous,
            })),
            "overall_rh_confidence": r.overall_rh_confidence,
        })),
        Err(e) => err_result(&format!("telescope failed: {e}")),
    }
}

/// Run batch telescope on multiple height ranges.
pub fn zeta_batch_run(params: ZetaBatchRunParams) -> Result<CallToolResult, McpError> {
    let mut sets: Vec<(String, Vec<nexcore_zeta::ZetaZero>)> = Vec::new();

    for (i, &(t_min, t_max)) in params.ranges.iter().enumerate() {
        match find_zeros(t_min, t_max, 0.05) {
            Ok(zeros) if zeros.len() >= params.min_zeros => {
                sets.push((format!("range_{i}_{t_min}_{t_max}"), zeros));
            }
            Ok(_) | Err(_) => {}
        }
    }

    if sets.is_empty() {
        return err_result("no valid zero sets found from provided ranges");
    }

    let raw_sets: Vec<(&str, &[nexcore_zeta::ZetaZero])> = sets
        .iter()
        .map(|(l, z)| (l.as_str(), z.as_slice()))
        .collect();

    let config = BatchConfig {
        min_zeros: params.min_zeros,
        collect_reports: false,
        ..BatchConfig::default()
    };

    match run_telescope_batch_raw(&raw_sets, &config) {
        Ok(report) => ok_json(serde_json::json!({
            "n_ranges": report.entries.len(),
            "entries": report.entries.iter().map(|e| serde_json::json!({
                "label": e.label,
                "n_zeros": e.n_zeros,
                "rh_confidence": e.rh_confidence,
            })).collect::<Vec<_>>(),
            "statistics": {
                "mean_confidence": report.statistics.mean_confidence,
                "min_confidence": report.statistics.min_confidence,
                "max_confidence": report.statistics.max_confidence,
            },
        })),
        Err(e) => err_result(&format!("batch telescope failed: {e}")),
    }
}

/// Fit scaling law C(N) = 1 - a·N^(-b) to confidence data.
pub fn zeta_scaling_fit(params: ZetaScalingFitParams) -> Result<CallToolResult, McpError> {
    let points: Vec<ScalingPoint> = params
        .points
        .iter()
        .map(|&(n, c)| ScalingPoint { n, confidence: c })
        .collect();

    match fit_scaling_law(&points) {
        Ok(law) => ok_json(serde_json::json!({
            "a": law.a,
            "b": law.b,
            "r_squared": law.r_squared,
            "n_points": law.n_points,
            "confidence_at_10k": predict_confidence(&law, 10_000),
            "confidence_at_100k": predict_confidence(&law, 100_000),
        })),
        Err(e) => err_result(&format!("scaling fit failed: {e}")),
    }
}

/// Predict confidence at a given N using fitted scaling parameters.
pub fn zeta_scaling_predict(params: ZetaScalingPredictParams) -> Result<CallToolResult, McpError> {
    let law = nexcore_zeta::ScalingLaw {
        a: params.a,
        b: params.b,
        r_squared: 1.0,
        n_points: 0,
        residuals: vec![],
    };
    let confidence = predict_confidence(&law, params.n);
    ok_json(serde_json::json!({
        "n": params.n,
        "predicted_confidence": confidence,
        "a": params.a,
        "b": params.b,
    }))
}

/// Run Cayley transform anomaly detection on zeros in a height range.
pub fn zeta_cayley(params: ZetaCayleyParams) -> Result<CallToolResult, McpError> {
    let zeros = match find_zeros(params.t_min, params.t_max, 0.05) {
        Ok(z) => z,
        Err(e) => return err_result(&format!("zero finding failed: {e}")),
    };

    if zeros.len() < 20 {
        return err_result(&format!("need at least 20 zeros, found {}", zeros.len()));
    }

    let cmv = match reconstruct_cmv(&zeros) {
        Ok(c) => c,
        Err(e) => return err_result(&format!("CMV reconstruction failed: {e}")),
    };

    let ct = match cayley_transform(&cmv) {
        Ok(c) => c,
        Err(e) => return err_result(&format!("Cayley transform failed: {e}")),
    };

    let anomaly = match cayley_anomaly_detect(&cmv, &zeros) {
        Ok(a) => a,
        Err(e) => return err_result(&format!("Cayley anomaly detection failed: {e}")),
    };

    ok_json(serde_json::json!({
        "n_eigenvalues": ct.eigenvalues.len(),
        "condition_number": ct.condition_number,
        "n_unstable": ct.n_unstable,
        "anomaly_score": anomaly.anomaly_score,
        "is_anomalous": anomaly.is_anomalous,
        "kl_divergence": anomaly.kl_divergence,
        "n_deviations": anomaly.deviations.len(),
    }))
}

/// Hunt for Hilbert-Pólya operator candidates across all 3 methods.
pub fn zeta_operator_hunt(params: ZetaOperatorHuntParams) -> Result<CallToolResult, McpError> {
    let zeros = match find_zeros(params.t_min, params.t_max, 0.05) {
        Ok(z) => z,
        Err(e) => return err_result(&format!("zero finding failed: {e}")),
    };

    if zeros.len() < 10 {
        return err_result(&format!("need at least 10 zeros, found {}", zeros.len()));
    }

    match hunt_operators(&zeros) {
        Ok(r) => ok_json(serde_json::json!({
            "n_zeros": r.n_zeros,
            "best_candidate": r.best_candidate,
            "best_rmse": r.best_rmse,
            "candidates": r.candidates.iter().map(|c| serde_json::json!({
                "name": c.name,
                "rmse": c.rmse,
                "mae": c.mae,
                "max_error": c.max_error,
                "correlation": c.correlation,
                "n_compared": c.n_compared,
            })).collect::<Vec<_>>(),
        })),
        Err(e) => err_result(&format!("operator hunt failed: {e}")),
    }
}

/// Run a specific operator candidate.
pub fn zeta_operator_candidate(
    params: ZetaOperatorCandidateParams,
) -> Result<CallToolResult, McpError> {
    let zeros = match find_zeros(params.t_min, params.t_max, 0.05) {
        Ok(z) => z,
        Err(e) => return err_result(&format!("zero finding failed: {e}")),
    };

    if zeros.len() < 10 {
        return err_result(&format!("need at least 10 zeros, found {}", zeros.len()));
    }

    let targets: Vec<f64> = zeros.iter().map(|z| z.t).collect();

    let fit = match params.operator.as_str() {
        "berry_keating" => nexcore_zeta::berry_keating_xp(zeros.len(), &targets),
        "xp_potential" => nexcore_zeta::xp_plus_potential(zeros.len(), &targets),
        "cmv_truncation" => nexcore_zeta::cmv_truncated_operator(&zeros),
        other => {
            return err_result(&format!(
                "unknown operator '{}'. Use: berry_keating, xp_potential, cmv_truncation",
                other
            ));
        }
    };

    match fit {
        Ok(f) => ok_json(serde_json::json!({
            "name": f.name,
            "rmse": f.rmse,
            "mae": f.mae,
            "max_error": f.max_error,
            "correlation": f.correlation,
            "n_compared": f.n_compared,
        })),
        Err(e) => err_result(&format!("operator failed: {e}")),
    }
}

/// Compare zero spacings to GUE random matrix predictions.
pub fn zeta_gue_compare(params: ZetaGueCompareParams) -> Result<CallToolResult, McpError> {
    let zeros = match find_zeros(params.t_min, params.t_max, 0.05) {
        Ok(z) => z,
        Err(e) => return err_result(&format!("zero finding failed: {e}")),
    };

    match nexcore_zeta::compare_to_gue(&zeros) {
        Ok(gc) => ok_json(serde_json::json!({
            "n_spacings": gc.n_spacings,
            "gue_match_score": gc.gue_match_score,
            "pair_correlation_mae": gc.pair_correlation_mae,
            "mean_spacing": gc.mean_spacing,
            "variance": gc.variance,
            "gue_predicted_variance": gc.gue_predicted_variance,
        })),
        Err(e) => err_result(&format!("GUE comparison failed: {e}")),
    }
}
