//! Admin: Intelligence sub-pages — data sources, analytics pipeline

use leptos::prelude::*;

/* ------------------------------------------------------------------ */
/*  Data Sources                                                       */
/* ------------------------------------------------------------------ */

#[component]
pub fn DataSourcesPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Data Sources"</h1>
            <p class="mt-1 text-slate-400">"Configure external safety databases and literature feeds."</p>

            <div class="mt-8 space-y-4">
                <SourceItem name="EudraVigilance" status="Connected" type_="EMA Gateway"/>
                <SourceItem name="FAERS" status="Syncing" type_="FDA OpenData"/>
                <SourceItem name="PubMed Central" status="Connected" type_="Literature Feed"/>
                <SourceItem name="VigiBase" status="Error" type_="WHO Database"/>
            </div>
        </div>
    }
}

#[component]
fn SourceItem(name: &'static str, status: &'static str, type_: &'static str) -> impl IntoView {
    let status_cls = match status {
        "Connected" => "text-emerald-400 bg-emerald-500/10",
        "Syncing" => "text-cyan-400 bg-cyan-500/10",
        "Error" => "text-red-400 bg-red-500/10",
        _ => "text-slate-500 bg-slate-800",
    };
    view! {
        <div class="flex items-center justify-between rounded-xl border border-slate-800 bg-slate-900/50 p-5">
            <div>
                <h3 class="font-bold text-white">{name}</h3>
                <p class="text-xs text-slate-500">{type_}</p>
            </div>
            <div class="flex items-center gap-4">
                <span class=format!("rounded-full px-3 py-1 text-[10px] font-bold uppercase {status_cls}")>{status}</span>
                <button class="text-xs text-slate-500 hover:text-white transition-colors">"CONFIGURE"</button>
            </div>
        </div>
    }
}

/* ------------------------------------------------------------------ */
/*  Signal Pipeline                                                    */
/* ------------------------------------------------------------------ */

#[component]
pub fn SignalPipelinePage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Signal Pipeline"</h1>
            <p class="mt-1 text-slate-400">"Monitor automated signal detection and evaluation workflows."</p>

            <div class="mt-8 grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                <PipelineCard stage="Ingestion" progress=100 status="Done"/>
                <PipelineCard stage="Normalization" progress=100 status="Done"/>
                <PipelineCard stage="MedDRA Coding" progress=85 status="In Progress"/>
                <PipelineCard stage="Statistical Mining" progress=0 status="Queued"/>
                <PipelineCard stage="Signal Validation" progress=0 status="Pending"/>
            </div>
        </div>
    }
}

#[component]
fn PipelineCard(stage: &'static str, progress: u32, status: &'static str) -> impl IntoView {
    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
            <div class="flex items-center justify-between mb-4">
                <h3 class="font-bold text-white">{stage}</h3>
                <span class="text-[10px] font-bold text-slate-500 uppercase">{status}</span>
            </div>
            <div class="h-1.5 w-full rounded-full bg-slate-800">
                <div class="h-full rounded-full bg-cyan-500" style=format!("width: {progress}%")></div>
            </div>
            <p class="mt-2 text-right text-[10px] font-mono text-slate-600">{progress}"%"</p>
        </div>
    }
}
