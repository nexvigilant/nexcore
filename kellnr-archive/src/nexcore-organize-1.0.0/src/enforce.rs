//! Step 8: **E**nforce — State snapshot for drift detection.
//!
//! Primitive: ς State — "what is the current state, and has it drifted?"
//!
//! Takes a snapshot of the organized directory state (file paths + hashes)
//! and compares against a previous snapshot to detect drift.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};

use crate::error::{OrganizeError, OrganizeResult};

// ============================================================================
// Types
// ============================================================================

/// A snapshot of the organized directory state.
///
/// Tier: T2-C (ς State + π Persistence — persisted state)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OrganizeState {
    /// Root directory.
    pub root: PathBuf,
    /// Snapshot timestamp.
    pub timestamp: DateTime<Utc>,
    /// Map of relative path → content hash.
    pub entries: HashMap<String, String>,
    /// Total number of entries.
    pub count: usize,
}

/// Report of changes detected between two snapshots.
///
/// Tier: T2-C (ς State + κ Comparison — state comparison)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DriftReport {
    /// Previous snapshot timestamp.
    pub previous: DateTime<Utc>,
    /// Current snapshot timestamp.
    pub current: DateTime<Utc>,
    /// Files added since previous snapshot.
    pub added: Vec<String>,
    /// Files removed since previous snapshot.
    pub removed: Vec<String>,
    /// Files whose content changed.
    pub modified: Vec<String>,
    /// Files unchanged.
    pub unchanged: usize,
    /// Whether any drift was detected.
    pub has_drift: bool,
}

// ============================================================================
// Snapshot
// ============================================================================

/// Take a state snapshot of the given directory.
pub fn snapshot(root: &Path) -> OrganizeResult<OrganizeState> {
    if !root.exists() {
        return Err(OrganizeError::Pipeline {
            step: "enforce".to_string(),
            message: format!("root does not exist: {}", root.display()),
        });
    }

    let mut entries = HashMap::new();

    let walker = walkdir::WalkDir::new(root).min_depth(1);

    for entry in walker {
        let entry = entry?;
        if entry.file_type().is_file() {
            let path = entry.path();
            if let Ok(relative) = path.strip_prefix(root) {
                let key = relative.to_string_lossy().to_string();
                let hash = hash_file_quick(path);
                entries.insert(key, hash);
            }
        }
    }

    let count = entries.len();

    Ok(OrganizeState {
        root: root.to_path_buf(),
        timestamp: Utc::now(),
        entries,
        count,
    })
}

/// Compare two snapshots and produce a drift report.
pub fn detect_drift(previous: &OrganizeState, current: &OrganizeState) -> DriftReport {
    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut modified = Vec::new();
    let mut unchanged: usize = 0;

    // Check for added and modified
    for (path, hash) in &current.entries {
        match previous.entries.get(path) {
            Some(prev_hash) if prev_hash == hash => {
                unchanged += 1;
            }
            Some(_) => {
                modified.push(path.clone());
            }
            None => {
                added.push(path.clone());
            }
        }
    }

    // Check for removed
    for path in previous.entries.keys() {
        if !current.entries.contains_key(path) {
            removed.push(path.clone());
        }
    }

    let has_drift = !added.is_empty() || !removed.is_empty() || !modified.is_empty();

    DriftReport {
        previous: previous.timestamp,
        current: current.timestamp,
        added,
        removed,
        modified,
        unchanged,
        has_drift,
    }
}

// ============================================================================
// State Persistence
// ============================================================================

impl OrganizeState {
    /// Save state to a JSON file.
    pub fn save(&self, path: &Path) -> OrganizeResult<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json).map_err(|e| OrganizeError::Io {
            path: path.to_path_buf(),
            source: e,
        })
    }

    /// Load state from a JSON file.
    pub fn load(path: &Path) -> OrganizeResult<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| OrganizeError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;
        let state: Self = serde_json::from_str(&content)?;
        Ok(state)
    }
}

impl DriftReport {
    /// Total number of changes.
    pub fn change_count(&self) -> usize {
        self.added.len() + self.removed.len() + self.modified.len()
    }
}

// ============================================================================
// Helpers
// ============================================================================

