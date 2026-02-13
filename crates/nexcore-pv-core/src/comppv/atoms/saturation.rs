//! Saturation atoms (Law 9)

use crate::comppv::types::ConservationError;

/// Calculate saturation fraction via Michaelis-Menten kinetics (Law 9).
///
/// f = [S] / (K_m + [S])
///
/// # Arguments
/// * `concentration` - Substrate concentration [S]
/// * `half_saturation` - Michaelis constant (K_m)
///
/// # Errors
/// Returns error if concentration < 0 or half_saturation ≤ 0
pub fn calculate_saturation_fraction(
    concentration: f64,
    half_saturation: f64,
) -> Result<f64, ConservationError> {
    if concentration < 0.0 || half_saturation <= 0.0 {
        return Err(ConservationError::InvalidParameter(
            "Concentration must be non-negative and half-saturation positive".to_string(),
        ));
    }
    Ok(concentration / (half_saturation + concentration))
}
