//! # Thermodynamic Kinetics
//!
//! Atoms for binding kinetics (kon, koff) and temperature-dependent
//! rate constants (Arrhenius equation).
//!
//! These complement Conservation Law 2 by providing the kinetic
//! aspects of drug-target interactions.
//!
//! References:
//! - Copeland RA et al. (2006) Nat Rev Drug Discov
//! - Arrhenius S (1889) Z Phys Chem

use super::binding::R_J_MOL_K;
use nexcore_error::Error;

/// Errors for kinetic calculations.
#[derive(Debug, Error, PartialEq)]
pub enum KineticsError {
    /// Rate constant is not positive.
    #[error("Rate constant (koff) must be positive")]
    KoffNotPositive,

    /// Dissociation constant is not positive.
    #[error("Dissociation constant (Kd) must be positive")]
    KdNotPositive,

    /// Rate constant is negative.
    #[error("Rate constant must be non-negative")]
    NegativeRateConstant,

    /// Temperature is not positive.
    #[error("Temperature must be positive Kelvin")]
    TemperatureNotPositive,

    /// Pre-exponential factor is negative.
    #[error("Pre-exponential factor must be non-negative")]
    NegativePreExponential,
}

/// Calculate drug-target residence time.
///
/// tau = 1/koff
///
/// Longer residence time often correlates with efficacy and selectivity.
///
/// # Arguments
///
/// * `koff` - Dissociation rate constant (s^-1)
///
/// # Returns
///
/// Residence time (seconds)
///
/// # Example
///
/// ```
/// use nexcore_vigilance::pv::thermodynamic::calculate_residence_time;
///
/// // koff = 0.001 s^-1 -> residence time = 1000 s (~17 min)
/// let tau = calculate_residence_time(0.001).unwrap();
/// assert!((tau - 1000.0).abs() < 0.1);
/// ```
pub fn calculate_residence_time(koff: f64) -> Result<f64, KineticsError> {
    if koff <= 0.0 {
        return Err(KineticsError::KoffNotPositive);
    }
    Ok(1.0 / koff)
}

/// Calculate koff from Kd and kon.
///
/// Kd = koff/kon -> koff = Kd * kon
///
/// # Arguments
///
/// * `kd` - Dissociation constant (M)
/// * `kon` - Association rate constant (M^-1 s^-1)
///
/// # Returns
///
/// Dissociation rate constant koff (s^-1)
///
/// # Example
///
/// ```
/// use nexcore_vigilance::pv::thermodynamic::calculate_koff_from_kd_kon;
///
/// // Kd = 1 nM, kon = 10^6 M^-1 s^-1
/// let koff = calculate_koff_from_kd_kon(1e-9, 1e6).unwrap();
/// assert!((koff - 0.001).abs() < 0.0001);
/// ```
pub fn calculate_koff_from_kd_kon(kd: f64, kon: f64) -> Result<f64, KineticsError> {
    if kd < 0.0 || kon < 0.0 {
        return Err(KineticsError::NegativeRateConstant);
    }
    Ok(kd * kon)
}

/// Calculate kon from Kd and koff.
///
/// Kd = koff/kon -> kon = koff/Kd
///
/// # Arguments
///
/// * `kd` - Dissociation constant (M)
/// * `koff` - Dissociation rate constant (s^-1)
///
/// # Returns
///
/// Association rate constant kon (M^-1 s^-1)
///
/// # Example
///
/// ```
/// use nexcore_vigilance::pv::thermodynamic::calculate_kon_from_kd_koff;
///
/// // Kd = 1 nM, koff = 0.001 s^-1
/// let kon = calculate_kon_from_kd_koff(1e-9, 0.001).unwrap();
/// assert!((kon - 1e6).abs() < 1e3);
/// ```
pub fn calculate_kon_from_kd_koff(kd: f64, koff: f64) -> Result<f64, KineticsError> {
    if kd <= 0.0 {
        return Err(KineticsError::KdNotPositive);
    }
    if koff < 0.0 {
        return Err(KineticsError::NegativeRateConstant);
    }
    Ok(koff / kd)
}

