//! Signal Emergence Feasibility (Gibbs free energy analogy).
//!
//! Tier: T2-P (maps to Causality `->` + Quantity `N`)
//!
//! Models whether a pharmacovigilance signal is "thermodynamically favorable"
//! to emerge, using the Gibbs free energy equation as an analogy:
//!
//! ```text
//! DeltaG(d, e) = DeltaH_mechanism - T_exposure * DeltaS_information
//! ```
//!
//! Where:
//! - `DeltaH_mechanism` = pharmacological harm enthalpy (mechanism predicts AE)
//! - `T_exposure` = market temperature in patient-years
//! - `DeltaS_information` = information entropy of evidence base
//!
//! When `DeltaG < 0`, signal emergence is thermodynamically favorable.

use crate::types::GibbsParams;

/// Computes the signal emergence feasibility (DeltaG).
///
/// Analogous to Gibbs free energy: `DeltaG = DeltaH - T * DeltaS`
///
/// Returns a negative value when signal emergence is favorable
/// (strong mechanism, high exposure, low entropy).
///
/// # Arguments
///
/// * `params` - The Gibbs parameters containing DeltaH, T, and DeltaS.
///
/// # Returns
///
/// The DeltaG value. Negative indicates favorable signal emergence.
#[must_use]
pub fn delta_g(params: &GibbsParams) -> f64 {
    params.delta_h_mechanism - params.t_exposure * params.delta_s_information
}

/// Returns true when signal emergence is thermodynamically favorable (DeltaG < 0).
#[must_use]
pub fn is_favorable(params: &GibbsParams) -> bool {
    delta_g(params) < 0.0
}

/// Normalized DeltaG mapped to [0, 1] feasibility score.
///
/// Uses a sigmoid transformation: `feasibility = 1 / (1 + e^(DeltaG))`
///
/// - DeltaG << 0 -> feasibility ~ 1.0 (highly favorable)
/// - DeltaG = 0 -> feasibility = 0.5 (neutral)
/// - DeltaG >> 0 -> feasibility ~ 0.0 (unfavorable)
#[must_use]
pub fn feasibility_score(params: &GibbsParams) -> f64 {
    let dg = delta_g(params);
    1.0 / (1.0 + dg.exp())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn favorable_signal_emergence() {
        // High exposure and mechanism, low entropy -> DeltaG < 0
        let params = GibbsParams::new(3.0, 10000.0, 0.001);
        let dg = delta_g(&params);
        // DeltaG = 3.0 - 10000 * 0.001 = 3.0 - 10.0 = -7.0
        assert!((dg - (-7.0)).abs() < f64::EPSILON);
        assert!(is_favorable(&params));
    }

    #[test]
    fn unfavorable_signal_emergence() {
        // Low exposure, high mechanism -> DeltaG > 0
        let params = GibbsParams::new(10.0, 10.0, 0.001);
        let dg = delta_g(&params);
        // DeltaG = 10.0 - 10 * 0.001 = 10.0 - 0.01 = 9.99
        assert!((dg - 9.99).abs() < f64::EPSILON);
        assert!(!is_favorable(&params));
    }

    #[test]
    fn neutral_equilibrium() {
        // DeltaH exactly equals T * DeltaS
        let params = GibbsParams::new(5.0, 1000.0, 0.005);
        let dg = delta_g(&params);
        // DeltaG = 5.0 - 1000 * 0.005 = 5.0 - 5.0 = 0.0
        assert!(dg.abs() < f64::EPSILON);
        assert!(!is_favorable(&params)); // DeltaG = 0 is not < 0
    }

    #[test]
    fn feasibility_score_favorable() {
        let params = GibbsParams::new(3.0, 10000.0, 0.001);
        let score = feasibility_score(&params);
        // DeltaG = -7.0, score = 1/(1 + e^-7) ~ 0.999
        assert!(score > 0.99);
    }

    #[test]
    fn feasibility_score_unfavorable() {
        let params = GibbsParams::new(10.0, 10.0, 0.001);
        let score = feasibility_score(&params);
        // DeltaG = 9.99, score = 1/(1 + e^9.99) ~ 0.00005
        assert!(score < 0.01);
    }

    #[test]
    fn feasibility_score_neutral() {
        let params = GibbsParams::new(5.0, 1000.0, 0.005);
        let score = feasibility_score(&params);
        // DeltaG = 0, score = 1/(1 + e^0) = 0.5
        assert!((score - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn zero_exposure_temperature() {
        let params = GibbsParams::new(5.0, 0.0, 0.01);
        let dg = delta_g(&params);
        // DeltaG = 5.0 - 0 = 5.0 (no exposure -> mechanism alone)
        assert!((dg - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn zero_mechanism_enthalpy() {
        let params = GibbsParams::new(0.0, 1000.0, 0.01);
        let dg = delta_g(&params);
        // DeltaG = 0 - 10 = -10 (no mechanism but high exposure/entropy)
        assert!((dg - (-10.0)).abs() < f64::EPSILON);
        assert!(is_favorable(&params));
    }
}
