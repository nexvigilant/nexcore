//! Artifact version diffing.
//!
//! Uses the `similar` crate for line-by-line comparison of
//! artifact versions (snapshots).

use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};

use crate::types::SnapshotDiff;

/// Statistics about a diff.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiffStats {
    /// Number of lines added
    pub additions: usize,
    /// Number of lines deleted
    pub deletions: usize,
    /// Number of lines unchanged
    pub unchanged: usize,
    /// Total lines in old version
    pub old_lines: usize,
    /// Total lines in new version
    pub new_lines: usize,
}

impl DiffStats {
    /// Calculate the change ratio (0.0 = no change, 1.0 = complete rewrite).
    #[must_use]
    pub fn change_ratio(&self) -> f64 {
        let total_changes = self.additions + self.deletions;
        let max_lines = self.old_lines.max(self.new_lines);
        if max_lines == 0 {
            return 0.0;
        }
        total_changes as f64 / max_lines as f64
    }

    /// Check if there are any changes.
    #[must_use]
    pub fn has_changes(&self) -> bool {
        self.additions > 0 || self.deletions > 0
    }
}

/// Generate a unified diff between two versions.
///
/// Uses `similar::TextDiff` for line-by-line comparison.
///
/// # Arguments
///
/// * `old` - The old version content
/// * `new` - The new version content
///
/// # Returns
///
/// A `SnapshotDiff` containing the diff text and statistics.
#[must_use]
pub fn diff_versions(old: &str, new: &str) -> SnapshotDiff {
    let diff = TextDiff::from_lines(old, new);
    let mut additions = 0;
    let mut deletions = 0;
    let mut diff_lines = Vec::new();

    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => {
                deletions += 1;
                "-"
            }
            ChangeTag::Insert => {
                additions += 1;
                "+"
            }
            ChangeTag::Equal => " ",
        };
        diff_lines.push(format!("{sign}{}", change.value().trim_end_matches('\n')));
    }

    SnapshotDiff {
        name: String::new(), // Caller should set this
        from_version: 0,     // Caller should set this
        to_version: 0,       // Caller should set this
        additions,
        deletions,
        diff_text: diff_lines.join("\n"),
    }
}

/// Generate a unified diff with version metadata.
///
/// # Arguments
///
/// * `name` - Name of the artifact
/// * `from_version` - Version number of old content
/// * `to_version` - Version number of new content
/// * `old` - The old version content
/// * `new` - The new version content
///
/// # Returns
///
/// A `SnapshotDiff` with full metadata.
#[must_use]
pub fn diff_versions_with_metadata(
    name: &str,
    from_version: u32,
    to_version: u32,
    old: &str,
    new: &str,
) -> SnapshotDiff {
    let mut diff = diff_versions(old, new);
    diff.name = name.to_string();
    diff.from_version = from_version;
    diff.to_version = to_version;
    diff
}

/// Calculate diff statistics without generating full diff text.
///
/// More efficient when you only need counts.
#[must_use]
pub fn diff_stats(old: &str, new: &str) -> DiffStats {
    let diff = TextDiff::from_lines(old, new);
    let mut stats = DiffStats {
        old_lines: old.lines().count(),
        new_lines: new.lines().count(),
        ..Default::default()
    };

    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Delete => stats.deletions += 1,
            ChangeTag::Insert => stats.additions += 1,
            ChangeTag::Equal => stats.unchanged += 1,
        }
    }

    stats
}

/// Generate a context diff (showing only changed regions with context).
///
/// # Arguments
///
/// * `old` - The old version content
/// * `new` - The new version content
/// * `context_lines` - Number of context lines around changes
///
/// # Returns
///
/// Context diff as a string.
#[must_use]
pub fn context_diff(old: &str, new: &str, context_lines: usize) -> String {
    let diff = TextDiff::from_lines(old, new);
    diff.unified_diff()
        .context_radius(context_lines)
        .to_string()
}

/// Check if two versions are identical.
#[must_use]
pub fn versions_identical(old: &str, new: &str) -> bool {
    old == new
}

/// Summarize changes in a human-readable format.
#[must_use]
pub fn change_summary(old: &str, new: &str) -> String {
    let stats = diff_stats(old, new);
    if !stats.has_changes() {
        return "No changes".to_string();
    }

    let mut parts = Vec::new();
    if stats.additions > 0 {
        parts.push(format!("+{} lines", stats.additions));
    }
    if stats.deletions > 0 {
        parts.push(format!("-{} lines", stats.deletions));
    }

    let ratio = stats.change_ratio();
    let severity = if ratio < 0.1 {
        "minor"
    } else if ratio < 0.3 {
        "moderate"
    } else if ratio < 0.6 {
        "significant"
    } else {
        "major"
    };

    format!("{} ({} changes)", parts.join(", "), severity)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_identical() {
        let content = "line 1\nline 2\nline 3\n";
        let diff = diff_versions(content, content);
        assert_eq!(diff.additions, 0);
        assert_eq!(diff.deletions, 0);
    }

    #[test]
    fn test_diff_addition() {
        let old = "line 1\nline 2\n";
        let new = "line 1\nline 2\nline 3\n";
        let diff = diff_versions(old, new);
        assert_eq!(diff.additions, 1);
        assert_eq!(diff.deletions, 0);
    }

    #[test]
    fn test_diff_deletion() {
        let old = "line 1\nline 2\nline 3\n";
        let new = "line 1\nline 2\n";
        let diff = diff_versions(old, new);
        assert_eq!(diff.additions, 0);
        assert_eq!(diff.deletions, 1);
    }

    #[test]
    fn test_diff_modification() {
        let old = "line 1\nold line\nline 3\n";
        let new = "line 1\nnew line\nline 3\n";
        let diff = diff_versions(old, new);
        // Modification shows as deletion + addition
        assert_eq!(diff.additions, 1);
        assert_eq!(diff.deletions, 1);
    }

    #[test]
    fn test_diff_stats() {
        let old = "a\nb\nc\n";
        let new = "a\nB\nc\nd\n";
        let stats = diff_stats(old, new);
        assert_eq!(stats.additions, 2); // B and d
        assert_eq!(stats.deletions, 1); // b
        assert!(stats.has_changes());
    }

    #[test]
    fn test_change_ratio() {
        let stats = DiffStats {
            additions: 5,
            deletions: 5,
            unchanged: 90,
            old_lines: 100,
            new_lines: 100,
        };
        assert!((stats.change_ratio() - 0.1).abs() < 0.001);
    }

    #[test]
    fn test_versions_identical() {
        assert!(versions_identical("foo", "foo"));
        assert!(!versions_identical("foo", "bar"));
    }

    #[test]
    fn test_change_summary() {
        let old =
            "line 1\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7\nline 8\nline 9\nline 10\n";
        let new = "line 1\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7\nline 8\nline 9\nline 10\nline 11\n";
        let summary = change_summary(old, new);
        assert!(summary.contains("+1 lines"));
        // 1 addition out of 11 lines = ~9% change = minor
        assert!(summary.contains("minor"));
    }

    #[test]
    fn test_with_metadata() {
        let diff = diff_versions_with_metadata("task.md", 1, 2, "old content", "new content");
        assert_eq!(diff.name, "task.md");
        assert_eq!(diff.from_version, 1);
        assert_eq!(diff.to_version, 2);
    }
}
