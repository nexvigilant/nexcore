//! Conservation Law Validators
//!
//! State structs and high-level composition for all 11 Conservation Laws.

use crate::comppv::types::ConservationValidationReport;
use serde::{Deserialize, Serialize};

// =============================================================================
// State Structs for all 11 Laws
// =============================================================================

/// State for Law 1: Mass Balance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MassBalanceState {
    /// Initial drug dose administered
    pub initial_dose: f64,
    /// Current amount in body (sum of all compartments)
    pub current_amount_in_body: f64,
    /// Total amount eliminated (metabolized + excreted)
    pub cumulative_eliminated: f64,
}

/// State for Law 2: Thermodynamic Binding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BindingState {
    /// Association constant Ka in M⁻¹
    pub association_constant_m_inv: f64,
    /// Temperature in Kelvin
    pub temperature_k: f64,
}

/// State for Law 3: Receptor Conservation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceptorState {
    /// Total receptor population (constant)
    pub total_constant: f64,
    /// Current free receptor concentration
    pub current_free: f64,
    /// Current bound receptor concentration
    pub current_bound: f64,
    /// Current desensitized receptor concentration
    pub current_desensitized: f64,
}

/// State for Law 4: Pathway Flux
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathwayFluxState {
    /// All incoming fluxes to the node
    pub fluxes_in: Vec<f64>,
    /// All outgoing fluxes from the node
    pub fluxes_out: Vec<f64>,
}

/// State for Law 5: Enzyme Regeneration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnzymeState {
    /// Enzyme synthesis rate constant
    pub k_syn: f64,
    /// Enzyme degradation rate constant
    pub k_deg: f64,
    /// Enzyme inactivation rate constant
    pub k_inact: f64,
    /// Inhibitor concentration
    pub inhibitor_conc: f64,
    /// Total enzyme concentration
    pub total_enzyme: f64,
    /// Measured rate of change for validation
    pub measured_rate_of_change: f64,
}

/// State for Law 6: ADME Rate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdmeRateState {
    /// All input rates to compartment
    pub rates_in: Vec<f64>,
    /// All output rates from compartment
    pub rates_out: Vec<f64>,
    /// Measured rate of change for validation
    pub measured_rate_of_change: f64,
}

/// State for Law 7: Steady-State PK
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteadyStateState {
    /// Bioavailability fraction (0-1)
    pub bioavailability: f64,
    /// Drug dose per interval
    pub dose: f64,
    /// Clearance in L/h
    pub clearance_l_h: f64,
    /// Dosing interval in hours
    pub dosing_interval_h: f64,
    /// Measured concentration for validation
    pub measured_concentration: f64,
}

/// State for Law 8: Ionization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IonizationStateData {
    /// Drug pKa value
    pub pka: f64,
    /// Environmental pH
    pub ph: f64,
    /// True if drug is acidic
    pub is_acid: bool,
    /// Measured unionized fraction for validation
    pub measured_fraction_unionized: f64,
}

/// State for Law 9: Saturation Kinetics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaturationState {
    /// Substrate concentration
    pub concentration: f64,
    /// Michaelis constant (Km)
    pub half_saturation: f64,
    /// Measured saturation fraction for validation
    pub measured_fraction: f64,
}

/// State for Law 10: Entropy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyState {
    /// Entropy change of the system
    pub delta_s_system: f64,
    /// Entropy change of surroundings
    pub delta_s_surroundings: f64,
}

/// State for Law 11: Genetic Conservation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneticState {
    /// Sequence before drug exposure
    pub sequence_before: String,
    /// Sequence after drug exposure
    pub sequence_after: String,
}

/// Aggregate state for validating all 11 Conservation Laws
#[derive(Default, Serialize, Deserialize)]
pub struct FullSystemState {
    /// Law 1: Mass Balance state
    pub mass_balance: Option<MassBalanceState>,
    /// Law 2: Thermodynamic Binding state
    pub binding: Option<BindingState>,
    /// Law 3: Receptor Conservation state
    pub receptor: Option<ReceptorState>,
    /// Law 4: Pathway Flux state
    pub pathway: Option<PathwayFluxState>,
    /// Law 5: Enzyme Regeneration state
    pub enzyme: Option<EnzymeState>,
    /// Law 6: ADME Rate state
    pub adme: Option<AdmeRateState>,
    /// Law 7: Steady-State PK state
    pub steady_state: Option<SteadyStateState>,
    /// Law 8: Ionization state
    pub ionization: Option<IonizationStateData>,
    /// Law 9: Saturation Kinetics state
    pub saturation: Option<SaturationState>,
    /// Law 10: Entropy state
    pub entropy: Option<EntropyState>,
    /// Law 11: Genetic Conservation state
    pub genetic: Option<GeneticState>,
}

/// Validate all provided states against their corresponding Conservation Laws.
pub fn validate_full_system(state: &FullSystemState) -> ConservationValidationReport {
    use super::molecules::*;
    let mut validations = Vec::new();
    let mut failed_laws = Vec::new();

    if let Some(s) = &state.mass_balance {
        validations.push(pharmacokinetics::validate_mass_balance(s, None));
    }
    if let Some(s) = &state.binding {
        validations.push(thermodynamic::validate_thermodynamic_binding(s, true));
    }
    if let Some(s) = &state.receptor {
        validations.push(biochemical::validate_receptor_state(s, None));
    }
    if let Some(s) = &state.pathway {
        validations.push(systemic::validate_pathway_flux(s, None));
    }
    if let Some(s) = &state.enzyme {
        validations.push(biochemical::validate_enzyme_regeneration(s, None));
    }
    if let Some(s) = &state.adme {
        validations.push(systemic::validate_adme_rate(s, None));
    }
    if let Some(s) = &state.steady_state {
        validations.push(pharmacokinetics::validate_steady_state(s, None));
    }
    if let Some(s) = &state.ionization {
        validations.push(systemic::validate_ionization_state(s, None));
    }
    if let Some(s) = &state.saturation {
        validations.push(biochemical::validate_saturation_kinetics(s, None));
    }
    if let Some(s) = &state.entropy {
        validations.push(thermodynamic::validate_entropy_increase(s));
    }
    if let Some(s) = &state.genetic {
        validations.push(systemic::validate_genetic_conservation(s));
    }

    for v in &validations {
        if !v.is_valid {
            failed_laws.push(v.law);
        }
    }
    ConservationValidationReport {
        validations,
        all_valid: failed_laws.is_empty(),
        failed_laws,
    }
}
