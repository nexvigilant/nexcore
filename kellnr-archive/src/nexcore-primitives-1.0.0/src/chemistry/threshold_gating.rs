//! # Threshold Gating (Arrhenius Equation)
//!
//! **T1 Components**: threshold × quantity × frequency × cause × effect
//!
//! **Chemistry**: k = A × e^(-Ea/RT)
//!
//! **Universal Pattern**: Rate depends exponentially on activation barrier
//! relative to available energy. Action is gated by threshold.
//!
//! **PV Application**: Signal detection threshold - signals must exceed
//! activation energy to trigger detection.

use thiserror::Error;

/// Errors for threshold gating calculations.
#[derive(Debug, Error, PartialEq, Clone)]
pub enum ThresholdError {
    /// Temperature/scaling factor must be positive.
    #[error("Temperature/scaling factor must be positive")]
    ScalingNotPositive,
    /// Pre-exponential factor must be non-negative.
    #[error("Pre-exponential factor must be non-negative")]
    NegativeSensitivity,
    /// Threshold must be non-negative.
    #[error("Threshold must be non-negative")]
    NegativeThreshold,
}

/// Threshold gate configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct ThresholdGate {
    /// Base sensitivity (A - pre-exponential factor)
    pub sensitivity: f64,
    /// Activation threshold (Ea - activation energy)
    pub threshold: f64,
}

impl ThresholdGate {
    /// Create a new threshold gate.
    pub fn new(sensitivity: f64, threshold: f64) -> Result<Self, ThresholdError> {
        if sensitivity < 0.0 {
            return Err(ThresholdError::NegativeSensitivity);
        }
        if threshold < 0.0 {
            return Err(ThresholdError::NegativeThreshold);
        }
        Ok(Self {
            sensitivity,
            threshold,
        })
    }

    /// Calculate rate at given signal strength.
    ///
    /// # Arguments
    /// * `signal` - Input signal strength (analogous to RT)
    pub fn rate(&self, signal: f64) -> Result<f64, ThresholdError> {
        if signal <= 0.0 {
            return Err(ThresholdError::ScalingNotPositive);
        }
        Ok(self.sensitivity * (-self.threshold / signal).exp())
    }

    /// Check if signal exceeds threshold (rate > cutoff).
    pub fn exceeds_threshold(&self, signal: f64, cutoff: f64) -> Result<bool, ThresholdError> {
        let rate = self.rate(signal)?;
        Ok(rate > cutoff)
    }
}

/// Gas constant in J/(mol·K).
pub const R_J_MOL_K: f64 = 8.314;

/// Calculate rate using Arrhenius equation.
///
/// k = A × e^(-Ea/RT)
///
/// Wraps `pv::thermodynamic::calculate_arrhenius_rate` with universal semantics.
///
/// # Arguments
/// * `pre_exponential` - A (sensitivity, same units as k)
/// * `activation_energy_kj` - Ea (threshold, kJ/mol)
/// * `temperature_k` - T (scaling factor, Kelvin)
///
/// # Returns
/// Rate constant k
pub fn arrhenius_rate(
    pre_exponential: f64,
    activation_energy_kj: f64,
    temperature_k: f64,
) -> Result<f64, ThresholdError> {
    if temperature_k <= 0.0 {
        return Err(ThresholdError::ScalingNotPositive);
    }
    if pre_exponential < 0.0 {
        return Err(ThresholdError::NegativeSensitivity);
    }
    let ea_j = activation_energy_kj * 1000.0;
    let exponent = -ea_j / (R_J_MOL_K * temperature_k);
    Ok(pre_exponential * exponent.exp())
}

/// Calculate activation probability (normalized 0-1).
///
/// Useful for probabilistic signal detection.
///
/// # Arguments
/// * `signal` - Observed signal strength
/// * `threshold` - Detection threshold
/// * `sensitivity` - Base sensitivity (default 1.0)
pub fn activation_probability(signal: f64, threshold: f64, sensitivity: f64) -> f64 {
    if signal <= 0.0 {
        return 0.0;
    }
    let raw = sensitivity * (-threshold / signal).exp();
    raw.min(1.0) // Cap at 1.0 for probability
}

/// Check if signal exceeds threshold for detection.
///
/// Simple boolean gate: signal > threshold
#[must_use]
pub fn threshold_exceeded(signal: f64, threshold: f64) -> bool {
    signal > threshold
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arrhenius_rate() {
        // A = 10^13, Ea = 50 kJ/mol, T = 298K
        let k = arrhenius_rate(1e13, 50.0, 298.15).unwrap();
        assert!(k > 15_000.0 && k < 20_000.0);
    }

    #[test]
    fn test_arrhenius_temperature_effect() {
        let k1 = arrhenius_rate(1e13, 50.0, 298.15).unwrap();
        let k2 = arrhenius_rate(1e13, 50.0, 310.15).unwrap();
        assert!(k2 > k1, "Higher temperature should increase rate");
    }

    #[test]
    fn test_arrhenius_zero_temp() {
        assert!(matches!(
            arrhenius_rate(1e13, 50.0, 0.0),
            Err(ThresholdError::ScalingNotPositive)
        ));
    }

    #[test]
    fn test_threshold_gate() {
        let gate = ThresholdGate::new(1.0, 10.0).unwrap();
        let rate = gate.rate(5.0).unwrap();
        assert!(rate > 0.0 && rate < 1.0);
    }

    #[test]
    fn test_activation_probability_bounds() {
        let prob = activation_probability(100.0, 10.0, 1.0);
        assert!(prob >= 0.0 && prob <= 1.0);
    }

    #[test]
    fn test_threshold_exceeded() {
        assert!(threshold_exceeded(2.1, 2.0));
        assert!(!threshold_exceeded(1.9, 2.0));
    }
}
