//! Community search — find posts, members, and circles

use leptos::prelude::*;

#[component]
pub fn SearchPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-4xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white">"Search Community"</h1>

            <div class="mt-6">
                <input
                    type="text"
                    placeholder="Search posts, members, circles..."
                    class="w-full rounded-lg border border-slate-700 bg-slate-800 px-4 py-3 text-white placeholder-slate-500 focus:border-cyan-500 focus:outline-none"
                />
            </div>

            <div class="mt-4 flex gap-2">
                <FilterChip label="All" active=true/>
                <FilterChip label="Posts" active=false/>
                <FilterChip label="Members" active=false/>
                <FilterChip label="Circles" active=false/>
            </div>

            <div class="mt-8 text-center text-sm text-slate-500">
                "Enter a search term to find content across the community."
            </div>
        </div>
    }
}

#[component]
fn FilterChip(label: &'static str, active: bool) -> impl IntoView {
    let class = if active {
        "rounded-full bg-cyan-500/20 px-4 py-1.5 text-sm font-medium text-cyan-400"
    } else {
        "rounded-full bg-slate-800 px-4 py-1.5 text-sm font-medium text-slate-400 hover:text-white transition-colors cursor-pointer"
    };
    view! { <button class=class>{label}</button> }
}
