#![doc = "Text integrity detection pipeline"]
//! Integrity detection pipeline — full orchestration
//!
//! Tier: T3 | Primitives: σ Sequence, → Causality, ∂ Boundary

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(missing_docs)]
pub mod aggregation;
pub mod bloom;
pub mod burstiness;
pub mod chemistry;
pub mod classify;
pub mod entropy;
pub mod perplexity;
pub mod tokenize;
pub mod zipf;

use crate::aggregation::RawFeatures;
use crate::bloom::BloomThresholds;
use crate::classify::Verdict;
use serde::{Deserialize, Serialize};

/// Minimum token count for reliable analysis.
pub const MIN_TOKENS: usize = 50;
/// Default entropy window size.
pub const DEFAULT_WINDOW_SIZE: usize = 50;
/// Default entropy window step.
pub const DEFAULT_WINDOW_STEP: usize = 25;

/// Flat pipeline result for server→client transfer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineResult {
    // Verdict
    pub verdict: Verdict,
    pub probability: f64,
    pub confidence: f64,
    pub threshold: f64,

    // Config
    pub bloom_level: u8,
    pub bloom_name: String,
    pub preset_name: String,

    // Token stats
    pub total_tokens: usize,
    pub unique_tokens: usize,
    pub ttr: f64,

    // Raw features
    pub zipf_deviation: f64,
    pub zipf_alpha: f64,
    pub zipf_r_squared: f64,
    pub entropy_mean: f64,
    pub entropy_std: f64,
    pub entropy_window_count: usize,
    pub burstiness_coeff: f64,
    pub burstiness_tokens_analyzed: usize,
    pub perplexity_mean: f64,
    pub perplexity_var: f64,
    pub perplexity_sentence_count: usize,
    pub ttr_deviation: f64,

    // Normalized features [0,1]
    pub normalized: [f64; 5],
    pub weights: [f64; 5],

    // Aggregation
    pub beer_lambert_score: f64,
    pub composite: f64,
    pub hill_score: f64,

    // Error (empty if success)
    pub error: String,
}

impl PipelineResult {
    /// Create an error result.
    fn error(msg: String) -> Self {
        Self {
            verdict: Verdict::Human,
            probability: 0.0,
            confidence: 0.0,
            threshold: 0.0,
            bloom_level: 0,
            bloom_name: String::new(),
            preset_name: String::new(),
            total_tokens: 0,
            unique_tokens: 0,
            ttr: 0.0,
            zipf_deviation: 0.0,
            zipf_alpha: 0.0,
            zipf_r_squared: 0.0,
            entropy_mean: 0.0,
            entropy_std: 0.0,
            entropy_window_count: 0,
            burstiness_coeff: 0.0,
            burstiness_tokens_analyzed: 0,
            perplexity_mean: 0.0,
            perplexity_var: 0.0,
            perplexity_sentence_count: 0,
            ttr_deviation: 0.0,
            normalized: [0.0; 5],
            weights: aggregation::WEIGHTS,
            beer_lambert_score: 0.0,
            composite: 0.0,
            hill_score: 0.0,
            error: msg,
        }
    }
}

/// Run the full integrity detection pipeline.
pub fn run_pipeline(text: &str, bloom_level: u8, preset: &str) -> PipelineResult {
    // Stage 1: Tokenize
    let stats = tokenize::tokenize(text);

    if stats.total_tokens < MIN_TOKENS {
        return PipelineResult::error(format!(
            "Need at least {} tokens, got {}",
            MIN_TOKENS, stats.total_tokens
        ));
    }

    // Resolve threshold
    let bloom_thresholds = BloomThresholds::from_name(preset);
    let threshold = bloom_thresholds
        .threshold_for_level(bloom_level)
        .unwrap_or(0.64);
    let bloom_name = BloomThresholds::level_name(bloom_level)
        .unwrap_or("Unknown")
        .to_string();

    // Stage 2: Feature extraction
    let zipf_result = zipf::zipf_analysis(&stats.frequencies);
    let entropy_profile =
        entropy::entropy_profile(&stats.tokens, DEFAULT_WINDOW_SIZE, DEFAULT_WINDOW_STEP);
    let burst_result = burstiness::burstiness_analysis(&stats.tokens, &stats.frequencies);
    let perp_result = perplexity::perplexity_variance(text);
    let ttr_dev = tokenize::ttr_deviation(stats.ttr);

    // Stage 3: Aggregate
    let raw = RawFeatures {
        zipf_deviation: zipf_result.deviation,
        entropy_std: entropy_profile.std_dev,
        burstiness: burst_result.coefficient,
        perplexity_var: perp_result.variance,
        ttr_deviation: ttr_dev,
    };
    let agg = aggregation::aggregate(&raw);

    // Stage 4: Classify
    let classification = classify::classify_with_threshold(agg.hill_score, threshold);

    PipelineResult {
        verdict: classification.verdict,
        probability: classification.probability,
        confidence: classification.confidence,
        threshold,
        bloom_level,
        bloom_name,
        preset_name: bloom_thresholds.name,
        total_tokens: stats.total_tokens,
        unique_tokens: stats.unique_tokens,
        ttr: stats.ttr,
        zipf_deviation: zipf_result.deviation,
        zipf_alpha: zipf_result.alpha,
        zipf_r_squared: zipf_result.r_squared,
        entropy_mean: entropy_profile.mean,
        entropy_std: entropy_profile.std_dev,
        entropy_window_count: entropy_profile.window_count,
        burstiness_coeff: burst_result.coefficient,
        burstiness_tokens_analyzed: burst_result.tokens_analyzed,
        perplexity_mean: perp_result.mean_entropy,
        perplexity_var: perp_result.variance,
        perplexity_sentence_count: perp_result.sentence_count,
        ttr_deviation: ttr_dev,
        normalized: agg.normalized,
        weights: aggregation::WEIGHTS,
        beer_lambert_score: agg.beer_lambert_score,
        composite: agg.composite,
        hill_score: agg.hill_score,
        error: String::new(),
    }
}
