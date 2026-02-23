//! # nexcore CABA - Competency-Based Assessment System
//!
//! Rust implementation of the CABA (Competency-Based Autonomous Business Agent)
//! domain model, migrated from Python `domains/regulatory/caba`.
//!
//! ## UACA Hierarchy
//!
//! - **L0 Quarks**: Configuration constants (thresholds, weights)
//! - **L1 Atoms**: Individual validation functions (<20 LOC)
//! - **L2 Molecules**: Composite validators (<50 LOC)
//!
//! ## Modules
//!
//! - [`ksb`] - Knowledge-Skills-Behaviors taxonomy
//! - [`proficiency`] - 7-level proficiency model (L1 Novice → L5++ Executive)
//! - [`domain`] - 15 competency domains (PDC framework)
//! - [`competency`] - Core competencies (10 categories, requirements, validation)
//! - [`epa`] - Entrustable Professional Activities (14 compliance categories)
//! - [`cpa`] - Critical Practice Activities (8 CPAs, integration modules)
//! - [`validation`] - Prerequisite validation atoms and molecules
//!
//! ## CABA Hierarchy
//!
//! ```text
//! KSB (atomic) → Core Competency (integrated) → EPA (process) → CPA (business)
//! ```
//!
//! ## Safety Axioms
//!
//! - **Score Bounds**: All scores constrained to [0.0, 1.0] range
//! - **Level Ordering**: Proficiency levels have total ordering
//! - **Threshold Immutability**: Validation thresholds are compile-time constants

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod competency;
pub mod cpa;
pub mod domain;
pub mod epa;
pub mod ksb;
pub mod proficiency;
pub mod validation;

// Re-export key types at crate root
pub use domain::{DomainCategory, DomainRequirement};
pub use ksb::{CodedKsb, KsbRequirement, KsbType, ResearchSource};
pub use proficiency::{ProficiencyLevel, ProficiencyMetrics};
pub use validation::{PrerequisiteValidationResult, PrerequisiteValidator};

// Re-export competency types
pub use competency::{
    CompetencyCategory, CompetencyError, CompetencyRequirement, CompetencyRequirementBuilder,
    CoreCompetency, IntegrationModel, SimpleProficiencyLevel, ValidationResult,
};

// Re-export EPA types
pub use epa::{
    CompetencyDeploymentStatus, CompetencyDeploymentStep, EPACategory, EPAExecutionPlan,
    EPAExecutionState, EPAExecutionStatus, EPARequirement, EPAValidationResult, Priority,
};

// Re-export CPA types
pub use cpa::{
    CPACategory, CPAExecutionPlan, CPAExecutionState, CPAExecutionStatus, CPARequirement,
    CPAValidationResult, EPAPrerequisite, IntegrationModule,
};

/// Bounded score type ensuring value is in [0.0, 1.0].
///
/// # L1 Atom - Score validation (<20 LOC)
///
/// Safety Axiom: Score bounds are enforced at construction AND deserialization.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, serde::Serialize)]
pub struct Score(f64);

impl Score {
    /// Create a new bounded score.
    ///
    /// # Errors
    /// Returns error if value is not in [0.0, 1.0] range.
    pub fn new(value: f64) -> Result<Self, ScoreError> {
        if !(0.0..=1.0).contains(&value) {
            return Err(ScoreError::OutOfBounds { value });
        }
        Ok(Self(value))
    }

    /// Get the inner value.
    #[must_use]
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Zero score.
    pub const ZERO: Self = Self(0.0);

    /// Maximum score.
    pub const MAX: Self = Self(1.0);
}

impl Default for Score {
    fn default() -> Self {
        Self::ZERO
    }
}

// Custom deserializer that enforces bounds validation (Safety Axiom)
impl<'de> serde::Deserialize<'de> for Score {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = f64::deserialize(deserializer)?;
        Score::new(value).map_err(serde::de::Error::custom)
    }
}

/// Error type for score validation.
#[derive(Debug, Clone, nexcore_error::Error)]
pub enum ScoreError {
    /// Score value is outside [0.0, 1.0] bounds.
    #[error("Score {value} is out of bounds [0.0, 1.0]")]
    OutOfBounds {
        /// The invalid value
        value: f64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_valid() {
        assert!(Score::new(0.0).is_ok());
        assert!(Score::new(0.5).is_ok());
        assert!(Score::new(1.0).is_ok());
    }

    #[test]
    fn test_score_invalid() {
        assert!(Score::new(-0.1).is_err());
        assert!(Score::new(1.1).is_err());
        assert!(Score::new(f64::NAN).is_err());
    }

    #[test]
    fn test_score_ordering() {
        let low = match Score::new(0.3) {
            Ok(s) => s,
            Err(_) => return,
        };
        let high = match Score::new(0.7) {
            Ok(s) => s,
            Err(_) => return,
        };
        assert!(low < high);
    }

    #[test]
    fn test_score_deserialization_valid() {
        let json = "0.75";
        let result: Result<Score, _> = serde_json::from_str(json);
        assert!(result.is_ok());
        if let Ok(score) = result {
            assert!((score.value() - 0.75).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_score_deserialization_rejects_invalid() {
        // Safety Axiom: Deserialization MUST enforce bounds
        let invalid_json = "1.5";
        let result: Result<Score, _> = serde_json::from_str(invalid_json);
        assert!(
            result.is_err(),
            "Score deserialization must reject values > 1.0"
        );

        let negative_json = "-0.1";
        let result: Result<Score, _> = serde_json::from_str(negative_json);
        assert!(
            result.is_err(),
            "Score deserialization must reject values < 0.0"
        );
    }
}
