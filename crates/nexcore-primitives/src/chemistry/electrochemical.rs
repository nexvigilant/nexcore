//! # Electrochemical Potential (Nernst Equation)
//!
//! **T1 Components**: threshold × state × transition × ratio × quantity
//!
//! **Chemistry**: E = E⁰ - (RT/nF)ln(Q)
//!
//! **Universal Pattern**: Potential shifts with concentration gradient.
//! Decision thresholds are concentration-dependent, not fixed.
//!
//! **PV Application**: Dynamic decision boundaries - the threshold for
//! signal detection shifts based on background rates.
//!
//! **Bond Application**: Bond activation threshold shifts based on
//! current system load (more bonds = harder to activate new ones).

use nexcore_error::Error;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Faraday constant (C/mol)
pub const FARADAY: f64 = 96_485.0;

/// Gas constant (J/(mol·K))
pub const GAS_CONSTANT: f64 = 8.314;

/// Errors for electrochemical calculations.
#[derive(Debug, Error, PartialEq, Clone)]
pub enum ElectrochemicalError {
    /// Temperature must be positive (Kelvin).
    #[error("Temperature must be positive (Kelvin)")]
    InvalidTemperature,
    /// Electron count must be positive.
    #[error("Number of electrons transferred must be positive")]
    InvalidElectronCount,
    /// Reaction quotient must be positive.
    #[error("Reaction quotient must be positive")]
    InvalidQuotient,
}

/// Electrochemical cell configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ElectrochemicalCell {
    /// Standard potential (E⁰) in volts
    pub e_standard: f64,
    /// Number of electrons transferred
    pub n_electrons: u32,
    /// Temperature in Kelvin
    pub temperature_k: f64,
}

/// Potential classification.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PotentialState {
    /// Well below threshold - action very unlikely
    FarBelowThreshold,
    /// Near threshold - sensitive region
    NearThreshold,
    /// Above threshold - action favorable
    AboveThreshold,
    /// Well above threshold - action highly favorable
    HighlyFavorable,
}

impl fmt::Display for PotentialState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FarBelowThreshold => write!(f, "far-below-threshold"),
            Self::NearThreshold => write!(f, "near-threshold"),
            Self::AboveThreshold => write!(f, "above-threshold"),
            Self::HighlyFavorable => write!(f, "highly-favorable"),
        }
    }
}

impl fmt::Display for ElectrochemicalCell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Cell(E\u{00B0}={:.3}V, n={})",
            self.e_standard, self.n_electrons
        )
    }
}

impl ElectrochemicalCell {
    /// Create new electrochemical cell model.
    pub fn new(
        e_standard: f64,
        n_electrons: u32,
        temperature_k: f64,
    ) -> Result<Self, ElectrochemicalError> {
        if temperature_k <= 0.0 {
            return Err(ElectrochemicalError::InvalidTemperature);
        }
        if n_electrons == 0 {
            return Err(ElectrochemicalError::InvalidElectronCount);
        }
        Ok(Self {
            e_standard,
            n_electrons,
            temperature_k,
        })
    }

    /// Calculate cell potential at given reaction quotient.
    pub fn potential(&self, q: f64) -> Result<f64, ElectrochemicalError> {
        if q <= 0.0 {
            return Err(ElectrochemicalError::InvalidQuotient);
        }
        Ok(nernst_potential(
            self.e_standard,
            self.temperature_k,
            self.n_electrons as f64,
            q,
        ))
    }

    /// Calculate the Nernst factor (RT/nF).
    #[must_use]
    pub fn nernst_factor(&self) -> f64 {
        (GAS_CONSTANT * self.temperature_k) / (self.n_electrons as f64 * FARADAY)
    }

    /// Calculate Q needed for target potential.
    ///
    /// Q = exp((E⁰ - E) × nF / RT)
    pub fn quotient_for_potential(&self, target_e: f64) -> f64 {
        let exponent = (self.e_standard - target_e) * self.n_electrons as f64 * FARADAY
            / (GAS_CONSTANT * self.temperature_k);
        exponent.exp()
    }

    /// Classify potential state relative to standard.
    #[must_use]
    pub fn classify(&self, q: f64) -> PotentialState {
        if let Ok(e) = self.potential(q) {
            let delta = e - self.e_standard;
            if delta < -0.1 {
                PotentialState::FarBelowThreshold
            } else if delta < 0.0 {
                PotentialState::NearThreshold
            } else if delta < 0.1 {
                PotentialState::AboveThreshold
            } else {
                PotentialState::HighlyFavorable
            }
        } else {
            PotentialState::FarBelowThreshold
        }
    }
}

