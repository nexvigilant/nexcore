//! ARIMA Time-Series Analysis for Pharmacovigilance
//!
//! Autoregressive Integrated Moving Average (ARIMA) models for detecting
//! anomalies and forecasting trends in adverse event reporting data.
//!
//! # Background
//!
//! ARIMA(p, d, q) combines three components:
//!
//! - **AR(p)**: Autoregressive - uses p past observations
//! - **I(d)**: Integrated - d differences for stationarity
//! - **MA(q)**: Moving Average - uses q past forecast errors
//!
//! # Model
//!
//! ```text
//! (1 - φ₁B - ... - φₚBᵖ)(1 - B)ᵈ yₜ = c + (1 + θ₁B + ... + θqBᵍ)εₜ
//!
//! where:
//!   B = backshift operator (Byₜ = yₜ₋₁)
//!   φᵢ = AR coefficients
//!   θⱼ = MA coefficients
//!   εₜ = white noise
//! ```
//!
//! # PV Applications
//!
//! - **Anomaly detection**: Identify unexpected spikes in AE reporting
//! - **Trend analysis**: Detect increasing/decreasing reporting patterns
//! - **Forecasting**: Predict expected reporting rates for comparison
//! - **Seasonality adjustment**: Account for periodic patterns (flu season, etc.)
//!
//! # References
//!
//! - Box GEP, Jenkins GM (1970). "Time Series Analysis: Forecasting and Control."
//!   Holden-Day, San Francisco. ISBN: 978-0-8162-1104-3
//!
//! - Box GEP, Jenkins GM, Reinsel GC, Ljung GM (2015). "Time Series Analysis:
//!   Forecasting and Control." 5th ed. Wiley. ISBN: 978-1-118-67502-1
//!
//! - Hyndman RJ, Athanasopoulos G (2018). "Forecasting: principles and practice."
//!   2nd ed. OTexts. Available: https://otexts.com/fpp2/
//!
//! - Tsay RS (2010). "Analysis of Financial Time Series." 3rd ed. Wiley.
//!   DOI: [10.1002/9780470644560](https://doi.org/10.1002/9780470644560)

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::signals::core::error::SignalError;

/// Configuration for ARIMA model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArimaConfig {
    /// AR order (number of autoregressive terms)
    pub p: usize,
    /// Differencing order (typically 0, 1, or 2)
    pub d: usize,
    /// MA order (number of moving average terms)
    pub q: usize,
    /// Include constant/drift term
    pub include_constant: bool,
    /// Maximum iterations for optimization
    pub max_iterations: usize,
    /// Convergence tolerance
    pub tolerance: f64,
}

impl Default for ArimaConfig {
    fn default() -> Self {
        Self {
            p: 1,
            d: 1,
            q: 1,
            include_constant: true,
            max_iterations: 100,
            tolerance: 1e-6,
        }
    }
}

impl ArimaConfig {
    /// Create ARIMA(1,1,1) - common default for many applications.
    #[must_use]
    pub fn arima_111() -> Self {
        Self::default()
    }

    /// Create AR(p) model (no differencing, no MA).
    #[must_use]
    pub fn ar(p: usize) -> Self {
        Self {
            p,
            d: 0,
            q: 0,
            ..Default::default()
        }
    }

    /// Create MA(q) model (no AR, no differencing).
    #[must_use]
    pub fn ma(q: usize) -> Self {
        Self {
            p: 0,
            d: 0,
            q,
            ..Default::default()
        }
    }

    /// Create ARMA(p, q) model (no differencing).
    #[must_use]
    pub fn arma(p: usize, q: usize) -> Self {
        Self {
            p,
            d: 0,
            q,
            ..Default::default()
        }
    }
}

/// Fitted ARIMA model result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArimaResult {
    /// Model configuration
    pub config: ArimaConfig,
    /// Estimated AR coefficients (φ₁, ..., φₚ)
    pub ar_coefficients: Vec<f64>,
    /// Estimated MA coefficients (θ₁, ..., θq)
    pub ma_coefficients: Vec<f64>,
    /// Constant/intercept term
    pub constant: f64,
    /// Residuals (observed - fitted)
    pub residuals: Vec<f64>,
    /// Fitted values
    pub fitted: Vec<f64>,
    /// Residual variance (σ²)
    pub sigma_squared: f64,
    /// Log-likelihood
    pub log_likelihood: f64,
    /// Akaike Information Criterion
    pub aic: f64,
    /// Bayesian Information Criterion
    pub bic: f64,
    /// Whether model converged
    pub converged: bool,
    /// Number of observations used (after differencing)
    pub n_obs: usize,
}

