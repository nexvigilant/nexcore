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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beer_lambert_basic() {
        let weights = [0.5, 0.5];
        let features = [0.8, 0.6];
        let result = beer_lambert_weighted_sum(&weights, &features);
        assert!((result - 0.7).abs() < 1e-10);
    }

    #[test]
    fn beer_lambert_empty() {
        assert_eq!(beer_lambert_weighted_sum(&[], &[]), 0.0);
    }

    #[test]
    fn hill_at_midpoint() {
        // At x = k_half, Hill should return 0.5
        let result = hill_amplify(0.5, 0.5, 2.0);
        assert!((result - 0.5).abs() < 1e-10);
    }

    #[test]
    fn hill_zero_input() {
        assert_eq!(hill_amplify(0.0, 0.5, 2.0), 0.0);
    }

    #[test]
    fn hill_negative_input() {
        assert_eq!(hill_amplify(-1.0, 0.5, 2.0), 0.0);
    }

    #[test]
    fn hill_approaches_one() {
        let result = hill_amplify(100.0, 0.5, 2.0);
        assert!(result > 0.99);
    }

    #[test]
    fn arrhenius_zero_score() {
        assert_eq!(arrhenius_probability(3.0, 0.0, 10.0), 0.0);
    }

    #[test]
    fn arrhenius_high_score() {
        let result = arrhenius_probability(3.0, 1.0, 10.0);
        assert!(result > 0.5);
        assert!(result <= 1.0);
    }

    #[test]
    fn arrhenius_bounded_by_one() {
        let result = arrhenius_probability(0.001, 100.0, 100.0);
        assert!(result <= 1.0);
    }
}
