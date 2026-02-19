//! User Competency Profile Types
//!
//! Comprehensive tracking of user's competency progression.

use serde::{Deserialize, Serialize};

use super::cpa::{Cpa, CpaProgress, CpaStatus};
use super::domain::{AchievedBehavioralAnchor, Domain, DomainProgress};
use super::epa::{EntrustmentLevel, Epa, EpaProgress};

/// User's complete competency profile
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserCompetencyProfile {
    /// User identifier
    pub user_id: String,
    /// Creation timestamp (ISO 8601)
    pub created_at: String,
    /// Last update timestamp (ISO 8601)
    pub updated_at: String,
    /// Domain competency tracking
    pub domain_progress: Vec<DomainProgress>,
    /// EPA entrustment tracking
    pub epa_progress: Vec<EpaProgress>,
    /// CPA achievement tracking
    pub cpa_progress: Vec<CpaProgress>,
    /// Behavioral anchor achievements
    pub achieved_anchors: Vec<AchievedBehavioralAnchor>,
    /// Digital portfolio reference
    pub portfolio_id: String,
    /// Overall metrics
    pub metrics: CompetencyMetrics,
}

/// Overall competency metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompetencyMetrics {
    /// Overall competency score (0-100)
    pub overall_competency_score: u8,
    /// L1-L2 achievement percentage
    pub foundation_score: u8,
    /// L3 achievement percentage
    pub professional_score: u8,
    /// L4-L5 achievement percentage
    pub advanced_score: u8,
    /// L5+ achievement percentage
    pub executive_score: u8,
    /// EPA readiness counts
    pub epa_readiness: EpaReadiness,
    /// CPA achievement statuses
    pub cpa_achievements: Vec<CpaAchievement>,
    /// AI Gateway status
    pub ai_gateway_status: AiGatewayStatus,
    /// Last assessment date (ISO 8601)
    pub last_assessment_date: String,
    /// Anchors achieved in last 30 days
    pub anchors_achieved_last_30_days: u32,
    /// Anchors achieved in last 90 days
    pub anchors_achieved_last_90_days: u32,
    /// Recent EPA advancements
    pub recent_epa_advancements: Vec<EpaAdvancement>,
}

/// EPA readiness counts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct EpaReadiness {
    /// How many of 10 core EPAs are ready
    pub core_epas_ready: u8,
    /// How many of 10 executive EPAs are ready
    pub executive_epas_ready: u8,
}

/// CPA achievement record
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CpaAchievement {
    /// The CPA
    pub cpa: Cpa,
    /// Current status
    pub status: CpaStatus,
}

/// AI Gateway (EPA10) status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiGatewayStatus {
    /// EPA10 entrustment level
    pub epa10_level: EntrustmentLevel,
    /// Whether CPA8 is eligible
    pub cpa8_eligible: bool,
    /// CPA8 status if eligible
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpa8_status: Option<CpaStatus>,
    /// Completed training module IDs
    pub completed_training_modules: Vec<String>,
    /// AI tool usage count
    pub ai_tool_usage_count: u32,
    /// Average AI validation accuracy (0.0-1.0)
    pub average_ai_validation_accuracy: f64,
    /// Progress toward Level 4 (0-100)
    pub progress_to_level_4: u8,
}

/// EPA advancement record
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpaAdvancement {
    /// The EPA that advanced
    pub epa: Epa,
    /// Previous entrustment level
    pub previous_level: EntrustmentLevel,
    /// New entrustment level
    pub new_level: EntrustmentLevel,
    /// Date of advancement (ISO 8601)
    pub advanced_date: String,
}

/// Competency assessment record
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompetencyAssessment {
    /// Assessment ID
    pub id: String,
    /// User being assessed
    pub user_id: String,
    /// Assessor's user ID
    pub assessor_id: String,
    /// Type of assessment
    pub assessment_type: AssessmentType,
    /// Assessment date (ISO 8601)
    pub assessment_date: String,
    /// Domain assessed (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<Domain>,
    /// EPA assessed (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub epa: Option<Epa>,
    /// CPA assessed (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpa: Option<Cpa>,
    /// Assessment results
    pub results: AssessmentResults,
    /// Evidence artifacts reviewed
    pub evidence_artifacts: Vec<String>,
}

/// Assessment type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssessmentType {
    /// Domain-specific assessment
    Domain,
    /// EPA-specific assessment
    Epa,
    /// CPA-specific assessment
    Cpa,
    /// Comprehensive assessment
    Comprehensive,
}

/// Assessment results
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssessmentResults {
    /// Whether competency was achieved
    pub achieved: bool,
    /// Level achieved (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
    /// Assessor feedback
    pub feedback: String,
    /// Identified strengths
    pub strengths: Vec<String>,
    /// Development areas
    pub development_areas: Vec<String>,
    /// Recommended actions
    pub recommended_actions: Vec<String>,
}

/// Learning milestone
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LearningMilestone {
    /// Milestone ID
    pub id: String,
    /// User who achieved
    pub user_id: String,
    /// Type of milestone
    pub milestone_type: MilestoneType,
    /// Achievement date (ISO 8601)
    pub achieved_date: String,
    /// Domain (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<Domain>,
    /// EPA (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub epa: Option<Epa>,
    /// CPA (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpa: Option<Cpa>,
    /// Level achieved
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
    /// Milestone title
    pub title: String,
    /// Description
    pub description: String,
    /// Celebration message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub celebration_message: Option<String>,
    /// Metadata
    pub metadata: MilestoneMetadata,
}

