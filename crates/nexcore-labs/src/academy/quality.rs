//! Quality Scoring System
//!
//! Migrated from Python `course-builder/course-builder-service/app/models/quality.py`.
//!
//! ## UACA Hierarchy
//!
//! - **L0 Quarks**: Weight constants, threshold constants
//! - **L1 Atoms**: Individual score calculations (<20 LOC)
//! - **L2 Molecules**: Weighted aggregate scoring (<50 LOC)
//!
//! ## Quality Agents
//!
//! Six validation agents with fixed weights:
//! - Content Quality (25%)
//! - Pedagogical Soundness (20%)
//! - Accessibility (15%)
//! - Technical Accuracy (15%)
//! - Engagement (15%)
//! - Assessment Quality (10%)
//!
//! ## Safety Axiom
//!
//! Weights are compile-time constants that sum to exactly 1.0.

use serde::{Deserialize, Serialize};

/// L0 Quark - Quality weight constants.
///
/// Safety Axiom: These weights sum to exactly 1.0.
/// Verification: 0.25 + 0.20 + 0.15 + 0.15 + 0.15 + 0.10 = 1.00
pub mod weights {
    /// Content Quality agent weight
    pub const CONTENT_QUALITY: f64 = 0.25;
    /// Pedagogical Soundness agent weight
    pub const PEDAGOGICAL_SOUNDNESS: f64 = 0.20;
    /// Accessibility agent weight
    pub const ACCESSIBILITY: f64 = 0.15;
    /// Technical Accuracy agent weight
    pub const TECHNICAL_ACCURACY: f64 = 0.15;
    /// Engagement agent weight
    pub const ENGAGEMENT: f64 = 0.15;
    /// Assessment Quality agent weight
    pub const ASSESSMENT_QUALITY: f64 = 0.10;

    /// Sum of all weights (must be 1.0)
    pub const WEIGHT_SUM: f64 = CONTENT_QUALITY
        + PEDAGOGICAL_SOUNDNESS
        + ACCESSIBILITY
        + TECHNICAL_ACCURACY
        + ENGAGEMENT
        + ASSESSMENT_QUALITY;

    /// Default passing threshold (85%)
    pub const DEFAULT_THRESHOLD: f64 = 85.0;
}

// Compile-time verification that weights sum to 1.0
const _: () = {
    // Allow small floating point epsilon
    assert!(weights::WEIGHT_SUM > 0.999 && weights::WEIGHT_SUM < 1.001);
};

/// Bounded quality score type ensuring value is in [0.0, 100.0].
///
/// # L1 Atom - Score validation (<20 LOC)
///
/// Safety Axiom: Score bounds enforced at construction AND deserialization.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize)]
pub struct QualityScore(f64);

impl QualityScore {
    /// Create a new bounded quality score.
    ///
    /// # Errors
    /// Returns error if value is not in [0.0, 100.0] range.
    pub fn new(value: f64) -> Result<Self, QualityScoreError> {
        if !(0.0..=100.0).contains(&value) {
            return Err(QualityScoreError::OutOfBounds { value });
        }
        Ok(Self(value))
    }

    /// Get the inner value.
    #[must_use]
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Check if score passes the default threshold (85%).
    #[must_use]
    pub fn passes_default(&self) -> bool {
        self.0 >= weights::DEFAULT_THRESHOLD
    }

    /// Check if score passes a custom threshold.
    #[must_use]
    pub fn passes(&self, threshold: f64) -> bool {
        self.0 >= threshold
    }

    /// Zero score.
    pub const ZERO: Self = Self(0.0);

    /// Maximum score.
    pub const MAX: Self = Self(100.0);
}

impl Default for QualityScore {
    fn default() -> Self {
        Self::ZERO
    }
}

// Custom deserializer enforcing bounds (Safety Axiom)
impl<'de> Deserialize<'de> for QualityScore {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = f64::deserialize(deserializer)?;
        QualityScore::new(value).map_err(serde::de::Error::custom)
    }
}

