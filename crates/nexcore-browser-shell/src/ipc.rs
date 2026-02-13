//! Frontend IPC bindings for Tauri command invocation.
//!
//! Provides typed wrappers around Tauri's `window.__TAURI__.core.invoke()`
//! for use by Leptos components compiled to WASM.
//!
//! Tier: T3 (Domain-specific IPC bridge)

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::models::{ShellError, TabInfo};

// Raw JS binding to Tauri's invoke function.
// Requires `withGlobalTauri: true` in tauri.conf.json.
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

/// Typed invoke helper — serializes args, deserializes response.
async fn invoke_typed<A: Serialize, R: for<'de> Deserialize<'de>>(
    cmd: &str,
    args: &A,
) -> Result<R, ShellError> {
    let js_args =
        serde_wasm_bindgen::to_value(args).map_err(|e| ShellError::Ipc(e.to_string()))?;

    let result = invoke(cmd, js_args)
        .await
        .map_err(|e| ShellError::Ipc(format!("{e:?}")))?;

    serde_wasm_bindgen::from_value(result).map_err(|e| ShellError::Ipc(e.to_string()))
}

/// Invoke a command with no return value.
async fn invoke_void<A: Serialize>(cmd: &str, args: &A) -> Result<(), ShellError> {
    let js_args =
        serde_wasm_bindgen::to_value(args).map_err(|e| ShellError::Ipc(e.to_string()))?;

    invoke(cmd, js_args)
        .await
        .map_err(|e| ShellError::Ipc(format!("{e:?}")))?;

    Ok(())
}

// ── Typed IPC Commands ──────────────────────────────────────────────

/// Launch the browser backend.
pub async fn ipc_browser_launch() -> Result<(), ShellError> {
    #[derive(Serialize)]
    struct Args {}
    invoke_void("browser_launch", &Args {}).await
}

/// Navigate current page to URL.
pub async fn ipc_browser_navigate(url: &str) -> Result<TabInfo, ShellError> {
    #[derive(Serialize)]
    struct Args<'a> {
        url: &'a str,
    }
    invoke_typed("browser_navigate", &Args { url }).await
}

/// Create new page at URL.
pub async fn ipc_browser_new_page(url: &str) -> Result<TabInfo, ShellError> {
    #[derive(Serialize)]
    struct Args<'a> {
        url: &'a str,
    }
    invoke_typed("browser_new_page", &Args { url }).await
}

/// Close page by ID.
pub async fn ipc_browser_close_page(page_id: &str) -> Result<(), ShellError> {
    #[derive(Serialize)]
    struct Args<'a> {
        page_id: &'a str,
    }
    invoke_void("browser_close_page", &Args { page_id }).await
}

/// Select page by ID.
pub async fn ipc_browser_select_page(page_id: &str) -> Result<(), ShellError> {
    #[derive(Serialize)]
    struct Args<'a> {
        page_id: &'a str,
    }
    invoke_void("browser_select_page", &Args { page_id }).await
}

/// List all pages.
pub async fn ipc_browser_list_pages() -> Result<Vec<TabInfo>, ShellError> {
    #[derive(Serialize)]
    struct Args {}
    invoke_typed("browser_list_pages", &Args {}).await
}

/// Check if browser is running.
pub async fn ipc_browser_is_running() -> Result<bool, ShellError> {
    #[derive(Serialize)]
    struct Args {}
    invoke_typed("browser_is_running", &Args {}).await
}
