//! # Drug Mass Balance (Conservation Law 1)
//!
//! D(t) = D_0 - integral_0^t CL * C(tau) dtau
//!
//! The total mass of drug in the body at any time equals the administered
//! dose minus the cumulative amount eliminated.
//!
//! ## Horus Pattern Applied
//!
//! `Dose`, `Clearance`, and `TimeConcProfile` newtypes eliminate
//! negative-value and ordering checks from function bodies.

use serde::{Deserialize, Serialize};

use super::types::{Clearance, Dose, PkError, TimeConcProfile};

/// Result of a mass balance verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MassBalanceResult {
    /// Whether Conservation Law 1 is satisfied within tolerance.
    pub is_satisfied: bool,
    /// The fraction by which mass balance deviates.
    pub deviation_fraction: f64,
}

/// Calculate remaining drug in body per Conservation Law 1.
///
/// # Arguments
///
/// * `initial_dose` - Validated non-negative dose
/// * `cumulative_elimination` - Validated non-negative elimination amount
///
/// # Errors
///
/// Returns `PkError::MassViolation` if eliminated exceeds initial dose.
pub fn calculate_remaining_drug(
    initial_dose: Dose,
    cumulative_elimination: Dose,
) -> Result<Dose, PkError> {
    let remaining = initial_dose.value() - cumulative_elimination.value();
    if remaining < 0.0 {
        return Err(PkError::MassViolation {
            eliminated: cumulative_elimination.value(),
            dose: initial_dose.value(),
        });
    }
    // remaining >= 0.0 and finite (difference of two finite non-negative values)
    Ok(Dose(remaining))
}

/// Calculate cumulative drug eliminated using trapezoidal integration.
///
/// # Arguments
///
/// * `clearance` - Validated positive clearance
/// * `profile` - Validated time-concentration profile (sorted, ascending)
///
/// # Returns
///
/// Cumulative elimination amount. Infallible — inputs are pre-validated.
#[must_use]
pub fn calculate_cumulative_elimination(clearance: Clearance, profile: &TimeConcProfile) -> f64 {
    let points = profile.points();
    let mut total_auc = 0.0;
    for i in 0..points.len() - 1 {
        let (t1, c1) = points[i];
        let (t2, c2) = points[i + 1];
        total_auc += (c1 + c2) / 2.0 * (t2 - t1);
    }
    clearance.value() * total_auc
}

/// Verify Conservation Law 1 is satisfied within tolerance.
///
/// This is a diagnostic function using raw f64 values from measurement data.
/// No newtype wrapping needed — this is an observation, not a computation input.
pub fn check_mass_balance(
    initial_dose: f64,
    amount_in_body: f64,
    amount_eliminated: f64,
    tolerance: f64,
) -> MassBalanceResult {
    if initial_dose <= 0.0 {
        return MassBalanceResult {
            is_satisfied: true,
            deviation_fraction: 0.0,
        };
    }
    let expected_in_body = initial_dose - amount_eliminated;
    let actual_in_body = amount_in_body;
    let deviation = (expected_in_body - actual_in_body).abs();
    let deviation_fraction = deviation / initial_dose;

    MassBalanceResult {
        is_satisfied: deviation_fraction <= tolerance,
        deviation_fraction,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remaining_drug() {
        let initial = Dose::new(100.0);
        let eliminated = Dose::new(40.0);
        assert!(initial.is_ok());
        assert!(eliminated.is_ok());
        if let (Ok(i), Ok(e)) = (initial, eliminated) {
            let result = calculate_remaining_drug(i, e);
            assert!(result.is_ok());
            if let Ok(remaining) = result {
                assert!((remaining.value() - 60.0).abs() < f64::EPSILON);
            }
        }
    }

    #[test]
    fn test_mass_violation() {
        let initial = Dose::new(100.0);
        let eliminated = Dose::new(110.0);
        if let (Ok(i), Ok(e)) = (initial, eliminated) {
            let result = calculate_remaining_drug(i, e);
            assert!(matches!(result, Err(PkError::MassViolation { .. })));
        }
    }

    #[test]
    fn test_cumulative_elimination() {
        let cl = Clearance::new(5.0);
        let profile = TimeConcProfile::new(vec![(0.0, 10.0), (1.0, 10.0)]);
        assert!(cl.is_ok());
        assert!(profile.is_ok());
        if let (Ok(c), Ok(p)) = (cl, profile) {
            // AUC = 10.0, Elim = 5.0 * 10.0 = 50.0
            let elim = calculate_cumulative_elimination(c, &p);
            assert!((elim - 50.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_check_mass_balance() {
        let res = check_mass_balance(100.0, 50.0, 50.0, 0.05);
        assert!(res.is_satisfied);

        let res = check_mass_balance(100.0, 40.0, 50.0, 0.05);
        assert!(!res.is_satisfied); // 10% deviation > 5%
    }

    #[test]
    fn test_dose_negative_caught_at_boundary() {
        let result = Dose::new(-1.0);
        assert!(matches!(result, Err(PkError::Negative { .. })));
    }
}
