//! Pipeline state tracking with checkpoints
//!
//! Provides run-level state management with checkpoint markers.
//! Integrates with BrainSession for persistence.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::error::{BrainError, Result};

/// A pipeline run with checkpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineRun {
    /// Unique identifier for this run
    pub id: String,
    /// When the run started
    pub started: DateTime<Utc>,
    /// Current status of the run
    pub status: RunStatus,
    /// Checkpoints recorded during the run
    pub checkpoints: Vec<Checkpoint>,
    /// When the run completed (if finished)
    pub completed: Option<DateTime<Utc>>,
}

/// Run status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RunStatus {
    /// Run is currently executing
    Running,
    /// Run completed successfully
    Completed,
    /// Run failed with an error
    Failed,
    /// Run was cancelled by user
    Cancelled,
}

/// A checkpoint within a run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Name of the checkpoint
    pub name: String,
    /// When the checkpoint was recorded
    pub timestamp: DateTime<Utc>,
    /// Arbitrary data associated with the checkpoint
    pub data: serde_json::Value,
}

/// Pipeline state manager
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PipelineState {
    /// Name of the pipeline
    pub name: String,
    /// All runs for this pipeline
    pub runs: Vec<PipelineRun>,
    /// Most recent checkpoint across all runs
    pub last_checkpoint: Option<Checkpoint>,
}

impl PipelineState {
    /// Create new pipeline state
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            runs: Vec::new(),
            last_checkpoint: None,
        }
    }

    /// Load from JSON file
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    }

    /// Save to JSON file
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Start a new run
    pub fn start_run(&mut self, run_id: Option<String>) -> &PipelineRun {
        let id = run_id.unwrap_or_else(|| Utc::now().format("%Y%m%d_%H%M%S").to_string());
        let run = PipelineRun {
            id,
            started: Utc::now(),
            status: RunStatus::Running,
            checkpoints: Vec::new(),
            completed: None,
        };
        self.runs.push(run);
        self.runs.last().expect("just pushed")
    }

    /// Add checkpoint to current run
    pub fn checkpoint(&mut self, name: impl Into<String>, data: serde_json::Value) -> Result<()> {
        let cp = Checkpoint {
            name: name.into(),
            timestamp: Utc::now(),
            data,
        };
        self.last_checkpoint = Some(cp.clone());
        if let Some(run) = self.runs.last_mut() {
            run.checkpoints.push(cp);
            Ok(())
        } else {
            Err(BrainError::ArtifactNotFound("No active run".into()))
        }
    }

    /// Complete current run
    pub fn complete_run(&mut self, status: RunStatus) -> Result<()> {
        if let Some(run) = self.runs.last_mut() {
            run.status = status;
            run.completed = Some(Utc::now());
            Ok(())
        } else {
            Err(BrainError::ArtifactNotFound("No active run".into()))
        }
    }

    /// Get current run
    pub fn current_run(&self) -> Option<&PipelineRun> {
        self.runs.last()
    }

    /// Summary for display
    pub fn summary(&self) -> String {
        let total = self.runs.len();
        let last = self
            .current_run()
            .map(|r| format!("{} ({})", r.id, r.status_str()))
            .unwrap_or_default();
        let cps = self
            .last_checkpoint
            .as_ref()
            .map(|c| c.name.clone())
            .unwrap_or_default();
        format!(
            "Pipeline: {} | Runs: {} | Last: {} | Checkpoint: {}",
            self.name, total, last, cps
        )
    }
}

impl PipelineRun {
    fn status_str(&self) -> &'static str {
        match self.status {
            RunStatus::Running => "running",
            RunStatus::Completed => "completed",
            RunStatus::Failed => "failed",
            RunStatus::Cancelled => "cancelled",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_lifecycle() {
        let mut state = PipelineState::new("test-pipeline");
        state.start_run(None);
        state
            .checkpoint("step1", serde_json::json!({"progress": 50}))
            .unwrap();
        state
            .checkpoint("step2", serde_json::json!({"progress": 100}))
            .unwrap();
        state.complete_run(RunStatus::Completed).unwrap();

        assert_eq!(state.runs.len(), 1);
        assert_eq!(state.runs[0].checkpoints.len(), 2);
        assert_eq!(state.runs[0].status, RunStatus::Completed);
    }

    #[test]
    fn test_summary() {
        let mut state = PipelineState::new("my-pipe");
        state.start_run(Some("run-001".into()));
        let summary = state.summary();
        assert!(summary.contains("my-pipe"));
        assert!(summary.contains("run-001"));
    }

    #[test]
    fn test_checkpoint_without_run() {
        let mut state = PipelineState::new("test");
        let result = state.checkpoint("orphan", serde_json::json!({}));
        assert!(result.is_err());
    }

    #[test]
    fn test_complete_without_run() {
        let mut state = PipelineState::new("test");
        let result = state.complete_run(RunStatus::Completed);
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_runs() {
        let mut state = PipelineState::new("multi");
        state.start_run(Some("run-1".into()));
        state.complete_run(RunStatus::Completed).unwrap();
        state.start_run(Some("run-2".into()));
        state.complete_run(RunStatus::Failed).unwrap();

        assert_eq!(state.runs.len(), 2);
        assert_eq!(state.runs[0].status, RunStatus::Completed);
        assert_eq!(state.runs[1].status, RunStatus::Failed);
    }

    #[test]
    fn test_run_status_serialization() {
        let status = RunStatus::Running;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"running\"");

        let decoded: RunStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, RunStatus::Running);
    }
}
