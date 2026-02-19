//! Education engine error types.
//!
//! # T1 Grounding
//! - ∂ (boundary): Each variant guards a domain constraint
//! - ∅ (void): Errors signal absence of valid state

use std::fmt;

/// All errors that can occur in the education engine.
///
/// Tier: T2-P (newtypes wrapping domain violations)
#[derive(Debug, Clone, PartialEq)]
pub enum EduError {
    /// Mastery level must be in [0, 1].
    InvalidMasteryLevel {
        /// The invalid value provided.
        value: f64,
    },
    /// Difficulty must be in [0, 1].
    InvalidDifficulty {
        /// The invalid value provided.
        value: f64,
    },
    /// Phase transition is not allowed by the FSM.
    InvalidPhaseTransition {
        /// Source phase name.
        from: String,
        /// Target phase name.
        to: String,
    },
    /// Subject not found.
    SubjectNotFound {
        /// Subject identifier.
        id: String,
    },
    /// Lesson not found.
    LessonNotFound {
        /// Lesson identifier.
        id: String,
    },
    /// Assessment cannot be empty.
    EmptyAssessment,
    /// Learner not enrolled in subject.
    NotEnrolled {
        /// Subject identifier.
        subject_id: String,
    },
}

impl fmt::Display for EduError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMasteryLevel { value } => {
                write!(f, "mastery level must be in [0, 1], got {value}")
            }
            Self::InvalidDifficulty { value } => {
                write!(f, "difficulty must be in [0, 1], got {value}")
            }
            Self::InvalidPhaseTransition { from, to } => {
                write!(f, "invalid transition: {from} -> {to}")
            }
            Self::SubjectNotFound { id } => {
                write!(f, "subject not found: {id}")
            }
            Self::LessonNotFound { id } => {
                write!(f, "lesson not found: {id}")
            }
            Self::EmptyAssessment => {
                write!(f, "assessment cannot be empty")
            }
            Self::NotEnrolled { subject_id } => {
                write!(f, "not enrolled in subject: {subject_id}")
            }
        }
    }
}

impl std::error::Error for EduError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_invalid_mastery() {
        let e = EduError::InvalidMasteryLevel { value: 1.5 };
        assert!(e.to_string().contains("1.5"));
    }

    #[test]
    fn display_invalid_difficulty() {
        let e = EduError::InvalidDifficulty { value: -0.1 };
        assert!(e.to_string().contains("-0.1"));
    }

    #[test]
    fn display_invalid_transition() {
        let e = EduError::InvalidPhaseTransition {
            from: "Discover".to_string(),
            to: "Master".to_string(),
        };
        assert!(e.to_string().contains("Discover"));
        assert!(e.to_string().contains("Master"));
    }

    #[test]
    fn display_subject_not_found() {
        let e = EduError::SubjectNotFound {
            id: "rust-101".to_string(),
        };
        assert!(e.to_string().contains("rust-101"));
    }

    #[test]
    fn display_lesson_not_found() {
        let e = EduError::LessonNotFound {
            id: "lesson-1".to_string(),
        };
        assert!(e.to_string().contains("lesson-1"));
    }

    #[test]
    fn display_empty_assessment() {
        let e = EduError::EmptyAssessment;
        assert!(e.to_string().contains("empty"));
    }

    #[test]
    fn display_not_enrolled() {
        let e = EduError::NotEnrolled {
            subject_id: "math".to_string(),
        };
        assert!(e.to_string().contains("math"));
    }

    #[test]
    fn error_trait_is_implemented() {
        let e: Box<dyn std::error::Error> = Box::new(EduError::InvalidMasteryLevel { value: -1.0 });
        assert!(!e.to_string().is_empty());
    }
}
