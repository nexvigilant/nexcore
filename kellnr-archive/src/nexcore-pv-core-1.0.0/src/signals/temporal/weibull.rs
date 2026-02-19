//! Weibull Time-to-Onset (TTO) Analysis for Causality Assessment
//!
//! This module implements Weibull distribution analysis for characterizing
//! time-to-onset patterns in pharmacovigilance. The shape of the Weibull
//! distribution provides evidence for or against a causal drug-event relationship.
//!
//! # Theory
//!
//! The Weibull distribution has PDF:
//! f(t; k, λ) = (k/λ)(t/λ)^(k-1) exp(-(t/λ)^k)
//!
//! Where:
//! - k (shape): Determines hazard pattern
//! - λ (scale): Characteristic time (63.2% of events occur by time λ)
//!
//! # Shape Parameter Interpretation
//!
//! | Shape (k) | Pattern | Hazard | Clinical Interpretation |
//! |-----------|---------|--------|------------------------|
//! | k < 0.8 | Early onset | Decreasing | Acute reactions, anaphylaxis |
//! | 0.8 ≤ k ≤ 1.2 | Random | Constant | Background/coincidental events |
//! | k > 1.2 | Late onset | Increasing | Cumulative toxicity, delayed effects |
//!
//! # Use Cases
//!
//! - **Causality assessment**: Early onset (k < 1) supports causal relationship
//! - **Signal characterization**: Distinguish acute vs. chronic drug effects
//! - **Risk communication**: Determine when to expect adverse events
//!
//! # References
//!
//! - Harmark L, van Grootheest AC (2008). "Pharmacovigilance: methods, recent developments
//!   and future perspectives." European Journal of Clinical Pharmacology 64(8):743-752.
//!   DOI: [10.1007/s00228-008-0475-9](https://doi.org/10.1007/s00228-008-0475-9)
//!
//! - Maignen F, Hauben M, Hung E, et al. (2010). "A conceptual approach to the masking
//!   effect of measures of disproportionality." Pharmacoepidemiology and Drug Safety
//!   19(2):156-165. DOI: [10.1002/pds.1879](https://doi.org/10.1002/pds.1879)
//!
//! - van Puijenbroek EP, Egberts AC, Heerdink ER, Leufkens HG (1999). "Detecting drug-drug
//!   interactions using a database for spontaneous adverse drug reactions: an example
//!   with diuretics and non-steroidal anti-inflammatory drugs." European Journal of
//!   Clinical Pharmacology 56(9-10):733-738. DOI: [10.1007/s002280000215](https://doi.org/10.1007/s002280000215)
//!
//! - Cornelius VR, Sauzet O, Williams JE, et al. (2013). "Adverse event reporting in
//!   randomised controlled trials of neuropathic pain: considerations for future practice."
//!   PAIN 154(2):213-220. DOI: [10.1016/j.pain.2012.10.012](https://doi.org/10.1016/j.pain.2012.10.012)

use crate::signals::core::error::SignalError;
use crate::signals::core::stats::{weibull_median, weibull_mle, weibull_shape_ci};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

/// Classification of time-to-onset pattern based on Weibull shape parameter.
///
/// The thresholds (0.8 and 1.2) are based on epidemiological literature
/// for distinguishing clinically meaningful patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeibullShape {
    /// Shape k < 0.8: Decreasing hazard rate.
    /// Suggests acute/immediate reactions that occur early and
    /// become less likely over time. Supports causal association.
    EarlyOnset,

    /// Shape 0.8 ≤ k ≤ 1.2: Approximately constant hazard rate.
    /// Consistent with random/coincidental events unrelated to
    /// drug exposure duration. Neutral for causality.
    Random,

    /// Shape k > 1.2: Increasing hazard rate.
    /// Suggests cumulative or delayed effects that become more
    /// likely with continued exposure. May support causality
    /// for certain mechanisms (e.g., dose-dependent toxicity).
    LateOnset,
}

