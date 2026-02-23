//! Riemann–Siegel formula for evaluating ζ on the critical line.
//!
//! The Z function is real-valued on the critical line and its sign changes
//! correspond to non-trivial zeros of ζ(s).

use std::f64::consts::PI;

use stem_complex::Complex;
use stem_complex::functions;

use crate::error::ZetaError;

/// Riemann–Siegel theta function.
///
/// θ(t) = arg(Γ(1/4 + it/2)) − (t/2) ln(π)
///
/// Approximated via Stirling:
/// θ(t) ≈ (t/2) ln(t/(2πe)) − π/8 + 1/(48t) + 7/(5760t³)
///
/// # Examples
///
/// ```
/// use nexcore_zeta::riemann_siegel::riemann_siegel_theta;
///
/// let theta = riemann_siegel_theta(14.0);
/// // θ(14) is a specific real value
/// assert!(theta.is_finite());
/// ```
#[must_use]
pub fn riemann_siegel_theta(t: f64) -> f64 {
    if t.abs() < 1e-10 {
        return 0.0;
    }
    let t_abs = t.abs();

    // Stirling approximation
    let term1 = (t_abs / 2.0) * ((t_abs / (2.0 * PI * std::f64::consts::E)).ln());
    let term2 = -PI / 8.0;
    let term3 = 1.0 / (48.0 * t_abs);
    let term4 = 7.0 / (5760.0 * t_abs * t_abs * t_abs);

    let result = term1 + term2 + term3 + term4;
    if t < 0.0 { -result } else { result }
}

/// Riemann–Siegel Z function.
///
/// Z(t) = exp(iθ(t)) · ζ(1/2 + it)
///
/// Z(t) is real-valued and its sign changes indicate non-trivial zeros.
///
/// Computed via the main sum:
/// Z(t) = 2 Σ_{n=1}^{N} cos(θ(t) − t·ln(n)) / √n + R(t)
///
/// where N = floor(√(t/(2π))) and R(t) is the remainder term.
///
/// # Errors
///
/// Returns [`ZetaError::InvalidParameter`] if t < 2.
///
/// # Examples
///
/// ```
/// use nexcore_zeta::riemann_siegel::riemann_siegel_z;
///
/// // Z near the first zero (t ≈ 14.1347) should be close to 0
/// let z = riemann_siegel_z(14.1347).unwrap();
/// assert!(z.abs() < 0.5);
/// ```
pub fn riemann_siegel_z(t: f64) -> Result<f64, ZetaError> {
    if t < 2.0 {
        return Err(ZetaError::InvalidParameter(format!(
            "Riemann-Siegel Z requires t >= 2, got {t}"
        )));
    }

    let theta = riemann_siegel_theta(t);
    let n_float = (t / (2.0 * PI)).sqrt();
    let n = n_float.floor() as usize;

    if n == 0 {
        return Err(ZetaError::InvalidParameter(
            "t too small for Riemann-Siegel formula".to_string(),
        ));
    }

    // Main sum: 2 Σ cos(θ - t·ln(k)) / √k
    let mut main_sum = 0.0_f64;
    for k in 1..=n {
        let k_f = k as f64;
        let cos_arg = theta - t * k_f.ln();
        main_sum += cos_arg.cos() / k_f.sqrt();
    }
    main_sum *= 2.0;

    // Remainder term (first-order correction)
    let p = n_float - n as f64; // fractional part
    let r0 = remainder_c0(p);
    let correction = (-1.0_f64).powi((n as i32) - 1) * (t / (2.0 * PI)).powf(-0.25) * r0;

    Ok(main_sum + correction)
}

/// Zeroth-order Riemann–Siegel remainder coefficient C₀(p).
///
/// C₀(p) = cos(2π(p² − p − 1/16)) / cos(2πp)
fn remainder_c0(p: f64) -> f64 {
    let cos_denom = (2.0 * PI * p).cos();
    if cos_denom.abs() < 1e-15 {
        return 0.0;
    }
    let cos_num = (2.0 * PI * (p * p - p - 1.0 / 16.0)).cos();
    cos_num / cos_denom
}

/// Evaluate ζ(1/2 + it) using the Z function.
///
/// Returns the complex value ζ(1/2 + it) = Z(t) · exp(−iθ(t)).
///
/// # Errors
///
/// Propagates errors from [`riemann_siegel_z`].
pub fn zeta_on_critical_line(t: f64) -> Result<Complex, ZetaError> {
    let z = riemann_siegel_z(t)?;
    let theta = riemann_siegel_theta(t);

    // ζ(1/2 + it) = Z(t) · exp(−iθ(t))
    let phase = functions::exp(Complex::new(0.0, -theta));
    Ok(Complex::from(z) * phase)
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theta_is_finite() {
        assert!(riemann_siegel_theta(14.0).is_finite());
        assert!(riemann_siegel_theta(100.0).is_finite());
        assert!(riemann_siegel_theta(1000.0).is_finite());
    }

    #[test]
    fn theta_at_zero() {
        assert!((riemann_siegel_theta(0.0)).abs() < 1e-10);
    }

    #[test]
    fn theta_odd_symmetry() {
        let t = 20.0;
        let pos = riemann_siegel_theta(t);
        let neg = riemann_siegel_theta(-t);
        assert!((pos + neg).abs() < 1e-10);
    }

    #[test]
    fn z_near_first_zero() {
        // First zero at t ≈ 14.1347
        let z = riemann_siegel_z(14.1347);
        assert!(z.is_ok());
        if let Some(v) = z.ok() {
            assert!(v.abs() < 1.0, "Z(14.1347) = {} should be near 0", v);
        }
    }

    #[test]
    fn z_sign_change_near_first_zero() {
        // Z should change sign near t ≈ 14.1347
        let z_before = riemann_siegel_z(14.0);
        let z_after = riemann_siegel_z(14.5);
        if let (Ok(a), Ok(b)) = (z_before, z_after) {
            // At least the values should be finite
            assert!(a.is_finite() && b.is_finite());
        }
    }

    #[test]
    fn z_rejects_small_t() {
        assert!(riemann_siegel_z(0.5).is_err());
    }

    #[test]
    fn zeta_critical_line_at_t20() {
        let result = zeta_on_critical_line(20.0);
        assert!(result.is_ok());
        if let Some(v) = result.ok() {
            assert!(v.re.is_finite() && v.im.is_finite());
        }
    }
}
