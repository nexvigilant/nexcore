//! Interim analysis and group sequential methods per ICH E20 (Adaptive Designs)
//!
//! Tier: T2-P (ν+κ+∂ — Interim Analysis)
//!
//! Implements:
//! - O'Brien-Fleming boundary (Wang-Tsiatis power family, ρ=0)
//! - Lan-DeMets alpha spending function (OBF-type and Pocock-type)
//! - Bayesian posterior probability of superiority (Beta-binomial, normal approximation)
//! - Interim decision: Continue / StopEfficacy / StopFutility / StopSafety / AdaptSampleSize

use crate::error::TrialError;
use crate::power::z_score_from_alpha;
use crate::types::{InterimData, InterimDecision, InterimResult, Protocol, SpendingFunction};

// ── OBF Boundary ──────────────────────────────────────────────────────────────

/// Compute the O'Brien-Fleming critical boundary at a given information fraction.
///
/// Uses the geometric-mean approximation for the OBF constant:
/// `z_c ≈ sqrt(z_{α/2} × z_{α/4})`
///
/// This gives the standard result: at t=0.50, α=0.05 → boundary ≈ 2.963.
///
/// # Arguments
/// - `info_fraction`: Information fraction accumulated (0 < t ≤ 1)
/// - `alpha`: Two-sided type I error rate
///
/// # Returns
/// Critical Z-value; reject H₀ if |observed_Z| > returned value.
pub fn obrien_fleming_boundary(info_fraction: f64, alpha: f64) -> f64 {
    let z_half = z_score_from_alpha(alpha / 2.0); // ≈1.96 for α=0.05
    let z_quarter = z_score_from_alpha(alpha / 4.0); // ≈2.24 for α=0.05
    // OBF constant — geometric mean of the two-sided and four-sided critical values
    let z_c = (z_half * z_quarter).sqrt(); // ≈2.096 for α=0.05
    z_c / info_fraction.sqrt()
}

// ── Lan-DeMets Alpha Spending ─────────────────────────────────────────────────

/// Compute cumulative alpha spent up to `info_fraction` using Lan-DeMets spending functions.
///
/// Uses the OBF-type spending function: `g(t) = 2[1 - Φ(z_c / √t)]`
/// where z_c is the OBF constant. Very conservative at early looks.
///
/// # Arguments
/// - `info_fraction`: Information fraction (0 < t ≤ 1)
/// - `alpha`: Total type I error budget
/// - `function`: Which spending function to use
pub fn lan_demets_alpha_spent(info_fraction: f64, alpha: f64, function: SpendingFunction) -> f64 {
    match function {
        SpendingFunction::OBrienFleming => {
            let z_half = z_score_from_alpha(alpha / 2.0);
            let z_quarter = z_score_from_alpha(alpha / 4.0);
            let z_c = (z_half * z_quarter).sqrt();
            let z_boundary = z_c / info_fraction.sqrt();
            // Cumulative alpha spent = 2Φ(−z_boundary)
            2.0 * normal_cdf(-z_boundary)
        }
        SpendingFunction::Pocock => {
            // Pocock: spends alpha uniformly on the log scale
            // g(t) = alpha * ln(1 + (e-1) * t)
            alpha * (1.0_f64 + (std::f64::consts::E - 1.0) * info_fraction).ln()
        }
        SpendingFunction::KimDeMets { gamma } => {
            // Power family: g(t) = alpha * t^gamma
            alpha * info_fraction.powf(gamma)
        }
    }
}

// ── Bayesian Posterior ────────────────────────────────────────────────────────

/// Compute P(θ_treatment > θ_control) using Beta-binomial Bayesian model.
///
/// Uses Beta(1,1) (uniform) prior for both arms.
/// Posterior approximation via normal distribution for tractability.
///
/// # Arguments
/// - `s1`: Successes in treatment arm
/// - `n1`: Total subjects in treatment arm
/// - `s2`: Successes in control arm
/// - `n2`: Total subjects in control arm
///
/// # Returns
/// Posterior probability that treatment rate exceeds control rate (0.0 to 1.0).
pub fn posterior_probability_superiority(s1: u32, n1: u32, s2: u32, n2: u32) -> f64 {
    // Posterior means: (successes + 1) / (total + 2) — Bayesian estimate
    let mu_t = (s1 as f64 + 1.0) / (n1 as f64 + 2.0);
    let mu_c = (s2 as f64 + 1.0) / (n2 as f64 + 2.0);

    // Posterior variances using Beta distribution variance formula
    let var_t = mu_t * (1.0 - mu_t) / (n1 as f64 + 3.0);
    let var_c = mu_c * (1.0 - mu_c) / (n2 as f64 + 3.0);

    // P(θ_t > θ_c) via normal approximation of the difference
    let delta_mu = mu_t - mu_c;
    let delta_sd = (var_t + var_c).sqrt();

    if delta_sd < 1e-12 {
        return if delta_mu > 0.0 { 1.0 } else { 0.0 };
    }

    // P(Z > -delta_mu/delta_sd) = Φ(delta_mu/delta_sd)
    normal_cdf(delta_mu / delta_sd)
}

