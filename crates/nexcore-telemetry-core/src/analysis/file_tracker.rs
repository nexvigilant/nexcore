//! File access pattern tracking and analysis.
//!
//! Analyzes external source sessions to identify file access patterns,
//! distinguishing between read and write operations.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::{ActivityType, Source};

/// Tracks file access patterns across telemetry sources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAccessPattern {
    /// Path to the file being accessed
    pub path: String,
    /// Number of read operations on this file
    pub read_count: u32,
    /// Number of write/edit operations on this file
    pub write_count: u32,
    /// Timestamp of last access
    pub last_accessed: DateTime<Utc>,
}

impl FileAccessPattern {
    /// Create a new file access pattern for a path.
    #[must_use]
    pub fn new(path: String) -> Self {
        Self {
            path,
            read_count: 0,
            write_count: 0,
            last_accessed: Utc::now(),
        }
    }

    /// Total number of accesses (reads + writes).
    #[must_use]
    pub fn total_accesses(&self) -> u32 {
        self.read_count + self.write_count
    }

    /// Check if this file has been modified (has write operations).
    #[must_use]
    pub fn was_modified(&self) -> bool {
        self.write_count > 0
    }

    /// Calculate read/write ratio. Returns None if no accesses.
    #[must_use]
    pub fn read_write_ratio(&self) -> Option<f64> {
        if self.write_count == 0 {
            return None;
        }
        Some(f64::from(self.read_count) / f64::from(self.write_count))
    }
}

/// Analyze file access patterns from telemetry sources.
///
/// Groups operations by file path and counts read vs write operations.
///
/// # Arguments
///
/// * `sources` - Slice of telemetry sources to analyze
///
/// # Returns
///
/// Vector of file access patterns, sorted by total access count descending.
#[must_use]
pub fn analyze_file_access(sources: &[Source]) -> Vec<FileAccessPattern> {
    let mut patterns: HashMap<String, FileAccessPattern> = HashMap::new();

    for source in sources {
        for operation in source.all_operations() {
            process_operation(&mut patterns, &operation);
        }
    }

    // Collect and sort by total accesses descending
    let mut result: Vec<FileAccessPattern> = patterns.into_values().collect();
    result.sort_by(|a, b| b.total_accesses().cmp(&a.total_accesses()));
    result
}

/// Helper to process a single operation and update patterns.
fn process_operation(
    patterns: &mut HashMap<String, FileAccessPattern>,
    operation: &crate::types::Operation,
) {
    // Only process file-related operations
    if !operation.is_file_operation() {
        return;
    }

    // Extract file path from operation
    let Some(path) = operation.file_path() else {
        return;
    };

    let pattern = patterns
        .entry(path.clone())
        .or_insert_with(|| FileAccessPattern::new(path));

    // Update access counts based on activity type
    match operation.activity_type() {
        ActivityType::FileRead | ActivityType::DirectoryList | ActivityType::ContentSearch => {
            pattern.read_count += 1;
        }
        ActivityType::FileWrite => {
            pattern.write_count += 1;
        }
        ActivityType::ShellCommand | ActivityType::Unknown(_) => {
            // Shell commands might read/write but we can't tell
            // Unknown operations are ignored
        }
    }

    // Update last accessed timestamp
    if operation.timestamp > pattern.last_accessed {
        pattern.last_accessed = operation.timestamp;
    }
}

/// Filter file access patterns to only those above a threshold.
#[must_use]
pub fn filter_high_access(
    patterns: &[FileAccessPattern],
    min_accesses: u32,
) -> Vec<&FileAccessPattern> {
    patterns
        .iter()
        .filter(|p| p.total_accesses() >= min_accesses)
        .collect()
}

/// Filter file access patterns to only modified files.
#[must_use]
pub fn filter_modified(patterns: &[FileAccessPattern]) -> Vec<&FileAccessPattern> {
    patterns.iter().filter(|p| p.was_modified()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{EntryType, Operation, OperationStatus, SourceEntry, TokenUsage};
    use std::collections::HashMap;

    fn make_source_with_ops(operations: Vec<Operation>) -> Source {
        Source {
            id: "test-session".to_string(),
            project_hash: "test-hash".to_string(),
            start_time: Utc::now(),
            last_updated: Utc::now(),
            messages: vec![SourceEntry {
                id: "entry-1".to_string(),
                timestamp: Utc::now(),
                entry_type: EntryType::Assistant,
                content: String::new(),
                thoughts: vec![],
                tokens: Some(TokenUsage::default()),
                model: None,
                operations,
            }],
        }
    }

    fn make_read_op(path: &str) -> Operation {
        let mut args = HashMap::new();
        args.insert("file_path".to_string(), serde_json::json!(path));
        Operation {
            id: "op-1".to_string(),
            name: "read_file".to_string(),
            args,
            result: vec![],
            status: OperationStatus::Success,
            timestamp: Utc::now(),
            result_display: None,
            display_name: None,
            description: None,
        }
    }

    fn make_write_op(path: &str) -> Operation {
        let mut args = HashMap::new();
        args.insert("file_path".to_string(), serde_json::json!(path));
        Operation {
            id: "op-2".to_string(),
            name: "write_file".to_string(),
            args,
            result: vec![],
            status: OperationStatus::Success,
            timestamp: Utc::now(),
            result_display: None,
            display_name: None,
            description: None,
        }
    }

    #[test]
    fn test_analyze_file_access_empty() {
        let sources: Vec<Source> = vec![];
        let patterns = analyze_file_access(&sources);
        assert!(patterns.is_empty());
    }

    #[test]
    fn test_analyze_file_access_counts() {
        let source = make_source_with_ops(vec![
            make_read_op("/test/file.rs"),
            make_read_op("/test/file.rs"),
            make_write_op("/test/file.rs"),
        ]);

        let patterns = analyze_file_access(&[source]);
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].read_count, 2);
        assert_eq!(patterns[0].write_count, 1);
        assert_eq!(patterns[0].total_accesses(), 3);
    }

    #[test]
    fn test_filter_modified() {
        let source = make_source_with_ops(vec![
            make_read_op("/test/readonly.rs"),
            make_write_op("/test/modified.rs"),
        ]);

        let patterns = analyze_file_access(&[source]);
        let modified = filter_modified(&patterns);

        assert_eq!(modified.len(), 1);
        assert_eq!(modified[0].path, "/test/modified.rs");
    }
}
