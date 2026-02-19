//! GMP Personnel Training and Competency Models.
//!
//! Training records, courses, and competency assessments for GMP compliance.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Types of training courses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TrainingCourseType {
    #[default]
    Gmp,
    Safety,
    Technical,
    Compliance,
    Quality,
    DataIntegrity,
    Cybersecurity,
}

/// Training record status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TrainingStatus {
    #[default]
    Assigned,
    InProgress,
    Passed,
    Failed,
    Expired,
}

/// Competency assessment results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CompetencyResult {
    #[default]
    Competent,
    NeedsTraining,
    NeedsRetraining,
}

/// Quiz question for training course.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizQuestion {
    pub question: String,
    /// 4 answer options.
    pub options: Vec<String>,
    /// Index of correct answer (0-3).
    pub correct_answer: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub explanation: Option<String>,
}

/// Training course definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingCourse {
    pub id: String,
    pub title: String,
    pub description: String,
    pub version: String,
    pub course_type: TrainingCourseType,
    pub duration_hours: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_url: Option<String>,
    #[serde(default)]
    pub quiz_questions: Vec<QuizQuestion>,
    #[serde(default = "default_passing_score")]
    pub passing_score: i32,
    #[serde(default = "default_validity_days")]
    pub valid_for_days: Option<i32>,
    #[serde(default)]
    pub required_for_roles: Vec<String>,
    pub created_by: String,
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
    pub tenant_id: String,
}

fn default_passing_score() -> i32 {
    80
}

fn default_validity_days() -> Option<i32> {
    Some(365)
}

/// Training record for a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingRecord {
    pub id: String,
    pub user_id: String,
    pub user_name: String,
    pub course_id: String,
    pub course_title: String,
    pub course_version: String,
    pub assigned_date: DateTime<Utc>,
    pub assigned_by: String,
    pub due_date: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_date: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_date: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expiration_date: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<i32>,
    #[serde(default)]
    pub attempts: i32,
    #[serde(default = "default_max_attempts")]
    pub max_attempts: i32,
    #[serde(default)]
    pub status: TrainingStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub certificate_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    pub tenant_id: String,
}

fn default_max_attempts() -> i32 {
    3
}

/// Competency assessment record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetencyAssessment {
    pub id: String,
    pub user_id: String,
    pub user_name: String,
    pub competency_type: String,
    #[serde(default = "Utc::now")]
    pub assessment_date: DateTime<Utc>,
    pub assessor_id: String,
    pub assessor_name: String,
    pub result: CompetencyResult,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<i32>,
    pub evidence: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observations: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_required: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_assessment_date: Option<DateTime<Utc>>,
    pub tenant_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_training_status_default() {
        assert_eq!(TrainingStatus::default(), TrainingStatus::Assigned);
    }

    #[test]
    fn test_competency_result_default() {
        assert_eq!(CompetencyResult::default(), CompetencyResult::Competent);
    }

    #[test]
    fn test_course_type_default() {
        assert_eq!(TrainingCourseType::default(), TrainingCourseType::Gmp);
    }
}
