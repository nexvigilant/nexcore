//! Cross-reference analysis with governance modules.
//!
//! Identifies file accesses to governance-related paths such as
//! primitives, governance modules, and capability definitions.

use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};

use crate::types::{ActivityType, Source};

/// Paths that indicate governance-related file access.
const GOVERNANCE_PATH_PATTERNS: &[&str] = &[
    "governance",
    "primitives",
    "hud/capabilities",
    "capabilities",
    "constitution",
    "codex",
];

/// Classification of governance module access.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GovernanceCategory {
    /// Core primitives (T1/T2/T3 types)
    Primitives,
    /// Governance rules and policies
    Governance,
    /// HUD capabilities and permissions
    Capabilities,
    /// Constitutional/foundational documents
    Constitutional,
    /// Unknown governance-related access
    Unknown,
}

impl GovernanceCategory {
    /// Categorize a path into a governance category.
    #[must_use]
    pub fn from_path(path: &str) -> Self {
        let lower = path.to_lowercase();
        if lower.contains("primitives") {
            Self::Primitives
        } else if lower.contains("governance") {
            Self::Governance
        } else if lower.contains("capabilities") || lower.contains("hud") {
            Self::Capabilities
        } else if lower.contains("constitution") || lower.contains("codex") {
            Self::Constitutional
        } else {
            Self::Unknown
        }
    }
}

/// Record of governance file access.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceAccess {
    /// Path to the governance file
    pub path: String,
    /// Category of governance module
    pub category: GovernanceCategory,
    /// Whether this was a write operation
    pub was_modified: bool,
    /// Number of read operations
    pub read_count: u32,
    /// Number of write operations
    pub write_count: u32,
    /// Session ID where access occurred
    pub session_id: String,
    /// Timestamp of last access
    pub last_accessed: DateTime,
}

impl GovernanceAccess {
    /// Create a new governance access record.
    #[must_use]
    pub fn new(path: String, session_id: String) -> Self {
        Self {
            category: GovernanceCategory::from_path(&path),
            path,
            was_modified: false,
            read_count: 0,
            write_count: 0,
            session_id,
            last_accessed: DateTime::now(),
        }
    }

    /// Total number of accesses.
    #[must_use]
    pub fn total_accesses(&self) -> u32 {
        self.read_count + self.write_count
    }
}

/// Check if a path is governance-related.
#[must_use]
pub fn is_governance_path(path: &str) -> bool {
    let lower = path.to_lowercase();
    GOVERNANCE_PATH_PATTERNS
        .iter()
        .any(|pattern| lower.contains(pattern))
}

/// Analyze sources for governance file accesses.
///
/// Filters for files in paths containing governance-related patterns
/// and tracks what modules are being accessed or modified.
///
/// # Arguments
///
/// * `sources` - Slice of telemetry sources to analyze
///
/// # Returns
///
/// Vector of governance access records, sorted by modification status then access count.
#[must_use]
pub fn governance_file_access(sources: &[Source]) -> Vec<GovernanceAccess> {
    use std::collections::HashMap;

    let mut accesses: HashMap<String, GovernanceAccess> = HashMap::new();

    for source in sources {
        for operation in source.all_operations() {
            process_governance_operation(&mut accesses, &operation, &source.id);
        }
    }

    // Collect and sort: modified first, then by total accesses
    let mut result: Vec<GovernanceAccess> = accesses.into_values().collect();
    result.sort_by(|a, b| {
        // Modified files first
        match (a.was_modified, b.was_modified) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => b.total_accesses().cmp(&a.total_accesses()),
        }
    });
    result
}

/// Helper to process a single governance-related operation.
fn process_governance_operation(
    accesses: &mut std::collections::HashMap<String, GovernanceAccess>,
    operation: &crate::types::Operation,
    source_id: &str,
) {
    // Only process file-related operations
    if !operation.is_file_operation() {
        return;
    }

    // Extract file path
    let Some(path) = operation.file_path() else {
        return;
    };

    // Check if this is a governance-related path
    if !is_governance_path(&path) {
        return;
    }

    let access = accesses
        .entry(path.clone())
        .or_insert_with(|| GovernanceAccess::new(path, source_id.to_string()));

    // Update access counts based on activity type
    match operation.activity_type() {
        ActivityType::FileRead | ActivityType::DirectoryList | ActivityType::ContentSearch => {
            access.read_count += 1;
        }
        ActivityType::FileWrite => {
            access.write_count += 1;
            access.was_modified = true;
        }
        ActivityType::ShellCommand | ActivityType::Unknown(_) => {}
    }

    // Update last accessed timestamp
    if operation.timestamp > access.last_accessed {
        access.last_accessed = operation.timestamp;
    }
}

