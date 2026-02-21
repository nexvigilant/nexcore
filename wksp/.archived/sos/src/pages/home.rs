/// Dashboard — system health, active alerts
/// Tier: T3 (varsigma State + nu Frequency + Sigma Sum)
use leptos::prelude::*;

use crate::api::guardian;

/// Home/Dashboard page — shows Guardian status and system vitals
#[component]
pub fn HomePage() -> impl IntoView {
    let status = LocalResource::new(|| guardian::get_status());

    view! {
        <div class="page">
            <header class="page-header">
                <h1 class="page-title">"SOS"</h1>
                <p class="page-subtitle">"The Vigilance Machine"</p>
            </header>

            <Suspense fallback=move || view! { <div class="loading">"Loading..."</div> }>
                {move || {
                    status.read().as_ref().map(|result| {
                        match result {
                            Ok(s) => view! {
                                <div class="card-grid">
                                    <div class="stat-card">
                                        <div class="stat-label">"Guardian State"</div>
                                        <div class="stat-value">{s.state.clone()}</div>
                                    </div>
                                    <div class="stat-card">
                                        <div class="stat-label">"Active Alerts"</div>
                                        <div class="stat-value stat-alert">{s.active_alerts}</div>
                                    </div>
                                    <div class="stat-card">
                                        <div class="stat-label">"Uptime"</div>
                                        <div class="stat-value">{format_uptime(s.uptime_seconds)}</div>
                                    </div>
                                    <div class="stat-card">
                                        <div class="stat-label">"Last Tick"</div>
                                        <div class="stat-value stat-small">{s.last_tick.clone()}</div>
                                    </div>
                                </div>
                            }.into_any(),
                            Err(e) => view! {
                                <div class="error-card">
                                    <div class="error-icon">"!"</div>
                                    <div class="error-msg">{e.message.clone()}</div>
                                    <p class="error-hint">"Check Settings for API URL"</p>
                                </div>
                            }.into_any(),
                        }
                    })
                }}
            </Suspense>

            <section class="info-section">
                <h2 class="section-title">"NexVigilant"</h2>
                <p class="section-body">"Empowerment Through Vigilance"</p>
                <div class="primitive-bar">
                    <span class="primitive">"lambda Location"</span>
                    <span class="primitive">"mu Mapping"</span>
                    <span class="primitive">"partial Boundary"</span>
                    <span class="primitive">"varsigma State"</span>
                    <span class="primitive">"nu Frequency"</span>
                    <span class="primitive">"sigma Sequence"</span>
                </div>
            </section>
        </div>
    }
}

fn format_uptime(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    if hours > 0 {
        format!("{hours}h {minutes}m")
    } else {
        format!("{minutes}m")
    }
}
