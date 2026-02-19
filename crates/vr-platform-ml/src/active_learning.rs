//! Cross-tenant active learning optimization.
//!
//! Implements uncertainty-based sampling strategies and Bayesian optimization
//! primitives for efficient model improvement with minimal labeling budget.

use serde::{Deserialize, Serialize};

/// An uncertainty estimate for a single prediction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UncertaintyEstimate {
    /// The model's predicted value.
    pub prediction: f64,
    /// Estimated uncertainty (higher = less certain).
    pub uncertainty: f64,
    /// The model's self-reported confidence [0.0, 1.0].
    pub model_confidence: f64,
}

/// Calculate the Shannon entropy of a discrete probability distribution.
///
/// H = -sum(p * log2(p)) for all p > 0.
///
/// Handles p=0 by treating 0*log2(0) as 0 (the limit as p->0+).
/// Returns 0.0 for an empty distribution.
#[must_use]
pub fn prediction_entropy(class_probabilities: &[f64]) -> f64 {
    if class_probabilities.is_empty() {
        return 0.0;
    }

    let mut entropy = 0.0;
    for &p in class_probabilities {
        if p > 0.0 {
            entropy -= p * p.log2();
        }
    }
    entropy
}

/// Calculate the margin sampling score.
///
/// Margin = 1.0 - (max_probability - second_max_probability)
///
/// Higher scores indicate greater uncertainty (the model can't distinguish
/// between the top two classes). Returns 1.0 for distributions with
/// fewer than 2 classes (maximally uncertain about nothing to compare).
#[must_use]
pub fn margin_sampling_score(class_probabilities: &[f64]) -> f64 {
    if class_probabilities.len() < 2 {
        return 1.0;
    }

    // Find the two highest probabilities without sorting the whole array.
    let mut max1 = f64::NEG_INFINITY;
    let mut max2 = f64::NEG_INFINITY;

    for &p in class_probabilities {
        if p >= max1 {
            max2 = max1;
            max1 = p;
        } else if p > max2 {
            max2 = p;
        }
    }

    let margin = max1 - max2;
    1.0 - margin
}

/// Select the indices of the most uncertain samples, up to a budget limit.
///
/// Returns indices sorted by uncertainty (highest first), capped at `budget`.
/// If `budget` exceeds the number of estimates, returns all indices.
#[must_use]
pub fn select_most_uncertain(estimates: &[UncertaintyEstimate], budget: usize) -> Vec<usize> {
    if estimates.is_empty() || budget == 0 {
        return Vec::new();
    }

    // Pair each estimate with its original index.
    let mut indexed: Vec<(usize, f64)> = estimates
        .iter()
        .enumerate()
        .map(|(i, e)| (i, e.uncertainty))
        .collect();

    // Sort by uncertainty descending. Use total_cmp for deterministic NaN handling.
    indexed.sort_by(|a, b| b.1.total_cmp(&a.1));

    let count = budget.min(indexed.len());
    indexed[..count].iter().map(|&(i, _)| i).collect()
}

/// Approximate the standard normal CDF using the logistic approximation.
///
/// Phi(z) ~ 1 / (1 + exp(-1.7 * z))
///
/// This is accurate to within ~0.01 across the entire range and avoids
/// needing the error function.
fn standard_normal_cdf(z: f64) -> f64 {
    1.0 / (1.0 + (-1.7 * z).exp())
}

/// Standard normal PDF: phi(z) = (1/sqrt(2*pi)) * exp(-z^2/2)
fn standard_normal_pdf(z: f64) -> f64 {
    const INV_SQRT_2PI: f64 = 0.398_942_280_401_432_7; // 1/sqrt(2*pi)
    INV_SQRT_2PI * (-0.5 * z * z).exp()
}

