//! Competitive Inhibition Intervention Model.
//!
//! Tier: T2-C (maps to Product `x` + Comparison `kappa` + Quantity `N`)
//!
//! Models preemptive action as a competitive inhibitor of harm, using
//! Michaelis-Menten enzyme kinetics with competitive inhibition:
//!
//! ```text
//! v_inhibited = v_max * [S] / (K_m * (1 + [I]/K_i) + [S])
//! ```
//!
//! Where:
//! - `v_max` = maximum harm rate (uninhibited)
//! - `[S]` = substrate concentration (exposure level / patient-years)
//! - `K_m` = Michaelis constant (harm rate at half-max without inhibition)
//! - `[I]` = intervention strength:
//!   - 0 = no intervention
//!   - 10 = moderate (Dear Healthcare Professional Communication)
//!   - 25 = strong (REMS/restriction)
//! - `K_i` = inhibition constant (intervention potency)

use crate::types::InterventionResult;

/// Default Michaelis constant (harm rate at half-max).
pub const DEFAULT_K_M: f64 = 5.0;

/// Default inhibition constant (intervention potency).
pub const DEFAULT_K_I: f64 = 5.0;

/// Reference intervention strengths.
pub const INTERVENTION_NONE: f64 = 0.0;
/// Dear Healthcare Professional Communication.
pub const INTERVENTION_DHPC: f64 = 10.0;
/// Risk Evaluation and Mitigation Strategy / restriction.
pub const INTERVENTION_REMS: f64 = 25.0;
/// Market withdrawal.
pub const INTERVENTION_WITHDRAWAL: f64 = 100.0;

/// Computes the inhibited harm rate using competitive inhibition kinetics.
///
/// ```text
/// v = v_max * [S] / (K_m * (1 + [I]/K_i) + [S])
/// ```
///
/// # Arguments
///
/// * `v_max` - Maximum harm rate (uninhibited).
/// * `substrate` - Substrate concentration (patient-years of exposure).
/// * `inhibitor` - Intervention strength [I].
/// * `k_m` - Michaelis constant.
/// * `k_i` - Inhibition constant.
///
/// Returns the inhibited harm rate. Returns 0.0 if `k_i <= 0`.
#[must_use]
pub fn inhibited_rate(v_max: f64, substrate: f64, inhibitor: f64, k_m: f64, k_i: f64) -> f64 {
    if k_i <= 0.0 {
        return 0.0;
    }
    if substrate <= 0.0 {
        return 0.0;
    }

    let apparent_km = k_m * (1.0 + inhibitor / k_i);
    let denominator = apparent_km + substrate;

    if denominator <= 0.0 {
        return 0.0;
    }

    v_max * substrate / denominator
}

/// Computes the uninhibited harm rate (no intervention).
///
/// ```text
/// v = v_max * [S] / (K_m + [S])
/// ```
#[must_use]
pub fn uninhibited_rate(v_max: f64, substrate: f64, k_m: f64) -> f64 {
    inhibited_rate(v_max, substrate, 0.0, k_m, 1.0) // K_i irrelevant when I=0
}

/// Computes the full intervention result including rate reduction metrics.
///
/// # Arguments
///
/// * `v_max` - Maximum harm rate.
/// * `substrate` - Substrate concentration (patient-years).
/// * `inhibitor` - Intervention strength [I].
/// * `k_m` - Michaelis constant.
/// * `k_i` - Inhibition constant.
#[must_use]
pub fn intervention_effect(
    v_max: f64,
    substrate: f64,
    inhibitor: f64,
    k_m: f64,
    k_i: f64,
) -> InterventionResult {
    let original = uninhibited_rate(v_max, substrate, k_m);
    let inhibited = inhibited_rate(v_max, substrate, inhibitor, k_m, k_i);

    let (reduction_fraction, reduction_percentage) = if original > 0.0 {
        let fraction = 1.0 - inhibited / original;
        (fraction, fraction * 100.0)
    } else {
        (0.0, 0.0)
    };

    InterventionResult {
        inhibited_rate: inhibited,
        original_rate: original,
        reduction_fraction,
        reduction_percentage,
    }
}

/// Computes intervention effect with default K_m and K_i.
#[must_use]
pub fn intervention_effect_default(
    v_max: f64,
    substrate: f64,
    inhibitor: f64,
) -> InterventionResult {
    intervention_effect(v_max, substrate, inhibitor, DEFAULT_K_M, DEFAULT_K_I)
}

