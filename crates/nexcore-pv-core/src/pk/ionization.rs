//! # Ionization State (Conservation Law 8)
//!
//! log([A-]/[HA]) = pH - pKa (Henderson-Hasselbalch equation)
//!
//! The ratio of ionized to un-ionized drug is determined by pH and pKa;
//! only un-ionized drug crosses membranes passively.

/// Calculate ratio of ionized to un-ionized drug.
///
/// For acids: [A-]/[HA] = 10^(pH - pKa)
/// For bases: [BH+]/[B] = 10^(pKa - pH)
#[must_use]
pub fn calculate_ionization_ratio(pka: f64, ph: f64, is_acid: bool) -> f64 {
    if is_acid {
        10.0_f64.powf(ph - pka)
    } else {
        10.0_f64.powf(pka - ph)
    }
}

/// Calculate fraction of drug in un-ionized (membrane-permeable) form.
///
/// For acids: fu = 1 / (1 + 10^(pH - pKa))
/// For bases: fu = 1 / (1 + 10^(pKa - pH))
#[must_use]
pub fn calculate_fraction_unionized(pka: f64, ph: f64, is_acid: bool) -> f64 {
    let ratio = calculate_ionization_ratio(pka, ph, is_acid);
    1.0 / (1.0 + ratio)
}

/// Calculate drug concentration ratio between compartments (pH partitioning).
///
/// Ion trapping: Drug accumulates in compartment where it is more ionized.
/// Returns ratio [Drug]1 / [Drug]2 at equilibrium.
#[must_use]
pub fn calculate_ph_partition(pka: f64, ph1: f64, ph2: f64, is_acid: bool) -> f64 {
    if is_acid {
        (1.0 + 10.0_f64.powf(ph2 - pka)) / (1.0 + 10.0_f64.powf(ph1 - pka))
    } else {
        (1.0 + 10.0_f64.powf(pka - ph2)) / (1.0 + 10.0_f64.powf(pka - ph1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ionization_acid() {
        // pH 7.4, pKa 4.4 (aspirin) -> ratio = 10^3 = 1000
        let ratio = calculate_ionization_ratio(4.4, 7.4, true);
        assert!((ratio - 1000.0).abs() < 0.1);
    }

    #[test]
    fn test_fraction_unionized() {
        // pH = pKa -> 50% ionized, 50% un-ionized
        let fu = calculate_fraction_unionized(7.0, 7.0, true);
        assert_eq!(fu, 0.5);
    }

    #[test]
    fn test_ph_partition() {
        // Weak acid, pKa 4, Compartment 1 (pH 7), Compartment 2 (pH 1)
        // Ratio [7] / [1] = (1 + 10^-3) / (1 + 10^3) ≈ 10^-3
        let ratio = calculate_ph_partition(4.0, 7.0, 1.0, true);
        assert!(ratio < 0.01);
    }
}
