//! Kellnr Sequential Surveillance computation tools (3).
//! Consolidated from kellnr-mcp/src/surveillance.rs.

use crate::params::kellnr::{
    KellnrSignalCusumParams, KellnrSignalSprtParams, KellnrSignalWeibullTtoParams,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

fn json_result(value: serde_json::Value) -> CallToolResult {
    CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| "{}".into()),
    )])
}

/// Sequential Probability Ratio Test (SPRT).
pub fn compute_signal_sprt(params: KellnrSignalSprtParams) -> Result<CallToolResult, McpError> {
    let expected = params.expected;
    if expected <= 0.0 {
        return Ok(json_result(
            json!({"success": false, "error": "expected must be > 0"}),
        ));
    }
    let alpha = params.alpha.unwrap_or(0.05);
    let beta = params.beta.unwrap_or(0.20);
    let o = params.observed as f64;
    let rr = o / expected;
    let llr = if o > 0.0 {
        o * (o / expected).ln() - (o - expected)
    } else {
        expected
    };
    let upper_boundary = ((1.0 - beta) / alpha).ln();
    let lower_boundary = (beta / (1.0 - alpha)).ln();
    let decision = if llr >= upper_boundary {
        "reject_null"
    } else if llr <= lower_boundary {
        "accept_null"
    } else {
        "continue"
    };
    Ok(json_result(json!({
        "success": true,
        "observed": params.observed,
        "expected": expected,
        "relative_risk": rr,
        "log_likelihood_ratio": llr,
        "upper_boundary": upper_boundary,
        "lower_boundary": lower_boundary,
        "decision": decision,
        "alpha": alpha,
        "beta": beta
    })))
}

/// CUSUM control chart.
pub fn compute_signal_cusum(params: KellnrSignalCusumParams) -> Result<CallToolResult, McpError> {
    let values = &params.values;
    if values.is_empty() {
        return Ok(json_result(
            json!({"success": false, "error": "values must be non-empty"}),
        ));
    }
    let target = params.target;
    let threshold = params.threshold.unwrap_or(5.0);
    let mut s_high = vec![0.0f64; values.len()];
    let mut s_low = vec![0.0f64; values.len()];
    let mut alerts_high = Vec::new();
    let mut alerts_low = Vec::new();
    for i in 0..values.len() {
        let prev_h = if i > 0 { s_high[i - 1] } else { 0.0 };
        let prev_l = if i > 0 { s_low[i - 1] } else { 0.0 };
        s_high[i] = (prev_h + (values[i] - target)).max(0.0);
        s_low[i] = (prev_l - (values[i] - target)).max(0.0);
        if s_high[i] > threshold {
            alerts_high.push(i);
        }
        if s_low[i] > threshold {
            alerts_low.push(i);
        }
    }
    Ok(json_result(json!({
        "success": true,
        "cusum_high": s_high,
        "cusum_low": s_low,
        "alerts_high": alerts_high,
        "alerts_low": alerts_low,
        "target": target,
        "threshold": threshold,
        "n": values.len(),
        "signal_detected": !alerts_high.is_empty() || !alerts_low.is_empty()
    })))
}

/// Weibull time-to-onset fit using method of moments.
pub fn compute_signal_weibull_tto(
    params: KellnrSignalWeibullTtoParams,
) -> Result<CallToolResult, McpError> {
    let onset_times = &params.onset_times;
    if onset_times.len() < 3 {
        return Ok(json_result(
            json!({"success": false, "error": "need at least 3 onset times"}),
        ));
    }
    let n = onset_times.len() as f64;
    let mean_t: f64 = onset_times.iter().sum::<f64>() / n;
    let var_t: f64 = onset_times
        .iter()
        .map(|&t| (t - mean_t).powi(2))
        .sum::<f64>()
        / (n - 1.0);
    let cv = var_t.sqrt() / mean_t;

    let shape = if cv > 0.01 { (1.2 / cv).max(0.1) } else { 10.0 };
    let scale = mean_t / gamma_approx(1.0 + 1.0 / shape);
    let median = scale * (2.0_f64.ln()).powf(1.0 / shape);
    let pattern = if shape < 1.0 {
        "early_onset"
    } else if shape > 1.5 {
        "late_onset"
    } else {
        "constant_rate"
    };
    Ok(json_result(json!({
        "success": true,
        "shape": shape,
        "scale": scale,
        "mean_onset": mean_t,
        "median_onset": median,
        "cv": cv,
        "pattern": pattern,
        "interpretation": match pattern {
            "early_onset" => "Decreasing hazard - events cluster early",
            "late_onset" => "Increasing hazard - events cluster later",
            _ => "Constant hazard - exponential-like distribution",
        },
        "n": onset_times.len()
    })))
}

// Lanczos approximation for Gamma function
#[allow(clippy::excessive_precision)]
fn gamma_approx(x: f64) -> f64 {
    if x < 0.5 {
        std::f64::consts::PI / ((std::f64::consts::PI * x).sin() * gamma_approx(1.0 - x))
    } else {
        let t = x - 1.0 + 7.5;
        let coeffs = [
            0.99999999999980993,
            676.5203681218851,
            -1259.1392167224028,
            771.32342877765313,
            -176.61502916214059,
            12.507343278686905,
            -0.13857109526572012,
            9.9843695780195716e-6,
            1.5056327351493116e-7,
        ];
        let mut sum = coeffs[0];
        for (i, &c) in coeffs[1..].iter().enumerate() {
            sum += c / (x + i as f64);
        }
        (2.0 * std::f64::consts::PI).sqrt() * t.powf(x - 0.5) * (-t).exp() * sum
    }
}
