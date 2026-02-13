//! Ionization atoms (Law 8)

/// Calculate fraction of drug in unionized form (Law 8: Henderson-Hasselbalch).
///
/// For acids: f_unionized = 1 / (1 + 10^(pH - pKa))
/// For bases: f_unionized = 1 / (1 + 10^(pKa - pH))
///
/// # Arguments
/// * `pka` - Acid dissociation constant (pKa)
/// * `ph` - Environmental pH
/// * `is_acid` - True if drug is acidic, false if basic
#[must_use]
pub fn calculate_fraction_unionized(pka: f64, ph: f64, is_acid: bool) -> f64 {
    let ratio = if is_acid {
        10.0_f64.powf(ph - pka)
    } else {
        10.0_f64.powf(pka - ph)
    };
    1.0 / (1.0 + ratio)
}
