//! Navigation bar component
//!
//! Browser navigation controls: back, forward, refresh, address bar.
//!
//! Tier: T3 (Domain-specific UI component)

use leptos::prelude::*;

/// Navigation bar component
#[component]
pub fn NavBar(
    current_url: Signal<String>,
    on_navigate: Callback<String>,
    on_back: Callback<()>,
    on_forward: Callback<()>,
    on_refresh: Callback<()>,
) -> impl IntoView {
    let (input_value, set_input_value) = signal(String::new());

    // Sync input with current URL when it changes externally
    Effect::new(move || {
        set_input_value.set(current_url.get());
    });

    view! {
        <nav class="nav-bar">
            <button
                class="nav-btn back"
                on:click=move |_| on_back.run(())
                title="Back"
            >
                "←"
            </button>
            <button
                class="nav-btn forward"
                on:click=move |_| on_forward.run(())
                title="Forward"
            >
                "→"
            </button>
            <button
                class="nav-btn refresh"
                on:click=move |_| on_refresh.run(())
                title="Refresh"
            >
                "⟳"
            </button>

            <input
                type="text"
                class="address-bar"
                prop:value=move || input_value.get()
                on:input=move |ev| {
                    set_input_value.set(event_target_value(&ev));
                }
                on:keydown=move |ev| {
                    if ev.key() == "Enter" {
                        on_navigate.run(input_value.get());
                    }
                }
                placeholder="Enter URL..."
            />

            <button class="nav-btn menu" title="Menu">"☰"</button>
        </nav>
    }
}
