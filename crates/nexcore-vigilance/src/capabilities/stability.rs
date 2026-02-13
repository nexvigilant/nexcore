//! Henderson-Hasselbalch → Stability Buffer
//!
//! Chemistry: pH = pKa + log([Base]/[Acid])
//! Capability: Stability = Baseline + log(Stabilizing / Destabilizing)

use super::types::NormalizedScore;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Buffer ratio (stabilizing / destabilizing)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct BufferRatio(f64);

impl BufferRatio {
    #[must_use]
    pub fn new(stabilizing: f64, destabilizing: f64) -> Self {
        let ratio = stabilizing.max(0.01) / destabilizing.max(0.01);
        Self(ratio)
    }

    #[must_use]
    pub const fn value(&self) -> f64 {
        self.0
    }

    #[must_use]
    pub fn risk_level(&self) -> RiskLevel {
        match self.0 {
            r if r < 0.1 => RiskLevel::Critical,
            r if r < 1.0 => RiskLevel::High,
            r if r < 3.0 => RiskLevel::Medium,
            r if r < 10.0 => RiskLevel::Low,
            _ => RiskLevel::Minimal,
        }
    }
}

/// Risk level from buffer ratio
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Critical,
    High,
    Medium,
    Low,
    Minimal,
}

impl fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Critical => write!(f, "🔴 Critical"),
            Self::High => write!(f, "🟠 High"),
            Self::Medium => write!(f, "🟡 Medium"),
            Self::Low => write!(f, "🟢 Low"),
            Self::Minimal => write!(f, "✅ Minimal"),
        }
    }
}

/// Stability score via Henderson-Hasselbalch model
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StabilityScore {
    /// Stabilizing factors (docs, automation, redundancy)
    pub stabilizing: f64,
    /// Destabilizing factors (turnover, debt, dependencies)
    pub destabilizing: f64,
    /// Buffer ratio
    pub ratio: BufferRatio,
    /// Raw stability score
    pub raw_score: f64,
    /// Normalized [0, 1]
    pub normalized: NormalizedScore,
    /// Risk level
    pub risk: RiskLevel,
}

impl StabilityScore {
    /// Calculate stability using Henderson-Hasselbalch model
    #[must_use]
    pub fn calculate(stabilizing: f64, destabilizing: f64) -> Self {
        let stabilizing = stabilizing.max(0.1);
        let destabilizing = destabilizing.max(0.1);
        let ratio = BufferRatio::new(stabilizing, destabilizing);

        // Henderson-Hasselbalch: pH = pKa + log([A-]/[HA])
        // Stability = 0.5 (neutral) + log10(ratio) scaled
        let raw_score = 0.5 + (ratio.value().log10() / 4.0);
        let normalized = NormalizedScore::new(raw_score);
        let risk = ratio.risk_level();

        Self {
            stabilizing,
            destabilizing,
            ratio,
            raw_score,
            normalized,
            risk,
        }
    }

    /// Factors needed to reach target stability
    #[must_use]
    pub fn stabilizing_needed_for(&self, target: f64) -> f64 {
        // Solve: target = 0.5 + log10(S/D)/4
        // S/D = 10^((target-0.5)*4)
        // S = D × 10^((target-0.5)*4)
        let target_ratio = 10_f64.powf((target - 0.5) * 4.0);
        self.destabilizing * target_ratio
    }
}

impl fmt::Display for StabilityScore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Stability: {} (ratio: {:.1}, risk: {})",
            self.normalized,
            self.ratio.value(),
            self.risk
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_balanced() {
        let s = StabilityScore::calculate(5.0, 5.0);
        assert!((s.normalized.value() - 0.5).abs() < 0.1);
    }

    #[test]
    fn test_stable() {
        let s = StabilityScore::calculate(10.0, 1.0);
        assert!(s.normalized.value() > 0.7);
        assert!(matches!(s.risk, RiskLevel::Low | RiskLevel::Minimal));
    }

    #[test]
    fn test_unstable() {
        let s = StabilityScore::calculate(1.0, 10.0);
        assert!(s.normalized.value() < 0.3);
        assert!(matches!(s.risk, RiskLevel::High | RiskLevel::Critical));
    }
}
