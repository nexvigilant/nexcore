/// Guardian live feed — ν Frequency (real-time) + ς State (alert lifecycle)
/// WebSocket connection to Guardian alert stream
use leptos::prelude::*;

use crate::api::guardian::{self, GuardianAlert};
use crate::components::alert_card::AlertCard;

#[component]
pub fn GuardianPage() -> impl IntoView {
    let (alerts, set_alerts) = signal(Vec::<GuardianAlert>::new());
    let (connected, set_connected) = signal(false);

    let connect = move |_| {
        set_connected.set(true);
        set_alerts.set(Vec::new());

        guardian::connect_ws(move |alert| {
            set_alerts.update(|list| {
                list.insert(0, alert); // newest first
                if list.len() > 50 {
                    list.truncate(50); // cap at 50 visible
                }
            });
        });
    };

    view! {
        <div class="page">
            <header class="page-header">
                <h1 class="page-title">"Guardian Feed"</h1>
                <p class="page-subtitle">"Real-time Alert Stream"</p>
            </header>

            <div class="ws-controls">
                <button
                    class="btn-primary"
                    on:click=connect
                    disabled=connected
                >
                    {move || if connected.get() { "Connected" } else { "Connect WebSocket" }}
                </button>
                <div class="ws-status">
                    <span class=move || if connected.get() { "status-dot connected" } else { "status-dot disconnected" }></span>
                    <span>{move || if connected.get() { "Live" } else { "Disconnected" }}</span>
                </div>
            </div>

            <div class="alert-list">
                {move || {
                    let current = alerts.get();
                    if current.is_empty() {
                        view! {
                            <div class="empty-state">
                                <p>"No alerts yet"</p>
                                <p class="empty-hint">
                                    {move || if connected.get() {
                                        "Listening for Guardian alerts..."
                                    } else {
                                        "Connect to start receiving alerts"
                                    }}
                                </p>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="alert-feed">
                                {current.iter().map(|a| {
                                    view! { <AlertCard alert=a.clone() /> }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}
