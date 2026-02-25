//! Tool operation types.
//!
//! Represents tool calls made during telemetry sessions.

use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Activity classification for operations.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActivityType {
    /// File read operation
    FileRead,
    /// File write/edit operation
    FileWrite,
    /// Directory listing
    DirectoryList,
    /// Content search (grep/ripgrep)
    ContentSearch,
    /// Shell command execution
    ShellCommand,
    /// Unknown operation type
    Unknown(String),
}

impl ActivityType {
    /// Classify an operation by its name.
    #[must_use]
    pub fn from_name(name: &str) -> Self {
        match name {
            "read_file" | "ReadFile" => Self::FileRead,
            "write_file" | "WriteFile" | "edit_file" | "EditFile" => Self::FileWrite,
            "list_directory" | "ListDirectory" => Self::DirectoryList,
            "search_file_content" | "SearchText" | "Grep" => Self::ContentSearch,
            "run_shell_command" | "Shell" | "Bash" => Self::ShellCommand,
            other => Self::Unknown(other.to_string()),
        }
    }

    /// Check if this is a file-related operation.
    #[must_use]
    pub fn is_file_related(&self) -> bool {
        matches!(
            self,
            Self::FileRead | Self::FileWrite | Self::DirectoryList | Self::ContentSearch
        )
    }
}

/// A single tool operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub args: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub result: Vec<serde_json::Value>,
    pub status: OperationStatus,
    pub timestamp: DateTime,
    #[serde(default, rename = "resultDisplay")]
    pub result_display: Option<String>,
    #[serde(default, rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

/// Status of an operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OperationStatus {
    Success,
    Error,
    Pending,
    #[serde(other)]
    Unknown,
}

impl Operation {
    /// Get the activity type for this operation.
    #[must_use]
    pub fn activity_type(&self) -> ActivityType {
        ActivityType::from_name(&self.name)
    }

    /// Check if this is a file operation.
    #[must_use]
    pub fn is_file_operation(&self) -> bool {
        self.activity_type().is_file_related()
    }

    /// Extract file path from operation arguments.
    #[must_use]
    pub fn file_path(&self) -> Option<String> {
        self.args
            .get("file_path")
            .or_else(|| self.args.get("path"))
            .or_else(|| self.args.get("dir_path"))
            .and_then(|v| v.as_str())
            .map(String::from)
    }

    /// Extract search pattern if this is a search operation.
    #[must_use]
    pub fn search_pattern(&self) -> Option<String> {
        self.args
            .get("pattern")
            .or_else(|| self.args.get("query"))
            .and_then(|v| v.as_str())
            .map(String::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_chrono::DateTime;

    fn make_operation(name: &str, args: HashMap<String, serde_json::Value>) -> Operation {
        Operation {
            id: "op-001".to_string(),
            name: name.to_string(),
            args,
            result: Vec::new(),
            status: OperationStatus::Success,
            timestamp: DateTime::now(),
            result_display: None,
            display_name: None,
            description: None,
        }
    }

    #[test]
    fn activity_type_from_name_file_read() {
        assert_eq!(ActivityType::from_name("read_file"), ActivityType::FileRead);
        assert_eq!(ActivityType::from_name("ReadFile"), ActivityType::FileRead);
    }

    #[test]
    fn activity_type_from_name_file_write() {
        assert_eq!(
            ActivityType::from_name("write_file"),
            ActivityType::FileWrite
        );
        assert_eq!(ActivityType::from_name("EditFile"), ActivityType::FileWrite);
        assert_eq!(
            ActivityType::from_name("edit_file"),
            ActivityType::FileWrite
        );
    }

    #[test]
    fn activity_type_from_name_shell_and_search() {
        assert_eq!(ActivityType::from_name("Bash"), ActivityType::ShellCommand);
        assert_eq!(ActivityType::from_name("Grep"), ActivityType::ContentSearch);
        assert_eq!(
            ActivityType::from_name("SearchText"),
            ActivityType::ContentSearch
        );
    }

    #[test]
    fn activity_type_unknown_for_unrecognized() {
        let at = ActivityType::from_name("custom_tool");
        assert_eq!(at, ActivityType::Unknown("custom_tool".to_string()));
    }

    #[test]
    fn is_file_related_true_for_file_ops() {
        assert!(ActivityType::FileRead.is_file_related());
        assert!(ActivityType::FileWrite.is_file_related());
        assert!(ActivityType::DirectoryList.is_file_related());
        assert!(ActivityType::ContentSearch.is_file_related());
    }

    #[test]
    fn is_file_related_false_for_non_file_ops() {
        assert!(!ActivityType::ShellCommand.is_file_related());
        assert!(!ActivityType::Unknown("foo".to_string()).is_file_related());
    }

    #[test]
    fn operation_activity_type_delegates_correctly() {
        let op = make_operation("read_file", HashMap::new());
        assert_eq!(op.activity_type(), ActivityType::FileRead);
        assert!(op.is_file_operation());
    }

    #[test]
    fn operation_file_path_extracts_from_args() {
        let mut args = HashMap::new();
        args.insert("file_path".to_string(), serde_json::json!("/src/main.rs"));
        let op = make_operation("read_file", args);
        assert_eq!(op.file_path(), Some("/src/main.rs".to_string()));
    }

    #[test]
    fn operation_file_path_fallback_to_path_key() {
        let mut args = HashMap::new();
        args.insert("path".to_string(), serde_json::json!("/tmp/file.txt"));
        let op = make_operation("read_file", args);
        assert_eq!(op.file_path(), Some("/tmp/file.txt".to_string()));
    }

    #[test]
    fn operation_file_path_none_when_absent() {
        let op = make_operation("Bash", HashMap::new());
        assert_eq!(op.file_path(), None);
    }

    #[test]
    fn operation_search_pattern_extracts() {
        let mut args = HashMap::new();
        args.insert("pattern".to_string(), serde_json::json!("fn main"));
        let op = make_operation("Grep", args);
        assert_eq!(op.search_pattern(), Some("fn main".to_string()));
    }

    #[test]
    fn operation_search_pattern_fallback_to_query() {
        let mut args = HashMap::new();
        args.insert("query".to_string(), serde_json::json!("TODO"));
        let op = make_operation("SearchText", args);
        assert_eq!(op.search_pattern(), Some("TODO".to_string()));
    }

    #[test]
    fn operation_status_serialization() {
        let json = serde_json::to_string(&OperationStatus::Success).unwrap_or_default();
        assert!(json.contains("success"));
        let json2 = serde_json::to_string(&OperationStatus::Error).unwrap_or_default();
        assert!(json2.contains("error"));
    }
}
