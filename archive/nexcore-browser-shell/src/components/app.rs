//! Root application component
//!
//! Main Leptos component for browser shell UI with Tauri IPC integration.
//!
//! Tier: T3 (Domain-specific UI component)

use crate::components::nav_bar::NavBar;
use crate::components::tab_bar::{TabBar, TabData};
use leptos::prelude::*;

/// Root application component
///
/// Provides the main layout structure for the browser shell.
#[component]
pub fn App() -> impl IntoView {
    // Tab state
    let (tabs, set_tabs) = signal(vec![TabData {
        id: "tab-1".to_string(),
        title: "New Tab".to_string(),
        active: true,
        loading: false,
    }]);
    let (current_url, set_current_url) = signal(String::from("about:blank"));
    let (active_tab_id, set_active_tab_id) = signal(String::from("tab-1"));
    let (status_text, set_status_text) = signal(String::from("Ready"));

    // Tab selection handler
    let on_tab_select = Callback::new(move |tab_id: String| {
        set_active_tab_id.set(tab_id.clone());
        set_tabs.update(|tabs| {
            for tab in tabs.iter_mut() {
                tab.active = tab.id == tab_id;
            }
        });
    });

    // Tab close handler
    let on_tab_close = Callback::new(move |tab_id: String| {
        set_tabs.update(|tabs| {
            tabs.retain(|t| t.id != tab_id);
            // Ensure at least one tab remains active
            if !tabs.iter().any(|t| t.active) {
                if let Some(first) = tabs.first_mut() {
                    first.active = true;
                    set_active_tab_id.set(first.id.clone());
                }
            }
        });
    });

    // New tab handler
    let tab_counter = StoredValue::new(2u32);
    let on_new_tab = Callback::new(move |(): ()| {
        let count = tab_counter.get_value();
        tab_counter.set_value(count + 1);
        let new_id = format!("tab-{count}");
        let new_tab = TabData {
            id: new_id.clone(),
            title: "New Tab".to_string(),
            active: true,
            loading: false,
        };
        set_tabs.update(|tabs| {
            for tab in tabs.iter_mut() {
                tab.active = false;
            }
            tabs.push(new_tab);
        });
        set_active_tab_id.set(new_id);
        set_current_url.set("about:blank".to_string());
    });

    // Navigation handler
    let on_navigate = Callback::new(move |url: String| {
        set_status_text.set(format!("Loading {url}..."));
        set_current_url.set(url.clone());
        // Mark current tab as loading
        let tab_id = active_tab_id.get();
        set_tabs.update(|tabs| {
            if let Some(tab) = tabs.iter_mut().find(|t| t.id == tab_id) {
                tab.loading = true;
                tab.title = url.clone();
            }
        });
        // TODO: Invoke Tauri command to navigate
        // For now, simulate completion
        set_status_text.set("Ready".to_string());
        set_tabs.update(|tabs| {
            if let Some(tab) = tabs.iter_mut().find(|t| t.id == tab_id) {
                tab.loading = false;
            }
        });
    });

    // Back/forward/refresh handlers (placeholder)
    let on_back = Callback::new(move |(): ()| {
        set_status_text.set("Back".to_string());
    });
    let on_forward = Callback::new(move |(): ()| {
        set_status_text.set("Forward".to_string());
    });
    let on_refresh = Callback::new(move |(): ()| {
        set_status_text.set("Refreshing...".to_string());
    });

    view! {
        <div class="browser-shell">
            // Browser Chrome
            <header class="browser-chrome">
                <TabBar
                    tabs=tabs.into()
                    on_select=on_tab_select
                    on_close=on_tab_close
                    on_new_tab=on_new_tab
                />
                <NavBar
                    current_url=current_url.into()
                    on_navigate=on_navigate
                    on_back=on_back
                    on_forward=on_forward
                    on_refresh=on_refresh
                />
            </header>

            // Main Content Area (WebView managed by Tauri)
            <main class="browser-content">
                <div class="webview-placeholder">
                    <p>"NexVigilant Browser"</p>
                    <p class="subtitle">"Rust-native browser shell"</p>
                    <p class="current-url">{move || current_url.get()}</p>
                </div>
            </main>

            // Status Bar
            <footer class="status-bar">
                <span class="status-text">{move || status_text.get()}</span>
                <span class="guardian-indicator" title="Guardian Active">"🛡️"</span>
            </footer>
        </div>
    }
}
