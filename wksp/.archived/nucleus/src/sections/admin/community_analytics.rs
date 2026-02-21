//! Admin: Community analytics dashboard

use leptos::prelude::*;

#[component]
pub fn CommunityAnalyticsPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Community Analytics"</h1>
                    <p class="mt-1 text-slate-400">"Engagement metrics, growth trends, and member activity."</p>
                </div>
                <a href="/admin/community" class="text-sm text-slate-400 hover:text-white transition-colors font-mono">"\u{2190} Community Admin"</a>
            </div>

            <div class="mt-8 grid gap-4 md:grid-cols-2 lg:grid-cols-4">
                <AnalyticCard label="Active Members" value="1,247" change="+12%" positive=true />
                <AnalyticCard label="Posts This Week" value="384" change="+8%" positive=true />
                <AnalyticCard label="Avg. Engagement" value="67%" change="-2%" positive=false />
                <AnalyticCard label="New Signups" value="89" change="+23%" positive=true />
            </div>

            <div class="mt-8 grid gap-6 lg:grid-cols-2">
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                    <h3 class="text-sm font-bold text-white font-mono uppercase tracking-widest mb-4">"Top Circles by Activity"</h3>
                    <div class="space-y-3">
                        {[("Signal Detection", 142u32, 890u32), ("Regulatory EU", 98, 650), ("Career Development", 76, 420), ("ICSR Processing", 54, 310)]
                            .into_iter().map(|(name, posts, members)| view! {
                                <div class="flex items-center justify-between py-2 border-b border-slate-800 last:border-0">
                                    <span class="text-sm text-white font-medium">{name}</span>
                                    <div class="flex gap-4 text-xs text-slate-500 font-mono">
                                        <span>{posts}" posts"</span>
                                        <span>{members}" members"</span>
                                    </div>
                                </div>
                            }).collect::<Vec<_>>()}
                    </div>
                </div>

                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                    <h3 class="text-sm font-bold text-white font-mono uppercase tracking-widest mb-4">"Content Distribution"</h3>
                    <div class="space-y-3">
                        {[("Discussion Posts", 45u32), ("Questions", 25), ("Resources Shared", 18), ("Announcements", 12)]
                            .into_iter().map(|(label, pct)| view! {
                                <div class="space-y-1">
                                    <div class="flex justify-between text-xs">
                                        <span class="text-slate-300">{label}</span>
                                        <span class="text-slate-500 font-mono">{pct}"%"</span>
                                    </div>
                                    <div class="h-1.5 w-full bg-slate-800 rounded-full">
                                        <div class="h-full bg-cyan-500 rounded-full" style=format!("width: {pct}%")></div>
                                    </div>
                                </div>
                            }).collect::<Vec<_>>()}
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn AnalyticCard(
    label: &'static str,
    value: &'static str,
    change: &'static str,
    positive: bool,
) -> impl IntoView {
    let change_cls = if positive {
        "text-emerald-400"
    } else {
        "text-red-400"
    };
    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5">
            <p class="text-[9px] font-bold text-slate-500 uppercase tracking-widest font-mono">{label}</p>
            <p class="text-2xl font-black text-white font-mono mt-2">{value}</p>
            <p class=format!("text-xs font-bold font-mono mt-1 {change_cls}")>{change}" vs last week"</p>
        </div>
    }
}
