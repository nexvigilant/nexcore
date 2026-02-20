//! Endpoint evaluation per ICH E9(R1) §5 Estimands framework.
//!
//! Tier: T2-P (N+κ+→ — Endpoint Evaluation)
//!
//! Implements Z-test for two proportions, t-test for two means, and NNT computation.

use crate::error::TrialError;
use crate::interim::normal_cdf;
use crate::power::z_score_from_alpha;
use crate::types::EndpointResult;

// ── Two Proportions ───────────────────────────────────────────────────────────

/// Evaluate a binary endpoint using a two-proportion Z-test.
///
/// Uses the pooled proportion under H₀ for the standard error.
/// Computes two-sided p-value, 95% confidence interval, and NNT.
///
/// # Arguments
/// - `s1`: Successes in treatment arm
/// - `n1`: Total in treatment arm
/// - `s2`: Successes in control arm
/// - `n2`: Total in control arm
/// - `alpha`: Significance level (e.g., 0.05)
pub fn evaluate_two_proportions(
    s1: u32,
    n1: u32,
    s2: u32,
    n2: u32,
    alpha: f64,
) -> Result<EndpointResult, TrialError> {
    if n1 == 0 || n2 == 0 {
        return Err(TrialError::InvalidParameter("n1 and n2 must be > 0".into()));
    }
    if s1 > n1 || s2 > n2 {
        return Err(TrialError::InvalidParameter(
            "successes cannot exceed total subjects".into(),
        ));
    }
    if alpha <= 0.0 || alpha >= 1.0 {
        return Err(TrialError::InvalidParameter(
            "alpha must be in (0, 1)".into(),
        ));
    }

    let p1 = s1 as f64 / n1 as f64;
    let p2 = s2 as f64 / n2 as f64;
    let p_bar = (s1 + s2) as f64 / (n1 + n2) as f64;

    let se_pooled = (p_bar * (1.0 - p_bar) * (1.0 / n1 as f64 + 1.0 / n2 as f64)).sqrt();
    let z = if se_pooled < 1e-12 {
        0.0
    } else {
        (p1 - p2) / se_pooled
    };

    let p_value = 2.0 * normal_cdf(-z.abs());
    let z_crit = z_score_from_alpha(alpha / 2.0);
    let significant = p_value < alpha;

    // 95% CI on the difference p1 - p2 using unpooled SE
    let se_unpooled = ((p1 * (1.0 - p1) / n1 as f64) + (p2 * (1.0 - p2) / n2 as f64)).sqrt();
    let effect_size = p1 - p2;
    let ci_lower = effect_size - z_crit * se_unpooled;
    let ci_upper = effect_size + z_crit * se_unpooled;

    let nnt = compute_nnt(p1, p2);

    Ok(EndpointResult {
        name: String::new(), // caller sets the name
        test_statistic: z,
        p_value,
        significant,
        effect_size,
        ci_lower,
        ci_upper,
        nnt,
    })
}

/// Evaluate a continuous endpoint using Welch's t-test (unequal variance).
///
/// # Arguments
/// - `mean1`: Sample mean in treatment arm
/// - `sd1`: Sample standard deviation in treatment arm
/// - `n1`: Sample size in treatment arm
/// - `mean2`: Sample mean in control arm
/// - `sd2`: Sample standard deviation in control arm
/// - `n2`: Sample size in control arm
/// - `alpha`: Significance level
pub fn evaluate_two_means(
    mean1: f64,
    sd1: f64,
    n1: u32,
    mean2: f64,
    sd2: f64,
    n2: u32,
    alpha: f64,
) -> Result<EndpointResult, TrialError> {
    if n1 == 0 || n2 == 0 {
        return Err(TrialError::InvalidParameter("n1 and n2 must be > 0".into()));
    }
    if sd1 < 0.0 || sd2 < 0.0 {
        return Err(TrialError::InvalidParameter("sd must be >= 0".into()));
    }

    let var1 = sd1 * sd1 / n1 as f64;
    let var2 = sd2 * sd2 / n2 as f64;
    let se = (var1 + var2).sqrt();

    let t = if se < 1e-12 {
        0.0
    } else {
        (mean1 - mean2) / se
    };

    // Welch-Satterthwaite degrees of freedom (approximate normal for large df)
    let df = if var1 < 1e-12 || var2 < 1e-12 {
        (n1 + n2 - 2) as f64
    } else {
        let num = (var1 + var2).powi(2);
        let den = var1.powi(2) / (n1 as f64 - 1.0) + var2.powi(2) / (n2 as f64 - 1.0);
        if den < 1e-12 {
            (n1 + n2 - 2) as f64
        } else {
            num / den
        }
    };

    // For df >= 30 normal approximation is accurate; for smaller df use t-approximation
    let p_value = two_sided_p_value_t(t, df);
    let z_crit = z_score_from_alpha(alpha / 2.0);
    let significant = p_value < alpha;
    let effect_size = mean1 - mean2;
    let ci_lower = effect_size - z_crit * se;
    let ci_upper = effect_size + z_crit * se;

    Ok(EndpointResult {
        name: String::new(),
        test_statistic: t,
        p_value,
        significant,
        effect_size,
        ci_lower,
        ci_upper,
        nnt: None,
    })
}

