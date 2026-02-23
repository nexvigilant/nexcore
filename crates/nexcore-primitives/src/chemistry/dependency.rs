//! # Dependency Propagation (Rate Laws)
//!
//! **T1 Components**: dependency × frequency × quantity × cause × effect
//!
//! **Chemistry**: rate = k[A]^n[B]^m (overall order = n + m)
//!
//! **Universal Pattern**: Output rate depends on input concentrations
//! raised to powers. Order determines sensitivity to each input.
//!
//! **PV Application**: Signal dependency - how inputs affect detection rate.

use nexcore_error::Error;
use std::cmp::Ordering;

/// Errors for rate law calculations.
#[derive(Debug, Error, PartialEq, Clone)]
pub enum DependencyError {
    /// Rate constant must be positive.
    #[error("Rate constant must be positive")]
    NonPositiveRateConstant,
    /// Concentrations must be non-negative.
    #[error("Concentrations must be non-negative")]
    NegativeConcentration,
    /// Must have at least one reactant.
    #[error("Must have at least one reactant")]
    NoReactants,
}

/// Rate law configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct RateLaw {
    /// Rate constant k
    pub rate_constant: f64,
    /// Reactant concentrations and orders: (concentration, order)
    pub reactants: Vec<(f64, f64)>,
}

impl RateLaw {
    /// Create new rate law.
    pub fn new(rate_constant: f64, reactants: Vec<(f64, f64)>) -> Result<Self, DependencyError> {
        if rate_constant <= 0.0 {
            return Err(DependencyError::NonPositiveRateConstant);
        }
        if reactants.is_empty() {
            return Err(DependencyError::NoReactants);
        }
        for (conc, _) in &reactants {
            if *conc < 0.0 {
                return Err(DependencyError::NegativeConcentration);
            }
        }
        Ok(Self {
            rate_constant,
            reactants,
        })
    }

    /// Calculate overall reaction order.
    pub fn overall_order(&self) -> f64 {
        self.reactants.iter().map(|(_, order)| order).sum()
    }

    /// Calculate rate.
    pub fn rate(&self) -> f64 {
        let mut rate = self.rate_constant;
        for (conc, order) in &self.reactants {
            rate *= conc.powf(*order);
        }
        rate
    }

    /// Find rate-limiting factor (lowest contribution).
    pub fn rate_limiting_factor(&self) -> Option<usize> {
        self.reactants
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                let contrib_a = a.0.powf(a.1);
                let contrib_b = b.0.powf(b.1);
                contrib_a.partial_cmp(&contrib_b).unwrap_or(Ordering::Equal)
            })
            .map(|(i, _)| i)
    }
}

/// Calculate rate using general rate law.
///
/// rate = k × [A]^n × [B]^m × ...
///
/// # Arguments
/// * `k` - Rate constant
/// * `reactants` - Slice of (concentration, order) tuples
pub fn calculate_rate_law(k: f64, reactants: &[(f64, f64)]) -> Result<f64, DependencyError> {
    if k <= 0.0 {
        return Err(DependencyError::NonPositiveRateConstant);
    }
    if reactants.is_empty() {
        return Err(DependencyError::NoReactants);
    }

    let mut rate = k;
    for (conc, order) in reactants {
        if *conc < 0.0 {
            return Err(DependencyError::NegativeConcentration);
        }
        rate *= conc.powf(*order);
    }
    Ok(rate)
}

/// Calculate first-order rate.
///
/// rate = k[A]
pub fn first_order_rate(k: f64, concentration: f64) -> Result<f64, DependencyError> {
    calculate_rate_law(k, &[(concentration, 1.0)])
}

/// Calculate second-order rate (same reactant).
///
/// rate = k[A]²
pub fn second_order_rate_same(k: f64, concentration: f64) -> Result<f64, DependencyError> {
    calculate_rate_law(k, &[(concentration, 2.0)])
}

/// Calculate second-order rate (different reactants).
///
/// rate = k[A][B]
pub fn second_order_rate_mixed(k: f64, conc_a: f64, conc_b: f64) -> Result<f64, DependencyError> {
    calculate_rate_law(k, &[(conc_a, 1.0), (conc_b, 1.0)])
}

/// Identify rate-limiting factor from reactant contributions.
///
/// Returns index of the reactant contributing least to the rate.
pub fn rate_limiting_factor(reactants: &[(f64, f64)]) -> Option<usize> {
    reactants
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            let contrib_a = a.0.powf(a.1);
            let contrib_b = b.0.powf(b.1);
            contrib_a.partial_cmp(&contrib_b).unwrap_or(Ordering::Equal)
        })
        .map(|(i, _)| i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_order() {
        // rate = 0.1 × 2.0 = 0.2
        let rate = first_order_rate(0.1, 2.0).unwrap();
        assert!((rate - 0.2).abs() < 0.001);
    }

    #[test]
    fn test_second_order_same() {
        // rate = 0.1 × 2.0² = 0.4
        let rate = second_order_rate_same(0.1, 2.0).unwrap();
        assert!((rate - 0.4).abs() < 0.001);
    }

    #[test]
    fn test_second_order_mixed() {
        // rate = 0.1 × 2.0 × 3.0 = 0.6
        let rate = second_order_rate_mixed(0.1, 2.0, 3.0).unwrap();
        assert!((rate - 0.6).abs() < 0.001);
    }

    #[test]
    fn test_rate_law_struct() {
        let law = RateLaw::new(0.1, vec![(2.0, 1.0), (3.0, 2.0)]).unwrap();
        // rate = 0.1 × 2.0^1 × 3.0^2 = 0.1 × 2 × 9 = 1.8
        assert!((law.rate() - 1.8).abs() < 0.001);
        assert!((law.overall_order() - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_rate_limiting_factor() {
        // (0.5, 1.0) → 0.5, (2.0, 1.0) → 2.0
        // First reactant is limiting
        let idx = rate_limiting_factor(&[(0.5, 1.0), (2.0, 1.0)]);
        assert_eq!(idx, Some(0));
    }

    #[test]
    fn test_error_conditions() {
        assert!(first_order_rate(-0.1, 2.0).is_err());
        assert!(first_order_rate(0.1, -2.0).is_err());
        assert!(calculate_rate_law(0.1, &[]).is_err());
    }
}