/// Calculate Nernst equation potential.
///
/// E = E⁰ - (RT/nF)ln(Q)
///
/// # Arguments
/// * `e_standard` - Standard cell potential (V)
/// * `temperature_k` - Temperature (Kelvin)
/// * `n_electrons` - Number of electrons transferred
/// * `q` - Reaction quotient [products]/[reactants]
///
/// # Returns
/// Cell potential in volts
#[must_use]
pub fn nernst_potential(e_standard: f64, temperature_k: f64, n_electrons: f64, q: f64) -> f64 {
    if temperature_k <= 0.0 || n_electrons <= 0.0 || q <= 0.0 {
        return e_standard;
    }
    let rt_nf = (GAS_CONSTANT * temperature_k) / (n_electrons * FARADAY);
    e_standard - rt_nf * q.ln()
}

/// Calculate Nernst factor at standard conditions (298K).
///
/// At 298K with n=1: RT/nF ≈ 0.0257 V
/// Commonly expressed as 59.2 mV per decade (log₁₀)
#[must_use]
pub fn nernst_factor_standard(n_electrons: f64) -> f64 {
    (GAS_CONSTANT * 298.15) / (n_electrons * FARADAY)
}

/// Calculate potential shift per decade of concentration change.
///
/// ΔE = (59.2 mV / n) per 10× change in Q at 298K
#[must_use]
pub fn millivolts_per_decade(n_electrons: f64) -> f64 {
    // 2.303 × RT/nF × 1000 = 59.2 mV/n at 298K
    2.303 * nernst_factor_standard(n_electrons) * 1000.0
}

/// Calculate dynamic threshold based on background.
///
/// Applies Nernst-like shift to decision threshold based on
/// ratio of current signal to background.
///
/// # Arguments
/// * `base_threshold` - Standard threshold (no background)
/// * `signal` - Current signal level
/// * `background` - Background level
/// * `sensitivity` - How much threshold shifts (like n_electrons)
#[must_use]
pub fn dynamic_threshold(
    base_threshold: f64,
    signal: f64,
    background: f64,
    sensitivity: f64,
) -> f64 {
    if signal <= 0.0 || background <= 0.0 || sensitivity <= 0.0 {
        return base_threshold;
    }
    let ratio = signal / background;
    // Nernst-like: threshold shifts down when signal/background is high
    base_threshold - (1.0 / sensitivity) * ratio.ln()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nernst_at_equilibrium() {
        // At Q = 1, E = E⁰ (ln(1) = 0)
        let e = nernst_potential(1.1, 298.15, 2.0, 1.0);
        assert!((e - 1.1).abs() < 0.001);
    }

    #[test]
    fn test_nernst_low_q() {
        // Low Q (more reactants) -> higher potential
        let e_low_q = nernst_potential(1.1, 298.15, 2.0, 0.01);
        let e_high_q = nernst_potential(1.1, 298.15, 2.0, 100.0);
        assert!(e_low_q > e_high_q);
    }

    #[test]
    fn test_nernst_factor_standard() {
        // At 298K, n=1: RT/nF ≈ 0.0257 V
        let factor = nernst_factor_standard(1.0);
        assert!((factor - 0.0257).abs() < 0.001);
    }

    #[test]
    fn test_millivolts_per_decade() {
        // At n=1: ~59.2 mV per decade
        let mv = millivolts_per_decade(1.0);
        assert!((mv - 59.2).abs() < 0.5);
    }

    #[test]
    fn test_electrochemical_cell() {
        let cell = ElectrochemicalCell::new(1.1, 2, 298.15).unwrap();
        let e = cell.potential(0.01).unwrap();
        // Low Q -> higher E (more favorable)
        assert!(e > 1.1);
    }

    #[test]
    fn test_quotient_for_potential() {
        let cell = ElectrochemicalCell::new(1.1, 2, 298.15).unwrap();
        // At E = E⁰, Q should be 1
        let q = cell.quotient_for_potential(1.1);
        assert!((q - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_dynamic_threshold() {
        let base = 2.0;
        // High signal/background -> lower threshold
        let thresh_high = dynamic_threshold(base, 10.0, 1.0, 1.0);
        let thresh_low = dynamic_threshold(base, 1.0, 10.0, 1.0);
        assert!(thresh_high < base);
        assert!(thresh_low > base);
    }
}
