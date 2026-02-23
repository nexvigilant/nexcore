//! # Transition State Theory (Eyring Equation)
//!
//! **T1 Components**: threshold × frequency × state × transition × quantity
//!
//! **Chemistry**: k = (kB*T/h) * exp(-ΔG‡/RT)
//!
//! **Universal Pattern**: Rate depends on energy barrier AND entropy of transition.
//! More accurate than Arrhenius because it accounts for transition state structure.
//!
//! **PV Application**: Signal escalation rate - accounts for both energy barrier
//! (detection threshold) and "transition complexity" (review process structure).
//!
//! **Bond Application**: Bond activation rate considering both energy and
//! organizational overhead (entropy cost of coordination).

use nexcore_error::Error;

/// Boltzmann constant (J/K)
pub const BOLTZMANN: f64 = 1.380649e-23;

/// Planck constant (J·s)
pub const PLANCK: f64 = 6.62607015e-34;

/// Gas constant (J/(mol·K))
pub const GAS_CONSTANT: f64 = 8.314;

/// Errors for transition state calculations.
#[derive(Debug, Error, PartialEq, Clone)]
pub enum TransitionStateError {
    /// Temperature must be positive (Kelvin).
    #[error("Temperature must be positive (Kelvin)")]
    InvalidTemperature,
    /// Transmission coefficient must be between 0 and 1.
    #[error("Transmission coefficient must be between 0 and 1")]
    InvalidTransmission,
}

/// Transition state theory configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct TransitionState {
    /// Gibbs free energy of activation (ΔG‡) in J/mol
    pub delta_g_activation: f64,
    /// Temperature in Kelvin
    pub temperature_k: f64,
    /// Transmission coefficient (κ), typically ~1
    pub kappa: f64,
}

/// Activation parameter decomposition.
#[derive(Debug, Clone, PartialEq)]
pub struct ActivationParameters {
    /// Enthalpy of activation (ΔH‡) in J/mol
    pub delta_h: f64,
    /// Entropy of activation (ΔS‡) in J/(mol·K)
    pub delta_s: f64,
    /// Computed ΔG‡ at given temperature
    pub delta_g: f64,
    /// Temperature used for calculation
    pub temperature_k: f64,
}

impl TransitionState {
    /// Create new transition state model.
    pub fn new(
        delta_g_activation: f64,
        temperature_k: f64,
        kappa: f64,
    ) -> Result<Self, TransitionStateError> {
        if temperature_k <= 0.0 {
            return Err(TransitionStateError::InvalidTemperature);
        }
        if !(0.0..=1.0).contains(&kappa) {
            return Err(TransitionStateError::InvalidTransmission);
        }
        Ok(Self {
            delta_g_activation,
            temperature_k,
            kappa,
        })
    }

    /// Calculate rate constant using Eyring equation.
    ///
    /// k = κ × (kB*T/h) × exp(-ΔG‡/RT)
    #[must_use]
    pub fn rate_constant(&self) -> f64 {
        eyring_rate(self.delta_g_activation, self.temperature_k, self.kappa)
    }

    /// Calculate rate constant from ΔH‡ and ΔS‡.
    ///
    /// ΔG‡ = ΔH‡ - T×ΔS‡
    #[must_use]
    pub fn rate_from_components(&self, delta_h: f64, delta_s: f64) -> f64 {
        let delta_g = delta_h - self.temperature_k * delta_s;
        eyring_rate(delta_g, self.temperature_k, self.kappa)
    }

    /// Calculate half-life from rate constant.
    ///
    /// t½ = ln(2) / k
    #[must_use]
    pub fn half_life(&self) -> f64 {
        let k = self.rate_constant();
        if k <= 0.0 {
            return f64::INFINITY;
        }
        0.693 / k
    }
}

/// Calculate Eyring equation rate constant.
///
/// k = κ × (kB×T/h) × exp(-ΔG‡/RT)
///
/// # Arguments
/// * `delta_g` - Gibbs free energy of activation (J/mol)
/// * `temperature_k` - Temperature (Kelvin)
/// * `kappa` - Transmission coefficient (0-1, typically ~1)
///
/// # Returns
/// Rate constant (s⁻¹)
#[must_use]
pub fn eyring_rate(delta_g: f64, temperature_k: f64, kappa: f64) -> f64 {
    if temperature_k <= 0.0 || kappa <= 0.0 {
        return 0.0;
    }
    let prefactor = kappa * BOLTZMANN * temperature_k / PLANCK;
    let exponent = -delta_g / (GAS_CONSTANT * temperature_k);
    prefactor * exponent.exp()
}

/// Calculate ΔG‡ from ΔH‡ and ΔS‡.
///
/// ΔG‡ = ΔH‡ - T×ΔS‡
#[must_use]
pub fn gibbs_activation(delta_h: f64, delta_s: f64, temperature_k: f64) -> f64 {
    delta_h - temperature_k * delta_s
}

