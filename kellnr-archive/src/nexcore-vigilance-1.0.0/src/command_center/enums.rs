//! Enumeration types for the Command Center domain.
//!
//! This module contains all enum types used across the command center,
//! organized by domain area.

use serde::{Deserialize, Serialize};

// ============================================================================
// User & Role Enums
// ============================================================================

/// User roles for Role-Based Access Control (RBAC).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    /// Full ownership and control.
    Owner,
    /// Administrative privileges.
    Admin,
    /// Management capabilities.
    Manager,
    /// Financial access.
    Accountant,
    /// Read-only access.
    #[default]
    Viewer,
}

/// User persona types based on market research.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserPersona {
    /// P1 pharmacy student.
    StrategicStudentP1,
    /// P2 pharmacy student.
    StrategicStudentP2,
    /// P3 pharmacy student.
    StrategicStudentP3,
    /// P4 pharmacy student.
    StrategicStudentP4,
    /// 0-2 years post-graduation.
    CareerPivoter0To2,
    /// 3-5 years post-graduation.
    CareerPivoter3To5,
    /// 5+ years post-graduation.
    CareerPivoter5Plus,
    /// Other persona type.
    #[default]
    Other,
}

/// Career vertical interests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CareerVertical {
    /// Medical Affairs.
    MedicalAffairs,
    /// Regulatory Affairs.
    RegulatoryAffairs,
    /// Health Economics & Outcomes Research.
    Heor,
    /// Clinical Development.
    ClinicalDevelopment,
    /// Pharmacovigilance.
    Pharmacovigilance,
    /// Quality Assurance.
    QualityAssurance,
    /// Market Access.
    MarketAccess,
    /// Consulting.
    Consulting,
    /// Undecided.
    #[default]
    Undecided,
    /// Other career path.
    Other,
}

// ============================================================================
// Task & Document Enums
// ============================================================================

/// Task status enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    /// Task not yet started.
    #[default]
    Pending,
    /// Task currently being worked on.
    InProgress,
    /// Task completed successfully.
    Completed,
    /// Task was cancelled.
    Cancelled,
}

/// Task priority enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskPriority {
    /// Low priority.
    Low,
    /// Normal priority.
    #[default]
    Medium,
    /// High priority.
    High,
    /// Critical priority.
    Urgent,
}

/// Recurrence type for scheduled tasks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecurrenceType {
    /// No recurrence.
    #[default]
    None,
    /// Daily recurrence.
    Daily,
    /// Weekly recurrence.
    Weekly,
    /// Monthly recurrence.
    Monthly,
    /// Quarterly recurrence.
    Quarterly,
    /// Annual recurrence.
    Annually,
}

/// Document status enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentStatus {
    /// Document is active and valid.
    #[default]
    Active,
    /// Document is expiring soon.
    ExpiringSoon,
    /// Document has expired.
    Expired,
    /// Document has been renewed.
    Renewed,
}

// ============================================================================
// Financial & Business Enums
// ============================================================================

/// Transaction type enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    /// Income/revenue transaction.
    Revenue,
    /// Expense transaction.
    Expense,
}

/// Compliance status enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceStatus {
    /// In compliance.
    #[default]
    Compliant,
    /// Due soon.
    DueSoon,
    /// Past due date.
    Overdue,
    /// Already filed.
    Filed,
    /// Not applicable to this entity.
    NotApplicable,
}

// ============================================================================
// Job & Automation Enums
// ============================================================================

/// Job status enumeration for STARK automation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    /// Job is queued.
    #[default]
    Pending,
    /// Job is currently executing.
    Running,
    /// Job completed successfully.
    Completed,
    /// Job failed with error.
    Failed,
    /// Job was cancelled.
    Cancelled,
}

/// Job type enumeration for STARK automation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobType {
    /// Google Drive search job.
    DriveSearch,
    /// Google Calendar sync job.
    CalendarSync,
    /// Google Apps Script trigger.
    AppsScriptTrigger,
    /// ABAS analysis job.
    AbasAnalysis,
}

// ============================================================================
// Audit Enums
// ============================================================================

/// Audit action enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    /// Create action.
    Create,
    /// Read action.
    Read,
    /// Update action.
    Update,
    /// Delete action.
    Delete,
    /// User login.
    Login,
    /// User logout.
    Logout,
    /// Failed login attempt.
    FailedLogin,
    /// Data export.
    Export,
    /// Data import.
    Import,
    /// Email sent.
    EmailSent,
}

