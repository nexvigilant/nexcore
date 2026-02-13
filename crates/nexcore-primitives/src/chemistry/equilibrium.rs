//! # Equilibrium (Equilibrium Constant)
//!
//! **T1 Components**: state × persists × ratio × balance
//!
//! **Chemistry**: K = [C]^c[D]^d / [A]^a[B]^b
//!
//! **Universal Pattern**: Forward/reverse rates balance at steady state.
//! Equilibrium constant describes position of balance.
//!
//! **PV Application**: Reporting baseline - expected vs observed steady state.

use thiserror::Error;

/// Errors for equilibrium calculations.
#[derive(Debug, Error, PartialEq, Clone)]
pub enum EquilibriumError {
    /// Concentrations must be positive.
    #[error("Concentrations must be positive")]
    NonPositiveConcentration,
    /// Equilibrium constant must be positive.
    #[error("Equilibrium constant must be positive")]
    NonPositiveK,
    /// Rate constants must be positive.
    #[error("Rate constants must be positive")]
    NonPositiveRateConstant,
}

/// Equilibrium system configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct EquilibriumSystem {
    /// Forward rate constant (k_f)
    pub k_forward: f64,
    /// Reverse rate constant (k_r)
    pub k_reverse: f64,
}

impl EquilibriumSystem {
    /// Create new equilibrium system from rate constants.
    pub fn new(k_forward: f64, k_reverse: f64) -> Result<Self, EquilibriumError> {
        if k_forward <= 0.0 || k_reverse <= 0.0 {
            return Err(EquilibriumError::NonPositiveRateConstant);
        }
        Ok(Self {
            k_forward,
            k_reverse,
        })
    }

    /// Calculate equilibrium constant K = k_f / k_r.
    pub fn equilibrium_constant(&self) -> f64 {
        self.k_forward / self.k_reverse
    }

    /// Calculate time constant for equilibration (τ ≈ 1/(k_f + k_r)).
    pub fn time_constant(&self) -> f64 {
        1.0 / (self.k_forward + self.k_reverse)
    }

    /// Estimate time to reach x% of equilibrium.
    ///
    /// t = -τ × ln(1 - x)
    pub fn time_to_fraction(&self, fraction: f64) -> Option<f64> {
        if fraction <= 0.0 || fraction >= 1.0 {
            return None;
        }
        Some(-self.time_constant() * (1.0 - fraction).ln())
    }

    /// Calculate steady-state fractions [P]/[P]+[S] and [S]/[P]+[S].
    ///
    /// Returns (product_fraction, substrate_fraction).
    pub fn steady_state_fractions(&self) -> (f64, f64) {
        let k = self.equilibrium_constant();
        let product_frac = k / (1.0 + k);
        let substrate_frac = 1.0 / (1.0 + k);
        (product_frac, substrate_frac)
    }
}

/// Calculate equilibrium constant from concentrations.
///
/// K = [products] / [reactants] (simplified A ⇌ B)
pub fn equilibrium_constant(products: f64, reactants: f64) -> Result<f64, EquilibriumError> {
    if products <= 0.0 || reactants <= 0.0 {
        return Err(EquilibriumError::NonPositiveConcentration);
    }
    Ok(products / reactants)
}

/// Calculate equilibrium constant from rate constants.
///
/// K = k_forward / k_reverse
pub fn equilibrium_from_rates(k_forward: f64, k_reverse: f64) -> Result<f64, EquilibriumError> {
    if k_forward <= 0.0 || k_reverse <= 0.0 {
        return Err(EquilibriumError::NonPositiveRateConstant);
    }
    Ok(k_forward / k_reverse)
}

/// Calculate steady-state product fraction.
///
/// [P]_eq / ([P]_eq + [S]_eq) = K / (1 + K)
pub fn steady_state_fractions(k_eq: f64) -> Result<(f64, f64), EquilibriumError> {
    if k_eq <= 0.0 {
        return Err(EquilibriumError::NonPositiveK);
    }
    let product_frac = k_eq / (1.0 + k_eq);
    let substrate_frac = 1.0 / (1.0 + k_eq);
    Ok((product_frac, substrate_frac))
}

/// Estimate time to reach equilibrium (5τ ≈ 99%).
///
/// τ = 1 / (k_f + k_r)
pub fn time_to_equilibrium(k_forward: f64, k_reverse: f64) -> Result<f64, EquilibriumError> {
    if k_forward <= 0.0 || k_reverse <= 0.0 {
        return Err(EquilibriumError::NonPositiveRateConstant);
    }
    // 5 time constants ≈ 99.3% to equilibrium
    Ok(5.0 / (k_forward + k_reverse))
}

/// Check if system is at equilibrium (within tolerance).
///
/// Compares current ratio to equilibrium constant.
#[must_use]
pub fn is_at_equilibrium(products: f64, reactants: f64, k_eq: f64, tolerance: f64) -> bool {
    if reactants <= 0.0 {
        return false;
    }
    let current_ratio = products / reactants;
    (current_ratio - k_eq).abs() / k_eq <= tolerance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equilibrium_constant() {
        let k = equilibrium_constant(4.0, 2.0).unwrap();
        assert!((k - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_equilibrium_from_rates() {
        let k = equilibrium_from_rates(0.1, 0.05).unwrap();
        assert!((k - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_steady_state_fractions() {
        // K = 1 -> 50/50 split
        let (prod, sub) = steady_state_fractions(1.0).unwrap();
        assert!((prod - 0.5).abs() < 0.001);
        assert!((sub - 0.5).abs() < 0.001);

        // K = 9 -> 90/10 split
        let (prod, sub) = steady_state_fractions(9.0).unwrap();
        assert!((prod - 0.9).abs() < 0.001);
        assert!((sub - 0.1).abs() < 0.001);
    }

    #[test]
    fn test_time_to_equilibrium() {
        // k_f = 0.1, k_r = 0.1 -> τ = 5, t_eq ≈ 25
        let t = time_to_equilibrium(0.1, 0.1).unwrap();
        assert!((t - 25.0).abs() < 0.001);
    }

    #[test]
    fn test_equilibrium_system() {
        let sys = EquilibriumSystem::new(0.2, 0.1).unwrap();
        assert!((sys.equilibrium_constant() - 2.0).abs() < 0.001);

        let (prod, sub) = sys.steady_state_fractions();
        assert!((prod - 2.0 / 3.0).abs() < 0.001);
        assert!((sub - 1.0 / 3.0).abs() < 0.001);
    }

    #[test]
    fn test_is_at_equilibrium() {
        assert!(is_at_equilibrium(2.0, 1.0, 2.0, 0.01));
        assert!(!is_at_equilibrium(3.0, 1.0, 2.0, 0.01));
    }
}