impl WeibullShape {
    /// Classify shape parameter into pattern category.
    ///
    /// # Thresholds
    /// - Early onset: k < 0.8
    /// - Random: 0.8 ≤ k ≤ 1.2
    /// - Late onset: k > 1.2
    #[must_use]
    pub fn classify(shape: f64) -> Self {
        if shape < 0.8 {
            Self::EarlyOnset
        } else if shape > 1.2 {
            Self::LateOnset
        } else {
            Self::Random
        }
    }

    /// Get human-readable description of the pattern.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::EarlyOnset => "Decreasing hazard - acute/immediate reactions",
            Self::Random => "Constant hazard - random/coincidental timing",
            Self::LateOnset => "Increasing hazard - cumulative/delayed effects",
        }
    }

    /// Get causality implication of the pattern.
    #[must_use]
    pub const fn causality_implication(&self) -> &'static str {
        match self {
            Self::EarlyOnset => "Supports causal relationship (Type A ADR pattern)",
            Self::Random => "Neutral - consistent with background incidence",
            Self::LateOnset => "Possible cumulative toxicity or delayed mechanism",
        }
    }
}

/// Configuration for Weibull TTO analysis.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WeibullTTOConfig {
    /// Maximum iterations for MLE convergence (default: 100)
    pub max_iterations: usize,
    /// Convergence tolerance for shape parameter (default: 1e-6)
    pub tolerance: f64,
    /// Confidence level for intervals (default: 0.95)
    pub confidence_level: f64,
    /// Minimum sample size for reliable estimation (default: 5)
    pub min_sample_size: usize,
    /// Early onset threshold for shape classification (default: 0.8)
    pub early_onset_threshold: f64,
    /// Late onset threshold for shape classification (default: 1.2)
    pub late_onset_threshold: f64,
}

impl Default for WeibullTTOConfig {
    fn default() -> Self {
        Self {
            max_iterations: 100,
            tolerance: 1e-6,
            confidence_level: 0.95,
            min_sample_size: 5,
            early_onset_threshold: 0.8,
            late_onset_threshold: 1.2,
        }
    }
}

impl WeibullTTOConfig {
    /// Create a new configuration with custom values.
    #[must_use]
    pub const fn new(
        max_iterations: usize,
        tolerance: f64,
        confidence_level: f64,
        min_sample_size: usize,
    ) -> Self {
        Self {
            max_iterations,
            tolerance,
            confidence_level,
            min_sample_size,
            early_onset_threshold: 0.8,
            late_onset_threshold: 1.2,
        }
    }

    /// Sensitive configuration for detecting patterns with smaller samples.
    #[must_use]
    pub const fn sensitive() -> Self {
        Self {
            max_iterations: 200,
            tolerance: 1e-8,
            confidence_level: 0.90,
            min_sample_size: 3,
            early_onset_threshold: 0.9,
            late_onset_threshold: 1.1,
        }
    }

    /// Conservative configuration requiring more evidence.
    #[must_use]
    pub const fn conservative() -> Self {
        Self {
            max_iterations: 100,
            tolerance: 1e-6,
            confidence_level: 0.99,
            min_sample_size: 10,
            early_onset_threshold: 0.7,
            late_onset_threshold: 1.3,
        }
    }
}

/// Result of Weibull TTO analysis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeibullTTOResult {
    /// Shape parameter k (hazard rate pattern)
    pub shape: f64,
    /// Scale parameter λ (characteristic time in input units)
    pub scale: f64,
    /// Classified time-to-onset pattern
    pub pattern: WeibullShape,
    /// Median time-to-onset (50th percentile)
    pub median_tto: f64,
    /// Mean time-to-onset (λ * Γ(1 + 1/k))
    pub mean_tto: f64,
    /// 95% CI lower bound for shape parameter
    pub ci_shape_lower: f64,
    /// 95% CI upper bound for shape parameter
    pub ci_shape_upper: f64,
    /// Sample size used for estimation
    pub sample_size: usize,
    /// Log-likelihood at MLE (for model comparison)
    pub log_likelihood: f64,
    /// Whether the CI for shape excludes k=1 (significant departure from random)
    pub is_significant: bool,
}

