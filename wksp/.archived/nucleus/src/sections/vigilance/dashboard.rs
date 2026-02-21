//! Vigilance dashboard — system overview with health, Guardian, Patrol status

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DashboardMetrics {
    pub health_status: String,
    pub guardian_state: String,
    pub guardian_iteration: u64,
    pub vigil_status: String,
    pub llm_calls: u64,
    pub llm_tokens: u64,
}

/// Server function to fetch dashboard metrics from nexcore-api
#[server(GetDashboardMetrics, "/api")]
pub async fn get_dashboard_metrics() -> Result<DashboardMetrics, ServerFnError> {
    use crate::api_client::server::ApiClient;

    let api_url = std::env::var("NEXCORE_API_URL").unwrap_or_else(|_| "http://localhost:3030".to_string());
    let api_key = std::env::var("NEXCORE_API_KEY").ok();
    let client = ApiClient::new(api_url, api_key);

    let health = client.health().await.unwrap_or_default();
    let guardian = client.guardian_status().await.unwrap_or_default();
    let vigil = client.vigil_status().await.unwrap_or_default();
    let llm = client.llm_stats().await.unwrap_or_default();

    Ok(DashboardMetrics {
        health_status: health.status,
        guardian_state: guardian.state,
        guardian_iteration: guardian.iteration,
        vigil_status: vigil.status,
        llm_calls: llm.total_calls,
        llm_tokens: llm.total_tokens,
    })
}

#[component]
pub fn DashboardPage() -> impl IntoView {
    let metrics = Resource::new(|| (), |_| get_dashboard_metrics());

    view! {
        <div class="mx-auto max-w-7xl px-4 py-12">
            <header class="flex justify-between items-end mb-12 border-b border-slate-800/50 pb-8">
                <div>
                    <div class="flex items-center gap-3 mb-2">
                        <span class="h-1.5 w-10 bg-cyan-500 rounded-full"></span>
                        <span class="text-xs font-mono font-bold text-cyan-500 uppercase tracking-widest">"System Status"</span>
                    </div>
                    <h1 class="text-5xl font-black text-white font-mono tracking-tighter uppercase">"VIGILANCE"</h1>
                </div>
                <button 
                    on:click=move |_| metrics.refetch()
                    class="p-3 rounded-full glass-panel hover:glow-border-cyan transition-all text-slate-400 hover:text-cyan-400"
                    title="Synchronize"
                >
                    <RefreshIcon />
                </button>
            </header>

            <div class="grid gap-6 sm:grid-cols-2 lg:grid-cols-4">
                <DashboardMetricsView metrics=metrics />
            </div>

            <QuickActions />
            <RecentActivity />
        </div>
    }
}

#[component]
fn DashboardMetricsView(metrics: Resource<Result<DashboardMetrics, ServerFnError>>) -> impl IntoView {
    view! {
        <Suspense fallback=|| view! { <LoadingGrid /> }>
            {move || metrics.get().map(|result| match result {
                Ok(m) => view! { <StatusCards metrics=m /> }.into_any(),
                Err(e) => view! { <ErrorCard message=e.to_string() /> }.into_any()
            })}
        </Suspense>
    }
}

#[component]
fn LoadingGrid() -> impl IntoView {
    view! {
        <div class="col-span-full grid gap-6 sm:grid-cols-2 lg:grid-cols-4 w-full">
            {(0..4).map(|_| view! { 
                <div class="rounded-2xl glass-panel p-6 h-36 animate-pulse">
                    <div class="h-2 w-16 bg-slate-800 rounded mb-4"></div>
                    <div class="h-8 w-24 bg-slate-800 rounded"></div>
                </div>
            }).collect_view()}
        </div>
    }
}

#[component]
fn ErrorCard(message: String) -> impl IntoView {
    view! {
        <div class="col-span-full rounded-2xl border border-red-500/20 bg-red-500/5 p-8 text-red-400 backdrop-blur-md">
            <h3 class="font-mono font-bold uppercase tracking-widest text-xs mb-2">"Critical Interrupt"</h3>
            <p class="text-sm font-mono">{message}</p>
        </div>
    }
}

#[component]
fn StatusCards(metrics: DashboardMetrics) -> impl IntoView {
    view! {
        <HealthCard status=metrics.health_status />
        <GuardianCard state=metrics.guardian_state iteration=metrics.guardian_iteration />
        <PatrolCard status=metrics.vigil_status />
        <LlmUsageCard calls=metrics.llm_calls tokens=metrics.llm_tokens />
    }
}

#[component]
fn HealthCard(status: String) -> impl IntoView {
    let color = if status == "UP" || status == "healthy" { "bg-cyan-500" } else { "bg-red-500" };
    let border = if status == "UP" || status == "healthy" { "glow-border-cyan" } else { "border-red-500/30" };
    
    view! {
        <StatCard title="Core Integrity" value=status color=color border=border subtitle="Neural Link Established" />
    }
}

