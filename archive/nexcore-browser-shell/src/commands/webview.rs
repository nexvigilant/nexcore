//! `WebView` management commands
//!
//! Tauri commands for creating and managing child views within the browser shell.
//!
//! Tier: T3 (Domain-specific view management)

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::ShellError;

/// View bounds for positioning
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ViewBounds {
    /// X position from left
    pub x: f64,
    /// Y position from top (below chrome)
    pub y: f64,
    /// Width in logical pixels
    pub width: f64,
    /// Height in logical pixels
    pub height: f64,
}

/// Create a new content view for browsing
///
/// Creates a child window that displays web content, controlled via CDP.
#[tauri::command]
pub async fn content_view_create(
    app: AppHandle,
    label: String,
    url: String,
    bounds: ViewBounds,
) -> Result<String, ShellError> {
    // Validate URL
    let view_url = if url.starts_with("http://") || url.starts_with("https://") {
        WebviewUrl::External(
            url.parse()
                .map_err(|e| ShellError::Ipc(format!("Invalid URL: {e}")))?,
        )
    } else if url == "about:blank" {
        WebviewUrl::External(
            "about:blank"
                .parse()
                .map_err(|e| ShellError::Ipc(format!("URL parse error: {e}")))?,
        )
    } else {
        // Assume it's an app path
        WebviewUrl::App(url.into())
    };

    // Create content window (separate window approach)
    let content = WebviewWindowBuilder::new(&app, &label, view_url)
        .title("NexVigilant - Content")
        .inner_size(bounds.width, bounds.height)
        .position(bounds.x, bounds.y)
        .decorations(false)
        .transparent(false)
        .build()
        .map_err(|e| ShellError::Ipc(format!("View creation failed: {e}")))?;

    Ok(content.label().to_string())
}

/// Navigate a content view to a new URL
#[tauri::command]
pub async fn content_view_navigate(
    app: AppHandle,
    label: String,
    url: String,
) -> Result<(), ShellError> {
    let view = app
        .get_webview_window(&label)
        .ok_or_else(|| ShellError::State(format!("View not found: {label}")))?;

    let parsed_url: tauri::Url = url
        .parse()
        .map_err(|e| ShellError::Ipc(format!("Invalid URL: {e}")))?;

    view.navigate(parsed_url)
        .map_err(|e| ShellError::Ipc(format!("Navigation failed: {e}")))
}

/// Close a content view
#[tauri::command]
pub async fn content_view_close(app: AppHandle, label: String) -> Result<(), ShellError> {
    let view = app
        .get_webview_window(&label)
        .ok_or_else(|| ShellError::State(format!("View not found: {label}")))?;

    view.close()
        .map_err(|e| ShellError::Ipc(format!("Close failed: {e}")))
}

/// Resize a content view
#[tauri::command]
pub async fn content_view_resize(
    app: AppHandle,
    label: String,
    bounds: ViewBounds,
) -> Result<(), ShellError> {
    let view = app
        .get_webview_window(&label)
        .ok_or_else(|| ShellError::State(format!("View not found: {label}")))?;

    view.set_size(tauri::LogicalSize::new(bounds.width, bounds.height))
        .map_err(|e| ShellError::Ipc(format!("Resize failed: {e}")))?;

    view.set_position(tauri::LogicalPosition::new(bounds.x, bounds.y))
        .map_err(|e| ShellError::Ipc(format!("Reposition failed: {e}")))
}

/// List all content view labels
#[tauri::command]
pub async fn content_view_list(app: AppHandle) -> Result<Vec<String>, ShellError> {
    Ok(app
        .webview_windows()
        .keys()
        .filter(|label| *label != "main")
        .cloned()
        .collect())
}
