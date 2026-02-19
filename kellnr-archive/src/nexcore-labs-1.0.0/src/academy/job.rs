//! Job State Machine
//!
//! Migrated from Python `course-builder/course-builder-service/app/models/job.py`.
//!
//! ## UACA Hierarchy
//!
//! - **L0 Quarks**: Status and stage enumerations
//! - **L1 Atoms**: State transition functions (<20 LOC)
//!
//! ## State Machine
//!
//! ```text
//! QUEUED → PROCESSING → COMPLETED
//!              ↓
//!           FAILED
//! ```
//!
//! ## Safety Axiom
//!
//! State transitions are type-safe. Invalid transitions are compile-time errors
//! when using the typed state pattern.

use serde::{Deserialize, Serialize};

/// Job status enumeration.
///
/// # L0 Quark - Status states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    /// Job is waiting to be processed
    #[default]
    Queued,
    /// Job is currently being processed
    Processing,
    /// Job completed successfully
    Completed,
    /// Job failed with an error
    Failed,
}

impl JobStatus {
    /// Get display string.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Processing => "processing",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }

    /// Check if job is in a terminal state.
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed)
    }

    /// Check if job can be cancelled.
    #[must_use]
    pub const fn is_cancellable(&self) -> bool {
        matches!(self, Self::Queued | Self::Processing)
    }

    /// Check if this is a success state.
    #[must_use]
    pub const fn is_success(&self) -> bool {
        matches!(self, Self::Completed)
    }
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Job pipeline stages.
///
/// # L0 Quark - Stage enumeration
///
/// Represents the current stage in the course generation pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum JobStage {
    /// Initial setup
    #[default]
    Initializing,
    /// Decomposing topic into KSB framework
    KsbDecomposition,
    /// Researching each component
    Research,
    /// Generating learning content
    ContentGeneration,
    /// Validating content quality
    QualityValidation,
    /// Packaging as SCORM
    ScormPackaging,
    /// All stages complete
    Completed,
}

impl JobStage {
    /// Get display string.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Initializing => "initializing",
            Self::KsbDecomposition => "ksb_decomposition",
            Self::Research => "research",
            Self::ContentGeneration => "content_generation",
            Self::QualityValidation => "quality_validation",
            Self::ScormPackaging => "scorm_packaging",
            Self::Completed => "completed",
        }
    }

    /// Get human-readable description.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Initializing => "Initializing job",
            Self::KsbDecomposition => "Decomposing topic into KSB framework",
            Self::Research => "Researching components",
            Self::ContentGeneration => "Generating learning content",
            Self::QualityValidation => "Validating content quality",
            Self::ScormPackaging => "Packaging as SCORM",
            Self::Completed => "All stages complete",
        }
    }

    /// Get stage number (1-7).
    #[must_use]
    pub const fn stage_number(&self) -> u8 {
        match self {
            Self::Initializing => 1,
            Self::KsbDecomposition => 2,
            Self::Research => 3,
            Self::ContentGeneration => 4,
            Self::QualityValidation => 5,
            Self::ScormPackaging => 6,
            Self::Completed => 7,
        }
    }

    /// Total number of stages.
    pub const TOTAL_STAGES: u8 = 7;

    /// Get progress percentage for this stage.
    ///
    /// # Safety Axiom
    /// Maximum value is 100 (7*100/7), which fits in u8.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub const fn progress_percent(&self) -> u8 {
        // Each stage represents ~14% of progress
        ((self.stage_number() as u16 * 100) / Self::TOTAL_STAGES as u16) as u8
    }
}

impl std::fmt::Display for JobStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Task progress tracking.
///
/// # L1 Atom - Progress container
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct TaskProgress {
    /// Total tasks
    pub total: u32,
    /// Pending tasks
    pub pending: u32,
    /// Processing tasks
    pub processing: u32,
    /// Completed tasks
    pub completed: u32,
    /// Failed tasks
    pub failed: u32,
}

impl TaskProgress {
    /// Create new progress tracker.
    #[must_use]
    pub fn new(total: u32) -> Self {
        Self {
            total,
            pending: total,
            processing: 0,
            completed: 0,
            failed: 0,
        }
    }