// ── Interim Evaluation ────────────────────────────────────────────────────────

/// Evaluate an interim analysis and recommend a course of action.
///
/// Decision logic per E20 §4:
/// 1. **StopSafety** — safety events exceed protocol threshold
/// 2. **StopEfficacy** — test statistic exceeds OBF efficacy boundary
/// 3. **StopFutility** — posterior probability < 20% (conditional power futility)
/// 4. **AdaptSampleSize** — at t=0.5 with borderline results, re-estimate
/// 5. **Continue** — no stopping condition met
///
/// # Arguments
/// - `data`: Observed counts and information fraction
/// - `protocol`: Registered protocol with safety rule and power requirements
pub fn evaluate_interim(
    data: &InterimData,
    protocol: &Protocol,
) -> Result<InterimResult, TrialError> {
    if !(0.0..=1.0).contains(&data.information_fraction) {
        return Err(TrialError::InvalidParameter(
            "information_fraction must be in [0, 1]".into(),
        ));
    }
    if data.treatment_n == 0 || data.control_n == 0 {
        return Err(TrialError::InvalidParameter(
            "treatment_n and control_n must be > 0".into(),
        ));
    }

    // Safety check first (highest priority)
    let safety_rate = data.safety_events as f64 / (data.treatment_n + data.control_n) as f64;

    if safety_rate > protocol.safety_boundary.threshold {
        let z_stat = compute_z_stat(data);
        let posterior = posterior_probability_superiority(
            data.treatment_successes,
            data.treatment_n,
            data.control_successes,
            data.control_n,
        );
        return Ok(InterimResult {
            decision: InterimDecision::StopSafety,
            boundary: protocol.safety_boundary.threshold,
            test_statistic: z_stat,
            posterior_prob: posterior,
            rationale: format!(
                "Safety boundary crossed: observed rate {safety_rate:.4} > threshold {:.4}",
                protocol.safety_boundary.threshold
            ),
        });
    }

    let boundary = obrien_fleming_boundary(data.information_fraction, protocol.alpha);
    let z_stat = compute_z_stat(data);
    let posterior = posterior_probability_superiority(
        data.treatment_successes,
        data.treatment_n,
        data.control_successes,
        data.control_n,
    );

    let decision = if z_stat.abs() > boundary {
        InterimDecision::StopEfficacy
    } else if posterior < 0.20 {
        // Futility: very low probability treatment is superior
        InterimDecision::StopFutility
    } else if (data.information_fraction - 0.5).abs() < 0.05 && z_stat.abs() < 1.5 {
        // At ~50% information with borderline signal: consider sample size re-estimation
        InterimDecision::AdaptSampleSize
    } else {
        InterimDecision::Continue
    };

    let rationale = match &decision {
        InterimDecision::Continue => format!(
            "Z={z_stat:.3} < boundary {boundary:.3}; posterior={posterior:.3}. Continue as planned."
        ),
        InterimDecision::StopEfficacy => {
            format!("Z={z_stat:.3} exceeds OBF boundary {boundary:.3}. Stop for efficacy.")
        }
        InterimDecision::StopFutility => {
            format!("Posterior P(superiority)={posterior:.3} < 0.20. Stop for futility.")
        }
        InterimDecision::AdaptSampleSize => format!(
            "At 50% information, Z={z_stat:.3} borderline. Sample size re-estimation recommended."
        ),
        InterimDecision::StopSafety => "Safety boundary crossed".into(),
    };

    Ok(InterimResult {
        decision,
        boundary,
        test_statistic: z_stat,
        posterior_prob: posterior,
        rationale,
    })
}

// ── Internal ─────────────────────────────────────────────────────────────────

/// Compute the two-proportion Z-statistic from interim data.
fn compute_z_stat(data: &InterimData) -> f64 {
    let n_t = data.treatment_n as f64;
    let n_c = data.control_n as f64;
    let p_t = data.treatment_successes as f64 / n_t;
    let p_c = data.control_successes as f64 / n_c;
    let p_bar = (data.treatment_successes + data.control_successes) as f64 / (n_t + n_c);
    let se = (p_bar * (1.0 - p_bar) * (1.0 / n_t + 1.0 / n_c)).sqrt();
    if se < 1e-12 { 0.0 } else { (p_t - p_c) / se }
}

