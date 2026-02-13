//! Biochemical molecules (Laws 3, 5, 9)

use crate::comppv::atoms::enzyme::calculate_enzyme_rate_of_change;
use crate::comppv::atoms::saturation::calculate_saturation_fraction;
use crate::comppv::specs::CONSERVATION_LAW_SPECS;
use crate::comppv::types::{ConservationLaw, LawValidationResult};
use crate::comppv::validators::{EnzymeState, ReceptorState, SaturationState};

/// Validate Law 3: Receptor State Conservation.
pub fn validate_receptor_state(
    state: &ReceptorState,
    tolerance: Option<f64>,
) -> LawValidationResult {
    let spec_tol = CONSERVATION_LAW_SPECS
        .get(&ConservationLaw::ReceptorState)
        .map_or(1e-6, |s| s.validation_tolerance);
    let tol = tolerance.unwrap_or(spec_tol);
    let current_total = state.current_free + state.current_bound + state.current_desensitized;
    let deviation = (state.total_constant - current_total).abs();
    let rel_dev = if state.total_constant > 0.0 {
        deviation / state.total_constant
    } else {
        deviation
    };
    LawValidationResult {
        law: ConservationLaw::ReceptorState,
        is_valid: rel_dev <= tol,
        deviation,
        relative_deviation: rel_dev,
        tolerance: tol,
    }
}

/// Validate Law 5: Enzyme Regeneration.
pub fn validate_enzyme_regeneration(
    state: &EnzymeState,
    tolerance: Option<f64>,
) -> LawValidationResult {
    let spec_tol = CONSERVATION_LAW_SPECS
        .get(&ConservationLaw::EnzymeRegeneration)
        .map_or(1e-6, |s| s.validation_tolerance);
    let tol = tolerance.unwrap_or(spec_tol);
    let calc_rate = calculate_enzyme_rate_of_change(
        state.k_syn,
        state.k_deg,
        state.k_inact,
        state.inhibitor_conc,
        state.total_enzyme,
    );
    let deviation = (calc_rate - state.measured_rate_of_change).abs();
    let rel_dev = if state.k_syn != 0.0 {
        deviation / state.k_syn
    } else {
        deviation
    };
    LawValidationResult {
        law: ConservationLaw::EnzymeRegeneration,
        is_valid: rel_dev <= tol,
        deviation,
        relative_deviation: rel_dev,
        tolerance: tol,
    }
}

/// Validate Law 9: Saturation Kinetics.
pub fn validate_saturation_kinetics(
    state: &SaturationState,
    tolerance: Option<f64>,
) -> LawValidationResult {
    let spec_tol = CONSERVATION_LAW_SPECS
        .get(&ConservationLaw::SaturationKinetics)
        .map_or(1e-6, |s| s.validation_tolerance);
    let tol = tolerance.unwrap_or(spec_tol);
    let calc_frac =
        calculate_saturation_fraction(state.concentration, state.half_saturation).unwrap_or(0.0);
    let deviation = (calc_frac - state.measured_fraction).abs();
    LawValidationResult {
        law: ConservationLaw::SaturationKinetics,
        is_valid: deviation <= tol,
        deviation,
        relative_deviation: deviation,
        tolerance: tol,
    }
}