/// Error type for quality score validation.
#[derive(Debug, Clone, nexcore_error::Error)]
pub enum QualityScoreError {
    /// Score value is outside [0.0, 100.0] bounds.
    #[error("Quality score {value} is out of bounds [0.0, 100.0]")]
    OutOfBounds {
        /// The invalid value
        value: f64,
    },
}

/// Severity levels for validation issues.
///
/// # L0 Quark - Severity enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ValidationSeverity {
    /// Critical issues that must be fixed
    Critical,
    /// High priority issues
    High,
    /// Medium priority issues
    Medium,
    /// Low priority issues
    Low,
}

impl ValidationSeverity {
    /// Get display string for severity.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Critical => "critical",
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }

    /// Check if this is a blocking severity (critical or high).
    #[must_use]
    pub const fn is_blocking(&self) -> bool {
        matches!(self, Self::Critical | Self::High)
    }
}

/// Quality weights for the six validation agents.
///
/// # L0 Quark - Weight configuration
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct QualityWeights {
    /// Content Quality weight
    pub content_quality: f64,
    /// Pedagogical Soundness weight
    pub pedagogical_soundness: f64,
    /// Accessibility weight
    pub accessibility: f64,
    /// Technical Accuracy weight
    pub technical_accuracy: f64,
    /// Engagement weight
    pub engagement: f64,
    /// Assessment Quality weight
    pub assessment_quality: f64,
}

impl Default for QualityWeights {
    fn default() -> Self {
        Self {
            content_quality: weights::CONTENT_QUALITY,
            pedagogical_soundness: weights::PEDAGOGICAL_SOUNDNESS,
            accessibility: weights::ACCESSIBILITY,
            technical_accuracy: weights::TECHNICAL_ACCURACY,
            engagement: weights::ENGAGEMENT,
            assessment_quality: weights::ASSESSMENT_QUALITY,
        }
    }
}

impl QualityWeights {
    /// Create new weights with validation.
    ///
    /// # Errors
    /// Returns error if:
    /// - Any weight is negative
    /// - Weights don't sum to approximately 1.0
    pub fn new(
        content_quality: f64,
        pedagogical_soundness: f64,
        accessibility: f64,
        technical_accuracy: f64,
        engagement: f64,
        assessment_quality: f64,
    ) -> Result<Self, WeightError> {
        // Safety Axiom: All weights must be non-negative
        let all_weights = [
            content_quality,
            pedagogical_soundness,
            accessibility,
            technical_accuracy,
            engagement,
            assessment_quality,
        ];
        for &w in &all_weights {
            if w < 0.0 {
                return Err(WeightError::NegativeWeight { value: w });
            }
        }

        let sum: f64 = all_weights.iter().sum();
        if (sum - 1.0).abs() > 0.001 {
            return Err(WeightError::InvalidSum { sum });
        }

        Ok(Self {
            content_quality,
            pedagogical_soundness,
            accessibility,
            technical_accuracy,
            engagement,
            assessment_quality,
        })
    }
}

/// Error type for weight validation.
#[derive(Debug, Clone, nexcore_error::Error)]
pub enum WeightError {
    /// Weights don't sum to 1.0.
    #[error("Weights sum to {sum}, expected 1.0")]
    InvalidSum {
        /// The actual sum
        sum: f64,
    },
    /// Weight is negative.
    #[error("Weight {value} is negative, must be >= 0.0")]
    NegativeWeight {
        /// The negative value
        value: f64,
    },
}

/// Scores from all six quality validation agents.
///
/// # L1 Atom - Score container
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct AgentScores {
    /// Content Quality score [0, 100]
    #[serde(default)]
    pub content_quality: Option<QualityScore>,
    /// Pedagogical Soundness score [0, 100]
    #[serde(default)]
    pub pedagogical_soundness: Option<QualityScore>,
    /// Accessibility score [0, 100]
    #[serde(default)]
    pub accessibility: Option<QualityScore>,
    /// Technical Accuracy score [0, 100]
    #[serde(default)]
    pub technical_accuracy: Option<QualityScore>,
    /// Engagement score [0, 100]
    #[serde(default)]
    pub engagement: Option<QualityScore>,
    /// Assessment Quality score [0, 100]
    #[serde(default)]
    pub assessment_quality: Option<QualityScore>,
}