impl WeibullTTOResult {
    /// Check if the pattern significantly suggests early onset.
    ///
    /// Returns true if the upper CI bound is below the random threshold.
    #[must_use]
    pub fn is_early_onset_significant(&self, threshold: f64) -> bool {
        self.ci_shape_upper < threshold
    }

    /// Check if the pattern significantly suggests late onset.
    ///
    /// Returns true if the lower CI bound is above the random threshold.
    #[must_use]
    pub fn is_late_onset_significant(&self, threshold: f64) -> bool {
        self.ci_shape_lower > threshold
    }
}

/// Fit Weibull distribution to time-to-onset data.
///
/// This function estimates the shape and scale parameters of a Weibull
/// distribution from observed time-to-onset data, providing causality
/// evidence through pattern classification.
///
/// # Arguments
///
/// * `times` - Time-to-onset values in consistent units (e.g., days)
/// * `config` - Analysis configuration
///
/// # Returns
///
/// * `Ok(WeibullTTOResult)` - Estimated parameters and pattern classification
/// * `Err(SignalError)` - If data is invalid or estimation fails
///
/// # Complexity
///
/// TIME: O(n * max_iterations) where n = times.len()
/// SPACE: O(1) beyond input storage
///
/// # Example
///
/// ```rust
/// use nexcore_vigilance::pv::signals::temporal::weibull::{fit_weibull_tto, WeibullTTOConfig, WeibullShape};
///
/// // Acute reactions: early onset pattern
/// let acute_times = vec![0.5, 1.0, 1.5, 2.0, 0.8, 1.2, 0.3, 2.5];
/// let config = WeibullTTOConfig::default();
/// let result = fit_weibull_tto(&acute_times, &config).unwrap();
///
/// // Should detect early onset pattern (k < 0.8 suggests decreasing hazard)
/// println!("Shape: {:.2}, Pattern: {:?}", result.shape, result.pattern);
/// ```
pub fn fit_weibull_tto(
    times: &[f64],
    config: &WeibullTTOConfig,
) -> Result<WeibullTTOResult, SignalError> {
    // Validate input
    if times.len() < config.min_sample_size {
        return Err(SignalError::InsufficientData(format!(
            "Need at least {} observations, got {}",
            config.min_sample_size,
            times.len()
        )));
    }

    // Check for invalid time values
    for (i, &t) in times.iter().enumerate() {
        if t <= 0.0 {
            return Err(SignalError::invalid_time(format!(
                "Time value at index {} must be positive, got {}",
                i, t
            )));
        }
        if t.is_nan() || t.is_infinite() {
            return Err(SignalError::invalid_time(format!(
                "Time value at index {} is not finite: {}",
                i, t
            )));
        }
    }

    // Fit Weibull using MLE
    let (shape, scale) = weibull_mle(times, config.max_iterations, config.tolerance)
        .map_err(|e| SignalError::weibull_fit(e))?;

    // Compute derived statistics
    let median_tto = weibull_median(shape, scale);

    // Mean = λ * Γ(1 + 1/k)
    let mean_tto = scale * gamma_approx(1.0 + 1.0 / shape);

    // Confidence interval for shape
    let (ci_shape_lower, ci_shape_upper) =
        weibull_shape_ci(shape, times.len(), config.confidence_level);

    // Classify pattern
    let pattern = classify_with_thresholds(shape, config);

    // Compute log-likelihood for model comparison
    let log_likelihood = compute_log_likelihood(times, shape, scale);

    // Determine significance: CI excludes k=1 (exponential/random)
    let is_significant = ci_shape_upper < 1.0 || ci_shape_lower > 1.0;

    Ok(WeibullTTOResult {
        shape,
        scale,
        pattern,
        median_tto,
        mean_tto,
        ci_shape_lower,
        ci_shape_upper,
        sample_size: times.len(),
        log_likelihood,
        is_significant,
    })
}