// ============================================================================
// Learning & Curriculum Enums
// ============================================================================

/// 3-tier curriculum model based on market research.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CurriculumTier {
    /// Industry fluency (free tier).
    #[default]
    Foundation,
    /// Job-ready skills (premium).
    Specialization,
    /// Getting hired (premium).
    Accelerator,
}

/// Module completion status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModuleStatus {
    /// Module not started.
    #[default]
    NotStarted,
    /// Module in progress.
    InProgress,
    /// Module completed.
    Completed,
    /// Module skipped.
    Skipped,
}

// ============================================================================
// Community & Integration Enums
// ============================================================================

/// Sync status between our system and Flarum forum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlarumSyncStatus {
    /// Sync pending.
    #[default]
    Pending,
    /// Successfully synced.
    Synced,
    /// Sync failed.
    Failed,
    /// Local data is outdated.
    Outdated,
}

/// Engagement level classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EngagementLevel {
    /// Minimal participation.
    #[default]
    Lurker,
    /// Occasional participation.
    Casual,
    /// Regular participation.
    Active,
    /// Heavy participation.
    PowerUser,
    /// Domain expert.
    Expert,
}

// ============================================================================
// Gamification Enums
// ============================================================================

/// Badge categories based on market research.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BadgeType {
    // Onboarding Badges
    /// New member badge.
    NewMember,
    /// Profile completed.
    ProfileComplete,
    /// Quiz completed.
    QuizComplete,

    // Community Engagement
    /// First post made.
    FirstPost,
    /// 10+ posts.
    ActiveContributor,
    /// 50+ posts.
    PowerUser,
    /// 100+ posts.
    CommunityChampion,

    // Helpfulness
    /// 5+ best answers.
    Helpful,
    /// 20+ best answers.
    Expert,
    /// 50+ best answers.
    Mentor,

    // Learning Progress
    /// Completed foundation tier.
    FoundationComplete,
    /// Completed specialization tier.
    Specialist,
    /// Completed accelerator tier.
    CareerReady,

    // Vertical Expertise
    /// Medical Affairs expert.
    MedicalAffairsExpert,
    /// Regulatory Affairs expert.
    RegulatoryExpert,
    /// HEOR expert.
    HeorExpert,
    /// Clinical Development expert.
    ClinicalDevExpert,

    // Streaks & Consistency
    /// 7-day streak.
    WeekStreak,
    /// 30-day streak.
    MonthStreak,
    /// 1 year member.
    YearMember,

    // Premium
    /// Premium subscriber.
    PremiumMember,
    /// Founding member.
    FoundingMember,

    // Special
    /// Monthly top contributor.
    TopContributor,
    /// Early adopter.
    EarlyAdopter,
    /// Beta tester.
    BetaTester,
}

/// Achievement rarity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AchievementRarity {
    /// Common achievement.
    #[default]
    Common,
    /// Uncommon achievement.
    Uncommon,
    /// Rare achievement.
    Rare,
    /// Legendary achievement.
    Legendary,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_role_default() {
        assert_eq!(UserRole::default(), UserRole::Viewer);
    }

    #[test]
    fn test_task_status_serialization() {
        let status = TaskStatus::InProgress;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"in_progress\"");

        let parsed: TaskStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, TaskStatus::InProgress);
    }

    #[test]
    fn test_badge_type_serialization() {
        let badge = BadgeType::MedicalAffairsExpert;
        let json = serde_json::to_string(&badge).unwrap();
        assert_eq!(json, "\"medical_affairs_expert\"");
    }

    #[test]
    fn test_curriculum_tier_default() {
        assert_eq!(CurriculumTier::default(), CurriculumTier::Foundation);
    }

    #[test]
    fn test_career_vertical_all_variants() {
        let verticals = [
            CareerVertical::MedicalAffairs,
            CareerVertical::RegulatoryAffairs,
            CareerVertical::Heor,
            CareerVertical::ClinicalDevelopment,
            CareerVertical::Pharmacovigilance,
            CareerVertical::QualityAssurance,
            CareerVertical::MarketAccess,
            CareerVertical::Consulting,
            CareerVertical::Undecided,
            CareerVertical::Other,
        ];

        for vertical in verticals {
            let json = serde_json::to_string(&vertical).unwrap();
            let parsed: CareerVertical = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, vertical);
        }
    }
}
