//! Step 7: **Z**ero-out — Remove empties and detect duplicates.
//!
//! Primitive: ∅ Void — "what should cease to exist?"
//!
//! After integration, scans for empty directories and duplicate files
//! (by SHA-256 content hash). Reports findings for cleanup.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::error::{OrganizeError, OrganizeResult};
use crate::integrate::IntegrationPlan;

// ============================================================================
// Types
// ============================================================================

/// A group of duplicate files sharing the same content hash.
///
/// Tier: T2-P (∅ Void — candidates for nullification)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DuplicateGroup {
    /// SHA-256 content hash.
    pub hash: String,
    /// Paths sharing this hash.
    pub paths: Vec<PathBuf>,
    /// Size of each file (all identical).
    pub size_bytes: u64,
    /// Wasted bytes (count - 1) * size.
    pub wasted_bytes: u64,
}

/// Report from the zero-out step.
///
/// Tier: T2-C (∅ Void + Σ Sum — aggregated void findings)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CleanupReport {
    /// Root directory.
    pub root: PathBuf,
    /// Empty directories found.
    pub empty_dirs: Vec<PathBuf>,
    /// Duplicate file groups.
    pub duplicates: Vec<DuplicateGroup>,
    /// Total wasted bytes from duplicates.
    pub total_wasted_bytes: u64,
    /// Whether cleanup was executed (vs dry-run).
    pub executed: bool,
}

// ============================================================================
// Zero-out Function
// ============================================================================

/// Scan for empty directories and duplicate files after integration.
pub fn zero_out(plan: &IntegrationPlan, dry_run: bool) -> OrganizeResult<CleanupReport> {
    let root = &plan.root;

    // Find empty directories
    let empty_dirs = find_empty_dirs(root)?;

    // Find duplicates among surviving files
    let duplicates = find_duplicates(root)?;

    let total_wasted_bytes: u64 = duplicates.iter().map(|d| d.wasted_bytes).sum();

    // Execute cleanup if not dry-run
    if !dry_run {
        for dir in &empty_dirs {
            let _ = std::fs::remove_dir(dir);
        }
    }

    Ok(CleanupReport {
        root: root.clone(),
        empty_dirs,
        duplicates,
        total_wasted_bytes,
        executed: !dry_run,
    })
}

// ============================================================================
// Empty Directory Detection
// ============================================================================

/// Find all empty directories under root.
fn find_empty_dirs(root: &Path) -> OrganizeResult<Vec<PathBuf>> {
    let mut empty = Vec::new();

    if !root.exists() {
        return Ok(empty);
    }

    let walker = walkdir::WalkDir::new(root)
        .min_depth(1)
        .contents_first(true);

    for entry in walker {
        let entry = entry?;
        if entry.file_type().is_dir() {
            let path = entry.path();
            if is_dir_empty(path) {
                empty.push(path.to_path_buf());
            }
        }
    }

    Ok(empty)
}

/// Check if a directory is empty (no entries).
fn is_dir_empty(path: &Path) -> bool {
    match std::fs::read_dir(path) {
        Ok(mut entries) => entries.next().is_none(),
        Err(_) => false,
    }
}

// ============================================================================
// Duplicate Detection
// ============================================================================

/// Find duplicate files by content hash.
fn find_duplicates(root: &Path) -> OrganizeResult<Vec<DuplicateGroup>> {
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut hash_map: HashMap<String, Vec<(PathBuf, u64)>> = HashMap::new();

    let walker = walkdir::WalkDir::new(root).min_depth(1);

    for entry in walker {
        let entry = entry?;
        if entry.file_type().is_file() {
            let path = entry.path();
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);

            // Skip very large files (>100MB) and empty files for performance
            if size == 0 || size > 100_000_000 {
                continue;
            }

            if let Ok(hash) = hash_file(path) {
                hash_map
                    .entry(hash)
                    .or_default()
                    .push((path.to_path_buf(), size));
            }
        }
    }

    // Filter to only groups with 2+ files
    let duplicates: Vec<DuplicateGroup> = hash_map
        .into_iter()
        .filter(|(_, files)| files.len() > 1)
        .map(|(hash, files)| {
            let size_bytes = files.first().map(|(_, s)| *s).unwrap_or(0);
            let count = files.len() as u64;
            let wasted = size_bytes.saturating_mul(count.saturating_sub(1));
            DuplicateGroup {
                hash,
                paths: files.into_iter().map(|(p, _)| p).collect(),
                size_bytes,
                wasted_bytes: wasted,
            }
        })
        .collect();

    Ok(duplicates)
}

