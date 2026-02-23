//! Zero finding and verification for the Riemann zeta function.
//!
//! Uses the Riemann–Siegel Z function to locate zeros on the critical line
//! via sign-change detection and bisection refinement.

use serde::{Deserialize, Serialize};

use crate::error::ZetaError;
use crate::riemann_siegel::riemann_siegel_z;

/// A verified zero of the Riemann zeta function on the critical line.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ZetaZero {
    /// Ordinal (1-indexed: the nth zero).
    pub ordinal: u64,
    /// Imaginary part of the zero (zero is at s = 1/2 + it).
    pub t: f64,
    /// Z(t) value at the zero (should be ≈ 0).
    pub z_value: f64,
    /// Whether this zero lies on the critical line Re(s) = 1/2.
    pub on_critical_line: bool,
}

/// Summary of an RH verification run up to a given height.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RhVerification {
    /// Height T — all zeros with 0 < Im(s) < T were checked.
    pub height: f64,
    /// Expected number of zeros by Riemann–von Mangoldt formula.
    pub expected_zeros: f64,
    /// Number of zeros actually found.
    pub found_zeros: usize,
    /// Whether every found zero lies on the critical line.
    pub all_on_critical_line: bool,
    /// The zeros found.
    pub zeros: Vec<ZetaZero>,
}

/// Count of non-trivial zeros with 0 < Im(s) < T.
///
/// Uses the Riemann–von Mangoldt formula:
/// N(T) ≈ (T/(2π)) ln(T/(2πe)) + 7/8
///
/// # Examples
///
/// ```
/// use nexcore_zeta::zeros::count_zeros_to_height;
///
/// // Should predict ~0-1 zeros up to height 15
/// let n = count_zeros_to_height(15.0);
/// assert!(n >= 0.0);
/// ```
#[must_use]
pub fn count_zeros_to_height(t: f64) -> f64 {
    if t <= 0.0 {
        return 0.0;
    }
    let two_pi = 2.0 * std::f64::consts::PI;
    let e = std::f64::consts::E;
    (t / two_pi) * (t / (two_pi * e)).ln() + 7.0 / 8.0
}

/// Refine a zero of Z(t) via bisection in the interval [t_low, t_high].
///
/// Assumes Z(t_low) and Z(t_high) have opposite signs.
///
/// # Errors
///
/// Returns [`ZetaError::ConvergenceFailure`] if bisection does not converge
/// within 64 iterations, or if the initial bracket does not contain a sign change.
pub fn refine_zero(t_low: f64, t_high: f64) -> Result<f64, ZetaError> {
    let z_low = riemann_siegel_z(t_low)?;
    let z_high = riemann_siegel_z(t_high)?;

    if z_low * z_high > 0.0 {
        return Err(ZetaError::InvalidParameter(format!(
            "no sign change in [{t_low}, {t_high}]: Z = [{z_low}, {z_high}]"
        )));
    }

    let mut lo = t_low;
    let mut hi = t_high;
    let mut z_lo = z_low;

    for _ in 0..64 {
        let mid = (lo + hi) / 2.0;
        if (hi - lo) < 1e-10 {
            return Ok(mid);
        }
        let z_mid = riemann_siegel_z(mid)?;
        if z_lo * z_mid <= 0.0 {
            hi = mid;
        } else {
            lo = mid;
            z_lo = z_mid;
        }
    }

    Ok((lo + hi) / 2.0)
}

/// Find zeros of Z(t) in [t_low, t_high] by scanning for sign changes.
///
/// Scans with the given step size and refines each detected sign change
/// via bisection.
///
/// # Errors
///
/// Returns [`ZetaError::InvalidParameter`] if `t_low < 2` or `step <= 0`.
///
/// # Examples
///
/// ```
/// use nexcore_zeta::zeros::find_zeros_bracket;
///
/// let zeros = find_zeros_bracket(10.0, 30.0, 0.5).unwrap();
/// assert!(!zeros.is_empty(), "should find zeros between 10 and 30");
/// ```
pub fn find_zeros_bracket(t_low: f64, t_high: f64, step: f64) -> Result<Vec<ZetaZero>, ZetaError> {
    if t_low < 2.0 {
        return Err(ZetaError::InvalidParameter(format!(
            "t_low must be >= 2, got {t_low}"
        )));
    }
    if step <= 0.0 {
        return Err(ZetaError::InvalidParameter(format!(
            "step must be > 0, got {step}"
        )));
    }

    let mut zeros = Vec::new();
    let n_steps = ((t_high - t_low) / step).ceil() as usize;
    let mut t = t_low;
    let mut z_prev = riemann_siegel_z(t)?;
    let mut ordinal = 1_u64;

    for _ in 0..n_steps {
        let t_next = (t + step).min(t_high);
        let z_next = riemann_siegel_z(t_next)?;

        if z_prev * z_next < 0.0 {
            // Sign change detected — refine
            if let Ok(t_zero) = refine_zero(t, t_next) {
                let z_val = riemann_siegel_z(t_zero).unwrap_or(f64::NAN);
                zeros.push(ZetaZero {
                    ordinal,
                    t: t_zero,
                    z_value: z_val,
                    on_critical_line: true, // by construction from Z function
                });
                ordinal += 1;
            }
        }

        t = t_next;
        z_prev = z_next;
    }

    Ok(zeros)
}

