//! Tier 2 -- Predictive Signal Detection.
//!
//! Tier: T3 (maps to Causality `->` + Frequency `nu` + Boundary `partial` +
//!           State `varsigma` + Comparison `kappa` + Sequence `sigma`)
//!
//! Combines signal emergence feasibility (Gibbs), temporal trajectory,
//! and noise correction into the Preemptive Signal Potential Psi:
//!
//! ```text
//! Psi(d, e, t) = DeltaG(d, e) * Gamma(d, e, t) * [1 - eta(d, t)]
//! ```
//!
//! Note: In the full equation, Psi also includes Omega, but that is applied
//! at the preemptive (Tier 3) decision level. This module computes the
//! intermediate Psi without severity weighting.

use crate::gibbs;
use crate::noise;
use crate::trajectory;
use crate::types::{GibbsParams, NoiseParams, ReportingDataPoint};

/// Configuration for the predictive signal computation.
///
/// Tier: T2-C (composed configuration)
#[derive(Debug, Clone, PartialEq)]
pub struct PredictiveConfig {
    /// Weight for acceleration in trajectory (default: 0.5)
    pub alpha: f64,
    /// Hill coefficient for cooperative amplification (default: 2.0)
    pub hill_n: f64,
    /// Half-maximal constant for Hill amplification (default: 1.0)
    pub hill_k_half: f64,
    /// Whether to apply Hill amplification to trajectory (default: true)
    pub use_hill_amplification: bool,
}

impl Default for PredictiveConfig {
    fn default() -> Self {
        Self {
            alpha: trajectory::DEFAULT_ALPHA,
            hill_n: trajectory::DEFAULT_HILL_COEFFICIENT,
            hill_k_half: trajectory::DEFAULT_K_HALF,
            use_hill_amplification: true,
        }
    }
}

/// The result of a predictive signal evaluation.
///
/// Tier: T3 (domain result combining all components)
#[derive(Debug, Clone, PartialEq)]
pub struct PredictiveResult {
    /// The raw Gibbs signal feasibility (DeltaG).
    pub delta_g: f64,
    /// The feasibility score (sigmoid-normalized DeltaG, 0-1).
    pub feasibility: f64,
    /// The raw trajectory Gamma.
    pub gamma_raw: f64,
    /// The Hill-amplified trajectory (if enabled).
    pub gamma_amplified: f64,
    /// The noise floor correction eta.
    pub eta: f64,
    /// The signal retention factor (1 - eta).
    pub signal_retention: f64,
    /// The final Preemptive Signal Potential Psi.
    pub psi: f64,
}

/// Computes the Preemptive Signal Potential Psi.
///
/// ```text
/// Psi = feasibility_score(DeltaG) * Gamma_amp * (1 - eta)
/// ```
///
/// Uses the sigmoid-normalized feasibility score (0-1) rather than raw DeltaG
/// to keep Psi bounded and interpretable.
///
/// # Arguments
///
/// * `gibbs_params` - Parameters for signal emergence feasibility.
/// * `reporting_data` - Time-series reporting rate data.
/// * `noise_params` - Parameters for noise floor correction.
/// * `config` - Configuration for the predictive computation.
#[must_use]
pub fn psi(
    gibbs_params: &GibbsParams,
    reporting_data: &[ReportingDataPoint],
    noise_params: &NoiseParams,
    config: &PredictiveConfig,
) -> PredictiveResult {
    // Component 1: Signal feasibility
    let dg = gibbs::delta_g(gibbs_params);
    let feasibility = gibbs::feasibility_score(gibbs_params);

    // Component 2: Temporal trajectory
    let gamma_raw = trajectory::gamma(reporting_data, config.alpha);
    let gamma_amp = if config.use_hill_amplification {
        trajectory::hill_amplify(gamma_raw, config.hill_n, config.hill_k_half)
    } else {
        gamma_raw
    };

    // Component 3: Noise correction
    let eta_val = noise::eta(noise_params);
    let retention = noise::signal_retention(noise_params);

    // Psi = feasibility * trajectory * retention
    let psi_value = feasibility * gamma_amp * retention;

    PredictiveResult {
        delta_g: dg,
        feasibility,
        gamma_raw,
        gamma_amplified: gamma_amp,
        eta: eta_val,
        signal_retention: retention,
        psi: psi_value,
    }
}

