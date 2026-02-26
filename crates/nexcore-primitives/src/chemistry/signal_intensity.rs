//! # Signal Intensity (Beer-Lambert Law)
//!
//! **T1 Components**: signal × proportion × quantity × cause × effect
//!
//! **Chemistry**: A = εlc (absorbance = molar absorptivity × path length × concentration)
//!
//! **Universal Pattern**: Signal strength is proportional to concentration
//! and path length. Linear relationship enables quantification.
//!
//! **PV Application**: Dose-response linearity - signal proportional to exposure.

use nexcore_error::Error;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Errors for signal intensity calculations.
#[derive(Debug, Error, PartialEq, Clone)]
pub enum SignalError {
    /// Absorbance must be non-negative.
    #[error("Absorbance must be non-negative")]
    NegativeAbsorbance,
    /// Parameters must be positive.
    #[error("Parameters must be positive")]
    NonPositiveParameter,
    /// Detection limit exceeded.
    #[error("Signal below detection limit")]
    BelowDetectionLimit,
}

/// Signal detector configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignalDetector {
    /// Molar absorptivity ε (L/(mol·cm))
    pub absorptivity: f64,
    /// Path length l (cm)
    pub path_length: f64,
    /// Detection limit (minimum absorbance)
    pub detection_limit: f64,
}

impl fmt::Display for SignalDetector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Signal(\u{03B5}={:.1}, l={:.2})",
            self.absorptivity, self.path_length
        )
    }
}

impl SignalDetector {
    /// Create new signal detector.
    pub fn new(
        absorptivity: f64,
        path_length: f64,
        detection_limit: f64,
    ) -> Result<Self, SignalError> {
        if absorptivity <= 0.0 || path_length <= 0.0 {
            return Err(SignalError::NonPositiveParameter);
        }
        if detection_limit < 0.0 {
            return Err(SignalError::NegativeAbsorbance);
        }
        Ok(Self {
            absorptivity,
            path_length,
            detection_limit,
        })
    }

    /// Calculate absorbance from concentration.
    pub fn absorbance(&self, concentration: f64) -> Result<f64, SignalError> {
        if concentration < 0.0 {
            return Err(SignalError::NonPositiveParameter);
        }
        Ok(self.absorptivity * self.path_length * concentration)
    }

    /// Infer concentration from absorbance.
    pub fn concentration(&self, absorbance: f64) -> Result<f64, SignalError> {
        if absorbance < 0.0 {
            return Err(SignalError::NegativeAbsorbance);
        }
        Ok(absorbance / (self.absorptivity * self.path_length))
    }

    /// Check if signal is detectable.
    pub fn is_detectable(&self, absorbance: f64) -> bool {
        absorbance >= self.detection_limit
    }

    /// Calculate minimum detectable concentration.
    pub fn min_detectable_concentration(&self) -> f64 {
        self.detection_limit / (self.absorptivity * self.path_length)
    }
}

/// Calculate absorbance using Beer-Lambert law.
///
/// A = εlc
///
/// # Arguments
/// * `absorptivity` - Molar absorptivity ε (L/(mol·cm))
/// * `path_length` - Path length l (cm)
/// * `concentration` - Concentration c (mol/L)
pub fn beer_lambert_absorbance(
    absorptivity: f64,
    path_length: f64,
    concentration: f64,
) -> Result<f64, SignalError> {
    if absorptivity <= 0.0 || path_length <= 0.0 || concentration < 0.0 {
        return Err(SignalError::NonPositiveParameter);
    }
    Ok(absorptivity * path_length * concentration)
}

/// Infer concentration from absorbance.
///
/// c = A / (εl)
pub fn infer_concentration(
    absorbance: f64,
    absorptivity: f64,
    path_length: f64,
) -> Result<f64, SignalError> {
    if absorptivity <= 0.0 || path_length <= 0.0 {
        return Err(SignalError::NonPositiveParameter);
    }
    if absorbance < 0.0 {
        return Err(SignalError::NegativeAbsorbance);
    }
    Ok(absorbance / (absorptivity * path_length))
}

/// Calculate detection limit (minimum concentration).
///
/// c_min = A_min / (εl)
pub fn detection_limit(
    min_absorbance: f64,
    absorptivity: f64,
    path_length: f64,
) -> Result<f64, SignalError> {
    infer_concentration(min_absorbance, absorptivity, path_length)
}

/// Calculate transmittance from absorbance.
///
/// T = 10^(-A)
#[must_use]
pub fn transmittance(absorbance: f64) -> f64 {
    10.0_f64.powf(-absorbance)
}

/// Calculate absorbance from transmittance.
///
/// A = -log₁₀(T)
pub fn absorbance_from_transmittance(transmittance: f64) -> Result<f64, SignalError> {
    if transmittance <= 0.0 || transmittance > 1.0 {
        return Err(SignalError::NonPositiveParameter);
    }
    Ok(-transmittance.log10())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beer_lambert() {
        // ε = 100, l = 1, c = 0.01 -> A = 1.0
        let a = beer_lambert_absorbance(100.0, 1.0, 0.01).unwrap();
        assert!((a - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_infer_concentration() {
        let c = infer_concentration(1.0, 100.0, 1.0).unwrap();
        assert!((c - 0.01).abs() < 0.001);
    }

    #[test]
    fn test_transmittance_absorbance() {
        // A = 1 -> T = 0.1
        let t = transmittance(1.0);
        assert!((t - 0.1).abs() < 0.001);

        // T = 0.1 -> A = 1
        let a = absorbance_from_transmittance(0.1).unwrap();
        assert!((a - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_signal_detector() {
        let detector = SignalDetector::new(1000.0, 1.0, 0.01).unwrap();

        // c = 0.001 -> A = 1.0
        let a = detector.absorbance(0.001).unwrap();
        assert!((a - 1.0).abs() < 0.001);

        // A = 1.0 -> c = 0.001
        let c = detector.concentration(1.0).unwrap();
        assert!((c - 0.001).abs() < 0.0001);

        // Detection limit
        let min_c = detector.min_detectable_concentration();
        assert!((min_c - 0.00001).abs() < 0.000001);
    }

    #[test]
    fn test_detection_limit() {
        // min A = 0.01, ε = 1000, l = 1 -> c_min = 0.00001
        let c_min = detection_limit(0.01, 1000.0, 1.0).unwrap();
        assert!((c_min - 0.00001).abs() < 0.000001);
    }

    #[test]
    fn test_error_conditions() {
        assert!(beer_lambert_absorbance(-100.0, 1.0, 0.01).is_err());
        assert!(infer_concentration(-1.0, 100.0, 1.0).is_err());
        assert!(absorbance_from_transmittance(0.0).is_err());
        assert!(absorbance_from_transmittance(1.5).is_err());
    }
}
