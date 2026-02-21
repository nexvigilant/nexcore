//! Community analytics — engagement metrics and growth data

use leptos::prelude::*;

#[component]
pub fn AnalyticsPage() -> impl IntoView {
    let (time_range, set_time_range) = signal("weekly");

    view! {
        <div class="mx-auto max-w-6xl px-4 py-12">
            <div class="flex items-center justify-between mb-12">
                <div>
                    <h1 class="text-4xl font-bold text-white font-mono uppercase tracking-tight">"ANALYTICS"</h1>
                    <p class="mt-2 text-slate-400">"Community engagement metrics and growth insights"</p>
                </div>
                <TimeRangeSelector selected=time_range set_selected=set_time_range />
            </div>

            /* Summary stat cards */
            <div class="grid gap-6 md:grid-cols-2 lg:grid-cols-4 mb-12">
                <StatCard
                    label="POSTS / WEEK"
                    value="47"
                    change="+12%"
                    trend="up"
                />
                <StatCard
                    label="ACTIVE MEMBERS"
                    value="234"
                    change="+8%"
                    trend="up"
                />
                <StatCard
                    label="CIRCLE GROWTH"
                    value="18"
                    change="+3"
                    trend="up"
                />
                <StatCard
                    label="ENGAGEMENT RATE"
                    value="64%"
                    change="-2%"
                    trend="down"
                />
            </div>

            /* Engagement breakdown */
            <div class="grid gap-6 lg:grid-cols-2 mb-12">
                <div class="rounded-2xl border border-slate-800 bg-slate-900/50 p-8">
                    <h2 class="text-sm font-bold text-slate-400 font-mono uppercase tracking-widest mb-6">"ACTIVITY BREAKDOWN"</h2>
                    <div class="space-y-5">
                        <MetricBar label="Posts Created" value=47 max_val=100 color="bg-cyan-500" />
                        <MetricBar label="Comments" value=128 max_val=200 color="bg-violet-500" />
                        <MetricBar label="Likes Given" value=312 max_val=400 color="bg-emerald-500" />
                        <MetricBar label="Circle Joins" value=23 max_val=100 color="bg-amber-500" />
                        <MetricBar label="Direct Messages" value=89 max_val=150 color="bg-rose-500" />
                    </div>
                </div>

                <div class="rounded-2xl border border-slate-800 bg-slate-900/50 p-8">
                    <h2 class="text-sm font-bold text-slate-400 font-mono uppercase tracking-widest mb-6">"TOP CIRCLES"</h2>
                    <div class="space-y-4">
                        <CircleRankRow rank=1 name="Signal Detection" members=89 posts=156 />
                        <CircleRankRow rank=2 name="AI & Automation in PV" members=67 posts=98 />
                        <CircleRankRow rank=3 name="Regulatory Affairs" members=54 posts=87 />
                        <CircleRankRow rank=4 name="Risk Management" members=41 posts=63 />
                        <CircleRankRow rank=5 name="ICSR Processing" members=38 posts=45 />
                    </div>
                </div>
            </div>

            /* Member growth timeline */
            <div class="rounded-2xl border border-slate-800 bg-slate-900/50 p-8">
                <h2 class="text-sm font-bold text-slate-400 font-mono uppercase tracking-widest mb-6">"MEMBER GROWTH"</h2>
                <div class="grid grid-cols-7 gap-3">
                    <GrowthColumn day="Mon" value=12 max_val=30 />
                    <GrowthColumn day="Tue" value=8 max_val=30 />
                    <GrowthColumn day="Wed" value=19 max_val=30 />
                    <GrowthColumn day="Thu" value=24 max_val=30 />
                    <GrowthColumn day="Fri" value=15 max_val=30 />
                    <GrowthColumn day="Sat" value=6 max_val=30 />
                    <GrowthColumn day="Sun" value=4 max_val=30 />
                </div>
            </div>
        </div>
    }
}

