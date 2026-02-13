//! Betting signal detection thresholds.
//!
//! # Codex Compliance
//! - **Tier**: T2-P / T2-C
//! - **Grounding**: All thresholds map to T1 constants.
//! - **Quantification**: Qualitative strength maps to quantitative multiplier.

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

// =============================================================================
// CONSTANTS - T1 Primitives
// =============================================================================

/// BDI Threshold for Elite signals.
pub const BDI_ELITE: f64 = 6.0;
/// BDI Threshold for Strong signals.
pub const BDI_STRONG: f64 = 4.0;
/// BDI Threshold for Moderate signals.
pub const BDI_MODERATE: f64 = 2.0;
/// BDI Threshold for Weak signals.
pub const BDI_WEAK: f64 = 1.0;

/// ECS Threshold for Elite signals.
pub const ECS_ELITE: f64 = 5.0;
/// ECS Threshold for Strong signals.
pub const ECS_STRONG: f64 = 3.0;
/// ECS Threshold for Moderate signals.
pub const ECS_MODERATE: f64 = 2.0;
/// ECS Threshold for Weak signals.
pub const ECS_WEAK: f64 = 1.0;

// =============================================================================
// ENUMS - Tier: T2-C
// =============================================================================

/// Signal strength classification.
///
/// # Tier: T2-C
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalStrength {
    /// Very strong signal (Elite).
    Elite = 5,
    /// Strong signal.
    Strong = 4,
    /// Moderate signal.
    Moderate = 3,
    /// Weak/borderline signal.
    Weak = 2,
    /// Opposite direction, avoid.
    Avoid = 1,
}

impl Ord for SignalStrength {
    fn cmp(&self, other: &Self) -> Ordering {
        (*self as u8).cmp(&(*other as u8))
    }
}

impl PartialOrd for SignalStrength {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Quantity multiplier derived from signal strength.
///
/// # Tier: T2-P
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuantityMultiplier(pub u64);

impl From<SignalStrength> for QuantityMultiplier {
    fn from(strength: SignalStrength) -> Self {
        match strength {
            SignalStrength::Elite => Self(5),
            SignalStrength::Strong => Self(2),
            SignalStrength::Moderate => Self(1),
            SignalStrength::Weak | SignalStrength::Avoid => Self(0),
        }
    }
}

impl SignalStrength {
    /// Classify BDI value into signal strength.
    #[must_use]
    pub fn from_bdi(bdi: f64) -> Self {
        if bdi >= BDI_ELITE {
            Self::Elite
        } else if bdi >= BDI_STRONG {
            Self::Strong
        } else if bdi >= BDI_MODERATE {
            Self::Moderate
        } else if bdi >= BDI_WEAK {
            Self::Weak
        } else {
            Self::Avoid
        }
    }

    /// Classify ECS value into signal strength.
    #[must_use]
    pub fn from_ecs(ecs: f64) -> Self {
        if ecs >= ECS_ELITE {
            Self::Elite
        } else if ecs >= ECS_STRONG {
            Self::Strong
        } else if ecs >= ECS_MODERATE {
            Self::Moderate
        } else if ecs >= ECS_WEAK {
            Self::Weak
        } else {
            Self::Avoid
        }
    }

    /// Check if signal is actionable.
    #[must_use]
    pub fn is_actionable(&self) -> bool {
        *self >= Self::Moderate
    }
}

/// Threshold configuration preset.
///
/// # Tier: T2-P
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ThresholdPreset {
    /// Standard Evans-adapted thresholds.
    #[default]
    Evans,
    /// Stricter thresholds for high-stakes.
    Strict,
    /// More sensitive for early detection.
    Sensitive,
}

/// Betting signal detection thresholds.
///
/// # Tier: T2-C
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BettingThresholds {
    /// Minimum BDI for signal detection.
    pub bdi_min: f64,
    /// Minimum chi-square for statistical significance.
    pub chi_square_min: f64,
    /// Minimum observation count.
    pub n_min: u32,
    /// Minimum ECS for actionable signal.
    pub ecs_min: f64,
    /// Minimum lower credibility bound (EB05 analog).
    pub lower_credibility_min: f64,
}

impl BettingThresholds {
    /// Create thresholds from preset.
    #[must_use]
    pub fn from_preset(preset: ThresholdPreset) -> Self {
        match preset {
            ThresholdPreset::Evans => Self {
                bdi_min: BDI_MODERATE,
                chi_square_min: 3.841,
                n_min: 3,
                ecs_min: ECS_MODERATE,
                lower_credibility_min: 0.0,
            },
            ThresholdPreset::Strict => Self {
                bdi_min: 3.0,
                chi_square_min: 6.635,
                n_min: 5,
                ecs_min: ECS_STRONG,
                lower_credibility_min: 1.0,
            },
            ThresholdPreset::Sensitive => Self {
                bdi_min: 1.5,
                chi_square_min: 2.706,
                n_min: 2,
                ecs_min: 1.5,
                lower_credibility_min: -0.5,
            },
        }
    }

    /// Check if BDI result meets thresholds.
    #[must_use]
    pub fn bdi_meets_threshold(&self, bdi: f64, chi_square: f64, n: u32) -> bool {
        bdi >= self.bdi_min && chi_square >= self.chi_square_min && n >= self.n_min
    }

    /// Check if ECS result meets thresholds.
    #[must_use]
    pub fn ecs_meets_threshold(&self, ecs: f64, lower_credibility: f64) -> bool {
        ecs >= self.ecs_min && lower_credibility >= self.lower_credibility_min
    }
}

impl Default for BettingThresholds {
    fn default() -> Self {
        Self::from_preset(ThresholdPreset::Evans)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_strength_from_bdi() {
        assert_eq!(SignalStrength::from_bdi(7.0), SignalStrength::Elite);
        assert_eq!(SignalStrength::from_bdi(5.0), SignalStrength::Strong);
        assert_eq!(SignalStrength::from_bdi(2.5), SignalStrength::Moderate);
        assert_eq!(SignalStrength::from_bdi(1.2), SignalStrength::Weak);
        assert_eq!(SignalStrength::from_bdi(0.5), SignalStrength::Avoid);
    }

    #[test]
    fn test_thresholds_evans() {
        let t = BettingThresholds::from_preset(ThresholdPreset::Evans);
        assert!(t.bdi_meets_threshold(2.5, 4.0, 5));
        assert!(!t.bdi_meets_threshold(1.5, 4.0, 5)); // BDI too low
        assert!(!t.bdi_meets_threshold(2.5, 3.0, 5)); // Chi-square too low
        assert!(!t.bdi_meets_threshold(2.5, 4.0, 2)); // N too low
    }
}
