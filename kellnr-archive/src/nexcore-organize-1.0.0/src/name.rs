//! Step 5: **N**ame — Naming conventions and conflict detection.
//!
//! Primitive: ∂ Boundary — "what are the naming constraints?"
//!
//! Applies naming rules (lowercase, space replacement, length limits)
//! and detects/resolves filename collisions at target paths.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::assign::{AssignedInventory, Assignment};
use crate::config::{CollisionStrategy, FileOp, NamingConfig, OrganizeConfig};
use crate::error::{OrganizeError, OrganizeResult};

// ============================================================================
// Types
// ============================================================================

/// A rename operation to apply.
///
/// Tier: T2-P (∂ Boundary — old name → new name boundary)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RenameOp {
    /// Original assignment.
    pub assignment: Assignment,
    /// Final target path (after naming rules + collision resolution).
    pub target_path: PathBuf,
    /// Whether a rename is needed (target differs from source).
    pub needs_rename: bool,
}

/// Inventory after naming.
///
/// Tier: T2-C (∂ Boundary + → Causality — named causal assignments)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NamedInventory {
    /// Root directory.
    pub root: PathBuf,
    /// All rename operations.
    pub operations: Vec<RenameOp>,
    /// Number of collisions detected and resolved.
    pub collisions_resolved: usize,
    /// Observation timestamp (carried forward).
    pub observed_at: chrono::DateTime<chrono::Utc>,
}

// ============================================================================
// Name Function
// ============================================================================

/// Apply naming rules and resolve collisions.
pub fn name(
    assigned: AssignedInventory,
    config: &OrganizeConfig,
) -> OrganizeResult<NamedInventory> {
    let naming = &config.naming;
    let mut operations = Vec::new();
    let mut collisions_resolved: usize = 0;

    // Track occupied target paths to detect collisions
    let mut occupied: HashMap<PathBuf, PathBuf> = HashMap::new();

    for assignment in assigned.assignments {
        let target_path = compute_target_path(&assignment, naming)?;

        // Check for collision
        let final_path = if let Some(existing) = occupied.get(&target_path) {
            match naming.collision_strategy {
                CollisionStrategy::Suffix => {
                    let resolved = resolve_with_suffix(&target_path);
                    collisions_resolved += 1;
                    resolved
                }
                CollisionStrategy::Skip => {
                    // Keep original path (no move)
                    assignment.entry.meta.path.clone()
                }
                CollisionStrategy::Error => {
                    return Err(OrganizeError::NamingConflict {
                        existing: existing.clone(),
                        incoming: assignment.entry.meta.path.clone(),
                        target: target_path,
                    });
                }
            }
        } else {
            target_path.clone()
        };

        let needs_rename = final_path != assignment.entry.meta.path;

        occupied.insert(final_path.clone(), assignment.entry.meta.path.clone());

        operations.push(RenameOp {
            assignment,
            target_path: final_path,
            needs_rename,
        });
    }

    Ok(NamedInventory {
        root: assigned.root,
        operations,
        collisions_resolved,
        observed_at: assigned.observed_at,
    })
}

// ============================================================================
// Naming Helpers
// ============================================================================

/// Compute the target path for an assignment after applying naming rules.
fn compute_target_path(assignment: &Assignment, naming: &NamingConfig) -> OrganizeResult<PathBuf> {
    let source = &assignment.entry.meta.path;

    // For Keep actions, target is the same as source
    if assignment.action == FileOp::Keep {
        let filename = apply_naming_rules(source, naming);
        if let Some(parent) = source.parent() {
            return Ok(parent.join(filename));
        }
        return Ok(PathBuf::from(filename));
    }

    // For Move/Archive, use target_dir if specified
    let target_dir = assignment
        .target_dir
        .as_deref()
        .unwrap_or_else(|| source.parent().unwrap_or(Path::new(".")));

    let filename = apply_naming_rules(source, naming);
    Ok(target_dir.join(filename))
}

/// Apply naming conventions to a filename.
fn apply_naming_rules(path: &Path, naming: &NamingConfig) -> String {
    let mut name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    if naming.replace_spaces {
        name = name.replace(' ', "_");
    }

    if naming.lowercase {
        name = name.to_lowercase();
    }

    if naming.max_length > 0 && name.len() > naming.max_length {
        // Preserve extension when truncating
        if let Some(dot_pos) = name.rfind('.') {
            let ext = &name[dot_pos..];
            let stem_max = naming.max_length.saturating_sub(ext.len());
            let stem = &name[..stem_max.min(dot_pos)];
            name = format!("{stem}{ext}");
        } else {
            name.truncate(naming.max_length);
        }
    }

    name
}

/// Resolve a collision by appending a numeric suffix.
fn resolve_with_suffix(path: &Path) -> PathBuf {
    let parent = path.parent().unwrap_or(Path::new("."));
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let ext = path
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();

    // Try _1, _2, _3, ...
    for i in 1..=1000 {
        let candidate = parent.join(format!("{stem}_{i}{ext}"));
        if !candidate.exists() {
            return candidate;
        }
    }

    // Fallback: use timestamp
    let ts = chrono::Utc::now().timestamp();
    parent.join(format!("{stem}_{ts}{ext}"))
}

impl NamedInventory {
    /// Count of operations that actually need a rename/move.
    pub fn pending_renames(&self) -> usize {
        self.operations.iter().filter(|op| op.needs_rename).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::NamingConfig;
    use std::path::Path;

    #[test]
    fn test_apply_naming_spaces() {
        let naming = NamingConfig {
            replace_spaces: true,
            ..NamingConfig::default()
        };
        let result = apply_naming_rules(Path::new("/tmp/my file.rs"), &naming);
        assert_eq!(result, "my_file.rs");
    }

    #[test]
    fn test_apply_naming_lowercase() {
        let naming = NamingConfig {
            lowercase: true,
            ..NamingConfig::default()
        };
        let result = apply_naming_rules(Path::new("/tmp/MyFile.RS"), &naming);
        assert_eq!(result, "myfile.rs");
    }

    #[test]
    fn test_apply_naming_truncate() {
        let naming = NamingConfig {
            max_length: 10,
            ..NamingConfig::default()
        };
        let result = apply_naming_rules(Path::new("/tmp/very_long_filename.rs"), &naming);
        assert!(result.len() <= 10);
        assert!(result.ends_with(".rs"));
    }

    #[test]
    fn test_resolve_with_suffix() {
        let path = Path::new("/tmp/nonexistent_test_file.rs");
        let resolved = resolve_with_suffix(path);
        assert!(resolved.to_string_lossy().contains("_1"));
    }

    #[test]
    fn test_naming_noop_for_keep() {
        let naming = NamingConfig::default();
        let result = apply_naming_rules(Path::new("/tmp/already_good.rs"), &naming);
        assert_eq!(result, "already_good.rs");
    }
}
