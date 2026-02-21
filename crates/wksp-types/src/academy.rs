//! Academy types — courses, modules, KSBs, EPAs, certificates
//!
//! Terminology: Course = Capability Pathway, Lesson = Practice Activity

use serde::{Deserialize, Serialize};
use crate::common::Timestamp;

/// Proficiency level for skill assessment
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProficiencyLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

/// Course (Capability Pathway)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Course {
    pub id: String,
    pub title: String,
    pub description: String,
    pub slug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,
    pub difficulty: ProficiencyLevel,
    pub is_published: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_hours: Option<f32>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub skill_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<Timestamp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<Timestamp>,
}

/// Course module (skill grouping)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Module {
    pub id: String,
    pub course_id: String,
    pub title: String,
    pub description: String,
    pub order: u32,
    #[serde(default)]
    pub lesson_ids: Vec<String>,
}

/// Lesson (Practice Activity)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Lesson {
    pub id: String,
    pub module_id: String,
    pub title: String,
    pub content: String,
    pub order: u32,
    pub lesson_type: LessonType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_minutes: Option<u32>,
}

/// Type of lesson content
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LessonType {
    Text,
    Video,
    Quiz,
    Assignment,
    Project,
}

/// Student enrollment in a course
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Enrollment {
    pub id: String,
    pub user_id: String,
    pub course_id: String,
    pub progress_percent: f32,
    pub status: EnrollmentStatus,
    #[serde(default)]
    pub completed_lesson_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enrolled_at: Option<Timestamp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<Timestamp>,
}

/// Enrollment status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EnrollmentStatus {
    Active,
    Completed,
    Paused,
    Dropped,
}

/// Certificate (Capability Verification)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Certificate {
    pub id: String,
    pub user_id: String,
    pub course_id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issued_at: Option<Timestamp>,
}

/// Knowledge, Skill, or Behavior component
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ksb {
    pub id: String,
    pub code: String,
    pub title: String,
    pub description: String,
    pub ksb_type: KsbType,
    pub domain_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bloom_level: Option<u8>,
}

/// KSB type classification
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum KsbType {
    Knowledge,
    Skill,
    Behavior,
}

/// Entrustable Professional Activity
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Epa {
    pub id: String,
    pub number: u32,
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub ksb_ids: Vec<String>,
}

/// Learning pathway (curated sequence of courses)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pathway {
    pub id: String,
    pub title: String,
    pub description: String,
    pub slug: String,
    #[serde(default)]
    pub course_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_hours: Option<f32>,
    pub is_published: bool,
}

/// Academy assessment (quiz/exam)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcademyAssessment {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub course_id: Option<String>,
    pub question_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_minutes: Option<u32>,
    pub status: AssessmentStatus,
}

/// Assessment availability status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AssessmentStatus {
    Available,
    Locked,
    Completed,
}

// =============================================================================
// Learner State Machine (ς State primitive)
// =============================================================================

/// Learner journey state — models the lifecycle of a platform user.
///
/// Transitions: Onboarding → Exploring → Assessed → Learning → Certified
///              (each state may loop or skip forward based on conditions)
///
/// ## Primitive Grounding
/// - ς (State): Each variant is a distinct learner disposition
/// - σ (Sequence): States form an ordered progression
/// - → (Causality): Transitions are caused by learner actions
/// - ∂ (Boundary): State gates control access to features
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LearnerState {
    /// Just joined — completing onboarding flow
    Onboarding,
    /// Browsing domains, courses, community — no assessment yet
    Exploring,
    /// Has taken at least one competency self-assessment
    Assessed,
    /// Actively enrolled in one or more courses
    Learning,
    /// Has completed at least one certification pathway
    Certified,
}