/// Compute SHA-256 hash of a file's contents.
fn hash_file(path: &Path) -> OrganizeResult<String> {
    let content = std::fs::read(path).map_err(|e| OrganizeError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;
    let mut hasher = Sha256::new();
    hasher.update(&content);
    let result = hasher.finalize();
    Ok(hex::encode(result))
}

impl CleanupReport {
    /// Total number of issues found.
    pub fn issue_count(&self) -> usize {
        self.empty_dirs.len() + self.duplicates.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_is_dir_empty_true() {
        let tmp = tempfile::tempdir().ok();
        if let Some(ref dir) = tmp {
            let sub = dir.path().join("empty_sub");
            let _ = fs::create_dir(&sub);
            assert!(is_dir_empty(&sub));
        }
    }

    #[test]
    fn test_is_dir_empty_false() {
        let tmp = tempfile::tempdir().ok();
        if let Some(ref dir) = tmp {
            let _ = fs::write(dir.path().join("file.txt"), "content");
            assert!(!is_dir_empty(dir.path()));
        }
    }

    #[test]
    fn test_hash_file_deterministic() {
        let tmp = tempfile::tempdir().ok();
        if let Some(ref dir) = tmp {
            let path = dir.path().join("test.txt");
            let _ = fs::write(&path, "hello world");
            let h1 = hash_file(&path);
            let h2 = hash_file(&path);
            assert!(h1.is_ok());
            assert!(h2.is_ok());
            if let (Ok(h1), Ok(h2)) = (h1, h2) {
                assert_eq!(h1, h2);
            }
        }
    }

    #[test]
    fn test_find_duplicates() {
        let tmp = tempfile::tempdir().ok();
        if let Some(ref dir) = tmp {
            let _ = fs::write(dir.path().join("a.txt"), "same content");
            let _ = fs::write(dir.path().join("b.txt"), "same content");
            let _ = fs::write(dir.path().join("c.txt"), "different");

            let dupes = find_duplicates(dir.path());
            assert!(dupes.is_ok());
            if let Ok(dupes) = dupes {
                assert_eq!(dupes.len(), 1);
                assert_eq!(dupes[0].paths.len(), 2);
            }
        }
    }

    #[test]
    fn test_find_empty_dirs() {
        let tmp = tempfile::tempdir().ok();
        if let Some(ref dir) = tmp {
            let empty = dir.path().join("empty");
            let notempty = dir.path().join("notempty");
            let _ = fs::create_dir(&empty);
            let _ = fs::create_dir(&notempty);
            let _ = fs::write(notempty.join("file.txt"), "hi");

            let empties = find_empty_dirs(dir.path());
            assert!(empties.is_ok());
            if let Ok(empties) = empties {
                assert_eq!(empties.len(), 1);
                assert_eq!(empties[0], empty);
            }
        }
    }

    #[test]
    fn test_cleanup_report_issue_count() {
        let report = CleanupReport {
            root: PathBuf::from("/tmp"),
            empty_dirs: vec![PathBuf::from("/tmp/a"), PathBuf::from("/tmp/b")],
            duplicates: vec![DuplicateGroup {
                hash: "abc".to_string(),
                paths: vec![PathBuf::from("/tmp/x"), PathBuf::from("/tmp/y")],
                size_bytes: 100,
                wasted_bytes: 100,
            }],
            total_wasted_bytes: 100,
            executed: false,
        };
        assert_eq!(report.issue_count(), 3);
    }
}
