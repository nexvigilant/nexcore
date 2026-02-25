//! Learning domain types: progress tracking and onboarding quizzes.

use nexcore_chrono::DateTime;
use nexcore_id::NexId;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::enums::{CareerVertical, CurriculumTier, ModuleStatus, UserPersona};

/// Track user progress through curriculum modules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningProgress {
    /// Unique identifier.
    pub id: NexId,

    /// Associated user profile ID.
    pub user_profile_id: NexId,

    /// Curriculum tier.
    pub tier: CurriculumTier,

    /// Module identifier (e.g., "foundation_industry_101").
    pub module_id: String,

    /// Module display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module_name: Option<String>,

    /// Career vertical this module belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub career_vertical: Option<String>,

    /// Completion status.
    #[serde(default)]
    pub status: ModuleStatus,

    /// Progress percentage (0-100).
    #[serde(default)]
    pub progress_percentage: i32,

    /// When the user started.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<DateTime>,

    /// When the user completed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime>,

    /// Time spent in minutes.
    #[serde(default)]
    pub time_spent_minutes: i32,

    /// Quiz score percentage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quiz_score: Option<Decimal>,

    /// Number of quiz attempts.
    #[serde(default)]
    pub quiz_attempts: i32,

    /// Whether assessment was passed.
    #[serde(default)]
    pub passed_assessment: bool,

    /// Last access time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_accessed_at: Option<DateTime>,

    /// Number of accesses.
    #[serde(default)]
    pub access_count: i32,

    /// Lessons completed.
    #[serde(default)]
    pub lessons_completed: i32,

    /// Total lessons in module.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lessons_total: Option<i32>,

    /// Exercises completed.
    #[serde(default)]
    pub exercises_completed: i32,

    /// Total exercises in module.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exercises_total: Option<i32>,

    /// Resources downloaded.
    #[serde(default)]
    pub resources_downloaded: i32,

    /// User's personal notes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,

    /// Module content version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module_version: Option<String>,

    /// Custom module-specific data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_data: Option<Value>,

    /// Creation timestamp.
    pub created_at: DateTime,

    /// Last update timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime>,
}

impl LearningProgress {
    /// Create a new learning progress record.
    #[must_use]
    pub fn new(user_profile_id: NexId, tier: CurriculumTier, module_id: impl Into<String>) -> Self {
        Self {
            id: NexId::v4(),
            user_profile_id,
            tier,
            module_id: module_id.into(),
            module_name: None,
            career_vertical: None,
            status: ModuleStatus::NotStarted,
            progress_percentage: 0,
            started_at: None,
            completed_at: None,
            time_spent_minutes: 0,
            quiz_score: None,
            quiz_attempts: 0,
            passed_assessment: false,
            last_accessed_at: None,
            access_count: 0,
            lessons_completed: 0,
            lessons_total: None,
            exercises_completed: 0,
            exercises_total: None,
            resources_downloaded: 0,
            notes: None,
            module_version: None,
            custom_data: None,
            created_at: DateTime::now(),
            updated_at: None,
        }
    }

    /// Calculate lessons progress percentage.
    #[must_use]
    pub fn lesson_progress(&self) -> Option<f64> {
        self.lessons_total
            .filter(|&total| total > 0)
            .map(|total| f64::from(self.lessons_completed) / f64::from(total) * 100.0)
    }

    /// Calculate exercises progress percentage.
    #[must_use]
    pub fn exercise_progress(&self) -> Option<f64> {
        self.exercises_total
            .filter(|&total| total > 0)
            .map(|total| f64::from(self.exercises_completed) / f64::from(total) * 100.0)
    }

    /// Start the module.
    pub fn start(&mut self) {
        if self.status == ModuleStatus::NotStarted {
            self.status = ModuleStatus::InProgress;
            self.started_at = Some(DateTime::now());
            self.updated_at = Some(DateTime::now());
        }
    }

    /// Complete the module.
    pub fn complete(&mut self) {
        self.status = ModuleStatus::Completed;
        self.progress_percentage = 100;
        self.completed_at = Some(DateTime::now());
        self.updated_at = Some(DateTime::now());
    }

    /// Record a module access.
    pub fn record_access(&mut self) {
        self.access_count += 1;
        self.last_accessed_at = Some(DateTime::now());
        self.updated_at = Some(DateTime::now());
    }

    /// Add time spent.
    pub fn add_time(&mut self, minutes: i32) {
        self.time_spent_minutes += minutes;
        self.updated_at = Some(DateTime::now());
    }
}

/// Onboarding quiz responses and results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingQuiz {
    /// Unique identifier.
    pub id: NexId,

    /// Associated user profile ID.
    pub user_profile_id: NexId,

    /// Quiz version for A/B testing.
    #[serde(default = "default_quiz_version")]
    pub quiz_version: String,

    /// Full quiz responses as JSON.
    pub responses: Value,

    /// Calculated persona from responses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calculated_persona: Option<UserPersona>,

    /// Primary career vertical recommendation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommended_vertical: Option<CareerVertical>,

    /// Alternative career path recommendations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alternative_verticals: Option<Vec<CareerVertical>>,

    /// Medical Affairs affinity score.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub medical_affairs_score: Option<i32>,

    /// Regulatory Affairs affinity score.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regulatory_affairs_score: Option<i32>,

    /// HEOR affinity score.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heor_score: Option<i32>,

    /// Clinical Development affinity score.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clinical_dev_score: Option<i32>,

    /// Engagement likelihood prediction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engagement_likelihood: Option<String>,

    /// Recommended curriculum tier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommended_tier: Option<CurriculumTier>,

    /// Time to complete quiz in seconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_time_seconds: Option<i32>,

    /// Source of quiz (landing_page, registration, etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    /// Creation timestamp.
    pub created_at: DateTime,

    /// Last update timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime>,
}

