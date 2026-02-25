//! Build target and workspace scan types.
//!
//! ## Primitive Foundation
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | T1: λ (Location) | Path to crate source |
//! | T1: ∃ (Existence) | Whether artifact exists |
//! | T1: μ (Mapping) | Name → metadata |

use serde::{Deserialize, Serialize};

/// A single build target (crate) in the workspace.
///
/// Tier: T2-C (λ + ∃ + μ + κ, dominant λ)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildTarget {
    /// Crate name.
    pub name: String,
    /// Absolute path to crate directory.
    pub path: String,
    /// SHA-256 hash of source files.
    pub source_hash: Option<String>,
    /// Whether this crate needs rebuilding.
    pub needs_build: bool,
}

/// Result of scanning the workspace.
///
/// Tier: T3 (λ + ∃ + μ + σ + N + π, dominant λ)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceScan {
    /// Workspace root path.
    pub workspace_root: String,
    /// Overall workspace source hash.
    pub workspace_hash: Option<String>,
    /// Number of crates discovered.
    pub crate_count: usize,
    /// Per-crate build targets.
    pub targets: Vec<BuildTarget>,
    /// When the scan was performed.
    pub scanned_at: nexcore_chrono::DateTime,
}

impl WorkspaceScan {
    /// Count of crates needing rebuild.
    #[must_use]
    pub fn dirty_count(&self) -> usize {
        self.targets.iter().filter(|t| t.needs_build).count()
    }

    /// Count of crates up-to-date.
    #[must_use]
    pub fn clean_count(&self) -> usize {
        self.targets.iter().filter(|t| !t.needs_build).count()
    }
}
