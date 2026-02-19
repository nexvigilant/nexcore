//! Enzyme Regeneration atoms (Law 5)

/// Calculate enzyme rate of change under inhibition (Law 5).
///
/// dE/dt = k_syn - k_deg·E - k_inact·[I]·E
///
/// # Arguments
/// * `k_syn` - Synthesis rate constant
/// * `k_deg` - Degradation rate constant
/// * `k_inact` - Inactivation rate constant
/// * `inhibitor_conc` - Inhibitor concentration [I]
/// * `total_enzyme` - Total enzyme concentration [E]
#[must_use]
pub fn calculate_enzyme_rate_of_change(
    k_syn: f64,
    k_deg: f64,
    k_inact: f64,
    inhibitor_conc: f64,
    total_enzyme: f64,
) -> f64 {
    k_syn - (k_deg * total_enzyme) - (k_inact * inhibitor_conc * total_enzyme)
}
