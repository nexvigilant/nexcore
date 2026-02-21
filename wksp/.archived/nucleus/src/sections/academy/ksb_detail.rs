//! KSB detail page — individual Knowledge, Skill, or Behavior view

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

#[component]
pub fn KsbDetailPage() -> impl IntoView {
    let params = use_params_map();
    let ksb_id = move || params.get().get("id").unwrap_or_default();

    view! {
        <div class="mx-auto max-w-3xl px-4 py-8">
            <nav class="text-sm text-slate-500">
                <a href="/academy" class="hover:text-cyan-400">"Academy"</a>
                " / "
                <a href="/academy/skills" class="hover:text-cyan-400">"Skills"</a>
                " / "
                <span class="text-slate-300">{ksb_id}</span>
            </nav>

            <div class="mt-6">
                <span class="rounded-full bg-cyan-500/10 px-3 py-1 text-xs font-medium text-cyan-400">"Knowledge"</span>
                <h1 class="mt-3 text-3xl font-bold text-white">{move || format!("KSB: {}", ksb_id())}</h1>
                <p class="mt-2 text-slate-400">"Loading KSB details..."</p>
            </div>

            <div class="mt-8 space-y-6">
                <DetailSection title="Description" content="Detailed description of this Knowledge, Skill, or Behavior component."/>
                <DetailSection title="Domain" content="Domain assignment and related KSBs."/>
                <DetailSection title="Bloom Level" content="Cognitive complexity level (1-6)."/>
                <DetailSection title="Related Courses" content="Courses that develop this KSB."/>
            </div>
        </div>
    }
}

#[component]
fn DetailSection(title: &'static str, content: &'static str) -> impl IntoView {
    view! {
        <div class="rounded-lg border border-slate-800 bg-slate-900/50 p-5">
            <h2 class="font-semibold text-white">{title}</h2>
            <p class="mt-2 text-sm text-slate-400">{content}</p>
        </div>
    }
}
