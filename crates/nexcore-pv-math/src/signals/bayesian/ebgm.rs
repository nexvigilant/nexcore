//! Empirical Bayes Geometric Mean (EBGM) — DuMouchel MGPS method.
//!
//! Prior: mixture of two Gamma distributions
//!   Prior = p × Γ(α₁, β₁) + (1-p) × Γ(α₂, β₂)
//!
//! Signal criteria: EBGM ≥ 2.0, EB05 ≥ 2.0, n ≥ 3.
//!
//! # References
//!
//! DuMouchel W (1999). Am Stat 53(3):177-190.

use crate::error::PvMathError;
use crate::stats::{log_gamma, normal_quantile};
use crate::types::{SignalCriteria, SignalResult, TwoByTwoTable};
use nexcore_signal_types::SignalMethod;

/// DuMouchel recommended prior parameters for MGPS.
#[derive(Debug, Clone, Copy)]
pub struct MgpsPriors {
    /// Shape parameter for the signal (high-RR) Gamma component.
    pub alpha1: f64,
    /// Rate parameter for the signal Gamma component.
    pub beta1: f64,
    /// Shape parameter for the null (background) Gamma component.
    pub alpha2: f64,
    /// Rate parameter for the null Gamma component.
    pub beta2: f64,
    /// Mixing proportion for the signal component (prior probability of a signal).
    pub p: f64,
}

impl Default for MgpsPriors {
    fn default() -> Self {
        Self {
            alpha1: 0.2,
            beta1: 0.1,
            alpha2: 2.0,
            beta2: 4.0,
            p: 0.1,
        }
    }
}

/// Calculate EBGM with default DuMouchel priors.
///
/// # Errors
///
/// Returns `PvMathError` when the table is invalid.
pub fn calculate_ebgm(
    table: &TwoByTwoTable,
    criteria: &SignalCriteria,
) -> Result<SignalResult, PvMathError> {
    calculate_ebgm_with_priors(table, criteria, &MgpsPriors::default())
}

/// Calculate EBGM with custom prior parameters.
///
/// # Errors
///
/// Returns `PvMathError` when the table is invalid.
pub fn calculate_ebgm_with_priors(
    table: &TwoByTwoTable,
    criteria: &SignalCriteria,
    priors: &MgpsPriors,
) -> Result<SignalResult, PvMathError> {
    if !table.is_valid() {
        return Err(PvMathError::invalid_table("empty contingency table"));
    }

    let observed = table.a as f64;
    let expected = table.expected_count();

    if expected <= 0.0 {
        return Ok(SignalResult {
            method: SignalMethod::Ebgm,
            point_estimate: 0.0,
            lower_ci: 0.0,
            upper_ci: 0.0,
            chi_square: None,
            is_signal: false,
            case_count: table.a,
            total_reports: table.total(),
        });
    }

    let w1 = gamma_poisson_weight(observed, expected, priors.alpha1, priors.beta1, priors.p);
    let w2 = gamma_poisson_weight(
        observed,
        expected,
        priors.alpha2,
        priors.beta2,
        1.0 - priors.p,
    );
    let total_w = w1 + w2;

    let (q1, q2) = if total_w <= 0.0 || (w1 / total_w).is_nan() {
        // Fall back to raw ratio
        let raw = observed / expected;
        return Ok(SignalResult {
            method: SignalMethod::Ebgm,
            point_estimate: raw,
            lower_ci: raw * 0.5,
            upper_ci: raw * 2.0,
            chi_square: None,
            is_signal: false,
            case_count: table.a,
            total_reports: table.total(),
        });
    } else {
        (w1 / total_w, w2 / total_w)
    };

    let mean1 = (observed + priors.alpha1) / (expected + priors.beta1);
    let mean2 = (observed + priors.alpha2) / (expected + priors.beta2);

    let ebgm = if mean1 > 0.0 && mean2 > 0.0 {
        let log_ebgm = q1 * mean1.ln() + q2 * mean2.ln();
        if log_ebgm.is_nan() || log_ebgm > 700.0 {
            observed / expected
        } else if log_ebgm < -700.0 {
            0.0
        } else {
            log_ebgm.exp()
        }
    } else {
        mean1.max(mean2)
    };

    let ebgm = if ebgm.is_nan() || ebgm.is_infinite() {
        observed / expected
    } else {
        ebgm
    };

    let variance = posterior_variance(observed, priors);
    let (eb05, eb95) = if variance > 0.0 && ebgm > 0.0 && ebgm.is_finite() {
        let sd = variance.sqrt();
        let log_e = ebgm.ln();
        if log_e.is_finite() {
            let e05 = normal_quantile(0.05).mul_add(sd, log_e).exp();
            let e95 = normal_quantile(0.95).mul_add(sd, log_e).exp();
            (
                if e05.is_finite() { e05 } else { ebgm * 0.5 },
                if e95.is_finite() { e95 } else { ebgm * 2.0 },
            )
        } else {
            (ebgm * 0.5, ebgm * 2.0)
        }
    } else {
        (ebgm * 0.5, ebgm * 2.0)
    };

    let is_signal = ebgm >= criteria.ebgm_threshold
        && eb05 >= criteria.eb05_threshold
        && table.a >= u64::from(criteria.min_cases);

    Ok(SignalResult {
        method: SignalMethod::Ebgm,
        point_estimate: ebgm,
        lower_ci: eb05,
        upper_ci: eb95,
        chi_square: None,
        is_signal,
        case_count: table.a,
        total_reports: table.total(),
    })
}