/// Calculate rate constant using Arrhenius equation.
///
/// k = A * exp(-Ea/RT)
///
/// # Arguments
///
/// * `pre_exponential_factor` - A (same units as k)
/// * `activation_energy_kj_mol` - Ea (kJ/mol)
/// * `temperature_k` - Temperature (K)
///
/// # Returns
///
/// Rate constant k
///
/// # Example
///
/// ```
/// use nexcore_vigilance::pv::thermodynamic::calculate_arrhenius_rate;
///
/// // A = 10^13 s^-1, Ea = 50 kJ/mol at 298K
/// let k = calculate_arrhenius_rate(1e13, 50.0, 298.15).unwrap();
/// // k = 10^13 * exp(-50000 / (8.314 * 298.15)) ≈ 17,374
/// assert!(k > 10_000.0 && k < 25_000.0);
/// ```
pub fn calculate_arrhenius_rate(
    pre_exponential_factor: f64,
    activation_energy_kj_mol: f64,
    temperature_k: f64,
) -> Result<f64, KineticsError> {
    if temperature_k <= 0.0 {
        return Err(KineticsError::TemperatureNotPositive);
    }
    if pre_exponential_factor < 0.0 {
        return Err(KineticsError::NegativePreExponential);
    }
    // Convert Ea from kJ/mol to J/mol
    let ea_j_mol = activation_energy_kj_mol * 1000.0;
    let exponent = -ea_j_mol / (R_J_MOL_K * temperature_k);
    Ok(pre_exponential_factor * exponent.exp())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_residence_time() {
        let tau = calculate_residence_time(0.001).unwrap();
        assert!((tau - 1000.0).abs() < 0.1);
    }

    #[test]
    fn test_residence_time_fast_off() {
        let tau = calculate_residence_time(1.0).unwrap();
        assert!((tau - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_residence_time_zero_koff() {
        assert!(matches!(
            calculate_residence_time(0.0),
            Err(KineticsError::KoffNotPositive)
        ));
    }

    #[test]
    fn test_koff_from_kd_kon() {
        // Kd = 1 nM, kon = 10^6 M^-1 s^-1 -> koff = 10^-9 * 10^6 = 0.001
        let koff = calculate_koff_from_kd_kon(1e-9, 1e6).unwrap();
        assert!((koff - 0.001).abs() < 0.0001);
    }

    #[test]
    fn test_koff_negative_input() {
        assert!(matches!(
            calculate_koff_from_kd_kon(-1e-9, 1e6),
            Err(KineticsError::NegativeRateConstant)
        ));
    }

    #[test]
    fn test_kon_from_kd_koff() {
        // Kd = 1 nM, koff = 0.001 s^-1 -> kon = 0.001 / 10^-9 = 10^6
        let kon = calculate_kon_from_kd_koff(1e-9, 0.001).unwrap();
        assert!((kon - 1e6).abs() < 1e3);
    }

    #[test]
    fn test_kon_zero_kd() {
        assert!(matches!(
            calculate_kon_from_kd_koff(0.0, 0.001),
            Err(KineticsError::KdNotPositive)
        ));
    }

    #[test]
    fn test_arrhenius_rate() {
        // A = 10^13 s^-1, Ea = 50 kJ/mol at 298K
        let k = calculate_arrhenius_rate(1e13, 50.0, 298.15).unwrap();
        // k = 10^13 * exp(-50000 / (8.314 * 298.15)) = 10^13 * exp(-20.17) ≈ 17,374
        assert!(k > 15_000.0 && k < 20_000.0);
    }

    #[test]
    fn test_arrhenius_rate_low_ea() {
        // Low activation energy -> fast reaction
        let k = calculate_arrhenius_rate(1e13, 10.0, 298.15).unwrap();
        assert!(k > 1e9); // Much faster
    }

    #[test]
    fn test_arrhenius_rate_high_ea() {
        // High activation energy -> slow reaction
        let k = calculate_arrhenius_rate(1e13, 100.0, 298.15).unwrap();
        assert!(k < 1.0); // Very slow
    }

    #[test]
    fn test_arrhenius_rate_zero_temp() {
        assert!(matches!(
            calculate_arrhenius_rate(1e13, 50.0, 0.0),
            Err(KineticsError::TemperatureNotPositive)
        ));
    }

    #[test]
    fn test_arrhenius_rate_negative_prefactor() {
        assert!(matches!(
            calculate_arrhenius_rate(-1e13, 50.0, 298.15),
            Err(KineticsError::NegativePreExponential)
        ));
    }

    #[test]
    fn test_arrhenius_temperature_dependence() {
        // Higher temperature -> faster reaction
        let k1 = calculate_arrhenius_rate(1e13, 50.0, 298.15).unwrap();
        let k2 = calculate_arrhenius_rate(1e13, 50.0, 310.15).unwrap();
        assert!(k2 > k1);
    }
}