/// Quick hash of file contents. Returns "error" on failure.
fn hash_file_quick(path: &Path) -> String {
    match std::fs::read(path) {
        Ok(content) => {
            let mut hasher = Sha256::new();
            hasher.update(&content);
            hex::encode(hasher.finalize())
        }
        Err(_) => "error".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_snapshot_empty_dir() {
        let tmp = tempfile::tempdir().ok();
        if let Some(ref dir) = tmp {
            let state = snapshot(dir.path());
            assert!(state.is_ok());
            if let Ok(state) = state {
                assert_eq!(state.count, 0);
                assert!(state.entries.is_empty());
            }
        }
    }

    #[test]
    fn test_snapshot_with_files() {
        let tmp = tempfile::tempdir().ok();
        if let Some(ref dir) = tmp {
            let _ = fs::write(dir.path().join("a.txt"), "hello");
            let _ = fs::write(dir.path().join("b.txt"), "world");

            let state = snapshot(dir.path());
            assert!(state.is_ok());
            if let Ok(state) = state {
                assert_eq!(state.count, 2);
                assert!(state.entries.contains_key("a.txt"));
                assert!(state.entries.contains_key("b.txt"));
            }
        }
    }

    #[test]
    fn test_detect_drift_no_changes() {
        let mut entries = HashMap::new();
        entries.insert("a.txt".to_string(), "hash1".to_string());

        let s1 = OrganizeState {
            root: PathBuf::from("/tmp"),
            timestamp: Utc::now(),
            entries: entries.clone(),
            count: 1,
        };
        let s2 = OrganizeState {
            root: PathBuf::from("/tmp"),
            timestamp: Utc::now(),
            entries,
            count: 1,
        };

        let drift = detect_drift(&s1, &s2);
        assert!(!drift.has_drift);
        assert_eq!(drift.unchanged, 1);
    }

    #[test]
    fn test_detect_drift_added() {
        let mut entries1 = HashMap::new();
        entries1.insert("a.txt".to_string(), "hash1".to_string());

        let mut entries2 = entries1.clone();
        entries2.insert("b.txt".to_string(), "hash2".to_string());

        let s1 = OrganizeState {
            root: PathBuf::from("/tmp"),
            timestamp: Utc::now(),
            entries: entries1,
            count: 1,
        };
        let s2 = OrganizeState {
            root: PathBuf::from("/tmp"),
            timestamp: Utc::now(),
            entries: entries2,
            count: 2,
        };

        let drift = detect_drift(&s1, &s2);
        assert!(drift.has_drift);
        assert_eq!(drift.added.len(), 1);
        assert_eq!(drift.added[0], "b.txt");
    }

    #[test]
    fn test_detect_drift_removed() {
        let mut entries1 = HashMap::new();
        entries1.insert("a.txt".to_string(), "hash1".to_string());
        entries1.insert("b.txt".to_string(), "hash2".to_string());

        let mut entries2 = HashMap::new();
        entries2.insert("a.txt".to_string(), "hash1".to_string());

        let s1 = OrganizeState {
            root: PathBuf::from("/tmp"),
            timestamp: Utc::now(),
            entries: entries1,
            count: 2,
        };
        let s2 = OrganizeState {
            root: PathBuf::from("/tmp"),
            timestamp: Utc::now(),
            entries: entries2,
            count: 1,
        };

        let drift = detect_drift(&s1, &s2);
        assert!(drift.has_drift);
        assert_eq!(drift.removed.len(), 1);
    }

    #[test]
    fn test_detect_drift_modified() {
        let mut entries1 = HashMap::new();
        entries1.insert("a.txt".to_string(), "hash1".to_string());

        let mut entries2 = HashMap::new();
        entries2.insert("a.txt".to_string(), "hash_changed".to_string());

        let s1 = OrganizeState {
            root: PathBuf::from("/tmp"),
            timestamp: Utc::now(),
            entries: entries1,
            count: 1,
        };
        let s2 = OrganizeState {
            root: PathBuf::from("/tmp"),
            timestamp: Utc::now(),
            entries: entries2,
            count: 1,
        };

        let drift = detect_drift(&s1, &s2);
        assert!(drift.has_drift);
        assert_eq!(drift.modified.len(), 1);
    }

    #[test]
    fn test_state_save_load_roundtrip() {
        let tmp = tempfile::tempdir().ok();
        if let Some(ref dir) = tmp {
            let mut entries = HashMap::new();
            entries.insert("test.txt".to_string(), "abc123".to_string());

            let state = OrganizeState {
                root: PathBuf::from("/tmp"),
                timestamp: Utc::now(),
                entries,
                count: 1,
            };

            let save_path = dir.path().join("state.json");
            let save_result = state.save(&save_path);
            assert!(save_result.is_ok());

            let loaded = OrganizeState::load(&save_path);
            assert!(loaded.is_ok());
            if let Ok(loaded) = loaded {
                assert_eq!(loaded.count, 1);
                assert!(loaded.entries.contains_key("test.txt"));
            }
        }
    }

    #[test]
    fn test_drift_report_change_count() {
        let drift = DriftReport {
            previous: Utc::now(),
            current: Utc::now(),
            added: vec!["a".to_string()],
            removed: vec!["b".to_string(), "c".to_string()],
            modified: vec![],
            unchanged: 5,
            has_drift: true,
        };
        assert_eq!(drift.change_count(), 3);
    }
}
