//! Skill Quality Index (SQI) — Unified Capability Assessment
//!
//! Combines all chemistry equations into a single score:
//! ```text
//! SQI = (Adoption×0.20 + Capacity×0.25 + Synergy×0.20 + Stability×0.20 + Freshness×0.15) × 10
//! ```

use super::{
    arrhenius::AdoptionPotential,
    capacity::CapacityEfficiency,
    decay::FreshnessFactor,
    hill::SynergyCoefficient,
    stability::StabilityScore,
    types::{Capability, CapabilityMetrics, NormalizedScore},
};
use serde::{Deserialize, Serialize};
use std::fmt;

/// SQI component weights
pub const WEIGHT_ADOPTION: f64 = 0.20;
pub const WEIGHT_CAPACITY: f64 = 0.25;
pub const WEIGHT_SYNERGY: f64 = 0.20;
pub const WEIGHT_STABILITY: f64 = 0.20;
pub const WEIGHT_FRESHNESS: f64 = 0.15;

/// SQI rating categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SqiRating {
    Critical,  // 0-2
    Weak,      // 2-4
    Adequate,  // 4-6
    Good,      // 6-8
    Excellent, // 8-10
}

impl SqiRating {
    #[must_use]
    pub fn from_score(score: f64) -> Self {
        match score {
            s if s < 2.0 => Self::Critical,
            s if s < 4.0 => Self::Weak,
            s if s < 6.0 => Self::Adequate,
            s if s < 8.0 => Self::Good,
            _ => Self::Excellent,
        }
    }

    #[must_use]
    pub const fn action(&self) -> &'static str {
        match self {
            Self::Critical => "Build or acquire immediately",
            Self::Weak => "Major development needed",
            Self::Adequate => "Invest in gaps",
            Self::Good => "Maintain, minor improvements",
            Self::Excellent => "Leverage as competitive advantage",
        }
    }

    #[must_use]
    pub const fn emoji(&self) -> &'static str {
        match self {
            Self::Critical => "🔴",
            Self::Weak => "🟠",
            Self::Adequate => "🟡",
            Self::Good => "🟢",
            Self::Excellent => "⭐",
        }
    }
}

impl fmt::Display for SqiRating {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Critical => write!(f, "Critical (0-2)"),
            Self::Weak => write!(f, "Weak (2-4)"),
            Self::Adequate => write!(f, "Adequate (4-6)"),
            Self::Good => write!(f, "Good (6-8)"),
            Self::Excellent => write!(f, "Excellent (8-10)"),
        }
    }
}

/// Skill Quality Index
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SkillQualityIndex {
    /// Final SQI score (0-10)
    pub score: f64,
    /// Rating category
    pub rating: SqiRating,
    /// Individual components
    pub adoption: NormalizedScore,
    pub capacity: NormalizedScore,
    pub synergy: NormalizedScore,
    pub stability: NormalizedScore,
    pub freshness: NormalizedScore,
}

impl SkillQualityIndex {
    /// Calculate SQI from normalized component scores
    #[must_use]
    pub fn from_components(
        adoption: NormalizedScore,
        capacity: NormalizedScore,
        synergy: NormalizedScore,
        stability: NormalizedScore,
        freshness: NormalizedScore,
    ) -> Self {
        let weighted_sum = adoption.value() * WEIGHT_ADOPTION
            + capacity.value() * WEIGHT_CAPACITY
            + synergy.value() * WEIGHT_SYNERGY
            + stability.value() * WEIGHT_STABILITY
            + freshness.value() * WEIGHT_FRESHNESS;

        let score = weighted_sum * 10.0;
        let rating = SqiRating::from_score(score);

        Self {
            score,
            rating,
            adoption,
            capacity,
            synergy,
            stability,
            freshness,
        }
    }

    /// Calculate SQI from raw metrics
    #[must_use]
    pub fn from_metrics(metrics: &CapabilityMetrics) -> Self {
        let adoption = AdoptionPotential::calculate(
            metrics.learning_barrier,
            metrics.motivation,
            metrics.resources,
        );

        let capacity = CapacityEfficiency::calculate(
            metrics.max_capacity,
            metrics.current_demand,
            metrics.half_saturation,
        );

        let synergy = SynergyCoefficient::calculate(
            metrics.hill_coefficient,
            metrics.skill_count,
            metrics.synergy_threshold,
        );

        let stability =
            StabilityScore::calculate(metrics.stabilizing_factors, metrics.destabilizing_factors);

        let freshness =
            FreshnessFactor::calculate(metrics.days_since_update, metrics.half_life_days);

        Self::from_components(
            adoption.normalized,
            capacity.normalized,
            synergy.normalized,
            stability.normalized,
            freshness.normalized,
        )
    }

    /// Identify weakest component
    #[must_use]
    pub fn weakest_component(&self) -> (&'static str, f64) {
        let components = [
            ("Adoption", self.adoption.value()),
            ("Capacity", self.capacity.value()),
            ("Synergy", self.synergy.value()),
            ("Stability", self.stability.value()),
            ("Freshness", self.freshness.value()),
        ];
        components
            .into_iter()
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .unwrap_or(("Unknown", 0.0))
    }

