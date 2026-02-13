//! Convenience re-exports for common education engine types.

pub use crate::assessment::{
    Assessment, AssessmentResult, BayesianPrior, Question, QuestionResult, evaluate_assessment,
};
pub use crate::error::EduError;
pub use crate::learner::{AssessmentRecord, Enrollment, Learner};
pub use crate::lesson::{Lesson, LessonContent, LessonStep, PrimitiveMapping};
pub use crate::spaced_repetition::ReviewState;
pub use crate::state_machine::{
    PhaseTransition, can_transition, completion_percentage, execute_transition, phases_remaining,
    suggest_next_phase,
};
pub use crate::subject::{LessonRef, Subject};
pub use crate::types::{
    CompetencyLevel, DEVELOPING_THRESHOLD, Difficulty, Grade, LearningPhase, MASTERY_THRESHOLD,
    MasteryLevel, MasteryVerdict,
};
