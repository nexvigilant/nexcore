//! Step 1: **O**bserve — Recursive inventory with metadata.
//!
//! Primitive: ∃ Existence — "does this file exist, and what are its properties?"
//!
//! Walks the root directory recursively, collecting `EntryMeta` for every
//! file and directory that passes exclusion filters.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use nexcore_chrono::DateTime;
use nexcore_fs::walk::WalkDir;

use crate::config::OrganizeConfig;
use crate::error::{OrganizeError, OrganizeResult};

// ============================================================================
// Types
// ============================================================================

/// Metadata for a single filesystem entry.
///
/// Tier: T2-P (∃ Existence — records that an entry exists with properties)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EntryMeta {
    /// Absolute path to the entry.
    pub path: PathBuf,
    /// Whether this is a directory.
    pub is_dir: bool,
    /// Size in bytes (0 for directories at this stage).
    pub size_bytes: u64,
    /// Last modification time.
    pub modified: DateTime,
    /// File extension (empty for directories or extensionless files).
    pub extension: String,
    /// Depth relative to the root.
    pub depth: usize,
    /// Filename (last component).
    pub name: String,
}

/// Complete inventory of observed entries.
///
/// Tier: T2-C (∃ Existence + σ Sequence — ordered collection of existences)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Inventory {
    /// Root directory that was observed.
    pub root: PathBuf,
    /// All discovered entries.
    pub entries: Vec<EntryMeta>,
    /// Number of entries excluded by filters.
    pub excluded_count: usize,
    /// Timestamp when observation started.
    pub observed_at: DateTime,
}

// ============================================================================
// Observe Function
// ============================================================================

/// Observe the filesystem rooted at `config.root`.
///
/// Returns an `Inventory` containing metadata for all entries that
/// pass the exclusion filters.
pub fn observe(config: &OrganizeConfig) -> OrganizeResult<Inventory> {
    let root = &config.root;

    if !root.exists() {
        return Err(OrganizeError::Pipeline {
            step: "observe".to_string(),
            message: format!("root directory does not exist: {}", root.display()),
        });
    }

    let exclude_set: HashSet<&str> = config.exclude_patterns.iter().map(|s| s.as_str()).collect();
    let observed_at = DateTime::now();

    let mut walker = WalkDir::new(root).follow_links(false);
    if config.max_depth > 0 {
        walker = walker.max_depth(config.max_depth);
    }

    let mut entries = Vec::new();
    let mut excluded_count: usize = 0;

    for entry_result in walker {
        let entry = entry_result?;
        let path = entry.path();

        // Check exclusion patterns against each path component
        if should_exclude(path, root, &exclude_set) {
            excluded_count = excluded_count.saturating_add(1);
            continue;
        }

        // Skip the root directory itself
        if path == root {
            continue;
        }

        let meta = entry_meta_from_direntry(&entry, root)?;
        entries.push(meta);
    }

    Ok(Inventory {
        root: root.clone(),
        entries,
        excluded_count,
        observed_at,
    })
}

// ============================================================================
// Helpers
// ============================================================================

/// Check if a path should be excluded based on patterns.
fn should_exclude(path: &Path, root: &Path, exclude_set: &HashSet<&str>) -> bool {
    if let Ok(relative) = path.strip_prefix(root) {
        for component in relative.components() {
            let name = component.as_os_str().to_string_lossy();
            if exclude_set.contains(name.as_ref()) {
                return true;
            }
        }
    }
    false
}

/// Build an `EntryMeta` from a `nexcore_fs::walk::DirEntry`.
fn entry_meta_from_direntry(
    entry: &nexcore_fs::walk::DirEntry,
    root: &Path,
) -> OrganizeResult<EntryMeta> {
    let path = entry.path().to_path_buf();
    let is_dir = entry.file_type().is_dir();

    let fs_meta = std::fs::metadata(&path).map_err(|e| OrganizeError::Io {
        path: path.clone(),
        source: e.into(),
    })?;

    let size_bytes = if is_dir { 0 } else { fs_meta.len() };

    let modified = fs_meta
        .modified()
        .map(DateTime::from)
        .unwrap_or_else(|_| DateTime::now());

    let extension = path
        .extension()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_default();

    let depth = path
        .strip_prefix(root)
        .map(|r| r.components().count())
        .unwrap_or(0);

    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    Ok(EntryMeta {
        path,
        is_dir,
        size_bytes,
        modified,
        extension,
        depth,
        name,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_observe_nonexistent_root() {
        let config = OrganizeConfig::default_for("/nonexistent/path/that/does/not/exist");
        let result = observe(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_observe_empty_dir() {
        let tmp = tempfile::tempdir().ok();
        if let Some(ref dir) = tmp {
            let config = OrganizeConfig::default_for(dir.path());
            let inv = observe(&config);
            assert!(inv.is_ok());
            let inv = inv.unwrap_or_else(|_| Inventory {
                root: dir.path().to_path_buf(),
                entries: vec![],
                excluded_count: 0,
                observed_at: DateTime::now(),
            });
            assert!(inv.entries.is_empty());
        }
    }

    #[test]
    fn test_observe_with_files() {
        let tmp = tempfile::tempdir().ok();
        if let Some(ref dir) = tmp {
            let _ = fs::write(dir.path().join("hello.rs"), "fn main() {}");
            let _ = fs::write(dir.path().join("readme.md"), "# Hello");
            let _ = fs::create_dir(dir.path().join("subdir"));
            let _ = fs::write(dir.path().join("subdir").join("nested.txt"), "nested");

            let config = OrganizeConfig::default_for(dir.path());
            let inv = observe(&config);
            assert!(inv.is_ok());
            if let Ok(inv) = inv {
                // Should find: hello.rs, readme.md, subdir/, subdir/nested.txt
                assert_eq!(inv.entries.len(), 4);
            }
        }
    }

    #[test]
    fn test_observe_excludes_patterns() {
        let tmp = tempfile::tempdir().ok();
        if let Some(ref dir) = tmp {
            let _ = fs::create_dir(dir.path().join(".git"));
            let _ = fs::write(dir.path().join(".git").join("HEAD"), "ref");
            let _ = fs::write(dir.path().join("keep.rs"), "fn main() {}");

            let config = OrganizeConfig::default_for(dir.path());
            let inv = observe(&config);
            assert!(inv.is_ok());
            if let Ok(inv) = inv {
                // .git and its contents excluded, only keep.rs remains
                assert_eq!(inv.entries.len(), 1);
                assert!(inv.excluded_count > 0);
            }
        }
    }

    #[test]
    fn test_should_exclude() {
        let root = Path::new("/home/user/project");
        let exclude: HashSet<&str> = ["target", ".git"].into_iter().collect();

        assert!(should_exclude(
            Path::new("/home/user/project/target/debug"),
            root,
            &exclude
        ));
        assert!(should_exclude(
            Path::new("/home/user/project/.git/HEAD"),
            root,
            &exclude
        ));
        assert!(!should_exclude(
            Path::new("/home/user/project/src/main.rs"),
            root,
            &exclude
        ));
    }
}
