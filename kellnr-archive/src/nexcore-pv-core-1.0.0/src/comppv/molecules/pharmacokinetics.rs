//! Pharmacokinetics molecules (Laws 1, 7)

use crate::comppv::atoms::pk::calculate_steady_state_concentration;
use crate::comppv::specs::CONSERVATION_LAW_SPECS;
use crate::comppv::types::{ConservationLaw, LawValidationResult};
use crate::comppv::validators::{MassBalanceState, SteadyStateState};

/// Validate Law 1: Drug Mass Balance.
pub fn validate_mass_balance(
    state: &MassBalanceState,
    tolerance: Option<f64>,
) -> LawValidationResult {
    let spec_tol = CONSERVATION_LAW_SPECS
        .get(&ConservationLaw::DrugMassBalance)
        .map_or(1e-6, |s| s.validation_tolerance);
    let tol = tolerance.unwrap_or(spec_tol);
    let expected = state.initial_dose - state.cumulative_eliminated;
    let deviation = (expected - state.current_amount_in_body).abs();
    let rel_dev = if state.initial_dose > 0.0 {
        deviation / state.initial_dose
    } else {
        deviation
    };
    LawValidationResult {
        law: ConservationLaw::DrugMassBalance,
        is_valid: rel_dev <= tol,
        deviation,
        relative_deviation: rel_dev,
        tolerance: tol,
    }
}

/// Validate Law 7: Steady-State Equilibrium.
pub fn validate_steady_state(
    state: &SteadyStateState,
    tolerance: Option<f64>,
) -> LawValidationResult {
    let spec_tol = CONSERVATION_LAW_SPECS
        .get(&ConservationLaw::SteadyState)
        .map_or(1e-6, |s| s.validation_tolerance);
    let tol = tolerance.unwrap_or(spec_tol);
    let calc_css = calculate_steady_state_concentration(
        state.bioavailability,
        state.dose,
        state.clearance_l_h,
        state.dosing_interval_h,
    )
    .unwrap_or(0.0);
    let deviation = (calc_css - state.measured_concentration).abs();
    let rel_dev = if calc_css > 0.0 {
        deviation / calc_css
    } else {
        deviation
    };
    LawValidationResult {
        law: ConservationLaw::SteadyState,
        is_valid: rel_dev <= tol,
        deviation,
        relative_deviation: rel_dev,
        tolerance: tol,
    }
}
