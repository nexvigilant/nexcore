use crate::components::card::{Card, CardLoading};
use crate::components::metric::Metric;
use crate::components::status_badge::{Status, StatusBadge};
use leptos::prelude::*;

/// Dashboard page — system overview with health, Guardian, Vigil status
/// Tier: T3 (full domain composite)
#[component]
pub fn DashboardPage() -> impl IntoView {
    let health_status = RwSignal::new(Status::Unknown);
    let guardian_state = RwSignal::new(String::from("Loading..."));
    let guardian_iteration = RwSignal::new(String::from("\u{2014}"));
    let vigil_state = RwSignal::new(String::from("Loading..."));
    let llm_calls = RwSignal::new(String::from("\u{2014}"));
    let llm_tokens = RwSignal::new(String::from("\u{2014}"));

    view! {
        <div class="page dashboard">
            <h1 class="page-title">"NCOS Dashboard"</h1>

            <div class="dashboard-grid">
                <Card title="System Health">
                    <StatusBadge status=Signal::derive(move || health_status.get()) label="API"/>
                    <p class="card-hint">"NexCore API status"</p>
                </Card>

                <Card title="Guardian">
                    <Metric label="State" value=Signal::derive(move || guardian_state.get())/>
                    <Metric label="Iteration" value=Signal::derive(move || guardian_iteration.get())/>
                </Card>

                <Card title="Vigil">
                    <Metric label="Status" value=Signal::derive(move || vigil_state.get())/>
                </Card>

                <Card title="LLM Usage">
                    <Metric label="Calls" value=Signal::derive(move || llm_calls.get())/>
                    <Metric label="Tokens" value=Signal::derive(move || llm_tokens.get())/>
                </Card>
            </div>

            <div class="quick-actions">
                <h2 class="section-title">"Quick Actions"</h2>
                <div class="action-grid">
                    <button class="action-btn">"Run Signal Check"</button>
                    <button class="action-btn">"Guardian Tick"</button>
                    <button class="action-btn">"New Session"</button>
                    <button class="action-btn">"System Health"</button>
                </div>
            </div>

            <h2 class="section-title">"Recent Activity"</h2>
            <CardLoading/>
        </div>
    }
}
