//! Snapshot (artifact) types.
//!
//! Represents versioned artifacts from external brain systems.

use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Type of artifact/snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SnapshotType {
    Task,
    Plan,
    Walkthrough,
    Implementation,
    #[serde(other)]
    Unknown,
}

impl SnapshotType {
    /// Parse from artifact type string.
    #[must_use]
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "ARTIFACT_TYPE_TASK" => Self::Task,
            "ARTIFACT_TYPE_PLAN" => Self::Plan,
            "ARTIFACT_TYPE_WALKTHROUGH" => Self::Walkthrough,
            "ARTIFACT_TYPE_IMPLEMENTATION" => Self::Implementation,
            _ => Self::Unknown,
        }
    }
}

/// Metadata for a snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    #[serde(rename = "artifactType")]
    pub artifact_type: String,
    pub summary: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime,
}

/// A versioned snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// Session UUID this snapshot belongs to
    pub session_id: String,
    /// Base filename (e.g., "task.md")
    pub name: String,
    /// Snapshot type
    pub snapshot_type: SnapshotType,
    /// Current content
    pub content: String,
    /// Metadata
    pub metadata: Option<SnapshotMetadata>,
    /// Available resolved versions
    pub versions: Vec<u32>,
    /// Path to the snapshot file
    pub path: PathBuf,
}

impl Snapshot {
    /// Get the latest resolved version number.
    #[must_use]
    pub fn latest_version(&self) -> Option<u32> {
        self.versions.iter().max().copied()
    }

    /// Check if this snapshot has multiple versions.
    #[must_use]
    pub fn has_history(&self) -> bool {
        self.versions.len() > 1
    }
}

/// Difference between two snapshot versions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotDiff {
    pub name: String,
    pub from_version: u32,
    pub to_version: u32,
    pub additions: usize,
    pub deletions: usize,
    pub diff_text: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_snapshot(versions: Vec<u32>) -> Snapshot {
        Snapshot {
            session_id: "sess-001".to_string(),
            name: "task.md".to_string(),
            snapshot_type: SnapshotType::Task,
            content: "# Task\nDo something.".to_string(),
            metadata: None,
            versions,
            path: PathBuf::from("/tmp/task.md"),
        }
    }

    #[test]
    fn snapshot_type_from_str_task() {
        assert_eq!(
            SnapshotType::from_str("ARTIFACT_TYPE_TASK"),
            SnapshotType::Task
        );
    }

    #[test]
    fn snapshot_type_from_str_plan() {
        assert_eq!(
            SnapshotType::from_str("ARTIFACT_TYPE_PLAN"),
            SnapshotType::Plan
        );
    }

    #[test]
    fn snapshot_type_from_str_walkthrough() {
        assert_eq!(
            SnapshotType::from_str("ARTIFACT_TYPE_WALKTHROUGH"),
            SnapshotType::Walkthrough
        );
    }

    #[test]
    fn snapshot_type_from_str_implementation() {
        assert_eq!(
            SnapshotType::from_str("ARTIFACT_TYPE_IMPLEMENTATION"),
            SnapshotType::Implementation
        );
    }

    #[test]
    fn snapshot_type_from_str_unknown_for_unrecognized() {
        assert_eq!(
            SnapshotType::from_str("something_else"),
            SnapshotType::Unknown
        );
    }

    #[test]
    fn snapshot_type_from_str_case_insensitive() {
        // from_str converts to uppercase, so lowercase input should work
        assert_eq!(
            SnapshotType::from_str("artifact_type_task"),
            SnapshotType::Task
        );
    }

    #[test]
    fn latest_version_returns_max() {
        let snap = make_snapshot(vec![1, 3, 2]);
        assert_eq!(snap.latest_version(), Some(3));
    }

    #[test]
    fn latest_version_none_when_empty() {
        let snap = make_snapshot(vec![]);
        assert_eq!(snap.latest_version(), None);
    }

    #[test]
    fn has_history_true_for_multiple_versions() {
        let snap = make_snapshot(vec![1, 2]);
        assert!(snap.has_history());
    }

    #[test]
    fn has_history_false_for_single_version() {
        let snap = make_snapshot(vec![1]);
        assert!(!snap.has_history());
    }

    #[test]
    fn has_history_false_for_no_versions() {
        let snap = make_snapshot(vec![]);
        assert!(!snap.has_history());
    }

    #[test]
    fn snapshot_diff_fields() {
        let diff = SnapshotDiff {
            name: "task.md".to_string(),
            from_version: 1,
            to_version: 3,
            additions: 10,
            deletions: 2,
            diff_text: "+new line\n-old line".to_string(),
        };
        assert_eq!(diff.name, "task.md");
        assert_eq!(diff.from_version, 1);
        assert_eq!(diff.to_version, 3);
        assert_eq!(diff.additions, 10);
        assert_eq!(diff.deletions, 2);
    }
}
