//! External source session types.
//!
//! Represents telemetry sessions from external AI coding assistants.

use nexcore_chrono::DateTime;
use nexcore_id::NexId;
use serde::{Deserialize, Serialize};

use super::operation::Operation;

/// Unique identifier for a telemetry source session.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourceId(pub NexId);

impl From<NexId> for SourceId {
    fn from(id: NexId) -> Self {
        Self(id)
    }
}

impl std::fmt::Display for SourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Project hash identifier (SHA-256 of project path).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectHash(pub String);

impl From<String> for ProjectHash {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl std::fmt::Display for ProjectHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Token usage statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input: u64,
    pub output: u64,
    pub cached: u64,
    pub thoughts: u64,
    pub tool: u64,
    pub total: u64,
}

/// A single entry in a source session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceEntry {
    pub id: String,
    pub timestamp: DateTime,
    #[serde(rename = "type")]
    pub entry_type: EntryType,
    pub content: String,
    #[serde(default)]
    pub thoughts: Vec<Thought>,
    #[serde(default)]
    pub tokens: Option<TokenUsage>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default, rename = "toolCalls")]
    pub operations: Vec<Operation>,
}

/// Type of entry in the session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntryType {
    User,
    #[serde(rename = "gemini")]
    Assistant,
    System,
    #[serde(other)]
    Unknown,
}

/// A thought/reasoning step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thought {
    pub subject: String,
    pub description: String,
    pub timestamp: DateTime,
}

/// A complete telemetry source session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    #[serde(rename = "sessionId")]
    pub id: String,
    #[serde(rename = "projectHash")]
    pub project_hash: String,
    #[serde(rename = "startTime")]
    pub start_time: DateTime,
    #[serde(rename = "lastUpdated")]
    pub last_updated: DateTime,
    pub messages: Vec<SourceEntry>,
}

impl Source {
    /// Get total token usage across all entries.
    #[must_use]
    pub fn total_tokens(&self) -> TokenUsage {
        let mut total = TokenUsage::default();
        for entry in &self.messages {
            if let Some(tokens) = &entry.tokens {
                total.input += tokens.input;
                total.output += tokens.output;
                total.cached += tokens.cached;
                total.thoughts += tokens.thoughts;
                total.tool += tokens.tool;
                total.total += tokens.total;
            }
        }
        total
    }

    /// Get all operations from this session.
    #[must_use]
    pub fn all_operations(&self) -> Vec<&Operation> {
        self.messages.iter().flat_map(|e| &e.operations).collect()
    }

    /// Get operations that accessed files.
    #[must_use]
    pub fn file_operations(&self) -> Vec<&Operation> {
        self.all_operations()
            .into_iter()
            .filter(|op| op.is_file_operation())
            .collect()
    }
}