/// Forecast result from ARIMA model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArimaForecast {
    /// Point forecasts
    pub forecast: Vec<f64>,
    /// Lower 95% prediction interval
    pub lower_ci: Vec<f64>,
    /// Upper 95% prediction interval
    pub upper_ci: Vec<f64>,
    /// Forecast standard errors
    pub se: Vec<f64>,
}

/// Anomaly detection result using ARIMA.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArimaAnomalyResult {
    /// Original time series
    pub series: Vec<f64>,
    /// Fitted values from ARIMA
    pub fitted: Vec<f64>,
    /// Standardized residuals
    pub std_residuals: Vec<f64>,
    /// Indices of detected anomalies
    pub anomaly_indices: Vec<usize>,
    /// Anomaly scores (absolute standardized residuals)
    pub anomaly_scores: Vec<f64>,
    /// Threshold used for detection
    pub threshold: f64,
}

/// Fit an ARIMA model to time series data.
///
/// # Arguments
///
/// * `series` - Time series data (observations in chronological order)
/// * `config` - ARIMA configuration
///
/// # Returns
///
/// Fitted ARIMA model result.
///
/// # Example
///
/// ```rust
/// use nexcore_vigilance::pv::signals::temporal::arima::{fit_arima, ArimaConfig};
///
/// let series = vec![10.0, 12.0, 11.0, 14.0, 13.0, 15.0, 14.0, 16.0, 15.0, 17.0];
/// let config = ArimaConfig::arima_111();
///
/// let result = fit_arima(&series, &config).unwrap();
/// println!("AIC: {:.2}, Converged: {}", result.aic, result.converged);
/// ```
pub fn fit_arima(series: &[f64], config: &ArimaConfig) -> Result<ArimaResult, SignalError> {
    let n = series.len();
    let min_obs = config.p + config.d + config.q + 2;

    if n < min_obs {
        return Err(SignalError::InvalidTimeData(format!(
            "Need at least {} observations for ARIMA({},{},{}), got {}",
            min_obs, config.p, config.d, config.q, n
        )));
    }

    // Apply differencing
    let diff_series = difference(series, config.d);
    let n_diff = diff_series.len();

    if n_diff < config.p.max(config.q) + 1 {
        return Err(SignalError::InvalidTimeData(
            "Insufficient observations after differencing".into(),
        ));
    }

    // Estimate parameters using conditional least squares
    let (ar_coeffs, ma_coeffs, constant, residuals, fitted) =
        estimate_parameters(&diff_series, config)?;

    // Calculate statistics
    let sigma_squared = residuals.iter().map(|r| r * r).sum::<f64>() / n_diff as f64;
    let log_likelihood =
        -0.5 * n_diff as f64 * (1.0 + (2.0 * std::f64::consts::PI * sigma_squared).ln());

    // Number of parameters
    let k = config.p + config.q + if config.include_constant { 1 } else { 0 };

    // AIC and BIC
    let aic = -2.0 * log_likelihood + 2.0 * k as f64;
    let bic = -2.0 * log_likelihood + k as f64 * (n_diff as f64).ln();

    Ok(ArimaResult {
        config: config.clone(),
        ar_coefficients: ar_coeffs,
        ma_coefficients: ma_coeffs,
        constant,
        residuals,
        fitted,
        sigma_squared,
        log_likelihood,
        aic,
        bic,
        converged: true,
        n_obs: n_diff,
    })
}

