//! # NexVigilant Core — Browser
//!
//! Browser automation foundation for Vigil/Guardian ecosystem.
//!
//! Provides:
//! - Browser state management with CDP event routing
//! - Console and network collectors with FIFO bounds
//! - Event broadcasting for Vigil integration
//! - Guardian-compatible threat pattern detection
//!
//! ## Architecture
//!
//! ```text
//! nexcore-browser (This crate)
//!     ↑              ↑
//! nexcore-mcp    nexcore-vigil    ← Both depend on browser crate
//!                    ↓
//!             nexcore-vigilance   ← Guardian sensor integration
//! ```
//!
//! ## Example
//!
//! ```ignore
//! use nexcore_browser::{state, events::BrowserEvent};
//!
//! // Launch browser
//! state::ensure_browser().await?;
//!
//! // Create page (events auto-collected)
//! let page_info = state::new_page("https://example.com").await?;
//!
//! // Subscribe to events for Vigil
//! let mut rx = state::subscribe_events();
//! while let Ok(event) = rx.recv().await {
//!     match event {
//!         BrowserEvent::ConsoleMessage { .. } => { /* handle */ }
//!         BrowserEvent::NetworkFailure { .. } => { /* handle */ }
//!         _ => {}
//!     }
//! }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod collectors;
pub mod events;
pub mod grounding;
pub mod handlers;
pub mod mcp_bridge;
pub mod state;

// Re-exports for convenience
pub use events::BrowserEvent;
pub use state::{
    BrowserError, BrowserSettings, PageInfo, close_browser, close_page, current_page_id,
    ensure_browser, get_context, get_current_page, get_page, is_browser_running, launch_browser,
    list_pages, navigate, new_page, page_count, select_page, subscribe_events, update_settings,
};

pub use collectors::console::{
    ConsoleCollector, ConsoleEntry, ConsoleLevel, get_console_collector,
};
pub use collectors::network::{
    NetworkCollector, NetworkEntry, NetworkStatus, get_network_collector,
};

pub use mcp_bridge::{
    McpBridge, McpBridgeConfig, McpBridgeError, McpBridgeStats, McpConsoleMessage,
    McpNetworkRequest, sync_console_from_json, sync_network_from_json,
};
