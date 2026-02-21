//! Admin: Community sub-pages — moderation, circles, reports

use leptos::prelude::*;

/* ------------------------------------------------------------------ */
/*  Moderation Queue                                                   */
/* ------------------------------------------------------------------ */

#[component]
pub fn ModerationQueuePage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Moderation Queue"</h1>
            <p class="mt-1 text-slate-400">"Review and act on flagged content and user reports."</p>

            <div class="mt-8 space-y-4">
                <ModerationItem
                    author="User123"
                    type_="Post"
                    reason="Spam"
                    content="Check out this cool new signal detection tool I found on a random site..."
                />
                <ModerationItem
                    author="NewbiePV"
                    type_="Comment"
                    reason="Off-topic"
                    content="Does anyone know where I can buy cheap coffee near the office?"
                />
            </div>
        </div>
    }
}

#[component]
fn ModerationItem(
    author: &'static str,
    type_: &'static str,
    reason: &'static str,
    content: &'static str,
) -> impl IntoView {
    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
            <div class="flex items-center justify-between mb-4">
                <div class="flex items-center gap-3">
                    <span class="rounded bg-red-500/10 px-2 py-0.5 text-[10px] font-bold uppercase text-red-400">{reason}</span>
                    <span class="text-xs text-slate-500">{type_}" by "{author}</span>
                </div>
                <div class="flex gap-2">
                    <button class="rounded bg-emerald-600 px-3 py-1 text-xs font-bold text-white hover:bg-emerald-500 transition-colors">"APPROVE"</button>
                    <button class="rounded bg-red-600 px-3 py-1 text-xs font-bold text-white hover:bg-red-500 transition-colors">"REJECT"</button>
                </div>
            </div>
            <p class="text-sm text-slate-300 italic">"{"{content}"}"</p>
        </div>
    }
}

/* ------------------------------------------------------------------ */
/*  Circle Management                                                  */
/* ------------------------------------------------------------------ */

#[component]
pub fn CircleManagementPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Circle Management"</h1>
            <p class="mt-1 text-slate-400">"Create, edit, and archive community circles."</p>

            <div class="mt-8 grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                <CircleCard name="Signal Detection" members=1240 posts=450 status="Active"/>
                <CircleCard name="Regulatory EU" members=850 posts=210 status="Active"/>
                <CircleCard name="Career Transitions" members=520 posts=180 status="Active"/>
                <CircleCard name="Legacy Systems" members=120 posts=15 status="Archived"/>
            </div>
        </div>
    }
}

#[component]
fn CircleCard(name: &'static str, members: u32, posts: u32, status: &'static str) -> impl IntoView {
    let status_cls = if status == "Active" {
        "text-emerald-400"
    } else {
        "text-slate-500"
    };
    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6 hover:border-amber-500/30 transition-colors">
            <div class="flex items-center justify-between">
                <h3 class="font-bold text-white">{name}</h3>
                <span class=format!("text-[10px] font-bold uppercase {status_cls}")>{status}</span>
            </div>
            <div class="mt-4 flex gap-4 text-xs text-slate-500 font-mono">
                <span>{members}" members"</span>
                <span>{posts}" posts"</span>
            </div>
            <div class="mt-6 flex gap-2">
                <button class="flex-1 rounded border border-slate-700 py-1 text-xs font-bold text-slate-400 hover:text-white transition-colors">"EDIT"</button>
                <button class="flex-1 rounded border border-slate-700 py-1 text-xs font-bold text-slate-400 hover:text-white transition-colors">"ARCHIVE"</button>
            </div>
        </div>
    }
}