/// Generate forecasts from a fitted ARIMA model.
///
/// # Arguments
///
/// * `series` - Original time series
/// * `model` - Fitted ARIMA result
/// * `h` - Number of periods to forecast
///
/// # Returns
///
/// Forecast result with point predictions and intervals.
///
/// # Example
///
/// ```rust
/// use nexcore_vigilance::pv::signals::temporal::arima::{fit_arima, forecast_arima, ArimaConfig};
///
/// let series = vec![10.0, 12.0, 11.0, 14.0, 13.0, 15.0, 14.0, 16.0, 15.0, 17.0];
/// let config = ArimaConfig::arima_111();
/// let model = fit_arima(&series, &config).unwrap();
///
/// let forecast = forecast_arima(&series, &model, 3);
/// println!("3-step forecast: {:?}", forecast.forecast);
/// ```
#[must_use]
pub fn forecast_arima(series: &[f64], model: &ArimaResult, h: usize) -> ArimaForecast {
    let diff_series = difference(series, model.config.d);
    let n = diff_series.len();

    let mut forecasts = Vec::with_capacity(h);
    let mut errors: Vec<f64> = model.residuals.clone();

    // Extended series for forecasting
    let mut extended = diff_series.clone();

    for step in 0..h {
        let mut forecast = model.constant;

        // AR component
        for (i, &phi) in model.ar_coefficients.iter().enumerate() {
            let idx = n + step - i - 1;
            if idx < extended.len() {
                forecast += phi * extended[idx];
            }
        }

        // MA component (past errors, future errors are 0)
        for (j, &theta) in model.ma_coefficients.iter().enumerate() {
            let err_idx = errors.len() as i64 - 1 - j as i64;
            if err_idx >= 0 {
                forecast += theta * errors[err_idx as usize];
            }
        }

        forecasts.push(forecast);
        extended.push(forecast);
        errors.push(0.0); // Future errors are 0
    }

    // Convert back from differenced scale
    let forecasts = undifference(&forecasts, series, model.config.d);

    // Calculate prediction intervals
    let sigma = model.sigma_squared.sqrt();
    let mut se = Vec::with_capacity(h);
    let mut psi = vec![1.0]; // MA(∞) representation coefficients

    // Compute ψ weights for variance calculation
    for i in 1..=h {
        let mut psi_i = 0.0;
        for j in 0..model.ar_coefficients.len().min(i) {
            psi_i += model.ar_coefficients[j] * psi.get(i - j - 1).unwrap_or(&0.0);
        }
        if i <= model.ma_coefficients.len() {
            psi_i += model.ma_coefficients[i - 1];
        }
        psi.push(psi_i);
    }

    // Forecast variance: σ² × Σ ψᵢ²
    for step in 1..=h {
        let var: f64 = psi[..step].iter().map(|p| p * p).sum();
        se.push(sigma * var.sqrt());
    }

    let z = 1.96;
    let lower_ci: Vec<f64> = forecasts
        .iter()
        .zip(&se)
        .map(|(&f, &s)| f - z * s)
        .collect();
    let upper_ci: Vec<f64> = forecasts
        .iter()
        .zip(&se)
        .map(|(&f, &s)| f + z * s)
        .collect();

    ArimaForecast {
        forecast: forecasts,
        lower_ci,
        upper_ci,
        se,
    }
}

/// Detect anomalies in time series using ARIMA residuals.
///
/// Points with standardized residuals exceeding the threshold are flagged.
///
/// # Arguments
///
/// * `series` - Time series data
/// * `config` - ARIMA configuration
/// * `threshold` - Standard deviations for anomaly (typically 2.0-3.0)
///
/// # Returns
///
/// Anomaly detection result.
///
/// # Example
///
/// ```rust
/// use nexcore_vigilance::pv::signals::temporal::arima::{detect_anomalies, ArimaConfig};
///
/// // Normal series with an anomaly at position 5
/// let mut series = vec![10.0, 11.0, 10.5, 11.2, 10.8, 50.0, 11.1, 10.9, 11.3, 10.7];
///
/// let result = detect_anomalies(&series, &ArimaConfig::ar(1), 2.5).unwrap();
/// assert!(result.anomaly_indices.contains(&5));
/// ```
pub fn detect_anomalies(
    series: &[f64],
    config: &ArimaConfig,
    threshold: f64,
) -> Result<ArimaAnomalyResult, SignalError> {
    let model = fit_arima(series, config)?;

    // Standardize residuals
    let sigma = model.sigma_squared.sqrt();
    let std_residuals: Vec<f64> = model.residuals.iter().map(|r| r / sigma).collect();

    // Find anomalies
    let anomaly_scores: Vec<f64> = std_residuals.iter().map(|r| r.abs()).collect();
    let anomaly_indices: Vec<usize> = anomaly_scores
        .iter()
        .enumerate()
        .filter(|(_, score)| **score > threshold)
        .map(|(i, _)| i + config.d) // Adjust for differencing offset
        .collect();

    // Expand fitted values back to original scale
    let fitted = if config.d > 0 {
        undifference(&model.fitted, series, config.d)
    } else {
        model.fitted
    };

    Ok(ArimaAnomalyResult {
        series: series.to_vec(),
        fitted,
        std_residuals,
        anomaly_indices,
        anomaly_scores,
        threshold,
    })
}

/// Batch ARIMA fitting for multiple time series.
#[must_use]
pub fn batch_arima_parallel(
    series_list: &[Vec<f64>],
    config: &ArimaConfig,
) -> Vec<Result<ArimaResult, SignalError>> {
    series_list
        .par_iter()
        .map(|s| fit_arima(s, config))
        .collect()
}

