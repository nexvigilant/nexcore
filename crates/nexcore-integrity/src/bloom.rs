//! # Bloom Taxonomy → Threshold Mapping
//!
//! Maps Bloom taxonomy cognitive levels (1-7) to AI detection thresholds.
//! Higher cognitive levels (analysis, synthesis, evaluation) require more
//! original thought, so detection thresholds are *lower* — we're more
//! suspicious of AI assistance at higher cognitive demands.
//!
//! ## Primitive Grounding
//! - ∂ Boundary: threshold values define decision boundaries
//! - κ Comparison: Bloom level ordering determines threshold selection
//! - N Quantity: numeric threshold values

use crate::error::IntegrityError;
use serde::Serialize;

/// Bloom taxonomy level names for reference.
pub const BLOOM_LEVELS: [&str; 7] = [
    "Remember",    // L1: recall facts
    "Understand",  // L2: explain concepts
    "Apply",       // L3: use in new situations
    "Analyze",     // L4: break into components
    "Evaluate",    // L5: justify decisions
    "Create",      // L6: produce original work
    "Meta-Create", // L7: create frameworks
];

/// Tier: T2-C (domain composite)
///
/// Threshold configuration for Bloom-adapted AI detection.
/// Lower thresholds = stricter detection = more sensitive to AI.
#[derive(Debug, Clone, Serialize)]
pub struct BloomThresholds {
    /// Preset name
    pub name: &'static str,
    /// Thresholds per Bloom level (index 0 = L1, index 6 = L7)
    pub thresholds: [f64; 7],
}

impl BloomThresholds {
    /// PV Education preset — calibrated via recalibration simulation on
    /// 40-sample KSB fixture corpus (Experiment 1).
    ///
    /// Key finding: Arrhenius probabilities cluster in [0.57, 0.68] for
    /// short text (~80-120 words). Per-Bloom Youden's J analysis shows
    /// L1/L2/L4 are separable; L3/L5/L6/L7 overlap. A gentle slope from
    /// 0.66 → 0.63 balances FPR/FNR in this narrow discrimination band.
    ///
    /// Recalibration data: recalibrate + recalibrate_deep examples.
    #[must_use]
    pub fn pv_education() -> Self {
        Self {
            name: "pv_education",
            thresholds: [0.66, 0.65, 0.64, 0.64, 0.64, 0.64, 0.63],
        }
    }

    /// Strict preset — catches more AI at cost of higher FPR.
    /// Use for high-stakes summative assessments.
    #[must_use]
    pub fn strict() -> Self {
        Self {
            name: "strict",
            thresholds: [0.63, 0.62, 0.62, 0.62, 0.61, 0.61, 0.60],
        }
    }

    /// Lenient preset — fewer false positives, misses some AI.
    /// Use for low-stakes formative assessments.
    #[must_use]
    pub fn lenient() -> Self {
        Self {
            name: "lenient",
            thresholds: [0.68, 0.67, 0.67, 0.67, 0.66, 0.66, 0.66],
        }
    }

    /// Get threshold for a specific Bloom level (1-7).
    ///
    /// # Errors
    /// Returns `InvalidBloomLevel` if level is outside 1-7.
    pub fn threshold_for_level(&self, level: u8) -> Result<f64, IntegrityError> {
        if level < 1 || level > 7 {
            return Err(IntegrityError::InvalidBloomLevel { level });
        }
        Ok(self.thresholds[(level - 1) as usize])
    }

    /// Get Bloom level name (1-indexed).
    #[must_use]
    pub fn level_name(level: u8) -> Option<&'static str> {
        if level >= 1 && level <= 7 {
            Some(BLOOM_LEVELS[(level - 1) as usize])
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pv_education_thresholds() {
        let bt = BloomThresholds::pv_education();
        assert_eq!(bt.name, "pv_education");
        assert!((bt.thresholds[0] - 0.66).abs() < 1e-10); // L1
        assert!(
            (bt.thresholds[6] - 0.63).abs() < 1e-10,
            "L7 threshold: {}",
            bt.thresholds[6]
        ); // L7
    }

    #[test]
    fn test_threshold_ordering() {
        // Higher Bloom = lower threshold (stricter)
        let bt = BloomThresholds::pv_education();
        for i in 0..6 {
            assert!(
                bt.thresholds[i] >= bt.thresholds[i + 1],
                "L{} ({}) should be >= L{} ({})",
                i + 1,
                bt.thresholds[i],
                i + 2,
                bt.thresholds[i + 1]
            );
        }
    }

    #[test]
    fn test_threshold_for_valid_levels() {
        let bt = BloomThresholds::pv_education();
        for level in 1..=7 {
            let result = bt.threshold_for_level(level);
            assert!(result.is_ok(), "Level {level} should be valid");
            let threshold = result.unwrap_or(0.0);
            assert!(threshold > 0.0 && threshold <= 1.0);
        }
    }

    #[test]
    fn test_threshold_for_invalid_levels() {
        let bt = BloomThresholds::pv_education();
        assert!(bt.threshold_for_level(0).is_err());
        assert!(bt.threshold_for_level(8).is_err());
        assert!(bt.threshold_for_level(255).is_err());
    }

    #[test]
    fn test_strict_lower_than_pv_education() {
        let pv = BloomThresholds::pv_education();
        let strict = BloomThresholds::strict();
        for i in 0..7 {
            assert!(
                strict.thresholds[i] <= pv.thresholds[i],
                "Strict L{} ({}) should be <= PV L{} ({})",
                i + 1,
                strict.thresholds[i],
                i + 1,
                pv.thresholds[i]
            );
        }
    }

    #[test]
    fn test_lenient_higher_than_pv_education() {
        let pv = BloomThresholds::pv_education();
        let lenient = BloomThresholds::lenient();
        for i in 0..7 {
            assert!(
                lenient.thresholds[i] >= pv.thresholds[i],
                "Lenient L{} ({}) should be >= PV L{} ({})",
                i + 1,
                lenient.thresholds[i],
                i + 1,
                pv.thresholds[i]
            );
        }
    }

    #[test]
    fn test_level_name() {
        assert_eq!(BloomThresholds::level_name(1), Some("Remember"));
        assert_eq!(BloomThresholds::level_name(7), Some("Meta-Create"));
        assert_eq!(BloomThresholds::level_name(0), None);
        assert_eq!(BloomThresholds::level_name(8), None);
    }
}
