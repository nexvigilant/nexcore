//! # State Manager
//!
//! Execution state persistence and checkpoint management.
//!
//! ## Features
//! - Create execution contexts with step tracking
//! - Save/load checkpoints to JSON files
//! - Find resumable executions
//! - Mark steps complete/failed/skipped
//! - Cleanup old checkpoints
//!
//! ## Storage
//! - Location: `~/.claude/chain-state/`
//! - Format: JSON (one file per context)
//! - Naming: `{id}.json`
//!
//! ## Performance Targets
//! - Save checkpoint: < 5ms
//! - Load checkpoint: < 2ms
//! - List checkpoints: < 50ms for 1000 files

use nexcore_chrono::DateTime;
use nexcore_error::Error;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

// ═══════════════════════════════════════════════════════════════════════════
// ERROR TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// State management errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum StateError {
    /// IO error
    #[error("IO error: {0}")]
    IoError(String),
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializeError(String),
    /// Deserialization error
    #[error("Deserialization error: {0}")]
    DeserializeError(String),
    /// Context not found
    #[error("Context not found: {0}")]
    NotFound(String),
    /// Invalid state
    #[error("Invalid state: {0}")]
    InvalidState(String),
    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

// ═══════════════════════════════════════════════════════════════════════════
// TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Execution status for a context.
///
/// ## Codex Classification
/// - **Tier**: T2-C (Cross-Domain Composite — state machine over T1 variants)
/// - **T1 Grounding**: STATE (encapsulated context with transitions)
/// - **Quantification**: `From<ExecutionStatus> for u8` maps Created=0..Cancelled=5
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ExecutionStatus {
    /// Created but not started
    #[default]
    Created,
    /// Currently running
    Running,
    /// Paused (can resume)
    Paused,
    /// All steps completed
    Completed,
    /// Failed with error
    Failed(String),
    /// Cancelled by user
    Cancelled,
}

/// Codex I (QUANTIFY): Every quality becomes a quantity.
impl From<&ExecutionStatus> for u8 {
    fn from(status: &ExecutionStatus) -> Self {
        match status {
            ExecutionStatus::Created => 0,
            ExecutionStatus::Running => 1,
            ExecutionStatus::Paused => 2,
            ExecutionStatus::Completed => 3,
            ExecutionStatus::Failed(_) => 4,
            ExecutionStatus::Cancelled => 5,
        }
    }
}

/// Result of a single step execution.
///
/// ## Codex Classification
/// - **Tier**: T2-C (Cross-Domain Composite — step outcome with timing)
/// - **T1 Grounding**: MAPPING (input step → output result)
/// - **Fields**: `step` (T1 index), `success` (T1 bool), `duration_ms` (T1 u64),
///   `message` (T1 String), `output` (T3 JSON), `completed_at` (T2-P DateTime)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    /// Step index (T1: usize — natural index, no domain wrapping needed)
    pub step: usize,
    /// Whether the step succeeded
    pub success: bool,
    /// Result message
    pub message: String,
    /// Output data (arbitrary JSON)
    pub output: Value,
    /// Duration in milliseconds (T1: u64 — raw measurement)
    pub duration_ms: u64,
    /// Timestamp when step completed
    pub completed_at: DateTime,
}

/// An execution context representing a pipeline run.
///
/// ## Codex Classification
/// - **Tier**: T3 (Domain-Specific — pipeline execution state)
/// - **T1 Grounding**: STATE (encapsulated mutable context with lifecycle)
/// - **Fields**: `id` (T2-P identifier), `name` (T1 String), `status` (T2-C enum),
///   `total_steps`/`completed_steps`/`failed_steps`/`skipped_steps` (T1 indices),
///   `step_results` (T2-C map), `artifacts` (T3 JSON map), `tags` (T1 Vec)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    /// Unique identifier (UUID)
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Current status
    pub status: ExecutionStatus,
    /// Total number of steps
    pub total_steps: usize,
    /// Indices of completed steps
    pub completed_steps: Vec<usize>,
    /// Indices of failed steps
    pub failed_steps: Vec<usize>,
    /// Indices of skipped steps
    pub skipped_steps: Vec<usize>,
    /// Results for each step
    pub step_results: HashMap<String, StepResult>,
    /// When execution started
    pub started_at: DateTime,
    /// When last updated
    pub updated_at: DateTime,
    /// Arbitrary metadata/artifacts
    pub artifacts: HashMap<String, Value>,
    /// Parent context ID (for nested executions)
    pub parent_id: Option<String>,
    /// Tags for filtering
    pub tags: Vec<String>,
}

