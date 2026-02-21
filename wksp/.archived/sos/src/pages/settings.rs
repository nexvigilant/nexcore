/// Settings page — ∂ Boundary (config edge) + pi Persistence (local storage)
/// API URL, auth token, theme configuration
use leptos::prelude::*;

use crate::api;

#[component]
pub fn SettingsPage() -> impl IntoView {
    let (api_url, set_api_url) = signal(api::api_base_url());
    let (auth_token, set_auth_token_val) = signal(api::auth_token().unwrap_or_default());
    let (saved, set_saved) = signal(false);

    let save = move |_| {
        api::set_api_base_url(&api_url.get());
        let token = auth_token.get();
        if token.is_empty() {
            api::clear_auth_token();
        } else {
            api::set_auth_token(&token);
        }
        set_saved.set(true);

        // Clear "saved" after 2 seconds
        wasm_bindgen_futures::spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(2_000).await;
            set_saved.set(false);
        });
    };

    view! {
        <div class="page">
            <header class="page-header">
                <h1 class="page-title">"Settings"</h1>
                <p class="page-subtitle">"Configuration"</p>
            </header>

            <div class="settings-form">
                <div class="input-group">
                    <label class="input-label">"API Base URL"</label>
                    <input
                        class="input-field"
                        type="url"
                        placeholder="http://localhost:3030"
                        prop:value=api_url
                        on:input=move |ev| set_api_url.set(event_target_value(&ev))
                    />
                    <p class="input-hint">"nexcore-api endpoint (no trailing slash)"</p>
                </div>

                <div class="input-group">
                    <label class="input-label">"Auth Token"</label>
                    <input
                        class="input-field"
                        type="password"
                        placeholder="Optional API key"
                        prop:value=auth_token
                        on:input=move |ev| set_auth_token_val.set(event_target_value(&ev))
                    />
                    <p class="input-hint">"Leave empty for unauthenticated access"</p>
                </div>

                <button class="btn-primary" on:click=save>
                    {move || if saved.get() { "Saved!" } else { "Save Settings" }}
                </button>
            </div>

            <section class="info-section">
                <h2 class="section-title">"About"</h2>
                <div class="about-grid">
                    <div class="about-row">
                        <span class="about-label">"App"</span>
                        <span class="about-value">"SOS v0.1.0"</span>
                    </div>
                    <div class="about-row">
                        <span class="about-label">"Stack"</span>
                        <span class="about-value">"Leptos + WASM + Capacitor"</span>
                    </div>
                    <div class="about-row">
                        <span class="about-label">"Backend"</span>
                        <span class="about-value">"nexcore-api (Rust/Axum)"</span>
                    </div>
                    <div class="about-row">
                        <span class="about-label">"Entity"</span>
                        <span class="about-value">"NexVigilant, LLC"</span>
                    </div>
                </div>
            </section>
        </div>
    }
}

fn event_target_value(ev: &leptos::ev::Event) -> String {
    use wasm_bindgen::JsCast;
    let target = ev.target().expect("event target");
    target
        .unchecked_into::<web_sys::HtmlInputElement>()
        .value()
}
