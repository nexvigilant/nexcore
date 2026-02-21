//! Discover matches — personalized member and circle matching
//!
//! Shows AI-powered match suggestions based on user's profile,
//! interests, expertise domains, and activity patterns.

use leptos::prelude::*;

/* ── Match data ─────────────────────────────────────── */

struct MemberMatch {
    name: &'static str,
    role: &'static str,
    org: &'static str,
    match_pct: u8,
    shared_domains: &'static [&'static str],
    reason: &'static str,
}

struct CircleMatch {
    name: &'static str,
    description: &'static str,
    members: u32,
    match_pct: u8,
    reason: &'static str,
}

const MEMBER_MATCHES: &[MemberMatch] = &[
    MemberMatch {
        name: "Dr. Elena Vasquez",
        role: "Head of Signal Management",
        org: "Novartis",
        match_pct: 94,
        shared_domains: &["Signal Detection", "Disproportionality Analysis", "AI/ML"],
        reason: "Shares your interest in automated signal detection and has published on PRR optimization.",
    },
    MemberMatch {
        name: "James Okonkwo",
        role: "Senior PV Scientist",
        org: "AstraZeneca",
        match_pct: 87,
        shared_domains: &["Benefit-Risk", "PBRER", "Aggregate Reporting"],
        reason: "Active contributor to benefit-risk methodology discussions in your primary focus area.",
    },
    MemberMatch {
        name: "Dr. Mei-Lin Chen",
        role: "QPPV Deputy",
        org: "Roche",
        match_pct: 82,
        shared_domains: &["Regulatory Strategy", "GVP Compliance"],
        reason: "Experienced QPPV network member who mentors professionals transitioning to senior PV roles.",
    },
    MemberMatch {
        name: "Aisha Patel",
        role: "PV Data Engineer",
        org: "Moderna",
        match_pct: 79,
        shared_domains: &["Automation", "NLP", "Data Engineering"],
        reason: "Building similar ICSR automation pipelines and interested in collaborative benchmarking.",
    },
    MemberMatch {
        name: "Dr. Thomas Richter",
        role: "Medical Director",
        org: "Bayer",
        match_pct: 75,
        shared_domains: &["Signal Detection", "Clinical Safety"],
        reason: "Focuses on rare disease safety signals, complementing your therapeutic area expertise.",
    },
    MemberMatch {
        name: "Sarah Williams",
        role: "PV Compliance Lead",
        org: "GSK",
        match_pct: 71,
        shared_domains: &["GVP", "Audit Readiness"],
        reason: "Leading a GVP Module I implementation project and seeking knowledge exchange partners.",
    },
];

const CIRCLE_MATCHES: &[CircleMatch] = &[
    CircleMatch {
        name: "Signal Hunters",
        description: "Collaborative disproportionality analysis and emerging safety signal identification.",
        members: 342,
        match_pct: 96,
        reason: "Directly aligned with your signal detection interests and analytics focus.",
    },
    CircleMatch {
        name: "AI & Automation in PV",
        description: "NLP for case processing, LLM safety assessment, and workflow automation.",
        members: 187,
        match_pct: 91,
        reason: "Your automation and data engineering interests are core to this circle's mission.",
    },
    CircleMatch {
        name: "Regulatory Roundtable",
        description: "ICH guidelines, PSMF updates, and global regulatory submission strategies.",
        members: 218,
        match_pct: 84,
        reason: "Your regulatory knowledge areas overlap with active discussion threads here.",
    },
    CircleMatch {
        name: "Real-World Evidence Network",
        description: "Using RWD for safety signal validation, effectiveness comparisons, and regulatory submissions.",
        members: 98,
        match_pct: 78,
        reason: "Growing community focused on RWE applications in pharmacovigilance, your emerging interest.",
    },
];

/* ── Page component ─────────────────────────────────── */

