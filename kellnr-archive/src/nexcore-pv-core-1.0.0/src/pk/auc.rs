//! # Area Under Curve (AUC) Calculations
//!
//! AUC quantifies total drug exposure over time and is central to
//! Conservation Laws 1 (mass balance) and 7 (steady state).
//!
//! Key relationships:
//! - CL = Dose / AUC
//! - t_half = 0.693 * Vd / CL
//!
//! ## Horus Pattern Applied
//!
//! All validation occurs at the boundary (newtype construction).
//! Functions consuming validated types are infallible where possible.

use super::types::{Auc, Bioavailability, Clearance, Dose, HalfLife, TimeConcProfile, Volume};

/// Calculate AUC using linear trapezoidal method.
///
/// AUC = sum (C_1 + C_2)/2 * (t_2 - t_1)
///
/// # Arguments
///
/// * `profile` - Validated time-concentration profile
///
/// # Returns
///
/// AUC value (concentration * time units). Infallible since
/// all invariants are guaranteed by `TimeConcProfile`.
#[must_use]
pub fn calculate_auc_linear(profile: &TimeConcProfile) -> f64 {
    let points = profile.points();
    let mut auc = 0.0;
    for i in 0..points.len() - 1 {
        let (t1, c1) = points[i];
        let (t2, c2) = points[i + 1];
        let dt = t2 - t1;
        let avg_conc = (c1 + c2) / 2.0;
        auc += avg_conc * dt;
    }
    auc
}

/// Calculate AUC using log-linear trapezoidal method.
///
/// Preferred for elimination phase where concentrations decline
/// exponentially. Uses linear method for ascending portions.
///
/// # Arguments
///
/// * `profile` - Validated time-concentration profile
///
/// # Returns
///
/// AUC value (concentration * time units). Infallible.
#[must_use]
pub fn calculate_auc_log_linear(profile: &TimeConcProfile) -> f64 {
    let points = profile.points();
    let mut auc = 0.0;
    for i in 0..points.len() - 1 {
        let (t1, c1) = points[i];
        let (t2, c2) = points[i + 1];
        let dt = t2 - t1;

        if c1 <= 0.0 || c2 <= 0.0 || c2 >= c1 {
            // Use linear for ascending or zero concentrations
            auc += (c1 + c2) / 2.0 * dt;
        } else {
            // Log-linear for descending phase
            auc += (c1 - c2) / (c1 / c2).ln() * dt;
        }
    }
    auc
}

/// Calculate systemic clearance from AUC.
///
/// CL = (F * Dose) / AUC
///
/// # Arguments
///
/// * `dose` - Administered dose (validated non-negative)
/// * `auc` - Area under curve (validated positive)
/// * `bioavailability` - Fraction absorbed (validated 0.0-1.0)
///
/// # Returns
///
/// Clearance value. Infallible — all constraints guaranteed by newtypes.
#[must_use]
pub fn calculate_clearance_from_auc(
    dose: Dose,
    auc: Auc,
    bioavailability: Bioavailability,
) -> Clearance {
    let cl_value = (bioavailability.value() * dose.value()) / auc.value();
    // Clearance is always positive when F*D > 0 and AUC > 0.
    // Edge case: dose=0 → cl=0, which is not valid for Clearance (positive).
    // However, this is pharmacologically correct (no drug = no clearance to measure).
    // We saturate at a minimum positive value to maintain the Clearance invariant.
    if cl_value <= 0.0 {
        // Safety: Clearance::new won't fail for a positive value
        // When dose is 0, clearance is meaningless; return minimal positive.
        Clearance::new(f64::MIN_POSITIVE).unwrap_or_else(|_| {
            // This branch is unreachable but satisfies deny(unwrap_used)
            Clearance(f64::MIN_POSITIVE)
        })
    } else {
        Clearance::new(cl_value).unwrap_or_else(|_| {
            // Unreachable: cl_value is positive and finite (f64 / f64 where both finite, denom > 0)
            Clearance(cl_value)
        })
    }
}

