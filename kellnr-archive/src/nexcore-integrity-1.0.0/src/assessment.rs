//! # Assessment Orchestration
//!
//! End-to-end assessment pipeline: tokenize → features → aggregate →
//! Bloom-adapted classify. Combines all stages into a single entry point.
//!
//! ## Primitive Grounding
//! - σ Sequence: pipeline stage ordering
//! - → Causality: features → score → verdict chain
//! - ∂ Boundary: Bloom-adapted threshold selection

use serde::Serialize;

use crate::aggregation::{self, RawFeatures};
use crate::bloom::BloomThresholds;
use crate::burstiness;
use crate::classify::{self, Classification, Verdict};
use crate::entropy;
use crate::error::IntegrityError;
use crate::perplexity;
use crate::profile::{self, CalibrationProfile};
use crate::tokenize::{self, TokenStats};
use crate::zipf;

/// Minimum token count for reliable analysis.
pub const MIN_TOKENS: usize = 50;

/// Default entropy window size.
pub const DEFAULT_WINDOW_SIZE: usize = 50;
/// Default entropy window step.
pub const DEFAULT_WINDOW_STEP: usize = 25;

/// Tier: T2-C (domain composite)
///
/// Assessment context configuration.
#[derive(Debug, Clone)]
pub struct AssessmentContext {
    /// Bloom taxonomy level (1-7)
    pub bloom_level: u8,
    /// Domain calibration profile
    pub domain_id: Option<String>,
    /// Custom threshold override (bypasses Bloom mapping)
    pub custom_threshold: Option<f64>,
    /// Bloom threshold preset
    pub bloom_preset: BloomThresholds,
    /// Whether to use strict mode
    pub strict_mode: bool,
}

impl AssessmentContext {
    /// Create a new assessment context with PV education defaults.
    ///
    /// # Errors
    /// Returns error if bloom_level is not 1-7.
    pub fn new(bloom_level: u8) -> Result<Self, IntegrityError> {
        if bloom_level < 1 || bloom_level > 7 {
            return Err(IntegrityError::InvalidBloomLevel { level: bloom_level });
        }
        Ok(Self {
            bloom_level,
            domain_id: None,
            bloom_preset: BloomThresholds::pv_education(),
            custom_threshold: None,
            strict_mode: false,
        })
    }

    /// Set domain calibration.
    #[must_use]
    pub fn with_domain(mut self, domain_id: &str) -> Self {
        self.domain_id = Some(domain_id.to_string());
        self
    }

    /// Set custom threshold (overrides Bloom mapping).
    #[must_use]
    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.custom_threshold = Some(threshold);
        self
    }

    /// Set strict mode.
    #[must_use]
    pub fn with_strict(mut self, strict: bool) -> Self {
        if strict {
            self.bloom_preset = BloomThresholds::strict();
        }
        self.strict_mode = strict;
        self
    }

    /// Resolve the effective threshold for this context.
    fn effective_threshold(&self) -> Result<f64, IntegrityError> {
        if let Some(t) = self.custom_threshold {
            if !(0.0..=1.0).contains(&t) {
                return Err(IntegrityError::InvalidThreshold { value: t });
            }
            return Ok(t);
        }
        self.bloom_preset.threshold_for_level(self.bloom_level)
    }

    /// Resolve the calibration profile.
    fn calibration_profile(&self) -> &CalibrationProfile {
        if let Some(ref id) = self.domain_id {
            profile::get_profile(id).unwrap_or_else(|_| profile::default_profile())
        } else {
            profile::default_profile()
        }
    }
}

/// Tier: T3 (domain-specific)
///
/// Complete integrity assessment result.
#[derive(Debug, Clone, Serialize)]
pub struct IntegrityAssessment {
    /// Classification verdict and probability
    pub classification: Classification,
    /// Effective threshold used
    pub threshold: f64,
    /// Bloom level assessed at
    pub bloom_level: u8,
    /// Bloom level name
    pub bloom_name: &'static str,
    /// Domain ID used
    pub domain_id: String,
    /// Raw feature values
    pub features: FeatureReport,
    /// Token count of input
    pub token_count: usize,
}

/// Feature report for transparency.
#[derive(Debug, Clone, Serialize)]
pub struct FeatureReport {
    pub zipf_deviation: f64,
    pub zipf_alpha: f64,
    pub zipf_r_squared: f64,
    pub entropy_std: f64,
    pub entropy_mean: f64,
    pub burstiness: f64,
    pub perplexity_var: f64,
    pub ttr: f64,
    pub ttr_deviation: f64,
    pub composite_score: f64,
    pub hill_score: f64,
}