/// Calculate the Expected Improvement (EI) for Bayesian optimization.
///
/// EI(x) = sigma * (Z * Phi(Z) + phi(Z))
///
/// where Z = (predicted_mean - current_best) / predicted_std,
/// Phi is the standard normal CDF, and phi is the standard normal PDF.
///
/// Returns 0.0 if `predicted_std` is approximately zero (< 1e-12),
/// since there is no uncertainty to exploit.
///
/// - `current_best`: The best observed value so far (to maximize).
/// - `predicted_mean`: The model's predicted mean at the candidate point.
/// - `predicted_std`: The model's predicted standard deviation (uncertainty).
#[must_use]
pub fn expected_improvement(current_best: f64, predicted_mean: f64, predicted_std: f64) -> f64 {
    if predicted_std < 1e-12 {
        return 0.0;
    }

    let z = (predicted_mean - current_best) / predicted_std;
    let phi_z = standard_normal_pdf(z);
    let big_phi_z = standard_normal_cdf(z);

    predicted_std * (z * big_phi_z + phi_z)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn entropy_uniform_binary() {
        // Uniform binary distribution: H = 1.0 bit
        let h = prediction_entropy(&[0.5, 0.5]);
        assert!((h - 1.0).abs() < 1e-10);
    }

    #[test]
    fn entropy_certain() {
        // Certain prediction: H = 0.0
        let h = prediction_entropy(&[1.0, 0.0]);
        assert!((h - 0.0).abs() < 1e-10);
    }

    #[test]
    fn entropy_uniform_four_classes() {
        // Uniform 4-class: H = 2.0 bits
        let h = prediction_entropy(&[0.25, 0.25, 0.25, 0.25]);
        assert!((h - 2.0).abs() < 1e-10);
    }

    #[test]
    fn entropy_empty() {
        assert!((prediction_entropy(&[]) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn entropy_single_class() {
        let h = prediction_entropy(&[1.0]);
        assert!((h - 0.0).abs() < 1e-10);
    }

    #[test]
    fn entropy_skewed() {
        // H([0.9, 0.1]) = -0.9*log2(0.9) - 0.1*log2(0.1)
        let h = prediction_entropy(&[0.9, 0.1]);
        let expected = -0.9 * 0.9_f64.log2() - 0.1 * 0.1_f64.log2();
        assert!((h - expected).abs() < 1e-10);
    }

    #[test]
    fn margin_high_confidence() {
        // max=0.9, second=0.1 => margin=0.8 => score=0.2
        let score = margin_sampling_score(&[0.9, 0.1]);
        assert!((score - 0.2).abs() < 1e-10);
    }

    #[test]
    fn margin_low_confidence() {
        // max=0.51, second=0.49 => margin=0.02 => score=0.98
        let score = margin_sampling_score(&[0.51, 0.49]);
        assert!((score - 0.98).abs() < 1e-10);
    }

    #[test]
    fn margin_three_classes() {
        // [0.5, 0.3, 0.2] => max=0.5, second=0.3 => margin=0.2 => score=0.8
        let score = margin_sampling_score(&[0.5, 0.3, 0.2]);
        assert!((score - 0.8).abs() < 1e-10);
    }

    #[test]
    fn margin_single_class() {
        let score = margin_sampling_score(&[1.0]);
        assert!((score - 1.0).abs() < 1e-10);
    }

    #[test]
    fn margin_empty() {
        let score = margin_sampling_score(&[]);
        assert!((score - 1.0).abs() < 1e-10);
    }

    #[test]
    fn select_most_uncertain_basic() {
        let estimates = vec![
            UncertaintyEstimate {
                prediction: 0.5,
                uncertainty: 0.1,
                model_confidence: 0.9,
            },
            UncertaintyEstimate {
                prediction: 0.5,
                uncertainty: 0.9,
                model_confidence: 0.1,
            },
            UncertaintyEstimate {
                prediction: 0.5,
                uncertainty: 0.5,
                model_confidence: 0.5,
            },
        ];
        let selected = select_most_uncertain(&estimates, 2);
        assert_eq!(selected.len(), 2);
        assert_eq!(selected[0], 1); // highest uncertainty
        assert_eq!(selected[1], 2); // second highest
    }

    #[test]
    fn select_most_uncertain_budget_exceeds() {
        let estimates = vec![
            UncertaintyEstimate {
                prediction: 0.5,
                uncertainty: 0.3,
                model_confidence: 0.7,
            },
            UncertaintyEstimate {
                prediction: 0.5,
                uncertainty: 0.8,
                model_confidence: 0.2,
            },
        ];
        let selected = select_most_uncertain(&estimates, 10);
        assert_eq!(selected.len(), 2);
    }

    #[test]
    fn select_most_uncertain_empty() {
        let selected = select_most_uncertain(&[], 5);
        assert!(selected.is_empty());
    }

    #[test]
    fn select_most_uncertain_zero_budget() {
        let estimates = vec![UncertaintyEstimate {
            prediction: 0.5,
            uncertainty: 0.5,
            model_confidence: 0.5,
        }];
        let selected = select_most_uncertain(&estimates, 0);
        assert!(selected.is_empty());
    }

    #[test]
    fn expected_improvement_zero_std() {
        // No uncertainty => no improvement expected
        let ei = expected_improvement(0.5, 0.6, 0.0);
        assert!((ei - 0.0).abs() < 1e-12);
    }

    #[test]
    fn expected_improvement_positive_when_mean_above_best() {
        // predicted_mean > current_best with nonzero std => positive EI
        let ei = expected_improvement(0.5, 0.8, 0.1);
        assert!(ei > 0.0);
    }

    #[test]
    fn expected_improvement_still_positive_when_mean_below_best() {
        // Even when predicted_mean < current_best, EI > 0 due to uncertainty
        let ei = expected_improvement(0.8, 0.5, 0.5);
        assert!(ei > 0.0);
    }

    #[test]
    fn expected_improvement_increases_with_std() {
        // Higher uncertainty => higher EI (more exploration value)
        let ei_low = expected_improvement(0.5, 0.6, 0.1);
        let ei_high = expected_improvement(0.5, 0.6, 1.0);
        assert!(ei_high > ei_low);
    }

    #[test]
    fn expected_improvement_increases_with_mean() {
        // Higher predicted mean => higher EI
        let ei_low = expected_improvement(0.5, 0.6, 0.2);
        let ei_high = expected_improvement(0.5, 1.0, 0.2);
        assert!(ei_high > ei_low);
    }

    #[test]
    fn standard_normal_cdf_symmetry() {
        // Phi(0) should be ~0.5
        let phi_0 = standard_normal_cdf(0.0);
        assert!((phi_0 - 0.5).abs() < 0.01);
    }

    #[test]
    fn standard_normal_cdf_tails() {
        // Phi(-3) should be very small
        let phi_neg3 = standard_normal_cdf(-3.0);
        assert!(phi_neg3 < 0.01);
        // Phi(3) should be close to 1
        let phi_3 = standard_normal_cdf(3.0);
        assert!(phi_3 > 0.99);
    }

    #[test]
    fn standard_normal_pdf_peak() {
        // phi(0) = 1/sqrt(2*pi) ~ 0.3989
        let phi_0 = standard_normal_pdf(0.0);
        assert!((phi_0 - 0.3989).abs() < 0.001);
    }

    #[test]
    fn uncertainty_estimate_serialization() {
        let est = UncertaintyEstimate {
            prediction: 0.75,
            uncertainty: 0.3,
            model_confidence: 0.85,
        };
        let json = serde_json::to_string(&est).unwrap();
        let back: UncertaintyEstimate = serde_json::from_str(&json).unwrap();
        assert!((back.prediction - 0.75).abs() < f64::EPSILON);
        assert!((back.uncertainty - 0.3).abs() < f64::EPSILON);
    }
}