impl ExecutionContext {
    /// Create a new execution context
    #[must_use]
    pub fn new(name: &str, total_steps: usize) -> Self {
        let now = DateTime::now();
        Self {
            id: nexcore_id::NexId::v4().to_string(),
            name: name.to_string(),
            status: ExecutionStatus::Created,
            total_steps,
            completed_steps: Vec::new(),
            failed_steps: Vec::new(),
            skipped_steps: Vec::new(),
            step_results: HashMap::new(),
            started_at: now,
            updated_at: now,
            artifacts: HashMap::new(),
            parent_id: None,
            tags: Vec::new(),
        }
    }

    /// Get the next step to execute (first non-completed, non-failed, non-skipped)
    #[must_use]
    pub fn next_step(&self) -> Option<usize> {
        (0..self.total_steps).find(|&i| {
            !self.completed_steps.contains(&i)
                && !self.failed_steps.contains(&i)
                && !self.skipped_steps.contains(&i)
        })
    }

    /// Check if execution is complete (all steps processed)
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.completed_steps.len() + self.failed_steps.len() + self.skipped_steps.len()
            >= self.total_steps
    }

    /// Calculate progress percentage
    #[must_use]
    pub fn progress_percent(&self) -> f32 {
        if self.total_steps == 0 {
            return 100.0;
        }
        (self.completed_steps.len() as f32 / self.total_steps as f32) * 100.0
    }

    /// Mark a step as started
    pub fn start_step(&mut self, _step: usize) {
        self.status = ExecutionStatus::Running;
        self.updated_at = DateTime::now();
    }

    /// Mark a step as completed
    pub fn complete_step(&mut self, step: usize, result: StepResult) {
        if !self.completed_steps.contains(&step) {
            self.completed_steps.push(step);
        }
        self.step_results.insert(step.to_string(), result);
        self.updated_at = DateTime::now();

        if self.is_complete() {
            self.status = if self.failed_steps.is_empty() {
                ExecutionStatus::Completed
            } else {
                ExecutionStatus::Failed("One or more steps failed".to_string())
            };
        }
    }

    /// Mark a step as failed
    pub fn fail_step(&mut self, step: usize, _error: &str, result: StepResult) {
        if !self.failed_steps.contains(&step) {
            self.failed_steps.push(step);
        }
        self.step_results.insert(step.to_string(), result);
        self.updated_at = DateTime::now();

        if self.is_complete() {
            self.status = ExecutionStatus::Failed("One or more steps failed".to_string());
        }
    }

    /// Mark a step as skipped
    pub fn skip_step(&mut self, step: usize, reason: &str) {
        if !self.skipped_steps.contains(&step) {
            self.skipped_steps.push(step);
        }
        self.step_results.insert(
            step.to_string(),
            StepResult {
                step,
                success: false,
                message: format!("Skipped: {reason}"),
                output: Value::Null,
                duration_ms: 0,
                completed_at: DateTime::now(),
            },
        );
        self.updated_at = DateTime::now();
    }

    /// Add an artifact
    pub fn add_artifact(&mut self, key: &str, value: Value) {
        self.artifacts.insert(key.to_string(), value);
        self.updated_at = DateTime::now();
    }

    /// Add a tag
    pub fn add_tag(&mut self, tag: &str) {
        if !self.tags.contains(&tag.to_string()) {
            self.tags.push(tag.to_string());
        }
    }
}

