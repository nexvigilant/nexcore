//! Step 4: **A**ssign — Map groups to actions.
//!
//! Primitive: → Causality — "what action does this group cause?"
//!
//! Each group gets its action from the config rule, or the default action.
//! Produces an `AssignedInventory` where every entry has a concrete action.

use std::collections::HashMap;
use std::path::PathBuf;

use crate::config::{FileOp, OrganizeConfig};
use crate::error::OrganizeResult;
use crate::group::GroupedInventory;
use crate::rank::RankedEntry;

// ============================================================================
// Types
// ============================================================================

/// An entry with its assigned action.
///
/// Tier: T2-C (→ Causality + κ Comparison — causal consequence of ranking)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Assignment {
    /// The ranked entry.
    pub entry: RankedEntry,
    /// Assigned action.
    pub action: FileOp,
    /// Source group name.
    pub group: String,
    /// Target directory for Move/Archive (if applicable).
    pub target_dir: Option<PathBuf>,
}

/// Inventory after action assignment.
///
/// Tier: T2-C (→ Causality + μ Mapping — every entry mapped to an action)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AssignedInventory {
    /// Root directory.
    pub root: PathBuf,
    /// All assignments.
    pub assignments: Vec<Assignment>,
    /// Observation timestamp (carried forward).
    pub observed_at: chrono::DateTime<chrono::Utc>,
}

// ============================================================================
// Assign Function
// ============================================================================

/// Assign actions to all grouped entries.
pub fn assign(
    grouped: GroupedInventory,
    config: &OrganizeConfig,
) -> OrganizeResult<AssignedInventory> {
    let mut assignments = Vec::new();

    for (group_name, group) in &grouped.groups {
        let (action, target_dir) = resolve_action(group_name, config);

        for entry in &group.entries {
            assignments.push(Assignment {
                entry: entry.clone(),
                action: action.clone(),
                group: group_name.clone(),
                target_dir: target_dir.clone(),
            });
        }
    }

    Ok(AssignedInventory {
        root: grouped.root,
        assignments,
        observed_at: grouped.observed_at,
    })
}

/// Resolve the action and target directory for a group.
fn resolve_action(group_name: &str, config: &OrganizeConfig) -> (FileOp, Option<PathBuf>) {
    if let Some(rule) = config.groups.get(group_name) {
        let target = rule.target_dir.as_ref().map(|t| config.root.join(t));
        (rule.action.clone(), target)
    } else {
        (config.default_action.clone(), None)
    }
}

impl AssignedInventory {
    /// Count assignments by action type.
    pub fn count_by_action(&self) -> HashMap<String, usize> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for a in &self.assignments {
            *counts.entry(a.action.to_string()).or_insert(0) += 1;
        }
        counts
    }

    /// Filter assignments to only those with a specific action.
    pub fn with_action(&self, action: &FileOp) -> Vec<&Assignment> {
        self.assignments
            .iter()
            .filter(|a| &a.action == action)
            .collect()
    }

    /// Total bytes scheduled for deletion.
    pub fn bytes_to_delete(&self) -> u64 {
        self.assignments
            .iter()
            .filter(|a| a.action == FileOp::Delete)
            .map(|a| a.entry.meta.size_bytes)
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::group::{Group, GroupedInventory};
    use crate::observe::EntryMeta;
    use crate::rank::{RankedEntry, ScoreBreakdown};
    use chrono::Utc;

    fn make_assignment_entry(name: &str, ext: &str) -> RankedEntry {
        RankedEntry {
            meta: EntryMeta {
                path: PathBuf::from(format!("/tmp/{name}.{ext}")),
                is_dir: false,
                size_bytes: 500,
                modified: Utc::now(),
                extension: ext.to_string(),
                depth: 1,
                name: format!("{name}.{ext}"),
            },
            score: ScoreBreakdown {
                recency: 1.0,
                size: 0.5,
                relevance: 0.8,
                depth: 0.9,
                composite: 0.8,
            },
        }
    }

    #[test]
    fn test_assign_uses_group_action() {
        let mut groups = HashMap::new();
        groups.insert(
            "rust".to_string(),
            Group {
                name: "rust".to_string(),
                entries: vec![make_assignment_entry("lib", "rs")],
                count: 1,
                total_bytes: 500,
            },
        );

        let grouped = GroupedInventory {
            root: PathBuf::from("/tmp"),
            groups,
            observed_at: Utc::now(),
        };

        let config = OrganizeConfig::default_for("/tmp");
        let assigned = assign(grouped, &config);
        assert!(assigned.is_ok());
        if let Ok(assigned) = assigned {
            assert_eq!(assigned.assignments.len(), 1);
            assert_eq!(assigned.assignments[0].action, FileOp::Keep);
        }
    }

    #[test]
    fn test_assign_ungrouped_gets_default() {
        let mut groups = HashMap::new();
        groups.insert(
            "ungrouped".to_string(),
            Group {
                name: "ungrouped".to_string(),
                entries: vec![make_assignment_entry("mystery", "xyz")],
                count: 1,
                total_bytes: 500,
            },
        );

        let grouped = GroupedInventory {
            root: PathBuf::from("/tmp"),
            groups,
            observed_at: Utc::now(),
        };

        let config = OrganizeConfig::default_for("/tmp");
        let assigned = assign(grouped, &config);
        assert!(assigned.is_ok());
        if let Ok(assigned) = assigned {
            // Default action is Keep
            assert_eq!(assigned.assignments[0].action, FileOp::Keep);
        }
    }

    #[test]
    fn test_count_by_action() {
        let assigned = AssignedInventory {
            root: PathBuf::from("/tmp"),
            assignments: vec![
                Assignment {
                    entry: make_assignment_entry("a", "rs"),
                    action: FileOp::Keep,
                    group: "rust".to_string(),
                    target_dir: None,
                },
                Assignment {
                    entry: make_assignment_entry("b", "rs"),
                    action: FileOp::Keep,
                    group: "rust".to_string(),
                    target_dir: None,
                },
                Assignment {
                    entry: make_assignment_entry("c", "log"),
                    action: FileOp::Delete,
                    group: "logs".to_string(),
                    target_dir: None,
                },
            ],
            observed_at: Utc::now(),
        };

        let counts = assigned.count_by_action();
        assert_eq!(counts.get("Keep"), Some(&2));
        assert_eq!(counts.get("Delete"), Some(&1));
    }
}
