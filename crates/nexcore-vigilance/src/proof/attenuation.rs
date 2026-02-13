//! Attenuation Theorem (T10.2) Implementation
//!
//! This module provides the implementation of the Attenuation Theorem
//! from the Theory of Vigilance (§10.2).
//!
//! ## Theorem Statement
//!
//! Under the Markov assumption (Axiom 5), if all propagation probabilities P_{i→i+1} < 1,
//! then the harm probability at level H is:
//!
//! **ℙ(H|δs₁) = e^{-α(H-1)}**
//!
//! where **α = -log(geometric mean of propagation probabilities)**
//!
//! ## Versions
//!
//! | Version | Formula | Description |
//! |---------|---------|-------------|
//! | A | ℙ(H) ≤ P_max^{H-1} | Uniform bound |
//! | B | ℙ(H) = ∏ᵢ P_{i→i+1} | Product form |
//! | C | ℙ(H) = (P̄)^{H-1} | Geometric mean form |
//! | D | ℙ(H) = e^{-α(H-1)} | Exponential decay |
//!
//! ## Corollary: Protective Depth
//!
//! For target probability ε:
//! **H ≥ 1 + log(1/ε)/α**
//!
//! This tells us how many hierarchy levels provide sufficient protection.

use serde::{Deserialize, Serialize};

/// Propagation probability (must be in (0, 1)).
///
/// Represents P_{i→i+1}, the probability that a perturbation at level i
/// propagates to level i+1.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PropagationProbability {
    value: f64,
}

impl PropagationProbability {
    /// Create a new propagation probability.
    ///
    /// # Panics
    /// Panics if value is not in (0, 1).
    #[must_use]
    pub fn new(value: f64) -> Self {
        assert!(
            value > 0.0 && value < 1.0,
            "Probability must be in (0, 1), got {value}"
        );
        Self { value }
    }

    /// Create a propagation probability, returning None if out of bounds.
    #[must_use]
    pub fn try_new(value: f64) -> Option<Self> {
        if value > 0.0 && value < 1.0 {
            Some(Self { value })
        } else {
            None
        }
    }

    /// Get the probability value.
    #[must_use]
    pub fn get(&self) -> f64 {
        self.value
    }
}

/// Compute harm probability using product formula (Axiom 5 / Version B).
///
/// **ℙ(H|δs₁) = ∏ᵢ P_{i→i+1}**
///
/// This is the fundamental product form from the Markov assumption.
#[must_use]
pub fn harm_probability(probs: &[PropagationProbability]) -> f64 {
    probs.iter().map(|p| p.get()).product()
}

/// Compute attenuation rate α = -log(P̄).
///
/// where P̄ is the geometric mean of propagation probabilities.
///
/// Higher α means stronger attenuation (faster probability decay with depth).
#[must_use]
pub fn attenuation_rate(probs: &[PropagationProbability]) -> f64 {
    if probs.is_empty() {
        return 0.0;
    }
    let log_sum: f64 = probs.iter().map(|p| p.get().ln()).sum();
    -log_sum / probs.len() as f64
}

/// Compute harm probability using exponential form (Version D).
///
/// **ℙ(H) = e^{-α(H-1)}**
///
/// # Arguments
/// * `alpha` - Attenuation rate (from `attenuation_rate()`)
/// * `harm_level` - Hierarchy level H (1-based)
#[must_use]
pub fn harm_probability_exponential(alpha: f64, harm_level: usize) -> f64 {
    (-alpha * (harm_level as f64 - 1.0)).exp()
}

/// Compute protective depth for target probability (Corollary).
///
/// Returns minimum H such that ℙ(H) < target_probability.
///
/// **Formula: H ≥ 1 + log(1/ε)/α**
///
/// # Arguments
/// * `target_probability` - ε, the maximum acceptable harm probability
/// * `attenuation_rate` - α, from `attenuation_rate()`
///
/// # Panics
/// Panics if target_probability is not in (0, 1) or attenuation_rate ≤ 0.
#[must_use]
pub fn protective_depth(target_probability: f64, attenuation_rate: f64) -> usize {
    assert!(
        target_probability > 0.0 && target_probability < 1.0,
        "Target probability must be in (0, 1)"
    );
    assert!(attenuation_rate > 0.0, "Attenuation rate must be positive");
    let min_depth = 1.0 + (-target_probability.ln()) / attenuation_rate;
    min_depth.ceil() as usize
}