/// EBGM point estimate only.
#[must_use]
pub fn ebgm_only(table: &TwoByTwoTable) -> Option<f64> {
    calculate_ebgm(table, &SignalCriteria::evans())
        .ok()
        .map(|r| r.point_estimate)
}

/// EB05 — lower 5th-percentile bound.
#[must_use]
pub fn eb05(table: &TwoByTwoTable) -> Option<f64> {
    calculate_ebgm(table, &SignalCriteria::evans())
        .ok()
        .map(|r| r.lower_ci)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Gamma-Poisson mixture component weight (log-space to avoid overflow).
#[allow(clippy::suboptimal_flops)] // formula clarity for MGPS math
fn gamma_poisson_weight(n: f64, e: f64, alpha: f64, beta: f64, prior_p: f64) -> f64 {
    if prior_p <= 0.0 || alpha <= 0.0 || beta <= 0.0 || n > 1e6 || e > 1e6 {
        return 0.0;
    }
    let log_w = log_gamma(n + alpha) - log_gamma(alpha) + alpha * beta.ln()
        - (n + alpha) * (e + beta).ln()
        + prior_p.ln();
    if log_w.is_nan() || log_w < -700.0 {
        0.0
    } else if log_w > 700.0 {
        f64::MAX
    } else {
        log_w.exp()
    }
}

/// Approximate posterior variance of log(λ).
fn posterior_variance(n: f64, priors: &MgpsPriors) -> f64 {
    let v1 = if n + priors.alpha1 > 0.0 {
        1.0 / (n + priors.alpha1)
    } else {
        1.0
    };
    let v2 = if n + priors.alpha2 > 0.0 {
        1.0 / (n + priors.alpha2)
    } else {
        1.0
    };
    priors.p * v1 + (1.0 - priors.p) * v2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ebgm_positive_for_strong_signal() {
        let table = TwoByTwoTable::new(10, 90, 100, 9800);
        let result = calculate_ebgm(&table, &SignalCriteria::evans()).unwrap();
        assert!(result.point_estimate > 0.0);
        // Shrinkage: EBGM ≤ raw ratio + 1
        let raw = 10.0 / table.expected_count();
        assert!(result.point_estimate <= raw + 1.0);
    }

    #[test]
    fn ebgm_shrinkage_on_small_sample() {
        let table = TwoByTwoTable::new(3, 97, 300, 9600);
        let result = calculate_ebgm(&table, &SignalCriteria::evans()).unwrap();
        let raw = 3.0 / table.expected_count();
        // Conservative priors pull EBGM below raw ratio
        assert!(result.point_estimate < raw);
    }

    #[test]
    fn ebgm_zero_cases_not_signal() {
        let table = TwoByTwoTable::new(0, 100, 100, 9800);
        let result = calculate_ebgm(&table, &SignalCriteria::evans()).unwrap();
        assert!(!result.is_signal);
    }
}
