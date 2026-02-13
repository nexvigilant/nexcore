//! Pipeline and stage runtime state.
//!
//! ## Primitive Foundation
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | T1: ς (State) | RunStatus FSM with enforced transitions |
//! | T1: Σ (Sum) | Status variants |
//! | T1: N (Quantity) | Timing data |
//! | T1: σ (Sequence) | Ordered stage results |

use crate::error::{BuildOrcError, BuildOrcResult};
use crate::types::{BuildDuration, LogStream, PipelineId, StageId};
use serde::{Deserialize, Serialize};

/// Run status for a pipeline or stage.
///
/// Tier: T2-P (ς + Σ, dominant ς, StateMode::Modal)
///
/// Enforced transitions:
/// - `Pending → Running`
/// - `Running → Completed | Failed | Cancelled`
/// - `Pending → Skipped`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RunStatus {
    /// Waiting to start.
    Pending,
    /// Currently executing.
    Running,
    /// Finished successfully.
    Completed,
    /// Finished with errors.
    Failed,
    /// Cancelled by user or dependency failure.
    Cancelled,
    /// Skipped (dependency not met or not applicable).
    Skipped,
}

impl RunStatus {
    /// Check if this status represents a terminal state.
    #[must_use]
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Failed | Self::Cancelled | Self::Skipped
        )
    }

    /// Check if this status represents success.
    #[must_use]
    pub fn is_success(self) -> bool {
        self == Self::Completed
    }

    /// Validate a state transition. Returns error if invalid.
    pub fn transition_to(self, next: RunStatus) -> BuildOrcResult<RunStatus> {
        let valid = match (self, next) {
            (Self::Pending, Self::Running) => true,
            (Self::Pending, Self::Skipped) => true,
            (Self::Pending, Self::Cancelled) => true,
            (Self::Running, Self::Completed) => true,
            (Self::Running, Self::Failed) => true,
            (Self::Running, Self::Cancelled) => true,
            _ => false,
        };
        if valid {
            Ok(next)
        } else {
            Err(BuildOrcError::InvalidTransition {
                from: format!("{self:?}"),
                to: format!("{next:?}"),
            })
        }
    }
}

impl std::fmt::Display for RunStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "⏳ Pending"),
            Self::Running => write!(f, "🔄 Running"),
            Self::Completed => write!(f, "✅ Completed"),
            Self::Failed => write!(f, "❌ Failed"),
            Self::Cancelled => write!(f, "⛔ Cancelled"),
            Self::Skipped => write!(f, "⏭️ Skipped"),
        }
    }
}

/// Runtime state for a single stage execution.
///
/// Tier: T2-C (ς + N + σ + Σ, dominant ς)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageRunState {
    /// Stage identifier.
    pub stage_id: StageId,
    /// Current status.
    pub status: RunStatus,
    /// Exit code (if completed/failed).
    pub exit_code: Option<i32>,
    /// Execution duration.
    pub duration: Option<BuildDuration>,
    /// Captured log output.
    pub logs: LogStream,
    /// When execution started.
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    /// When execution ended.
    pub ended_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl StageRunState {
    /// Create a new pending stage state.
    #[must_use]
    pub fn new(stage_id: StageId) -> Self {
        Self {
            stage_id,
            status: RunStatus::Pending,
            exit_code: None,
            duration: None,
            logs: Vec::new(),
            started_at: None,
            ended_at: None,
        }
    }

    /// Mark as running.
    pub fn start(&mut self) -> BuildOrcResult<()> {
        self.status = self.status.transition_to(RunStatus::Running)?;
        self.started_at = Some(chrono::Utc::now());
        Ok(())
    }

    /// Mark as completed with exit code.
    pub fn complete(&mut self, exit_code: i32) -> BuildOrcResult<()> {
        let next = if exit_code == 0 {
            RunStatus::Completed
        } else {
            RunStatus::Failed
        };
        self.status = self.status.transition_to(next)?;
        self.exit_code = Some(exit_code);
        self.ended_at = Some(chrono::Utc::now());
        if let Some(start) = self.started_at {
            let elapsed = chrono::Utc::now() - start;
            self.duration = Some(BuildDuration {
                millis: elapsed.num_milliseconds().max(0) as u64,
            });
        }
        Ok(())
    }

    /// Mark as cancelled.
    pub fn cancel(&mut self) -> BuildOrcResult<()> {
        self.status = self.status.transition_to(RunStatus::Cancelled)?;
        self.ended_at = Some(chrono::Utc::now());
        Ok(())
    }

    /// Mark as skipped.
    pub fn skip(&mut self) -> BuildOrcResult<()> {
        self.status = self.status.transition_to(RunStatus::Skipped)?;
        Ok(())
    }
}