/// Checkpoint manager for persisting execution state
#[derive(Debug, Clone)]
pub struct CheckpointManager {
    /// Directory for checkpoint files
    state_dir: PathBuf,
    /// Cache of ID to filename for O(1) lookups
    id_map: HashMap<String, PathBuf>,
}

impl CheckpointManager {
    /// Create a new checkpoint manager
    ///
    /// # Errors
    ///
    /// Returns `StateError` if the directory is invalid or inaccessible.
    pub fn new(state_dir: &str) -> Result<Self, StateError> {
        let path = PathBuf::from(state_dir);

        if path.exists() {
            if !path.is_dir() {
                return Err(StateError::InvalidState(format!(
                    "Path exists but is not a directory: {state_dir}"
                )));
            }
            let metadata =
                std::fs::metadata(&path).map_err(|e| StateError::IoError(e.to_string()))?;
            if metadata.permissions().readonly() {
                return Err(StateError::PermissionDenied(format!(
                    "Directory is read-only: {state_dir}"
                )));
            }
        } else {
            std::fs::create_dir_all(&path).map_err(|e| StateError::IoError(e.to_string()))?;
        }

        let mut manager = Self {
            state_dir: path,
            id_map: HashMap::new(),
        };

        manager.refresh_id_map()?;

        Ok(manager)
    }