/// Filter governance accesses to only modified files.
#[must_use]
pub fn filter_modified_governance(accesses: &[GovernanceAccess]) -> Vec<&GovernanceAccess> {
    accesses.iter().filter(|a| a.was_modified).collect()
}

/// Filter governance accesses by category.
#[must_use]
pub fn filter_by_category(
    accesses: &[GovernanceAccess],
    category: GovernanceCategory,
) -> Vec<&GovernanceAccess> {
    accesses.iter().filter(|a| a.category == category).collect()
}

/// Summary of governance access across all categories.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GovernanceSummary {
    /// Total governance files accessed
    pub total_files: usize,
    /// Total governance files modified
    pub files_modified: usize,
    /// Primitives module access count
    pub primitives_accesses: u32,
    /// Governance module access count
    pub governance_accesses: u32,
    /// Capabilities module access count
    pub capabilities_accesses: u32,
    /// Constitutional document access count
    pub constitutional_accesses: u32,
}

/// Generate a summary of governance accesses.
#[must_use]
pub fn governance_summary(accesses: &[GovernanceAccess]) -> GovernanceSummary {
    let mut summary = GovernanceSummary {
        total_files: accesses.len(),
        files_modified: accesses.iter().filter(|a| a.was_modified).count(),
        ..Default::default()
    };

    for access in accesses {
        let count = access.total_accesses();
        match access.category {
            GovernanceCategory::Primitives => summary.primitives_accesses += count,
            GovernanceCategory::Governance => summary.governance_accesses += count,
            GovernanceCategory::Capabilities => summary.capabilities_accesses += count,
            GovernanceCategory::Constitutional => summary.constitutional_accesses += count,
            GovernanceCategory::Unknown => {}
        }
    }

    summary
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{EntryType, Operation, OperationStatus, Source, SourceEntry, TokenUsage};
    use std::collections::HashMap;

    fn make_source_with_ops(id: &str, operations: Vec<Operation>) -> Source {
        Source {
            id: id.to_string(),
            project_hash: "test-hash".to_string(),
            start_time: DateTime::now(),
            last_updated: DateTime::now(),
            messages: vec![SourceEntry {
                id: "entry-1".to_string(),
                timestamp: DateTime::now(),
                entry_type: EntryType::Assistant,
                content: String::new(),
                thoughts: vec![],
                tokens: Some(TokenUsage::default()),
                model: None,
                operations,
            }],
        }
    }

    fn make_op(name: &str, path: &str) -> Operation {
        let mut args = HashMap::new();
        args.insert("file_path".to_string(), serde_json::json!(path));
        Operation {
            id: "op-1".to_string(),
            name: name.to_string(),
            args,
            result: vec![],
            status: OperationStatus::Success,
            timestamp: DateTime::now(),
            result_display: None,
            display_name: None,
            description: None,
        }
    }

    #[test]
    fn test_is_governance_path() {
        assert!(is_governance_path("/src/primitives/governance/mod.rs"));
        assert!(is_governance_path("/hud/capabilities/permissions.rs"));
        assert!(is_governance_path("~/nexcore/crates/governance/rules.rs"));
        assert!(!is_governance_path("/src/main.rs"));
        assert!(!is_governance_path("/test/utils.rs"));
    }

    #[test]
    fn test_governance_category() {
        assert_eq!(
            GovernanceCategory::from_path("/src/primitives/t1.rs"),
            GovernanceCategory::Primitives
        );
        assert_eq!(
            GovernanceCategory::from_path("/src/governance/rules.rs"),
            GovernanceCategory::Governance
        );
        assert_eq!(
            GovernanceCategory::from_path("/hud/capabilities/list.rs"),
            GovernanceCategory::Capabilities
        );
    }

    #[test]
    fn test_governance_file_access() {
        let source = make_source_with_ops(
            "session-1",
            vec![
                make_op("read_file", "/src/primitives/t1.rs"),
                make_op("write_file", "/src/governance/rules.rs"),
                make_op("read_file", "/src/main.rs"), // Not governance
            ],
        );

        let accesses = governance_file_access(&[source]);

        assert_eq!(accesses.len(), 2);
        // Modified file should be first
        assert!(accesses[0].was_modified);
        assert_eq!(accesses[0].path, "/src/governance/rules.rs");
    }

    #[test]
    fn test_governance_summary() {
        let source = make_source_with_ops(
            "session-1",
            vec![
                make_op("read_file", "/src/primitives/t1.rs"),
                make_op("read_file", "/src/primitives/t2.rs"),
                make_op("write_file", "/src/governance/rules.rs"),
            ],
        );

        let accesses = governance_file_access(&[source]);
        let summary = governance_summary(&accesses);

        assert_eq!(summary.total_files, 3);
        assert_eq!(summary.files_modified, 1);
        assert_eq!(summary.primitives_accesses, 2);
        assert_eq!(summary.governance_accesses, 1);
    }
}
