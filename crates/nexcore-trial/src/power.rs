//! Sample size and power analysis per ICH E9(R1) §3.5
//!
//! Uses the Abramowitz & Stegun (1964) normal quantile approximation.
//! No external statistics crate required.

use crate::error::TrialError;

/// Approximates the standard normal quantile (inverse CDF) for probability `p`.
///
/// Uses Abramowitz & Stegun (1964) formula 26.2.17.
/// Valid for `p` in (0, 1). Returns the z-score such that Φ(z) = 1 - p.
pub fn z_score_from_alpha(p: f64) -> f64 {
    // We want z such that the tail probability is p (one-sided).
    // For alpha=0.05 two-sided: call with p=0.025, returns ~1.96
    let p = p.min(1.0 - 1e-10).max(1e-10);
    let p_adj = if p > 0.5 { 1.0 - p } else { p };

    let t = (-2.0_f64 * p_adj.ln()).sqrt();
    let c0 = 2.515_517;
    let c1 = 0.802_853;
    let c2 = 0.010_328;
    let d1 = 1.432_788;
    let d2 = 0.189_269;
    let d3 = 0.001_308;

    let z = t - (c0 + c1 * t + c2 * t * t) / (1.0 + d1 * t + d2 * t * t + d3 * t * t * t);

    if p > 0.5 { -z } else { z }
}

/// Compute per-arm sample size for a two-proportion Z-test.
///
/// Based on the standard formula from E9(R1) §3.5:
/// `n = (z_α/2 * √(2p̄(1-p̄)) + z_β * √(p1(1-p1) + p2(1-p2)))² / (p1-p2)²`
///
/// # Arguments
/// - `p1`: Event rate in arm 1 (control)
/// - `p2`: Event rate in arm 2 (treatment)
/// - `alpha`: Two-sided type I error rate
/// - `power`: Required statistical power (1 - β)
///
/// # Returns
/// Per-arm sample size (round up).
pub fn sample_size_two_proportion(
    p1: f64,
    p2: f64,
    alpha: f64,
    power: f64,
) -> Result<u32, TrialError> {
    validate_proportion(p1, "p1")?;
    validate_proportion(p2, "p2")?;
    validate_alpha(alpha)?;
    validate_power(power)?;

    if (p1 - p2).abs() < 1e-10 {
        return Err(TrialError::InvalidParameter(
            "p1 and p2 must differ to compute sample size".into(),
        ));
    }

    let z_alpha = z_score_from_alpha(alpha / 2.0); // two-sided
    let z_beta = z_score_from_alpha(1.0 - power);

    let p_bar = (p1 + p2) / 2.0;
    let delta = (p1 - p2).abs();

    let numerator =
        z_alpha * (2.0 * p_bar * (1.0 - p_bar)).sqrt()
        + z_beta * (p1 * (1.0 - p1) + p2 * (1.0 - p2)).sqrt();
    let n = (numerator / delta).powi(2);

    Ok(n.ceil() as u32)
}

/// Compute per-arm sample size for a two-sample t-test (Cohen's d).
///
/// Formula: `n_per_arm = 2 * (z_α/2 + z_β)² / d²`
///
/// # Arguments
/// - `effect_size`: Cohen's d (standardized mean difference)
/// - `alpha`: Two-sided type I error rate
/// - `power`: Required statistical power
///
/// # Returns
/// Per-arm sample size (round up).
pub fn sample_size_two_mean(
    effect_size: f64,
    alpha: f64,
    power: f64,
) -> Result<u32, TrialError> {
    if effect_size <= 0.0 {
        return Err(TrialError::InvalidParameter(
            "effect_size must be positive".into(),
        ));
    }
    validate_alpha(alpha)?;
    validate_power(power)?;

    let z_alpha = z_score_from_alpha(alpha / 2.0);
    let z_beta = z_score_from_alpha(1.0 - power);

    let n = 2.0 * (z_alpha + z_beta).powi(2) / effect_size.powi(2);
    Ok(n.ceil() as u32)
}