/// Maximum probability in a slice.
#[must_use]
pub fn max_probability(probs: &[PropagationProbability]) -> f64 {
    probs
        .iter()
        .map(PropagationProbability::get)
        .fold(0.0, f64::max)
}

/// Compute uniform bound (Version A).
///
/// **ℙ(H) ≤ P_max^{H-1}**
///
/// This is a conservative upper bound using the maximum probability.
#[must_use]
pub fn uniform_bound(probs: &[PropagationProbability]) -> f64 {
    let p_max = max_probability(probs);
    let h_minus_1 = probs.len();
    p_max.powi(h_minus_1 as i32)
}

/// Compute geometric mean of probabilities (P̄).
#[must_use]
pub fn geometric_mean(probs: &[PropagationProbability]) -> f64 {
    if probs.is_empty() {
        return 0.0;
    }
    let log_sum: f64 = probs.iter().map(|p| p.get().ln()).sum();
    (log_sum / probs.len() as f64).exp()
}

/// Compute harm probability using geometric mean form (Version C).
///
/// **ℙ(H) = (P̄)^{H-1}**
#[must_use]
pub fn harm_probability_geometric(probs: &[PropagationProbability]) -> f64 {
    let p_bar = geometric_mean(probs);
    let h_minus_1 = probs.len();
    p_bar.powi(h_minus_1 as i32)
}

/// Verify attenuation property: harm decreases with depth.
///
/// Returns true if harm probability strictly decreases as more levels are added.
#[must_use]
pub fn verify_attenuation(probs: &[PropagationProbability]) -> bool {
    if probs.is_empty() {
        return true;
    }

    // Compute harm probabilities for increasing depths
    let mut last_hp = 1.0;
    for i in 1..=probs.len() {
        let hp = harm_probability(&probs[..i]);
        if hp >= last_hp {
            return false;
        }
        last_hp = hp;
    }
    true
}

/// Result of attenuation analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttenuationAnalysis {
    /// Number of hierarchy levels.
    pub levels: usize,
    /// Attenuation rate α.
    pub alpha: f64,
    /// Harm probability at deepest level.
    pub harm_probability: f64,
    /// Uniform upper bound.
    pub uniform_bound: f64,
    /// Geometric mean of probabilities.
    pub geometric_mean: f64,
    /// Whether attenuation property holds.
    pub attenuation_verified: bool,
}