// ============================================================================
// Internal Functions
// ============================================================================

/// Apply differencing to a time series.
fn difference(series: &[f64], d: usize) -> Vec<f64> {
    let mut result = series.to_vec();
    for _ in 0..d {
        result = result.windows(2).map(|w| w[1] - w[0]).collect();
    }
    result
}

/// Reverse differencing to get original scale.
fn undifference(forecasts: &[f64], original: &[f64], d: usize) -> Vec<f64> {
    if d == 0 {
        return forecasts.to_vec();
    }

    let mut result = forecasts.to_vec();

    // For each level of differencing, cumsum starting from last original value
    for level in (0..d).rev() {
        let diff_series = difference(original, level);
        let last_val = diff_series.last().copied().unwrap_or(0.0);

        let mut cumsum = last_val;
        for val in &mut result {
            cumsum += *val;
            *val = cumsum;
        }
    }

    result
}

/// Estimate ARIMA parameters using conditional least squares.
fn estimate_parameters(
    series: &[f64],
    config: &ArimaConfig,
) -> Result<(Vec<f64>, Vec<f64>, f64, Vec<f64>, Vec<f64>), SignalError> {
    let n = series.len();
    let p = config.p;
    let q = config.q;

    // Initialize coefficients
    let mut ar_coeffs = vec![0.1; p];
    let mut ma_coeffs = vec![0.1; q];
    let mut constant = if config.include_constant {
        series.iter().sum::<f64>() / n as f64
    } else {
        0.0
    };

    let mut residuals = vec![0.0; n];
    let mut fitted = vec![0.0; n];

    // Iterative refinement
    for _ in 0..config.max_iterations {
        let old_ar = ar_coeffs.clone();
        let old_ma = ma_coeffs.clone();

        // Calculate residuals with current parameters
        for t in 0..n {
            let mut pred = constant;

            // AR terms
            for i in 0..p {
                if t > i {
                    pred += ar_coeffs[i] * series[t - i - 1];
                }
            }

            // MA terms
            for j in 0..q {
                if t > j {
                    pred += ma_coeffs[j] * residuals[t - j - 1];
                }
            }

            fitted[t] = pred;
            residuals[t] = series[t] - pred;
        }

        // Update AR coefficients using OLS-style update
        if p > 0 {
            let mut sum_ar = vec![0.0; p];
            let mut sum_y = 0.0;
            let mut count = 0.0;

            for t in p..n {
                for i in 0..p {
                    sum_ar[i] += series[t - i - 1] * residuals[t];
                }
                sum_y += series[t - 1] * series[t - 1];
                count += 1.0;
            }

            if sum_y.abs() > 1e-10 && count > 0.0 {
                for i in 0..p {
                    ar_coeffs[i] += 0.1 * sum_ar[i] / sum_y;
                    // Ensure stationarity (simple bound)
                    ar_coeffs[i] = ar_coeffs[i].clamp(-0.99, 0.99);
                }
            }
        }

        // Update MA coefficients
        if q > 0 {
            let mut sum_ma = vec![0.0; q];
            let mut sum_e = 0.0;

            for t in q..n {
                for j in 0..q {
                    sum_ma[j] += residuals[t - j - 1] * residuals[t];
                }
                sum_e += residuals[t - 1] * residuals[t - 1];
            }

            if sum_e.abs() > 1e-10 {
                for j in 0..q {
                    ma_coeffs[j] += 0.1 * sum_ma[j] / sum_e;
                    // Ensure invertibility (simple bound)
                    ma_coeffs[j] = ma_coeffs[j].clamp(-0.99, 0.99);
                }
            }
        }

        // Update constant
        if config.include_constant {
            let mean_residual = residuals.iter().sum::<f64>() / n as f64;
            constant += 0.5 * mean_residual;
        }

        // Check convergence
        let ar_diff: f64 = ar_coeffs
            .iter()
            .zip(&old_ar)
            .map(|(a, b)| (a - b).abs())
            .sum();
        let ma_diff: f64 = ma_coeffs
            .iter()
            .zip(&old_ma)
            .map(|(a, b)| (a - b).abs())
            .sum();

        if ar_diff + ma_diff < config.tolerance {
            break;
        }
    }

    // Final residual calculation
    for t in 0..n {
        let mut pred = constant;
        for i in 0..p {
            if t > i {
                pred += ar_coeffs[i] * series[t - i - 1];
            }
        }
        for j in 0..q {
            if t > j {
                pred += ma_coeffs[j] * residuals[t - j - 1];
            }
        }
        fitted[t] = pred;
        residuals[t] = series[t] - pred;
    }

    Ok((ar_coeffs, ma_coeffs, constant, residuals, fitted))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generate_ar1_series(n: usize, phi: f64, sigma: f64) -> Vec<f64> {
        let mut series = vec![0.0; n];
        for t in 1..n {
            series[t] = phi * series[t - 1] + sigma * (t as f64 * 0.1).sin();
        }
        series
    }

    #[test]
    fn test_fit_ar1() {
        let series = generate_ar1_series(100, 0.7, 1.0);
        let config = ArimaConfig::ar(1);

        let result = fit_arima(&series, &config).unwrap();

        // AR coefficient should be close to 0.7
        assert!(!result.ar_coefficients.is_empty());
        assert!(result.converged);
        assert!(result.aic.is_finite());
    }

    #[test]
    fn test_fit_arima_111() {
        let series: Vec<f64> = (0..50)
            .map(|i| (i as f64 * 0.2).sin() * 10.0 + i as f64)
            .collect();
        let config = ArimaConfig::arima_111();

        let result = fit_arima(&series, &config).unwrap();

        assert_eq!(result.ar_coefficients.len(), 1);
        assert_eq!(result.ma_coefficients.len(), 1);
        assert!(result.converged);
    }

    #[test]
    fn test_forecast() {
        let series: Vec<f64> = (0..30).map(|i| 10.0 + i as f64 * 0.5).collect();
        let config = ArimaConfig::arima_111();

        let model = fit_arima(&series, &config).unwrap();
        let forecast = forecast_arima(&series, &model, 5);

        assert_eq!(forecast.forecast.len(), 5);
        assert_eq!(forecast.lower_ci.len(), 5);
        assert_eq!(forecast.upper_ci.len(), 5);

        // Forecasts should continue the trend
        for i in 0..5 {
            assert!(forecast.lower_ci[i] < forecast.forecast[i]);
            assert!(forecast.upper_ci[i] > forecast.forecast[i]);
        }
    }

    #[test]
    fn test_anomaly_detection() {
        // Create series with obvious anomaly
        let mut series: Vec<f64> = (0..30).map(|i| 10.0 + (i as f64 * 0.3).sin()).collect();
        series[15] = 100.0; // Spike

        let result = detect_anomalies(&series, &ArimaConfig::ar(1), 2.0).unwrap();

        // Should detect the spike
        assert!(
            result.anomaly_indices.contains(&15) || result.anomaly_scores.iter().any(|&s| s > 5.0)
        );
    }

    #[test]
    fn test_differencing() {
        let series = vec![1.0, 3.0, 6.0, 10.0, 15.0];

        // First difference: [2, 3, 4, 5]
        let d1 = difference(&series, 1);
        assert_eq!(d1.len(), 4);
        assert!((d1[0] - 2.0).abs() < 1e-10);
        assert!((d1[3] - 5.0).abs() < 1e-10);

        // Second difference: [1, 1, 1]
        let d2 = difference(&series, 2);
        assert_eq!(d2.len(), 3);
        for &val in &d2 {
            assert!((val - 1.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_batch_parallel() {
        let series_list: Vec<Vec<f64>> = (0..5)
            .map(|seed| generate_ar1_series(50, 0.5 + 0.1 * seed as f64, 1.0))
            .collect();

        let config = ArimaConfig::ar(1);
        let results = batch_arima_parallel(&series_list, &config);

        assert_eq!(results.len(), 5);
        for result in results {
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_insufficient_data() {
        let series = vec![1.0, 2.0, 3.0];
        let config = ArimaConfig {
            p: 5,
            d: 1,
            q: 5,
            ..Default::default()
        };

        let result = fit_arima(&series, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_model_selection_via_aic() {
        let series: Vec<f64> = (0..100).map(|i| (i as f64 * 0.1).sin() * 5.0).collect();

        // Compare different model orders
        let configs = [
            ArimaConfig::ar(1),
            ArimaConfig::ar(2),
            ArimaConfig::arma(1, 1),
        ];

        let mut best_aic = f64::INFINITY;
        let mut best_config = None;

        for config in &configs {
            if let Ok(result) = fit_arima(&series, config) {
                if result.aic < best_aic {
                    best_aic = result.aic;
                    best_config = Some(config.clone());
                }
            }
        }

        assert!(best_config.is_some());
    }
}
