/*! Admin: Waitlist management — queue, invitations, conversion tracking */

use leptos::prelude::*;

struct WaitlistEntry {
    position: u32,
    email: &'static str,
    name: &'static str,
    signed_up: &'static str,
    source: &'static str,
    status: &'static str,
}

const WAITLIST: [WaitlistEntry; 10] = [
    WaitlistEntry {
        position: 1,
        email: "elena.vasquez@biopharma.co",
        name: "Elena Vasquez",
        signed_up: "2026-01-03",
        source: "Organic",
        status: "Accepted",
    },
    WaitlistEntry {
        position: 2,
        email: "james.okoro@signalrx.io",
        name: "James Okoro",
        signed_up: "2026-01-05",
        source: "LinkedIn",
        status: "Accepted",
    },
    WaitlistEntry {
        position: 3,
        email: "priya.mehta@cro-global.com",
        name: "Priya Mehta",
        signed_up: "2026-01-08",
        source: "Conference",
        status: "Invited",
    },
    WaitlistEntry {
        position: 4,
        email: "lucas.brandt@novavigil.de",
        name: "Lucas Brandt",
        signed_up: "2026-01-12",
        source: "Referral",
        status: "Invited",
    },
    WaitlistEntry {
        position: 5,
        email: "sarah.kim@fda-consult.gov",
        name: "Sarah Kim",
        signed_up: "2026-01-15",
        source: "Partner",
        status: "Expired",
    },
    WaitlistEntry {
        position: 6,
        email: "tomasz.nowak@ema-watch.eu",
        name: "Tomasz Nowak",
        signed_up: "2026-01-19",
        source: "Organic",
        status: "Waiting",
    },
    WaitlistEntry {
        position: 7,
        email: "amira.hassan@dra-mena.org",
        name: "Amira Hassan",
        signed_up: "2026-01-22",
        source: "LinkedIn",
        status: "Waiting",
    },
    WaitlistEntry {
        position: 8,
        email: "ryan.chen@safetystack.com",
        name: "Ryan Chen",
        signed_up: "2026-01-28",
        source: "Referral",
        status: "Waiting",
    },
    WaitlistEntry {
        position: 9,
        email: "chloe.dupont@pharmaveil.fr",
        name: "Chloe Dupont",
        signed_up: "2026-02-01",
        source: "Conference",
        status: "Waiting",
    },
    WaitlistEntry {
        position: 10,
        email: "daniel.reyes@vigilcore.mx",
        name: "Daniel Reyes",
        signed_up: "2026-02-04",
        source: "Organic",
        status: "Waiting",
    },
];

fn count_by_status(target: &str) -> usize {
    WAITLIST.iter().filter(|e| e.status == target).count()
}

fn source_badge_class(source: &str) -> &'static str {
    match source {
        "Organic" => "bg-cyan-900/40 text-cyan-400 border border-cyan-800/50",
        "Referral" => "bg-violet-900/40 text-violet-400 border border-violet-800/50",
        "LinkedIn" => "bg-blue-900/40 text-blue-400 border border-blue-800/50",
        "Conference" => "bg-amber-900/40 text-amber-400 border border-amber-800/50",
        "Partner" => "bg-emerald-900/40 text-emerald-400 border border-emerald-800/50",
        _ => "bg-slate-900/40 text-slate-400 border border-slate-800/50",
    }
}

fn status_badge_class(status: &str) -> &'static str {
    match status {
        "Waiting" => "bg-slate-800/60 text-slate-400 border border-slate-700/50",
        "Invited" => "bg-amber-900/40 text-amber-400 border border-amber-800/50",
        "Accepted" => "bg-emerald-900/40 text-emerald-400 border border-emerald-800/50",
        "Expired" => "bg-red-900/40 text-red-400 border border-red-800/50",
        _ => "bg-slate-800/60 text-slate-400 border border-slate-700/50",
    }
}

