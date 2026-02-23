//! Knowledge-Skills-Behaviors (KSB) Component Types
//!
//! Migrated from Python `course-builder/course-builder-service/app/models/course.py`.
//!
//! ## UACA Hierarchy
//!
//! - **L0 Quarks**: Component type enumeration, difficulty levels
//! - **L1 Atoms**: Validation functions (<20 LOC)
//!
//! ## KSB Framework
//!
//! Components are categorized as:
//! - **Knowledge (K)**: Facts, concepts, principles, mental models
//! - **Skills (S)**: Procedures, techniques, tool usage
//! - **Behaviors (B)**: Patterns, heuristics, decision approaches
//!
//! ## Submodules
//!
//! - [`research`] - KSB research pipeline types (quality, metrics, grading)
//!
//! ## Safety Axiom
//!
//! Component types are exhaustive enums - no invalid states possible.

pub mod research;

use serde::{Deserialize, Serialize};

/// Component type classification.
///
/// # L0 Quark - Type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComponentType {
    /// Knowledge: Facts, concepts, principles
    Knowledge,
    /// Skill: Procedures, techniques, tool usage
    Skill,
    /// Behavior: Patterns, heuristics, decision approaches
    Behavior,
}

impl ComponentType {
    /// Get single-letter prefix (K, S, or B).
    #[must_use]
    pub const fn prefix(&self) -> char {
        match self {
            Self::Knowledge => 'K',
            Self::Skill => 'S',
            Self::Behavior => 'B',
        }
    }

    /// Get display string.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Knowledge => "knowledge",
            Self::Skill => "skill",
            Self::Behavior => "behavior",
        }
    }

    /// Get estimated base duration in minutes for this type.
    ///
    /// Knowledge components typically take longer to absorb than skills.
    #[must_use]
    pub const fn base_duration_minutes(&self) -> u32 {
        match self {
            Self::Knowledge => 15,
            Self::Skill => 20,
            Self::Behavior => 25,
        }
    }
}

impl std::fmt::Display for ComponentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Difficulty level for learning content.
///
/// # L0 Quark - Difficulty enumeration with numeric mapping
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
#[serde(rename_all = "lowercase")]
pub enum DifficultyLevel {
    /// Entry-level content, no prerequisites
    Beginner = 1,
    /// Requires foundational knowledge
    #[default]
    Intermediate = 2,
    /// Expert-level content
    Advanced = 3,
}

impl DifficultyLevel {
    /// Get numeric value (1, 2, or 3).
    #[must_use]
    pub const fn numeric_value(&self) -> u8 {
        match self {
            Self::Beginner => 1,
            Self::Intermediate => 2,
            Self::Advanced => 3,
        }
    }

    /// Get display string.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Beginner => "beginner",
            Self::Intermediate => "intermediate",
            Self::Advanced => "advanced",
        }
    }

    /// Get duration multiplier for this difficulty.
    ///
    /// Advanced content takes longer to absorb.
    #[must_use]
    pub const fn duration_multiplier(&self) -> f64 {
        match self {
            Self::Beginner => 1.0,
            Self::Intermediate => 1.5,
            Self::Advanced => 2.0,
        }
    }
}

impl std::fmt::Display for DifficultyLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// KSB component definition.
///
/// # L1 Atom - Component structure
///
/// Represents a single Knowledge, Skill, or Behavior component
/// in a learning framework.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KsbComponent {
    /// Unique identifier (format: K-XXX-001, S-XXX-001, B-XXX-001)
    pub id: String,
    /// Component type
    pub component_type: ComponentType,
    /// Component title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Learning objectives (recommended: 3-5)
    #[serde(default)]
    pub learning_objectives: Vec<String>,
    /// Key teaching points
    #[serde(default)]
    pub key_points: Vec<String>,
    /// Real-world examples
    #[serde(default)]
    pub examples: Vec<String>,
    /// Assessment criteria for mastery
    #[serde(default)]
    pub assessment_criteria: Vec<String>,
    /// Prerequisite component IDs
    #[serde(default)]
    pub prerequisites: Vec<String>,
}

/// L0 Quark - Cardinality constraints
pub mod constraints {
    /// Minimum learning objectives per component
    pub const MIN_OBJECTIVES: usize = 3;
    /// Maximum learning objectives per component
    pub const MAX_OBJECTIVES: usize = 5;
    /// Minimum key points per component
    pub const MIN_KEY_POINTS: usize = 2;
    /// Maximum key points per component
    pub const MAX_KEY_POINTS: usize = 10;
}

impl KsbComponent {
    /// Validate component structure.
    ///
    /// # L2 Molecule - Composite validation (<50 LOC)
    ///
    /// Validates title, description, objectives, and key points cardinality.
    #[must_use]
    pub fn validate(&self) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        if self.title.is_empty() {
            issues.push(ValidationIssue::EmptyTitle);
        }
        if self.description.is_empty() {
            issues.push(ValidationIssue::EmptyDescription);
        }

        // Validate learning objectives cardinality
        let obj_count = self.learning_objectives.len();
        if obj_count < constraints::MIN_OBJECTIVES {
            issues.push(ValidationIssue::TooFewObjectives {
                count: obj_count,
                min: constraints::MIN_OBJECTIVES,
            });
        } else if obj_count > constraints::MAX_OBJECTIVES {
            issues.push(ValidationIssue::TooManyObjectives {
                count: obj_count,
                max: constraints::MAX_OBJECTIVES,
            });
        }