/// Compute number of events required for a log-rank test (survival endpoint).
///
/// # Arguments
/// - `hazard_ratio`: Expected hazard ratio (treatment / control)
/// - `alpha`: Two-sided type I error rate
/// - `power`: Required statistical power
/// - `event_prob`: Expected probability of observing the event during follow-up
///
/// # Returns
/// Total sample size (both arms combined, round up).
pub fn sample_size_survival(
    hazard_ratio: f64,
    alpha: f64,
    power: f64,
    event_prob: f64,
) -> Result<u32, TrialError> {
    if hazard_ratio <= 0.0 {
        return Err(TrialError::InvalidParameter(
            "hazard_ratio must be positive".into(),
        ));
    }
    if (hazard_ratio - 1.0).abs() < 1e-10 {
        return Err(TrialError::InvalidParameter(
            "hazard_ratio of 1.0 implies no difference — cannot compute sample size".into(),
        ));
    }
    validate_proportion(event_prob, "event_prob")?;
    validate_alpha(alpha)?;
    validate_power(power)?;

    let z_alpha = z_score_from_alpha(alpha / 2.0);
    let z_beta = z_score_from_alpha(1.0 - power);

    // Events required per arm (Schoenfeld formula)
    let events_per_arm = (z_alpha + z_beta).powi(2) / hazard_ratio.ln().powi(2);
    let total_events = 2.0 * events_per_arm;
    let total_n = total_events / event_prob;

    Ok(total_n.ceil() as u32)
}

// ── Validation helpers ────────────────────────────────────────────────────────

fn validate_proportion(p: f64, name: &str) -> Result<(), TrialError> {
    if !(0.0..=1.0).contains(&p) {
        return Err(TrialError::InvalidParameter(format!(
            "{name} must be in [0, 1], got {p}"
        )));
    }
    Ok(())
}

fn validate_alpha(alpha: f64) -> Result<(), TrialError> {
    if alpha <= 0.0 || alpha >= 1.0 {
        return Err(TrialError::InvalidParameter(format!(
            "alpha must be in (0, 1), got {alpha}"
        )));
    }
    Ok(())
}

fn validate_power(power: f64) -> Result<(), TrialError> {
    if power <= 0.0 || power >= 1.0 {
        return Err(TrialError::InvalidParameter(format!(
            "power must be in (0, 1), got {power}"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_z_score_known_values() {
        // z for alpha/2 = 0.025 should be ~1.96
        let z = z_score_from_alpha(0.025);
        assert!((z - 1.96).abs() < 0.005, "Expected ~1.96, got {z}");

        // z for 1 - power = 0.20 should be ~0.842
        let z_beta = z_score_from_alpha(0.20);
        assert!((z_beta - 0.842).abs() < 0.005, "Expected ~0.842, got {z_beta}");
    }

    #[test]
    fn test_two_proportion_power() {
        // p1=0.50, p2=0.60, alpha=0.05, power=0.80 → ~388 per arm
        let n = sample_size_two_proportion(0.50, 0.60, 0.05, 0.80);
        assert!(n.is_ok(), "Expected Ok, got {n:?}");
        let n = n.unwrap();
        assert!(
            n > 300 && n < 500,
            "Expected 300 < n < 500, got {n}"
        );
    }

    #[test]
    fn test_two_mean_power() {
        // effect_size=0.5 (medium Cohen's d), alpha=0.05, power=0.80 → ~64 per arm
        let n = sample_size_two_mean(0.5, 0.05, 0.80);
        assert!(n.is_ok(), "Expected Ok, got {n:?}");
        let n = n.unwrap();
        assert!(
            n > 50 && n < 80,
            "Expected 50 < n < 80, got {n}"
        );
    }

    #[test]
    fn test_survival_power() {
        // hazard_ratio=0.7, alpha=0.05, power=0.80, event_prob=0.80
        let n = sample_size_survival(0.7, 0.05, 0.80, 0.80);
        assert!(n.is_ok(), "Expected Ok, got {n:?}");
        let n = n.unwrap();
        assert!(n > 50, "Expected n > 50, got {n}");
    }

    #[test]
    fn test_invalid_equal_proportions() {
        let result = sample_size_two_proportion(0.50, 0.50, 0.05, 0.80);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_power_too_low() {
        let result = sample_size_two_proportion(0.50, 0.60, 0.05, 0.50);
        // Should succeed (0.50 is valid), but verify we accept it
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_alpha_out_of_range() {
        let result = sample_size_two_proportion(0.50, 0.60, 1.5, 0.80);
        assert!(result.is_err());
    }
}
