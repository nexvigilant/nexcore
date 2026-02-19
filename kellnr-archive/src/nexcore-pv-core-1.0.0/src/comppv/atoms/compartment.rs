//! ADME Rate atoms (Law 6)

/// Calculate net rate of change in a compartment (Law 6).
///
/// dA/dt = Σ rates_in - Σ rates_out
///
/// # Arguments
/// * `rates_in` - All input rates to the compartment (absorption, distribution in)
/// * `rates_out` - All output rates from the compartment (elimination, distribution out)
#[must_use]
pub fn calculate_compartment_rate(rates_in: &[f64], rates_out: &[f64]) -> f64 {
    let sum_in: f64 = rates_in.iter().sum();
    let sum_out: f64 = rates_out.iter().sum();
    sum_in - sum_out
}