/// Run full integrity assessment on raw text.
///
/// # Errors
/// - `InsufficientText` if fewer than `MIN_TOKENS` tokens
/// - `InvalidBloomLevel` if context has invalid level
/// - `InvalidThreshold` if custom threshold out of range
pub fn assess_text(
    text: &str,
    context: &AssessmentContext,
) -> Result<IntegrityAssessment, IntegrityError> {
    // Stage 1: Tokenize
    let stats: TokenStats = tokenize::tokenize(text);

    if stats.total_tokens < MIN_TOKENS {
        return Err(IntegrityError::InsufficientText {
            token_count: stats.total_tokens,
            minimum: MIN_TOKENS,
        });
    }

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

    // Stage 4: Bloom-adapted classify
    let threshold = context.effective_threshold()?;
    let classification = classify::classify_with_threshold(agg.hill_score, threshold);

    let cal = context.calibration_profile();
    let bloom_name = BloomThresholds::level_name(context.bloom_level).unwrap_or("Unknown");
    let domain_id = context
        .domain_id
        .clone()
        .unwrap_or_else(|| cal.domain_id.to_string());

    Ok(IntegrityAssessment {
        classification,
        threshold,
        bloom_level: context.bloom_level,
        bloom_name,
        domain_id,
        features: FeatureReport {
            zipf_deviation: zipf_result.deviation,
            zipf_alpha: zipf_result.alpha,
            zipf_r_squared: zipf_result.r_squared,
            entropy_std: entropy_profile.std_dev,
            entropy_mean: entropy_profile.mean,
            burstiness: burst_result.coefficient,
            perplexity_var: perp_result.variance,
            ttr: stats.ttr,
            ttr_deviation: ttr_dev,
            composite_score: agg.composite,
            hill_score: agg.hill_score,
        },
        token_count: stats.total_tokens,
    })
}

