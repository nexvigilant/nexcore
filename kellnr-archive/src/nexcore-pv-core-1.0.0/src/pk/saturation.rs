//! # Saturation Kinetics (Conservation Law 9)
//!
//! v = Vmax * [S] / (Km + [S]) (Michaelis-Menten)
//!
//! Biological processes with finite capacity exhibit saturable kinetics;
//! exceeding capacity causes non-linear behavior.
//!
//! ## Horus Pattern Applied
//!
//! `Concentration`, `Vmax`, `Km`, `Kd` newtypes eliminate
//! negative/zero parameter checks from function bodies.
//! All four functions are now infallible.

use super::types::{Concentration, Kd, Km, Vmax};

/// Calculate reaction rate using Michaelis-Menten kinetics.
///
/// v = Vmax * [S] / (Km + [S])
///
/// # Arguments
///
/// * `substrate_conc` - Substrate concentration (validated non-negative)
/// * `vmax` - Maximum velocity (validated non-negative)
/// * `km` - Michaelis constant (validated positive, > 0)
///
/// # Returns
///
/// Reaction rate. Infallible — all parameters pre-validated.
#[must_use]
pub fn calculate_michaelis_menten_rate(substrate_conc: Concentration, vmax: Vmax, km: Km) -> f64 {
    // Km > 0 guaranteed by newtype, so no division-by-zero possible
    (vmax.value() * substrate_conc.value()) / (km.value() + substrate_conc.value())
}

/// Calculate fraction of receptors occupied by drug.
///
/// Occupancy = [Drug] / (Kd + [Drug])
///
/// # Arguments
///
/// * `drug_conc` - Drug concentration (validated non-negative)
/// * `kd` - Dissociation constant (validated positive)
///
/// # Returns
///
/// Fractional occupancy [0.0, 1.0]. Infallible.
#[must_use]
pub fn calculate_receptor_occupancy(drug_conc: Concentration, kd: Kd) -> f64 {
    drug_conc.value() / (kd.value() + drug_conc.value())
}

/// Calculate generic saturation fraction.
///
/// Fraction = [C] / (Km + [C])
///
/// # Arguments
///
/// * `concentration` - Validated non-negative
/// * `half_saturation` - Km (validated positive)
///
/// # Returns
///
/// Saturation fraction [0.0, 1.0). Infallible.
#[must_use]
pub fn calculate_saturation_fraction(concentration: Concentration, half_saturation: Km) -> f64 {
    concentration.value() / (half_saturation.value() + concentration.value())
}

/// Check if kinetics are effectively linear (first-order).
///
/// Linear kinetics occur when [S] << Km (typically [S] < threshold * Km).
///
/// # Arguments
///
/// * `concentration` - Substrate concentration (validated non-negative)
/// * `km` - Michaelis constant (validated positive)
/// * `threshold` - Linearity threshold (typically 0.1)
#[must_use]
pub fn is_linear_kinetics(concentration: Concentration, km: Km, threshold: f64) -> bool {
    // km > 0 guaranteed
    concentration.value() < (threshold * km.value())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_michaelis_menten() {
        let conc = Concentration::new(10.0);
        let vmax = Vmax::new(100.0);
        let km = Km::new(10.0);
        if let (Ok(c), Ok(v), Ok(k)) = (conc, vmax, km) {
            // v = (100 * 10) / (10 + 10) = 1000 / 20 = 50
            let rate = calculate_michaelis_menten_rate(c, v, k);
            assert!((rate - 50.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_receptor_occupancy() {
        let conc = Concentration::new(5.0);
        let kd = Kd::new(5.0);
        if let (Ok(c), Ok(k)) = (conc, kd) {
            // Occupancy = 5 / (5 + 5) = 0.5
            let occ = calculate_receptor_occupancy(c, k);
            assert!((occ - 0.5).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_is_linear() {
        let low = Concentration::new(1.0);
        let high = Concentration::new(20.0);
        let km = Km::new(100.0);
        if let (Ok(l), Ok(h), Ok(k)) = (low, high, km) {
            assert!(is_linear_kinetics(l, k, 0.1)); // 1 < 0.1*100 = 10
            assert!(!is_linear_kinetics(h, k, 0.1)); // 20 > 10
        }
    }

    #[test]
    fn test_negative_concentration_caught_at_boundary() {
        let result = Concentration::new(-1.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_zero_km_caught_at_boundary() {
        let result = Km::new(0.0);
        assert!(result.is_err());
    }
}
