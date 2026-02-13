//! Transcript parsing utilities.
//!
//! Parse and query JSONL transcript files from Claude Code sessions.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::error::HookResult;

/// A single entry in a transcript.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TranscriptEntry {
    /// Entry type (e.g., "user", "assistant", "tool_use", "tool_result")
    #[serde(rename = "type")]
    pub entry_type: String,

    /// Timestamp when the entry was created
    #[serde(default)]
    pub timestamp: Option<String>,

    /// Message content (for user/assistant messages)
    #[serde(default)]
    pub content: Option<Value>,

    /// Tool name (for tool_use entries)
    #[serde(default)]
    pub tool_name: Option<String>,

    /// Tool input parameters
    #[serde(default)]
    pub tool_input: Option<Value>,

    /// Tool use ID
    #[serde(default)]
    pub tool_use_id: Option<String>,

    /// Tool result (for tool_result entries)
    #[serde(default)]
    pub result: Option<Value>,

    /// Error information
    #[serde(default)]
    pub error: Option<Value>,

    /// Raw entry for accessing any field
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, Value>,
}

/// Parsed transcript with query methods.
#[derive(Debug, Clone, Default)]
pub struct Transcript {
    entries: Vec<TranscriptEntry>,
}

impl Transcript {
    /// Load a transcript from a JSONL file.
    pub fn load(path: impl AsRef<Path>) -> HookResult<Self> {
        let file = File::open(path.as_ref())?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str(&line) {
                Ok(entry) => entries.push(entry),
                Err(_) => continue, // Skip malformed lines
            }
        }

        Ok(Self { entries })
    }

    /// Get all entries.
    pub fn entries(&self) -> &[TranscriptEntry] {
        &self.entries
    }

    /// Get entries of a specific type.
    pub fn entries_by_type(&self, entry_type: &str) -> Vec<&TranscriptEntry> {
        self.entries
            .iter()
            .filter(|e| e.entry_type == entry_type)
            .collect()
    }

    /// Get all tool uses.
    pub fn tool_uses(&self) -> Vec<&TranscriptEntry> {
        self.entries_by_type("tool_use")
    }

    /// Get all tool results.
    pub fn tool_results(&self) -> Vec<&TranscriptEntry> {
        self.entries_by_type("tool_result")
    }

    /// Get tool uses for a specific tool.
    pub fn tool_uses_for(&self, tool_name: &str) -> Vec<&TranscriptEntry> {
        self.entries
            .iter()
            .filter(|e| e.entry_type == "tool_use" && e.tool_name.as_deref() == Some(tool_name))
            .collect()
    }

    /// Get the last N entries.
    pub fn last_n(&self, n: usize) -> Vec<&TranscriptEntry> {
        self.entries.iter().rev().take(n).rev().collect()
    }

    /// Search for entries containing text in content.
    pub fn search(&self, query: &str) -> Vec<&TranscriptEntry> {
        let query_lower = query.to_lowercase();
        self.entries
            .iter()
            .filter(|e| {
                if let Some(content) = &e.content {
                    content.to_string().to_lowercase().contains(&query_lower)
                } else {
                    false
                }
            })
            .collect()
    }

    /// Get user messages.
    pub fn user_messages(&self) -> Vec<&TranscriptEntry> {
        self.entries_by_type("user")
    }

    /// Get assistant messages.
    pub fn assistant_messages(&self) -> Vec<&TranscriptEntry> {
        self.entries_by_type("assistant")
    }

    /// Check if any tool use resulted in an error.
    pub fn has_errors(&self) -> bool {
        self.entries.iter().any(|e| e.error.is_some())
    }

    /// Get all errors.
    pub fn errors(&self) -> Vec<&TranscriptEntry> {
        self.entries.iter().filter(|e| e.error.is_some()).collect()
    }

    /// Count entries by type.
    pub fn count_by_type(&self) -> std::collections::HashMap<String, usize> {
        let mut counts = std::collections::HashMap::new();
        for entry in &self.entries {
            *counts.entry(entry.entry_type.clone()).or_insert(0) += 1;
        }
        counts
    }

    /// Generate a summary of the transcript.
    pub fn summary(&self) -> TranscriptSummary {
        let counts = self.count_by_type();
        TranscriptSummary {
            total_entries: self.entries.len(),
            user_messages: counts.get("user").copied().unwrap_or(0),
            assistant_messages: counts.get("assistant").copied().unwrap_or(0),
            tool_uses: counts.get("tool_use").copied().unwrap_or(0),
            tool_results: counts.get("tool_result").copied().unwrap_or(0),
            errors: self.errors().len(),
        }
    }
}

/// Summary statistics for a transcript.
#[derive(Debug, Clone, Default)]
pub struct TranscriptSummary {
    pub total_entries: usize,
    pub user_messages: usize,
    pub assistant_messages: usize,
    pub tool_uses: usize,
    pub tool_results: usize,
    pub errors: usize,
}

impl std::fmt::Display for TranscriptSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Transcript: {} entries ({} user, {} assistant, {} tool uses, {} results, {} errors)",
            self.total_entries,
            self.user_messages,
            self.assistant_messages,
            self.tool_uses,
            self.tool_results,
            self.errors
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transcript_summary_display() {
        let summary = TranscriptSummary {
            total_entries: 10,
            user_messages: 2,
            assistant_messages: 3,
            tool_uses: 3,
            tool_results: 2,
            errors: 0,
        };
        let display = format!("{}", summary);
        assert!(display.contains("10 entries"));
    }
}
