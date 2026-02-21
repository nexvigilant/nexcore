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