        // Validate key points cardinality
        let kp_count = self.key_points.len();
        if kp_count < constraints::MIN_KEY_POINTS {
            issues.push(ValidationIssue::TooFewKeyPoints {
                count: kp_count,
                min: constraints::MIN_KEY_POINTS,
            });
        } else if kp_count > constraints::MAX_KEY_POINTS {
            issues.push(ValidationIssue::TooManyKeyPoints {
                count: kp_count,
                max: constraints::MAX_KEY_POINTS,
            });
        }

        issues
    }

    /// Check if component is valid.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.validate().is_empty()
    }

    /// Get single-letter type prefix.
    #[must_use]
    pub fn type_prefix(&self) -> char {
        self.component_type.prefix()
    }
}

/// Validation issues for KSB components.
#[derive(Debug, Clone, PartialEq, Eq, nexcore_error::Error)]
pub enum ValidationIssue {
    /// Title is empty
    #[error("Component title is empty")]
    EmptyTitle,
    /// Description is empty
    #[error("Component description is empty")]
    EmptyDescription,
    /// Too few learning objectives
    #[error("Too few objectives: {count} (minimum: {min})")]
    TooFewObjectives {
        /// Actual count
        count: usize,
        /// Minimum required
        min: usize,
    },
    /// Too many learning objectives
    #[error("Too many objectives: {count} (maximum: {max})")]
    TooManyObjectives {
        /// Actual count
        count: usize,
        /// Maximum allowed
        max: usize,
    },
    /// Too few key points
    #[error("Too few key points: {count} (minimum: {min})")]
    TooFewKeyPoints {
        /// Actual count
        count: usize,
        /// Minimum required
        min: usize,
    },
    /// Too many key points
    #[error("Too many key points: {count} (maximum: {max})")]
    TooManyKeyPoints {
        /// Actual count
        count: usize,
        /// Maximum allowed
        max: usize,
    },
}

/// Estimate learning duration for a component.
///
/// # L1 Atom - Duration estimation (<20 LOC)
///
/// Combines base duration from component type with difficulty multiplier.
///
/// # Safety Axiom
/// Maximum value is 50 (25 * 2.0), always positive and fits in u32.
#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn estimate_duration_minutes(
    component_type: ComponentType,
    difficulty: DifficultyLevel,
) -> u32 {
    let base = f64::from(component_type.base_duration_minutes());
    let multiplier = difficulty.duration_multiplier();
    (base * multiplier).round() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_type_prefix() {
        assert_eq!(ComponentType::Knowledge.prefix(), 'K');
        assert_eq!(ComponentType::Skill.prefix(), 'S');
        assert_eq!(ComponentType::Behavior.prefix(), 'B');
    }

    #[test]
    fn test_difficulty_ordering() {
        assert!(DifficultyLevel::Beginner < DifficultyLevel::Intermediate);
        assert!(DifficultyLevel::Intermediate < DifficultyLevel::Advanced);
    }

    #[test]
    fn test_duration_estimation() {
        // Knowledge + Beginner: 15 * 1.0 = 15
        assert_eq!(
            estimate_duration_minutes(ComponentType::Knowledge, DifficultyLevel::Beginner),
            15
        );
        // Skill + Intermediate: 20 * 1.5 = 30
        assert_eq!(
            estimate_duration_minutes(ComponentType::Skill, DifficultyLevel::Intermediate),
            30
        );
        // Behavior + Advanced: 25 * 2.0 = 50
        assert_eq!(
            estimate_duration_minutes(ComponentType::Behavior, DifficultyLevel::Advanced),
            50
        );
    }

    #[test]
    fn test_component_validation_valid() {
        let component = KsbComponent {
            id: "K-PV-001".to_string(),
            component_type: ComponentType::Knowledge,
            title: "Signal Detection Fundamentals".to_string(),
            description: "Understanding the basics of pharmacovigilance signal detection"
                .to_string(),
            learning_objectives: vec![
                "Objective 1".to_string(),
                "Objective 2".to_string(),
                "Objective 3".to_string(),
            ],
            key_points: vec!["Point 1".to_string(), "Point 2".to_string()],
            examples: vec![],
            assessment_criteria: vec![],
            prerequisites: vec![],
        };
        assert!(component.is_valid());
    }

    #[test]
    fn test_component_validation_empty_title() {
        let component = KsbComponent {
            id: "K-PV-001".to_string(),
            component_type: ComponentType::Knowledge,
            title: String::new(),
            description: "Description".to_string(),
            learning_objectives: vec!["1".to_string(), "2".to_string(), "3".to_string()],
            key_points: vec![],
            examples: vec![],
            assessment_criteria: vec![],
            prerequisites: vec![],
        };
        let issues = component.validate();
        assert!(issues.contains(&ValidationIssue::EmptyTitle));
    }

    #[test]
    fn test_component_validation_too_few_objectives() {
        let component = KsbComponent {
            id: "K-PV-001".to_string(),
            component_type: ComponentType::Knowledge,
            title: "Title".to_string(),
            description: "Description".to_string(),
            learning_objectives: vec!["1".to_string()],
            key_points: vec![],
            examples: vec![],
            assessment_criteria: vec![],
            prerequisites: vec![],
        };
        let issues = component.validate();
        assert!(matches!(
            issues.first(),
            Some(ValidationIssue::TooFewObjectives { .. })
        ));
    }
}