/// Calculate activation parameters from two rate measurements.
///
/// Using Eyring plot: ln(k/T) = -ΔH‡/RT + ΔS‡/R + ln(kB/h)
///
/// # Arguments
/// * `k1` - Rate constant at T1
/// * `t1` - Temperature 1 (Kelvin)
/// * `k2` - Rate constant at T2
/// * `t2` - Temperature 2 (Kelvin)
pub fn activation_from_rates(
    k1: f64,
    t1: f64,
    k2: f64,
    t2: f64,
) -> Result<ActivationParameters, TransitionStateError> {
    if t1 <= 0.0 || t2 <= 0.0 {
        return Err(TransitionStateError::InvalidTemperature);
    }
    if k1 <= 0.0 || k2 <= 0.0 || (t1 - t2).abs() < 1e-6 {
        // Can't compute from invalid data
        return Ok(ActivationParameters {
            delta_h: 0.0,
            delta_s: 0.0,
            delta_g: 0.0,
            temperature_k: t1,
        });
    }

    // Eyring plot: ln(k/T) vs 1/T
    // Slope = -ΔH‡/R
    // Intercept = ΔS‡/R + ln(kB/h)
    let y1 = (k1 / t1).ln();
    let y2 = (k2 / t2).ln();
    let x1 = 1.0 / t1;
    let x2 = 1.0 / t2;

    let slope = (y2 - y1) / (x2 - x1);
    let delta_h = -slope * GAS_CONSTANT;

    let ln_kb_h = (BOLTZMANN / PLANCK).ln();
    let intercept = y1 - slope * x1;
    let delta_s = (intercept - ln_kb_h) * GAS_CONSTANT;

    let delta_g = delta_h - t1 * delta_s;

    Ok(ActivationParameters {
        delta_h,
        delta_s,
        delta_g,
        temperature_k: t1,
    })
}

/// Compare Eyring to Arrhenius rate.
///
/// Returns (eyring_rate, arrhenius_rate, ratio)
#[must_use]
pub fn compare_to_arrhenius(
    delta_g: f64,
    temperature_k: f64,
    pre_exponential: f64,
    activation_energy: f64,
) -> (f64, f64, f64) {
    let k_eyring = eyring_rate(delta_g, temperature_k, 1.0);
    let k_arrhenius = pre_exponential * (-activation_energy / (GAS_CONSTANT * temperature_k)).exp();
    let ratio = if k_arrhenius > 0.0 {
        k_eyring / k_arrhenius
    } else {
        0.0
    };
    (k_eyring, k_arrhenius, ratio)
}

/// Calculate frequency factor from Eyring theory.
///
/// A = (kB×T/h) × exp(ΔS‡/R)
#[must_use]
pub fn frequency_factor(delta_s: f64, temperature_k: f64) -> f64 {
    if temperature_k <= 0.0 {
        return 0.0;
    }
    let prefactor = BOLTZMANN * temperature_k / PLANCK;
    prefactor * (delta_s / GAS_CONSTANT).exp()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eyring_rate_positive_barrier() {
        // Typical activation barrier ~50 kJ/mol at 298K
        let rate = eyring_rate(50_000.0, 298.15, 1.0);
        // Should be a reasonable rate (not too fast, not too slow)
        assert!(rate > 0.0);
        assert!(rate < 1e15); // Less than vibrational frequency
    }

    #[test]
    fn test_eyring_rate_zero_barrier() {
        // Zero barrier = diffusion-limited
        let rate = eyring_rate(0.0, 298.15, 1.0);
        // Should be ~ kB*T/h ≈ 6.2e12 s⁻¹ at 298K
        assert!((rate - 6.2e12).abs() / 6.2e12 < 0.1);
    }

    #[test]
    fn test_temperature_effect() {
        // Higher temperature = faster rate
        let rate_low = eyring_rate(50_000.0, 298.15, 1.0);
        let rate_high = eyring_rate(50_000.0, 373.15, 1.0);
        assert!(rate_high > rate_low);
    }

    #[test]
    fn test_transmission_coefficient() {
        // κ=0.5 should give half the rate
        let rate_full = eyring_rate(50_000.0, 298.15, 1.0);
        let rate_half = eyring_rate(50_000.0, 298.15, 0.5);
        assert!((rate_half / rate_full - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_gibbs_activation() {
        // ΔG‡ = ΔH‡ - T×ΔS‡
        // At 298K with ΔH‡=50kJ and ΔS‡=-100 J/K
        let delta_g = gibbs_activation(50_000.0, -100.0, 298.15);
        // ΔG‡ = 50000 - 298.15×(-100) = 50000 + 29815 = 79815
        assert!((delta_g - 79_815.0).abs() < 1.0);
    }

    #[test]
    fn test_transition_state_struct() {
        let ts = TransitionState::new(50_000.0, 298.15, 1.0).unwrap();
        let rate = ts.rate_constant();
        assert!(rate > 0.0);

        let half_life = ts.half_life();
        assert!(half_life > 0.0);
    }

    #[test]
    fn test_activation_from_rates() {
        // Create rates at two temperatures
        let ts = TransitionState::new(50_000.0, 298.15, 1.0).unwrap();
        let k1 = ts.rate_constant();

        let ts2 = TransitionState::new(50_000.0, 323.15, 1.0).unwrap();
        let k2 = ts2.rate_constant();

        // Recover parameters
        let params = activation_from_rates(k1, 298.15, k2, 323.15).unwrap();

        // ΔG‡ should be close to original
        assert!((params.delta_g - 50_000.0).abs() / 50_000.0 < 0.1);
    }

    #[test]
    fn test_invalid_temperature() {
        let result = TransitionState::new(50_000.0, -100.0, 1.0);
        assert_eq!(result, Err(TransitionStateError::InvalidTemperature));
    }

    #[test]
    fn test_invalid_transmission() {
        let result = TransitionState::new(50_000.0, 298.15, 1.5);
        assert_eq!(result, Err(TransitionStateError::InvalidTransmission));
    }
}
