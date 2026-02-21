//! Signal aggregation — normalize → Beer-Lambert → Hill
//!
//! Tier: T2-C | Primitives: Σ Sum, ρ Recursion

use crate::chemistry;

/// Feature weights (Beer-Lambert absorptivity coefficients).
pub const WEIGHTS: [f64; 5] = [
    2.5, // Zipf deviation
    2.0, // Entropy std
    1.8, // Burstiness
    2.2, // Perplexity variance
    1.5, // TTR deviation
];

/// Hill equation parameters.
pub const HILL_K_HALF: f64 = 0.5;
pub const HILL_N: f64 = 2.5;

/// Raw feature values before normalization.
#[derive(Debug, Clone)]
pub struct RawFeatures {
    pub zipf_deviation: f64,
    pub entropy_std: f64,
    pub burstiness: f64,
    pub perplexity_var: f64,
    pub ttr_deviation: f64,
}

/// Aggregation result with intermediate values.
#[derive(Debug, Clone)]
pub struct AggregationResult {
    pub normalized: [f64; 5],
    pub beer_lambert_score: f64,
    pub composite: f64,
    pub hill_score: f64,
}

/// Normalize a "low = suspicious" feature.
fn normalize_inverted(value: f64, human_typical: f64) -> f64 {
    if human_typical <= 0.0 {
        return 0.5;
    }
    let ratio = 1.0 - (value / human_typical).min(1.0);
    ratio.clamp(0.0, 1.0)
}

/// Normalize a "deviation" feature.
fn normalize_deviation(deviation: f64, max_expected: f64) -> f64 {
    if max_expected <= 0.0 {
        return 0.0;
    }
    (deviation / max_expected).clamp(0.0, 1.0)
}

/// Aggregate 5 features into composite score.
pub fn aggregate(features: &RawFeatures) -> AggregationResult {
    let normalized = [
        normalize_deviation(features.zipf_deviation, 1.0),
        normalize_inverted(features.entropy_std, 1.0),
        normalize_inverted(features.burstiness, 0.3),
        normalize_inverted(features.perplexity_var, 0.5),
        normalize_deviation(features.ttr_deviation, 0.3),
    ];

    let beer_lambert_score = chemistry::beer_lambert_weighted_sum(&WEIGHTS, &normalized);
    let max_score: f64 = WEIGHTS.iter().sum();
    let composite = (beer_lambert_score / max_score).clamp(0.0, 1.0);
    let hill_score = chemistry::hill_amplify(composite, HILL_K_HALF, HILL_N);

    AggregationResult {
        normalized,
        beer_lambert_score,
        composite,
        hill_score,
    }
}
