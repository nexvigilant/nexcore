//! Admin: Research tools and data exploration

use leptos::prelude::*;

#[component]
pub fn ResearchPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white">"Research Tools"</h1>
            <p class="mt-1 text-slate-400">"Admin tools for data exploration, FAERS analysis, and content research."</p>

            <div class="mt-8 grid gap-4 md:grid-cols-2">
                <ToolCard
                    title="FAERS Query"
                    desc="Run queries against the FDA Adverse Event Reporting System database."
                    status="Available"
                />
                <ToolCard
                    title="Signal Sandbox"
                    desc="Test signal detection algorithms with custom datasets."
                    status="Available"
                />
                <ToolCard
                    title="Content Generator"
                    desc="AI-assisted educational content generation for courses."
                    status="Coming Soon"
                />
                <ToolCard
                    title="Literature Review"
                    desc="Search and aggregate PV literature for course development."
                    status="Coming Soon"
                />
            </div>
        </div>
    }
}

#[component]
fn ToolCard(title: &'static str, desc: &'static str, status: &'static str) -> impl IntoView {
    let status_class = match status {
        "Available" => "text-emerald-400 bg-emerald-500/10",
        _ => "text-slate-400 bg-slate-800",
    };
    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5 hover:border-slate-700 transition-colors">
            <div class="flex items-center justify-between">
                <h3 class="font-semibold text-white">{title}</h3>
                <span class=format!("rounded-full px-2.5 py-0.5 text-xs font-medium {status_class}")>{status}</span>
            </div>
            <p class="mt-2 text-sm text-slate-400">{desc}</p>
        </div>
    }
}