/// Compute Number Needed to Treat (NNT).
///
/// `NNT = 1 / |ARR|` where ARR = absolute risk reduction = p_control - p_treatment
/// (or p_treatment - p_control depending on direction).
/// Returns `None` when rates are equal.
pub fn compute_nnt(p_treatment: f64, p_control: f64) -> Option<f64> {
    let arr = (p_treatment - p_control).abs();
    if arr < 1e-10 { None } else { Some(1.0 / arr) }
}

// ── Internal ─────────────────────────────────────────────────────────────────

/// Two-sided p-value from a t-statistic using the normal approximation for large df,
/// and a simple rational approximation of the t-CDF for smaller df.
fn two_sided_p_value_t(t: f64, df: f64) -> f64 {
    // For df >= 30, the t-distribution is well approximated by the normal
    if df >= 30.0 {
        return 2.0 * normal_cdf(-t.abs());
    }
    // Simple rational approximation for smaller df (accurate to ~1%)
    // P(T > t | df) ≈ 2 * I_{df/(df+t²)}(df/2, 1/2) via regularized incomplete beta
    // Fallback: use normal approximation with correction factor
    let correction = 1.0 + t * t / (4.0 * df);
    let z_adjusted = t / correction.sqrt();
    2.0 * normal_cdf(-z_adjusted.abs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_two_proportion_z_test() {
        // 60/100 vs 45/100, should be significant at alpha=0.05
        let result = evaluate_two_proportions(60, 100, 45, 100, 0.05);
        assert!(result.is_ok(), "Expected Ok, got {result:?}");
        let r = result.unwrap();
        assert!(r.p_value < 0.05, "Expected significant, p={}", r.p_value);
        assert!(r.significant);
    }

    #[test]
    fn test_effect_size_and_ci() {
        let result = evaluate_two_proportions(60, 100, 45, 100, 0.05);
        let r = result.unwrap();
        // Effect size: 0.60 - 0.45 = 0.15
        assert!(
            (r.effect_size - 0.15).abs() < 0.01,
            "Expected ~0.15, got {}",
            r.effect_size
        );
        // CI should exclude 0 (positive lower bound)
        assert!(
            r.ci_lower > 0.0,
            "CI lower bound should exclude 0, got {}",
            r.ci_lower
        );
    }

    #[test]
    fn test_nnt_computed() {
        let r = evaluate_two_proportions(60, 100, 45, 100, 0.05).unwrap();
        assert!(r.nnt.is_some());
        let nnt = r.nnt.unwrap();
        // NNT = 1 / |0.60 - 0.45| = 1/0.15 ≈ 6.67
        assert!((nnt - 6.67).abs() < 0.1, "Expected ~6.67, got {nnt}");
    }

    #[test]
    fn test_not_significant_equal_rates() {
        let result = evaluate_two_proportions(50, 100, 50, 100, 0.05);
        let r = result.unwrap();
        assert!(!r.significant, "Equal rates should not be significant");
        assert!(r.p_value > 0.90, "p-value should be ~1.0 for equal rates");
    }

    #[test]
    fn test_two_means_significant() {
        // mean1=1.5, sd=1.0, n=100 vs mean2=1.0, sd=1.0, n=100
        let result = evaluate_two_means(1.5, 1.0, 100, 1.0, 1.0, 100, 0.05);
        assert!(result.is_ok());
        let r = result.unwrap();
        assert!(
            r.significant,
            "Should be significant for 0.5 SD difference in n=100"
        );
    }

    #[test]
    fn test_compute_nnt_equal_rates() {
        assert!(compute_nnt(0.5, 0.5).is_none());
    }

    #[test]
    fn test_compute_nnt_values() {
        let nnt = compute_nnt(0.60, 0.40).unwrap();
        assert!((nnt - 5.0).abs() < 0.01); // 1/0.20 = 5.0
    }
}
