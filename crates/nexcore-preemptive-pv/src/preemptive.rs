//! Tier 3 -- Preemptive Decision Engine.
//!
//! Tier: T3 (maps to Causality `->` + Frequency `nu` + Boundary `partial` +
//!           State `varsigma` + Comparison `kappa` + Irreversibility `proportional` + Sequence `sigma`)
//!
//! The final tier that combines the Preemptive Signal Potential Psi
//! with irreversibility-weighted severity Omega to produce an intervention decision:
//!
//! ```text
//! Pi(d, e, t) = Psi(d, e, t) * Omega(irreversibility) - C(I)
//!
//! INTERVENE when Pi > 0
//! ```
//!
//! Where:
//! - `Psi` = preemptive signal potential (from Tier 2)
//! - `Omega` = irreversibility-weighted severity
//! - `C(I)` = cost of intervention (opportunity cost, implementation burden)
//!
//! The preemptive threshold is lowered for severe outcomes:
//! ```text
//! theta_preemptive = theta_detection * lambda_safety
//! ```

use crate::intervention;
use crate::predictive::{self, PredictiveConfig, PredictiveResult};
use crate::severity;
use crate::types::{
    Decision, GibbsParams, InterventionResult, NoiseParams, ReportingDataPoint, SafetyLambda,
    Seriousness,
};

/// Default cost of intervention C(I).
///
/// Represents the opportunity cost and implementation burden of acting.
/// Higher values require stronger evidence before recommending intervention.
pub const DEFAULT_INTERVENTION_COST: f64 = 0.1;

/// Default detection threshold before safety lambda adjustment.
pub const DEFAULT_DETECTION_THRESHOLD: f64 = 0.5;

/// Configuration for preemptive decision-making.
///
/// Tier: T3 (full domain configuration)
#[derive(Debug, Clone, PartialEq)]
pub struct PreemptiveConfig {
    /// Predictive signal configuration.
    pub predictive: PredictiveConfig,
    /// Cost of intervention C(I).
    pub intervention_cost: f64,
    /// Base detection threshold (before safety lambda adjustment).
    pub detection_threshold: f64,
    /// Parameters for competitive inhibition intervention model.
    pub intervention_v_max: f64,
    /// Substrate concentration for intervention model.
    pub intervention_substrate: f64,
    /// Michaelis constant for intervention model.
    pub intervention_k_m: f64,
    /// Inhibition constant for intervention model.
    pub intervention_k_i: f64,
}

impl Default for PreemptiveConfig {
    fn default() -> Self {
        Self {
            predictive: PredictiveConfig::default(),
            intervention_cost: DEFAULT_INTERVENTION_COST,
            detection_threshold: DEFAULT_DETECTION_THRESHOLD,
            intervention_v_max: 100.0,
            intervention_substrate: 10.0,
            intervention_k_m: intervention::DEFAULT_K_M,
            intervention_k_i: intervention::DEFAULT_K_I,
        }
    }
}

/// Complete result of a preemptive evaluation.
///
/// Tier: T3 (full domain result)
#[derive(Debug, Clone, PartialEq)]
pub struct PreemptiveResult {
    /// The predictive signal evaluation result.
    pub predictive: PredictiveResult,
    /// The irreversibility-weighted severity Omega.
    pub omega: f64,
    /// The safety lambda applied.
    pub safety_lambda: f64,
    /// The preemptive threshold (theta_detection * lambda_safety).
    pub preemptive_threshold: f64,
    /// The intervention index Pi = Psi * Omega - C(I).
    pub pi: f64,
    /// The final three-tier decision.
    pub decision: Decision,
    /// The intervention modeling result (only populated for Tier 3 decisions).
    pub intervention: Option<InterventionResult>,
}

