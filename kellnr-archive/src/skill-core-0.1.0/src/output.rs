//! Skill output types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Output produced by skill execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillOutput {
    /// Primary output content
    pub content: OutputContent,
    /// Metadata about execution
    pub metadata: HashMap<String, serde_json::Value>,
    /// Suggested follow-up actions
    pub suggestions: Vec<String>,
}

/// Content types that skills can output
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum OutputContent {
    /// Plain text
    Text(String),
    /// Markdown formatted
    Markdown(String),
    /// JSON data
    Json(serde_json::Value),
    /// Table (headers, rows)
    Table {
        /// Column headers
        headers: Vec<String>,
        /// Row data
        rows: Vec<Vec<String>>,
    },
    /// Multiple outputs
    Multi(Vec<OutputContent>),
}

impl SkillOutput {
    /// Create text output
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            content: OutputContent::Text(content.into()),
            metadata: HashMap::new(),
            suggestions: Vec::new(),
        }
    }

    /// Create markdown output
    pub fn markdown(content: impl Into<String>) -> Self {
        Self {
            content: OutputContent::Markdown(content.into()),
            metadata: HashMap::new(),
            suggestions: Vec::new(),
        }
    }

    /// Create JSON output
    pub fn json(value: serde_json::Value) -> Self {
        Self {
            content: OutputContent::Json(value),
            metadata: HashMap::new(),
            suggestions: Vec::new(),
        }
    }

    /// Create table output
    pub fn table(headers: Vec<String>, rows: Vec<Vec<String>>) -> Self {
        Self {
            content: OutputContent::Table { headers, rows },
            metadata: HashMap::new(),
            suggestions: Vec::new(),
        }
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Add suggestion
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestions.push(suggestion.into());
        self
    }
}
