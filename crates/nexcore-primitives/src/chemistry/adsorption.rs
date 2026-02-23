//! # Langmuir Isotherm (Resource Binding)
//!
//! **T1 Components**: state × ratio × maximum × quantity × saturation
//!
//! **Chemistry**: θ = K[A] / (1 + K[A])
//!
//! **Universal Pattern**: Surface coverage follows hyperbolic saturation.
//! Limited binding sites compete for adsorbate molecules.
//!
//! **PV Application**: Case handling capacity - finite reviewer slots
//! compete for incoming cases. Coverage = fraction of capacity utilized.
//!
//! **Bond Application**: Hook slot occupancy - limited concurrent bonds
//! mean new bonds must wait for slot availability.

use nexcore_error::Error;

/// Errors for adsorption calculations.
#[derive(Debug, Error, PartialEq, Clone)]
pub enum AdsorptionError {
    /// Concentration must be non-negative.
    #[error("Concentration must be non-negative")]
    NegativeConcentration,
    /// Equilibrium constant must be positive.
    #[error("Equilibrium constant must be positive")]
    InvalidEquilibrium,
    /// Coverage must be between 0 and 1.
    #[error("Coverage must be between 0 and 1")]
    InvalidCoverage,
    /// Maximum capacity must be positive.
    #[error("Maximum capacity must be positive")]
    InvalidCapacity,
}

/// Langmuir adsorption system configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct LangmuirIsotherm {
    /// Equilibrium constant K (affinity)
    pub k_eq: f64,
    /// Maximum adsorption capacity (Γmax or qmax)
    pub max_capacity: f64,
}

/// Multi-component Langmuir for competitive adsorption.
#[derive(Debug, Clone, PartialEq)]
pub struct CompetitiveLangmuir {
    /// Components: (name, K_eq, concentration)
    pub components: Vec<(String, f64, f64)>,
    /// Maximum capacity (shared)
    pub max_capacity: f64,
}

/// Coverage classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoverageState {
    /// θ < 0.1: Mostly empty (linear region)
    Linear,
    /// 0.1 ≤ θ < 0.5: Partial coverage
    Partial,
    /// 0.5 ≤ θ < 0.9: High coverage
    High,
    /// θ ≥ 0.9: Near saturation
    Saturated,
}

impl LangmuirIsotherm {
    /// Create new Langmuir isotherm model.
    pub fn new(k_eq: f64, max_capacity: f64) -> Result<Self, AdsorptionError> {
        if k_eq <= 0.0 {
            return Err(AdsorptionError::InvalidEquilibrium);
        }
        if max_capacity <= 0.0 {
            return Err(AdsorptionError::InvalidCapacity);
        }
        Ok(Self { k_eq, max_capacity })
    }

    /// Calculate fractional coverage (θ).
    ///
    /// θ = K[A] / (1 + K[A])
    pub fn coverage(&self, concentration: f64) -> Result<f64, AdsorptionError> {
        if concentration < 0.0 {
            return Err(AdsorptionError::NegativeConcentration);
        }
        Ok(langmuir_coverage(concentration, self.k_eq))
    }

    /// Calculate absolute amount adsorbed.
    ///
    /// q = Γmax × θ
    pub fn amount_adsorbed(&self, concentration: f64) -> Result<f64, AdsorptionError> {
        let theta = self.coverage(concentration)?;
        Ok(self.max_capacity * theta)
    }

    /// Calculate concentration for target coverage.
    ///
    /// [A] = θ / (K × (1 - θ))
    pub fn concentration_for_coverage(&self, theta: f64) -> Result<f64, AdsorptionError> {
        if !(0.0..1.0).contains(&theta) {
            return Err(AdsorptionError::InvalidCoverage);
        }
        Ok(theta / (self.k_eq * (1.0 - theta)))
    }

    /// Classify coverage state.
    #[must_use]
    pub fn classify(&self, concentration: f64) -> CoverageState {
        let theta = langmuir_coverage(concentration, self.k_eq);
        classify_coverage(theta)
    }

    /// Calculate remaining capacity.
    #[must_use]
    pub fn remaining_capacity(&self, concentration: f64) -> f64 {
        let theta = langmuir_coverage(concentration, self.k_eq);
        self.max_capacity * (1.0 - theta)
    }
}

impl CompetitiveLangmuir {
    /// Create new competitive adsorption model.
    pub fn new(max_capacity: f64) -> Result<Self, AdsorptionError> {
        if max_capacity <= 0.0 {
            return Err(AdsorptionError::InvalidCapacity);
        }
        Ok(Self {
            components: Vec::new(),
            max_capacity,
        })
    }

    /// Add a competing component.
    pub fn add_component(
        &mut self,
        name: &str,
        k_eq: f64,
        concentration: f64,
    ) -> Result<(), AdsorptionError> {
        if k_eq <= 0.0 {
            return Err(AdsorptionError::InvalidEquilibrium);
        }
        if concentration < 0.0 {
            return Err(AdsorptionError::NegativeConcentration);
        }
        self.components
            .push((name.to_string(), k_eq, concentration));
        Ok(())
    }

    /// Calculate coverage for each component.
    ///
    /// θᵢ = Kᵢ[Aᵢ] / (1 + Σ Kⱼ[Aⱼ])
    #[must_use]
    pub fn coverages(&self) -> Vec<(String, f64)> {
        let denominator: f64 = 1.0 + self.components.iter().map(|(_, k, c)| k * c).sum::<f64>();

        self.components
            .iter()
            .map(|(name, k, c)| (name.clone(), (k * c) / denominator))
            .collect()
    }