#[component]
pub fn DiscoverMatchesPage() -> impl IntoView {
    let active_tab = RwSignal::new("members");

    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            /* Header */
            <div class="mb-8">
                <div class="flex items-center gap-3 mb-2">
                    <a href="/community/discover" class="text-slate-500 hover:text-white transition-colors text-sm font-mono">
                        {"\u{2190} Discover"}
                    </a>
                </div>
                <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Your Matches"</h1>
                <p class="mt-2 text-slate-400">"Personalized connections based on your expertise, interests, and activity"</p>
            </div>

            /* Match quality banner */
            <div class="rounded-xl border border-cyan-500/20 bg-cyan-500/5 p-5 mb-8 flex flex-col sm:flex-row items-start sm:items-center gap-4">
                <div class="flex items-center gap-3">
                    <div class="h-10 w-10 rounded-full bg-cyan-500/20 flex items-center justify-center">
                        <svg class="h-5 w-5 text-cyan-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
                        </svg>
                    </div>
                    <div>
                        <p class="text-sm font-bold text-cyan-400">"Match Algorithm Active"</p>
                        <p class="text-[10px] text-slate-500 font-mono">"Based on your profile, 3 shared domains, and 12 interactions"</p>
                    </div>
                </div>
                <a href="/community/find-your-home" class="ml-auto text-[10px] font-bold text-cyan-400 hover:text-cyan-300 font-mono uppercase tracking-widest transition-colors">
                    "Retake Quiz"
                </a>
            </div>

            /* Tabs */
            <div class="flex gap-2 mb-8 border-b border-slate-800 pb-3">
                <button
                    on:click=move |_| active_tab.set("members")
                    class=move || if active_tab.get() == "members" {
                        "px-5 py-2 text-xs font-bold text-cyan-400 border-b-2 border-cyan-400 font-mono uppercase tracking-widest"
                    } else {
                        "px-5 py-2 text-xs font-medium text-slate-500 hover:text-slate-300 font-mono uppercase tracking-widest transition-colors"
                    }
                >
                    {format!("Members ({})", MEMBER_MATCHES.len())}
                </button>
                <button
                    on:click=move |_| active_tab.set("circles")
                    class=move || if active_tab.get() == "circles" {
                        "px-5 py-2 text-xs font-bold text-cyan-400 border-b-2 border-cyan-400 font-mono uppercase tracking-widest"
                    } else {
                        "px-5 py-2 text-xs font-medium text-slate-500 hover:text-slate-300 font-mono uppercase tracking-widest transition-colors"
                    }
                >
                    {format!("Circles ({})", CIRCLE_MATCHES.len())}
                </button>
            </div>

            /* Content */
            {move || match active_tab.get() {
                "circles" => view! { <CircleMatchesSection /> }.into_any(),
                _ => view! { <MemberMatchesSection /> }.into_any(),
            }}
        </div>
    }
}

/* ── Member matches ─────────────────────────────────── */