/// Estimates the intervention strength needed to reduce the harm rate by a target fraction.
///
/// Solves the competitive inhibition equation for [I]:
///
/// ```text
/// [I] = K_i * ((v_max * [S] / (v_target * (K_m + [S]/v_target)) ) - 1) * K_m
/// ```
///
/// Simplified: given target_rate = original_rate * (1 - target_reduction):
///
/// ```text
/// [I] = K_i * (original_rate / target_rate - 1) * K_m / (K_m + [S] - K_m * original_rate / target_rate)
/// ```
///
/// Returns `None` if the target reduction is not achievable (>= 100%) or inputs are invalid.
#[must_use]
pub fn required_intervention_strength(
    v_max: f64,
    substrate: f64,
    target_reduction_fraction: f64,
    k_m: f64,
    k_i: f64,
) -> Option<f64> {
    if target_reduction_fraction >= 1.0 || target_reduction_fraction <= 0.0 {
        return None;
    }
    if v_max <= 0.0 || substrate <= 0.0 || k_m <= 0.0 || k_i <= 0.0 {
        return None;
    }

    let original = uninhibited_rate(v_max, substrate, k_m);
    if original <= 0.0 {
        return None;
    }

    let target_rate = original * (1.0 - target_reduction_fraction);
    if target_rate <= 0.0 {
        return None;
    }

    // From v_inhibited = v_max * S / (K_m * (1 + I/K_i) + S)
    // Solve for I:
    // v_target * (K_m * (1 + I/K_i) + S) = v_max * S
    // K_m * (1 + I/K_i) = v_max * S / v_target - S
    // 1 + I/K_i = (v_max * S / v_target - S) / K_m
    // I/K_i = (v_max * S / v_target - S) / K_m - 1
    // I = K_i * ((v_max * S / v_target - S) / K_m - 1)

    let numerator = v_max * substrate / target_rate - substrate;
    let ratio = numerator / k_m - 1.0;

    if ratio < 0.0 {
        return None;
    }

    Some(k_i * ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uninhibited_rate_basic() {
        // v = 100 * 10 / (5 + 10) = 1000/15 = 66.67
        let v = uninhibited_rate(100.0, 10.0, 5.0);
        let expected = 100.0 * 10.0 / (5.0 + 10.0);
        assert!((v - expected).abs() < 1e-10);
    }

    #[test]
    fn inhibited_rate_reduces_harm() {
        let v_uninhibited = uninhibited_rate(100.0, 10.0, DEFAULT_K_M);
        let v_inhibited = inhibited_rate(100.0, 10.0, INTERVENTION_DHPC, DEFAULT_K_M, DEFAULT_K_I);
        assert!(v_inhibited < v_uninhibited);
    }

    #[test]
    fn inhibited_rate_stronger_intervention() {
        let v_dhpc = inhibited_rate(100.0, 10.0, INTERVENTION_DHPC, DEFAULT_K_M, DEFAULT_K_I);
        let v_rems = inhibited_rate(100.0, 10.0, INTERVENTION_REMS, DEFAULT_K_M, DEFAULT_K_I);
        assert!(v_rems < v_dhpc);
    }

    #[test]
    fn inhibited_rate_no_intervention() {
        let v_none = inhibited_rate(100.0, 10.0, INTERVENTION_NONE, DEFAULT_K_M, DEFAULT_K_I);
        let v_uninhibited = uninhibited_rate(100.0, 10.0, DEFAULT_K_M);
        assert!((v_none - v_uninhibited).abs() < 1e-10);
    }

    #[test]
    fn inhibited_rate_zero_substrate() {
        let v = inhibited_rate(100.0, 0.0, INTERVENTION_DHPC, DEFAULT_K_M, DEFAULT_K_I);
        assert!((v - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn inhibited_rate_zero_ki() {
        let v = inhibited_rate(100.0, 10.0, INTERVENTION_DHPC, DEFAULT_K_M, 0.0);
        assert!((v - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn intervention_effect_basic() {
        let result = intervention_effect_default(100.0, 10.0, INTERVENTION_DHPC);
        assert!(result.inhibited_rate < result.original_rate);
        assert!(result.reduction_fraction > 0.0);
        assert!(result.reduction_fraction < 1.0);
        assert!((result.reduction_percentage - result.reduction_fraction * 100.0).abs() < 1e-10);
    }

    #[test]
    fn intervention_effect_no_intervention() {
        let result = intervention_effect_default(100.0, 10.0, INTERVENTION_NONE);
        assert!((result.inhibited_rate - result.original_rate).abs() < 1e-10);
        assert!(result.reduction_fraction.abs() < 1e-10);
    }

    #[test]
    fn intervention_effect_rems_stronger_than_dhpc() {
        let dhpc = intervention_effect_default(100.0, 10.0, INTERVENTION_DHPC);
        let rems = intervention_effect_default(100.0, 10.0, INTERVENTION_REMS);
        assert!(rems.reduction_fraction > dhpc.reduction_fraction);
    }

    #[test]
    fn intervention_effect_withdrawal() {
        let result = intervention_effect_default(100.0, 10.0, INTERVENTION_WITHDRAWAL);
        // Very strong intervention should reduce rate significantly
        assert!(result.reduction_percentage > 50.0);
    }

    #[test]
    fn required_intervention_50_percent() {
        let strength = required_intervention_strength(100.0, 10.0, 0.5, DEFAULT_K_M, DEFAULT_K_I);
        assert!(strength.is_some());
        let i = strength.unwrap_or(0.0);
        assert!(i > 0.0);

        // Verify: applying this strength should give ~50% reduction
        let result = intervention_effect(100.0, 10.0, i, DEFAULT_K_M, DEFAULT_K_I);
        assert!((result.reduction_fraction - 0.5).abs() < 1e-6);
    }

    #[test]
    fn required_intervention_invalid_reduction() {
        // 100% reduction is not achievable
        assert!(
            required_intervention_strength(100.0, 10.0, 1.0, DEFAULT_K_M, DEFAULT_K_I).is_none()
        );
        // Negative reduction doesn't make sense
        assert!(
            required_intervention_strength(100.0, 10.0, -0.1, DEFAULT_K_M, DEFAULT_K_I).is_none()
        );
        // Zero reduction
        assert!(
            required_intervention_strength(100.0, 10.0, 0.0, DEFAULT_K_M, DEFAULT_K_I).is_none()
        );
    }

    #[test]
    fn required_intervention_invalid_params() {
        assert!(required_intervention_strength(0.0, 10.0, 0.5, DEFAULT_K_M, DEFAULT_K_I).is_none());
        assert!(
            required_intervention_strength(100.0, 0.0, 0.5, DEFAULT_K_M, DEFAULT_K_I).is_none()
        );
        assert!(required_intervention_strength(100.0, 10.0, 0.5, 0.0, DEFAULT_K_I).is_none());
        assert!(required_intervention_strength(100.0, 10.0, 0.5, DEFAULT_K_M, 0.0).is_none());
    }
}
