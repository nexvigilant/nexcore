//! Steady-State PK atoms (Law 7)

use crate::comppv::types::ConservationError;

/// Calculate steady-state concentration (Law 7: PK Conservation).
///
/// C_ss = (F × Dose) / (CL × τ)
///
/// # Arguments
/// * `bioavailability` - Fraction reaching systemic circulation (F, 0-1)
/// * `dose` - Drug dose per interval
/// * `clearance_l_h` - Clearance in L/h
/// * `dosing_interval_h` - Time between doses (τ) in hours
///
/// # Errors
/// Returns error if bioavailability outside [0,1] or clearance/interval ≤ 0
pub fn calculate_steady_state_concentration(
    bioavailability: f64,
    dose: f64,
    clearance_l_h: f64,
    dosing_interval_h: f64,
) -> Result<f64, ConservationError> {
    if !(0.0..=1.0).contains(&bioavailability) || clearance_l_h <= 0.0 || dosing_interval_h <= 0.0 {
        return Err(ConservationError::InvalidParameter(
            "Invalid PK parameters".to_string(),
        ));
    }
    Ok((bioavailability * dose) / (clearance_l_h * dosing_interval_h))
}
