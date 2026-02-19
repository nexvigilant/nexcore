//! Pathway Flux atoms (Law 4)

/// Calculate net flux at a metabolic pathway node (Law 4).
///
/// J_net = Σ J_in - Σ J_out
///
/// At steady state, J_net = 0 for each node.
///
/// # Arguments
/// * `fluxes_in` - All incoming fluxes to the node
/// * `fluxes_out` - All outgoing fluxes from the node
#[must_use]
pub fn calculate_node_net_flux(fluxes_in: &[f64], fluxes_out: &[f64]) -> f64 {
    let sum_in: f64 = fluxes_in.iter().sum();
    let sum_out: f64 = fluxes_out.iter().sum();
    sum_in - sum_out
}
