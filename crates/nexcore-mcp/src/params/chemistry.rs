//! Chemistry Primitives Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Bio-chemical reaction rates, decay, and feasibility parameters applied to digital state.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for Arrhenius rate calculation (threshold gating)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemistryThresholdRateParams {
    /// Pre-exponential factor A (sensitivity)
    pub pre_exponential: f64,
    /// Activation energy in kJ/mol (threshold)
    pub activation_energy_kj: f64,
    /// Temperature in Kelvin (scaling factor)
    pub temperature_k: f64,
}

/// Parameters for decay remaining calculation (half-life kinetics)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemistryDecayRemainingParams {
    /// Initial amount
    pub initial: f64,
    /// Half-life (time for 50% decay)
    pub half_life: f64,
    /// Time elapsed
    pub time: f64,
}

/// Parameters for Michaelis-Menten saturation rate
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemistrySaturationRateParams {
    /// Substrate/input concentration
    pub substrate: f64,
    /// Maximum rate (Vmax)
    pub v_max: f64,
    /// Half-saturation constant (Km)
    pub k_m: f64,
}

/// Parameters for Gibbs free energy feasibility calculation
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemistryFeasibilityParams {
    /// Enthalpy change ΔH in kJ/mol (direct benefit/cost)
    pub delta_h: f64,
    /// Entropy change ΔS in J/(mol·K) (disorder/complexity)
    pub delta_s: f64,
    /// Temperature in Kelvin (uncertainty scaling)
    pub temperature_k: f64,
}

/// Parameters for rate law dependency calculation
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemistryDependencyRateParams {
    /// Rate constant k
    pub k: f64,
    /// Reactants as (concentration, order) pairs
    pub reactants: Vec<(f64, f64)>,
}

/// Parameters for buffer capacity calculation
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemistryBufferCapacityParams {
    /// Total buffer concentration
    pub total_conc: f64,
    /// [A⁻]/[HA] ratio (base to acid)
    pub ratio: f64,
}

/// Parameters for Beer-Lambert absorbance calculation
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemistrySignalAbsorbanceParams {
    /// Molar absorptivity ε (L/(mol·cm))
    pub absorptivity: f64,
    /// Path length l (cm)
    pub path_length: f64,
    /// Concentration c (mol/L)
    pub concentration: f64,
}

/// Parameters for equilibrium steady-state fractions
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemistryEquilibriumParams {
    /// Equilibrium constant K
    pub k_eq: f64,
}

/// Parameters for simple threshold exceeded check
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemistryThresholdExceededParams {
    /// Signal value
    pub signal: f64,
    /// Threshold value
    pub threshold: f64,
}

/// Parameters for PV mappings (no args needed)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemistryPvMappingsParams {}

/// Parameters for Hill equation (cooperative binding)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemistryHillResponseParams {
    /// Input concentration or signal strength
    pub input: f64,
    /// Half-saturation constant (K₀.₅)
    pub k_half: f64,
    /// Hill coefficient (nH)
    pub n_hill: f64,
}

/// Parameters for Nernst equation (dynamic threshold)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemistryNernstParams {
    /// Standard potential (E⁰) in volts
    pub e_standard: f64,
    /// Temperature in Kelvin
    pub temperature_k: f64,
    /// Number of electrons transferred
    pub n_electrons: f64,
    /// Reaction quotient Q = [products]/[reactants]
    pub q: f64,
}

/// Parameters for competitive inhibition
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemistryInhibitionParams {
    /// Substrate concentration [S]
    pub substrate: f64,
    /// Maximum rate (Vmax)
    pub v_max: f64,
    /// Half-saturation constant (Km)
    pub k_m: f64,
    /// Inhibitor concentration [I]
    pub inhibitor: f64,
    /// Inhibition constant (Ki)
    pub k_i: f64,
}

/// Parameters for Eyring equation (transition state theory)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemistryEyringRateParams {
    /// Gibbs free energy of activation (ΔG‡) in J/mol
    pub delta_g: f64,
    /// Temperature in Kelvin
    pub temperature_k: f64,
    /// Transmission coefficient (κ)
    #[serde(default = "default_kappa")]
    pub kappa: f64,
}

fn default_kappa() -> f64 {
    1.0
}

/// Parameters for Langmuir isotherm (resource binding)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemistryLangmuirParams {
    /// Adsorbate concentration [A]
    pub concentration: f64,
    /// Equilibrium constant K (affinity)
    pub k_eq: f64,
}

/// Parameters for First Law closed system energy balance
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemistryFirstLawClosedParams {
    /// Initial internal energy (J)
    pub u_initial: f64,
    /// Heat added to system (J)
    pub heat_in: f64,
    /// Work done by system (J)
    pub work_out: f64,
}

/// Parameters for First Law open system energy balance
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemistryFirstLawOpenParams {
    /// Heat transfer rate (W)
    pub heat_rate: f64,
    /// Power output (W)
    pub power_out: f64,
    /// Inlet mass flow rates (kg/s)
    pub inflow_mass_rates: Vec<f64>,
    /// Inlet specific enthalpies (J/kg)
    pub inflow_enthalpies: Vec<f64>,
    /// Outlet mass flow rates (kg/s)
    pub outflow_mass_rates: Vec<f64>,
    /// Outlet specific enthalpies (J/kg)
    pub outflow_enthalpies: Vec<f64>,
}

/// Parameters for Gaussian primitive overlap integral calculation.
///
/// Computes the overlap integral between two s-type Gaussian primitives
/// with proper normalization factors N = (2α/π)^(3/4). Essential for
/// STO-nG basis set calculations in quantum chemistry visualization.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemistryGaussianOverlapParams {
    /// Exponents of Gaussian primitives on center A
    pub exponents_a: Vec<f64>,
    /// Contraction coefficients for center A
    pub coefficients_a: Vec<f64>,
    /// Position of center A as [x, y, z]
    pub center_a: [f64; 3],
    /// Exponents of Gaussian primitives on center B
    pub exponents_b: Vec<f64>,
    /// Contraction coefficients for center B
    pub coefficients_b: Vec<f64>,
    /// Position of center B as [x, y, z]
    pub center_b: [f64; 3],
}