/// Convenience function for KSB response assessment.
///
/// Takes text and bloom_level, uses PV education defaults.
///
/// # Errors
/// Same as `assess_text`.
pub fn assess_ksb_response(
    text: &str,
    bloom_level: u8,
    domain_id: Option<&str>,
) -> Result<IntegrityAssessment, IntegrityError> {
    let mut context = AssessmentContext::new(bloom_level)?;
    if let Some(id) = domain_id {
        context = context.with_domain(id);
    }
    assess_text(text, &context)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn long_human_text() -> String {
        // ~100 tokens of naturally varied text
        "The pharmacovigilance system requires careful monitoring of adverse drug reactions. \
         Each case report must be evaluated individually, considering the patient's medical \
         history, concomitant medications, and temporal relationship between drug exposure \
         and the onset of symptoms. Signal detection algorithms like PRR and ROR help \
         identify potential safety concerns. However, statistical signals alone are \
         insufficient — clinical judgment remains essential. The Bradford Hill criteria \
         provide a framework for causality assessment, examining consistency, specificity, \
         and biological plausibility. Regulatory submissions must follow ICH E2C guidelines \
         for periodic benefit-risk evaluation reports. Healthcare professionals play a \
         crucial role in spontaneous reporting systems."
            .to_string()
    }

    fn long_uniform_text() -> String {
        // ~100 tokens of suspiciously uniform text
        let sentence = "The system provides comprehensive functionality for processing data. ";
        sentence.repeat(12)
    }

    #[test]
    fn test_context_creation() {
        assert!(AssessmentContext::new(1).is_ok());
        assert!(AssessmentContext::new(7).is_ok());
        assert!(AssessmentContext::new(0).is_err());
        assert!(AssessmentContext::new(8).is_err());
    }

    #[test]
    fn test_context_builder() {
        let ctx = AssessmentContext::new(3);
        assert!(ctx.is_ok());
        let ctx = ctx.unwrap_or_else(|_| {
            AssessmentContext::new(1).unwrap_or_else(|_| {
                // fallback that should never happen
                AssessmentContext {
                    bloom_level: 1,
                    domain_id: None,
                    bloom_preset: BloomThresholds::pv_education(),
                    custom_threshold: None,
                    strict_mode: false,
                }
            })
        });
        let ctx = ctx.with_domain("D08").with_strict(false);
        assert_eq!(ctx.bloom_level, 3);
        assert_eq!(ctx.domain_id, Some("D08".to_string()));
    }

    #[test]
    fn test_insufficient_text() {
        let short = "Too short for analysis.";
        let ctx = AssessmentContext::new(3);
        assert!(ctx.is_ok());
        if let Ok(ctx) = ctx {
            let result = assess_text(short, &ctx);
            assert!(result.is_err());
            assert!(matches!(
                result,
                Err(IntegrityError::InsufficientText { .. })
            ));
        }
    }

    #[test]
    fn test_assess_human_text() {
        let text = long_human_text();
        let ctx = AssessmentContext::new(3);
        assert!(ctx.is_ok());
        if let Ok(ctx) = ctx {
            let result = assess_text(&text, &ctx);
            assert!(result.is_ok(), "Assessment failed: {result:?}");
            if let Ok(assessment) = result {
                assert!(assessment.token_count >= MIN_TOKENS);
                assert_eq!(assessment.bloom_level, 3);
                assert_eq!(assessment.bloom_name, "Apply");
                // Feature report should be populated
                assert!(assessment.features.zipf_alpha > 0.0);
            }
        }
    }

    #[test]
    fn test_assess_uniform_text() {
        let text = long_uniform_text();
        let ctx = AssessmentContext::new(5);
        assert!(ctx.is_ok());
        if let Ok(ctx) = ctx {
            let result = assess_text(&text, &ctx);
            assert!(result.is_ok());
            if let Ok(assessment) = result {
                // Uniform text should have low entropy variance
                assert!(assessment.features.entropy_std < 0.5);
            }
        }
    }

    #[test]
    fn test_custom_threshold() {
        let text = long_human_text();
        let ctx = AssessmentContext::new(3);
        assert!(ctx.is_ok());
        if let Ok(ctx) = ctx {
            let ctx = ctx.with_threshold(0.9);
            let result = assess_text(&text, &ctx);
            assert!(result.is_ok());
            if let Ok(assessment) = result {
                assert!((assessment.threshold - 0.9).abs() < 1e-10);
            }
        }
    }

    #[test]
    fn test_invalid_custom_threshold() {
        let text = long_human_text();
        let ctx = AssessmentContext::new(3);
        assert!(ctx.is_ok());
        if let Ok(ctx) = ctx {
            let ctx = ctx.with_threshold(1.5);
            let result = assess_text(&text, &ctx);
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_assess_ksb_response_convenience() {
        let text = long_human_text();
        let result = assess_ksb_response(&text, 4, Some("D08"));
        assert!(result.is_ok());
        if let Ok(assessment) = result {
            assert_eq!(assessment.bloom_level, 4);
            assert_eq!(assessment.domain_id, "D08");
        }
    }

    #[test]
    fn test_bloom_threshold_adaptation() {
        // Same text at different Bloom levels should get different thresholds
        let text = long_human_text();
        let r1 = assess_ksb_response(&text, 1, None);
        let r5 = assess_ksb_response(&text, 5, None);
        assert!(r1.is_ok());
        assert!(r5.is_ok());
        if let (Ok(a1), Ok(a5)) = (r1, r5) {
            // L5 should have lower threshold (stricter)
            assert!(
                a5.threshold < a1.threshold,
                "L5 threshold {} should be < L1 threshold {}",
                a5.threshold,
                a1.threshold
            );
            // Same features, different threshold → possibly different verdict
            assert!((a1.features.hill_score - a5.features.hill_score).abs() < 1e-10);
        }
    }

    #[test]
    fn test_strict_mode() {
        let text = long_human_text();
        let normal = AssessmentContext::new(3);
        let strict = AssessmentContext::new(3);
        assert!(normal.is_ok());
        assert!(strict.is_ok());
        if let (Ok(normal), Ok(strict)) = (normal, strict) {
            let strict = strict.with_strict(true);
            let rn = assess_text(&text, &normal);
            let rs = assess_text(&text, &strict);
            assert!(rn.is_ok());
            assert!(rs.is_ok());
            if let (Ok(an), Ok(a_s)) = (rn, rs) {
                // Strict threshold should be lower
                assert!(a_s.threshold <= an.threshold);
            }
        }
    }

    #[test]
    fn test_feature_report_completeness() {
        let text = long_human_text();
        let result = assess_ksb_response(&text, 3, None);
        assert!(result.is_ok());
        if let Ok(assessment) = result {
            let f = &assessment.features;
            // All features should be populated (non-NaN)
            assert!(!f.zipf_deviation.is_nan());
            assert!(!f.entropy_std.is_nan());
            assert!(!f.burstiness.is_nan());
            assert!(!f.perplexity_var.is_nan());
            assert!(!f.ttr.is_nan());
            assert!(!f.composite_score.is_nan());
            assert!(!f.hill_score.is_nan());
        }
    }
}
