//! Kellnr Advanced Statistics computation tools (5).
//! Consolidated from kellnr-mcp/src/stats.rs.

use crate::params::kellnr::{
    KellnrStatsBayesianParams, KellnrStatsEntropyParams, KellnrStatsOlsParams,
    KellnrStatsPoissonCiParams, KellnrStatsWelchParams,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

fn json_result(value: serde_json::Value) -> CallToolResult {
    CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| "{}".into()),
    )])
}

fn mean(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    xs.iter().sum::<f64>() / xs.len() as f64
}

fn variance(xs: &[f64]) -> f64 {
    let m = mean(xs);
    xs.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (xs.len() as f64 - 1.0)
}

// Abramowitz & Stegun approximation for normal CDF
fn normal_cdf(x: f64) -> f64 {
    let t = 1.0 / (1.0 + 0.2316419 * x.abs());
    let d = 0.3989422804014327; // 1/sqrt(2*pi)
    let p = d
        * (-x * x / 2.0).exp()
        * (t * (0.319381530
            + t * (-0.356563782 + t * (1.781477937 + t * (-1.821255978 + t * 1.330274429)))));
    if x >= 0.0 { 1.0 - p } else { p }
}

// Approximate normal quantile (Beasley-Springer-Moro)
fn normal_quantile(p: f64) -> f64 {
    if p <= 0.0 {
        return f64::NEG_INFINITY;
    }
    if p >= 1.0 {
        return f64::INFINITY;
    }
    let t = if p < 0.5 {
        (-2.0 * p.ln()).sqrt()
    } else {
        (-2.0 * (1.0 - p).ln()).sqrt()
    };
    let c0 = 2.515517;
    let c1 = 0.802853;
    let c2 = 0.010328;
    let d1 = 1.432788;
    let d2 = 0.189269;
    let d3 = 0.001308;
    let q = t - (c0 + c1 * t + c2 * t * t) / (1.0 + d1 * t + d2 * t * t + d3 * t * t * t);
    if p < 0.5 { -q } else { q }
}

/// Welch t-test for two samples with unequal variance.
pub fn compute_stats_welch_ttest(
    params: KellnrStatsWelchParams,
) -> Result<CallToolResult, McpError> {
    let s1 = &params.sample1;
    let s2 = &params.sample2;
    if s1.len() < 2 || s2.len() < 2 {
        return Ok(json_result(
            json!({"success": false, "error": "each sample needs at least 2 values"}),
        ));
    }
    let n1 = s1.len() as f64;
    let n2 = s2.len() as f64;
    let m1 = mean(s1);
    let m2 = mean(s2);
    let v1 = variance(s1);
    let v2 = variance(s2);
    let se = (v1 / n1 + v2 / n2).sqrt();
    if se < 1e-15 {
        return Ok(json_result(
            json!({"success": false, "error": "zero variance in both samples"}),
        ));
    }
    let t = (m1 - m2) / se;
    let num = (v1 / n1 + v2 / n2).powi(2);
    let den = (v1 / n1).powi(2) / (n1 - 1.0) + (v2 / n2).powi(2) / (n2 - 1.0);
    let df = num / den;
    let p_approx = 2.0 * (1.0 - normal_cdf(t.abs()));
    Ok(json_result(json!({
        "success": true,
        "t_statistic": t,
        "degrees_of_freedom": df,
        "p_value_approx": p_approx,
        "mean1": m1,
        "mean2": m2,
        "variance1": v1,
        "variance2": v2,
        "significant_at_005": p_approx < 0.05
    })))
}