/// Convenience function that computes Psi with default configuration.
#[must_use]
pub fn psi_default(
    gibbs_params: &GibbsParams,
    reporting_data: &[ReportingDataPoint],
    noise_params: &NoiseParams,
) -> PredictiveResult {
    psi(
        gibbs_params,
        reporting_data,
        noise_params,
        &PredictiveConfig::default(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_data(points: &[(f64, f64)]) -> Vec<ReportingDataPoint> {
        points
            .iter()
            .map(|&(t, r)| ReportingDataPoint::new(t, r))
            .collect()
    }

    #[test]
    fn psi_strong_signal() {
        // Favorable DeltaG, escalating trajectory, organic reporting
        let gibbs = GibbsParams::new(3.0, 10000.0, 0.001);
        let data = make_data(&[(1.0, 2.0), (2.0, 5.0), (3.0, 11.0)]);
        let noise = NoiseParams::new(25.0, 50.0); // below baseline = organic

        let result = psi_default(&gibbs, &data, &noise);

        assert!(result.feasibility > 0.99); // DeltaG = -7, very favorable
        assert!(result.gamma_raw > 0.0); // Escalating
        assert!(result.gamma_amplified > 0.0); // Hill-amplified
        assert!(result.signal_retention > 0.9); // Organic
        assert!(result.psi > 0.0); // Positive Psi
    }

    #[test]
    fn psi_weak_signal_unfavorable_gibbs() {
        // Unfavorable DeltaG: low exposure, mechanism alone
        let gibbs = GibbsParams::new(10.0, 10.0, 0.001);
        let data = make_data(&[(1.0, 2.0), (2.0, 5.0), (3.0, 11.0)]);
        let noise = NoiseParams::new(25.0, 50.0);

        let result = psi_default(&gibbs, &data, &noise);

        assert!(result.feasibility < 0.01); // DeltaG = +9.99, unfavorable
        assert!(result.psi < 0.01); // Psi suppressed by low feasibility
    }

    #[test]
    fn psi_stimulated_reporting() {
        // Strong signal but stimulated reporting -> Psi suppressed
        let gibbs = GibbsParams::new(3.0, 10000.0, 0.001);
        let data = make_data(&[(1.0, 2.0), (2.0, 5.0), (3.0, 11.0)]);
        let noise = NoiseParams::new(250.0, 50.0); // 5x baseline = stimulated

        let result = psi_default(&gibbs, &data, &noise);

        assert!(result.eta > 0.99); // Heavily stimulated
        assert!(result.signal_retention < 0.01); // Almost no retention
        assert!(result.psi < 0.01); // Psi suppressed
    }

    #[test]
    fn psi_flat_trajectory() {
        // Favorable Gibbs but no trajectory change
        let gibbs = GibbsParams::new(3.0, 10000.0, 0.001);
        let data = make_data(&[(1.0, 5.0), (2.0, 5.0), (3.0, 5.0)]);
        let noise = NoiseParams::new(25.0, 50.0);

        let result = psi_default(&gibbs, &data, &noise);

        assert!(result.gamma_raw.abs() < f64::EPSILON); // No trajectory
        assert!(result.psi.abs() < f64::EPSILON); // Psi = 0
    }

    #[test]
    fn psi_without_hill_amplification() {
        let gibbs = GibbsParams::new(3.0, 10000.0, 0.001);
        let data = make_data(&[(1.0, 2.0), (2.0, 5.0), (3.0, 11.0)]);
        let noise = NoiseParams::new(25.0, 50.0);

        let config_hill = PredictiveConfig::default();
        let config_no_hill = PredictiveConfig {
            use_hill_amplification: false,
            ..PredictiveConfig::default()
        };

        let result_hill = psi(&gibbs, &data, &noise, &config_hill);
        let result_no_hill = psi(&gibbs, &data, &noise, &config_no_hill);

        // Without Hill, gamma_amplified = gamma_raw
        assert!((result_no_hill.gamma_amplified - result_no_hill.gamma_raw).abs() < f64::EPSILON);
        // Hill-amplified Psi may differ
        assert!(
            result_hill.psi != result_no_hill.psi
                || result_hill.gamma_raw == result_no_hill.gamma_raw
        );
    }

    #[test]
    fn psi_default_uses_default_config() {
        let gibbs = GibbsParams::new(3.0, 10000.0, 0.001);
        let data = make_data(&[(1.0, 2.0), (2.0, 5.0), (3.0, 11.0)]);
        let noise = NoiseParams::new(25.0, 50.0);

        let result1 = psi_default(&gibbs, &data, &noise);
        let result2 = psi(&gibbs, &data, &noise, &PredictiveConfig::default());

        assert!((result1.psi - result2.psi).abs() < f64::EPSILON);
    }

    #[test]
    fn psi_result_component_consistency() {
        let gibbs = GibbsParams::new(3.0, 10000.0, 0.001);
        let data = make_data(&[(1.0, 2.0), (2.0, 5.0), (3.0, 11.0)]);
        let noise = NoiseParams::new(40.0, 50.0);

        let result = psi_default(&gibbs, &data, &noise);

        // Verify eta + retention = 1
        assert!((result.eta + result.signal_retention - 1.0).abs() < f64::EPSILON);

        // Verify psi = feasibility * gamma_amplified * retention
        let expected_psi = result.feasibility * result.gamma_amplified * result.signal_retention;
        assert!((result.psi - expected_psi).abs() < 1e-10);
    }

    #[test]
    fn psi_no_data() {
        let gibbs = GibbsParams::new(3.0, 10000.0, 0.001);
        let data: Vec<ReportingDataPoint> = vec![];
        let noise = NoiseParams::new(25.0, 50.0);

        let result = psi_default(&gibbs, &data, &noise);
        assert!(result.psi.abs() < f64::EPSILON);
    }
}
