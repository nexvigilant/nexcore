//! NotebookLM MCP tool implementations.
//!
//! Phase 1: 11 sync tools for library CRUD and session management.
//! Phase 2: 3 async tools for auth/health (browser required).
//! Phase 3: 2 async tools for query + cleanup.
//!
//! Grounding: μ(Mapping) + π(Persistence) — maps notebook operations to persistent state.

use crate::params::{
    NlmAddNotebookParams, NlmAskQuestionParams, NlmCleanupDataParams, NlmCloseSessionParams,
    NlmGetHealthParams, NlmGetLibraryStatsParams, NlmGetNotebookParams, NlmListNotebooksParams,
    NlmListSessionsParams, NlmReAuthParams, NlmRemoveNotebookParams, NlmResetSessionParams,
    NlmSearchNotebooksParams, NlmSelectNotebookParams, NlmSetupAuthParams, NlmUpdateNotebookParams,
};
use nexcore_chrono::DateTime;
use nexcore_notebooklm::{HealthStatus, Library, Notebook, SessionStore};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

// ── Helpers ────────────────────────────────────────────────────────────────

fn ok_json(value: serde_json::Value) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

fn err_text(msg: &str) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::error(vec![Content::text(msg.to_string())]))
}

/// Generate a slug-style ID from a name.
fn slugify(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

// ── Phase 1: Library tools (sync) ──────────────────────────────────────────

/// Add a notebook to the library.
pub fn add_notebook(params: NlmAddNotebookParams) -> Result<CallToolResult, McpError> {
    let mut lib = Library::load().map_err(|e| McpError::internal_error(format!("{e}"), None))?;

    let now = DateTime::now();
    let id = slugify(&params.name);

    let notebook = Notebook {
        id: id.clone(),
        name: params.name,
        url: params.url,
        description: params.description,
        topics: params.topics,
        content_types: params.content_types,
        use_cases: params.use_cases,
        tags: params.tags,
        created_at: now,
        updated_at: now,
    };

    if let Err(e) = lib.add(notebook) {
        return err_text(&format!("failed to add notebook: {e}"));
    }

    ok_json(json!({
        "success": true,
        "id": id,
        "message": "notebook added to library"
    }))
}

/// List all notebooks in the library.
pub fn list_notebooks(_params: NlmListNotebooksParams) -> Result<CallToolResult, McpError> {
    let lib = Library::load().map_err(|e| McpError::internal_error(format!("{e}"), None))?;

    let notebooks: Vec<serde_json::Value> = lib
        .notebooks
        .iter()
        .map(|n| {
            json!({
                "id": n.id,
                "name": n.name,
                "url": n.url,
                "description": n.description,
                "topics": n.topics,
                "tags": n.tags,
                "active": lib.active_id.as_deref() == Some(&n.id),
            })
        })
        .collect();

    ok_json(json!({
        "total": notebooks.len(),
        "active_notebook": lib.active_id,
        "notebooks": notebooks
    }))
}

/// Get details of a specific notebook.
pub fn get_notebook(params: NlmGetNotebookParams) -> Result<CallToolResult, McpError> {
    let lib = Library::load().map_err(|e| McpError::internal_error(format!("{e}"), None))?;

    match lib.get(&params.id) {
        Ok(nb) => ok_json(json!({
            "id": nb.id,
            "name": nb.name,
            "url": nb.url,
            "description": nb.description,
            "topics": nb.topics,
            "content_types": nb.content_types,
            "use_cases": nb.use_cases,
            "tags": nb.tags,
            "created_at": nb.created_at.to_rfc3339(),
            "updated_at": nb.updated_at.to_rfc3339(),
            "active": lib.active_id.as_deref() == Some(&nb.id),
        })),
        Err(e) => err_text(&format!("{e}")),
    }
}

/// Set the active notebook.
pub fn select_notebook(params: NlmSelectNotebookParams) -> Result<CallToolResult, McpError> {
    let mut lib = Library::load().map_err(|e| McpError::internal_error(format!("{e}"), None))?;

    if let Err(e) = lib.select(&params.id) {
        return err_text(&format!("{e}"));
    }

    let name = lib
        .get(&params.id)
        .map(|n| n.name.clone())
        .unwrap_or_default();

    ok_json(json!({
        "success": true,
        "active_notebook": params.id,
        "name": name,
        "message": format!("switched to '{name}'")
    }))
}

/// Update notebook metadata.
pub fn update_notebook(params: NlmUpdateNotebookParams) -> Result<CallToolResult, McpError> {
    let mut lib = Library::load().map_err(|e| McpError::internal_error(format!("{e}"), None))?;

    match lib.update(
        &params.id,
        params.name,
        params.description,
        params.url,
        params.topics,
        params.content_types,
        params.use_cases,
        params.tags,
    ) {
        Ok(nb) => ok_json(json!({
            "success": true,
            "id": nb.id,
            "name": nb.name,
            "updated_at": nb.updated_at.to_rfc3339(),
        })),
        Err(e) => err_text(&format!("{e}")),
    }
}

/// Remove a notebook from the library.
pub fn remove_notebook(params: NlmRemoveNotebookParams) -> Result<CallToolResult, McpError> {
    let mut lib = Library::load().map_err(|e| McpError::internal_error(format!("{e}"), None))?;

    match lib.remove(&params.id) {
        Ok(removed) => ok_json(json!({
            "success": true,
            "removed": {
                "id": removed.id,
                "name": removed.name,
            },
            "message": format!("removed '{}' from library", removed.name)
        })),
        Err(e) => err_text(&format!("{e}")),
    }
}

/// Search notebooks by keyword.
pub fn search_notebooks(params: NlmSearchNotebooksParams) -> Result<CallToolResult, McpError> {
    let lib = Library::load().map_err(|e| McpError::internal_error(format!("{e}"), None))?;

    let results: Vec<serde_json::Value> = lib
        .search(&params.query)
        .iter()
        .map(|n| {
            json!({
                "id": n.id,
                "name": n.name,
                "description": n.description,
                "topics": n.topics,
                "tags": n.tags,
            })
        })
        .collect();

    ok_json(json!({
        "query": params.query,
        "total": results.len(),
        "results": results
    }))
}

/// Get library statistics.
pub fn get_library_stats(_params: NlmGetLibraryStatsParams) -> Result<CallToolResult, McpError> {
    let lib = Library::load().map_err(|e| McpError::internal_error(format!("{e}"), None))?;
    let stats = lib.stats();

    ok_json(json!({
        "total_notebooks": stats.total_notebooks,
        "total_topics": stats.total_topics,
        "total_tags": stats.total_tags,
        "most_recent": stats.most_recent,
        "active_notebook": lib.active_id,
    }))
}

// ── Phase 1: Session tools (sync) ──────────────────────────────────────────

/// List all active sessions.
pub fn list_sessions(_params: NlmListSessionsParams) -> Result<CallToolResult, McpError> {
    let store = SessionStore::load().map_err(|e| McpError::internal_error(format!("{e}"), None))?;

    let sessions: Vec<serde_json::Value> = store
        .list()
        .iter()
        .map(|s| {
            json!({
                "id": s.id,
                "notebook_id": s.notebook_id,
                "message_count": s.message_count,
                "created_at": s.created_at.to_rfc3339(),
                "last_activity": s.last_activity.to_rfc3339(),
            })
        })
        .collect();

    ok_json(json!({
        "total": sessions.len(),
        "sessions": sessions
    }))
}

/// Close a session.
pub fn close_session(params: NlmCloseSessionParams) -> Result<CallToolResult, McpError> {
    let mut store =
        SessionStore::load().map_err(|e| McpError::internal_error(format!("{e}"), None))?;

    match store.close(&params.session_id) {
        Ok(removed) => ok_json(json!({
            "success": true,
            "closed_session": removed.id,
            "notebook_id": removed.notebook_id,
        })),
        Err(e) => err_text(&format!("{e}")),
    }
}

/// Reset a session's chat history.
pub fn reset_session(params: NlmResetSessionParams) -> Result<CallToolResult, McpError> {
    let mut store =
        SessionStore::load().map_err(|e| McpError::internal_error(format!("{e}"), None))?;

    match store.reset(&params.session_id) {
        Ok(()) => ok_json(json!({
            "success": true,
            "session_id": params.session_id,
            "message": "session reset — chat history cleared"
        })),
        Err(e) => err_text(&format!("{e}")),
    }
}

// ── Phase 2: Auth + Health (async — browser automation) ────────────────────

/// Get health status — reports browser, auth, library, sessions.
pub fn get_health(_params: NlmGetHealthParams) -> Result<CallToolResult, McpError> {
    let lib = Library::load().unwrap_or_default();
    let store = SessionStore::load().unwrap_or_default();

    let browser_running = nexcore_notebooklm::browser::is_running();
    let has_cookies = nexcore_notebooklm::browser::has_auth_cookies();

    let data_dir = nexcore_notebooklm::persistence::data_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    ok_json(json!({
        "browser_running": browser_running,
        "authenticated": has_cookies,
        "library_size": lib.notebooks.len(),
        "active_sessions": store.sessions.len(),
        "active_notebook": lib.active_id,
        "data_dir": data_dir,
    }))
}

/// Interactive Google login — opens Chrome for user to sign in.
///
/// Records auth state on success so `has_auth_cookies()` returns true
/// after the user confirms they've logged in.
pub async fn setup_auth(_params: NlmSetupAuthParams) -> Result<CallToolResult, McpError> {
    match nexcore_notebooklm::auth::setup_auth().await {
        Ok(result) => {
            // Record auth success — user will confirm login before next call
            nexcore_notebooklm::browser::record_auth_success(None);

            ok_json(json!({
                "success": true,
                "browser_opened": result.browser_opened,
                "url": result.url,
                "message": result.message,
            }))
        }
        Err(e) => err_text(&format!("setup_auth failed: {e}")),
    }
}

/// Clear auth data and re-authenticate with fresh Chrome profile.
pub async fn re_auth(_params: NlmReAuthParams) -> Result<CallToolResult, McpError> {
    match nexcore_notebooklm::auth::re_auth().await {
        Ok(result) => {
            // Record auth success — user will confirm login before next call
            nexcore_notebooklm::browser::record_auth_success(None);

            ok_json(json!({
                "success": true,
                "browser_opened": result.browser_opened,
                "url": result.url,
                "message": result.message,
            }))
        }
        Err(e) => err_text(&format!("re_auth failed: {e}")),
    }
}

/// Ask a question to a NotebookLM notebook.
///
/// Resolves notebook URL from library, launches browser if needed,
/// types question, waits for response with 3-poll stability check.
pub async fn ask_question(params: NlmAskQuestionParams) -> Result<CallToolResult, McpError> {
    let lib = Library::load().unwrap_or_default();

    // Resolve notebook URL
    let (notebook_id, notebook_url) = if let Some(ref url) = params.notebook_url {
        // Direct URL override
        let id = params
            .notebook_id
            .unwrap_or_else(|| "direct-url".to_string());
        (id, url.clone())
    } else if let Some(ref id) = params.notebook_id {
        // Look up by ID in library
        match lib.get(id) {
            Ok(nb) => (id.clone(), nb.url.clone()),
            Err(e) => return err_text(&format!("notebook not found: {e}")),
        }
    } else {
        // Use active notebook
        match lib.active() {
            Ok(nb) => (lib.active_id.clone().unwrap_or_default(), nb.url.clone()),
            Err(_) => {
                return err_text(
                    "no notebook specified and no active notebook set. \
                     Use notebook_url, notebook_id, or nlm_select_notebook first.",
                );
            }
        }
    };

    match nexcore_notebooklm::notebook::ask_question(
        &notebook_url,
        &params.question,
        params.session_id.as_deref(),
        &notebook_id,
    )
    .await
    {
        Ok(result) => ok_json(json!({
            "answer": result.answer,
            "session_id": result.session_id,
            "notebook_id": result.notebook_id,
            "rate_limited": result.rate_limited,
            "duration_ms": result.duration_ms,
        })),
        Err(e) => err_text(&format!("ask_question failed: {e}")),
    }
}

/// Cleanup NotebookLM data — closes browser, deletes data dir contents.
pub async fn cleanup_data(params: NlmCleanupDataParams) -> Result<CallToolResult, McpError> {
    // Close browser first if running
    if nexcore_notebooklm::browser::is_running() {
        if let Err(e) = nexcore_notebooklm::browser::close().await {
            tracing::warn!("failed to close browser during cleanup: {e}");
        }
    }

    let data_dir = nexcore_notebooklm::persistence::data_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    if !params.confirm {
        return ok_json(json!({
            "preview": true,
            "data_dir": data_dir,
            "preserve_library": params.preserve_library,
            "message": "pass confirm=true to actually delete data"
        }));
    }

    let dir = match nexcore_notebooklm::persistence::data_dir() {
        Ok(d) => d,
        Err(e) => return err_text(&format!("cannot determine data dir: {e}")),
    };

    let mut deleted = Vec::new();
    let mut preserved = Vec::new();

    // Walk the data directory
    if dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                // Preserve library if requested
                if params.preserve_library && name == "library.json" {
                    preserved.push(name);
                    continue;
                }

                // Delete files and directories
                if path.is_dir() {
                    if let Err(e) = std::fs::remove_dir_all(&path) {
                        return err_text(&format!("failed to remove {name}: {e}"));
                    }
                } else if let Err(e) = std::fs::remove_file(&path) {
                    return err_text(&format!("failed to remove {name}: {e}"));
                }
                deleted.push(name);
            }
        }
    }

    ok_json(json!({
        "success": true,
        "deleted": deleted,
        "preserved": preserved,
        "data_dir": data_dir,
    }))
}
