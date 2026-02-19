//! Browser automation module for Chrome DevTools MCP integration
//!
//! Re-exports from `nexcore-browser` for shared state with Vigil/Guardian.
//!
//! ## Architecture
//!
//! ```text
//! nexcore-browser (shared foundation)
//!     ↑              ↑
//! nexcore-mcp    nexcore-vigil    ← Both use same browser state
//!                    ↓
//!             nexcore-vigilance   ← Guardian sensor integration
//! ```
//!
//! ## Usage
//!
//! ```ignore
//! use crate::browser::state;
//!
//! // Launch browser (idempotent)
//! state::ensure_browser().await?;
//!
//! // Create new page
//! let page_info = state::new_page("https://example.com").await?;
//!
//! // Navigate
//! state::navigate("https://google.com").await?;
//!
//! // Get current page for operations
//! let page = state::get_current_page()?;
//! ```

// Re-export entire state module from nexcore-browser
pub use nexcore_browser::state;

// Re-export commonly used types at module level for backwards compatibility
pub use nexcore_browser::{
    BrowserError, BrowserEvent, BrowserSettings, ConsoleCollector, ConsoleEntry, ConsoleLevel,
    NetworkCollector, NetworkEntry, NetworkStatus, PageInfo, close_browser, close_page,
    current_page_id, ensure_browser, get_context, get_current_page, get_page, is_browser_running,
    launch_browser, list_pages, navigate, new_page, page_count, select_page, subscribe_events,
    update_settings,
};

// Re-export collector accessors
pub use nexcore_browser::{get_console_collector, get_network_collector};

/// Quantified code for BrowserError variants.
///
/// Tier: T2-P (Cross-domain primitive code)
/// Grounds to: T1 primitive `u8`
/// Ord: Implemented (numeric code ordering)
///
/// MCP-specific: Used for error code mapping in tool responses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BrowserErrorCode(pub u8);

impl From<BrowserError> for BrowserErrorCode {
    fn from(value: BrowserError) -> Self {
        match value {
            BrowserError::Launch(_) => BrowserErrorCode(1),
            BrowserError::NotConnected => BrowserErrorCode(2),
            BrowserError::NoPageSelected => BrowserErrorCode(3),
            BrowserError::PageNotFound(_) => BrowserErrorCode(4),
            BrowserError::Navigation(_) => BrowserErrorCode(5),
            BrowserError::PageClose(_) => BrowserErrorCode(6),
            BrowserError::ElementNotFound(_) => BrowserErrorCode(7),
            BrowserError::JsEval(_) => BrowserErrorCode(8),
            BrowserError::Screenshot(_) => BrowserErrorCode(9),
            BrowserError::Trace(_) => BrowserErrorCode(10),
            BrowserError::Network(_) => BrowserErrorCode(11),
            BrowserError::Input(_) => BrowserErrorCode(12),
        }
    }
}

impl From<&BrowserError> for BrowserErrorCode {
    fn from(value: &BrowserError) -> Self {
        match value {
            BrowserError::Launch(_) => BrowserErrorCode(1),
            BrowserError::NotConnected => BrowserErrorCode(2),
            BrowserError::NoPageSelected => BrowserErrorCode(3),
            BrowserError::PageNotFound(_) => BrowserErrorCode(4),
            BrowserError::Navigation(_) => BrowserErrorCode(5),
            BrowserError::PageClose(_) => BrowserErrorCode(6),
            BrowserError::ElementNotFound(_) => BrowserErrorCode(7),
            BrowserError::JsEval(_) => BrowserErrorCode(8),
            BrowserError::Screenshot(_) => BrowserErrorCode(9),
            BrowserError::Trace(_) => BrowserErrorCode(10),
            BrowserError::Network(_) => BrowserErrorCode(11),
            BrowserError::Input(_) => BrowserErrorCode(12),
        }
    }
}
