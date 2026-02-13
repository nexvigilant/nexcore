//! History store — JSON persistence for pipeline runs.
//!
//! ## Primitive Foundation
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | T1: π (Persistence) | JSON file storage |
//! | T1: σ (Sequence) | Ordered run history |
//! | T1: λ (Location) | Storage directory path |

use crate::error::{BuildOrcError, BuildOrcResult};
use crate::pipeline::state::PipelineRunState;
use crate::types::PipelineId;
use std::path::{Path, PathBuf};

/// History storage directory relative to workspace root.
const HISTORY_DIR: &str = ".build-orchestrator/history";

/// JSON-file-based history store.
///
/// Tier: T2-C (π + σ + λ + μ, dominant π)
pub struct HistoryStore {
    base_dir: PathBuf,
}

impl HistoryStore {
    /// Create a store rooted at the given workspace.
    pub fn new(workspace_root: &Path) -> BuildOrcResult<Self> {
        let base_dir = workspace_root.join(HISTORY_DIR);
        std::fs::create_dir_all(&base_dir).map_err(|e| BuildOrcError::Io {
            path: base_dir.clone(),
            source: e,
        })?;
        Ok(Self { base_dir })
    }

    /// Path for a given run ID.
    fn run_path(&self, id: &PipelineId) -> PathBuf {
        self.base_dir.join(format!("{}.json", id.0))
    }

    /// Save a pipeline run state.
    pub fn save(&self, state: &PipelineRunState) -> BuildOrcResult<()> {
        let path = self.run_path(&state.id);
        let json = serde_json::to_string_pretty(state)?;
        std::fs::write(&path, json).map_err(|e| BuildOrcError::Io { path, source: e })?;
        tracing::debug!("Saved run {} to history", state.id);
        Ok(())
    }

    /// Load a pipeline run state by ID.
    pub fn load(&self, id: &PipelineId) -> BuildOrcResult<PipelineRunState> {
        let path = self.run_path(id);
        let content =
            std::fs::read_to_string(&path).map_err(|e| BuildOrcError::Io { path, source: e })?;
        let state: PipelineRunState = serde_json::from_str(&content)?;
        Ok(state)
    }

    /// List all saved run IDs, sorted by timestamp (newest first).
    pub fn list_ids(&self) -> BuildOrcResult<Vec<PipelineId>> {
        let mut entries: Vec<(PipelineId, std::time::SystemTime)> = Vec::new();

        let dir_entries = std::fs::read_dir(&self.base_dir).map_err(|e| BuildOrcError::Io {
            path: self.base_dir.clone(),
            source: e,
        })?;

        for entry in dir_entries {
            let entry = entry.map_err(|e| BuildOrcError::Io {
                path: self.base_dir.clone(),
                source: e,
            })?;

            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    let modified = entry
                        .metadata()
                        .and_then(|m| m.modified())
                        .unwrap_or(std::time::UNIX_EPOCH);
                    entries.push((PipelineId(stem.to_string()), modified));
                }
            }
        }

        // Sort newest first
        entries.sort_by(|a, b| b.1.cmp(&a.1));
        Ok(entries.into_iter().map(|(id, _)| id).collect())
    }

    /// Load all runs.
    pub fn load_all(&self) -> BuildOrcResult<Vec<PipelineRunState>> {
        let ids = self.list_ids()?;
        let mut runs = Vec::new();
        for id in &ids {
            match self.load(id) {
                Ok(state) => runs.push(state),
                Err(e) => tracing::warn!("Failed to load run {}: {}", id, e),
            }
        }
        Ok(runs)
    }

    /// Delete a run by ID.
    pub fn delete(&self, id: &PipelineId) -> BuildOrcResult<()> {
        let path = self.run_path(id);
        if path.exists() {
            std::fs::remove_file(&path).map_err(|e| BuildOrcError::Io { path, source: e })?;
        }
        Ok(())
    }

    /// Prune history, keeping only the N most recent runs.
    pub fn prune(&self, keep: usize) -> BuildOrcResult<usize> {
        let ids = self.list_ids()?;
        let mut pruned = 0;
        for id in ids.iter().skip(keep) {
            self.delete(id)?;
            pruned += 1;
        }
        if pruned > 0 {
            tracing::info!("Pruned {} old runs (kept {})", pruned, keep);
        }
        Ok(pruned)
    }
}
