//! # Feasibility Assessment (Gibbs Free Energy)
//!
//! **T1 Components**: quantity × comparison × state × effect
//!
//! **Chemistry**: ΔG = ΔH - TΔS
//!
//! **Universal Pattern**: Net favorability = direct benefit minus
//! (uncertainty × disorder). Spontaneous when ΔG < 0.
//!
//! **PV Application**: Causality likelihood - is relationship plausible?

use thiserror::Error;

/// Errors for feasibility calculations.
#[derive(Debug, Error, PartialEq, Clone)]
pub enum FeasibilityError {
    /// Temperature/uncertainty must be positive.
    #[error("Temperature/uncertainty scaling must be positive")]
    ScalingNotPositive,
}

/// Favorability classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Favorability {
    /// Always favorable (ΔH < 0, ΔS > 0)
    AlwaysFavorable,
    /// Never favorable (ΔH > 0, ΔS < 0)
    NeverFavorable,
    /// Favorable at low uncertainty (ΔH < 0, ΔS < 0)
    FavorableAtLowUncertainty,
    /// Favorable at high uncertainty (ΔH > 0, ΔS > 0)
    FavorableAtHighUncertainty,
}

/// Feasibility assessment configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct FeasibilityAssessment {
    /// Direct benefit/cost (ΔH - negative = beneficial)
    pub delta_h: f64,
    /// Disorder/complexity (ΔS)
    pub delta_s: f64,
    /// Uncertainty scaling (T)
    pub temperature: f64,
}

impl FeasibilityAssessment {
    /// Create new feasibility assessment.
    pub fn new(delta_h: f64, delta_s: f64, temperature: f64) -> Result<Self, FeasibilityError> {
        if temperature <= 0.0 {
            return Err(FeasibilityError::ScalingNotPositive);
        }
        Ok(Self {
            delta_h,
            delta_s,
            temperature,
        })
    }

    /// Calculate Gibbs free energy equivalent.
    ///
    /// ΔG = ΔH - TΔS
    pub fn delta_g(&self) -> f64 {
        self.delta_h - self.temperature * self.delta_s
    }

    /// Is this spontaneously favorable? (ΔG < 0)
    pub fn is_favorable(&self) -> bool {
        self.delta_g() < 0.0
    }

    /// Classify favorability based on thermodynamic profile.
    pub fn classify(&self) -> Favorability {
        match (self.delta_h < 0.0, self.delta_s > 0.0) {
            (true, true) => Favorability::AlwaysFavorable,
            (false, false) => Favorability::NeverFavorable,
            (true, false) => Favorability::FavorableAtLowUncertainty,
            (false, true) => Favorability::FavorableAtHighUncertainty,
        }
    }
}

/// Calculate Gibbs free energy.
///
/// ΔG = ΔH - TΔS
///
/// # Arguments
/// * `delta_h` - Enthalpy change (direct benefit, kJ/mol)
/// * `delta_s` - Entropy change (disorder, J/(mol·K))
/// * `temperature` - Temperature (uncertainty scaling, K)
///
/// Note: delta_s should be in J/(mol·K), so we divide by 1000 for kJ units.
pub fn gibbs_free_energy(
    delta_h: f64,
    delta_s: f64,
    temperature: f64,
) -> Result<f64, FeasibilityError> {
    if temperature <= 0.0 {
        return Err(FeasibilityError::ScalingNotPositive);
    }
    // Convert delta_s from J/(mol·K) to kJ/(mol·K) for consistent units
    Ok(delta_h - temperature * (delta_s / 1000.0))
}

/// Check if action is favorable (spontaneous).
///
/// Simple version using pre-calculated ΔG.
#[must_use]
pub fn is_favorable(delta_h: f64, delta_s: f64, temperature: f64) -> bool {
    if temperature <= 0.0 {
        return false;
    }
    let delta_g = delta_h - temperature * delta_s;
    delta_g < 0.0
}

/// Classify favorability based on ΔH and ΔS signs.
#[must_use]
pub fn classify_favorability(delta_h: f64, delta_s: f64) -> Favorability {
    match (delta_h < 0.0, delta_s > 0.0) {
        (true, true) => Favorability::AlwaysFavorable,
        (false, false) => Favorability::NeverFavorable,
        (true, false) => Favorability::FavorableAtLowUncertainty,
        (false, true) => Favorability::FavorableAtHighUncertainty,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gibbs_favorable() {
        // ΔH = -50 kJ/mol, ΔS = +100 J/(mol·K), T = 298K
        // ΔG = -50 - 298 * 0.1 = -50 - 29.8 = -79.8 (favorable)
        let dg = gibbs_free_energy(-50.0, 100.0, 298.0).unwrap();
        assert!(dg < 0.0);
    }

    #[test]
    fn test_gibbs_unfavorable() {
        // ΔH = +50, ΔS = -100 -> always unfavorable
        let dg = gibbs_free_energy(50.0, -100.0, 298.0).unwrap();
        assert!(dg > 0.0);
    }

    #[test]
    fn test_classify_always_favorable() {
        // -ΔH, +ΔS
        assert_eq!(
            classify_favorability(-10.0, 10.0),
            Favorability::AlwaysFavorable
        );
    }

    #[test]
    fn test_classify_never_favorable() {
        // +ΔH, -ΔS
        assert_eq!(
            classify_favorability(10.0, -10.0),
            Favorability::NeverFavorable
        );
    }

    #[test]
    fn test_assessment_struct() {
        // delta_h < 0, delta_s > 0 -> AlwaysFavorable
        let assessment = FeasibilityAssessment::new(-50.0, 0.1, 298.0).unwrap();
        assert!(assessment.is_favorable());
        assert_eq!(assessment.classify(), Favorability::AlwaysFavorable);

        // delta_h < 0, delta_s < 0 -> FavorableAtLowUncertainty
        let assessment2 = FeasibilityAssessment::new(-50.0, -0.1, 298.0).unwrap();
        assert_eq!(
            assessment2.classify(),
            Favorability::FavorableAtLowUncertainty
        );
    }
}
