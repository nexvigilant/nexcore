//! # Buffer Stability (Henderson-Hasselbalch)
//!
//! **T1 Components**: state × ratio × persists × effect
//!
//! **Chemistry**: pH = pKa + log([A⁻]/[HA])
//!
//! **Universal Pattern**: System resists perturbation around setpoint.
//! Buffer capacity determines how much stress system can absorb.
//!
//! **PV Application**: Baseline stability - reporting rates resist random fluctuation.

use thiserror::Error;

/// Errors for buffer calculations.
#[derive(Debug, Error, PartialEq, Clone)]
pub enum BufferError {
    /// pKa must be in valid range.
    #[error("pKa must be in range 0-14")]
    InvalidPka,
    /// Concentrations must be positive.
    #[error("Concentrations must be positive")]
    NonPositiveConcentration,
    /// Ratio must be positive.
    #[error("Ratio must be positive")]
    NonPositiveRatio,
}

/// Buffer system configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct BufferSystem {
    /// Dissociation constant (pKa - setpoint)
    pub pka: f64,
    /// Base concentration [A⁻]
    pub base_conc: f64,
    /// Acid concentration [HA]
    pub acid_conc: f64,
}

impl BufferSystem {
    /// Create new buffer system.
    pub fn new(pka: f64, base_conc: f64, acid_conc: f64) -> Result<Self, BufferError> {
        if !(0.0..=14.0).contains(&pka) {
            return Err(BufferError::InvalidPka);
        }
        if base_conc <= 0.0 || acid_conc <= 0.0 {
            return Err(BufferError::NonPositiveConcentration);
        }
        Ok(Self {
            pka,
            base_conc,
            acid_conc,
        })
    }

    /// Calculate current pH.
    pub fn ph(&self) -> f64 {
        self.pka + (self.base_conc / self.acid_conc).log10()
    }

    /// Calculate ionization ratio [A⁻]/[HA].
    pub fn ionization_ratio(&self) -> f64 {
        self.base_conc / self.acid_conc
    }

    /// Calculate buffer capacity (β).
    ///
    /// Higher capacity = more resistant to pH change.
    pub fn buffer_capacity(&self) -> f64 {
        let total = self.base_conc + self.acid_conc;
        let ratio = self.base_conc / self.acid_conc;
        2.303 * total * ratio / (1.0 + ratio).powi(2)
    }

    /// Check if system is well-buffered (ratio near 1:1).
    pub fn is_well_buffered(&self) -> bool {
        let ratio = self.ionization_ratio();
        ratio >= 0.1 && ratio <= 10.0
    }
}

/// Calculate pH using Henderson-Hasselbalch equation.
///
/// pH = pKa + log([A⁻]/[HA])
///
/// # Arguments
/// * `pka` - Dissociation constant (setpoint)
/// * `base_conc` - Conjugate base concentration [A⁻]
/// * `acid_conc` - Weak acid concentration [HA]
pub fn henderson_hasselbalch(pka: f64, base_conc: f64, acid_conc: f64) -> Result<f64, BufferError> {
    if base_conc <= 0.0 || acid_conc <= 0.0 {
        return Err(BufferError::NonPositiveConcentration);
    }
    Ok(pka + (base_conc / acid_conc).log10())
}

/// Calculate ionization ratio from pH and pKa.
///
/// [A⁻]/[HA] = 10^(pH - pKa)
pub fn ionization_ratio(ph: f64, pka: f64) -> f64 {
    10.0_f64.powf(ph - pka)
}

/// Calculate buffer capacity.
///
/// β = 2.303 × C × [A⁻][HA] / ([A⁻] + [HA])²
///
/// # Arguments
/// * `total_conc` - Total buffer concentration (C)
/// * `ratio` - [A⁻]/[HA] ratio
pub fn buffer_capacity(total_conc: f64, ratio: f64) -> Result<f64, BufferError> {
    if total_conc <= 0.0 {
        return Err(BufferError::NonPositiveConcentration);
    }
    if ratio <= 0.0 {
        return Err(BufferError::NonPositiveRatio);
    }
    Ok(2.303 * total_conc * ratio / (1.0 + ratio).powi(2))
}

/// Check if buffer is in optimal range (ratio 0.1 to 10).
///
/// Best buffering occurs when pH ≈ pKa ± 1.
#[must_use]
pub fn is_optimal_buffer(ratio: f64) -> bool {
    ratio >= 0.1 && ratio <= 10.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_henderson_hasselbalch() {
        // pKa = 7.0, equal concentrations -> pH = pKa
        let ph = henderson_hasselbalch(7.0, 1.0, 1.0).unwrap();
        assert!((ph - 7.0).abs() < 0.001);
    }

    #[test]
    fn test_ionization_ratio() {
        // pH = pKa -> ratio = 1.0
        let ratio = ionization_ratio(7.0, 7.0);
        assert!((ratio - 1.0).abs() < 0.001);

        // pH = pKa + 1 -> ratio = 10.0
        let ratio = ionization_ratio(8.0, 7.0);
        assert!((ratio - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_buffer_capacity_max() {
        // Maximum capacity at ratio = 1.0
        let cap_1 = buffer_capacity(1.0, 1.0).unwrap();
        let cap_01 = buffer_capacity(1.0, 0.1).unwrap();
        let cap_10 = buffer_capacity(1.0, 10.0).unwrap();
        assert!(cap_1 > cap_01);
        assert!(cap_1 > cap_10);
    }

    #[test]
    fn test_buffer_system() {
        let buffer = BufferSystem::new(7.4, 0.024, 0.024).unwrap();
        assert!((buffer.ph() - 7.4).abs() < 0.01);
        assert!(buffer.is_well_buffered());
    }

    #[test]
    fn test_optimal_buffer() {
        assert!(is_optimal_buffer(1.0));
        assert!(is_optimal_buffer(0.1));
        assert!(is_optimal_buffer(10.0));
        assert!(!is_optimal_buffer(0.05));
        assert!(!is_optimal_buffer(20.0));
    }
}