    /// Refresh the internal ID to path mapping
    fn refresh_id_map(&mut self) -> Result<(), StateError> {
        self.id_map.clear();
        let entries =
            std::fs::read_dir(&self.state_dir).map_err(|e| StateError::IoError(e.to_string()))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "json") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    if let Some(id) = stem.split('-').next_back() {
                        self.id_map.insert(id.to_string(), path.clone());
                    } else {
                        self.id_map.insert(stem.to_string(), path.clone());
                    }
                }
            }
        }
        Ok(())
    }

    /// Create a new execution context
    #[must_use]
    pub fn create_context(&self, name: &str, total_steps: usize) -> ExecutionContext {
        ExecutionContext::new(name, total_steps)
    }

    /// Save a context to disk
    ///
    /// # Errors
    ///
    /// Returns `StateError` if serialization or writing fails.
    pub fn save(&mut self, context: &ExecutionContext) -> Result<PathBuf, StateError> {
        let filename = format!("{}.json", context.id);
        let path = self.state_dir.join(&filename);

        let json = serde_json::to_string_pretty(context)
            .map_err(|e| StateError::SerializeError(e.to_string()))?;

        std::fs::write(&path, json).map_err(|e| StateError::IoError(e.to_string()))?;

        self.id_map.insert(context.id.clone(), path.clone());

        Ok(path)
    }

    /// Load a context by ID (O(1) lookup via cache)
    ///
    /// # Errors
    ///
    /// Returns `StateError` if reading or parsing fails.
    pub fn load(&self, id: &str) -> Result<Option<ExecutionContext>, StateError> {
        if let Some(path) = self.id_map.get(id) {
            if path.exists() {
                let content = std::fs::read_to_string(path)
                    .map_err(|e| StateError::IoError(e.to_string()))?;

                let context: ExecutionContext = serde_json::from_str(&content)
                    .map_err(|e| StateError::DeserializeError(e.to_string()))?;

                return Ok(Some(context));
            }
        }

        Ok(None)
    }

    /// Find a resumable context by name
    ///
    /// # Errors
    ///
    /// Returns `StateError` if listing fails.
    pub fn find_resumable(&self, name: &str) -> Result<Option<ExecutionContext>, StateError> {
        let contexts = self.list_by_name(name)?;

        let resumable = contexts
            .into_iter()
            .filter(|ctx| {
                matches!(
                    ctx.status,
                    ExecutionStatus::Created | ExecutionStatus::Running | ExecutionStatus::Paused
                )
            })
            .max_by_key(|ctx| ctx.updated_at);

        Ok(resumable)
    }

    /// List all contexts
    ///
    /// # Errors
    ///
    /// Returns `StateError` if reading fails.
    pub fn list(&self) -> Result<Vec<ExecutionContext>, StateError> {
        let mut contexts = Vec::new();

        for path in self.id_map.values() {
            if path.exists() {
                let content = std::fs::read_to_string(path)
                    .map_err(|e| StateError::IoError(e.to_string()))?;

                if let Ok(context) = serde_json::from_str::<ExecutionContext>(&content) {
                    contexts.push(context);
                }
            }
        }

        contexts.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        Ok(contexts)
    }

    /// List contexts by name
    ///
    /// # Errors
    ///
    /// Returns `StateError` if listing fails.
    pub fn list_by_name(&self, name: &str) -> Result<Vec<ExecutionContext>, StateError> {
        let all = self.list()?;
        Ok(all.into_iter().filter(|ctx| ctx.name == name).collect())
    }

    /// List contexts by status
    ///
    /// # Errors
    ///
    /// Returns `StateError` if listing fails.
    pub fn list_by_status(
        &self,
        status: &ExecutionStatus,
    ) -> Result<Vec<ExecutionContext>, StateError> {
        let all = self.list()?;
        Ok(all
            .into_iter()
            .filter(|ctx| &ctx.status == status)
            .collect())
    }

    /// Delete a context by ID
    ///
    /// # Errors
    ///
    /// Returns `StateError` if deletion fails.
    pub fn delete(&mut self, id: &str) -> Result<bool, StateError> {
        if let Some(path) = self.id_map.remove(id) {
            if path.exists() {
                std::fs::remove_file(&path).map_err(|e| StateError::IoError(e.to_string()))?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Cleanup old checkpoints
    ///
    /// Removes checkpoints older than `max_age_days` that are completed, cancelled, or failed.
    ///
    /// # Errors
    ///
    /// Returns `StateError` if listing or deletion fails.
    pub fn cleanup(&mut self, max_age_days: u32) -> Result<usize, StateError> {
        let mut removed = 0;
        let cutoff = DateTime::now() - nexcore_chrono::Duration::days(i64::from(max_age_days));

        let contexts = self.list()?;

        for ctx in contexts {
            if ctx.updated_at < cutoff {
                match ctx.status {
                    ExecutionStatus::Completed
                    | ExecutionStatus::Cancelled
                    | ExecutionStatus::Failed(_) => {
                        if self.delete(&ctx.id)? {
                            removed += 1;
                        }
                    }
                    _ => {} // Don't delete in-progress
                }
            }
        }

        Ok(removed)
    }

    /// Get summary statistics
    ///
    /// # Errors
    ///
    /// Returns `StateError` if listing fails.
    pub fn stats(&self) -> Result<CheckpointStats, StateError> {
        let contexts = self.list()?;

        let mut stats = CheckpointStats {
            total: contexts.len(),
            ..Default::default()
        };

        for ctx in contexts {
            match ctx.status {
                ExecutionStatus::Created => stats.created += 1,
                ExecutionStatus::Running => stats.running += 1,
                ExecutionStatus::Paused => stats.paused += 1,
                ExecutionStatus::Completed => stats.completed += 1,
                ExecutionStatus::Failed(_) => stats.failed += 1,
                ExecutionStatus::Cancelled => stats.cancelled += 1,
            }
        }

        Ok(stats)
    }
}

/// Statistics about checkpoints
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CheckpointStats {
    /// Total checkpoints
    pub total: usize,
    /// Created but not started
    pub created: usize,
    /// Currently running
    pub running: usize,
    /// Paused
    pub paused: usize,
    /// Successfully completed
    pub completed: usize,
    /// Failed
    pub failed: usize,
    /// Cancelled
    pub cancelled: usize,
}

// ═══════════════════════════════════════════════════════════════════════════
// HELPER FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════

/// Quick function to mark a step complete without the full manager
pub fn mark_step_complete(
    context: &mut ExecutionContext,
    step: usize,
    output: Value,
    duration_ms: u64,
) {
    let result = StepResult {
        step,
        success: true,
        message: "Completed".to_string(),
        output,
        duration_ms,
        completed_at: DateTime::now(),
    };
    context.complete_step(step, result);
}

/// Quick function to mark a step failed
pub fn mark_step_failed(
    context: &mut ExecutionContext,
    step: usize,
    error: &str,
    duration_ms: u64,
) {
    let result = StepResult {
        step,
        success: false,
        message: error.to_string(),
        output: Value::Null,
        duration_ms,
        completed_at: DateTime::now(),
    };
    context.fail_step(step, error, result);
}

// ═══════════════════════════════════════════════════════════════════════════
// LEGACY COMPATIBILITY
// ═══════════════════════════════════════════════════════════════════════════

/// A checkpoint capturing execution state (legacy compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Unique checkpoint ID
    pub id: String,
    /// Timestamp when created
    pub timestamp: String,
    /// Task states at checkpoint time
    pub task_states: HashMap<String, String>,
    /// Variables at checkpoint time
    pub variables: HashMap<String, Value>,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

impl Checkpoint {
    /// Create a new checkpoint
    #[must_use]
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            timestamp: DateTime::now().to_rfc3339(),
            task_states: HashMap::new(),
            variables: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// Set a task state
    pub fn set_task_state(&mut self, task: &str, state: &str) {
        self.task_states.insert(task.to_string(), state.to_string());
    }

    /// Set a variable
    pub fn set_variable(&mut self, name: &str, value: Value) {
        self.variables.insert(name.to_string(), value);
    }
}

/// Legacy state manager (for backward compatibility)
#[derive(Debug, Clone)]
pub struct StateManager {
    checkpoint_dir: PathBuf,
    current: Option<Checkpoint>,
}

impl StateManager {
    /// Create a new state manager
    ///
    /// # Errors
    ///
    /// Returns an error if the checkpoint directory cannot be created.
    pub fn new(checkpoint_dir: PathBuf) -> Result<Self, std::io::Error> {
        if !checkpoint_dir.exists() {
            std::fs::create_dir_all(&checkpoint_dir)?;
        }

        Ok(Self {
            checkpoint_dir,
            current: None,
        })
    }

    /// Create a new checkpoint
    pub fn create_checkpoint(&mut self, id: &str) -> &mut Checkpoint {
        self.current.insert(Checkpoint::new(id))
    }

    /// Get the current checkpoint
    #[must_use]
    pub fn current(&self) -> Option<&Checkpoint> {
        self.current.as_ref()
    }

    /// Save the current checkpoint to disk
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or file writing fails.
    pub fn save(&self) -> Result<PathBuf, std::io::Error> {
        let checkpoint = self.current.as_ref().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "No current checkpoint")
        })?;

        let path = self.checkpoint_dir.join(format!("{}.json", checkpoint.id));
        let json = serde_json::to_string_pretty(checkpoint)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        std::fs::write(&path, json)?;
        Ok(path)
    }

    /// Load a checkpoint from disk
    ///
    /// # Errors
    ///
    /// Returns an error if the file doesn't exist or parsing fails.
    pub fn load(&mut self, id: &str) -> Result<&Checkpoint, std::io::Error> {
        let path = self.checkpoint_dir.join(format!("{id}.json"));
        let content = std::fs::read_to_string(&path)?;
        let checkpoint: Checkpoint = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        Ok(self.current.insert(checkpoint))
    }

    /// List all available checkpoints
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be read.
    pub fn list_checkpoints(&self) -> Result<Vec<String>, std::io::Error> {
        let mut checkpoints = Vec::new();

        for entry in std::fs::read_dir(&self.checkpoint_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "json") {
                if let Some(stem) = path.file_stem() {
                    checkpoints.push(stem.to_string_lossy().to_string());
                }
            }
        }

        Ok(checkpoints)
    }

    /// Delete a checkpoint
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be deleted.
    pub fn delete(&self, id: &str) -> Result<(), std::io::Error> {
        let path = self.checkpoint_dir.join(format!("{id}.json"));
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_manager() -> (CheckpointManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let manager = CheckpointManager::new(temp_dir.path().to_str().unwrap()).unwrap();
        (manager, temp_dir)
    }

    // ───────────────────────────────────────────────────────────────────────
    // EXECUTION CONTEXT TESTS
    // ───────────────────────────────────────────────────────────────────────

    #[test]
    fn test_context_creation() {
        let ctx = ExecutionContext::new("test-pipeline", 5);

        assert_eq!(ctx.name, "test-pipeline");
        assert_eq!(ctx.total_steps, 5);
        assert_eq!(ctx.status, ExecutionStatus::Created);
        assert!(ctx.completed_steps.is_empty());
        assert!(ctx.failed_steps.is_empty());
    }

    #[test]
    fn test_context_progress() {
        let mut ctx = ExecutionContext::new("test", 4);

        assert_eq!(ctx.progress_percent(), 0.0);
        assert_eq!(ctx.next_step(), Some(0));

        mark_step_complete(&mut ctx, 0, Value::Null, 100);
        assert_eq!(ctx.progress_percent(), 25.0);
        assert_eq!(ctx.next_step(), Some(1));

        mark_step_complete(&mut ctx, 1, Value::Null, 100);
        assert_eq!(ctx.progress_percent(), 50.0);
    }

    #[test]
    fn test_context_completion() {
        let mut ctx = ExecutionContext::new("test", 2);

        assert!(!ctx.is_complete());

        mark_step_complete(&mut ctx, 0, Value::Null, 100);
        assert!(!ctx.is_complete());

        mark_step_complete(&mut ctx, 1, Value::Null, 100);
        assert!(ctx.is_complete());
        assert_eq!(ctx.status, ExecutionStatus::Completed);
    }

    #[test]
    fn test_context_with_failures() {
        let mut ctx = ExecutionContext::new("test", 2);

        mark_step_complete(&mut ctx, 0, Value::Null, 100);
        mark_step_failed(&mut ctx, 1, "Something went wrong", 50);

        assert!(ctx.is_complete());
        assert!(matches!(ctx.status, ExecutionStatus::Failed(_)));
    }

    #[test]
    fn test_skip_step() {
        let mut ctx = ExecutionContext::new("test", 3);

        mark_step_complete(&mut ctx, 0, Value::Null, 100);
        ctx.skip_step(1, "Dependency failed");
        mark_step_complete(&mut ctx, 2, Value::Null, 100);

        assert!(ctx.is_complete());
        assert!(ctx.skipped_steps.contains(&1));
    }

    // ───────────────────────────────────────────────────────────────────────
    // CHECKPOINT MANAGER TESTS
    // ───────────────────────────────────────────────────────────────────────

    #[test]
    fn test_save_and_load() {
        let (mut manager, _temp) = create_test_manager();

        let ctx = manager.create_context("test-pipeline", 3);
        let id = ctx.id.clone();

        manager.save(&ctx).unwrap();

        let loaded = manager.load(&id).unwrap();
        assert!(loaded.is_some());

        let loaded = loaded.unwrap();
        assert_eq!(loaded.name, "test-pipeline");
        assert_eq!(loaded.total_steps, 3);
    }

    #[test]
    fn test_find_resumable() {
        let (mut manager, _temp) = create_test_manager();

        // Create a completed context
        let mut ctx1 = manager.create_context("my-pipeline", 2);
        mark_step_complete(&mut ctx1, 0, Value::Null, 100);
        mark_step_complete(&mut ctx1, 1, Value::Null, 100);
        manager.save(&ctx1).unwrap();

        // Create an in-progress context
        let mut ctx2 = manager.create_context("my-pipeline", 3);
        mark_step_complete(&mut ctx2, 0, Value::Null, 100);
        ctx2.status = ExecutionStatus::Paused;
        manager.save(&ctx2).unwrap();

        // Should find the paused one
        let resumable = manager.find_resumable("my-pipeline").unwrap();
        assert!(resumable.is_some());
        assert_eq!(resumable.unwrap().id, ctx2.id);
    }

    #[test]
    fn test_list_contexts() {
        let (mut manager, _temp) = create_test_manager();

        manager
            .save(&manager.create_context("pipeline-a", 2))
            .unwrap();
        manager
            .save(&manager.create_context("pipeline-b", 3))
            .unwrap();
        manager
            .save(&manager.create_context("pipeline-a", 4))
            .unwrap();

        let all = manager.list().unwrap();
        assert_eq!(all.len(), 3);

        let by_name = manager.list_by_name("pipeline-a").unwrap();
        assert_eq!(by_name.len(), 2);
    }

    #[test]
    fn test_delete_context() {
        let (mut manager, _temp) = create_test_manager();

        let ctx = manager.create_context("to-delete", 1);
        let id = ctx.id.clone();
        manager.save(&ctx).unwrap();

        assert!(manager.load(&id).unwrap().is_some());

        let deleted = manager.delete(&id).unwrap();
        assert!(deleted);

        assert!(manager.load(&id).unwrap().is_none());
    }

    #[test]
    fn test_stats() {
        let (mut manager, _temp) = create_test_manager();

        let mut ctx1 = manager.create_context("test", 1);
        mark_step_complete(&mut ctx1, 0, Value::Null, 100);
        manager.save(&ctx1).unwrap();

        let ctx2 = manager.create_context("test", 1);
        manager.save(&ctx2).unwrap();

        let stats = manager.stats().unwrap();
        assert_eq!(stats.total, 2);
        assert_eq!(stats.completed, 1);
        assert_eq!(stats.created, 1);
    }

    // ───────────────────────────────────────────────────────────────────────
    // EDGE CASES
    // ───────────────────────────────────────────────────────────────────────

    #[test]
    fn test_empty_pipeline() {
        let ctx = ExecutionContext::new("empty", 0);
        assert!(ctx.is_complete());
        assert_eq!(ctx.progress_percent(), 100.0);
        assert_eq!(ctx.next_step(), None);
    }

    #[test]
    fn test_load_nonexistent() {
        let (manager, _temp) = create_test_manager();
        let result = manager.load("nonexistent-id").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_artifacts() {
        let mut ctx = ExecutionContext::new("test", 1);
        ctx.add_artifact("key1", serde_json::json!({"value": 42}));
        ctx.add_tag("important");

        assert!(ctx.artifacts.contains_key("key1"));
        assert!(ctx.tags.contains(&"important".to_string()));
    }

    // ───────────────────────────────────────────────────────────────────────
    // LEGACY COMPATIBILITY TESTS
    // ───────────────────────────────────────────────────────────────────────

    #[test]
    fn test_legacy_checkpoint_creation() {
        let mut checkpoint = Checkpoint::new("test-1");
        checkpoint.set_task_state("task-a", "completed");
        checkpoint.set_variable("count", serde_json::json!(42));

        assert_eq!(checkpoint.id, "test-1");
        assert_eq!(
            checkpoint.task_states.get("task-a"),
            Some(&"completed".to_string())
        );
        assert_eq!(
            checkpoint.variables.get("count"),
            Some(&serde_json::json!(42))
        );
    }

    #[test]
    fn test_legacy_state_manager() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = StateManager::new(temp_dir.path().to_path_buf()).unwrap();

        {
            let checkpoint = manager.create_checkpoint("test-checkpoint");
            checkpoint.set_task_state("task-1", "running");
        }

        let path = manager.save().unwrap();
        assert!(path.exists());

        let mut manager2 = StateManager::new(temp_dir.path().to_path_buf()).unwrap();
        let loaded = manager2.load("test-checkpoint").unwrap();
        assert_eq!(
            loaded.task_states.get("task-1"),
            Some(&"running".to_string())
        );
    }
}
