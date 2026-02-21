//! Assessment hub — central dashboard for all career assessments

use leptos::prelude::*;

#[component]
pub fn AssessmentHubPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white">"Assessment Hub"</h1>
            <p class="mt-2 text-slate-400">"Track your progress across all career assessments in one place."</p>

            <div class="mt-8 grid gap-4 md:grid-cols-3">
                <SummaryCard label="Completed" value="0" total="14" color="emerald"/>
                <SummaryCard label="In Progress" value="0" total="14" color="amber"/>
                <SummaryCard label="Available" value="14" total="14" color="cyan"/>
            </div>

            <div class="mt-8">
                <h2 class="text-lg font-semibold text-white">"Recent Activity"</h2>
                <p class="mt-4 text-sm text-slate-500">"Complete your first assessment to see activity here."</p>
            </div>
        </div>
    }
}

#[component]
fn SummaryCard(label: &'static str, value: &'static str, total: &'static str, color: &'static str) -> impl IntoView {
    let value_class = format!("text-3xl font-bold text-{color}-400");
    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5 text-center">
            <div class=value_class>{value}<span class="text-lg text-slate-600">{format!("/{total}")}</span></div>
            <div class="mt-1 text-sm text-slate-500">{label}</div>
        </div>
    }
}
