//! Admin: Insights management — analytics configuration, data sources, dashboards

use leptos::prelude::*;

#[component]
pub fn InsightsAdminPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8 space-y-8">
            <div>
                <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Insights Admin"</h1>
                <p class="mt-1 text-slate-400">"Configure analytics dashboards, data sources, and reporting pipelines"</p>
            </div>

            <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
                <Stat label="Dashboards" value="3" color="text-violet-400"/>
                <Stat label="Data Sources" value="4" color="text-cyan-400"/>
                <Stat label="Reports" value="0" color="text-emerald-400"/>
                <Stat label="Scheduled Jobs" value="0" color="text-amber-400"/>
            </div>

            <div>
                <h2 class="text-sm font-bold uppercase tracking-widest text-slate-500 mb-4">"Dashboards"</h2>
                <div class="space-y-2">
                    <DashboardRow name="Overview" widgets=4 status="Active" refresh="Real-time"/>
                    <DashboardRow name="Trends" widgets=3 status="Active" refresh="Daily"/>
                    <DashboardRow name="Reports" widgets=2 status="Active" refresh="Weekly"/>
                </div>
            </div>

            <div>
                <h2 class="text-sm font-bold uppercase tracking-widest text-slate-500 mb-4">"Data Sources"</h2>
                <div class="space-y-2">
                    <SourceRow name="NexCore API" endpoint="/api/v1" status="Connected"/>
                    <SourceRow name="Firebase Auth" endpoint="Firebase SDK" status="Connected"/>
                    <SourceRow name="FAERS Database" endpoint="openFDA API" status="Available"/>
                    <SourceRow name="MedDRA Dictionary" endpoint="Local index" status="Indexed"/>
                </div>
            </div>

            <div>
                <h2 class="text-sm font-bold uppercase tracking-widest text-slate-500 mb-4">"Report Templates"</h2>
                <div class="grid gap-4 lg:grid-cols-3">
                    <ReportTemplate name="Weekly Safety Summary" format="PDF" schedule="Every Monday"/>
                    <ReportTemplate name="Signal Detection Report" format="PDF/CSV" schedule="Monthly"/>
                    <ReportTemplate name="Platform Usage Metrics" format="Dashboard" schedule="Real-time"/>
                </div>
            </div>
        </div>
    }
}

#[component]
fn Stat(label: &'static str, value: &'static str, color: &'static str) -> impl IntoView {
    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5">
            <p class="text-[10px] font-bold uppercase tracking-widest text-slate-500">{label}</p>
            <p class=format!("mt-2 text-3xl font-bold font-mono {color}")>{value}</p>
        </div>
    }
}

#[component]
fn DashboardRow(
    name: &'static str,
    widgets: u32,
    status: &'static str,
    refresh: &'static str,
) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between rounded-lg border border-slate-800/50 bg-slate-900/30 p-4">
            <div class="flex items-center gap-3">
                <div class="h-2 w-2 rounded-full bg-violet-400"></div>
                <span class="text-sm text-white font-medium">{name}</span>
                <span class="text-xs text-slate-500 font-mono">{format!("{} widgets", widgets)}</span>
            </div>
            <div class="flex items-center gap-4">
                <span class="text-xs text-slate-500">{refresh}</span>
                <span class="rounded bg-emerald-500/10 px-2 py-0.5 text-[10px] text-emerald-400 font-mono uppercase">{status}</span>
            </div>
        </div>
    }
}

#[component]
fn SourceRow(name: &'static str, endpoint: &'static str, status: &'static str) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between rounded-lg border border-slate-800/50 bg-slate-900/30 p-4">
            <div class="flex items-center gap-3">
                <div class="h-2 w-2 rounded-full bg-cyan-400"></div>
                <span class="text-sm text-white font-medium">{name}</span>
                <span class="text-xs text-slate-500 font-mono">{endpoint}</span>
            </div>
            <span class="rounded bg-emerald-500/10 px-2 py-0.5 text-[10px] text-emerald-400 font-mono uppercase">{status}</span>
        </div>
    }
}

#[component]
fn ReportTemplate(
    name: &'static str,
    format: &'static str,
    schedule: &'static str,
) -> impl IntoView {
    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5">
            <h3 class="text-sm font-bold text-white">{name}</h3>
            <div class="mt-3 space-y-1">
                <div class="flex justify-between text-xs">
                    <span class="text-slate-500">"Format"</span>
                    <span class="text-slate-300 font-mono">{format}</span>
                </div>
                <div class="flex justify-between text-xs">
                    <span class="text-slate-500">"Schedule"</span>
                    <span class="text-slate-300 font-mono">{schedule}</span>
                </div>
            </div>
        </div>
    }
}
