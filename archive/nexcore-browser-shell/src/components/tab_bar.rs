//! Tab bar component
//!
//! Displays browser tabs with close buttons and new tab control.
//!
//! Tier: T3 (Domain-specific UI component)

use leptos::prelude::*;

/// Single tab data
#[derive(Debug, Clone, PartialEq)]
pub struct TabData {
    /// Unique tab identifier
    pub id: String,
    /// Tab display title
    pub title: String,
    /// Whether this tab is active
    pub active: bool,
    /// Loading indicator
    pub loading: bool,
}

/// Tab bar component
#[component]
pub fn TabBar(
    tabs: Signal<Vec<TabData>>,
    on_select: Callback<String>,
    on_close: Callback<String>,
    on_new_tab: Callback<()>,
) -> impl IntoView {
    view! {
        <div class="tab-bar">
            <For
                each=move || tabs.get()
                key=|tab| tab.id.clone()
                children=move |tab| {
                    let tab_id = tab.id.clone();
                    let tab_id_close = tab.id.clone();
                    let is_active = tab.active;
                    let title = tab.title.clone();
                    let loading = tab.loading;

                    view! {
                        <div
                            class=move || if is_active { "tab active" } else { "tab" }
                            on:click=move |_| on_select.run(tab_id.clone())
                        >
                            {move || if loading {
                                view! { <span class="loading-spinner">"⏳"</span> }.into_any()
                            } else {
                                view! { <span></span> }.into_any()
                            }}
                            <span class="tab-title">{title.clone()}</span>
                            <button
                                class="tab-close"
                                on:click=move |ev| {
                                    ev.stop_propagation();
                                    on_close.run(tab_id_close.clone());
                                }
                            >
                                "×"
                            </button>
                        </div>
                    }
                }
            />
            <button
                class="new-tab-btn"
                on:click=move |_| on_new_tab.run(())
            >
                "+"
            </button>
        </div>
    }
}
