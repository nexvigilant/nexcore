//! MaxSPRT - Maximized Sequential Probability Ratio Test
//!
//! FDA Sentinel Initiative method for post-market drug surveillance.
//! Unlike SPRT, MaxSPRT does not require pre-specifying the alternative hypothesis.
//!
//! # Algorithm
//!
//! MaxSPRT maximizes the likelihood ratio over all possible alternative hypotheses,
//! making it ideal for vaccine/drug safety surveillance where the magnitude of
//! increased risk is unknown a priori.
//!
//! # References
//!
//! - Kulldorff M, Davis RL, Kolczak M, et al. (2011). "A maximized sequential probability
//!   ratio test for drug and vaccine safety surveillance." Sequential Analysis 30(1):58-78.
//!   DOI: [10.1080/07474946.2011.539924](https://doi.org/10.1080/07474946.2011.539924)
//!
//! - Silva IR, Kulldorff M (2015). "Continuous versus group sequential analysis for
//!   post-market drug and vaccine safety surveillance." Biometrics 71(3):851-858.
//!   DOI: [10.1111/biom.12324](https://doi.org/10.1111/biom.12324)
//!
//! - Li L, Kulldorff M (2010). "A conditional maximized sequential probability ratio test
//!   for pharmacovigilance." Statistics in Medicine 29(2):284-295.
//!   DOI: [10.1002/sim.3780](https://doi.org/10.1002/sim.3780)

use crate::signals::core::error::SignalError;
use serde::{Deserialize, Serialize};

/// MaxSPRT decision outcome
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaxSprtDecision {
    /// Signal detected
    Signal,
    /// No signal - monitoring complete
    NoSignal,
    /// Continue monitoring
    Continue,
}

/// MaxSPRT configuration
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MaxSprtConfig {
    /// Overall significance level (alpha)
    pub alpha: f64,
    /// Maximum expected count for stopping rule
    pub max_expected: f64,
}

impl Default for MaxSprtConfig {
    fn default() -> Self {
        Self {
            alpha: 0.05,
            max_expected: 100.0,
        }
    }
}

impl MaxSprtConfig {
    #[must_use]
    pub fn new(alpha: f64, max_expected: f64) -> Self {
        Self {
            alpha,
            max_expected,
        }
    }

    #[must_use]
    pub fn sensitive() -> Self {
        Self {
            alpha: 0.10,
            max_expected: 50.0,
        }
    }

    #[must_use]
    pub fn conservative() -> Self {
        Self {
            alpha: 0.01,
            max_expected: 200.0,
        }
    }
}

/// MaxSPRT result
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaxSprtResult {
    pub llr: f64,
    pub test_statistic: f64,
    pub critical_value: f64,
    pub decision: MaxSprtDecision,
    pub observed: u32,
    pub expected: f64,
    pub obs_exp_ratio: f64,
    pub monitoring_progress: f64,
}

/// Calculate MaxSPRT for single observation
///
/// # Complexity: O(1)
#[must_use]
pub fn calculate_maxsprt(
    observed: u32,
    expected: f64,
    config: &MaxSprtConfig,
) -> Result<MaxSprtResult, SignalError> {
    if expected <= 0.0 {
        return Err(SignalError::InvalidExpectedCount(expected));
    }

    let n = f64::from(observed);
    let e = expected;

    // Maximum log-likelihood ratio (Poisson)
    let llr = if n > e {
        n * (n / e).ln() - (n - e)
    } else {
        0.0
    };

    // Test statistic (likelihood ratio test)
    let test_statistic = if n > 0.0 { 2.0 * llr } else { 0.0 };

    // Critical value using Poisson MaxSPRT approximation
    let critical_value = critical_value_approximation(config.alpha, config.max_expected);

    // Decision
    let decision = if test_statistic >= critical_value {
        MaxSprtDecision::Signal
    } else if e >= config.max_expected {
        MaxSprtDecision::NoSignal
    } else {
        MaxSprtDecision::Continue
    };

    // Metrics
    let obs_exp_ratio = if e > 0.0 { n / e } else { 0.0 };
    let monitoring_progress = (e / config.max_expected).min(1.0);

    Ok(MaxSprtResult {
        llr,
        test_statistic,
        critical_value,
        decision,
        observed,
        expected,
        obs_exp_ratio,
        monitoring_progress,
    })
}

/// Batch MaxSPRT calculation (parallel)
#[must_use]
pub fn batch_maxsprt_parallel(
    observed: &[u32],
    expected: &[f64],
    config: &MaxSprtConfig,
) -> Vec<Result<MaxSprtResult, SignalError>> {
    use rayon::prelude::*;

    observed
        .par_iter()
        .zip(expected.par_iter())
        .map(|(&obs, &exp)| calculate_maxsprt(obs, exp, config))
        .collect()
}

/// Critical value approximation for MaxSPRT
///
/// Uses asymptotic approximation for Poisson MaxSPRT.
/// For more precision, use simulation-based critical values.
fn critical_value_approximation(alpha: f64, max_expected: f64) -> f64 {
    use std::f64::consts::PI;

    // Asymptotic approximation: cv ≈ -log(alpha) + 0.5 * log(2π * max_expected)
    if max_expected > 10.0 {
        -alpha.ln() + 0.5 * (2.0 * PI * max_expected).ln()
    } else {
        // Small sample lookup table (approximate)
        match alpha {
            a if a <= 0.001 => 15.0,
            a if a <= 0.01 => 10.0,
            a if a <= 0.05 => 6.0,
            _ => 4.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maxsprt_signal() {
        let config = MaxSprtConfig::default();
        let result = calculate_maxsprt(20, 5.0, &config).unwrap();

        assert_eq!(result.decision, MaxSprtDecision::Signal);
        assert!(result.test_statistic > result.critical_value);
        assert_eq!(result.observed, 20);
    }

    #[test]
    fn test_maxsprt_continue() {
        let config = MaxSprtConfig::default();
        let result = calculate_maxsprt(5, 4.0, &config).unwrap();

        assert_eq!(result.decision, MaxSprtDecision::Continue);
        assert!(result.test_statistic < result.critical_value);
    }

    #[test]
    fn test_maxsprt_no_signal_max_reached() {
        let config = MaxSprtConfig::new(0.05, 10.0);
        let result = calculate_maxsprt(8, 10.0, &config).unwrap();

        assert_eq!(result.decision, MaxSprtDecision::NoSignal);
        assert_eq!(result.monitoring_progress, 1.0);
    }

    #[test]
    fn test_batch_maxsprt() {
        let config = MaxSprtConfig::default();
        let observed = vec![20, 5, 8];
        let expected = vec![5.0, 4.0, 10.0];

        let results = batch_maxsprt_parallel(&observed, &expected, &config);

        assert_eq!(results.len(), 3);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
        assert!(results[2].is_ok());
    }
}
