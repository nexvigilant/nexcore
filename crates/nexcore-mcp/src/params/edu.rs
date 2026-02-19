//! Education Machine Parameters (Bayesian Mastery FSM)
//! Tier: T1-T3 (Curriculum and State Logic)
//!
//! Subjects, lessons, steps, learners, assessment, and spaced repetition.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for creating a subject.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EduSubjectCreateParams {
    /// Subject name.
    pub name: String,
    /// Description.
    pub description: String,
    /// Tags.
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Parameters for creating a lesson.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EduLessonCreateParams {
    /// Subject identifier.
    pub subject_id: String,
    /// Lesson title.
    pub title: String,
    /// Description.
    #[serde(default)]
    pub description: Option<String>,
    /// Difficulty (0.0-1.0).
    #[serde(default = "default_edu_difficulty")]
    pub difficulty: f64,
}

fn default_edu_difficulty() -> f64 {
    0.5
}

/// Parameters for adding a step.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EduLessonAddStepParams {
    /// Lesson identifier.
    pub lesson_id: String,
    /// Step type: "text", "exercise", "decomposition".
    pub step_type: String,
    /// Step title.
    pub title: String,
    /// Text body.
    #[serde(default)]
    pub body: Option<String>,
    /// Exercise prompt.
    #[serde(default)]
    pub prompt: Option<String>,
    /// Exercise solution.
    #[serde(default)]
    pub solution: Option<String>,
    /// Concept to decompose.
    #[serde(default)]
    pub concept: Option<String>,
}

/// Parameters for creating a learner.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EduLearnerCreateParams {
    /// Unique identifier.
    pub learner_id: String,
    /// Display name.
    pub name: String,
}

/// Parameters for enrollment.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EduEnrollParams {
    /// Learner identifier.
    pub learner_id: String,
    /// Subject identifier.
    pub subject_id: String,
}

/// A single assessment item.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EduAssessItem {
    /// Correctness.
    pub correct: bool,
    /// Difficulty.
    pub difficulty: f64,
}

/// Parameters for Bayesian assessment.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EduAssessParams {
    /// Subject identifier.
    pub subject_id: String,
    /// Assessment results.
    pub results: Vec<EduAssessItem>,
    /// Alpha prior.
    #[serde(default)]
    pub alpha: Option<f64>,
    /// Beta prior.
    #[serde(default)]
    pub beta: Option<f64>,
}

/// Parameters for mastery query.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EduMasteryParams {
    /// Probability value.
    pub mastery_value: f64,
}

/// Parameters for phase transitions.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EduPhaseTransitionParams {
    /// Source phase.
    pub from: String,
    /// Target phase.
    pub to: String,
    /// Reason for transition.
    #[serde(default)]
    pub reason: Option<String>,
}

/// Parameters for phase info query.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EduPhaseInfoParams {
    /// Current phase name.
    pub phase: String,
}

/// Parameters for creating a review item.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EduReviewCreateParams {
    /// Item identifier.
    pub item_id: String,
    /// Current time.
    #[serde(default)]
    pub current_time: Option<f64>,
}

/// Parameters for scheduling reviews.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EduReviewScheduleParams {
    /// Item identifier.
    pub item_id: String,
    /// Grade: "again", "hard", etc.
    pub grade: String,
    /// Current time.
    pub current_time: f64,
    /// Stability.
    #[serde(default)]
    pub stability: Option<f64>,
    /// Last review time.
    #[serde(default)]
    pub last_review: Option<f64>,
    /// Interval.
    #[serde(default)]
    pub interval_hours: Option<f64>,
    /// Review count.
    #[serde(default)]
    pub review_count: Option<u32>,
}

/// Parameters for review status query.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EduReviewStatusParams {
    /// Item identifier.
    pub item_id: String,
    /// Stability.
    pub stability: f64,
    /// Last review time.
    pub last_review: f64,
    /// Interval.
    pub interval_hours: f64,
    /// Current time.
    pub current_time: f64,
}

/// Parameters for Bayesian update.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EduBayesianUpdateParams {
    /// Current alpha.
    pub alpha: f64,
    /// Current beta.
    pub beta: f64,
    /// Correctness.
    pub correct: bool,
    /// Difficulty.
    pub difficulty: f64,
}

/// Parameters for concept mapping.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EduPrimitiveMapParams {
    /// Concept name.
    pub concept: String,
    /// Tier.
    pub tier: String,
    /// Primitives.
    pub primitives: Vec<String>,
    /// Dominant primitive.
    pub dominant: String,
}