impl AgentScores {
    /// Count how many agents have reported scores.
    #[must_use]
    pub fn agent_count(&self) -> usize {
        [
            self.content_quality.is_some(),
            self.pedagogical_soundness.is_some(),
            self.accessibility.is_some(),
            self.technical_accuracy.is_some(),
            self.engagement.is_some(),
            self.assessment_quality.is_some(),
        ]
        .iter()
        .filter(|&&x| x)
        .count()
    }

    /// Check if all agents have reported.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.agent_count() == 6
    }
}

/// Accumulate a single agent's contribution to weighted score.
///
/// # L1 Atom - Single accumulation (<20 LOC)
#[inline]
fn accumulate_score(
    score: Option<QualityScore>,
    weight: f64,
    weighted_sum: &mut f64,
    weight_sum: &mut f64,
) {
    if let Some(s) = score {
        *weighted_sum += s.value() * weight;
        *weight_sum += weight;
    }
}

/// Calculate weighted overall quality score.
///
/// # L2 Molecule - Weighted average orchestration (<50 LOC)
///
/// This is the core algorithm migrated from Python `QualityReport.calculate_overall_score()`.
/// Delegates to `accumulate_score` L1 atom for each agent.
///
/// # Arguments
/// * `scores` - Individual agent scores
/// * `weights` - Weights for each agent (must sum to 1.0)
///
/// # Returns
/// Weighted average of available scores, or 0.0 if no scores available.
#[must_use]
pub fn calculate_overall_score(scores: &AgentScores, weights: &QualityWeights) -> f64 {
    let mut weighted_sum = 0.0;
    let mut weight_sum = 0.0;

    accumulate_score(
        scores.content_quality,
        weights.content_quality,
        &mut weighted_sum,
        &mut weight_sum,
    );
    accumulate_score(
        scores.pedagogical_soundness,
        weights.pedagogical_soundness,
        &mut weighted_sum,
        &mut weight_sum,
    );
    accumulate_score(
        scores.accessibility,
        weights.accessibility,
        &mut weighted_sum,
        &mut weight_sum,
    );
    accumulate_score(
        scores.technical_accuracy,
        weights.technical_accuracy,
        &mut weighted_sum,
        &mut weight_sum,
    );
    accumulate_score(
        scores.engagement,
        weights.engagement,
        &mut weighted_sum,
        &mut weight_sum,
    );
    accumulate_score(
        scores.assessment_quality,
        weights.assessment_quality,
        &mut weighted_sum,
        &mut weight_sum,
    );

    if weight_sum > 0.0 {
        weighted_sum / weight_sum
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_score_valid() {
        assert!(QualityScore::new(0.0).is_ok());
        assert!(QualityScore::new(50.0).is_ok());
        assert!(QualityScore::new(100.0).is_ok());
    }

    #[test]
    fn test_quality_score_invalid() {
        assert!(QualityScore::new(-1.0).is_err());
        assert!(QualityScore::new(100.1).is_err());
        assert!(QualityScore::new(f64::NAN).is_err());
    }

    #[test]
    fn test_quality_score_passes() {
        let score = match QualityScore::new(90.0) {
            Ok(s) => s,
            Err(_) => return,
        };
        assert!(score.passes_default());
        assert!(score.passes(85.0));
        assert!(!score.passes(95.0));
    }

    #[test]
    fn test_weights_default_sum() {
        let weights = QualityWeights::default();
        let sum = weights.content_quality
            + weights.pedagogical_soundness
            + weights.accessibility
            + weights.technical_accuracy
            + weights.engagement
            + weights.assessment_quality;
        assert!((sum - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_overall_score_all_agents() {
        let scores = AgentScores {
            content_quality: QualityScore::new(80.0).ok(),
            pedagogical_soundness: QualityScore::new(90.0).ok(),
            accessibility: QualityScore::new(85.0).ok(),
            technical_accuracy: QualityScore::new(95.0).ok(),
            engagement: QualityScore::new(75.0).ok(),
            assessment_quality: QualityScore::new(88.0).ok(),
        };
        let weights = QualityWeights::default();
        let overall = calculate_overall_score(&scores, &weights);

        // Expected: 80*0.25 + 90*0.20 + 85*0.15 + 95*0.15 + 75*0.15 + 88*0.10
        //         = 20 + 18 + 12.75 + 14.25 + 11.25 + 8.8 = 85.05
        assert!((overall - 85.05).abs() < 0.01);
    }

    #[test]
    fn test_calculate_overall_score_partial() {
        let scores = AgentScores {
            content_quality: QualityScore::new(80.0).ok(),
            pedagogical_soundness: QualityScore::new(90.0).ok(),
            accessibility: None,
            technical_accuracy: None,
            engagement: None,
            assessment_quality: None,
        };
        let weights = QualityWeights::default();
        let overall = calculate_overall_score(&scores, &weights);

        // Expected: (80*0.25 + 90*0.20) / (0.25 + 0.20) = (20 + 18) / 0.45 = 84.44
        assert!((overall - 84.44).abs() < 0.01);
    }

    #[test]
    fn test_calculate_overall_score_none() {
        let scores = AgentScores::default();
        let weights = QualityWeights::default();
        let overall = calculate_overall_score(&scores, &weights);
        assert!((overall - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_deserialization_rejects_invalid() {
        let invalid_json = "101.0";
        let result: Result<QualityScore, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_severity_blocking() {
        assert!(ValidationSeverity::Critical.is_blocking());
        assert!(ValidationSeverity::High.is_blocking());
        assert!(!ValidationSeverity::Medium.is_blocking());
        assert!(!ValidationSeverity::Low.is_blocking());
    }

    // === Edge Case Tests ===

    #[test]
    fn test_quality_score_boundary_values() {
        // Exact boundaries
        assert!(QualityScore::new(0.0).is_ok());
        assert!(QualityScore::new(100.0).is_ok());
        // Just inside boundaries
        assert!(QualityScore::new(0.001).is_ok());
        assert!(QualityScore::new(99.999).is_ok());
        // Just outside boundaries
        assert!(QualityScore::new(-0.001).is_err());
        assert!(QualityScore::new(100.001).is_err());
    }

    #[test]
    fn test_quality_score_special_floats() {
        assert!(QualityScore::new(f64::NAN).is_err());
        assert!(QualityScore::new(f64::INFINITY).is_err());
        assert!(QualityScore::new(f64::NEG_INFINITY).is_err());
    }

    #[test]
    fn test_weights_invalid_sum() {
        // Weights that don't sum to 1.0 (sums to 0.9)
        let result = QualityWeights::new(0.2, 0.2, 0.1, 0.1, 0.1, 0.2);
        assert!(result.is_err());
    }

    #[test]
    fn test_weights_negative_rejected() {
        // Negative weight should be rejected even if sum is 1.0
        let result = QualityWeights::new(-0.5, 0.5, 0.25, 0.25, 0.25, 0.25);
        assert!(result.is_err());
    }

    #[test]
    fn test_agent_scores_count() {
        let empty = AgentScores::default();
        assert_eq!(empty.agent_count(), 0);
        assert!(!empty.is_complete());

        let partial = AgentScores {
            content_quality: QualityScore::new(80.0).ok(),
            pedagogical_soundness: QualityScore::new(90.0).ok(),
            accessibility: QualityScore::new(85.0).ok(),
            ..Default::default()
        };
        assert_eq!(partial.agent_count(), 3);
        assert!(!partial.is_complete());
    }

    #[test]
    fn test_calculate_score_single_agent() {
        let scores = AgentScores {
            content_quality: QualityScore::new(100.0).ok(),
            ..Default::default()
        };
        let weights = QualityWeights::default();
        let overall = calculate_overall_score(&scores, &weights);
        // With only content_quality, result should be 100.0
        assert!((overall - 100.0).abs() < 0.01);
    }
}