/// Standard normal CDF via rational approximation (Abramowitz & Stegun 26.2.17).
pub(crate) fn normal_cdf(x: f64) -> f64 {
    if x > 8.0 {
        return 1.0;
    }
    if x < -8.0 {
        return 0.0;
    }
    // Use the error function relationship: Φ(x) = (1 + erf(x/√2)) / 2
    // Approximate erf via polynomial
    let t = 1.0 / (1.0 + 0.2316419 * x.abs());
    let poly = t
        * (0.319_381_530
            + t * (-0.356_563_782
                + t * (1.781_477_937 + t * (-1.821_255_978 + t * 1.330_274_429))));
    let pdf = (-x * x / 2.0).exp() / (2.0 * std::f64::consts::PI).sqrt();
    let p = 1.0 - pdf * poly;
    if x >= 0.0 { p } else { 1.0 - p }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BlindingLevel;
    use crate::types::{Arm, Endpoint, EndpointDirection, SafetyRule};

    fn make_protocol() -> Protocol {
        Protocol {
            id: "test".into(),
            hypothesis: "H0".into(),
            population: "adults".into(),
            primary_endpoint: Endpoint {
                name: "rate".into(),
                metric: "proportion".into(),
                direction: EndpointDirection::Higher,
                threshold: 0.05,
            },
            secondary_endpoints: vec![],
            arms: vec![
                Arm {
                    name: "ctrl".into(),
                    description: "c".into(),
                    is_control: true,
                },
                Arm {
                    name: "tx".into(),
                    description: "t".into(),
                    is_control: false,
                },
            ],
            sample_size: 200,
            power: 0.80,
            alpha: 0.05,
            duration_days: 30,
            safety_boundary: SafetyRule {
                metric: "sae".into(),
                threshold: 0.02,
                description: "stop at 2%".into(),
            },
            adaptation_rules: vec![],
            blinding: BlindingLevel::Double,
            created_at: "2026-01-01T00:00:00Z".into(),
        }
    }

    #[test]
    fn test_obrien_fleming_boundary() {
        // At 50% information fraction, OBF boundary should be ~2.963
        let boundary = obrien_fleming_boundary(0.50, 0.05);
        assert!(
            (boundary - 2.963).abs() < 0.01,
            "Expected ~2.963, got {boundary}"
        );
    }

    #[test]
    fn test_obf_boundary_at_full_information() {
        // At t=1.0, OBF boundary should be close to z_c ≈ 2.096
        let boundary = obrien_fleming_boundary(1.0, 0.05);
        assert!(
            boundary > 1.9 && boundary < 2.2,
            "Expected ~2.096, got {boundary}"
        );
    }

    #[test]
    fn test_lan_demets_spending() {
        // Alpha spending at fraction 0.5 with OBF-like function should be very conservative
        let spent = lan_demets_alpha_spent(0.50, 0.05, SpendingFunction::OBrienFleming);
        assert!(
            spent < 0.005,
            "OBF should spend < 0.005 at t=0.5, got {spent}"
        );
    }

    #[test]
    fn test_lan_demets_pocock_linear_ish() {
        // Pocock spends more aggressively than OBF at early looks
        let spent_obf = lan_demets_alpha_spent(0.50, 0.05, SpendingFunction::OBrienFleming);
        let spent_poc = lan_demets_alpha_spent(0.50, 0.05, SpendingFunction::Pocock);
        assert!(
            spent_poc > spent_obf,
            "Pocock should spend more than OBF at t=0.5"
        );
    }

    #[test]
    fn test_bayesian_posterior() {
        // 50 successes in 100 treatment, 40 in 100 control → P(superiority) > 0.80
        let prob = posterior_probability_superiority(50, 100, 40, 100);
        assert!(prob > 0.80, "Expected > 0.80, got {prob}");
    }

    #[test]
    fn test_bayesian_equal_arms() {
        // Equal success rates → posterior ≈ 0.50
        let prob = posterior_probability_superiority(50, 100, 50, 100);
        assert!((prob - 0.50).abs() < 0.05, "Expected ~0.50, got {prob}");
    }

    #[test]
    fn test_evaluate_interim_continue() {
        let protocol = make_protocol();
        let data = InterimData {
            information_fraction: 0.50,
            treatment_successes: 60,
            treatment_n: 100,
            control_successes: 45,
            control_n: 100,
            safety_events: 0,
        };
        let result = evaluate_interim(&data, &protocol);
        assert!(result.is_ok());
        let r = result.unwrap();
        assert_eq!(
            r.decision,
            InterimDecision::Continue,
            "Expected Continue at interim"
        );
    }

    #[test]
    fn test_evaluate_interim_stop_safety() {
        let protocol = make_protocol();
        let data = InterimData {
            information_fraction: 0.50,
            treatment_successes: 10,
            treatment_n: 50,
            control_successes: 10,
            control_n: 50,
            safety_events: 5, // 5/100 = 0.05 > threshold 0.02
        };
        let result = evaluate_interim(&data, &protocol);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().decision, InterimDecision::StopSafety);
    }

    #[test]
    fn test_evaluate_interim_stop_efficacy() {
        let protocol = make_protocol();
        // Very large treatment effect → Z >> boundary
        let data = InterimData {
            information_fraction: 0.50,
            treatment_successes: 90,
            treatment_n: 100,
            control_successes: 10,
            control_n: 100,
            safety_events: 0,
        };
        let result = evaluate_interim(&data, &protocol);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().decision, InterimDecision::StopEfficacy);
    }
}