    /// Identify strongest component
    #[must_use]
    pub fn strongest_component(&self) -> (&'static str, f64) {
        let components = [
            ("Adoption", self.adoption.value()),
            ("Capacity", self.capacity.value()),
            ("Synergy", self.synergy.value()),
            ("Stability", self.stability.value()),
            ("Freshness", self.freshness.value()),
        ];
        components
            .into_iter()
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .unwrap_or(("Unknown", 0.0))
    }
}

impl fmt::Display for SkillQualityIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SQI: {:.1}/10 {} {}",
            self.score,
            self.rating.emoji(),
            self.rating
        )
    }
}

/// Full capability assessment with all details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityAssessment {
    /// Capability being assessed
    pub capability: Capability,
    /// Overall SQI
    pub sqi: SkillQualityIndex,
    /// Detailed component results
    pub adoption_detail: AdoptionPotential,
    pub capacity_detail: CapacityEfficiency,
    pub synergy_detail: SynergyCoefficient,
    pub stability_detail: StabilityScore,
    pub freshness_detail: FreshnessFactor,
}

impl CapabilityAssessment {
    /// Perform full capability assessment
    #[must_use]
    pub fn assess(capability: Capability) -> Self {
        let m = &capability.metrics;

        let adoption_detail =
            AdoptionPotential::calculate(m.learning_barrier, m.motivation, m.resources);

        let capacity_detail =
            CapacityEfficiency::calculate(m.max_capacity, m.current_demand, m.half_saturation);

        let synergy_detail =
            SynergyCoefficient::calculate(m.hill_coefficient, m.skill_count, m.synergy_threshold);

        let stability_detail =
            StabilityScore::calculate(m.stabilizing_factors, m.destabilizing_factors);

        let freshness_detail = FreshnessFactor::calculate(m.days_since_update, m.half_life_days);

        let sqi = SkillQualityIndex::from_components(
            adoption_detail.normalized,
            capacity_detail.normalized,
            synergy_detail.normalized,
            stability_detail.normalized,
            freshness_detail.normalized,
        );

        Self {
            capability,
            sqi,
            adoption_detail,
            capacity_detail,
            synergy_detail,
            stability_detail,
            freshness_detail,
        }
    }

    /// Generate assessment report
    #[must_use]
    pub fn report(&self) -> String {
        let (weak_name, weak_val) = self.sqi.weakest_component();
        let (strong_name, strong_val) = self.sqi.strongest_component();

        format!(
            "═══════════════════════════════════════════════════════════\n\
             CAPABILITY ASSESSMENT: {}\n\
             ═══════════════════════════════════════════════════════════\n\
             \n\
             Overall: {} — {}\n\
             \n\
             Components:\n\
             ├─ Adoption:   {:.1}% (barrier: {})\n\
             ├─ Capacity:   {:.1}% (zone: {})\n\
             ├─ Synergy:    {:.1}% ({})\n\
             ├─ Stability:  {:.1}% (risk: {})\n\
             └─ Freshness:  {:.1}% (urgency: {})\n\
             \n\
             Strongest: {} ({:.0}%)\n\
             Weakest:   {} ({:.0}%) ← Focus here\n\
             \n\
             Recommendation: {}\n\
             ═══════════════════════════════════════════════════════════",
            self.capability.name,
            self.sqi,
            self.sqi.rating.action(),
            self.adoption_detail.normalized.as_percentage(),
            self.adoption_detail.barrier.interpretation(),
            self.capacity_detail.normalized.as_percentage(),
            self.capacity_detail.zone,
            self.synergy_detail.normalized.as_percentage(),
            self.synergy_detail.cooperativity,
            self.stability_detail.normalized.as_percentage(),
            self.stability_detail.risk,
            self.freshness_detail.normalized.as_percentage(),
            self.freshness_detail.urgency,
            strong_name,
            strong_val * 100.0,
            weak_name,
            weak_val * 100.0,
            self.sqi.rating.action()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capabilities::types::CapabilityType;

    #[test]
    fn test_sqi_calculation() {
        let sqi = SkillQualityIndex::from_components(
            NormalizedScore::new(0.75),
            NormalizedScore::new(0.62),
            NormalizedScore::new(0.70),
            NormalizedScore::new(0.85),
            NormalizedScore::new(0.93),
        );
        // (0.75×0.20 + 0.62×0.25 + 0.70×0.20 + 0.85×0.20 + 0.93×0.15) × 10
        // = (0.15 + 0.155 + 0.14 + 0.17 + 0.1395) × 10 = 7.545
        assert!((sqi.score - 7.5).abs() < 0.1);
        assert_eq!(sqi.rating, SqiRating::Good);
    }

    #[test]
    fn test_full_assessment() {
        let cap = Capability::new(
            "Signal Detection",
            CapabilityType::Technical,
            "PV signal detection algorithms",
        )
        .with_metrics(CapabilityMetrics {
            learning_barrier: 6.0,
            motivation: 1.2,
            resources: 1.0,
            max_capacity: 10000.0,
            current_demand: 5000.0,
            half_saturation: 8000.0,
            hill_coefficient: 1.5,
            skill_count: 5.0,
            synergy_threshold: 3.0,
            stabilizing_factors: 8.0,
            destabilizing_factors: 3.0,
            days_since_update: 60.0,
            half_life_days: 548.0,
        });

        let assessment = CapabilityAssessment::assess(cap);
        assert!(assessment.sqi.score > 5.0);
        let report = assessment.report();
        assert!(report.contains("Signal Detection"));
    }
}