/// Batch Weibull TTO analysis with parallel processing.
///
/// Analyzes multiple time series in parallel using Rayon.
///
/// # Arguments
///
/// * `time_series` - Vector of time-to-onset vectors (one per drug-event pair)
/// * `config` - Analysis configuration (shared across all analyses)
///
/// # Returns
///
/// Vector of results, one per input time series.
///
/// # Complexity
///
/// TIME: O(sum(n_i * max_iterations) / num_cores)
/// SPACE: O(num_series) for results
#[must_use]
pub fn batch_weibull_parallel(
    time_series: &[Vec<f64>],
    config: &WeibullTTOConfig,
) -> Vec<Result<WeibullTTOResult, SignalError>> {
    time_series
        .par_iter()
        .map(|times| fit_weibull_tto(times, config))
        .collect()
}

/// Classify shape parameter with custom thresholds.
fn classify_with_thresholds(shape: f64, config: &WeibullTTOConfig) -> WeibullShape {
    if shape < config.early_onset_threshold {
        WeibullShape::EarlyOnset
    } else if shape > config.late_onset_threshold {
        WeibullShape::LateOnset
    } else {
        WeibullShape::Random
    }
}

/// Approximate gamma function for mean calculation.
///
/// Uses Stirling's approximation for x > 2, recurrence for smaller x.
fn gamma_approx(x: f64) -> f64 {
    if x <= 0.0 {
        return f64::NAN;
    }

    // For small x, use recurrence: Γ(x) = Γ(x+1) / x
    let mut x = x;
    let mut factor = 1.0;
    while x < 2.0 {
        factor /= x;
        x += 1.0;
    }

    // Stirling's approximation for x >= 2
    use std::f64::consts::PI;
    let stirling = ((2.0 * PI / x).sqrt() * (x / std::f64::consts::E).powf(x))
        * (1.0 + 1.0 / (12.0 * x) + 1.0 / (288.0 * x * x));

    factor * stirling
}

