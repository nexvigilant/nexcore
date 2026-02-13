//! Discover — trending topics and recommended content

use leptos::prelude::*;

#[component]
pub fn DiscoverPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white">"Discover"</h1>
            <p class="mt-2 text-slate-400">"Explore trending topics and recommended content"</p>

            <div class="mt-8">
                <h2 class="text-xl font-semibold text-white">"Trending Topics"</h2>
                <div class="mt-4 flex flex-wrap gap-3">
                    <TopicTag label="Signal Detection" count=45/>
                    <TopicTag label="ICH E2B(R3)" count=32/>
                    <TopicTag label="AI in PV" count=28/>
                    <TopicTag label="Career Growth" count=67/>
                    <TopicTag label="Causality Assessment" count=19/>
                    <TopicTag label="PBRER" count=15/>
                </div>
            </div>

            <div class="mt-12">
                <h2 class="text-xl font-semibold text-white">"Recommended Circles"</h2>
                <div class="mt-4 grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                    <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5">
                        <h3 class="font-semibold text-white">"AI & Automation in PV"</h3>
                        <p class="mt-1 text-sm text-slate-400">"Exploring how technology is transforming drug safety"</p>
                        <p class="mt-2 text-xs text-slate-500">"89 members"</p>
                    </div>
                    <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5">
                        <h3 class="font-semibold text-white">"PV Job Market"</h3>
                        <p class="mt-1 text-sm text-slate-400">"Opportunities, trends, and salary insights"</p>
                        <p class="mt-2 text-xs text-slate-500">"267 members"</p>
                    </div>
                    <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5">
                        <h3 class="font-semibold text-white">"Patient Safety Advocacy"</h3>
                        <p class="mt-1 text-sm text-slate-400">"Independence, transparency, accountability"</p>
                        <p class="mt-2 text-xs text-slate-500">"134 members"</p>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn TopicTag(label: &'static str, count: u32) -> impl IntoView {
    view! {
        <button class="rounded-full border border-slate-700 px-4 py-2 text-sm text-slate-300 hover:border-cyan-500/50 hover:text-cyan-400 transition-colors">
            {label}
            <span class="ml-2 text-xs text-slate-500">{format!("{count}")}</span>
        </button>
    }
}
