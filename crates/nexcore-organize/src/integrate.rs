//! Step 6: **I**ntegrate — Execute the plan (or dry-run report).
//!
//! Primitive: Σ Sum — "fold all operations into an execution result"
//!
//! Iterates through named operations and either executes them (live mode)
//! or accumulates them as a dry-run report.

use std::path::{Path, PathBuf};

use crate::config::FileOp;
use crate::error::{OrganizeError, OrganizeResult};
use crate::name::NamedInventory;

// ============================================================================
// Types
// ============================================================================

/// Result of executing (or simulating) a single operation.
///
/// Tier: T2-P (Σ Sum — one fold step in the execution)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutedOp {
    /// Source path.
    pub source: PathBuf,
    /// Target path.
    pub target: PathBuf,
    /// Action taken.
    pub action: FileOp,
    /// Whether this was a dry-run (simulated).
    pub dry_run: bool,
    /// Whether the operation succeeded.
    pub success: bool,
    /// Error message if failed.
    pub error: Option<String>,
    /// Bytes affected.
    pub bytes: u64,
}

/// The integration plan / execution result.
///
/// Tier: T2-C (Σ Sum + → Causality — aggregated causal results)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IntegrationPlan {
    /// Root directory.
    pub root: PathBuf,
    /// All executed operations.
    pub operations: Vec<ExecutedOp>,
    /// Whether this was a dry run.
    pub dry_run: bool,
    /// Total operations attempted.
    pub total: usize,
    /// Successful operations.
    pub succeeded: usize,
    /// Failed operations.
    pub failed: usize,
    /// Total bytes moved/deleted/archived.
    pub bytes_affected: u64,
}

// ============================================================================
// Integrate Function
// ============================================================================

/// Execute (or simulate) all named operations.
pub fn integrate(named: NamedInventory, dry_run: bool) -> OrganizeResult<IntegrationPlan> {
    let mut operations = Vec::new();
    let mut succeeded: usize = 0;
    let mut failed: usize = 0;
    let mut bytes_affected: u64 = 0;

    for op in &named.operations {
        let action = &op.assignment.action;
        let source = &op.assignment.entry.meta.path;
        let target = &op.target_path;
        let bytes = op.assignment.entry.meta.size_bytes;

        let (success, error) = if dry_run {
            // Dry run: just report what would happen
            (true, None)
        } else {
            execute_single(action, source, target)
        };

        if success {
            succeeded += 1;
            bytes_affected = bytes_affected.saturating_add(bytes);
        } else {
            failed += 1;
        }

        operations.push(ExecutedOp {
            source: source.clone(),
            target: target.clone(),
            action: action.clone(),
            dry_run,
            success,
            error,
            bytes,
        });
    }

    let total = operations.len();

    Ok(IntegrationPlan {
        root: named.root,
        operations,
        dry_run,
        total,
        succeeded,
        failed,
        bytes_affected,
    })
}

// ============================================================================
// Execution Helpers
// ============================================================================

/// Execute a single filesystem operation.
fn execute_single(action: &FileOp, source: &Path, target: &Path) -> (bool, Option<String>) {
    match action {
        FileOp::Move => execute_move(source, target),
        FileOp::Archive => execute_move(source, target), // simplified: move to archive dir
        FileOp::Delete => execute_delete(source),
        FileOp::Keep => (true, None),
        FileOp::Review => (true, None), // Review is a no-op for execution
    }
}

/// Move a file from source to target.
fn execute_move(source: &Path, target: &Path) -> (bool, Option<String>) {
    // Ensure target parent directory exists
    if let Some(parent) = target.parent() {
        if !parent.exists() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                return (false, Some(format!("failed to create directory: {e}")));
            }
        }
    }

    match std::fs::rename(source, target) {
        Ok(()) => (true, None),
        Err(e) => {
            // rename fails across filesystems, try copy+delete
            match std::fs::copy(source, target) {
                Ok(_) => match std::fs::remove_file(source) {
                    Ok(()) => (true, None),
                    Err(e2) => (
                        false,
                        Some(format!("copied but failed to remove source: {e2}")),
                    ),
                },
                Err(e2) => (
                    false,
                    Some(format!("rename failed: {e}, copy failed: {e2}")),
                ),
            }
        }
    }
}

/// Delete a file or directory.
fn execute_delete(source: &Path) -> (bool, Option<String>) {
    let result = if source.is_dir() {
        std::fs::remove_dir_all(source)
    } else {
        std::fs::remove_file(source)
    };

    match result {
        Ok(()) => (true, None),
        Err(e) => (false, Some(format!("delete failed: {e}"))),
    }
}

impl IntegrationPlan {
    /// Summary string for display.
    pub fn summary(&self) -> String {
        let mode = if self.dry_run { "DRY RUN" } else { "LIVE" };
        format!(
            "[{mode}] {total} ops: {ok} ok, {fail} failed, {bytes} bytes affected",
            total = self.total,
            ok = self.succeeded,
            fail = self.failed,
            bytes = self.bytes_affected,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assign::Assignment;
    use crate::config::FileOp;
    use crate::name::{NamedInventory, RenameOp};
    use crate::observe::EntryMeta;
    use crate::rank::{RankedEntry, ScoreBreakdown};
    use nexcore_chrono::DateTime;

    fn make_named_inventory(actions: Vec<FileOp>) -> NamedInventory {
        let operations = actions
            .into_iter()
            .enumerate()
            .map(|(i, action)| {
                let path = PathBuf::from(format!("/tmp/file_{i}.txt"));
                RenameOp {
                    assignment: Assignment {
                        entry: RankedEntry {
                            meta: EntryMeta {
                                path: path.clone(),
                                is_dir: false,
                                size_bytes: 100,
                                modified: DateTime::now(),
                                extension: "txt".to_string(),
                                depth: 1,
                                name: format!("file_{i}.txt"),
                            },
                            score: ScoreBreakdown {
                                recency: 1.0,
                                size: 0.5,
                                relevance: 0.5,
                                depth: 0.9,
                                composite: 0.7,
                            },
                        },
                        action,
                        group: "test".to_string(),
                        target_dir: None,
                    },
                    target_path: path,
                    needs_rename: false,
                }
            })
            .collect();

        NamedInventory {
            root: PathBuf::from("/tmp"),
            operations,
            collisions_resolved: 0,
            observed_at: DateTime::now(),
        }
    }

    #[test]
    fn test_integrate_dry_run() {
        let named = make_named_inventory(vec![FileOp::Keep, FileOp::Delete, FileOp::Move]);
        let plan = integrate(named, true);
        assert!(plan.is_ok());
        if let Ok(plan) = plan {
            assert!(plan.dry_run);
            assert_eq!(plan.total, 3);
            assert_eq!(plan.succeeded, 3); // All succeed in dry run
            assert_eq!(plan.failed, 0);
        }
    }

    #[test]
    fn test_integrate_summary_dry_run() {
        let named = make_named_inventory(vec![FileOp::Keep]);
        let plan = integrate(named, true);
        assert!(plan.is_ok());
        if let Ok(plan) = plan {
            let summary = plan.summary();
            assert!(summary.contains("DRY RUN"));
        }
    }

    #[test]
    fn test_integrate_summary_live() {
        let named = make_named_inventory(vec![FileOp::Keep]);
        let plan = integrate(named, false);
        assert!(plan.is_ok());
        if let Ok(plan) = plan {
            let summary = plan.summary();
            assert!(summary.contains("LIVE"));
        }
    }
}
