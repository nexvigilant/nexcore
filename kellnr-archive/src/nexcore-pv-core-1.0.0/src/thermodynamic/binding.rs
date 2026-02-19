//! # Thermodynamic Binding (Conservation Law 2)
//!
//! DeltaG = DeltaH - T*DeltaS < 0 for spontaneous binding
//! Kd = exp(DeltaG/RT)
//!
//! Drug-target binding is governed by thermodynamic equilibrium;
//! spontaneous binding requires negative Gibbs free energy.
//!
//! Safety Implication: Off-target binding occurs when DeltaG is favorable for
//! non-intended targets; selectivity = Delta_DeltaG between targets.

use thiserror::Error;

/// Gas constant in J/(mol*K).
pub const R_J_MOL_K: f64 = 8.314;

/// Gas constant in kJ/(mol*K).
pub const R_KJ_MOL_K: f64 = 0.008314;

/// Standard temperature in Kelvin (25°C).
pub const STANDARD_TEMP_K: f64 = 298.15;

/// Errors for thermodynamic binding calculations.
#[derive(Debug, Error, PartialEq)]
pub enum BindingError {
    /// Association constant is not positive.
    #[error("Association constant must be positive")]
    KaNotPositive,

    /// Dissociation constant is not positive.
    #[error("Dissociation constant must be positive")]
    KdNotPositive,

    /// Temperature is not positive.
    #[error("Temperature must be positive Kelvin")]
    TemperatureNotPositive,
}

/// Calculate Gibbs free energy of binding from association constant.
///
/// DeltaG = -RT ln(Ka)
///
/// # Arguments
///
/// * `ka` - Association constant (M^-1)
/// * `temperature_k` - Temperature in Kelvin (default 298.15K)
///
/// # Returns
///
/// DeltaG in kJ/mol (negative = spontaneous)
///
/// # Example
///
/// ```
/// use nexcore_vigilance::pv::thermodynamic::calculate_gibbs_free_energy;
///
/// // High affinity binding (Ka = 10^9 M^-1) -> strongly negative DeltaG
/// let dg = calculate_gibbs_free_energy(1e9, 298.15).unwrap();
/// assert!(dg < -50.0); // Very favorable
/// ```
pub fn calculate_gibbs_free_energy(ka: f64, temperature_k: f64) -> Result<f64, BindingError> {
    if ka <= 0.0 {
        return Err(BindingError::KaNotPositive);
    }
    if temperature_k <= 0.0 {
        return Err(BindingError::TemperatureNotPositive);
    }
    Ok(-R_KJ_MOL_K * temperature_k * ka.ln())
}

/// Calculate Kd from Gibbs free energy.
///
/// Kd = exp(DeltaG/RT)
///
/// # Arguments
///
/// * `delta_g` - Gibbs free energy (kJ/mol)
/// * `temperature_k` - Temperature in Kelvin
///
/// # Returns
///
/// Dissociation constant Kd (M)
///
/// # Example
///
/// ```
/// use nexcore_vigilance::pv::thermodynamic::calculate_dissociation_constant;
///
/// // Favorable binding (DeltaG = -40 kJ/mol)
/// let kd = calculate_dissociation_constant(-40.0, 298.15).unwrap();
/// assert!(kd < 1e-6); // nanomolar affinity
/// ```
pub fn calculate_dissociation_constant(
    delta_g: f64,
    temperature_k: f64,
) -> Result<f64, BindingError> {
    if temperature_k <= 0.0 {
        return Err(BindingError::TemperatureNotPositive);
    }
    Ok((delta_g / (R_KJ_MOL_K * temperature_k)).exp())
}

/// Calculate association constant from Kd.
///
/// Ka = 1/Kd
///
/// # Arguments
///
/// * `kd` - Dissociation constant (M)
///
/// # Returns
///
/// Association constant Ka (M^-1)
pub fn calculate_association_constant(kd: f64) -> Result<f64, BindingError> {
    if kd <= 0.0 {
        return Err(BindingError::KdNotPositive);
    }
    Ok(1.0 / kd)
}

/// Calculate selectivity as Delta_DeltaG between targets.
///
/// Selectivity = |DeltaG_target - DeltaG_off-target|
///
/// Higher Delta_DeltaG = more selective (less off-target binding).
///
/// # Arguments
///
/// * `delta_g_target` - DeltaG for intended target (kJ/mol)
/// * `delta_g_off_target` - DeltaG for off-target (kJ/mol)
///
/// # Returns
///
/// Selectivity as Delta_DeltaG (kJ/mol, higher = more selective)
///
/// # Example
///
/// ```
/// use nexcore_vigilance::pv::thermodynamic::calculate_selectivity;
///
/// // Good selectivity: strong target binding, weak off-target
/// let sel = calculate_selectivity(-50.0, -30.0);
/// assert!((sel - 20.0).abs() < 0.001);
/// ```
pub fn calculate_selectivity(delta_g_target: f64, delta_g_off_target: f64) -> f64 {
    (delta_g_target - delta_g_off_target).abs()
}

/// Check if binding is thermodynamically favorable.
///
/// Spontaneous when DeltaG < 0.
///
/// # Arguments
///
/// * `delta_g` - Gibbs free energy (kJ/mol)
///
/// # Returns
///
/// true if binding is spontaneous
pub fn is_spontaneous_binding(delta_g: f64) -> bool {
    delta_g < 0.0
}

