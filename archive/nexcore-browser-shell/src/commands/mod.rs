//! Tauri IPC commands for browser shell
//!
//! Provides the bridge between Leptos frontend and nexcore-browser backend.

pub mod browser;
pub mod webview;

pub use browser::*;
pub use webview::*;