/// Runtime state for an entire pipeline execution.
///
/// Tier: T3 (ς + σ + N + → + π + ∂ + μ, dominant ς, StateMode::Mutable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineRunState {
    /// Unique run identifier.
    pub id: PipelineId,
    /// Pipeline definition name.
    pub definition_name: String,
    /// Overall status.
    pub status: RunStatus,
    /// Per-stage states (σ: ordered sequence).
    pub stages: Vec<StageRunState>,
    /// When the pipeline started.
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// When the pipeline ended.
    pub ended_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Total duration.
    pub total_duration: Option<BuildDuration>,
    /// Source hash at start (∃: existence check).
    pub source_hash: Option<String>,
    /// Workspace root path (λ: location).
    pub workspace_root: String,
}

impl PipelineRunState {
    /// Create a new pipeline run state.
    #[must_use]
    pub fn new(definition_name: &str, stage_ids: &[StageId], workspace_root: &str) -> Self {
        Self {
            id: PipelineId::generate(),
            definition_name: definition_name.to_string(),
            status: RunStatus::Pending,
            stages: stage_ids
                .iter()
                .map(|id| StageRunState::new(id.clone()))
                .collect(),
            started_at: chrono::Utc::now(),
            ended_at: None,
            total_duration: None,
            source_hash: None,
            workspace_root: workspace_root.to_string(),
        }
    }

    /// Get a mutable reference to a stage by ID.
    pub fn stage_mut(&mut self, id: &StageId) -> Option<&mut StageRunState> {
        self.stages.iter_mut().find(|s| &s.stage_id == id)
    }

    /// Get a reference to a stage by ID.
    pub fn stage(&self, id: &StageId) -> Option<&StageRunState> {
        self.stages.iter().find(|s| &s.stage_id == id)
    }

    /// Count completed stages.
    #[must_use]
    pub fn completed_count(&self) -> usize {
        self.stages
            .iter()
            .filter(|s| s.status.is_terminal())
            .count()
    }

    /// Count successful stages.
    #[must_use]
    pub fn success_count(&self) -> usize {
        self.stages.iter().filter(|s| s.status.is_success()).count()
    }

