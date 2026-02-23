//! NotebookLM MCP tool parameters.
//!
//! 16 param structs for the `nlm_*` namespace.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

// ── Phase 1: Library tools (sync) ──────────────────────────────────────────

/// Parameters for `nlm_add_notebook`.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct NlmAddNotebookParams {
    /// NotebookLM URL.
    pub url: String,
    /// Display name for the notebook.
    pub name: String,
    /// What knowledge/content is in this notebook.
    pub description: String,
    /// Topics covered.
    #[serde(default)]
    pub topics: Vec<String>,
    /// Types of content (e.g., "documentation", "examples").
    #[serde(default)]
    pub content_types: Vec<String>,
    /// When to use this notebook.
    #[serde(default)]
    pub use_cases: Vec<String>,
    /// Organization tags.
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Parameters for `nlm_list_notebooks` (no params needed).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct NlmListNotebooksParams {}

/// Parameters for `nlm_get_notebook`.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct NlmGetNotebookParams {
    /// Notebook ID.
    pub id: String,
}

/// Parameters for `nlm_select_notebook`.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct NlmSelectNotebookParams {
    /// Notebook ID to set as active.
    pub id: String,
}

/// Parameters for `nlm_update_notebook`.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct NlmUpdateNotebookParams {
    /// Notebook ID to update.
    pub id: String,
    /// New display name.
    #[serde(default)]
    pub name: Option<String>,
    /// New description.
    #[serde(default)]
    pub description: Option<String>,
    /// New URL.
    #[serde(default)]
    pub url: Option<String>,
    /// New topics list.
    #[serde(default)]
    pub topics: Option<Vec<String>>,
    /// New content types.
    #[serde(default)]
    pub content_types: Option<Vec<String>>,
    /// New use cases.
    #[serde(default)]
    pub use_cases: Option<Vec<String>>,
    /// New tags.
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

/// Parameters for `nlm_remove_notebook`.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct NlmRemoveNotebookParams {
    /// Notebook ID to remove.
    pub id: String,
}

/// Parameters for `nlm_search_notebooks`.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct NlmSearchNotebooksParams {
    /// Search query (matches name, description, topics, tags).
    pub query: String,
}

/// Parameters for `nlm_get_library_stats` (no params needed).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct NlmGetLibraryStatsParams {}

// ── Phase 1: Session tools (sync) ──────────────────────────────────────────

/// Parameters for `nlm_list_sessions` (no params needed).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct NlmListSessionsParams {}

/// Parameters for `nlm_close_session`.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct NlmCloseSessionParams {
    /// Session ID to close.
    pub session_id: String,
}

/// Parameters for `nlm_reset_session`.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct NlmResetSessionParams {
    /// Session ID to reset.
    pub session_id: String,
}

// ── Phase 2: Auth tools (async) ────────────────────────────────────────────

/// Parameters for `nlm_setup_auth` (no params needed).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct NlmSetupAuthParams {}

/// Parameters for `nlm_re_auth` (no params needed).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct NlmReAuthParams {}

/// Parameters for `nlm_get_health` (no params needed).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct NlmGetHealthParams {}

// ── Phase 3: Query + cleanup (async) ───────────────────────────────────────

/// Parameters for `nlm_ask_question`.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct NlmAskQuestionParams {
    /// The question to ask NotebookLM.
    pub question: String,
    /// Notebook ID (uses active notebook if omitted).
    #[serde(default)]
    pub notebook_id: Option<String>,
    /// Notebook URL (overrides notebook_id).
    #[serde(default)]
    pub notebook_url: Option<String>,
    /// Session ID for contextual conversations.
    #[serde(default)]
    pub session_id: Option<String>,
}

/// Parameters for `nlm_cleanup_data`.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct NlmCleanupDataParams {
    /// Whether to actually delete (false = preview only).
    #[serde(default)]
    pub confirm: bool,
    /// Preserve library.json during cleanup.
    #[serde(default)]
    pub preserve_library: bool,
}
