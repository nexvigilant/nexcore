//! Bloom taxonomy → threshold mapping
//!
//! Tier: T2-C | Primitives: ∂ Boundary, κ Comparison, N Quantity

use serde::{Deserialize, Serialize};

/// Bloom taxonomy level names.
pub const BLOOM_LEVELS: [&str; 7] = [
    "Remember",
    "Understand",
    "Apply",
    "Analyze",
    "Evaluate",
    "Create",
    "Meta-Create",
];

/// Threshold configuration for Bloom-adapted AI detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BloomThresholds {
    pub name: String,
    pub thresholds: [f64; 7],
}

impl BloomThresholds {
    /// PV Education preset — calibrated via Experiment 1.
    #[must_use]
    pub fn pv_education() -> Self {
        Self {
            name: "pv_education".to_string(),
            thresholds: [0.66, 0.65, 0.64, 0.64, 0.64, 0.64, 0.63],
        }
    }

    /// Strict preset — catches more AI at cost of higher FPR.
    #[must_use]
    pub fn strict() -> Self {
        Self {
            name: "strict".to_string(),
            thresholds: [0.63, 0.62, 0.62, 0.62, 0.61, 0.61, 0.60],
        }
    }

    /// Lenient preset — fewer false positives.
    #[must_use]
    pub fn lenient() -> Self {
        Self {
            name: "lenient".to_string(),
            thresholds: [0.68, 0.67, 0.67, 0.67, 0.66, 0.66, 0.66],
        }
    }

    /// Get threshold for a specific Bloom level (1-7).
    #[must_use]
    pub fn threshold_for_level(&self, level: u8) -> Option<f64> {
        if (1..=7).contains(&level) {
            Some(self.thresholds[(level - 1) as usize])
        } else {
            None
        }
    }

    /// Get Bloom level name (1-indexed).
    #[must_use]
    pub fn level_name(level: u8) -> Option<&'static str> {
        if (1..=7).contains(&level) {
            Some(BLOOM_LEVELS[(level - 1) as usize])
        } else {
            None
        }
    }

    /// Get preset by name.
    #[must_use]
    pub fn from_name(name: &str) -> Self {
        match name {
            "strict" => Self::strict(),
            "lenient" => Self::lenient(),
            _ => Self::pv_education(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bloom_levels_count() {
        assert_eq!(BLOOM_LEVELS.len(), 7);
    }

    #[test]
    fn bloom_level_names() {
        assert_eq!(BloomThresholds::level_name(1), Some("Remember"));
        assert_eq!(BloomThresholds::level_name(6), Some("Create"));
        assert_eq!(BloomThresholds::level_name(7), Some("Meta-Create"));
        assert_eq!(BloomThresholds::level_name(0), None);
        assert_eq!(BloomThresholds::level_name(8), None);
    }

    #[test]
    fn threshold_for_valid_levels() {
        let bt = BloomThresholds::pv_education();
        for level in 1..=7 {
            assert!(bt.threshold_for_level(level).is_some(), "level {level}");
        }
    }

    #[test]
    fn threshold_for_invalid_levels() {
        let bt = BloomThresholds::pv_education();
        assert!(bt.threshold_for_level(0).is_none());
        assert!(bt.threshold_for_level(8).is_none());
    }

    #[test]
    fn thresholds_decrease_with_level() {
        let bt = BloomThresholds::pv_education();
        // Higher Bloom level should have equal or lower threshold
        for i in 1..7 {
            let t1 = bt.threshold_for_level(i).unwrap_or(1.0);
            let t2 = bt.threshold_for_level(i + 1).unwrap_or(1.0);
            assert!(t2 <= t1, "level {}: {} > level {}: {}", i, t1, i + 1, t2);
        }
    }

    #[test]
    fn strict_lower_than_pv() {
        let strict = BloomThresholds::strict();
        let pv = BloomThresholds::pv_education();
        for level in 1..=7 {
            let s = strict.threshold_for_level(level).unwrap_or(0.0);
            let p = pv.threshold_for_level(level).unwrap_or(0.0);
            assert!(s <= p, "strict level {} not stricter: {} > {}", level, s, p);
        }
    }

    #[test]
    fn from_name_defaults() {
        let default = BloomThresholds::from_name("unknown");
        assert_eq!(default.name, "pv_education");
        let strict = BloomThresholds::from_name("strict");
        assert_eq!(strict.name, "strict");
    }
}