    /// Check if all tasks are done (completed or failed).
    ///
    /// # L1 Atom - Completion check (<20 LOC)
    #[must_use]
    pub fn is_complete(&self) -> bool {
        (self.completed + self.failed) == self.total
    }

    /// Calculate success rate as percentage.
    ///
    /// # L1 Atom - Success rate (<20 LOC)
    #[must_use]
    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        (f64::from(self.completed) / f64::from(self.total)) * 100.0
    }

    /// Calculate overall progress as percentage.
    #[must_use]
    pub fn progress_percent(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        (f64::from(self.completed + self.failed) / f64::from(self.total)) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_status_terminal() {
        assert!(!JobStatus::Queued.is_terminal());
        assert!(!JobStatus::Processing.is_terminal());
        assert!(JobStatus::Completed.is_terminal());
        assert!(JobStatus::Failed.is_terminal());
    }

    #[test]
    fn test_job_status_cancellable() {
        assert!(JobStatus::Queued.is_cancellable());
        assert!(JobStatus::Processing.is_cancellable());
        assert!(!JobStatus::Completed.is_cancellable());
        assert!(!JobStatus::Failed.is_cancellable());
    }

    #[test]
    fn test_job_stage_progress() {
        assert_eq!(JobStage::Initializing.progress_percent(), 14);
        assert_eq!(JobStage::Completed.progress_percent(), 100);
    }

    #[test]
    fn test_task_progress_is_complete() {
        let mut progress = TaskProgress::new(10);
        assert!(!progress.is_complete());

        progress.completed = 8;
        progress.failed = 2;
        progress.pending = 0;
        assert!(progress.is_complete());
    }

    #[test]
    fn test_task_progress_success_rate() {
        let progress = TaskProgress {
            total: 10,
            pending: 0,
            processing: 0,
            completed: 8,
            failed: 2,
        };
        assert!((progress.success_rate() - 80.0).abs() < 0.01);
    }

    #[test]
    fn test_task_progress_zero_total() {
        let progress = TaskProgress::new(0);
        assert!((progress.success_rate() - 0.0).abs() < f64::EPSILON);
        assert!(progress.is_complete());
    }

    // === Edge Case Tests ===

    #[test]
    fn test_all_stages_have_valid_progress() {
        let stages = [
            JobStage::Initializing,
            JobStage::KsbDecomposition,
            JobStage::Research,
            JobStage::ContentGeneration,
            JobStage::QualityValidation,
            JobStage::ScormPackaging,
            JobStage::Completed,
        ];
        for stage in stages {
            let progress = stage.progress_percent();
            assert!(progress <= 100, "Stage {:?} has progress > 100", stage);
        }
    }

    #[test]
    fn test_stage_numbers_are_sequential() {
        assert_eq!(JobStage::Initializing.stage_number(), 1);
        assert_eq!(JobStage::KsbDecomposition.stage_number(), 2);
        assert_eq!(JobStage::Research.stage_number(), 3);
        assert_eq!(JobStage::ContentGeneration.stage_number(), 4);
        assert_eq!(JobStage::QualityValidation.stage_number(), 5);
        assert_eq!(JobStage::ScormPackaging.stage_number(), 6);
        assert_eq!(JobStage::Completed.stage_number(), 7);
    }

    #[test]
    fn test_task_progress_all_failed() {
        let progress = TaskProgress {
            total: 10,
            pending: 0,
            processing: 0,
            completed: 0,
            failed: 10,
        };
        assert!(progress.is_complete());
        assert!((progress.success_rate() - 0.0).abs() < f64::EPSILON);
        assert!((progress.progress_percent() - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_task_progress_all_completed() {
        let progress = TaskProgress {
            total: 10,
            pending: 0,
            processing: 0,
            completed: 10,
            failed: 0,
        };
        assert!(progress.is_complete());
        assert!((progress.success_rate() - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_job_status_is_success() {
        assert!(!JobStatus::Queued.is_success());
        assert!(!JobStatus::Processing.is_success());
        assert!(JobStatus::Completed.is_success());
        assert!(!JobStatus::Failed.is_success());
    }
}