/// Calculate elimination half-life from clearance and Vd.
///
/// t_half = 0.693 * Vd / CL
///
/// # Arguments
///
/// * `clearance` - Systemic clearance (validated positive)
/// * `volume_of_distribution` - Volume of distribution (validated positive)
///
/// # Returns
///
/// Elimination half-life. Infallible — both inputs are positive.
#[must_use]
pub fn calculate_half_life_from_clearance(
    clearance: Clearance,
    volume_of_distribution: Volume,
) -> HalfLife {
    let t_half = (0.693 * volume_of_distribution.value()) / clearance.value();
    HalfLife::new(t_half).unwrap_or_else(|_| {
        // Unreachable: 0.693 * positive / positive = positive finite
        HalfLife(t_half)
    })
}

#[cfg(test)]
mod tests {
    use super::super::types::PkError;
    use super::*;

    #[test]
    fn test_auc_linear_basic() {
        let profile = TimeConcProfile::new(vec![(0.0, 0.0), (1.0, 10.0), (2.0, 0.0)]);
        assert!(profile.is_ok());
        if let Ok(p) = profile {
            let auc = calculate_auc_linear(&p);
            assert!((auc - 10.0).abs() < 0.001);
        }
    }

    #[test]
    fn test_auc_linear_rectangle() {
        let profile = TimeConcProfile::new(vec![(0.0, 5.0), (1.0, 5.0), (2.0, 5.0)]);
        assert!(profile.is_ok());
        if let Ok(p) = profile {
            let auc = calculate_auc_linear(&p);
            assert!((auc - 10.0).abs() < 0.001);
        }
    }

    #[test]
    fn test_profile_length_mismatch_caught_at_boundary() {
        // This error is now caught at TimeConcProfile construction, not in AUC
        let result = TimeConcProfile::from_parallel_slices(&[0.0, 1.0], &[0.0]);
        assert!(result.is_err());
    }

    #[test]
    fn test_profile_time_not_ascending_caught_at_boundary() {
        let result = TimeConcProfile::new(vec![(0.0, 0.0), (2.0, 10.0), (1.0, 5.0)]);
        assert!(matches!(result, Err(PkError::TimeNotAscending)));
    }

    #[test]
    fn test_auc_log_linear_descending() {
        let profile = TimeConcProfile::new(vec![(0.0, 100.0), (1.0, 50.0)]);
        assert!(profile.is_ok());
        if let Ok(p) = profile {
            let auc_linear = calculate_auc_linear(&p);
            let auc_log = calculate_auc_log_linear(&p);
            assert!(auc_linear > 0.0);
            assert!(auc_log > 0.0);
            assert!((auc_linear - auc_log).abs() < 10.0);
        }
    }

    #[test]
    fn test_clearance_from_auc() {
        let dose = Dose::new(500.0);
        let auc = Auc::new(100.0);
        let bio = Bioavailability::new(1.0);
        assert!(dose.is_ok());
        assert!(auc.is_ok());
        assert!(bio.is_ok());
        if let (Ok(d), Ok(a), Ok(b)) = (dose, auc, bio) {
            let cl = calculate_clearance_from_auc(d, a, b);
            assert!((cl.value() - 5.0).abs() < 0.001);
        }
    }

    #[test]
    fn test_clearance_with_bioavailability() {
        let dose = Dose::new(500.0);
        let auc = Auc::new(50.0);
        let bio = Bioavailability::new(0.5);
        if let (Ok(d), Ok(a), Ok(b)) = (dose, auc, bio) {
            let cl = calculate_clearance_from_auc(d, a, b);
            assert!((cl.value() - 5.0).abs() < 0.001);
        }
    }

    #[test]
    fn test_bioavailability_invalid_caught_at_boundary() {
        let result = Bioavailability::new(1.5);
        assert!(matches!(result, Err(PkError::OutOfRange { .. })));
    }

    #[test]
    fn test_half_life_from_clearance() {
        let cl = Clearance::new(5.0);
        let vd = Volume::new(50.0);
        if let (Ok(c), Ok(v)) = (cl, vd) {
            let t_half = calculate_half_life_from_clearance(c, v);
            assert!((t_half.value() - 6.93).abs() < 0.01);
        }
    }

    #[test]
    fn test_clearance_zero_caught_at_boundary() {
        let result = Clearance::new(0.0);
        assert!(matches!(result, Err(PkError::NotPositive { .. })));
    }
}
