//! Admin: Community messaging oversight

use leptos::prelude::*;

struct MessageThread {
    participants: &'static str,
    last_message: &'static str,
    time_ago: &'static str,
    messages: u32,
    flagged: bool,
}

const THREADS: &[MessageThread] = &[
    MessageThread {
        participants: "Dr. Elena Vasquez \u{2194} Aisha Patel",
        last_message: "Can you review the signal detection module draft?",
        time_ago: "5m ago",
        messages: 12,
        flagged: false,
    },
    MessageThread {
        participants: "James Okonkwo \u{2194} Dr. Thomas Richter",
        last_message: "The PBRER template needs updating for the new EMA format",
        time_ago: "22m ago",
        messages: 8,
        flagged: false,
    },
    MessageThread {
        participants: "anon_user_42 \u{2194} new_member_19",
        last_message: "[FLAGGED] Potential sharing of confidential ICSR data",
        time_ago: "1h ago",
        messages: 3,
        flagged: true,
    },
    MessageThread {
        participants: "Dr. Mei-Lin Chen \u{2194} Sarah Williams",
        last_message: "Great work on the risk management course outline!",
        time_ago: "2h ago",
        messages: 24,
        flagged: false,
    },
    MessageThread {
        participants: "trial_user_88 \u{2194} Multiple (15)",
        last_message: "[FLAGGED] Bulk promotional messages detected",
        time_ago: "3h ago",
        messages: 15,
        flagged: true,
    },
    MessageThread {
        participants: "Dr. Ahmed Hassan \u{2194} Maria Santos",
        last_message: "The benefit-risk workshop received excellent feedback",
        time_ago: "5h ago",
        messages: 31,
        flagged: false,
    },
    MessageThread {
        participants: "Priya Sharma \u{2194} Dr. Henrik Larsson",
        last_message: "Nordic PV regulations update — can we discuss?",
        time_ago: "8h ago",
        messages: 6,
        flagged: false,
    },
    MessageThread {
        participants: "deleted_account \u{2194} pv_enthusiast",
        last_message: "[FLAGGED] Reported harassment",
        time_ago: "12h ago",
        messages: 4,
        flagged: true,
    },
];

#[component]
pub fn CommunityMessagesPage() -> impl IntoView {
    let total_msgs: u32 = THREADS.iter().map(|t| t.messages).sum();
    let flagged = THREADS.iter().filter(|t| t.flagged).count();

    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Messaging Oversight"</h1>
                    <p class="mt-1 text-slate-400">"Monitor messaging system health and flagged conversations."</p>
                </div>
                <a href="/admin/community" class="text-sm text-slate-400 hover:text-white transition-colors font-mono">"\u{2190} Community Admin"</a>
            </div>

            /* Stats */
            <div class="mt-6 grid gap-4 md:grid-cols-3">
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5">
                    <p class="text-[9px] font-bold text-slate-500 uppercase tracking-widest font-mono">"Active Threads"</p>
                    <p class="text-2xl font-black text-white font-mono mt-2">{THREADS.len().to_string()}</p>
                </div>
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5">
                    <p class="text-[9px] font-bold text-slate-500 uppercase tracking-widest font-mono">"Messages Today"</p>
                    <p class="text-2xl font-black text-cyan-400 font-mono mt-2">{total_msgs.to_string()}</p>
                </div>
                <div class="rounded-xl border border-red-500/20 bg-red-500/5 p-5">
                    <p class="text-[9px] font-bold text-red-400 uppercase tracking-widest font-mono">"Flagged"</p>
                    <p class="text-2xl font-black text-red-400 font-mono mt-2">{flagged.to_string()}</p>
                </div>
            </div>

            /* Thread list */
            <div class="mt-8 space-y-2">
                <h3 class="text-sm font-bold text-white font-mono uppercase tracking-widest mb-4">"Recent Threads"</h3>
                {THREADS.iter().map(|t| {
                    let border = if t.flagged { "border-l-red-500" } else { "border-l-slate-600" };
                    view! {
                        <div class=format!("rounded-xl border border-slate-800 bg-slate-900/50 p-4 border-l-2 {border} hover:bg-slate-800/30 transition-colors cursor-pointer")>
                            <div class="flex items-center justify-between">
                                <div class="min-w-0 flex-1">
                                    <div class="flex items-center gap-2">
                                        {t.flagged.then(|| view! {
                                            <span class="rounded-full bg-red-500/10 border border-red-500/20 px-2 py-0.5 text-[9px] font-bold text-red-400 font-mono uppercase">"Flagged"</span>
                                        })}
                                        <p class="text-xs text-slate-400 font-mono truncate">{t.participants}</p>
                                    </div>
                                    <p class="mt-1 text-sm text-slate-300 truncate">{t.last_message}</p>
                                </div>
                                <div class="flex items-center gap-4 shrink-0 ml-4">
                                    <span class="text-xs text-slate-500 font-mono">{format!("{} msgs", t.messages)}</span>
                                    <span class="text-[10px] text-slate-600 font-mono">{t.time_ago}</span>
                                </div>
                            </div>
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}