    /// Check if all stages are terminal.
    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.stages.iter().all(|s| s.status.is_terminal())
    }

    /// Finalize the pipeline run.
    pub fn finalize(&mut self) {
        self.ended_at = Some(chrono::Utc::now());
        let elapsed = chrono::Utc::now() - self.started_at;
        self.total_duration = Some(BuildDuration {
            millis: elapsed.num_milliseconds().max(0) as u64,
        });

        let any_failed = self.stages.iter().any(|s| s.status == RunStatus::Failed);
        let any_cancelled = self.stages.iter().any(|s| s.status == RunStatus::Cancelled);

        if any_failed {
            self.status = RunStatus::Failed;
        } else if any_cancelled {
            self.status = RunStatus::Cancelled;
        } else {
            self.status = RunStatus::Completed;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // RunStatus FSM transition tests
    // ========================================================================

    #[test]
    fn pending_to_running_valid() {
        let result = RunStatus::Pending.transition_to(RunStatus::Running);
        assert!(result.is_ok());
        assert_eq!(result.unwrap_or(RunStatus::Pending), RunStatus::Running);
    }

    #[test]
    fn pending_to_skipped_valid() {
        let result = RunStatus::Pending.transition_to(RunStatus::Skipped);
        assert!(result.is_ok());
    }

    #[test]
    fn pending_to_cancelled_valid() {
        let result = RunStatus::Pending.transition_to(RunStatus::Cancelled);
        assert!(result.is_ok());
    }

    #[test]
    fn running_to_completed_valid() {
        let result = RunStatus::Running.transition_to(RunStatus::Completed);
        assert!(result.is_ok());
    }

    #[test]
    fn running_to_failed_valid() {
        let result = RunStatus::Running.transition_to(RunStatus::Failed);
        assert!(result.is_ok());
    }

    #[test]
    fn running_to_cancelled_valid() {
        let result = RunStatus::Running.transition_to(RunStatus::Cancelled);
        assert!(result.is_ok());
    }

    #[test]
    fn completed_to_running_invalid() {
        let result = RunStatus::Completed.transition_to(RunStatus::Running);
        assert!(result.is_err());
    }

    #[test]
    fn failed_to_completed_invalid() {
        let result = RunStatus::Failed.transition_to(RunStatus::Completed);
        assert!(result.is_err());
    }

    #[test]
    fn pending_to_completed_invalid() {
        let result = RunStatus::Pending.transition_to(RunStatus::Completed);
        assert!(result.is_err());
    }

    #[test]
    fn pending_to_failed_invalid() {
        let result = RunStatus::Pending.transition_to(RunStatus::Failed);
        assert!(result.is_err());
    }

    #[test]
    fn terminal_states_correct() {
        assert!(!RunStatus::Pending.is_terminal());
        assert!(!RunStatus::Running.is_terminal());
        assert!(RunStatus::Completed.is_terminal());
        assert!(RunStatus::Failed.is_terminal());
        assert!(RunStatus::Cancelled.is_terminal());
        assert!(RunStatus::Skipped.is_terminal());
    }

    #[test]
    fn success_only_completed() {
        assert!(RunStatus::Completed.is_success());
        assert!(!RunStatus::Pending.is_success());
        assert!(!RunStatus::Running.is_success());
        assert!(!RunStatus::Failed.is_success());
        assert!(!RunStatus::Cancelled.is_success());
        assert!(!RunStatus::Skipped.is_success());
    }

    // ========================================================================
    // StageRunState lifecycle tests
    // ========================================================================

    #[test]
    fn stage_start_sets_running() {
        let mut stage = StageRunState::new(StageId("test".into()));
        assert_eq!(stage.status, RunStatus::Pending);
        assert!(stage.start().is_ok());
        assert_eq!(stage.status, RunStatus::Running);
        assert!(stage.started_at.is_some());
    }

    #[test]
    fn stage_complete_success() {
        let mut stage = StageRunState::new(StageId("test".into()));
        let _ = stage.start();
        assert!(stage.complete(0).is_ok());
        assert_eq!(stage.status, RunStatus::Completed);
        assert_eq!(stage.exit_code, Some(0));
        assert!(stage.ended_at.is_some());
        assert!(stage.duration.is_some());
    }

    #[test]
    fn stage_complete_failure() {
        let mut stage = StageRunState::new(StageId("test".into()));
        let _ = stage.start();
        assert!(stage.complete(1).is_ok());
        assert_eq!(stage.status, RunStatus::Failed);
        assert_eq!(stage.exit_code, Some(1));
    }

    #[test]
    fn stage_skip_from_pending() {
        let mut stage = StageRunState::new(StageId("test".into()));
        assert!(stage.skip().is_ok());
        assert_eq!(stage.status, RunStatus::Skipped);
    }

    #[test]
    fn stage_cancel_from_running() {
        let mut stage = StageRunState::new(StageId("test".into()));
        let _ = stage.start();
        assert!(stage.cancel().is_ok());
        assert_eq!(stage.status, RunStatus::Cancelled);
    }

    #[test]
    fn stage_double_start_invalid() {
        let mut stage = StageRunState::new(StageId("test".into()));
        let _ = stage.start();
        assert!(stage.start().is_err());
    }

    // ========================================================================
    // PipelineRunState tests
    // ========================================================================

    #[test]
    fn pipeline_counts() {
        let ids = vec![
            StageId("a".into()),
            StageId("b".into()),
            StageId("c".into()),
        ];
        let mut state = PipelineRunState::new("test", &ids, "/tmp");
        assert_eq!(state.completed_count(), 0);
        assert_eq!(state.success_count(), 0);
        assert!(!state.is_finished());

        // Complete stage a successfully
        if let Some(s) = state.stage_mut(&StageId("a".into())) {
            let _ = s.start();
            let _ = s.complete(0);
        }
        assert_eq!(state.completed_count(), 1);
        assert_eq!(state.success_count(), 1);

        // Fail stage b
        if let Some(s) = state.stage_mut(&StageId("b".into())) {
            let _ = s.start();
            let _ = s.complete(1);
        }
        assert_eq!(state.completed_count(), 2);
        assert_eq!(state.success_count(), 1);

        // Skip stage c
        if let Some(s) = state.stage_mut(&StageId("c".into())) {
            let _ = s.skip();
        }
        assert!(state.is_finished());
    }

    #[test]
    fn pipeline_finalize_sets_failed() {
        let ids = vec![StageId("a".into()), StageId("b".into())];
        let mut state = PipelineRunState::new("test", &ids, "/tmp");

        if let Some(s) = state.stage_mut(&StageId("a".into())) {
            let _ = s.start();
            let _ = s.complete(0);
        }
        if let Some(s) = state.stage_mut(&StageId("b".into())) {
            let _ = s.start();
            let _ = s.complete(1);
        }

        state.finalize();
        assert_eq!(state.status, RunStatus::Failed);
        assert!(state.total_duration.is_some());
    }

    #[test]
    fn pipeline_finalize_all_success() {
        let ids = vec![StageId("a".into())];
        let mut state = PipelineRunState::new("test", &ids, "/tmp");

        if let Some(s) = state.stage_mut(&StageId("a".into())) {
            let _ = s.start();
            let _ = s.complete(0);
        }

        state.finalize();
        assert_eq!(state.status, RunStatus::Completed);
    }

    #[test]
    fn run_status_serde_roundtrip() {
        let statuses = [
            RunStatus::Pending,
            RunStatus::Running,
            RunStatus::Completed,
            RunStatus::Failed,
            RunStatus::Cancelled,
            RunStatus::Skipped,
        ];
        for status in statuses {
            let json = serde_json::to_string(&status);
            assert!(json.is_ok());
            let back: Result<RunStatus, _> = serde_json::from_str(&json.unwrap_or_default());
            assert!(back.is_ok());
            assert_eq!(back.unwrap_or(RunStatus::Pending), status);
        }
    }
}
