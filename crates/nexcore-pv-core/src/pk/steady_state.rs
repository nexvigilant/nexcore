//! # Steady-State Equilibrium (Conservation Law 7)
//!
//! Css = (F*Dose)/(CL*tau)
//! Time to steady state ~ 4-5 half-lives
//!
//! At steady state, drug input rate equals elimination rate;
//! concentrations become constant.
//!
//! Safety Implication: Delayed toxicity may manifest only after steady state
//! is reached; loading doses may cause acute toxicity.
//!
//! ## Horus Pattern Applied
//!
//! `Bioavailability`, `Dose`, `Clearance`, `DosingInterval`, `HalfLife`,
//! and `AccumulationFactor` newtypes guarantee valid parameters.
//! All four functions are now infallible.

use super::types::{
    AccumulationFactor, Bioavailability, Clearance, Concentration, Dose, DosingInterval, HalfLife,
};

/// Standard half-lives to reach ~90% of steady state.
pub const HALF_LIVES_TO_STEADY_STATE: f64 = 4.5;

/// Calculate average steady-state concentration.
///
/// Css = (F*Dose)/(CL*tau)
///
/// # Arguments
///
/// * `bioavailability` - Fraction absorbed (F, 0.0-1.0)
/// * `dose` - Dose per interval (validated non-negative)
/// * `clearance` - Systemic clearance (validated positive)
/// * `dosing_interval` - Dosing interval tau (validated positive)
///
/// # Returns
///
/// Average steady-state concentration. Infallible.
#[must_use]
pub fn calculate_steady_state_concentration(
    bioavailability: Bioavailability,
    dose: Dose,
    clearance: Clearance,
    dosing_interval: DosingInterval,
) -> Concentration {
    let css =
        (bioavailability.value() * dose.value()) / (clearance.value() * dosing_interval.value());
    // css >= 0.0 always (non-neg * non-neg / (pos * pos))
    Concentration(css)
}

/// Calculate time to reach ~90% of steady state.
///
/// Time ~ 4.5 * t_half
///
/// # Arguments
///
/// * `half_life` - Elimination half-life (validated positive)
///
/// # Returns
///
/// Time to steady state (hours). Infallible.
#[must_use]
pub fn calculate_time_to_steady_state(half_life: HalfLife) -> f64 {
    HALF_LIVES_TO_STEADY_STATE * half_life.value()
}

/// Calculate accumulation factor for repeated dosing.
///
/// R = 1 / (1 - e^(-0.693*tau/t_half))
///
/// # Arguments
///
/// * `half_life` - Elimination half-life (validated positive)
/// * `dosing_interval` - Dosing interval tau (validated positive)
///
/// # Returns
///
/// Accumulation factor (Css/C_single_dose ratio). Infallible.
#[must_use]
pub fn calculate_accumulation_factor(
    half_life: HalfLife,
    dosing_interval: DosingInterval,
) -> AccumulationFactor {
    let ke = 0.693 / half_life.value(); // Elimination rate constant
    let fraction_remaining = (-ke * dosing_interval.value()).exp();

    if fraction_remaining >= 1.0 {
        // Drug accumulates indefinitely — saturate at a large factor
        // This shouldn't happen with valid positive half-life and interval,
        // but guard against floating-point edge cases.
        AccumulationFactor(f64::MAX)
    } else {
        let factor = 1.0 / (1.0 - fraction_remaining);
        // factor >= 1.0 always when 0 < fraction_remaining < 1
        AccumulationFactor(factor)
    }
}

/// Calculate loading dose to achieve steady-state immediately.
///
/// Loading Dose = Maintenance Dose * Accumulation Factor
///
/// # Arguments
///
/// * `maintenance_dose` - Regular maintenance dose (validated non-negative)
/// * `accumulation_factor` - Css/C_single_dose ratio (validated >= 1.0)
///
/// # Returns
///
/// Loading dose. Infallible.
///
/// Note: Loading doses may cause acute toxicity (Safety Implication)
#[must_use]
pub fn calculate_loading_dose(
    maintenance_dose: Dose,
    accumulation_factor: AccumulationFactor,
) -> Dose {
    let loading = maintenance_dose.value() * accumulation_factor.value();
    // loading >= 0.0 (non-neg * >= 1.0)
    Dose(loading)
}

#[cfg(test)]
mod tests {
    use super::super::types::PkError;
    use super::*;

    #[test]
    fn test_steady_state_concentration() {
        let bio = Bioavailability::new(1.0);
        let dose = Dose::new(500.0);
        let cl = Clearance::new(5.0);
        let tau = DosingInterval::new(12.0);
        if let (Ok(b), Ok(d), Ok(c), Ok(t)) = (bio, dose, cl, tau) {
            let css = calculate_steady_state_concentration(b, d, c, t);
            // Css = (1.0 * 500) / (5.0 * 12.0) = 500 / 60 = 8.333
            assert!((css.value() - 8.333).abs() < 0.01);
        }
    }

    #[test]
    fn test_steady_state_with_bioavailability() {
        let bio = Bioavailability::new(0.5);
        let dose = Dose::new(500.0);
        let cl = Clearance::new(5.0);
        let tau = DosingInterval::new(12.0);
        if let (Ok(b), Ok(d), Ok(c), Ok(t)) = (bio, dose, cl, tau) {
            let css = calculate_steady_state_concentration(b, d, c, t);
            // Css = (0.5 * 500) / (5.0 * 12.0) = 250 / 60 = 4.167
            assert!((css.value() - 4.167).abs() < 0.01);
        }
    }

    #[test]
    fn test_bioavailability_invalid_caught_at_boundary() {
        let result = Bioavailability::new(1.5);
        assert!(matches!(result, Err(PkError::OutOfRange { .. })));
    }

    #[test]
    fn test_time_to_steady_state() {
        let hl = HalfLife::new(10.0);
        if let Ok(h) = hl {
            let t_ss = calculate_time_to_steady_state(h);
            assert!((t_ss - 45.0).abs() < 0.01); // 4.5 * 10 = 45
        }
    }

    #[test]
    fn test_half_life_zero_caught_at_boundary() {
        let result = HalfLife::new(0.0);
        assert!(matches!(result, Err(PkError::NotPositive { .. })));
    }

    #[test]
    fn test_accumulation_factor() {
        let hl = HalfLife::new(12.0);
        let tau = DosingInterval::new(12.0);
        if let (Ok(h), Ok(t)) = (hl, tau) {
            let r = calculate_accumulation_factor(h, t);
            // ke = 0.693/12, fraction = exp(-0.693) = 0.5, R = 1/(1-0.5) = 2.0
            assert!((r.value() - 2.0).abs() < 0.01);
        }
    }

    #[test]
    fn test_accumulation_factor_frequent_dosing() {
        let hl = HalfLife::new(24.0);
        let tau = DosingInterval::new(6.0);
        if let (Ok(h), Ok(t)) = (hl, tau) {
            let r = calculate_accumulation_factor(h, t);
            // More frequent dosing → more accumulation
            assert!(r.value() > 5.0 && r.value() < 7.0);
        }
    }

    #[test]
    fn test_loading_dose() {
        let dose = Dose::new(100.0);
        let af = AccumulationFactor::new(2.0);
        if let (Ok(d), Ok(a)) = (dose, af) {
            let loading = calculate_loading_dose(d, a);
            assert!((loading.value() - 200.0).abs() < 0.01);
        }
    }

    #[test]
    fn test_accumulation_factor_invalid_caught_at_boundary() {
        let result = AccumulationFactor::new(0.5);
        assert!(matches!(result, Err(PkError::OutOfRange { .. })));
    }
}
