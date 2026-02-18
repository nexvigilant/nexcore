//! Data models for browser shell
//!
//! Tier: T2-C (Cross-domain composites for IPC serialization)

use serde::{Deserialize, Serialize};

/// Tab information for UI display
///
/// Tier: T2-C (Composed from T1 primitives: String, Option)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabInfo {
    /// Unique tab identifier
    pub id: String,
    /// Current URL
    pub url: String,
    /// Page title (if loaded)
    pub title: Option<String>,
    /// Favicon URL (if available)
    pub favicon: Option<String>,
    /// Loading state
    pub loading: bool,
}

impl From<nexcore_browser::PageInfo> for TabInfo {
    fn from(page: nexcore_browser::PageInfo) -> Self {
        Self {
            id: page.id,
            url: page.url,
            title: page.title,
            favicon: None,
            loading: false,
        }
    }
}

/// Shell error type
///
/// Tier: T3 (Domain-specific error wrapper)
/// Shell error type
///
/// Tier: T3 (Domain-specific error wrapper)
#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum ShellError {
    /// Browser operation failed
    #[error("Browser error: {0}")]
    Browser(String),

    /// State management error
    #[error("State error: {0}")]
    State(String),

    /// IPC communication error
    #[error("IPC error: {0}")]
    Ipc(String),
}

impl From<nexcore_browser::BrowserError> for ShellError {
    fn from(err: nexcore_browser::BrowserError) -> Self {
        Self::Browser(err.to_string())
    }
}
