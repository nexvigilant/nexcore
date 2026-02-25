//! Step 3: **G**roup — Cluster entries into categories.
//!
//! Primitive: μ Mapping — "which category does this entry map to?"
//!
//! Applies group rules from config to classify each ranked entry
//! into named groups. Entries matching no rule go to "ungrouped".

use std::collections::HashMap;
use std::path::PathBuf;

use crate::config::{GroupRule, OrganizeConfig};
use crate::error::OrganizeResult;
use crate::rank::{RankedEntry, RankedInventory};

// ============================================================================
// Types
// ============================================================================

/// A named group containing matched entries.
///
/// Tier: T2-P (μ Mapping — one mapping target)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Group {
    /// Group name (from config key).
    pub name: String,
    /// Entries that matched this group's rules.
    pub entries: Vec<RankedEntry>,
    /// Number of entries in this group.
    pub count: usize,
    /// Total size of entries in this group.
    pub total_bytes: u64,
}

/// Inventory after grouping.
///
/// Tier: T2-C (μ Mapping + κ Comparison — grouped and ranked)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GroupedInventory {
    /// Root directory.
    pub root: PathBuf,
    /// Named groups with their entries.
    pub groups: HashMap<String, Group>,
    /// Observation timestamp (carried forward).
    pub observed_at: nexcore_chrono::DateTime,
}

// ============================================================================
// Group Function
// ============================================================================

/// Group all ranked entries by config rules.
pub fn group(ranked: RankedInventory, config: &OrganizeConfig) -> OrganizeResult<GroupedInventory> {
    let mut groups: HashMap<String, Vec<RankedEntry>> = HashMap::new();

    for entry in ranked.entries {
        let group_name = find_matching_group(&entry, &config.groups);
        groups.entry(group_name).or_default().push(entry);
    }

    let groups = groups
        .into_iter()
        .map(|(name, entries)| {
            let count = entries.len();
            let total_bytes: u64 = entries.iter().map(|e| e.meta.size_bytes).sum();
            let group = Group {
                name: name.clone(),
                entries,
                count,
                total_bytes,
            };
            (name, group)
        })
        .collect();

    Ok(GroupedInventory {
        root: ranked.root,
        groups,
        observed_at: ranked.observed_at,
    })
}

// ============================================================================
// Matching
// ============================================================================

/// Find the first group whose rule matches the entry.
fn find_matching_group(entry: &RankedEntry, rules: &HashMap<String, GroupRule>) -> String {
    for (name, rule) in rules {
        if matches_rule(entry, rule) {
            return name.clone();
        }
    }
    "ungrouped".to_string()
}

/// Check if an entry matches a group rule.
fn matches_rule(entry: &RankedEntry, rule: &GroupRule) -> bool {
    let meta = &entry.meta;

    // Extension match
    if !rule.extensions.is_empty()
        && rule
            .extensions
            .iter()
            .any(|ext| ext.eq_ignore_ascii_case(&meta.extension))
    {
        return true;
    }

    // Pattern match (simple glob: * matches anything)
    if !rule.patterns.is_empty() && rule.patterns.iter().any(|p| matches_glob(p, &meta.name)) {
        return true;
    }

    // Size constraints
    if let Some(min) = rule.min_size {
        if meta.size_bytes >= min {
            return true;
        }
    }

    false
}

/// Minimal glob matching: supports * as wildcard.
fn matches_glob(pattern: &str, name: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    if let Some(prefix) = pattern.strip_suffix('*') {
        return name.starts_with(prefix);
    }

    if let Some(suffix) = pattern.strip_prefix('*') {
        return name.ends_with(suffix);
    }

    pattern == name
}

impl GroupedInventory {
    /// Total number of entries across all groups.
    pub fn total_entries(&self) -> usize {
        self.groups.values().map(|g| g.count).sum()
    }

    /// Get a flat list of all entries across all groups.
    pub fn all_entries(&self) -> Vec<&RankedEntry> {
        self.groups
            .values()
            .flat_map(|g| g.entries.iter())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observe::EntryMeta;
    use crate::rank::{RankedEntry, ScoreBreakdown};
    use nexcore_chrono::DateTime;

    fn make_ranked(name: &str, ext: &str) -> RankedEntry {
        RankedEntry {
            meta: EntryMeta {
                path: PathBuf::from(format!("/tmp/{name}.{ext}")),
                is_dir: false,
                size_bytes: 100,
                modified: DateTime::now(),
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
    fn test_matches_glob_exact() {
        assert!(matches_glob("Cargo.toml", "Cargo.toml"));
        assert!(!matches_glob("Cargo.toml", "cargo.toml"));
    }

    #[test]
    fn test_matches_glob_prefix() {
        assert!(matches_glob("Cargo.*", "Cargo.toml"));
        assert!(matches_glob("Cargo.*", "Cargo.lock"));
        assert!(!matches_glob("Cargo.*", "package.json"));
    }

    #[test]
    fn test_matches_glob_suffix() {
        assert!(matches_glob("*.rs", "main.rs"));
        assert!(!matches_glob("*.rs", "main.py"));
    }

    #[test]
    fn test_matches_glob_star() {
        assert!(matches_glob("*", "anything"));
    }

    #[test]
    fn test_group_by_extension() {
        let ranked = RankedInventory {
            root: PathBuf::from("/tmp"),
            entries: vec![
                make_ranked("lib", "rs"),
                make_ranked("readme", "md"),
                make_ranked("photo", "png"),
            ],
            observed_at: DateTime::now(),
        };

        let config = OrganizeConfig::default_for("/tmp");
        let grouped = group(ranked, &config);
        assert!(grouped.is_ok());
        if let Ok(grouped) = grouped {
            assert_eq!(grouped.total_entries(), 3);
        }
    }

    #[test]
    fn test_ungrouped_fallback() {
        let ranked = RankedInventory {
            root: PathBuf::from("/tmp"),
            entries: vec![make_ranked("mystery", "xyz")],
            observed_at: DateTime::now(),
        };

        let config = OrganizeConfig::default_for("/tmp");
        let grouped = group(ranked, &config);
        assert!(grouped.is_ok());
        if let Ok(grouped) = grouped {
            assert!(grouped.groups.contains_key("ungrouped"));
        }
    }
}