/// Calculate entropy contribution from DeltaG and DeltaH.
///
/// DeltaS = (DeltaH - DeltaG) / T
///
/// # Arguments
///
/// * `delta_g` - Gibbs free energy (kJ/mol)
/// * `delta_h` - Enthalpy (kJ/mol)
/// * `temperature_k` - Temperature (K)
///
/// # Returns
///
/// Entropy DeltaS (J/(mol*K))
///
/// # Example
///
/// ```
/// use nexcore_vigilance::pv::thermodynamic::calculate_binding_entropy;
///
/// // Enthalpy-driven binding: DeltaH = -50, DeltaG = -40
/// let ds = calculate_binding_entropy(-40.0, -50.0, 298.15).unwrap();
/// // DeltaS = (-50 - (-40)) * 1000 / 298.15 = -10000/298.15 = -33.5 J/(mol*K)
/// assert!(ds < 0.0); // Unfavorable entropy (enthalpy-driven)
/// ```
pub fn calculate_binding_entropy(
    delta_g: f64,
    delta_h: f64,
    temperature_k: f64,
) -> Result<f64, BindingError> {
    if temperature_k <= 0.0 {
        return Err(BindingError::TemperatureNotPositive);
    }
    // Convert kJ to J for entropy units
    Ok(((delta_h - delta_g) * 1000.0) / temperature_k)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gibbs_free_energy_high_affinity() {
        // Ka = 10^9 M^-1 (nanomolar Kd)
        let dg = calculate_gibbs_free_energy(1e9, STANDARD_TEMP_K).unwrap();
        // DeltaG = -0.008314 * 298.15 * ln(10^9) = -0.008314 * 298.15 * 20.72 = -51.4
        assert!((dg - (-51.4)).abs() < 0.5);
    }

    #[test]
    fn test_gibbs_free_energy_low_affinity() {
        // Ka = 10^3 M^-1 (millimolar Kd)
        let dg = calculate_gibbs_free_energy(1e3, STANDARD_TEMP_K).unwrap();
        // DeltaG = -0.008314 * 298.15 * ln(1000) = -0.008314 * 298.15 * 6.91 = -17.1
        assert!((dg - (-17.1)).abs() < 0.5);
    }

    #[test]
    fn test_gibbs_free_energy_zero_ka() {
        assert!(matches!(
            calculate_gibbs_free_energy(0.0, STANDARD_TEMP_K),
            Err(BindingError::KaNotPositive)
        ));
    }

    #[test]
    fn test_gibbs_free_energy_zero_temp() {
        assert!(matches!(
            calculate_gibbs_free_energy(1e6, 0.0),
            Err(BindingError::TemperatureNotPositive)
        ));
    }

    #[test]
    fn test_dissociation_constant_favorable() {
        // DeltaG = -40 kJ/mol (favorable)
        let kd = calculate_dissociation_constant(-40.0, STANDARD_TEMP_K).unwrap();
        // Kd = exp(-40 / (0.008314 * 298.15)) = exp(-16.14) ≈ 10^-7
        assert!(kd < 1e-6);
        assert!(kd > 1e-8);
    }

    #[test]
    fn test_dissociation_constant_unfavorable() {
        // DeltaG = +10 kJ/mol (unfavorable)
        let kd = calculate_dissociation_constant(10.0, STANDARD_TEMP_K).unwrap();
        // Should be > 1
        assert!(kd > 1.0);
    }

    #[test]
    fn test_association_constant() {
        let ka = calculate_association_constant(1e-9).unwrap();
        assert!((ka - 1e9).abs() < 1e6);
    }

    #[test]
    fn test_association_constant_zero_kd() {
        assert!(matches!(
            calculate_association_constant(0.0),
            Err(BindingError::KdNotPositive)
        ));
    }

    #[test]
    fn test_selectivity() {
        let sel = calculate_selectivity(-50.0, -30.0);
        assert!((sel - 20.0).abs() < 0.001);
    }

    #[test]
    fn test_selectivity_symmetric() {
        let sel1 = calculate_selectivity(-50.0, -30.0);
        let sel2 = calculate_selectivity(-30.0, -50.0);
        assert!((sel1 - sel2).abs() < 0.001);
    }

    #[test]
    fn test_is_spontaneous_binding() {
        assert!(is_spontaneous_binding(-10.0));
        assert!(is_spontaneous_binding(-0.001));
        assert!(!is_spontaneous_binding(0.0));
        assert!(!is_spontaneous_binding(10.0));
    }

    #[test]
    fn test_binding_entropy_enthalpy_driven() {
        // Enthalpy-driven: DeltaH < DeltaG (both negative)
        let ds = calculate_binding_entropy(-40.0, -50.0, STANDARD_TEMP_K).unwrap();
        // DeltaS = (-50 - (-40)) * 1000 / 298.15 = -33.5
        assert!(ds < 0.0); // Unfavorable entropy
        assert!((ds - (-33.5)).abs() < 1.0);
    }

    #[test]
    fn test_binding_entropy_entropy_driven() {
        // Entropy-driven: DeltaH > DeltaG
        let ds = calculate_binding_entropy(-40.0, -30.0, STANDARD_TEMP_K).unwrap();
        // DeltaS = (-30 - (-40)) * 1000 / 298.15 = +33.5
        assert!(ds > 0.0); // Favorable entropy
    }

    #[test]
    fn test_binding_entropy_zero_temp() {
        assert!(matches!(
            calculate_binding_entropy(-40.0, -50.0, 0.0),
            Err(BindingError::TemperatureNotPositive)
        ));
    }
}
