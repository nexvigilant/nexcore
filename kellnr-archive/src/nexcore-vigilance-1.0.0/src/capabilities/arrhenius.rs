//! Arrhenius Equation → Adoption Potential
//!
//! Chemistry: k = A × e^(-Ea/RT)
//! Capability: Skill_adoption_rate = Max × e^(-Barrier / (Motivation × Resources))
//!
//! Maps activation energy to learning barriers.

use super::types::NormalizedScore;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Learning barrier level (1-10)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct LearningBarrier(f64);

impl LearningBarrier {
    /// Create a new learning barrier (clamped to 1-10)
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value.clamp(1.0, 10.0))
    }

    #[must_use]
    pub const fn value(&self) -> f64 {
        self.0
    }

    /// Interpret barrier level
    #[must_use]
    pub fn interpretation(&self) -> BarrierLevel {
        match self.0 as u8 {
            1..=3 => BarrierLevel::Low,
            4..=6 => BarrierLevel::Moderate,
            7..=8 => BarrierLevel::High,
            _ => BarrierLevel::Extreme,
        }
    }
}

impl From<f64> for LearningBarrier {
    fn from(value: f64) -> Self {
        Self::new(value)
    }
}

/// Barrier level categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BarrierLevel {
    Low,
    Moderate,
    High,
    Extreme,
}

impl fmt::Display for BarrierLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Low => write!(f, "Low (easy adoption)"),
            Self::Moderate => write!(f, "Moderate (training needed)"),
            Self::High => write!(f, "High (significant investment)"),
            Self::Extreme => write!(f, "Extreme (expert only)"),
        }
    }
}

/// Adoption potential calculated via Arrhenius model
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AdoptionPotential {
    /// Raw adoption rate
    pub rate: f64,
    /// Normalized score [0, 1]
    pub normalized: NormalizedScore,
    /// Input barrier
    pub barrier: LearningBarrier,
    /// Motivation factor used
    pub motivation: f64,
    /// Resources factor used
    pub resources: f64,
}

impl AdoptionPotential {
    /// Calculate adoption potential using Arrhenius model
    ///
    /// # Arguments
    /// * `barrier` - Learning barrier (1-10, higher = harder)
    /// * `motivation` - Motivation factor (0.5-1.5, urgency/incentives)
    /// * `resources` - Resource availability (0.5-1.5, time/tools/budget)
    #[must_use]
    pub fn calculate(barrier: f64, motivation: f64, resources: f64) -> Self {
        let barrier = LearningBarrier::new(barrier);
        let motivation = motivation.clamp(0.5, 1.5);
        let resources = resources.clamp(0.5, 1.5);

        // Arrhenius: k = A × e^(-Ea/RT)
        // A = 1.0 (max adoption = 100%)
        // Ea = barrier (scaled)
        // R×T = motivation × resources (environmental factor)
        let exponent = -barrier.value() / (motivation * resources * 10.0);
        let rate = exponent.exp();

        // Normalize to [0, 1] - rate is already in this range for typical inputs
        let normalized = NormalizedScore::new(rate);

        Self {
            rate,
            normalized,
            barrier,
            motivation,
            resources,
        }
    }

    /// Quick calculation with default motivation/resources
    #[must_use]
    pub fn from_barrier(barrier: f64) -> Self {
        Self::calculate(barrier, 1.0, 1.0)
    }

    /// Get adoption timeline estimate (months)
    #[must_use]
    pub fn estimated_months(&self) -> f64 {
        // Higher potential = faster adoption
        // Base: 12 months at 0.5 potential
        if self.normalized.value() < 0.01 {
            return 36.0; // 3 years for extremely hard capabilities
        }
        6.0 / self.normalized.value()
    }
}

impl fmt::Display for AdoptionPotential {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Adoption: {} (barrier: {}, ~{:.0} months)",
            self.normalized,
            self.barrier.interpretation(),
            self.estimated_months()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adoption_low_barrier() {
        let adoption = AdoptionPotential::calculate(2.0, 1.2, 1.0);
        assert!(adoption.normalized.value() > 0.7);
        assert!(adoption.estimated_months() < 12.0);
    }

    #[test]
    fn test_adoption_high_barrier() {
        let adoption = AdoptionPotential::calculate(9.0, 0.8, 0.8);
        assert!(adoption.normalized.value() < 0.3);
        assert!(adoption.estimated_months() > 12.0);
    }

    #[test]
    fn test_motivation_effect() {
        let low_motivation = AdoptionPotential::calculate(5.0, 0.5, 1.0);
        let high_motivation = AdoptionPotential::calculate(5.0, 1.5, 1.0);
        assert!(high_motivation.normalized.value() > low_motivation.normalized.value());
    }
}