#[component]
fn PatrolCard(status: String) -> impl IntoView {
    let color = if status == "active" || status == "running" { "bg-cyan-500" } else { "bg-slate-700" };
    let border = if status == "active" || status == "running" { "glow-border-cyan" } else { "" };
    
    view! {
        <StatCard title="Patrol Orbit" value=status color=color border=border subtitle="Perimeter Secured" />
    }
}

#[component]
fn GuardianCard(state: String, iteration: u64) -> impl IntoView {
    view! {
        <div class="rounded-2xl glass-panel p-6 hover:glow-border-cyan transition-all group">
            <h3 class="text-[10px] font-mono font-bold text-slate-500 uppercase tracking-[0.2em] mb-4">"Guardian"</h3>
            <div class="space-y-3">
                <StatRow label="Cycle" value=state.to_uppercase() />
                <StatRow label="Pulse" value=format!("I-{}", iteration) />
            </div>
        </div>
    }
}

#[component]
fn LlmUsageCard(calls: u64, tokens: u64) -> impl IntoView {
    view! {
        <div class="rounded-2xl glass-panel p-6 hover:glow-border-cyan transition-all group">
            <h3 class="text-[10px] font-mono font-bold text-slate-500 uppercase tracking-[0.2em] mb-4">"Cognitive Load"</h3>
            <div class="space-y-3">
                <StatRow label="Invocations" value=calls.to_string() />
                <StatRow label="Quantum Flux" value=format!("{}T", tokens / 1000) />
            </div>
        </div>
    }
}

#[component]
fn StatCard(title: &'static str, value: String, color: &'static str, border: &'static str, subtitle: &'static str) -> impl IntoView {
    view! {
        <div class=format!("rounded-2xl glass-panel p-6 transition-all {}", border)>
            <h3 class="text-[10px] font-mono font-bold text-slate-500 uppercase tracking-[0.2em] mb-4">{title}</h3>
            <div class="flex items-center gap-3">
                <span class=format!("h-2 w-2 rounded-full animate-pulse {}", color)></span>
                <span class="text-2xl font-black text-white font-mono tracking-tighter uppercase">{value}</span>
            </div>
            <p class="mt-4 text-[9px] font-mono text-slate-600 font-bold uppercase tracking-widest">{subtitle}</p>
        </div>
    }
}

#[component]
fn StatRow(label: &'static str, value: String) -> impl IntoView {
    view! {
        <div class="flex justify-between items-center text-[11px] font-mono font-bold">
            <span class="text-slate-500 uppercase tracking-wider">{label}</span>
            <span class="text-white bg-slate-800/50 px-2 py-0.5 rounded">{value}</span>
        </div>
    }
}

#[component]
fn QuickActions() -> impl IntoView {
    view! {
        <div class="mt-16">
            <h2 class="text-xs font-mono font-bold text-slate-500 uppercase tracking-[0.3em] mb-6">"// DIRECTIVES"</h2>
            <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
                <ActionLink href="/vigilance/signals" label="SCAN SIGNALS" />
                <ActionLink href="/vigilance/guardian" label="TICK GUARDIAN" />
                <ActionLink href="/vigilance/brain" label="INITIALIZE BRAIN" />
                <ActionLink href="/vigilance/causality" label="ASSESS CAUSALITY" />
            </div>
        </div>
    }
}

#[component]
fn ActionLink(href: &'static str, label: &'static str) -> impl IntoView {
    view! {
        <a href=href class="rounded-xl glass-panel py-4 px-6 text-center text-xs font-mono font-bold text-amber-500 hover:glow-border-amber hover:text-amber-400 transition-all group flex items-center justify-between">
            <span>{label}</span>
            <span class="text-slate-700 group-hover:text-amber-500 transition-all font-mono">" >>"</span>
        </a>
    }
}

#[component]
fn RecentActivity() -> impl IntoView {
    view! {
        <div class="mt-16">
            <h2 class="text-xs font-mono font-bold text-slate-500 uppercase tracking-[0.3em] mb-6">"// TEMPORAL STREAM"</h2>
            <div class="rounded-2xl glass-panel p-12 text-center relative overflow-hidden group">
                <div class="absolute inset-0 bg-gradient-to-r from-transparent via-cyan-500/5 to-transparent -translate-x-full group-hover:translate-x-full transition-transform duration-1000"></div>
                <p class="text-slate-500 font-mono text-sm tracking-widest animate-pulse">"LISTENING FOR QUANTUM EVENTS..."</p>
            </div>
        </div>
    }
}

#[component]
fn RefreshIcon() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"></path>
            <path d="M21 3v5h-5"></path>
            <path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16"></path>
            <path d="M3 21v-5h5"></path>
        </svg>
    }
}