/// OLS linear regression: y = slope*x + intercept.
pub fn compute_stats_ols_regression(
    params: KellnrStatsOlsParams,
) -> Result<CallToolResult, McpError> {
    let x = &params.x;
    let y = &params.y;
    if x.len() != y.len() || x.len() < 2 {
        return Ok(json_result(
            json!({"success": false, "error": "x and y must have equal length >= 2"}),
        ));
    }
    let n = x.len() as f64;
    let mx = mean(x);
    let my = mean(y);
    let ss_xy: f64 = x.iter().zip(y).map(|(xi, yi)| (xi - mx) * (yi - my)).sum();
    let ss_xx: f64 = x.iter().map(|xi| (xi - mx).powi(2)).sum();
    if ss_xx < 1e-15 {
        return Ok(json_result(
            json!({"success": false, "error": "zero variance in x"}),
        ));
    }
    let slope = ss_xy / ss_xx;
    let intercept = my - slope * mx;
    let ss_res: f64 = x
        .iter()
        .zip(y)
        .map(|(xi, yi)| {
            let pred = slope * xi + intercept;
            (yi - pred).powi(2)
        })
        .sum();
    let ss_tot: f64 = y.iter().map(|yi| (yi - my).powi(2)).sum();
    let r_squared = if ss_tot > 1e-15 {
        1.0 - ss_res / ss_tot
    } else {
        0.0
    };
    let se_slope = if n > 2.0 {
        (ss_res / (n - 2.0) / ss_xx).sqrt()
    } else {
        f64::NAN
    };
    Ok(json_result(json!({
        "success": true,
        "slope": slope,
        "intercept": intercept,
        "r_squared": r_squared,
        "standard_error_slope": se_slope,
        "n": n as u64
    })))
}

/// Poisson confidence interval (Garwood exact approximation).
pub fn compute_stats_poisson_ci(
    params: KellnrStatsPoissonCiParams,
) -> Result<CallToolResult, McpError> {
    let count = params.count;
    let confidence = params.confidence_level.unwrap_or(0.95);
    let c = count as f64;
    let z = normal_quantile((1.0 + confidence) / 2.0);
    let lower = if count == 0 {
        0.0
    } else {
        (c.sqrt() - z / 2.0).powi(2).max(0.0)
    };
    let upper = (c.sqrt() + z / 2.0).powi(2);
    Ok(json_result(json!({
        "success": true,
        "count": count,
        "confidence_level": confidence,
        "lower_bound": lower,
        "upper_bound": upper,
        "rate": c,
        "z_score": z
    })))
}

/// Bayesian posterior with Beta-Binomial.
pub fn compute_stats_bayesian_posterior(
    params: KellnrStatsBayesianParams,
) -> Result<CallToolResult, McpError> {
    let alpha = params.prior_alpha.unwrap_or(1.0);
    let beta = params.prior_beta.unwrap_or(1.0);
    let post_alpha = alpha + params.successes as f64;
    let post_beta = beta + (params.trials - params.successes) as f64;
    let post_mean = post_alpha / (post_alpha + post_beta);
    let post_mode = if post_alpha > 1.0 && post_beta > 1.0 {
        (post_alpha - 1.0) / (post_alpha + post_beta - 2.0)
    } else {
        post_mean
    };
    let post_var = (post_alpha * post_beta)
        / ((post_alpha + post_beta).powi(2) * (post_alpha + post_beta + 1.0));
    let ci_lower = (post_mean - 1.96 * post_var.sqrt()).max(0.0);
    let ci_upper = (post_mean + 1.96 * post_var.sqrt()).min(1.0);
    Ok(json_result(json!({
        "success": true,
        "posterior_alpha": post_alpha,
        "posterior_beta": post_beta,
        "posterior_mean": post_mean,
        "posterior_mode": post_mode,
        "posterior_variance": post_var,
        "credible_interval_95": [ci_lower, ci_upper],
        "prior_alpha": alpha,
        "prior_beta": beta
    })))
}

/// Shannon entropy: H = -sum(p_i * log2(p_i)).
pub fn compute_stats_entropy(params: KellnrStatsEntropyParams) -> Result<CallToolResult, McpError> {
    let probs = &params.probabilities;
    let sum: f64 = probs.iter().sum();
    if (sum - 1.0).abs() > 0.01 {
        return Ok(json_result(
            json!({"success": false, "error": format!("probabilities sum to {sum}, expected ~1.0")}),
        ));
    }
    let h: f64 = probs
        .iter()
        .filter(|&&p| p > 0.0)
        .map(|&p| -p * p.log2())
        .sum();
    let max_entropy = (probs.len() as f64).log2();
    Ok(json_result(json!({
        "success": true,
        "entropy_bits": h,
        "max_entropy_bits": max_entropy,
        "normalized_entropy": if max_entropy > 0.0 { h / max_entropy } else { 0.0 },
        "n_categories": probs.len()
    })))
}
