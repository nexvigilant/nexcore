//! Thermodynamic Binding atoms (Law 2)

use crate::comppv::types::ConservationError;
use crate::comppv::types::constants::{GAS_CONSTANT_KJ_MOL_K, STANDARD_TEMPERATURE_K};

/// Calculate Gibbs free energy from association constant (Law 2).
///
/// ΔG = -RT ln(K_a)
///
/// # Arguments
/// * `ka_m_inv` - Association constant in M⁻¹
/// * `temperature_k` - Temperature in Kelvin (defaults to 310.15K / 37°C)
///
/// # Errors
/// Returns error if K_a ≤ 0 or T ≤ 0
pub fn calculate_gibbs_free_energy(
    ka_m_inv: f64,
    temperature_k: Option<f64>,
) -> Result<f64, ConservationError> {
    let t = temperature_k.unwrap_or(STANDARD_TEMPERATURE_K);
    if ka_m_inv <= 0.0 || t <= 0.0 {
        return Err(ConservationError::InvalidParameter(
            "Association constant and temperature must be positive".to_string(),
        ));
    }
    Ok(-GAS_CONSTANT_KJ_MOL_K * t * ka_m_inv.ln())
}

/// Check if binding is thermodynamically spontaneous.
///
/// Binding is spontaneous when ΔG < 0.
#[must_use]
pub fn is_spontaneous_binding(delta_g_kj_mol: f64) -> bool {
    delta_g_kj_mol < 0.0
}