/// Performs the full three-tier preemptive evaluation.
///
/// Evaluates in order:
/// 1. **Tier 1 (Reactive):** Is Psi above zero? If not, Monitor.
/// 2. **Tier 2 (Predictive):** Is Psi above the preemptive threshold? If not, Predict.
/// 3. **Tier 3 (Preemptive):** Is Pi > 0? If yes, Intervene.
///
/// # Arguments
///
/// * `gibbs_params` - Signal emergence feasibility parameters.
/// * `reporting_data` - Time-series reporting rate data.
/// * `noise_params` - Noise floor correction parameters.
/// * `seriousness` - ICH E2A seriousness category.
/// * `config` - Preemptive decision configuration.
#[must_use]
pub fn evaluate(
    gibbs_params: &GibbsParams,
    reporting_data: &[ReportingDataPoint],
    noise_params: &NoiseParams,
    seriousness: Seriousness,
    config: &PreemptiveConfig,
) -> PreemptiveResult {
    // Compute predictive signal
    let pred_result = predictive::psi(
        gibbs_params,
        reporting_data,
        noise_params,
        &config.predictive,
    );

    // Compute severity-weighted components
    let omega_val = severity::omega(seriousness);
    let lambda = SafetyLambda::from_seriousness(seriousness);
    let preemptive_threshold = lambda.apply(config.detection_threshold);

    // Compute intervention index: Pi = Psi * Omega - C(I)
    let pi = pred_result.psi * omega_val - config.intervention_cost;

    // Three-tier decision
    let (decision, intervention_result) = if pred_result.psi <= 0.0 {
        // Tier 1: No signal trajectory -> Monitor
        (
            Decision::Monitor {
                signal_strength: pred_result.psi,
            },
            None,
        )
    } else if pred_result.psi < preemptive_threshold {
        // Tier 2: Signal detected but below preemptive threshold -> Predict
        (
            Decision::Predict {
                psi: pred_result.psi,
                trajectory: pred_result.gamma_raw,
            },
            None,
        )
    } else if pi > 0.0 {
        // Tier 3: Intervention warranted
        let recommended_strength = estimate_intervention_strength(pi, omega_val, config);
        let intervention = intervention::intervention_effect(
            config.intervention_v_max,
            config.intervention_substrate,
            recommended_strength,
            config.intervention_k_m,
            config.intervention_k_i,
        );
        (
            Decision::Intervene {
                pi,
                omega: omega_val,
                recommended_intervention_strength: recommended_strength,
            },
            Some(intervention),
        )
    } else {
        // Signal above threshold but cost exceeds benefit -> Predict
        (
            Decision::Predict {
                psi: pred_result.psi,
                trajectory: pred_result.gamma_raw,
            },
            None,
        )
    };

    PreemptiveResult {
        predictive: pred_result,
        omega: omega_val,
        safety_lambda: lambda.value(),
        preemptive_threshold,
        pi,
        decision,
        intervention: intervention_result,
    }
}

/// Convenience function with default configuration.
#[must_use]
pub fn evaluate_default(
    gibbs_params: &GibbsParams,
    reporting_data: &[ReportingDataPoint],
    noise_params: &NoiseParams,
    seriousness: Seriousness,
) -> PreemptiveResult {
    evaluate(
        gibbs_params,
        reporting_data,
        noise_params,
        seriousness,
        &PreemptiveConfig::default(),
    )
}