    /// Calculate total coverage.
    #[must_use]
    pub fn total_coverage(&self) -> f64 {
        self.coverages().iter().map(|(_, theta)| theta).sum()
    }
}

/// Calculate Langmuir coverage (fractional).
///
/// θ = K[A] / (1 + K[A])
///
/// # Arguments
/// * `concentration` - Adsorbate concentration [A]
/// * `k_eq` - Equilibrium constant K
///
/// # Returns
/// Fractional coverage (0.0 to 1.0)
#[must_use]
pub fn langmuir_coverage(concentration: f64, k_eq: f64) -> f64 {
    if concentration <= 0.0 || k_eq <= 0.0 {
        return 0.0;
    }
    let ka = k_eq * concentration;
    ka / (1.0 + ka)
}

/// Calculate amount adsorbed from coverage.
///
/// q = Γmax × θ
#[must_use]
pub fn amount_from_coverage(max_capacity: f64, coverage: f64) -> f64 {
    max_capacity * coverage.clamp(0.0, 1.0)
}

/// Calculate equilibrium constant from coverage data.
///
/// K = θ / ([A] × (1 - θ))
pub fn equilibrium_from_coverage(
    concentration: f64,
    coverage: f64,
) -> Result<f64, AdsorptionError> {
    if concentration <= 0.0 {
        return Err(AdsorptionError::NegativeConcentration);
    }
    if !(0.0..1.0).contains(&coverage) {
        return Err(AdsorptionError::InvalidCoverage);
    }
    Ok(coverage / (concentration * (1.0 - coverage)))
}

/// Classify coverage state.
#[must_use]
pub fn classify_coverage(theta: f64) -> CoverageState {
    if theta < 0.1 {
        CoverageState::Linear
    } else if theta < 0.5 {
        CoverageState::Partial
    } else if theta < 0.9 {
        CoverageState::High
    } else {
        CoverageState::Saturated
    }
}

/// Calculate Gibbs free energy of adsorption.
///
/// ΔG_ads = -RT × ln(K)
#[must_use]
pub fn adsorption_free_energy(k_eq: f64, temperature_k: f64) -> f64 {
    if k_eq <= 0.0 || temperature_k <= 0.0 {
        return 0.0;
    }
    -8.314 * temperature_k * k_eq.ln()
}

/// Calculate half-coverage concentration.
///
/// At θ = 0.5: [A]₀.₅ = 1/K
#[must_use]
pub fn half_coverage_concentration(k_eq: f64) -> f64 {
    if k_eq <= 0.0 {
        return f64::INFINITY;
    }
    1.0 / k_eq
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_langmuir_coverage_at_half() {
        // At [A] = 1/K, coverage should be 0.5
        let k = 10.0;
        let conc = 1.0 / k;
        let theta = langmuir_coverage(conc, k);
        assert!((theta - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_langmuir_coverage_limits() {
        let k = 1.0;
        // Low concentration -> low coverage
        assert!(langmuir_coverage(0.01, k) < 0.02);
        // High concentration -> near saturation
        assert!(langmuir_coverage(100.0, k) > 0.99);
    }

    #[test]
    fn test_langmuir_isotherm_struct() {
        let iso = LangmuirIsotherm::new(1.0, 100.0).unwrap();
        let theta = iso.coverage(1.0).unwrap();
        assert!((theta - 0.5).abs() < 0.001);

        let amount = iso.amount_adsorbed(1.0).unwrap();
        assert!((amount - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_concentration_for_coverage() {
        let iso = LangmuirIsotherm::new(2.0, 100.0).unwrap();
        // At θ = 0.5, [A] should be 1/K = 0.5
        let conc = iso.concentration_for_coverage(0.5).unwrap();
        assert!((conc - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_classify_coverage() {
        assert_eq!(classify_coverage(0.05), CoverageState::Linear);
        assert_eq!(classify_coverage(0.3), CoverageState::Partial);
        assert_eq!(classify_coverage(0.7), CoverageState::High);
        assert_eq!(classify_coverage(0.95), CoverageState::Saturated);
    }

    #[test]
    fn test_competitive_langmuir() {
        let mut comp = CompetitiveLangmuir::new(100.0).unwrap();
        comp.add_component("A", 1.0, 1.0).unwrap();
        comp.add_component("B", 2.0, 0.5).unwrap();

        let coverages = comp.coverages();
        assert_eq!(coverages.len(), 2);

        let total = comp.total_coverage();
        // 1*1 + 2*0.5 = 2, denom = 1 + 2 = 3
        // θA = 1/3, θB = 1/3, total = 2/3
        assert!((total - 2.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn test_equilibrium_from_coverage() {
        // If θ = 0.5 at [A] = 0.5, K should be 2
        let k = equilibrium_from_coverage(0.5, 0.5).unwrap();
        assert!((k - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_half_coverage_concentration() {
        let half = half_coverage_concentration(5.0);
        assert!((half - 0.2).abs() < 0.001);
    }

    #[test]
    fn test_adsorption_free_energy() {
        // Strong binding (K=1000) at 298K
        let delta_g = adsorption_free_energy(1000.0, 298.15);
        // Should be negative (favorable)
        assert!(delta_g < 0.0);
    }

    #[test]
    fn test_error_conditions() {
        assert_eq!(
            LangmuirIsotherm::new(-1.0, 100.0),
            Err(AdsorptionError::InvalidEquilibrium)
        );
        assert_eq!(
            LangmuirIsotherm::new(1.0, -100.0),
            Err(AdsorptionError::InvalidCapacity)
        );
    }
}
