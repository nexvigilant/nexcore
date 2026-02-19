//! # nexcore Command Center
//!
//! Domain types for the Command Center business operations platform.
//!
//! This crate provides core domain types for:
//! - **Users**: Authentication, sessions, profiles, and RBAC
//! - **Business**: LLCs, transactions, and KPIs
//! - **Tasks**: Task management with recurrence, documents, and compliance
//! - **Learning**: Curriculum progress and onboarding quizzes
//! - **Gamification**: Badges, achievements, and level progression
//! - **Community**: Flarum integration and engagement tracking
//! - **Jobs**: STARK automation job queue
//! - **Audit**: Complete activity logging
//!
//! ## Architecture
//!
//! This crate contains only pure domain types (L1 Atoms in UACA hierarchy).
//! No database, HTTP, or authentication infrastructure - just types and logic.
//!
//! ## Example
//!
//! ```
//! use nexcore_vigilance::command_center::{User, Llc, Task, UserRole, TaskStatus};
//! use nexcore_id::NexId;
//!
//! // Create a user
//! let user = User::new("test@example.com", "Test User");
//! assert!(user.is_active);
//!
//! // Create an LLC
//! let llc = Llc::new("My Company LLC", "12-3456789", "DE");
//! assert_eq!(llc.state_of_formation, "DE");
//!
//! // Create a task
//! let task = Task::new(llc.id, user.id, "Complete quarterly review");
//! assert_eq!(task.status, TaskStatus::Pending);
//! ```

#![forbid(unsafe_code)]

pub mod audit;
pub mod business;
pub mod community;
pub mod enums;
pub mod error;
pub mod gamification;
pub mod jobs;
pub mod learning;
pub mod tasks;
pub mod users;

// Re-export main types at crate root for convenience

// Error types
pub use error::{CommandCenterError, CommandCenterResult};

// Enums
pub use enums::{
    AchievementRarity, AuditAction, BadgeType, CareerVertical, ComplianceStatus, CurriculumTier,
    DocumentStatus, EngagementLevel, FlarumSyncStatus, JobStatus, JobType, ModuleStatus,
    RecurrenceType, TaskPriority, TaskStatus, TransactionType, UserPersona, UserRole,
};

// Users
pub use users::{User, UserLlcRole, UserProfile, UserSession};

// Business
pub use business::{AccountingMethod, Kpi, Llc, Transaction};

// Tasks
pub use tasks::{ComplianceItem, Document, Task};

// Learning
pub use learning::{LearningProgress, OnboardingQuiz};

// Gamification
pub use gamification::{Achievement, LevelInfo, UserBadge};

// Community
pub use community::{CommunityEngagement, FlarumUser};

// Jobs
pub use jobs::StarkJob;

// Audit
pub use audit::{AuditChanges, AuditLog};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crate_integration() {
        // Create a user
        let user = User::new("test@example.com", "Test User");
        assert!(user.is_active);
        assert!(!user.is_verified);

        // Create an LLC
        let llc = Llc::new("Test LLC", "12-3456789", "DE");
        assert!(llc.is_active);

        // Create a task
        let task = Task::new(llc.id, user.id, "Test task");
        assert_eq!(task.status, TaskStatus::Pending);

        // Create an audit log
        let audit = AuditLog::entity_created(user.id, "task", task.id);
        assert_eq!(audit.action, AuditAction::Create);
    }

    #[test]
    fn test_level_calculation() {
        let info = LevelInfo::from_points(500);
        assert!(info.level >= 1);
        assert!(info.level_progress() >= 0.0);
    }
}
