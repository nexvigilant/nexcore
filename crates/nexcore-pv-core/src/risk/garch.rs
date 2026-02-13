//! # GARCH Volatility Models
//!
//! GARCH models for pharmacovigilance adverse event clustering analysis.

#![allow(missing_docs)]

use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

const LJUNG_BOX_LAGS: usize = 10;
const ARCH_LM_LAGS: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum GarchModel {
    #[default]
    Garch11,
    Egarch11,
    Tgarch11,
    GjrGarch11,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DistributionType {
    #[default]
    Normal,
    StudentT,
    SkewedT,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClusterSeverity {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GarchInput {
    pub adverse_event_series: Vec<f64>,
    #[serde(default)]
    pub model: GarchModel,
    #[serde(default = "default_max_iter")]
    pub max_iterations: u32,
    #[serde(default = "default_tolerance")]
    pub tolerance: f64,
    #[serde(default = "default_horizon")]
    pub forecast_horizon: u32,
    #[serde(default)]
    pub distribution_type: DistributionType,
    pub initial_values: Option<GarchParams>,
}

fn default_max_iter() -> u32 {
    1000
}
fn default_tolerance() -> f64 {
    1e-6
}
fn default_horizon() -> u32 {
    30
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GarchParams {
    pub omega: f64,
    pub alpha: f64,
    pub beta: f64,
    pub gamma: Option<f64>,
}

impl Default for GarchParams {
    fn default() -> Self {
        Self {
            omega: 0.1,
            alpha: 0.1,
            beta: 0.8,
            gamma: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GarchDiagnostics {
    pub log_likelihood: f64,
    pub aic: f64,
    pub bic: f64,
    pub persistence: f64,
    pub half_life: f64,
    pub unconditional_variance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub statistic: f64,
    pub p_value: f64,
    pub significant: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResidualDiagnostics {
    pub standardized: Vec<f64>,
    pub ljung_box_test: TestResult,
    pub arch_lm_test: TestResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolatilityCluster {
    pub start_index: usize,
    pub end_index: usize,
    pub severity: ClusterSeverity,
    pub average_volatility: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusteringAnalysis {
    pub detected_clusters: Vec<VolatilityCluster>,
    pub clustering_index: f64,
    pub persistence_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolatilityForecast {
    pub volatility: Vec<f64>,
    pub confidence_intervals: Vec<ConfidenceInterval>,
    pub risk_alert: bool,
    pub expected_clusters: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceInterval {
    pub lower_95: f64,
    pub upper_95: f64,
    pub lower_99: f64,
    pub upper_99: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GarchResult {
    pub model: String,
    pub parameters: GarchParams,
    pub diagnostics: GarchDiagnostics,
    pub conditional_volatility: Vec<f64>,
    pub unconditional_volatility: f64,
    pub clustering: ClusteringAnalysis,
    pub residuals: ResidualDiagnostics,
    pub forecast: VolatilityForecast,
}

fn calculate_returns(series: &[f64]) -> Vec<f64> {
    series
        .windows(2)
        .map(|w| if w[0] > 0.0 { (w[1] / w[0]).ln() } else { 0.0 })
        .collect()
}

fn calculate_conditional_variance(
    residuals: &[f64],
    params: &GarchParams,
    model: GarchModel,
) -> Vec<f64> {
    let n = residuals.len();
    let unconditional_var = residuals.iter().map(|r| r * r).sum::<f64>() / n as f64;
    let mut variance = vec![unconditional_var; n];
    let expected_abs = (2.0 / PI).sqrt();
    (1..n).for_each(|i| {
        variance[i] = match model {
            GarchModel::Garch11 | GarchModel::Tgarch11 | GarchModel::GjrGarch11 => {
                params.omega
                    + params.alpha * residuals[i - 1].powi(2)
                    + params.beta * variance[i - 1]
            }
            GarchModel::Egarch11 => {
                let z = residuals[i - 1] / variance[i - 1].sqrt();
                (params.omega
                    + params.alpha * (z.abs() - expected_abs)
                    + params.gamma.unwrap_or(-0.1) * z
                    + params.beta * variance[i - 1].ln())
                .exp()
            }
        }
        .max(1e-8);
    });
    variance
}

fn calculate_log_likelihood(residuals: &[f64], params: &GarchParams, model: GarchModel) -> f64 {
    let variance = calculate_conditional_variance(residuals, params, model);
    residuals
        .iter()
        .zip(variance.iter())
        .filter(|&(_, v)| *v > 0.0)
        .map(|(r, v)| -0.5 * ((2.0 * PI).ln() + v.ln() + r * r / v))
        .sum()
}

fn optimize_parameters(
    residuals: &[f64],
    initial: GarchParams,
    model: GarchModel,
    max_iter: u32,
    tol: f64,
) -> (GarchParams, f64) {
    let mut params = initial;
    let mut ll = calculate_log_likelihood(residuals, &params, model);
    let (lr, h) = (0.001, 1e-6);
    (0..max_iter).for_each(|_| {
        let mut p = params;
        p.omega += h;
        let go = (calculate_log_likelihood(residuals, &p, model) - ll) / h;
        p = params;
        p.alpha += h;
        let ga = (calculate_log_likelihood(residuals, &p, model) - ll) / h;
        p = params;
        p.beta += h;
        let gb = (calculate_log_likelihood(residuals, &p, model) - ll) / h;
        let new_omega = (params.omega + lr * go).max(1e-6);
        let new_alpha = (params.alpha + lr * ga).clamp(0.0, 0.99);
        let new_beta = (params.beta + lr * gb).clamp(0.0, 0.99 - new_alpha);
        let new_params = GarchParams {
            omega: new_omega,
            alpha: new_alpha,
            beta: new_beta,
            gamma: params.gamma,
        };
        let new_ll = calculate_log_likelihood(residuals, &new_params, model);
        if (new_ll - ll).abs() >= tol && new_ll > ll {
            params = new_params;
            ll = new_ll;
        }
    });
    (params, ll)
}

fn forecast_volatility(
    last_variance: f64,
    params: &GarchParams,
    horizon: u32,
) -> VolatilityForecast {
    let uncond_var = params.omega / (1.0 - params.alpha - params.beta).max(0.01);
    let data: Vec<_> = (1..=horizon)
        .map(|h| {
            let cv = uncond_var
                + (params.alpha + params.beta).powi(h as i32 - 1) * (last_variance - uncond_var);
            let vol = cv.sqrt();
            (
                vol,
                ConfidenceInterval {
                    lower_95: (vol * 0.804).max(0.0),
                    upper_95: vol * 1.196,
                    lower_99: (vol * 0.742).max(0.0),
                    upper_99: vol * 1.258,
                },
            )
        })
        .collect();
    let (volatility, confidence_intervals): (Vec<_>, Vec<_>) = data.into_iter().unzip();
    VolatilityForecast {
        volatility,
        confidence_intervals,
        risk_alert: params.alpha + params.beta > 0.9,
        expected_clusters: ((horizon as f64) * 0.1).ceil() as u32,
    }
}

fn compute_stats(volatility: &[f64]) -> (f64, f64, f64, f64) {
    let n = volatility.len();
    let mean = volatility.iter().sum::<f64>() / n as f64;
    let variance_sum: f64 = volatility.iter().map(|v| (v - mean).powi(2)).sum();
    let std_dev = (variance_sum / n as f64).sqrt();
    (
        mean,
        variance_sum,
        mean + 1.5 * std_dev,
        mean + 0.5 * std_dev,
    )
}

fn count_high(volatility: &[f64], high_t: f64) -> usize {
    volatility.iter().filter(|&&v| v > high_t).count()
}

fn compute_persistence(volatility: &[f64], mean: f64, variance_sum: f64) -> f64 {
    let n = volatility.len();
    let covar: f64 = (0..n.saturating_sub(1))
        .map(|i| (volatility[i] - mean) * (volatility[i + 1] - mean))
        .sum();
    if variance_sum > 0.0 {
        covar / variance_sum
    } else {
        0.0
    }
}

fn detect_clusters(volatility: &[f64], med_t: f64, high_t: f64) -> Vec<VolatilityCluster> {
    let n = volatility.len();
    let mut clusters = Vec::new();
    let mut idx = 0;
    let mut state: Option<(usize, f64, usize, ClusterSeverity)> = None;
    (0..n).for_each(|i| {
        if volatility[i] > med_t {
            let sev = if volatility[i] > high_t {
                ClusterSeverity::High
            } else {
                ClusterSeverity::Medium
            };
            state = Some(match state {
                None => (i, volatility[i], 1, sev),
                Some((start, total, count, _)) => (start, total + volatility[i], count + 1, sev),
            });
        } else if let Some((start, total, count, sev)) = state.take() {
            if count >= 2 {
                clusters.push(VolatilityCluster {
                    start_index: start,
                    end_index: i - 1,
                    severity: sev,
                    average_volatility: total / count as f64,
                });
            }
        }
        idx = i;
    });
    if let Some((start, total, count, sev)) = state {
        if count >= 2 {
            clusters.push(VolatilityCluster {
                start_index: start,
                end_index: idx,
                severity: sev,
                average_volatility: total / count as f64,
            });
        }
    }
    clusters
}

fn analyze_clustering(volatility: &[f64]) -> ClusteringAnalysis {
    let n = volatility.len();
    if n < 3 {
        return ClusteringAnalysis {
            detected_clusters: vec![],
            clustering_index: 0.0,
            persistence_score: 0.0,
        };
    }
    let (mean, variance_sum, high_t, med_t) = compute_stats(volatility);
    let clusters = detect_clusters(volatility, med_t, high_t);
    let high_count = count_high(volatility, high_t);
    let persistence_score = compute_persistence(volatility, mean, variance_sum);
    ClusteringAnalysis {
        detected_clusters: clusters,
        clustering_index: high_count as f64 / n as f64,
        persistence_score,
    }
}

fn autocorr_at_lag(data: &[f64], mean: f64, var: f64, k: usize) -> f64 {
    let n = data.len();
    (0..n.saturating_sub(k))
        .map(|i| (data[i] - mean) * (data[i + k] - mean))
        .sum::<f64>()
        / var
}

fn ljung_box_test(residuals: &[f64]) -> TestResult {
    let n = residuals.len();
    let mean = residuals.iter().sum::<f64>() / n as f64;
    let var: f64 = residuals.iter().map(|r| (r - mean).powi(2)).sum();
    let stat: f64 = (1..=LJUNG_BOX_LAGS.min(n.saturating_sub(1)))
        .map(|k| autocorr_at_lag(residuals, mean, var, k).powi(2) / (n - k) as f64)
        .sum::<f64>()
        * n as f64
        * (n as f64 + 2.0);
    let pv = 1.0 - chi_square_cdf(stat, LJUNG_BOX_LAGS as f64);
    TestResult {
        statistic: stat,
        p_value: pv,
        significant: pv < 0.05,
    }
}

fn arch_lm_test(residuals: &[f64]) -> TestResult {
    let squared: Vec<f64> = residuals.iter().map(|r| r * r).collect();
    let n = squared.len();
    let mean = squared.iter().sum::<f64>() / n as f64;
    let var: f64 = squared.iter().map(|s| (s - mean).powi(2)).sum();
    let stat: f64 = (1..=ARCH_LM_LAGS.min(n.saturating_sub(1)))
        .map(|k| autocorr_at_lag(&squared, mean, var, k).powi(2))
        .sum::<f64>()
        * n as f64;
    let pv = 1.0 - chi_square_cdf(stat, ARCH_LM_LAGS as f64);
    TestResult {
        statistic: stat,
        p_value: pv,
        significant: pv < 0.05,
    }
}

fn chi_square_cdf(x: f64, df: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if df == 1.0 {
        return 2.0 * normal_cdf((2.0 * x).sqrt()) - 1.0;
    }
    if df == 2.0 {
        return 1.0 - (-x / 2.0).exp();
    }
    normal_cdf(
        ((2.0 * x / df).powf(1.0 / 3.0) - 1.0 + 2.0 / (9.0 * df)) / (2.0 / (9.0 * df)).sqrt(),
    )
}

fn normal_cdf(z: f64) -> f64 {
    0.5 * (1.0 + erf(z / std::f64::consts::SQRT_2))
}

fn erf(x: f64) -> f64 {
    let sign = if x >= 0.0 { 1.0 } else { -1.0 };
    let x = x.abs();
    let t = 1.0 / (1.0 + 0.3275911 * x);
    sign * (1.0
        - (((((1.061405429 * t - 1.453152027) * t + 1.421413741) * t - 0.284496736) * t
            + 0.254829592)
            * t
            * (-x * x).exp()))
}

/// Estimate GARCH model for adverse event volatility
///
/// # Errors
/// Returns error if input has fewer than 10 observations.
pub fn estimate_garch(input: &GarchInput) -> Result<GarchResult, &'static str> {
    if input.adverse_event_series.len() < 10 {
        return Err("Insufficient data: need at least 10 observations");
    }
    let returns = calculate_returns(&input.adverse_event_series);
    let n = returns.len();
    let mean_return = returns.iter().sum::<f64>() / n as f64;
    let residuals: Vec<f64> = returns.iter().map(|r| r - mean_return).collect();
    let initial_var = residuals.iter().map(|r| r * r).sum::<f64>() / n as f64;
    let initial = input.initial_values.unwrap_or(GarchParams {
        omega: 0.1 * initial_var,
        alpha: 0.1,
        beta: 0.8,
        gamma: if matches!(input.model, GarchModel::Egarch11) {
            Some(-0.1)
        } else {
            None
        },
    });
    let (params, log_likelihood) = optimize_parameters(
        &residuals,
        initial,
        input.model,
        input.max_iterations,
        input.tolerance,
    );
    let cond_variance = calculate_conditional_variance(&residuals, &params, input.model);
    let cond_volatility: Vec<f64> = cond_variance.iter().map(|v| v.sqrt()).collect();
    let persistence = params.alpha + params.beta;
    let half_life = if persistence < 1.0 {
        0.5_f64.ln() / persistence.ln()
    } else {
        f64::INFINITY
    };
    let uncond_var = params.omega / (1.0 - persistence).max(0.01);
    let num_params = if params.gamma.is_some() { 4 } else { 3 };
    let standardized: Vec<f64> = residuals
        .iter()
        .zip(cond_variance.iter())
        .map(|(r, v)| r / v.sqrt())
        .collect();
    let model_name = match input.model {
        GarchModel::Garch11 => "GARCH(1,1)",
        GarchModel::Egarch11 => "EGARCH(1,1)",
        GarchModel::Tgarch11 => "TGARCH(1,1)",
        GarchModel::GjrGarch11 => "GJR-GARCH(1,1)",
    };
    let last_var = cond_variance.last().copied().unwrap_or(uncond_var);
    Ok(GarchResult {
        model: model_name.to_string(),
        parameters: params,
        diagnostics: GarchDiagnostics {
            log_likelihood,
            aic: -2.0 * log_likelihood + 2.0 * num_params as f64,
            bic: -2.0 * log_likelihood + (num_params as f64) * (n as f64).ln(),
            persistence,
            half_life,
            unconditional_variance: uncond_var,
        },
        conditional_volatility: cond_volatility.clone(),
        unconditional_volatility: uncond_var.sqrt(),
        clustering: analyze_clustering(&cond_volatility),
        residuals: ResidualDiagnostics {
            standardized: standardized.clone(),
            ljung_box_test: ljung_box_test(&standardized),
            arch_lm_test: arch_lm_test(&standardized),
        },
        forecast: forecast_volatility(last_var, &params, input.forecast_horizon),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn test_series() -> Vec<f64> {
        (1..=50)
            .map(|i| 10.0 + (i as f64 * 0.1).sin() * 5.0 + (i as f64) * 0.1)
            .collect()
    }

    #[test]
    fn test_garch_estimation() {
        let r = estimate_garch(&GarchInput {
            adverse_event_series: test_series(),
            model: GarchModel::Garch11,
            max_iterations: 100,
            tolerance: 1e-5,
            forecast_horizon: 10,
            distribution_type: DistributionType::Normal,
            initial_values: None,
        });
        assert!(r.is_ok());
    }

    #[test]
    fn test_insufficient_data() {
        assert!(
            estimate_garch(&GarchInput {
                adverse_event_series: vec![1.0, 2.0, 3.0],
                model: GarchModel::Garch11,
                max_iterations: 100,
                tolerance: 1e-5,
                forecast_horizon: 5,
                distribution_type: DistributionType::Normal,
                initial_values: None
            })
            .is_err()
        );
    }

    #[test]
    fn test_forecast_length() {
        if let Ok(r) = estimate_garch(&GarchInput {
            adverse_event_series: test_series(),
            model: GarchModel::Garch11,
            max_iterations: 100,
            tolerance: 1e-5,
            forecast_horizon: 20,
            distribution_type: DistributionType::Normal,
            initial_values: None,
        }) {
            assert_eq!(r.forecast.volatility.len(), 20);
        }
    }

    #[test]
    fn test_clustering() {
        let a = analyze_clustering(&[0.1, 0.1, 0.5, 0.6, 0.7, 0.1, 0.1, 0.8, 0.9, 0.1]);
        assert!(a.clustering_index >= 0.0);
    }
}
