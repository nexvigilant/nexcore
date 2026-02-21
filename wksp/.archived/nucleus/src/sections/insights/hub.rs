//! Insights hub — intelligence articles and analysis

use leptos::prelude::*;

#[component]
pub fn HubPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white">"Insights"</h1>
            <p class="mt-2 text-slate-400">"Pharmaceutical safety intelligence, analysis, and commentary."</p>

            <div class="mt-6 flex gap-2">
                <button class="rounded-full bg-cyan-500/20 px-4 py-1.5 text-sm font-medium text-cyan-400">"All"</button>
                <button class="rounded-full bg-slate-800 px-4 py-1.5 text-sm font-medium text-slate-400 hover:text-white transition-colors">"Safety Signals"</button>
                <button class="rounded-full bg-slate-800 px-4 py-1.5 text-sm font-medium text-slate-400 hover:text-white transition-colors">"Regulatory"</button>
                <button class="rounded-full bg-slate-800 px-4 py-1.5 text-sm font-medium text-slate-400 hover:text-white transition-colors">"Industry"</button>
            </div>

            <div class="mt-8 text-center text-sm text-slate-500">
                "Insights articles will be published here. Check back soon."
            </div>
        </div>
    }
}
