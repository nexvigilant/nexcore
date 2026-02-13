use crate::auth::use_auth;
use crate::components::card::Card;
use crate::components::input::TextInput;
use leptos::prelude::*;

/// Settings page — API URL, auth, theme, offline config
/// Tier: T2-C (State + Persistence + Boundary)
#[component]
pub fn SettingsPage() -> impl IntoView {
    let auth = use_auth();

    // Editable copies — sync to auth context on save
    let api_url = RwSignal::new(auth.api_url.get_untracked());
    let api_key = RwSignal::new(auth.api_key.get_untracked());
    let dark_mode = RwSignal::new(false);
    let save_msg = RwSignal::new(String::new());

    view! {
        <div class="page settings">
            <h1 class="page-title">"Settings"</h1>

            <Card title="API Connection">
                <TextInput
                    label="API URL"
                    value=api_url
                    placeholder="https://nexcore-api-xxx.run.app"
                    input_type="url"
                />
                <TextInput
                    label="API Key"
                    value=api_key
                    placeholder="X-API-Key header value"
                    input_type="password"
                />
                <button class="btn-primary"
                    on:click=move |_| {
                        auth.api_url.set(api_url.get());
                        auth.api_key.set(api_key.get());
                        save_msg.set("Settings saved to session".to_string());
                    }
                >"Save Connection"</button>
            </Card>

            <Card title="Appearance">
                <div class="toggle-row">
                    <span>"Dark Mode"</span>
                    <button
                        class=move || if dark_mode.get() { "toggle active" } else { "toggle" }
                        on:click=move |_| dark_mode.update(|v| *v = !*v)
                    >
                        {move || if dark_mode.get() { "ON" } else { "OFF" }}
                    </button>
                </div>
            </Card>

            <Card title="About">
                <p>"NCOS v0.1.0"</p>
                <p class="card-hint">"NexCore Operating System \u{2014} Mobile PWA"</p>
                <p class="card-hint">"Built with Leptos 0.7 + Axum"</p>
            </Card>

            <Show when=move || !save_msg.get().is_empty()>
                <div class="success-banner">{move || save_msg.get()}</div>
            </Show>
        </div>
    }
}
