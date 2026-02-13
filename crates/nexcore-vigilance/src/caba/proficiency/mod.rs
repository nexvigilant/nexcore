//! Proficiency Level System
//!
//! Migrated from Python `domains/regulatory/caba/caba/models/cpa.py`.
//!
//! ## 7-Level PDC Model
//!
//! Progression from rule-following novice to paradigm-creating global influencer:
//!
//! - L1: Novice - Rule-based, requires constant guidance
//! - L2: Advanced Beginner - Pattern recognition emerging
//! - L3: Competent - Independent practice
//! - L4: Proficient - Holistic perception, drives innovation
//! - L5: Expert - Transcends rules, creates knowledge
//! - L5+: Senior Expert - Organizational transformation
//! - L5++: Executive Expert - Global paradigm shifts

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Extended proficiency levels matching PDC 7-level model.
///
/// # L0 Quark - Level enumeration with total ordering
///
/// Safety Axiom: Levels have compile-time numeric values for comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ProficiencyLevel {
    /// L1: Rule-based, requires constant guidance
    #[serde(rename = "L1: Novice")]
    #[default]
    L1Novice,
    /// L2: Pattern recognition emerging
    #[serde(rename = "L2: Advanced Beginner")]
    L2AdvancedBeginner,
    /// L3: Independent practice
    #[serde(rename = "L3: Competent")]
    L3Competent,
    /// L4: Holistic perception, drives innovation
    #[serde(rename = "L4: Proficient")]
    L4Proficient,
    /// L5: Transcends rules, creates knowledge
    #[serde(rename = "L5: Expert")]
    L5Expert,
    /// L5+: Organizational transformation
    #[serde(rename = "L5+: Senior Expert")]
    L5PlusSeniorExpert,
    /// L5++: Global paradigm shifts
    #[serde(rename = "L5++: Executive Expert")]
    L5PlusPlusExecutive,
}

impl ProficiencyLevel {
    /// Get numeric value for comparisons.
    ///
    /// # L1 Atom - Numeric mapping (<20 LOC)
    #[must_use]
    pub const fn numeric_value(&self) -> f64 {
        match self {
            Self::L1Novice => 1.0,
            Self::L2AdvancedBeginner => 2.0,
            Self::L3Competent => 3.0,
            Self::L4Proficient => 4.0,
            Self::L5Expert => 5.0,
            Self::L5PlusSeniorExpert => 5.5,
            Self::L5PlusPlusExecutive => 6.0,
        }
    }

    /// Get display string.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::L1Novice => "L1: Novice",
            Self::L2AdvancedBeginner => "L2: Advanced Beginner",
            Self::L3Competent => "L3: Competent",
            Self::L4Proficient => "L4: Proficient",
            Self::L5Expert => "L5: Expert",
            Self::L5PlusSeniorExpert => "L5+: Senior Expert",
            Self::L5PlusPlusExecutive => "L5++: Executive Expert",
        }
    }

    /// Check if this level is considered "senior" (L5+).
    #[must_use]
    pub const fn is_senior(&self) -> bool {
        matches!(self, Self::L5PlusSeniorExpert | Self::L5PlusPlusExecutive)
    }

    /// Check if this level requires quantitative metrics validation.
    #[must_use]
    pub const fn requires_metrics_validation(&self) -> bool {
        self.numeric_value() >= 4.0
    }
}

impl PartialOrd for ProficiencyLevel {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ProficiencyLevel {
    fn cmp(&self, other: &Self) -> Ordering {
        // Use integer comparison to avoid floating point issues
        let self_int = (self.numeric_value() * 10.0) as i32;
        let other_int = (other.numeric_value() * 10.0) as i32;
        self_int.cmp(&other_int)
    }
}

impl std::fmt::Display for ProficiencyLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Quantitative metrics for proficiency level validation.
///
/// Based on PDC framework specifications for L4-L5++ validation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProficiencyMetrics {
    /// Target proficiency level
    pub proficiency_level: ProficiencyLevel,

    // L4 (Proficient) Metrics
    /// Number of projects led
    #[serde(default)]
    pub projects_led: u32,
    /// Number of professionals impacted
    #[serde(default)]
    pub professionals_impacted: u32,
    /// Efficiency improvement percentage
    #[serde(default)]
    pub efficiency_improvement_pct: f64,
    /// Whether practices influenced beyond organization
    #[serde(default)]
    pub practices_influenced_beyond_org: bool,