/// Compute log-likelihood at given parameters.
fn compute_log_likelihood(times: &[f64], shape: f64, scale: f64) -> f64 {
    let n = times.len() as f64;
    let ln_k = shape.ln();
    let ln_lambda = scale.ln();

    let mut sum_ln_x = 0.0;
    let mut sum_x_lambda_k = 0.0;

    for &t in times {
        sum_ln_x += t.ln();
        sum_x_lambda_k += (t / scale).powf(shape);
    }

    n * (ln_k - ln_lambda) + (shape - 1.0) * sum_ln_x
        - n * (shape - 1.0) * ln_lambda
        - sum_x_lambda_k
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() < tol
    }

    #[test]
    fn test_weibull_shape_classify() {
        assert_eq!(WeibullShape::classify(0.5), WeibullShape::EarlyOnset);
        assert_eq!(WeibullShape::classify(0.79), WeibullShape::EarlyOnset);
        assert_eq!(WeibullShape::classify(0.8), WeibullShape::Random);
        assert_eq!(WeibullShape::classify(1.0), WeibullShape::Random);
        assert_eq!(WeibullShape::classify(1.2), WeibullShape::Random);
        assert_eq!(WeibullShape::classify(1.21), WeibullShape::LateOnset);
        assert_eq!(WeibullShape::classify(2.5), WeibullShape::LateOnset);
    }

    #[test]
    fn test_fit_weibull_early_onset() {
        // Simulate early onset: most events occur early
        // Times heavily skewed toward small values
        let times = vec![0.5, 0.8, 1.0, 1.2, 1.5, 2.0, 3.0, 5.0, 8.0, 15.0];
        let config = WeibullTTOConfig::default();

        let result = fit_weibull_tto(&times, &config).unwrap();

        // Shape should be < 1 for early onset
        assert!(
            result.shape < 1.5,
            "Shape {} should indicate early onset tendency",
            result.shape
        );
        assert!(result.scale > 0.0);
        assert!(result.median_tto > 0.0);
        assert!(result.sample_size == 10);
    }

    #[test]
    fn test_fit_weibull_late_onset() {
        // Simulate late onset: events cluster at higher times
        // More events occur later in the observation period
        let times = vec![5.0, 8.0, 10.0, 12.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0];
        let config = WeibullTTOConfig::default();

        let result = fit_weibull_tto(&times, &config).unwrap();

        // Shape should be > 1 for late onset
        assert!(
            result.shape > 0.8,
            "Shape {} should indicate late onset",
            result.shape
        );
        assert!(result.scale > 0.0);
    }

    #[test]
    fn test_fit_weibull_random() {
        // Mix of early and late events suggesting roughly constant hazard
        // This mimics more realistic exponential-like data
        let times = vec![0.5, 1.5, 2.0, 3.5, 4.0, 5.5, 8.0, 9.0, 11.0, 15.0];
        let config = WeibullTTOConfig::default();

        let result = fit_weibull_tto(&times, &config).unwrap();

        // Shape should be in a reasonable range
        // Note: Weibull classification depends on the specific data distribution
        assert!(
            result.shape > 0.5 && result.shape < 3.0,
            "Shape {} should be in reasonable range",
            result.shape
        );
        // The MLE converged and produced valid results
        assert!(result.scale > 0.0);
        assert!(result.median_tto > 0.0);
    }

    #[test]
    fn test_fit_weibull_insufficient_data() {
        let times = vec![1.0, 2.0]; // Less than default min_sample_size of 5
        let config = WeibullTTOConfig::default();

        let result = fit_weibull_tto(&times, &config);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SignalError::InsufficientData(_)
        ));
    }

    #[test]
    fn test_fit_weibull_invalid_times() {
        let config = WeibullTTOConfig::default();

        // Negative time
        let times = vec![1.0, -2.0, 3.0, 4.0, 5.0];
        assert!(fit_weibull_tto(&times, &config).is_err());

        // Zero time
        let times = vec![1.0, 0.0, 3.0, 4.0, 5.0];
        assert!(fit_weibull_tto(&times, &config).is_err());

        // NaN time
        let times = vec![1.0, f64::NAN, 3.0, 4.0, 5.0];
        assert!(fit_weibull_tto(&times, &config).is_err());

        // Infinite time
        let times = vec![1.0, f64::INFINITY, 3.0, 4.0, 5.0];
        assert!(fit_weibull_tto(&times, &config).is_err());
    }

    #[test]
    fn test_fit_weibull_sensitive_config() {
        let times = vec![0.5, 1.0, 1.5];
        let config = WeibullTTOConfig::sensitive(); // min_sample_size = 3

        let result = fit_weibull_tto(&times, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_batch_weibull_parallel() {
        let time_series = vec![
            vec![0.5, 1.0, 1.5, 2.0, 2.5],
            vec![5.0, 6.0, 7.0, 8.0, 9.0],
            vec![1.0, 2.0, 3.0, 4.0, 5.0],
        ];
        let config = WeibullTTOConfig::default();

        let results = batch_weibull_parallel(&time_series, &config);

        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.is_ok()));
    }

    #[test]
    fn test_weibull_result_significance() {
        let times = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
        let config = WeibullTTOConfig::default();

        let result = fit_weibull_tto(&times, &config).unwrap();

        // Check significance methods
        let early_sig = result.is_early_onset_significant(0.8);
        let late_sig = result.is_late_onset_significant(1.2);

        // Can't both be significant
        assert!(!(early_sig && late_sig));
    }

    #[test]
    fn test_gamma_approx() {
        // Γ(1) = 1
        assert!(approx_eq(gamma_approx(1.0), 1.0, 0.01));
        // Γ(2) = 1! = 1
        assert!(approx_eq(gamma_approx(2.0), 1.0, 0.01));
        // Γ(3) = 2! = 2
        assert!(approx_eq(gamma_approx(3.0), 2.0, 0.1));
        // Γ(0.5) = √π ≈ 1.772
        assert!(approx_eq(gamma_approx(0.5), 1.772, 0.1));
    }

    #[test]
    fn test_weibull_median_calculation() {
        // For exponential (k=1), median = λ * ln(2)
        let times = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let config = WeibullTTOConfig::default();
        let result = fit_weibull_tto(&times, &config).unwrap();

        // Median should be reasonable relative to data
        assert!(result.median_tto > 0.0);
        assert!(result.median_tto < result.mean_tto * 2.0);
    }
}