impl LearnerState {
    /// Valid transitions from this state.
    #[must_use]
    pub fn valid_transitions(&self) -> &'static [LearnerState] {
        match self {
            Self::Onboarding => &[Self::Exploring],
            Self::Exploring => &[Self::Assessed, Self::Learning],
            Self::Assessed => &[Self::Learning, Self::Exploring],
            Self::Learning => &[Self::Certified, Self::Assessed, Self::Exploring],
            Self::Certified => &[Self::Learning, Self::Exploring],
        }
    }

    /// Whether the learner can transition to the target state.
    #[must_use]
    pub fn can_transition_to(&self, target: &Self) -> bool {
        self.valid_transitions().contains(target)
    }

    /// Display label for the UI.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Onboarding => "Onboarding",
            Self::Exploring => "Exploring",
            Self::Assessed => "Assessed",
            Self::Learning => "Learning",
            Self::Certified => "Certified",
        }
    }

    /// Progress percentage (0-100) for visual indicators.
    #[must_use]
    pub const fn progress_pct(&self) -> u8 {
        match self {
            Self::Onboarding => 0,
            Self::Exploring => 20,
            Self::Assessed => 40,
            Self::Learning => 70,
            Self::Certified => 100,
        }
    }

    /// Lex Primitiva symbol for this state.
    #[must_use]
    pub const fn primitive_symbol(&self) -> &'static str {
        match self {
            Self::Onboarding => "∃",  // Existence — establishing presence
            Self::Exploring => "λ",   // Location — finding your place
            Self::Assessed => "κ",    // Comparison — measuring yourself
            Self::Learning => "σ",    // Sequence — following pathways
            Self::Certified => "π",   // Persistence — lasting competence
        }
    }
}

/// Persisted learner journey state for a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LearnerJourney {
    pub user_id: String,
    pub state: LearnerState,
    pub domains_explored: Vec<String>,
    pub assessments_completed: u32,
    pub courses_enrolled: u32,
    pub courses_completed: u32,
    pub certificates_earned: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_activity: Option<Timestamp>,
}

impl LearnerJourney {
    /// Derive the appropriate state from activity counts.
    #[must_use]
    pub fn derived_state(&self) -> LearnerState {
        if self.certificates_earned > 0 {
            LearnerState::Certified
        } else if self.courses_enrolled > 0 {
            LearnerState::Learning
        } else if self.assessments_completed > 0 {
            LearnerState::Assessed
        } else if !self.domains_explored.is_empty() {
            LearnerState::Exploring
        } else {
            LearnerState::Onboarding
        }
    }
}

// =============================================================================
// Learning Pathway DAG (→ Causality + σ Sequence primitives)
// =============================================================================

/// A node in the learning pathway DAG.
///
/// Each course can have prerequisites (edges in the DAG).
/// The DAG is acyclic by construction — courses at lower tiers
/// cannot depend on higher-tier courses.
///
/// ## Primitive Grounding
/// - → (Causality): Prerequisites cause unlock of dependent courses
/// - σ (Sequence): Courses form ordered pathways
/// - ∂ (Boundary): Prerequisites gate access
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PathwayNode {
    pub course_id: String,
    pub code: String,
    pub title: String,
    pub tier: String,
    pub level: u8,
    /// Course IDs that must be completed before this course unlocks.
    #[serde(default)]
    pub prerequisites: Vec<String>,
    /// Whether the learner has completed this course.
    #[serde(default)]
    pub completed: bool,
    /// Whether all prerequisites are met (course is unlocked).
    #[serde(default)]
    pub unlocked: bool,
}

/// A complete learning pathway DAG.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningPathway {
    pub id: String,
    pub title: String,
    pub description: String,
    pub nodes: Vec<PathwayNode>,
}

impl LearningPathway {
    /// Count courses at each completion state.
    #[must_use]
    pub fn completion_stats(&self) -> (usize, usize, usize) {
        let completed = self.nodes.iter().filter(|n| n.completed).count();
        let unlocked = self.nodes.iter().filter(|n| n.unlocked && !n.completed).count();
        let locked = self.nodes.len() - completed - unlocked;
        (completed, unlocked, locked)
    }
}

/// Type aliases for capability-focused naming
pub type CapabilityPathway = Course;
pub type PracticeActivity = Lesson;
pub type CapabilityVerification = Certificate;
pub type SkillModule = Module;
