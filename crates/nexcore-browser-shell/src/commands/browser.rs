//! Browser control commands
//!
//! Tauri commands wrapping nexcore-browser state functions.
//!
//! Tier: T3 (Domain-specific Tauri IPC layer)

use tauri::State;

use crate::ShellError;
use crate::models::TabInfo;
use crate::state::BrowserShellState;

/// Launch browser if not running
#[tauri::command]
pub async fn browser_launch(state: State<'_, BrowserShellState>) -> Result<(), ShellError> {
    state.ensure_browser().await
}

/// Create new page and navigate to URL
#[tauri::command]
pub async fn browser_new_page(
    state: State<'_, BrowserShellState>,
    url: String,
) -> Result<TabInfo, ShellError> {
    state.new_page(&url).await
}

/// Navigate current page to URL
#[tauri::command]
pub async fn browser_navigate(
    state: State<'_, BrowserShellState>,
    url: String,
) -> Result<TabInfo, ShellError> {
    state.navigate(&url).await
}

/// Select page by ID
#[tauri::command]
pub async fn browser_select_page(
    state: State<'_, BrowserShellState>,
    page_id: String,
) -> Result<(), ShellError> {
    state.select_page(&page_id)
}

/// Close page by ID
#[tauri::command]
pub async fn browser_close_page(
    state: State<'_, BrowserShellState>,
    page_id: String,
) -> Result<(), ShellError> {
    state.close_page(&page_id).await
}

/// List all pages
#[tauri::command]
pub async fn browser_list_pages(
    state: State<'_, BrowserShellState>,
) -> Result<Vec<TabInfo>, ShellError> {
    state.list_pages().await
}

/// Check if browser is running
#[tauri::command]
pub async fn browser_is_running(state: State<'_, BrowserShellState>) -> Result<bool, ShellError> {
    Ok(state.is_running())
}

/// Get current page ID
#[tauri::command]
pub async fn browser_current_page_id(
    state: State<'_, BrowserShellState>,
) -> Result<Option<String>, ShellError> {
    Ok(state.current_page_id())
}
