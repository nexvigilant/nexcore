//! Noise Floor Correction (Nernst-inspired dynamic threshold).
//!
//! Tier: T2-P (maps to Boundary `partial` + Frequency `nu`)
//!
//! Computes the noise correction factor eta that distinguishes organic signal
//! from stimulated reporting (media attention, regulatory action, litigation):
//!
//! ```text
//! eta(d, t) = 1 / (1 + e^(-k * (R_stimulated / R_baseline - 1)))
//! ```
//!
//! Where:
//! - `R_stimulated` = reporting rate during stimulated period
//! - `R_baseline` = organic baseline reporting rate
//! - `k` = sensitivity parameter (default: 5.0)
//!
//! Behavior:
//! - `eta -> 0` when reporting is organic (`R_stimulated ~ R_baseline`)
//! - `eta -> 1` when reporting is heavily stimulated (`R_stimulated >> R_baseline`)
//!
//! Used in the predictive equation as: `Psi * (1 - eta)` to suppress stimulated noise.

use crate::types::NoiseParams;

/// Computes the noise floor correction factor eta.
///
/// Returns a value in [0, 1] where:
/// - 0.0 = purely organic reporting (no noise correction needed)
/// - 1.0 = heavily stimulated reporting (signal is noise)
///
/// When baseline is zero or negative, returns 1.0 (all signal is noise).
#[must_use]
pub fn eta(params: &NoiseParams) -> f64 {
    if params.r_baseline <= 0.0 {
        return 1.0;
    }

    let ratio = params.r_stimulated / params.r_baseline;
    let exponent = -params.k * (ratio - 1.0);
    1.0 / (1.0 + exponent.exp())
}

/// Computes the signal retention factor (1 - eta).
///
/// This is the fraction of signal that survives noise correction.
/// - 1.0 = full signal retained (organic reporting)
/// - 0.0 = no signal retained (all stimulated noise)
#[must_use]
pub fn signal_retention(params: &NoiseParams) -> f64 {
    1.0 - eta(params)
}

/// Determines whether reporting appears predominantly organic.
///
/// Returns true when `eta < 0.5` (more organic than stimulated).
#[must_use]
pub fn is_organic(params: &NoiseParams) -> bool {
    eta(params) < 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eta_organic_reporting() {
        // Stimulated = baseline -> ratio = 1 -> eta = 1/(1+e^0) = 0.5
        let params = NoiseParams::new(50.0, 50.0);
        let result = eta(&params);
        assert!((result - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn eta_heavily_stimulated() {
        // Stimulated = 5x baseline -> high eta
        let params = NoiseParams::new(250.0, 50.0);
        let result = eta(&params);
        // ratio = 5, exponent = -5*(5-1) = -20, eta = 1/(1+e^-20) ~ 1.0
        assert!(result > 0.99);
    }

    #[test]
    fn eta_below_baseline() {
        // Stimulated < baseline -> eta < 0.5
        let params = NoiseParams::new(25.0, 50.0);
        let result = eta(&params);
        // ratio = 0.5, exponent = -5*(0.5-1) = 2.5, eta = 1/(1+e^2.5) ~ 0.076
        assert!(result < 0.5);
        assert!(is_organic(&params));
    }

    #[test]
    fn eta_zero_baseline() {
        let params = NoiseParams::new(100.0, 0.0);
        let result = eta(&params);
        assert!((result - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn eta_negative_baseline() {
        let params = NoiseParams::with_k(100.0, -10.0, 5.0);
        let result = eta(&params);
        assert!((result - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn signal_retention_organic() {
        let params = NoiseParams::new(25.0, 50.0);
        let retention = signal_retention(&params);
        // eta ~ 0.076, retention ~ 0.924
        assert!(retention > 0.9);
    }

    #[test]
    fn signal_retention_stimulated() {
        let params = NoiseParams::new(250.0, 50.0);
        let retention = signal_retention(&params);
        // eta ~ 1.0, retention ~ 0.0
        assert!(retention < 0.01);
    }

    #[test]
    fn is_organic_at_baseline() {
        // At baseline, eta = 0.5, so is_organic returns false (not strictly < 0.5)
        let params = NoiseParams::new(50.0, 50.0);
        assert!(!is_organic(&params));
    }

    #[test]
    fn is_organic_below_baseline() {
        let params = NoiseParams::new(25.0, 50.0);
        assert!(is_organic(&params));
    }

    #[test]
    fn is_organic_above_baseline() {
        let params = NoiseParams::new(100.0, 50.0);
        assert!(!is_organic(&params));
    }

    #[test]
    fn eta_custom_k_sensitivity() {
        // Higher k = steeper transition
        let params_low_k = NoiseParams::with_k(75.0, 50.0, 1.0);
        let params_high_k = NoiseParams::with_k(75.0, 50.0, 10.0);

        let eta_low = eta(&params_low_k);
        let eta_high = eta(&params_high_k);

        // Both should be > 0.5 (above baseline), but high k should be more extreme
        assert!(eta_low > 0.5);
        assert!(eta_high > 0.5);
        assert!(eta_high > eta_low);
    }

    #[test]
    fn signal_retention_complementary() {
        let params = NoiseParams::new(75.0, 50.0);
        let e = eta(&params);
        let r = signal_retention(&params);
        assert!((e + r - 1.0).abs() < f64::EPSILON);
    }
}
