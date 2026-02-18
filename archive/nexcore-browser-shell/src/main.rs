//! NexVigilant Browser Shell
//!
//! Rust-native browser built with Tauri + Leptos.
//!
//! ## Features
//!
//! - Full browser automation via nexcore-browser (CDP)
//! - Guardian threat detection (PAMPs/DAMPs)
//! - Vigil AI orchestration integration
//! - DevTools panel (Console, Network, Performance)
//!
//! ## Architecture
//!
//! - **Frontend**: Leptos (WASM, CSR mode)
//! - **Backend**: Tauri (Rust native)
//! - **Browser**: nexcore-browser (chromiumoxide CDP)
//! - **Security**: nexcore-vigilance Guardian

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use nexcore_browser_shell::commands::browser::*;
use nexcore_browser_shell::commands::webview::*;
use nexcore_browser_shell::state::BrowserShellState;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("nexcore_browser_shell=debug".parse().unwrap_or_default()),
        )
        .init();

    tracing::info!("Starting NexVigilant Browser Shell");

    // Build Tauri application
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(BrowserShellState::new())
        .invoke_handler(tauri::generate_handler![
            // Browser CDP commands
            browser_launch,
            browser_new_page,
            browser_navigate,
            browser_select_page,
            browser_close_page,
            browser_list_pages,
            browser_is_running,
            browser_current_page_id,
            // Content view commands
            content_view_create,
            content_view_navigate,
            content_view_close,
            content_view_resize,
            content_view_list,
        ])
        .run(tauri::generate_context!());

    if let Err(e) = result {
        tracing::error!("Failed to run NexVigilant Browser: {e}");
        std::process::exit(1);
    }
}
