//! Browser shell state wrapper
//!
//! Provides Tauri-compatible state management over nexcore-browser.
//!
//! Tier: T3 (Domain-specific state wrapper)

use crate::ShellError;
use crate::models::TabInfo;

/// Browser shell application state
///
/// Wraps nexcore-browser functions for Tauri state management.
/// Thread-safe via nexcore-browser's internal synchronization.
#[derive(Default)]
pub struct BrowserShellState;

impl BrowserShellState {
    /// Create new browser shell state
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Ensure browser is launched
    pub async fn ensure_browser(&self) -> Result<(), ShellError> {
        nexcore_browser::ensure_browser()
            .await
            .map_err(ShellError::from)
    }

    /// Create new page
    pub async fn new_page(&self, url: &str) -> Result<TabInfo, ShellError> {
        let page_info = nexcore_browser::new_page(url)
            .await
            .map_err(ShellError::from)?;
        Ok(TabInfo::from(page_info))
    }

    /// Navigate current page
    pub async fn navigate(&self, url: &str) -> Result<TabInfo, ShellError> {
        let page_info = nexcore_browser::navigate(url)
            .await
            .map_err(ShellError::from)?;
        Ok(TabInfo::from(page_info))
    }

    /// Select page by ID
    pub fn select_page(&self, page_id: &str) -> Result<(), ShellError> {
        nexcore_browser::select_page(page_id).map_err(ShellError::from)
    }

    /// Close page by ID
    pub async fn close_page(&self, page_id: &str) -> Result<(), ShellError> {
        nexcore_browser::close_page(page_id)
            .await
            .map_err(ShellError::from)
    }

    /// List all pages
    pub async fn list_pages(&self) -> Result<Vec<TabInfo>, ShellError> {
        let pages = nexcore_browser::list_pages()
            .await
            .map_err(ShellError::from)?;
        Ok(pages.into_iter().map(TabInfo::from).collect())
    }

    /// Check if browser is running
    #[must_use]
    pub fn is_running(&self) -> bool {
        nexcore_browser::is_browser_running()
    }

    /// Get current page ID
    #[must_use]
    pub fn current_page_id(&self) -> Option<String> {
        nexcore_browser::current_page_id()
    }
}
