//! Genetic atoms (Law 11)

/// Calculate sequence homology ratio (Law 11: Genetic Conservation).
///
/// Measures the fraction of positions that are identical between sequences.
/// High homology indicates sequence conservation (expected unless mutagenic).
///
/// # Arguments
/// * `sequence_before` - Original sequence
/// * `sequence_after` - Sequence after drug exposure
#[must_use]
pub fn calculate_sequence_homology(sequence_before: &str, sequence_after: &str) -> f64 {
    if sequence_before.is_empty() || sequence_after.is_empty() {
        return if sequence_before == sequence_after {
            1.0
        } else {
            0.0
        };
    }

    let matches = sequence_before
        .chars()
        .zip(sequence_after.chars())
        .filter(|(a, b)| a == b)
        .count();

    matches as f64 / sequence_before.len().max(sequence_after.len()) as f64
}
