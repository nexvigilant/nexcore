//! Admin: Community management — moderation, circles, badges, users

use leptos::prelude::*;

#[component]
pub fn CommunityAdminPage() -> impl IntoView {
    let active_tab = RwSignal::new("moderation");

    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold text-white">"Community Admin"</h1>
                    <p class="mt-1 text-slate-400">"Moderation, circles, badges, and user management"</p>
                </div>
                <a href="/admin" class="text-sm text-slate-400 hover:text-white transition-colors">"\u{2190} Dashboard"</a>
            </div>

            <div class="mt-6 flex gap-4 overflow-x-auto border-b border-slate-800 pb-4">
                {["moderation", "circles", "badges", "reports", "analytics"].into_iter().map(|tab| {
                    view! {
                        <button
                            class=move || { if active_tab.get() == tab {
                                "whitespace-nowrap text-sm font-medium text-amber-400 border-b-2 border-amber-400 pb-1"
                            } else {
                                "whitespace-nowrap text-sm text-slate-400 hover:text-white transition-colors"
                            }}
                            on:click=move |_| active_tab.set(tab)
                        >{tab.to_uppercase()}</button>
                    }
                }).collect::<Vec<_>>()}
            </div>

            <Show when=move || active_tab.get() == "moderation">
                <div class="mt-6">
                    <h2 class="text-lg font-semibold text-white">"Content Moderation"</h2>
                    <div class="mt-4 grid gap-4 sm:grid-cols-3">
                        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5 text-center">
                            <p class="text-xs text-slate-500">"Pending Review"</p>
                            <p class="mt-2 text-3xl font-bold text-amber-400">"0"</p>
                        </div>
                        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5 text-center">
                            <p class="text-xs text-slate-500">"Flagged Posts"</p>
                            <p class="mt-2 text-3xl font-bold text-red-400">"0"</p>
                        </div>
                        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5 text-center">
                            <p class="text-xs text-slate-500">"Approved Today"</p>
                            <p class="mt-2 text-3xl font-bold text-emerald-400">"0"</p>
                        </div>
                    </div>
                    <div class="mt-6 rounded-xl border border-slate-800 bg-slate-900/50 p-8 text-center">
                        <p class="text-slate-400">"No items require moderation"</p>
                    </div>
                </div>
            </Show>

            <Show when=move || active_tab.get() == "circles">
                <div class="mt-6">
                    <div class="flex items-center justify-between">
                        <h2 class="text-lg font-semibold text-white">"Circles"</h2>
                        <button class="rounded-lg bg-amber-600 px-4 py-2 text-sm font-medium text-white hover:bg-amber-500">"+ Create Circle"</button>
                    </div>
                    <div class="mt-4 space-y-3">
                        {["AI & Automation in PV", "Signal Detection Methods", "Career Growth",
                          "Patient Safety Advocacy", "Regulatory Updates", "PV Job Market"].into_iter().map(|circle| {
                            view! {
                                <div class="flex items-center justify-between rounded-lg border border-slate-800 bg-slate-900/30 px-4 py-3">
                                    <div>
                                        <p class="font-medium text-white">{circle}</p>
                                        <p class="text-xs text-slate-500">"0 members \u{00B7} 0 posts"</p>
                                    </div>
                                    <div class="flex gap-2">
                                        <button class="text-xs text-amber-400">"Edit"</button>
                                        <button class="text-xs text-red-400">"Archive"</button>
                                    </div>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>
            </Show>

            <Show when=move || active_tab.get() == "badges">
                <div class="mt-6">
                    <div class="flex items-center justify-between">
                        <h2 class="text-lg font-semibold text-white">"Achievement Badges"</h2>
                        <button class="rounded-lg bg-amber-600 px-4 py-2 text-sm font-medium text-white hover:bg-amber-500">"+ New Badge"</button>
                    </div>
                    <div class="mt-4 grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
                        {["First Post", "Signal Spotter", "Mentor", "Course Completer",
                          "Active Contributor", "Domain Expert"].into_iter().map(|badge| {
                            view! {
                                <div class="rounded-lg border border-slate-800 bg-slate-900/30 px-4 py-3">
                                    <p class="font-medium text-white">{badge}</p>
                                    <p class="text-xs text-slate-500">"0 awarded"</p>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>
            </Show>

            <Show when=move || active_tab.get() == "reports">
                <div class="mt-6 rounded-xl border border-slate-800 bg-slate-900/50 p-8 text-center">
                    <p class="text-slate-400">"No community reports"</p>
                    <p class="mt-2 text-sm text-slate-500">"User reports and moderation actions will appear here."</p>
                </div>
            </Show>

            <Show when=move || active_tab.get() == "analytics">
                <div class="mt-6 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
                    <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5 text-center">
                        <p class="text-xs text-slate-500">"Total Members"</p>
                        <p class="mt-2 text-2xl font-bold text-cyan-400">"0"</p>
                    </div>
                    <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5 text-center">
                        <p class="text-xs text-slate-500">"Posts This Month"</p>
                        <p class="mt-2 text-2xl font-bold text-violet-400">"0"</p>
                    </div>
                    <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5 text-center">
                        <p class="text-xs text-slate-500">"Active Circles"</p>
                        <p class="mt-2 text-2xl font-bold text-amber-400">"6"</p>
                    </div>
                    <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5 text-center">
                        <p class="text-xs text-slate-500">"Engagement Rate"</p>
                        <p class="mt-2 text-2xl font-bold text-emerald-400">"0%"</p>
                    </div>
                </div>
            </Show>
        </div>
    }
}
