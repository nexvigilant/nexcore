//! Inlined chemistry primitives — Beer-Lambert, Hill, Arrhenius
//!
//! Tier: T2-P | Primitives: Σ Sum, ρ Recursion, ∂ Boundary, → Causality

/// Beer-Lambert weighted sum: A = Σ(ε_i × feature_i)
#[must_use]
pub fn beer_lambert_weighted_sum(weights: &[f64], features: &[f64]) -> f64 {
    weights
        .iter()
        .zip(features.iter())
        .map(|(w, f)| w * f)
        .sum()
}

/// Hill cooperative amplification: Y = x^nH / (K^nH + x^nH)
#[must_use]
pub fn hill_amplify(x: f64, k_half: f64, n_hill: f64) -> f64 {
    if x <= 0.0 || k_half <= 0.0 || n_hill <= 0.0 {
        return 0.0;
    }
    let x_n = x.powf(n_hill);
    let k_n = k_half.powf(n_hill);
    x_n / (k_n + x_n)
}

/// Arrhenius activation probability: p = exp(-Ea / (score × scale))
#[must_use]
pub fn arrhenius_probability(activation_energy: f64, score: f64, scale: f64) -> f64 {
    let effective = score * scale;
    if effective <= 0.0 {
        return 0.0;
    }
    let raw = (-activation_energy / effective).exp();
    raw.min(1.0)
}