/// Perform complete attenuation analysis.
#[must_use]
pub fn analyze_attenuation(probs: &[PropagationProbability]) -> AttenuationAnalysis {
    AttenuationAnalysis {
        levels: probs.len(),
        alpha: attenuation_rate(probs),
        harm_probability: harm_probability(probs),
        uniform_bound: uniform_bound(probs),
        geometric_mean: geometric_mean(probs),
        attenuation_verified: verify_attenuation(probs),
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_propagation_probability_bounds() {
        let p = PropagationProbability::new(0.5);
        assert!(p.get() > 0.0 && p.get() < 1.0);
    }

    #[test]
    fn test_propagation_probability_try_new() {
        assert!(PropagationProbability::try_new(0.5).is_some());
        assert!(PropagationProbability::try_new(0.0).is_none());
        assert!(PropagationProbability::try_new(1.0).is_none());
        assert!(PropagationProbability::try_new(-0.1).is_none());
        assert!(PropagationProbability::try_new(1.1).is_none());
    }

    #[test]
    #[should_panic]
    fn test_propagation_probability_rejects_1() {
        let _ = PropagationProbability::new(1.0);
    }

    #[test]
    #[should_panic]
    fn test_propagation_probability_rejects_0() {
        let _ = PropagationProbability::new(0.0);
    }

    #[test]
    fn test_harm_probability_is_product() {
        let probs = vec![
            PropagationProbability::new(0.5),
            PropagationProbability::new(0.3),
            PropagationProbability::new(0.2),
        ];
        let hp = harm_probability(&probs);
        let expected = 0.5 * 0.3 * 0.2;
        assert!((hp - expected).abs() < 1e-10);
    }

    #[test]
    fn test_harm_probability_monotonic_decrease() {
        let p = PropagationProbability::new(0.7);

        let hp1 = harm_probability(&[p]);
        let hp2 = harm_probability(&[p, p]);
        let hp3 = harm_probability(&[p, p, p]);
        let hp4 = harm_probability(&[p, p, p, p]);

        assert!(hp1 > hp2, "H=2 should have lower probability than H=1");
        assert!(hp2 > hp3, "H=3 should have lower probability than H=2");
        assert!(hp3 > hp4, "H=4 should have lower probability than H=3");
    }

    #[test]
    fn test_attenuation_rate_positive() {
        let probs: Vec<_> = vec![
            PropagationProbability::new(0.5),
            PropagationProbability::new(0.4),
            PropagationProbability::new(0.3),
        ];
        let alpha = attenuation_rate(&probs);
        assert!(alpha > 0.0, "Attenuation rate should be positive");
    }

    #[test]
    fn test_exponential_decay_formula() {
        let p_val = 0.6;
        let levels = 5;

        let probs: Vec<_> = (0..levels)
            .map(|_| PropagationProbability::new(p_val))
            .collect();

        let actual = harm_probability(&probs);
        let expected = p_val.powi(levels as i32);

        assert!(
            (actual - expected).abs() < 1e-10,
            "Harm probability should equal P^n for uniform P"
        );
    }

    #[test]
    fn test_protective_depth_achieves_target() {
        let alpha = 0.5;
        let target = 0.05;

        let depth = protective_depth(target, alpha);
        let actual_prob = harm_probability_exponential(alpha, depth);

        assert!(
            actual_prob < target,
            "Protective depth {depth} should achieve probability {actual_prob} < {target}"
        );
    }

    #[test]
    fn test_protective_depth_increases_with_stricter_target() {
        let alpha = 1.0;

        let depth_10pct = protective_depth(0.10, alpha);
        let depth_1pct = protective_depth(0.01, alpha);
        let depth_01pct = protective_depth(0.001, alpha);

        assert!(depth_1pct > depth_10pct);
        assert!(depth_01pct > depth_1pct);
    }

    #[test]
    fn test_attenuation_stronger_with_lower_probabilities() {
        let high_p: Vec<_> = (0..3).map(|_| PropagationProbability::new(0.8)).collect();
        let low_p: Vec<_> = (0..3).map(|_| PropagationProbability::new(0.2)).collect();

        let alpha_high = attenuation_rate(&high_p);
        let alpha_low = attenuation_rate(&low_p);

        assert!(
            alpha_low > alpha_high,
            "Lower probabilities should give stronger attenuation"
        );
    }

    #[test]
    fn test_uniform_bound() {
        let probs = vec![
            PropagationProbability::new(0.3),
            PropagationProbability::new(0.5),
            PropagationProbability::new(0.2),
        ];

        let hp = harm_probability(&probs);
        let bound = uniform_bound(&probs);

        assert!(
            hp <= bound,
            "Harm probability {hp} should be bounded by {bound}"
        );
    }

    #[test]
    fn test_verify_attenuation() {
        let probs: Vec<_> = (0..5).map(|_| PropagationProbability::new(0.5)).collect();

        assert!(
            verify_attenuation(&probs),
            "Attenuation property should hold"
        );
    }

    #[test]
    fn test_geometric_mean() {
        let probs = vec![
            PropagationProbability::new(0.2),
            PropagationProbability::new(0.8),
        ];
        let gm = geometric_mean(&probs);
        // Geometric mean of 0.2 and 0.8 = sqrt(0.16) = 0.4
        assert!((gm - 0.4).abs() < 1e-10);
    }

    #[test]
    fn test_analyze_attenuation() {
        let probs: Vec<_> = (0..4).map(|_| PropagationProbability::new(0.5)).collect();
        let analysis = analyze_attenuation(&probs);

        assert_eq!(analysis.levels, 4);
        assert!(analysis.alpha > 0.0);
        assert!(analysis.harm_probability < 1.0);
        assert!(analysis.attenuation_verified);
    }

    #[test]
    fn test_versions_consistent() {
        // For uniform probabilities, all versions should give same result
        let p = 0.5;
        let n = 4;
        let probs: Vec<_> = (0..n).map(|_| PropagationProbability::new(p)).collect();

        let version_b = harm_probability(&probs);
        let version_c = harm_probability_geometric(&probs);
        let alpha = attenuation_rate(&probs);
        let version_d = harm_probability_exponential(alpha, n + 1);

        // All should equal p^n = 0.5^4 = 0.0625
        let expected = p.powi(n as i32);
        assert!((version_b - expected).abs() < 1e-10, "Version B mismatch");
        assert!((version_c - expected).abs() < 1e-10, "Version C mismatch");
        assert!((version_d - expected).abs() < 1e-10, "Version D mismatch");
    }
}
