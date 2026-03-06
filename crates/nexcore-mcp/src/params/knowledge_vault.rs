//! Parameter structs for Knowledge Vault MCP tools.
//!
//! Operates on Obsidian-compatible markdown vaults (directories of .md files).

use rmcp::serde::Deserialize;
use schemars::JsonSchema;

/// Parameters for reading a note from the vault.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KnowledgeVaultReadParams {
    /// Path to the note relative to vault root (e.g., "400-projects/ksb-framework/overview.md").
    /// The .md extension is optional — it will be appended if missing.
    pub path: String,
}

/// Parameters for searching vault content.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KnowledgeVaultSearchParams {
    /// Search query — matched against note content and filenames.
    /// Supports simple substring matching. Case-insensitive.
    pub query: String,
    /// Optional subdirectory to scope the search (e.g., "400-projects").
    #[serde(default)]
    pub scope: Option<String>,
    /// Maximum number of results to return (default: 20).
    #[serde(default)]
    pub limit: Option<usize>,
}

/// Parameters for listing vault contents.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KnowledgeVaultListParams {
    /// Directory path relative to vault root (default: root).
    #[serde(default)]
    pub path: Option<String>,
    /// If true, list recursively. Default: false.
    #[serde(default)]
    pub recursive: Option<bool>,
}

/// Parameters for writing/creating a note.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KnowledgeVaultWriteParams {
    /// Path relative to vault root (e.g., "400-projects/cccp/overview.md").
    /// Parent directories will be created if they don't exist.
    /// The .md extension is optional — it will be appended if missing.
    pub path: String,
    /// Note content (markdown).
    pub content: String,
}

/// Parameters for moving/renaming a note.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KnowledgeVaultMoveParams {
    /// Current path relative to vault root.
    pub from: String,
    /// New path relative to vault root.
    pub to: String,
}

/// Parameters for listing tags across the vault.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KnowledgeVaultTagsParams {
    /// Optional: only show tags from notes in this subdirectory.
    #[serde(default)]
    pub scope: Option<String>,
}
