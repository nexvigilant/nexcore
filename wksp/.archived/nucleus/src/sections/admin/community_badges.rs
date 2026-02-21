//! Admin: Community badges management

use leptos::prelude::*;

#[component]
pub fn CommunityBadgesPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold text-white">"Badge Management"</h1>
                    <p class="mt-1 text-slate-400">"Create and assign community achievement badges."</p>
                </div>
                <button class="rounded-lg bg-cyan-500 px-4 py-2 text-sm font-medium text-white hover:bg-cyan-400 transition-colors">
                    "+ New Badge"
                </button>
            </div>

            <div class="mt-8 grid gap-4 md:grid-cols-4">
                <BadgeCard name="First Post" desc="Published first community post" awarded=0/>
                <BadgeCard name="Signal Spotter" desc="Completed signal detection course" awarded=0/>
                <BadgeCard name="Mentor" desc="Accepted as a community mentor" awarded=0/>
                <BadgeCard name="Top Contributor" desc="100+ community contributions" awarded=0/>
            </div>
        </div>
    }
}

#[component]
fn BadgeCard(name: &'static str, desc: &'static str, awarded: u32) -> impl IntoView {
    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5">
            <h3 class="font-semibold text-white">{name}</h3>
            <p class="mt-1 text-xs text-slate-400">{desc}</p>
            <p class="mt-3 text-xs text-slate-500">{format!("{awarded} awarded")}</p>
        </div>
    }
}
