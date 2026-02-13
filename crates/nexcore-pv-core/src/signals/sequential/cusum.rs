//! CuSum - Cumulative Sum Control Charts
//!
//! Detects small persistent shifts in adverse event rates over time.
//!
//! # Algorithm
//!
//! The CUSUM tracks cumulative deviations from an expected baseline rate:
//!
//! ```text
//! S_upper(t) = max(0, S_upper(t-1) + x_t - μ_0 - k)
//! S_lower(t) = max(0, S_lower(t-1) - x_t + μ_0 - k)
//! ```
//!
//! A signal is detected when either sum exceeds the control limit h.
//!
//! # Use Cases
//!
//! - Post-market drug/vaccine surveillance
//! - Hospital-acquired infection monitoring
//! - Manufacturing quality control
//!
//! # References
//!
//! - Page ES (1954). "Continuous inspection schemes." Biometrika 41(1-2):100-115.
//!   DOI: [10.1093/biomet/41.1-2.100](https://doi.org/10.1093/biomet/41.1-2.100)
//!
//! - Grigg OA, Farewell VT, Spiegelhalter DJ (2003). "Use of risk-adjusted CUSUM and
//!   RSPRT charts for monitoring in medical contexts." Statistical Methods in Medical
//!   Research 12(2):147-170. DOI: [10.1177/096228020301200205](https://doi.org/10.1177/096228020301200205)
//!
//! - Rogerson PA, Yamada I (2004). "Monitoring change in spatial patterns of disease:
//!   comparing univariate and multivariate cumulative sum approaches." Statistics in
//!   Medicine 23(14):2195-2214. DOI: [10.1002/sim.1806](https://doi.org/10.1002/sim.1806)

use crate::signals::core::error::SignalError;
use serde::{Deserialize, Serialize};

/// CuSum direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CuSumDirection {
    /// Detect increases only
    Upper,
    /// Detect decreases only
    Lower,
    /// Detect both increases and decreases
    Both,
}

/// CuSum configuration
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CuSumConfig {
    /// Baseline rate (mean under control)
    pub baseline_rate: f64,
    /// Allowable slack (threshold k)
    pub threshold_k: f64,
    /// Control limit (decision threshold h)
    pub control_limit: f64,
    /// Direction to monitor
    pub direction: CuSumDirection,
}

impl Default for CuSumConfig {
    fn default() -> Self {
        Self {
            baseline_rate: 1.0,
            threshold_k: 0.5,
            control_limit: 5.0,
            direction: CuSumDirection::Upper,
        }
    }
}

impl CuSumConfig {
    #[must_use]
    pub fn new(baseline_rate: f64, threshold_k: f64, control_limit: f64) -> Self {
        Self {
            baseline_rate,
            threshold_k,
            control_limit,
            direction: CuSumDirection::Upper,
        }
    }

    #[must_use]
    pub fn with_direction(mut self, direction: CuSumDirection) -> Self {
        self.direction = direction;
        self
    }
}

/// CuSum result
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CuSumResult {
    pub cusum_upper: Vec<f64>,
    pub cusum_lower: Vec<f64>,
    pub signal_detected: bool,
    pub signal_index_upper: Option<usize>,
    pub signal_index_lower: Option<usize>,
    pub observations_count: usize,
}

/// Calculate CuSum for time series
///
/// # Complexity: O(n) where n = observations.len()
#[must_use]
pub fn calculate_cusum(
    observations: &[f64],
    config: &CuSumConfig,
) -> Result<CuSumResult, SignalError> {
    if observations.is_empty() {
        return Err(SignalError::InsufficientData(
            "Empty observation series".into(),
        ));
    }

    if config.baseline_rate <= 0.0 {
        return Err(SignalError::InvalidExpectedCount(config.baseline_rate));
    }

    let n = observations.len();
    let mut cusum_upper = Vec::with_capacity(n + 1);
    let mut cusum_lower = Vec::with_capacity(n + 1);

    cusum_upper.push(0.0);
    cusum_lower.push(0.0);

    let mut signal_index_upper = None;
    let mut signal_index_lower = None;

    for (i, &x) in observations.iter().enumerate() {
        // Standardize observation (Poisson approximation)
        let z = if config.baseline_rate > 0.0 {
            (x - config.baseline_rate) / config.baseline_rate.sqrt()
        } else {
            0.0
        };

        // Upper CuSum (detect increases)
        let s_upper = (cusum_upper[i] + z - config.threshold_k).max(0.0);
        cusum_upper.push(s_upper);

        if s_upper > config.control_limit && signal_index_upper.is_none() {
            signal_index_upper = Some(i);
        }

        // Lower CuSum (detect decreases)
        let s_lower = (cusum_lower[i] - z - config.threshold_k).max(0.0);
        cusum_lower.push(s_lower);

        if s_lower > config.control_limit && signal_index_lower.is_none() {
            signal_index_lower = Some(i);
        }
    }

    let signal_detected = match config.direction {
        CuSumDirection::Upper => signal_index_upper.is_some(),
        CuSumDirection::Lower => signal_index_lower.is_some(),
        CuSumDirection::Both => signal_index_upper.is_some() || signal_index_lower.is_some(),
    };

    Ok(CuSumResult {
        cusum_upper,
        cusum_lower,
        signal_detected,
        signal_index_upper,
        signal_index_lower,
        observations_count: n,
    })
}

/// Batch CuSum calculation (parallel)
#[must_use]
pub fn batch_cusum_parallel(
    time_series: &[Vec<f64>],
    config: &CuSumConfig,
) -> Vec<Result<CuSumResult, SignalError>> {
    use rayon::prelude::*;

    time_series
        .par_iter()
        .map(|series| calculate_cusum(series, config))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cusum_upper_signal() {
        let config = CuSumConfig::new(1.0, 0.5, 5.0);
        // Need more extreme deviations to accumulate CuSum > 5.0
        let observations = vec![3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

        let result = calculate_cusum(&observations, &config).unwrap();

        assert!(result.signal_detected);
        assert!(result.signal_index_upper.is_some());
    }

    #[test]
    fn test_cusum_no_signal() {
        let config = CuSumConfig::new(1.0, 0.5, 5.0);
        let observations = vec![0.9, 1.0, 1.1, 0.95, 1.05];

        let result = calculate_cusum(&observations, &config).unwrap();

        assert!(!result.signal_detected);
        assert!(result.signal_index_upper.is_none());
    }

    #[test]
    fn test_cusum_both_directions() {
        let config = CuSumConfig::new(1.0, 0.5, 5.0).with_direction(CuSumDirection::Both);
        let observations = vec![0.1, 0.2, 0.15, 0.05, 0.1];

        let result = calculate_cusum(&observations, &config).unwrap();

        assert_eq!(result.observations_count, 5);
    }

    #[test]
    fn test_batch_cusum() {
        let config = CuSumConfig::default();
        let series1 = vec![1.2, 1.5, 1.8];
        let series2 = vec![0.9, 1.0, 1.1];
        let time_series = vec![series1, series2];

        let results = batch_cusum_parallel(&time_series, &config);

        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
    }
}