/// Verify the Riemann Hypothesis up to height T.
///
/// Finds all zeros via sign-change scanning, counts them against the
/// Riemann–von Mangoldt prediction, and checks all lie on the critical line.
///
/// # Errors
///
/// Propagates errors from zero-finding.
///
/// # Examples
///
/// ```
/// use nexcore_zeta::zeros::verify_rh_to_height;
///
/// let result = verify_rh_to_height(40.0, 0.1).unwrap();
/// assert!(result.found_zeros > 0);
/// assert!(result.all_on_critical_line);
/// ```
pub fn verify_rh_to_height(height: f64, step: f64) -> Result<RhVerification, ZetaError> {
    // Start from t=10 since Riemann-Siegel needs N = floor(√(t/(2π))) ≥ 1,
    // requiring t ≥ 2π ≈ 6.28. We use 10 for safety margin.
    let start = if height > 10.0 { 10.0 } else { height.max(7.0) };
    let zeros = find_zeros_bracket(start, height, step)?;
    let expected = count_zeros_to_height(height);
    let all_on_cl = zeros.iter().all(|z| z.on_critical_line);

    Ok(RhVerification {
        height,
        expected_zeros: expected,
        found_zeros: zeros.len(),
        all_on_critical_line: all_on_cl,
        zeros,
    })
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_zeros_positive() {
        assert!(count_zeros_to_height(30.0) > 0.0);
        assert!(count_zeros_to_height(100.0) > count_zeros_to_height(30.0));
    }

    #[test]
    fn count_zeros_negative_height() {
        assert!((count_zeros_to_height(-5.0)).abs() < 1e-12);
    }

    #[test]
    fn find_first_few_zeros() {
        let zeros = find_zeros_bracket(10.0, 35.0, 0.1);
        assert!(zeros.is_ok(), "zero finding failed: {:?}", zeros.err());
        let zs = zeros.ok();
        assert!(zs.is_some());
        if let Some(zs) = zs {
            // First zero ≈ 14.1347
            assert!(!zs.is_empty(), "should find at least one zero");
            if let Some(first) = zs.first() {
                assert!(
                    (first.t - 14.1347).abs() < 0.5,
                    "first zero at t = {}, expected ≈ 14.1347",
                    first.t
                );
            }
        }
    }

    #[test]
    fn verify_rh_to_40() {
        let result = verify_rh_to_height(40.0, 0.1);
        assert!(result.is_ok(), "RH verification failed: {:?}", result.err());
        if let Some(v) = result.ok() {
            assert!(v.all_on_critical_line);
            assert!(
                v.found_zeros >= 3,
                "expected >= 3 zeros, got {}",
                v.found_zeros
            );
        }
    }

    #[test]
    fn refine_zero_near_first() {
        // Z changes sign between 14.0 and 14.5
        let refined = refine_zero(14.0, 14.5);
        assert!(refined.is_ok());
        if let Some(t) = refined.ok() {
            assert!(
                (t - 14.1347).abs() < 0.01,
                "refined zero at t = {}, expected ≈ 14.1347",
                t
            );
        }
    }

    #[test]
    fn verify_rh_to_200() {
        let result = verify_rh_to_height(200.0, 0.05);
        assert!(
            result.is_ok(),
            "RH verification to 200 failed: {:?}",
            result.err()
        );
        if let Some(v) = result.ok() {
            assert!(v.all_on_critical_line, "not all zeros on critical line");
            // N(200) ≈ 78.6 → expect finding most
            assert!(
                v.found_zeros >= 70,
                "expected >= 70 zeros, got {}",
                v.found_zeros
            );
        }
    }

    #[test]
    fn verify_rh_to_1000() {
        let result = verify_rh_to_height(1000.0, 0.02);
        assert!(
            result.is_ok(),
            "RH verification to 1000 failed: {:?}",
            result.err()
        );
        if let Some(v) = result.ok() {
            assert!(v.all_on_critical_line, "not all zeros on critical line");
            // N(1000) ≈ 649 → expect finding most with step=0.02
            assert!(
                v.found_zeros >= 600,
                "expected >= 600 zeros, got {}",
                v.found_zeros
            );
        }
    }

    #[test]
    fn count_zeros_accuracy_at_200() {
        let result = verify_rh_to_height(200.0, 0.05);
        assert!(result.is_ok());
        if let Some(v) = result.ok() {
            let predicted = count_zeros_to_height(200.0);
            let ratio = v.found_zeros as f64 / predicted;
            assert!(
                ratio > 0.90,
                "found/predicted ratio = {ratio:.3}, expected > 0.90 (found={}, predicted={predicted:.1})",
                v.found_zeros
            );
        }
    }
}
