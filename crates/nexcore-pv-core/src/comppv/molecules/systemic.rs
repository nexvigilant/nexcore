//! Systemic molecules (Laws 4, 6, 8, 11)

use crate::comppv::atoms::compartment::calculate_compartment_rate;
use crate::comppv::atoms::flux::calculate_node_net_flux;
use crate::comppv::atoms::genetic::calculate_sequence_homology;
use crate::comppv::atoms::ionization::calculate_fraction_unionized;
use crate::comppv::specs::CONSERVATION_LAW_SPECS;
use crate::comppv::types::{ConservationLaw, LawValidationResult};
use crate::comppv::validators::{
    AdmeRateState, GeneticState, IonizationStateData, PathwayFluxState,
};

/// Validate Law 4: Pathway Flux Conservation.
pub fn validate_pathway_flux(
    state: &PathwayFluxState,
    tolerance: Option<f64>,
) -> LawValidationResult {
    let spec_tol = CONSERVATION_LAW_SPECS
        .get(&ConservationLaw::PathwayFlux)
        .map_or(1e-6, |s| s.validation_tolerance);
    let tol = tolerance.unwrap_or(spec_tol);
    let net_flux = calculate_node_net_flux(&state.fluxes_in, &state.fluxes_out);
    let sum_in: f64 = state.fluxes_in.iter().sum();
    let rel_dev = if sum_in > 0.0 {
        net_flux.abs() / sum_in
    } else {
        net_flux.abs()
    };
    LawValidationResult {
        law: ConservationLaw::PathwayFlux,
        is_valid: rel_dev <= tol,
        deviation: net_flux.abs(),
        relative_deviation: rel_dev,
        tolerance: tol,
    }
}

/// Validate Law 6: ADME Rate Conservation.
pub fn validate_adme_rate(state: &AdmeRateState, tolerance: Option<f64>) -> LawValidationResult {
    let spec_tol = CONSERVATION_LAW_SPECS
        .get(&ConservationLaw::AdmeRate)
        .map_or(1e-6, |s| s.validation_tolerance);
    let tol = tolerance.unwrap_or(spec_tol);
    let calc_rate = calculate_compartment_rate(&state.rates_in, &state.rates_out);
    let deviation = (calc_rate - state.measured_rate_of_change).abs();
    let sum_in: f64 = state.rates_in.iter().sum();
    let rel_dev = if sum_in > 0.0 {
        deviation / sum_in
    } else {
        deviation
    };
    LawValidationResult {
        law: ConservationLaw::AdmeRate,
        is_valid: rel_dev <= tol,
        deviation,
        relative_deviation: rel_dev,
        tolerance: tol,
    }
}

/// Validate Law 8: Ionization State.
pub fn validate_ionization_state(
    state: &IonizationStateData,
    tolerance: Option<f64>,
) -> LawValidationResult {
    let spec_tol = CONSERVATION_LAW_SPECS
        .get(&ConservationLaw::IonizationState)
        .map_or(1e-6, |s| s.validation_tolerance);
    let tol = tolerance.unwrap_or(spec_tol);
    let calc_fu = calculate_fraction_unionized(state.pka, state.ph, state.is_acid);
    let deviation = (calc_fu - state.measured_fraction_unionized).abs();
    LawValidationResult {
        law: ConservationLaw::IonizationState,
        is_valid: deviation <= tol,
        deviation,
        relative_deviation: deviation,
        tolerance: tol,
    }
}

/// Validate Law 11: Genetic Conservation.
pub fn validate_genetic_conservation(state: &GeneticState) -> LawValidationResult {
    let spec_tol = CONSERVATION_LAW_SPECS
        .get(&ConservationLaw::GeneticConservation)
        .map_or(1e-6, |s| s.validation_tolerance);
    let homology = calculate_sequence_homology(&state.sequence_before, &state.sequence_after);
    let is_valid = homology >= 1.0;
    let deviation = 1.0 - homology;
    LawValidationResult {
        law: ConservationLaw::GeneticConservation,
        is_valid,
        deviation,
        relative_deviation: deviation,
        tolerance: spec_tol,
    }
}