    // L5 (Expert) Metrics
    /// Number of frameworks created
    #[serde(default)]
    pub frameworks_created: u32,
    /// Number of organizations that adopted frameworks
    #[serde(default)]
    pub organizations_adopted: u32,
    /// Number of publications
    #[serde(default)]
    pub publications: u32,
    /// Number of citations
    #[serde(default)]
    pub citations: u32,
    /// Professionals mentored to advanced levels
    #[serde(default)]
    pub professionals_mentored_advanced: u32,

    // L5+ (Senior Expert) Metrics
    /// Organizational transformation percentage
    #[serde(default)]
    pub organizational_transformation_pct: f64,
    /// Industry practices influenced
    #[serde(default)]
    pub industry_practices_influenced: u32,
    /// Total professionals mentored
    #[serde(default)]
    pub professionals_mentored_total: u32,
    /// Value created in USD
    #[serde(default)]
    pub value_created_usd: f64,

    // L5++ (Executive Expert) Metrics
    /// Number of countries influenced
    #[serde(default)]
    pub countries_influenced: u32,
    /// Whether policy was influenced
    #[serde(default)]
    pub policy_influence: bool,
    /// Transformation value in USD
    #[serde(default)]
    pub transformation_value_usd: f64,
    /// Legacy duration in years
    #[serde(default)]
    pub legacy_duration_years: u32,
}

impl ProficiencyMetrics {
    /// Validate if metrics support claimed proficiency level.
    ///
    /// # L2 Molecule - Level validation (<50 LOC)
    ///
    /// Returns `true` if quantitative metrics support the claimed level.
    #[must_use]
    pub fn validate_level(&self) -> bool {
        match self.proficiency_level {
            ProficiencyLevel::L4Proficient => {
                self.professionals_impacted > 100
                    && self.efficiency_improvement_pct > 15.0
                    && self.practices_influenced_beyond_org
            }
            ProficiencyLevel::L5Expert => {
                self.organizations_adopted >= 3
                    && self.citations >= 50
                    && self.professionals_mentored_advanced >= 20
            }
            ProficiencyLevel::L5PlusSeniorExpert => {
                self.organizational_transformation_pct > 20.0
                    && self.industry_practices_influenced >= 5
                    && self.professionals_mentored_total >= 50
                    && self.value_created_usd > 10_000_000.0
            }
            ProficiencyLevel::L5PlusPlusExecutive => {
                self.countries_influenced >= 10
                    && self.policy_influence
                    && self.transformation_value_usd > 100_000_000.0
                    && self.legacy_duration_years >= 10
            }
            // Lower levels don't have specific quantitative requirements
            _ => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_ordering() {
        assert!(ProficiencyLevel::L1Novice < ProficiencyLevel::L3Competent);
        assert!(ProficiencyLevel::L5Expert < ProficiencyLevel::L5PlusSeniorExpert);
        assert!(ProficiencyLevel::L5PlusSeniorExpert < ProficiencyLevel::L5PlusPlusExecutive);
    }

    #[test]
    fn test_level_equality() {
        assert_eq!(ProficiencyLevel::L3Competent, ProficiencyLevel::L3Competent);
        assert_ne!(ProficiencyLevel::L4Proficient, ProficiencyLevel::L5Expert);
    }

    #[test]
    fn test_numeric_value() {
        assert!((ProficiencyLevel::L1Novice.numeric_value() - 1.0).abs() < f64::EPSILON);
        assert!((ProficiencyLevel::L5PlusSeniorExpert.numeric_value() - 5.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_metrics_validation_l4() {
        let mut metrics = ProficiencyMetrics {
            proficiency_level: ProficiencyLevel::L4Proficient,
            professionals_impacted: 150,
            efficiency_improvement_pct: 20.0,
            practices_influenced_beyond_org: true,
            ..Default::default()
        };
        assert!(metrics.validate_level());

        metrics.professionals_impacted = 50; // Below threshold
        assert!(!metrics.validate_level());
    }

    #[test]
    fn test_lower_levels_always_valid() {
        let metrics = ProficiencyMetrics {
            proficiency_level: ProficiencyLevel::L2AdvancedBeginner,
            ..Default::default()
        };
        assert!(metrics.validate_level());
    }
}
