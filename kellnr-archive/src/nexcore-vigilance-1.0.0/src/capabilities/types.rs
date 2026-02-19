//! Core capability types

use serde::{Deserialize, Serialize};
use std::fmt;

/// Capability type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CapabilityType {
    /// Technical infrastructure and tooling
    Technical,
    /// Human skills and expertise
    Human,
    /// Financial resources and budget
    Financial,
    /// Market access and relationships
    MarketAccess,
    /// Innovation and R&D capacity
    Innovation,
    /// Operational processes
    Operational,
    /// Regulatory and compliance
    Regulatory,
}

impl fmt::Display for CapabilityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Technical => write!(f, "Technical"),
            Self::Human => write!(f, "Human Capital"),
            Self::Financial => write!(f, "Financial"),
            Self::MarketAccess => write!(f, "Market Access"),
            Self::Innovation => write!(f, "Innovation"),
            Self::Operational => write!(f, "Operational"),
            Self::Regulatory => write!(f, "Regulatory"),
        }
    }
}

/// Normalized score in [0, 1]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct NormalizedScore(f64);

impl NormalizedScore {
    /// Create a new normalized score, clamping to [0, 1]
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    /// Get raw value
    #[must_use]
    pub const fn value(&self) -> f64 {
        self.0
    }

    /// Convert to percentage
    #[must_use]
    pub fn as_percentage(&self) -> f64 {
        self.0 * 100.0
    }
}

impl From<f64> for NormalizedScore {
    fn from(value: f64) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for NormalizedScore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.1}%", self.as_percentage())
    }
}

/// Raw capability metrics before SQI calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityMetrics {
    /// Learning barrier (1-10, higher = harder)
    pub learning_barrier: f64,
    /// Motivation factor (0.5-1.5)
    pub motivation: f64,
    /// Resource availability (0.5-1.5)
    pub resources: f64,
    /// Maximum capacity
    pub max_capacity: f64,
    /// Current demand/load
    pub current_demand: f64,
    /// Half-saturation point
    pub half_saturation: f64,
    /// Hill coefficient (synergy factor)
    pub hill_coefficient: f64,
    /// Skill count for synergy calculation
    pub skill_count: f64,
    /// Synergy threshold
    pub synergy_threshold: f64,
    /// Stabilizing factors score
    pub stabilizing_factors: f64,
    /// Destabilizing factors score
    pub destabilizing_factors: f64,
    /// Days since last update
    pub days_since_update: f64,
    /// Half-life in days
    pub half_life_days: f64,
}

impl Default for CapabilityMetrics {
    fn default() -> Self {
        Self {
            learning_barrier: 5.0,
            motivation: 1.0,
            resources: 1.0,
            max_capacity: 100.0,
            current_demand: 50.0,
            half_saturation: 75.0,
            hill_coefficient: 1.0,
            skill_count: 1.0,
            synergy_threshold: 1.0,
            stabilizing_factors: 5.0,
            destabilizing_factors: 5.0,
            days_since_update: 30.0,
            half_life_days: 365.0,
        }
    }
}

/// A capability with full assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    /// Capability name
    pub name: String,
    /// Type classification
    pub capability_type: CapabilityType,
    /// Description
    pub description: String,
    /// Raw metrics
    pub metrics: CapabilityMetrics,
    /// Current maturity level (1-5)
    pub maturity_level: u8,
    /// Strategic importance (1-10)
    pub strategic_importance: f64,
}

impl Capability {
    /// Create a new capability
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        capability_type: CapabilityType,
        description: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            capability_type,
            description: description.into(),
            metrics: CapabilityMetrics::default(),
            maturity_level: 3,
            strategic_importance: 5.0,
        }
    }

    /// Set metrics
    #[must_use]
    pub fn with_metrics(mut self, metrics: CapabilityMetrics) -> Self {
        self.metrics = metrics;
        self
    }

    /// Set maturity level
    #[must_use]
    pub fn with_maturity(mut self, level: u8) -> Self {
        self.maturity_level = level.clamp(1, 5);
        self
    }

    /// Set strategic importance
    #[must_use]
    pub fn with_importance(mut self, importance: f64) -> Self {
        self.strategic_importance = importance.clamp(1.0, 10.0);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalized_score() {
        assert_eq!(NormalizedScore::new(0.5).value(), 0.5);
        assert_eq!(NormalizedScore::new(1.5).value(), 1.0); // Clamped
        assert_eq!(NormalizedScore::new(-0.5).value(), 0.0); // Clamped
    }

    #[test]
    fn test_capability_creation() {
        let cap = Capability::new(
            "Signal Detection",
            CapabilityType::Technical,
            "PV signal detection algorithms",
        )
        .with_maturity(4)
        .with_importance(9.0);

        assert_eq!(cap.maturity_level, 4);
        assert_eq!(cap.strategic_importance, 9.0);
    }
}
