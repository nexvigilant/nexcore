//! Admin: Community discovery and recommendation settings

use leptos::prelude::*;

#[component]
pub fn CommunityDiscoveryPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Discovery Settings"</h1>
                    <p class="mt-1 text-slate-400">"Configure how members discover content, circles, and peers."</p>
                </div>
                <a href="/admin/community" class="text-sm text-slate-400 hover:text-white transition-colors font-mono">"\u{2190} Community Admin"</a>
            </div>

            <div class="mt-8 space-y-6">
                <DiscoveryBlock
                    title="For You Algorithm"
                    description="Controls the personalized feed recommendations."
                    items=vec![("Engagement Weight", "0.4"), ("Recency Weight", "0.3"), ("Domain Relevance", "0.2"), ("Network Proximity", "0.1")]
                />
                <DiscoveryBlock
                    title="Circle Suggestions"
                    description="How circles are recommended to new and existing members."
                    items=vec![("Min Members to Suggest", "10"), ("Activity Threshold", "5 posts/week"), ("Max Suggestions", "6")]
                />
                <DiscoveryBlock
                    title="Member Matching"
                    description="Peer matching based on domain expertise and interests."
                    items=vec![("Domain Overlap Min", "2"), ("Experience Level Range", "\u{00b1}2 years"), ("Geographic Preference", "Disabled")]
                />
            </div>
        </div>
    }
}

#[component]
fn DiscoveryBlock(
    title: &'static str,
    description: &'static str,
    items: Vec<(&'static str, &'static str)>,
) -> impl IntoView {
    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
            <div class="flex items-center justify-between mb-4">
                <div>
                    <h3 class="text-lg font-bold text-white">{title}</h3>
                    <p class="text-xs text-slate-500 mt-1">{description}</p>
                </div>
                <button class="rounded-lg border border-slate-700 px-4 py-1.5 text-[10px] font-bold text-slate-400 hover:text-white transition-colors uppercase tracking-widest">"Edit"</button>
            </div>
            <div class="space-y-2">
                {items.into_iter().map(|(label, value)| view! {
                    <div class="flex items-center justify-between py-2 border-b border-slate-800/50 last:border-0">
                        <span class="text-sm text-slate-400">{label}</span>
                        <span class="text-sm text-white font-mono">{value}</span>
                    </div>
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}
