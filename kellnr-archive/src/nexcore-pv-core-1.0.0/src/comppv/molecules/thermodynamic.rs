//! Thermodynamic molecules (Laws 2, 10)

use crate::comppv::atoms::entropy::calculate_total_entropy_change;
use crate::comppv::atoms::thermodynamic::{calculate_gibbs_free_energy, is_spontaneous_binding};
use crate::comppv::specs::CONSERVATION_LAW_SPECS;
use crate::comppv::types::{ConservationLaw, LawValidationResult};
use crate::comppv::validators::{BindingState, EntropyState};

/// Validate Law 2: Thermodynamic Binding.
pub fn validate_thermodynamic_binding(
    state: &BindingState,
    expected_spontaneous: bool,
) -> LawValidationResult {
    let spec_tol = CONSERVATION_LAW_SPECS
        .get(&ConservationLaw::ThermodynamicBinding)
        .map_or(1e-6, |s| s.validation_tolerance);
    let delta_g =
        calculate_gibbs_free_energy(state.association_constant_m_inv, Some(state.temperature_k))
            .unwrap_or(0.0);
    let is_valid = is_spontaneous_binding(delta_g) == expected_spontaneous;
    LawValidationResult {
        law: ConservationLaw::ThermodynamicBinding,
        is_valid,
        deviation: if !is_valid { delta_g.abs() } else { 0.0 },
        relative_deviation: if is_valid { 0.0 } else { 1.0 },
        tolerance: spec_tol,
    }
}

/// Validate Law 10: Entropy Increase.
pub fn validate_entropy_increase(state: &EntropyState) -> LawValidationResult {
    let spec_tol = CONSERVATION_LAW_SPECS
        .get(&ConservationLaw::EntropyIncrease)
        .map_or(1e-6, |s| s.validation_tolerance);
    let total_ds = calculate_total_entropy_change(state.delta_s_system, state.delta_s_surroundings);
    let is_valid = total_ds >= -1e-15;
    LawValidationResult {
        law: ConservationLaw::EntropyIncrease,
        is_valid,
        deviation: if total_ds < 0.0 { total_ds.abs() } else { 0.0 },
        relative_deviation: if total_ds < 0.0 { 1.0 } else { 0.0 },
        tolerance: spec_tol,
    }
}