/// Estimates the recommended intervention strength from Pi and Omega.
///
/// Maps the intervention index to a practical intervention level using
/// a proportional scaling relative to REMS-level intervention.
fn estimate_intervention_strength(pi: f64, omega: f64, config: &PreemptiveConfig) -> f64 {
    if omega <= 0.0 {
        return 0.0;
    }
    // Scale: Pi * normalized_omega maps to intervention strength
    // Higher Pi and Omega -> stronger intervention
    let normalized_omega = omega / severity::omega(Seriousness::Fatal);
    let raw_strength = pi * normalized_omega * intervention::INTERVENTION_REMS;

    // Clamp to [0, INTERVENTION_WITHDRAWAL]
    let _ = config; // config available for future tuning
    raw_strength.clamp(0.0, intervention::INTERVENTION_WITHDRAWAL)
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
    fn evaluate_monitor_no_trajectory() {
        let gibbs = GibbsParams::new(3.0, 10000.0, 0.001);
        let data = make_data(&[(1.0, 5.0), (2.0, 5.0), (3.0, 5.0)]); // flat
        let noise = NoiseParams::new(25.0, 50.0);

        let result = evaluate_default(&gibbs, &data, &noise, Seriousness::Fatal);

        assert_eq!(result.decision.tier(), 1);
        assert!(!result.decision.requires_intervention());
    }

    #[test]
    fn evaluate_intervene_strong_signal_fatal() {
        let gibbs = GibbsParams::new(3.0, 10000.0, 0.001); // DeltaG = -7
        let data = make_data(&[(1.0, 2.0), (2.0, 5.0), (3.0, 11.0)]); // escalating
        let noise = NoiseParams::new(25.0, 50.0); // organic

        let result = evaluate_default(&gibbs, &data, &noise, Seriousness::Fatal);

        // Feasibility > 0.99, gamma > 0, retention > 0.9
        // Psi should be substantial
        assert!(result.predictive.psi > 0.0);
        // Omega for Fatal = 10.0
        assert!((result.omega - 10.0).abs() < f64::EPSILON);
        // Safety lambda for Fatal = 0.3
        assert!((result.safety_lambda - 0.3).abs() < f64::EPSILON);
        // Pi = Psi * 10.0 - 0.1, should be positive
        assert!(result.pi > 0.0);
        // Should recommend intervention
        assert_eq!(result.decision.tier(), 3);
        assert!(result.decision.requires_intervention());
        assert!(result.intervention.is_some());
    }

    #[test]
    fn evaluate_predict_moderate_signal() {
        let gibbs = GibbsParams::new(3.0, 10000.0, 0.001);
        let data = make_data(&[(1.0, 5.0), (2.0, 5.1), (3.0, 5.2)]); // barely escalating
        let noise = NoiseParams::new(25.0, 50.0);

        let result = evaluate_default(&gibbs, &data, &noise, Seriousness::NonSerious);

        // Psi should be small but positive (tiny trajectory)
        // For non-serious: lambda = 0.7, threshold = 0.35
        // Small Psi might be below threshold -> Predict or Monitor
        assert!(result.decision.tier() <= 2);
    }

    #[test]
    fn evaluate_fatal_lower_threshold_than_non_serious() {
        let fatal_lambda = SafetyLambda::from_seriousness(Seriousness::Fatal);
        let ns_lambda = SafetyLambda::from_seriousness(Seriousness::NonSerious);

        let fatal_threshold = fatal_lambda.apply(DEFAULT_DETECTION_THRESHOLD);
        let ns_threshold = ns_lambda.apply(DEFAULT_DETECTION_THRESHOLD);

        assert!(fatal_threshold < ns_threshold);
    }

    #[test]
    fn evaluate_stimulated_blocks_intervention() {
        let gibbs = GibbsParams::new(3.0, 10000.0, 0.001);
        let data = make_data(&[(1.0, 2.0), (2.0, 5.0), (3.0, 11.0)]);
        let noise = NoiseParams::new(500.0, 50.0); // 10x stimulated

        let result = evaluate_default(&gibbs, &data, &noise, Seriousness::Fatal);

        // Stimulated reporting should suppress Psi near zero
        assert!(result.predictive.psi < 0.01);
        assert!(result.decision.tier() <= 2);
    }

    #[test]
    fn evaluate_pi_formula() {
        let gibbs = GibbsParams::new(3.0, 10000.0, 0.001);
        let data = make_data(&[(1.0, 2.0), (2.0, 5.0), (3.0, 11.0)]);
        let noise = NoiseParams::new(25.0, 50.0);

        let result = evaluate_default(&gibbs, &data, &noise, Seriousness::Fatal);

        // Verify Pi = Psi * Omega - C
        let expected_pi = result.predictive.psi * result.omega - DEFAULT_INTERVENTION_COST;
        assert!((result.pi - expected_pi).abs() < 1e-10);
    }

    #[test]
    fn evaluate_preemptive_threshold_formula() {
        let result = evaluate_default(
            &GibbsParams::new(3.0, 10000.0, 0.001),
            &make_data(&[(1.0, 2.0), (2.0, 5.0), (3.0, 11.0)]),
            &NoiseParams::new(25.0, 50.0),
            Seriousness::LifeThreatening,
        );

        // theta_preemptive = theta_detection * lambda_safety
        // For Life-threatening: lambda = 0.3
        let expected_threshold = DEFAULT_DETECTION_THRESHOLD * 0.3;
        assert!((result.preemptive_threshold - expected_threshold).abs() < f64::EPSILON);
    }

    #[test]
    fn evaluate_intervention_result_populated() {
        let gibbs = GibbsParams::new(3.0, 10000.0, 0.001);
        let data = make_data(&[(1.0, 2.0), (2.0, 5.0), (3.0, 11.0)]);
        let noise = NoiseParams::new(25.0, 50.0);

        let result = evaluate_default(&gibbs, &data, &noise, Seriousness::Fatal);

        if result.decision.requires_intervention() {
            let intervention = result.intervention.as_ref();
            assert!(intervention.is_some());
            if let Some(iv) = intervention {
                assert!(iv.inhibited_rate <= iv.original_rate);
                assert!(iv.reduction_fraction >= 0.0);
                assert!(iv.reduction_fraction <= 1.0);
            }
        }
    }

    #[test]
    fn evaluate_non_serious_higher_threshold() {
        let config = PreemptiveConfig::default();

        // Non-serious: lambda=0.7, threshold=0.35
        let ns_threshold = SafetyLambda::from_seriousness(Seriousness::NonSerious)
            .apply(config.detection_threshold);
        // Fatal: lambda=0.3, threshold=0.15
        let fatal_threshold =
            SafetyLambda::from_seriousness(Seriousness::Fatal).apply(config.detection_threshold);

        assert!(ns_threshold > fatal_threshold);
        assert!((ns_threshold - 0.35).abs() < f64::EPSILON);
        assert!((fatal_threshold - 0.15).abs() < f64::EPSILON);
    }

    #[test]
    fn evaluate_empty_data_monitors() {
        let gibbs = GibbsParams::new(3.0, 10000.0, 0.001);
        let data: Vec<ReportingDataPoint> = vec![];
        let noise = NoiseParams::new(25.0, 50.0);

        let result = evaluate_default(&gibbs, &data, &noise, Seriousness::Fatal);
        assert_eq!(result.decision.tier(), 1);
    }

    #[test]
    fn estimate_strength_scales_with_omega() {
        let config = PreemptiveConfig::default();

        let s1 = estimate_intervention_strength(1.0, 5.0, &config);
        let s2 = estimate_intervention_strength(1.0, 10.0, &config);

        // Higher omega -> stronger recommended intervention
        assert!(s2 > s1);
    }

    #[test]
    fn estimate_strength_clamped() {
        let config = PreemptiveConfig::default();

        // Very high Pi and Omega should clamp at withdrawal level
        let s = estimate_intervention_strength(1000.0, 10.0, &config);
        assert!((s - intervention::INTERVENTION_WITHDRAWAL).abs() < f64::EPSILON);
    }

    #[test]
    fn estimate_strength_zero_omega() {
        let config = PreemptiveConfig::default();
        let s = estimate_intervention_strength(5.0, 0.0, &config);
        assert!((s - 0.0).abs() < f64::EPSILON);
    }
}
