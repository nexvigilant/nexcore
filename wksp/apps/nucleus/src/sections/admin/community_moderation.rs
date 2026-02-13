//! Admin: Community moderation queue

use leptos::prelude::*;

#[component]
pub fn CommunityModerationPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white">"Community Moderation"</h1>
            <p class="mt-1 text-slate-400">"Review flagged content, manage reports, and enforce community guidelines."</p>

            <div class="mt-6 grid gap-4 md:grid-cols-3">
                <QueueCard label="Pending Reports" count=0 color="amber"/>
                <QueueCard label="Flagged Posts" count=0 color="rose"/>
                <QueueCard label="Resolved Today" count=0 color="emerald"/>
            </div>

            <div class="mt-8 rounded-xl border border-slate-800 bg-slate-900/50 p-8 text-center">
                <p class="text-slate-500">"No items in the moderation queue."</p>
            </div>
        </div>
    }
}

#[component]
fn QueueCard(label: &'static str, count: u32, color: &'static str) -> impl IntoView {
    let value_class = format!("text-2xl font-bold text-{color}-400");
    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5 text-center">
            <div class=value_class>{count}</div>
            <div class="mt-1 text-xs text-slate-500">{label}</div>
        </div>
    }
}