#[component]
fn TimeRangeSelector(
    selected: ReadSignal<&'static str>,
    set_selected: WriteSignal<&'static str>,
) -> impl IntoView {
    let options: Vec<(&str, &str)> = vec![
        ("weekly", "WEEK"),
        ("monthly", "MONTH"),
        ("quarterly", "QUARTER"),
    ];

    view! {
        <div class="flex gap-1 rounded-lg border border-slate-700 bg-slate-900 p-1">
            {options.into_iter().map(|(value, label)| {
                let v = value;
                view! {
                    <button
                        class=move || if selected.get() == v {
                            "rounded-md bg-cyan-600 px-4 py-1.5 text-[10px] font-bold text-white font-mono uppercase tracking-widest transition-all"
                        } else {
                            "rounded-md px-4 py-1.5 text-[10px] font-bold text-slate-500 font-mono uppercase tracking-widest hover:text-white transition-all"
                        }
                        on:click=move |_| set_selected.set(v)
                    >
                        {label}
                    </button>
                }
            }).collect_view()}
        </div>
    }
}

#[component]
fn StatCard(
    label: &'static str,
    value: &'static str,
    change: &'static str,
    trend: &'static str,
) -> impl IntoView {
    let trend_color = if trend == "up" {
        "text-emerald-400"
    } else {
        "text-red-400"
    };
    let trend_icon = if trend == "up" { "^" } else { "v" };

    view! {
        <div class="rounded-2xl border border-slate-800 bg-slate-900/50 p-6 hover:border-slate-700 transition-all group">
            <p class="text-[10px] font-bold text-slate-500 font-mono uppercase tracking-widest">{label}</p>
            <div class="mt-3 flex items-end gap-3">
                <span class="text-3xl font-black text-white font-mono">{value}</span>
                <span class=format!("text-xs font-bold font-mono {trend_color}")>
                    {trend_icon}" "{change}
                </span>
            </div>
        </div>
    }
}

#[component]
fn MetricBar(label: &'static str, value: u32, max_val: u32, color: &'static str) -> impl IntoView {
    let percentage = if max_val > 0 {
        (value * 100) / max_val
    } else {
        0
    };

    view! {
        <div>
            <div class="flex items-center justify-between mb-2">
                <span class="text-xs font-bold text-slate-300 font-mono uppercase tracking-wider">{label}</span>
                <span class="text-xs font-bold text-white font-mono">{value}</span>
            </div>
            <div class="h-2 w-full rounded-full bg-slate-800">
                <div
                    class=format!("h-2 rounded-full {} transition-all", color)
                    style=format!("width: {}%", percentage)
                />
            </div>
        </div>
    }
}

#[component]
fn CircleRankRow(rank: u32, name: &'static str, members: u32, posts: u32) -> impl IntoView {
    view! {
        <div class="flex items-center gap-4 hover:bg-slate-800/30 rounded-lg p-3 -mx-3 transition-all">
            <span class="text-xs font-bold text-slate-600 font-mono w-6 text-right">{format!("#{rank}")}</span>
            <div class="flex-1">
                <p class="text-sm font-bold text-white">{name}</p>
                <div class="flex gap-4 mt-1 text-[10px] font-bold text-slate-500 font-mono uppercase tracking-widest">
                    <span>{members}" members"</span>
                    <span>{posts}" posts"</span>
                </div>
            </div>
        </div>
    }
}

#[component]
fn GrowthColumn(day: &'static str, value: u32, max_val: u32) -> impl IntoView {
    let height_pct = if max_val > 0 {
        (value * 100) / max_val
    } else {
        0
    };

    view! {
        <div class="flex flex-col items-center gap-2">
            <span class="text-xs font-bold text-white font-mono">{value}</span>
            <div class="w-full h-32 bg-slate-800 rounded-lg flex items-end overflow-hidden">
                <div
                    class="w-full bg-cyan-500/60 rounded-t-lg transition-all"
                    style=format!("height: {}%", height_pct)
                />
            </div>
            <span class="text-[10px] font-bold text-slate-500 font-mono uppercase">{day}</span>
        </div>
    }
}
