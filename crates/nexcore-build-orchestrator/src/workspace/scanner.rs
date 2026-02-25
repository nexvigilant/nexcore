//! Workspace scanner — discover crates and detect changes.
//!
//! ## Primitive Foundation
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | T1: λ (Location) | Source paths |
//! | T1: ∃ (Existence) | Check if crate/artifact exists |
//! | T1: μ (Mapping) | Crate name → path mapping |

use crate::error::{BuildOrcError, BuildOrcResult};
use crate::workspace::target::{BuildTarget, WorkspaceScan};
use std::path::Path;

/// Scan a workspace to discover crates and their state.
pub fn scan_workspace(workspace_root: &Path) -> BuildOrcResult<WorkspaceScan> {
    let crates_dir = workspace_root.join("crates");
    if !crates_dir.exists() {
        return Err(BuildOrcError::WorkspaceScan(
            "crates/ directory not found".into(),
        ));
    }

    let mut targets = Vec::new();

    let entries = std::fs::read_dir(&crates_dir).map_err(|e| BuildOrcError::Io {
        path: crates_dir.clone(),
        source: e,
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| BuildOrcError::Io {
            path: crates_dir.clone(),
            source: e,
        })?;

        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let cargo_toml = path.join("Cargo.toml");
        if !cargo_toml.exists() {
            continue;
        }

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Compute source hash via build-gate
        let hash = nexcore_build_gate::hash_source_dir(&path).ok();

        // Check if build is needed
        let needs_build = nexcore_build_gate::should_build(&path).unwrap_or(true);

        targets.push(BuildTarget {
            name,
            path: path.display().to_string(),
            source_hash: hash,
            needs_build,
        });
    }

    targets.sort_by(|a, b| a.name.cmp(&b.name));

    // Compute workspace-level hash
    let workspace_hash = nexcore_build_gate::hash_source_dir(workspace_root).ok();

    Ok(WorkspaceScan {
        workspace_root: workspace_root.display().to_string(),
        workspace_hash,
        crate_count: targets.len(),
        targets,
        scanned_at: nexcore_chrono::DateTime::now(),
    })
}