/// Milestone type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MilestoneType {
    /// Domain level achievement
    DomainLevel,
    /// EPA entrustment achievement
    EpaEntrustment,
    /// CPA entry
    CpaEntry,
    /// CPA8 gateway achievement
    Cpa8Gateway,
}

/// Milestone metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MilestoneMetadata {
    /// Assessor ID (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assessor_id: Option<String>,
    /// Evidence count
    pub evidence_count: u32,
    /// Days from previous milestone
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_to_achieve: Option<u32>,
}

/// Development plan
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DevelopmentPlan {
    /// Plan ID
    pub id: String,
    /// User ID
    pub user_id: String,
    /// Creator (supervisor/mentor) ID
    pub created_by: String,
    /// Creation date (ISO 8601)
    pub created_date: String,
    /// Target completion date (ISO 8601)
    pub target_completion_date: String,
    /// Plan status
    pub status: PlanStatus,
    /// Development goals
    pub goals: Vec<DevelopmentGoal>,
    /// Last review date (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_review_date: Option<String>,
    /// Next review date (ISO 8601)
    pub next_review_date: String,
    /// Completion percentage (0-100)
    pub completion_percentage: u8,
}

/// Plan status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanStatus {
    /// Active plan
    #[default]
    Active,
    /// Completed
    Completed,
    /// Revised
    Revised,
}

/// Development goal
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DevelopmentGoal {
    /// Goal ID
    pub id: String,
    /// Goal type
    pub goal_type: GoalType,
    /// Target domain (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_domain: Option<Domain>,
    /// Target EPA (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_epa: Option<Epa>,
    /// Target CPA (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_cpa: Option<Cpa>,
    /// Target level
    pub target_level: String,
    /// Action items
    pub actions: Vec<ActionItem>,
    /// Goal status
    pub status: GoalStatus,
    /// Achievement date (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub achieved_date: Option<String>,
}

/// Goal type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoalType {
    /// Domain level advancement
    DomainAdvancement,
    /// EPA progression
    EpaProgression,
    /// CPA achievement
    CpaAchievement,
}

/// Action item in development goal
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionItem {
    /// Action description
    pub description: String,
    /// Due date (ISO 8601)
    pub due_date: String,
    /// Whether completed
    pub completed: bool,
    /// Completion date (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_date: Option<String>,
}

/// Goal status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoalStatus {
    /// Not started
    #[default]
    NotStarted,
    /// In progress
    InProgress,
    /// Achieved
    Achieved,
    /// Deferred
    Deferred,
}

// ============================================================================
// Pure Functions (L1 Atoms)
// ============================================================================

/// Score weights for overall competency calculation
const WEIGHTS: (f64, f64, f64, f64) = (0.2, 0.3, 0.3, 0.2);

/// Calculate overall competency score from component scores
///
/// Uses weighted average: 20% foundation, 30% professional, 30% advanced, 20% executive
#[must_use]
pub fn calculate_overall_score(
    foundation_score: u8,
    professional_score: u8,
    advanced_score: u8,
    executive_score: u8,
) -> u8 {
    let (w_f, w_p, w_a, w_e) = WEIGHTS;
    let score = f64::from(foundation_score) * w_f
        + f64::from(professional_score) * w_p
        + f64::from(advanced_score) * w_a
        + f64::from(executive_score) * w_e;

    // INVARIANT: score is always 0-100
    score.round().min(100.0) as u8
}

/// Get a summary label for competency score
#[must_use]
pub const fn get_profile_summary(score: u8) -> &'static str {
    match score {
        0..=39 => "Novice",
        40..=59 => "Advanced Beginner",
        60..=74 => "Competent Professional",
        75..=84 => "Proficient Expert",
        85..=100 => "Executive Leader",
        // SAFETY: u8 max is 255, but we handle it gracefully
        _ => "Executive Leader",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overall_score_calculation() {
        // Weights: 0.2 + 0.3 + 0.3 + 0.2 = 1.0
        // All 100s should give 100
        assert_eq!(calculate_overall_score(100, 100, 100, 100), 100);

        // All 0s should give 0
        assert_eq!(calculate_overall_score(0, 0, 0, 0), 0);

        // Mixed scores
        // 50 * 0.2 + 60 * 0.3 + 70 * 0.3 + 80 * 0.2 = 10 + 18 + 21 + 16 = 65
        assert_eq!(calculate_overall_score(50, 60, 70, 80), 65);
    }

    #[test]
    fn test_profile_summary() {
        assert_eq!(get_profile_summary(20), "Novice");
        assert_eq!(get_profile_summary(50), "Advanced Beginner");
        assert_eq!(get_profile_summary(65), "Competent Professional");
        assert_eq!(get_profile_summary(80), "Proficient Expert");
        assert_eq!(get_profile_summary(95), "Executive Leader");
    }

    #[test]
    fn test_default_statuses() {
        assert_eq!(CpaStatus::default(), CpaStatus::NotStarted);
        assert_eq!(PlanStatus::default(), PlanStatus::Active);
        assert_eq!(GoalStatus::default(), GoalStatus::NotStarted);
    }
}
