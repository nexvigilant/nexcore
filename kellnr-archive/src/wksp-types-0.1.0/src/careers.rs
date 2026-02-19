//! Career types — assessments, skills, career paths

use serde::{Deserialize, Serialize};

/// Career assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Assessment {
    pub id: String,
    pub title: String,
    pub description: String,
    pub category: AssessmentCategory,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_minutes: Option<u32>,
}

/// Assessment category
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AssessmentCategory {
    Competency,
    InterviewPrep,
    SalaryNegotiation,
    CareerTransition,
    Leadership,
    Mentoring,
    Networking,
    SkillGap,
}

/// Career skill with proficiency tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CareerSkill {
    pub id: String,
    pub name: String,
    pub category: String,
    pub current_level: SkillLevel,
    pub target_level: SkillLevel,
}

/// Skill proficiency level
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum SkillLevel {
    Novice,
    Intermediate,
    Advanced,
    Expert,
}

/// Career path definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CareerPath {
    pub id: String,
    pub title: String,
    pub description: String,
    pub from_role: String,
    pub to_role: String,
    #[serde(default)]
    pub required_skill_ids: Vec<String>,
    #[serde(default)]
    pub recommended_course_ids: Vec<String>,
}

/// Competency self-assessment result for a domain
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompetencyResult {
    pub domain_code: String,
    pub domain_name: String,
    pub self_rating: u8,
    pub skill_level: SkillLevel,
}

/// Career maturity level
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum MaturityLevel {
    Foundation,
    Practitioner,
    Specialist,
    Leader,
    Expert,
}

/// Value proposition components
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValueProposition {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_role: Option<String>,
    #[serde(default)]
    pub differentiators: Vec<String>,
    #[serde(default)]
    pub achievements: Vec<String>,
    #[serde(default)]
    pub domain_expertise: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated_statement: Option<String>,
}
