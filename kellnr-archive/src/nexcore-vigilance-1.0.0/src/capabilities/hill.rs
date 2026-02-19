//! Hill Equation → Synergy Coefficient
//!
//! Chemistry: Y = [L]^n / (Kd^n + [L]^n)
//! Capability: Synergy = Skills^n / (Threshold^n + Skills^n)
//!
//! Models cooperative effects when combining capabilities.

use super::types::NormalizedScore;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Cooperativity type based on Hill coefficient
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CooperativityType {
    /// n < 1: Skills interfere with each other
    AntiCooperative,
    /// n = 1: Linear addition (no synergy)
    Independent,
    /// 1 < n < 2: Moderate synergy
    Cooperative,
    /// n >= 2: Strong multiplicative effects
    HighlyCooperative,
    /// n >= 4: Critical mass / phase transition
    UltraCooperative,
}

impl CooperativityType {
    #[must_use]
    pub fn from_coefficient(n: f64) -> Self {
        match n {
            x if x < 0.8 => Self::AntiCooperative,
            x if x < 1.2 => Self::Independent,
            x if x < 2.0 => Self::Cooperative,
            x if x < 4.0 => Self::HighlyCooperative,
            _ => Self::UltraCooperative,
        }
    }

    /// Strategic implication
    #[must_use]
    pub const fn implication(&self) -> &'static str {
        match self {
            Self::AntiCooperative => "Focus on one skill, avoid overloading",
            Self::Independent => "Linear investment, predictable returns",
            Self::Cooperative => "Combo benefits, invest in pairs",
            Self::HighlyCooperative => "Stack skills for multiplier effect",
            Self::UltraCooperative => "Reach critical mass for breakthrough",
        }
    }
}

impl fmt::Display for CooperativityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AntiCooperative => write!(f, "Anti-cooperative (n<1)"),
            Self::Independent => write!(f, "Independent (n≈1)"),
            Self::Cooperative => write!(f, "Cooperative (n<2)"),
            Self::HighlyCooperative => write!(f, "Highly cooperative (n<4)"),
            Self::UltraCooperative => write!(f, "Ultra-cooperative (n≥4)"),
        }
    }
}

/// Synergy coefficient via Hill equation
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SynergyCoefficient {
    /// Hill coefficient (n)
    pub hill_n: f64,
    /// Current skill/resource count
    pub skill_count: f64,
    /// Threshold for half-maximal effect (Kd)
    pub threshold: f64,
    /// Fractional saturation (Y)
    pub saturation: f64,
    /// Cooperativity type
    pub cooperativity: CooperativityType,
    /// Normalized score [0, 1]
    pub normalized: NormalizedScore,
}

impl SynergyCoefficient {
    /// Calculate synergy using Hill equation
    ///
    /// # Arguments
    /// * `hill_n` - Hill coefficient (cooperativity factor)
    /// * `skill_count` - Number of skills/resources ([L])
    /// * `threshold` - Skills needed for 50% effect (Kd)
    #[must_use]
    pub fn calculate(hill_n: f64, skill_count: f64, threshold: f64) -> Self {
        let hill_n = hill_n.max(0.1);
        let skill_count = skill_count.max(0.0);
        let threshold = threshold.max(0.1);

        // Hill equation: Y = [L]^n / (Kd^n + [L]^n)
        let l_n = skill_count.powf(hill_n);
        let kd_n = threshold.powf(hill_n);
        let saturation = l_n / (kd_n + l_n);

        let cooperativity = CooperativityType::from_coefficient(hill_n);
        let normalized = NormalizedScore::new(saturation);

        Self {
            hill_n,
            skill_count,
            threshold,
            saturation,
            cooperativity,
            normalized,
        }
    }

    /// Calculate the derivative (sensitivity to adding more skills)
    #[must_use]
    pub fn sensitivity(&self) -> f64 {
        // dY/d[L] = n × Kd^n × [L]^(n-1) / (Kd^n + [L]^n)²
        if self.skill_count < 0.01 {
            return self.hill_n; // Initial slope
        }
        let l = self.skill_count;
        let kd = self.threshold;
        let n = self.hill_n;
        let kd_n = kd.powf(n);
        let l_n = l.powf(n);
        n * kd_n * l.powf(n - 1.0) / (kd_n + l_n).powi(2)
    }

    /// Skills needed to reach target saturation
    #[must_use]
    pub fn skills_for_target(&self, target: f64) -> f64 {
        // Solve for [L]: [L] = Kd × (Y / (1-Y))^(1/n)
        let target = target.clamp(0.01, 0.99);
        let ratio = target / (1.0 - target);
        self.threshold * ratio.powf(1.0 / self.hill_n)
    }

    /// Multiplier effect at current level vs linear baseline
    #[must_use]
    pub fn multiplier_vs_linear(&self) -> f64 {
        // Compare actual saturation to what linear (n=1) would give
        let linear_saturation = self.skill_count / (self.threshold + self.skill_count);
        if linear_saturation > 0.01 {
            self.saturation / linear_saturation
        } else {
            1.0
        }
    }
}

impl fmt::Display for SynergyCoefficient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Synergy: {} ({:.1}x multiplier, {} skills)",
            self.cooperativity,
            self.multiplier_vs_linear(),
            self.skill_count
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_hill() {
        // n=1 should give standard hyperbolic response
        let syn = SynergyCoefficient::calculate(1.0, 5.0, 5.0);
        assert!((syn.saturation - 0.5).abs() < 0.01);
        assert_eq!(syn.cooperativity, CooperativityType::Independent);
    }

    #[test]
    fn test_cooperative_hill() {
        // n=2 should give sigmoidal response
        let syn = SynergyCoefficient::calculate(2.0, 5.0, 5.0);
        assert!((syn.saturation - 0.5).abs() < 0.01); // At threshold
        assert_eq!(syn.cooperativity, CooperativityType::HighlyCooperative);
    }

    #[test]
    fn test_steepness() {
        // Higher n = steeper transition
        let low_n = SynergyCoefficient::calculate(1.0, 3.0, 5.0);
        let high_n = SynergyCoefficient::calculate(4.0, 3.0, 5.0);
        // Below threshold, high n should give lower saturation
        assert!(high_n.saturation < low_n.saturation);
    }

    #[test]
    fn test_skills_for_target() {
        let syn = SynergyCoefficient::calculate(2.0, 1.0, 5.0);
        let needed = syn.skills_for_target(0.9);
        // Verify by plugging back
        let check = SynergyCoefficient::calculate(2.0, needed, 5.0);
        assert!((check.saturation - 0.9).abs() < 0.01);
    }
}
