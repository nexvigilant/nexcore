//! Entropy atoms (Law 10)

/// Calculate total entropy change (Law 10: Second Law of Thermodynamics).
///
/// ΔS_total = ΔS_system + ΔS_surroundings ≥ 0
///
/// # Arguments
/// * `delta_s_system` - Entropy change of the drug-receptor system
/// * `delta_s_surroundings` - Entropy change of the environment
#[must_use]
pub fn calculate_total_entropy_change(delta_s_system: f64, delta_s_surroundings: f64) -> f64 {
    delta_s_system + delta_s_surroundings
}