#[component]
fn MemberMatchesSection() -> impl IntoView {
    view! {
        <div class="space-y-4">
            {MEMBER_MATCHES.iter().map(|m| {
                let initial = m.name.chars().next().unwrap_or('?').to_uppercase().to_string();
                let pct_color = if m.match_pct >= 90 {
                    "text-emerald-400 bg-emerald-500/10 border-emerald-500/20"
                } else if m.match_pct >= 80 {
                    "text-cyan-400 bg-cyan-500/10 border-cyan-500/20"
                } else if m.match_pct >= 70 {
                    "text-amber-400 bg-amber-500/10 border-amber-500/20"
                } else {
                    "text-slate-400 bg-slate-500/10 border-slate-500/20"
                };
                let bar_width = format!("{}%", m.match_pct);
                let bar_color = if m.match_pct >= 90 {
                    "bg-emerald-500"
                } else if m.match_pct >= 80 {
                    "bg-cyan-500"
                } else {
                    "bg-amber-500"
                };

                view! {
                    <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5 hover:border-slate-700 transition-colors">
                        <div class="flex items-start gap-4">
                            <div class="h-12 w-12 rounded-full bg-violet-500/10 flex items-center justify-center text-lg font-bold text-violet-400 shrink-0">
                                {initial}
                            </div>
                            <div class="flex-1 min-w-0">
                                <div class="flex items-center gap-3 flex-wrap">
                                    <h3 class="text-sm font-bold text-white">{m.name}</h3>
                                    <span class=format!("rounded-full border px-2 py-0.5 text-[10px] font-bold font-mono {pct_color}")>
                                        {format!("{}% match", m.match_pct)}
                                    </span>
                                </div>
                                <p class="text-xs text-slate-500 mt-0.5">{m.role}" \u{2022} "{m.org}</p>
                                <p class="text-xs text-slate-400 mt-2 leading-relaxed">{m.reason}</p>

                                /* Match bar */
                                <div class="mt-3 h-1.5 w-full rounded-full bg-slate-800 overflow-hidden">
                                    <div class=format!("h-full rounded-full {bar_color} transition-all") style=format!("width: {bar_width}")></div>
                                </div>

                                /* Shared domains */
                                <div class="mt-3 flex flex-wrap gap-1.5">
                                    {m.shared_domains.iter().map(|d| view! {
                                        <span class="rounded-md bg-slate-800 px-2 py-0.5 text-[9px] font-bold font-mono text-slate-400 uppercase tracking-wider">
                                            {*d}
                                        </span>
                                    }).collect_view()}
                                </div>
                            </div>
                            <button class="shrink-0 rounded-lg bg-cyan-600 px-4 py-2 text-xs font-bold text-white hover:bg-cyan-500 transition-colors">
                                "Connect"
                            </button>
                        </div>
                    </div>
                }
            }).collect_view()}
        </div>
    }
}

/* ── Circle matches ─────────────────────────────────── */

#[component]
fn CircleMatchesSection() -> impl IntoView {
    view! {
        <div class="space-y-4">
            {CIRCLE_MATCHES.iter().map(|c| {
                let initial = c.name.chars().next().unwrap_or('?').to_string();
                let pct_color = if c.match_pct >= 90 {
                    "text-emerald-400 bg-emerald-500/10 border-emerald-500/20"
                } else if c.match_pct >= 80 {
                    "text-cyan-400 bg-cyan-500/10 border-cyan-500/20"
                } else {
                    "text-amber-400 bg-amber-500/10 border-amber-500/20"
                };
                let bar_width = format!("{}%", c.match_pct);
                let bar_color = if c.match_pct >= 90 {
                    "bg-emerald-500"
                } else if c.match_pct >= 80 {
                    "bg-cyan-500"
                } else {
                    "bg-amber-500"
                };

                view! {
                    <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5 hover:border-slate-700 transition-colors">
                        <div class="flex items-start gap-4">
                            <div class="h-12 w-12 rounded-full bg-violet-500/10 flex items-center justify-center text-lg font-bold text-violet-400 shrink-0">
                                {initial}
                            </div>
                            <div class="flex-1 min-w-0">
                                <div class="flex items-center gap-3 flex-wrap">
                                    <h3 class="text-sm font-bold text-white">{c.name}</h3>
                                    <span class=format!("rounded-full border px-2 py-0.5 text-[10px] font-bold font-mono {pct_color}")>
                                        {format!("{}% match", c.match_pct)}
                                    </span>
                                </div>
                                <p class="text-xs text-slate-400 mt-1 leading-relaxed">{c.description}</p>
                                <p class="text-xs text-slate-500 mt-2 italic">{c.reason}</p>

                                /* Match bar */
                                <div class="mt-3 h-1.5 w-full rounded-full bg-slate-800 overflow-hidden">
                                    <div class=format!("h-full rounded-full {bar_color} transition-all") style=format!("width: {bar_width}")></div>
                                </div>

                                <span class="mt-2 inline-block text-[10px] text-slate-600 font-mono">{format!("{} members", c.members)}</span>
                            </div>
                            <button class="shrink-0 rounded-lg border border-violet-500/30 bg-violet-500/10 px-4 py-2 text-xs font-bold text-violet-400 hover:bg-violet-500/20 transition-colors">
                                "Join"
                            </button>
                        </div>
                    </div>
                }
            }).collect_view()}
        </div>
    }
}
