//! Admin: KSB taxonomy management

use leptos::prelude::*;

#[component]
pub fn AcademyKsbPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white">"KSB Management"</h1>
            <p class="mt-1 text-slate-400">"Manage the Knowledge, Skills, and Behaviors taxonomy across 15 domains."</p>

            <div class="mt-6 grid gap-4 md:grid-cols-3">
                <StatCard label="Total KSBs" value="1,462"/>
                <StatCard label="Domains" value="15"/>
                <StatCard label="EPAs" value="21"/>
            </div>

            <div class="mt-8 text-sm text-slate-500">
                "KSB management interface will support bulk import/export and individual editing."
            </div>
        </div>
    }
}

#[component]
fn StatCard(label: &'static str, value: &'static str) -> impl IntoView {
    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5 text-center">
            <div class="text-2xl font-bold text-white">{value}</div>
            <div class="mt-1 text-xs text-slate-500">{label}</div>
        </div>
    }
}
