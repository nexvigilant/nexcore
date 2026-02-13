//! Half-Life Decay → Freshness Factor
//!
//! Chemistry: N(t) = N₀ × e^(-λt)
//! Capability: Relevance(t) = Initial × e^(-decay_rate × t)

use super::types::NormalizedScore;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Capability half-life in days
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct CapabilityHalfLife(f64);

impl CapabilityHalfLife {
    #[must_use]
    pub fn new(days: f64) -> Self {
        Self(days.max(1.0))
    }

    #[must_use]
    pub const fn days(&self) -> f64 {
        self.0
    }

    /// Decay rate (λ = ln(2) / half_life)
    #[must_use]
    pub fn decay_rate(&self) -> f64 {
        0.693147 / self.0
    }

    /// Common half-lives by capability type
    #[must_use]
    pub fn for_type(cap_type: CapabilityDecayType) -> Self {
        Self::new(cap_type.typical_half_life_days())
    }
}

/// Capability decay type with typical half-lives
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityDecayType {
    /// Specific tool/API knowledge (1-2 years)
    ToolSpecific,
    /// Framework knowledge (2-3 years)
    Framework,
    /// Domain expertise (5-10 years)
    DomainExpertise,
    /// Foundational (math, logic) - essentially infinite
    Foundational,
    /// Regulatory knowledge (changes frequently)
    Regulatory,
    /// Market knowledge (very fast decay)
    Market,
}

impl CapabilityDecayType {
    #[must_use]
    pub const fn typical_half_life_days(&self) -> f64 {
        match self {
            Self::ToolSpecific => 548.0,     // ~1.5 years
            Self::Framework => 912.0,        // ~2.5 years
            Self::DomainExpertise => 2555.0, // ~7 years
            Self::Foundational => 36500.0,   // ~100 years (effectively infinite)
            Self::Regulatory => 365.0,       // ~1 year
            Self::Market => 180.0,           // ~6 months
        }
    }

    #[must_use]
    pub const fn refresh_cadence(&self) -> &'static str {
        match self {
            Self::ToolSpecific => "Quarterly",
            Self::Framework => "Annually",
            Self::DomainExpertise => "Biennial",
            Self::Foundational => "Never",
            Self::Regulatory => "Monthly",
            Self::Market => "Weekly",
        }
    }
}

impl fmt::Display for CapabilityDecayType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ToolSpecific => write!(f, "Tool-specific"),
            Self::Framework => write!(f, "Framework"),
            Self::DomainExpertise => write!(f, "Domain expertise"),
            Self::Foundational => write!(f, "Foundational"),
            Self::Regulatory => write!(f, "Regulatory"),
            Self::Market => write!(f, "Market"),
        }
    }
}

/// Freshness factor via exponential decay
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FreshnessFactor {
    /// Days since last update
    pub days_elapsed: f64,
    /// Half-life
    pub half_life: CapabilityHalfLife,
    /// Remaining relevance (0-1)
    pub relevance: f64,
    /// Normalized score
    pub normalized: NormalizedScore,
    /// Refresh urgency
    pub urgency: RefreshUrgency,
}

impl FreshnessFactor {
    /// Calculate freshness using exponential decay
    #[must_use]
    pub fn calculate(days_elapsed: f64, half_life_days: f64) -> Self {
        let days_elapsed = days_elapsed.max(0.0);
        let half_life = CapabilityHalfLife::new(half_life_days);

        // N(t) = N₀ × e^(-λt), where λ = ln(2)/half_life
        let decay_rate = half_life.decay_rate();
        let relevance = (-decay_rate * days_elapsed).exp();
        let normalized = NormalizedScore::new(relevance);
        let urgency = RefreshUrgency::from_relevance(relevance);

        Self {
            days_elapsed,
            half_life,
            relevance,
            normalized,
            urgency,
        }
    }

    /// Days until relevance drops to threshold
    #[must_use]
    pub fn days_until(&self, threshold: f64) -> f64 {
        // Solve: threshold = e^(-λt)
        // t = -ln(threshold) / λ
        let threshold = threshold.clamp(0.01, 0.99);
        -threshold.ln() / self.half_life.decay_rate() - self.days_elapsed
    }

    /// Half-lives elapsed
    #[must_use]
    pub fn half_lives_elapsed(&self) -> f64 {
        self.days_elapsed / self.half_life.days()
    }
}

impl fmt::Display for FreshnessFactor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Freshness: {} ({:.0} days old, {} urgency)",
            self.normalized, self.days_elapsed, self.urgency
        )
    }
}

/// Refresh urgency level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RefreshUrgency {
    None,
    Low,
    Medium,
    High,
    Critical,
}

impl RefreshUrgency {
    #[must_use]
    pub fn from_relevance(relevance: f64) -> Self {
        match relevance {
            r if r > 0.9 => Self::None,
            r if r > 0.7 => Self::Low,
            r if r > 0.5 => Self::Medium,
            r if r > 0.25 => Self::High,
            _ => Self::Critical,
        }
    }
}

impl fmt::Display for RefreshUrgency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Low => write!(f, "Low"),
            Self::Medium => write!(f, "Medium"),
            Self::High => write!(f, "High"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fresh() {
        let f = FreshnessFactor::calculate(0.0, 365.0);
        assert!((f.relevance - 1.0).abs() < 0.01);
        assert_eq!(f.urgency, RefreshUrgency::None);
    }

    #[test]
    fn test_one_half_life() {
        let f = FreshnessFactor::calculate(365.0, 365.0);
        assert!((f.relevance - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_two_half_lives() {
        let f = FreshnessFactor::calculate(730.0, 365.0);
        assert!((f.relevance - 0.25).abs() < 0.01);
    }

    #[test]
    fn test_days_until() {
        let f = FreshnessFactor::calculate(0.0, 365.0);
        let days = f.days_until(0.5);
        assert!((days - 365.0).abs() < 1.0);
    }
}