fn default_quiz_version() -> String {
    "1.0".to_string()
}

impl OnboardingQuiz {
    /// Create a new onboarding quiz record.
    #[must_use]
    pub fn new(user_profile_id: NexId, responses: Value) -> Self {
        Self {
            id: NexId::v4(),
            user_profile_id,
            quiz_version: default_quiz_version(),
            responses,
            calculated_persona: None,
            recommended_vertical: None,
            alternative_verticals: None,
            medical_affairs_score: None,
            regulatory_affairs_score: None,
            heor_score: None,
            clinical_dev_score: None,
            engagement_likelihood: None,
            recommended_tier: None,
            completion_time_seconds: None,
            source: None,
            created_at: DateTime::now(),
            updated_at: None,
        }
    }

    /// Get all vertical scores as a vector of tuples.
    #[must_use]
    pub fn vertical_scores(&self) -> Vec<(CareerVertical, i32)> {
        let mut scores = Vec::new();

        if let Some(score) = self.medical_affairs_score {
            scores.push((CareerVertical::MedicalAffairs, score));
        }
        if let Some(score) = self.regulatory_affairs_score {
            scores.push((CareerVertical::RegulatoryAffairs, score));
        }
        if let Some(score) = self.heor_score {
            scores.push((CareerVertical::Heor, score));
        }
        if let Some(score) = self.clinical_dev_score {
            scores.push((CareerVertical::ClinicalDevelopment, score));
        }

        // Sort by score descending
        scores.sort_by(|a, b| b.1.cmp(&a.1));
        scores
    }

    /// Calculate recommended vertical from scores.
    pub fn calculate_recommendation(&mut self) {
        let scores = self.vertical_scores();
        if let Some((vertical, _)) = scores.first() {
            self.recommended_vertical = Some(*vertical);
        }

        // Set alternatives (2nd and 3rd highest)
        if scores.len() > 1 {
            self.alternative_verticals =
                Some(scores.iter().skip(1).take(2).map(|(v, _)| *v).collect());
        }

        self.updated_at = Some(DateTime::now());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_learning_progress_new() {
        let profile_id = NexId::v4();
        let progress = LearningProgress::new(
            profile_id,
            CurriculumTier::Foundation,
            "foundation_intro_101",
        );

        assert_eq!(progress.user_profile_id, profile_id);
        assert_eq!(progress.tier, CurriculumTier::Foundation);
        assert_eq!(progress.status, ModuleStatus::NotStarted);
        assert_eq!(progress.progress_percentage, 0);
    }

    #[test]
    fn test_learning_progress_lifecycle() {
        let profile_id = NexId::v4();
        let mut progress = LearningProgress::new(
            profile_id,
            CurriculumTier::Specialization,
            "spec_regulatory_201",
        );

        progress.start();
        assert_eq!(progress.status, ModuleStatus::InProgress);
        assert!(progress.started_at.is_some());

        progress.record_access();
        assert_eq!(progress.access_count, 1);

        progress.add_time(30);
        assert_eq!(progress.time_spent_minutes, 30);

        progress.complete();
        assert_eq!(progress.status, ModuleStatus::Completed);
        assert_eq!(progress.progress_percentage, 100);
        assert!(progress.completed_at.is_some());
    }

    #[test]
    fn test_lesson_progress() {
        let profile_id = NexId::v4();
        let mut progress = LearningProgress::new(
            profile_id,
            CurriculumTier::Foundation,
            "foundation_intro_101",
        );

        progress.lessons_total = Some(10);
        progress.lessons_completed = 3;

        let pct = progress.lesson_progress();
        assert!(pct.is_some());
        assert!((pct.unwrap() - 30.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_onboarding_quiz_vertical_scores() {
        let profile_id = NexId::v4();
        let mut quiz = OnboardingQuiz::new(profile_id, serde_json::json!({}));

        quiz.medical_affairs_score = Some(85);
        quiz.regulatory_affairs_score = Some(70);
        quiz.heor_score = Some(90);
        quiz.clinical_dev_score = Some(60);

        let scores = quiz.vertical_scores();
        assert_eq!(scores[0].0, CareerVertical::Heor);
        assert_eq!(scores[0].1, 90);
        assert_eq!(scores[1].0, CareerVertical::MedicalAffairs);
    }

    #[test]
    fn test_quiz_recommendation() {
        let profile_id = NexId::v4();
        let mut quiz = OnboardingQuiz::new(profile_id, serde_json::json!({}));

        quiz.medical_affairs_score = Some(85);
        quiz.regulatory_affairs_score = Some(90);
        quiz.heor_score = Some(70);

        quiz.calculate_recommendation();

        assert_eq!(
            quiz.recommended_vertical,
            Some(CareerVertical::RegulatoryAffairs)
        );
        let alternatives = quiz.alternative_verticals.as_ref().unwrap();
        assert!(alternatives.contains(&CareerVertical::MedicalAffairs));
    }
}