fn status_dot_class(status: &str) -> &'static str {
    match status {
        "Waiting" => "bg-slate-500",
        "Invited" => "bg-amber-400",
        "Accepted" => "bg-emerald-400",
        "Expired" => "bg-red-400",
        _ => "bg-slate-500",
    }
}

#[component]
pub fn WaitlistPage() -> impl IntoView {
    let total = WAITLIST.len();
    let accepted = count_by_status("Accepted");
    let invited = count_by_status("Invited");
    let waiting = count_by_status("Waiting");
    let expired = count_by_status("Expired");

    /* conversion = accepted / (accepted + expired), as a percentage */
    let conversion_denom = accepted + expired;
    let conversion_pct = if conversion_denom > 0 {
        (accepted as f64 / conversion_denom as f64) * 100.0
    } else {
        0.0
    };

    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            /* ── Header ── */
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Waitlist"</h1>
                    <p class="mt-1 text-slate-400 text-sm">"Manage waitlisted users and send invitations."</p>
                </div>
                <a href="/admin" class="text-sm text-slate-400 hover:text-white transition-colors font-mono">"\u{2190} Dashboard"</a>
            </div>

            /* ── Stats Row ── */
            <div class="mt-8 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
                /* Total Waitlisted */
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5">
                    <p class="text-[9px] font-bold text-slate-500 uppercase tracking-widest font-mono">"Total Waitlisted"</p>
                    <p class="text-2xl font-black text-white font-mono mt-2">{total}</p>
                    <p class="text-[10px] text-slate-600 font-mono mt-1">
                        {format!("{waiting} waiting \u{00b7} {expired} expired")}
                    </p>
                </div>
                /* Invited This Week */
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5">
                    <p class="text-[9px] font-bold text-slate-500 uppercase tracking-widest font-mono">"Invited This Week"</p>
                    <p class="text-2xl font-black text-amber-400 font-mono mt-2">{invited}</p>
                    <p class="text-[10px] text-slate-600 font-mono mt-1">"Pending response"</p>
                </div>
                /* Accepted */
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5">
                    <p class="text-[9px] font-bold text-slate-500 uppercase tracking-widest font-mono">"Accepted"</p>
                    <p class="text-2xl font-black text-emerald-400 font-mono mt-2">{accepted}</p>
                    <p class="text-[10px] text-slate-600 font-mono mt-1">"Active accounts created"</p>
                </div>
                /* Conversion Rate */
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5">
                    <p class="text-[9px] font-bold text-slate-500 uppercase tracking-widest font-mono">"Conversion Rate"</p>
                    <p class="text-2xl font-black text-cyan-400 font-mono mt-2">
                        {format!("{conversion_pct:.1}%")}
                    </p>
                    <p class="text-[10px] text-slate-600 font-mono mt-1">"Accepted / (Accepted + Expired)"</p>
                </div>
            </div>

            /* ── Queue Header + Action ── */
            <div class="mt-8 flex items-center justify-between">
                <div>
                    <h2 class="text-lg font-bold text-white font-mono">"Waitlist Queue"</h2>
                    <p class="text-[10px] text-slate-500 font-mono uppercase tracking-widest mt-0.5">
                        {format!("{total} entries \u{00b7} sorted by position")}
                    </p>
                </div>
                <button class="rounded-lg bg-cyan-600 px-4 py-2 text-xs font-bold text-white hover:bg-cyan-500 transition-colors font-mono uppercase tracking-widest">
                    "\u{25b6} Send Next Batch"
                </button>
            </div>

            /* ── Table ── */
            <div class="mt-4 rounded-xl border border-slate-800 overflow-hidden">
                <table class="w-full text-left text-sm">
                    <thead class="bg-slate-900/80 text-[10px] uppercase text-slate-500 font-mono tracking-widest">
                        <tr>
                            <th class="px-4 py-3 w-16">"#"</th>
                            <th class="px-4 py-3">"Name / Email"</th>
                            <th class="px-4 py-3 w-28">"Signed Up"</th>
                            <th class="px-4 py-3 w-28">"Source"</th>
                            <th class="px-4 py-3 w-28">"Status"</th>
                            <th class="px-4 py-3 w-24 text-right">"Actions"</th>
                        </tr>
                    </thead>
                    <tbody class="text-slate-300">
                        {WAITLIST.iter().map(|entry| {
                            let src_class = source_badge_class(entry.source);
                            let stat_class = status_badge_class(entry.status);
                            let dot_class = status_dot_class(entry.status);
                            let is_actionable = entry.status == "Waiting";

                            view! {
                                <tr class="border-t border-slate-800/60 hover:bg-slate-800/30 transition-colors">
                                    /* Position */
                                    <td class="px-4 py-3">
                                        <span class="text-[10px] font-mono text-slate-500 font-bold">
                                            {format!("{:02}", entry.position)}
                                        </span>
                                    </td>
                                    /* Name + Email */
                                    <td class="px-4 py-3">
                                        <p class="text-sm font-medium text-white">{entry.name}</p>
                                        <p class="text-[10px] text-slate-500 font-mono">{entry.email}</p>
                                    </td>
                                    /* Signed Up */
                                    <td class="px-4 py-3">
                                        <span class="text-[10px] font-mono text-slate-400">{entry.signed_up}</span>
                                    </td>
                                    /* Source Badge */
                                    <td class="px-4 py-3">
                                        <span class={format!("inline-block rounded-full px-2 py-0.5 text-[9px] font-bold font-mono uppercase tracking-widest {src_class}")}>
                                            {entry.source}
                                        </span>
                                    </td>
                                    /* Status Badge */
                                    <td class="px-4 py-3">
                                        <span class={format!("inline-flex items-center gap-1.5 rounded-full px-2 py-0.5 text-[9px] font-bold font-mono uppercase tracking-widest {stat_class}")}>
                                            <span class={format!("inline-block h-1.5 w-1.5 rounded-full {dot_class}")}></span>
                                            {entry.status}
                                        </span>
                                    </td>
                                    /* Actions */
                                    <td class="px-4 py-3 text-right">
                                        {if is_actionable {
                                            view! {
                                                <button class="rounded-md bg-cyan-600/20 px-3 py-1 text-[10px] font-bold text-cyan-400 hover:bg-cyan-600/40 transition-colors font-mono uppercase tracking-widest border border-cyan-700/30">
                                                    "Invite"
                                                </button>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <span class="text-[10px] text-slate-600 font-mono">"\u{2014}"</span>
                                            }.into_any()
                                        }}
                                    </td>
                                </tr>
                            }
                        }).collect_view()}
                    </tbody>
                </table>
            </div>

            /* ── Footer ── */
            <div class="mt-6 flex items-center justify-between rounded-xl border border-slate-800 bg-slate-900/50 px-5 py-4">
                <div class="flex items-center gap-6">
                    <div class="flex items-center gap-2">
                        <span class="inline-block h-2 w-2 rounded-full bg-slate-500"></span>
                        <span class="text-[9px] font-mono uppercase tracking-widest text-slate-500">"Waiting"</span>
                    </div>
                    <div class="flex items-center gap-2">
                        <span class="inline-block h-2 w-2 rounded-full bg-amber-400"></span>
                        <span class="text-[9px] font-mono uppercase tracking-widest text-slate-500">"Invited"</span>
                    </div>
                    <div class="flex items-center gap-2">
                        <span class="inline-block h-2 w-2 rounded-full bg-emerald-400"></span>
                        <span class="text-[9px] font-mono uppercase tracking-widest text-slate-500">"Accepted"</span>
                    </div>
                    <div class="flex items-center gap-2">
                        <span class="inline-block h-2 w-2 rounded-full bg-red-400"></span>
                        <span class="text-[9px] font-mono uppercase tracking-widest text-slate-500">"Expired"</span>
                    </div>
                </div>
                <p class="text-[10px] text-slate-600 font-mono">
                    {format!("{total} entries \u{00b7} Last batch sent 2026-02-10")}
                </p>
            </div>
        </div>
    }
}
