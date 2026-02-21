//! Admin: Academy certificates — issue, revoke, and manage learner certificates

use leptos::prelude::*;

struct CertTemplate {
    name: &'static str,
    level: &'static str,
    issued: u32,
    status: &'static str,
    color: &'static str,
}

const TEMPLATES: &[CertTemplate] = &[
    CertTemplate {
        name: "PV Fundamentals",
        level: "Beginner",
        issued: 234,
        status: "Active",
        color: "text-emerald-400",
    },
    CertTemplate {
        name: "Signal Detection Specialist",
        level: "Intermediate",
        issued: 89,
        status: "Active",
        color: "text-cyan-400",
    },
    CertTemplate {
        name: "Case Processing Expert",
        level: "Intermediate",
        issued: 67,
        status: "Active",
        color: "text-amber-400",
    },
    CertTemplate {
        name: "Regulatory Affairs Professional",
        level: "Advanced",
        issued: 34,
        status: "Active",
        color: "text-violet-400",
    },
    CertTemplate {
        name: "Aggregate Reporting Specialist",
        level: "Advanced",
        issued: 23,
        status: "Active",
        color: "text-blue-400",
    },
    CertTemplate {
        name: "QPPV Ready",
        level: "Expert",
        issued: 12,
        status: "Active",
        color: "text-red-400",
    },
];

struct RecentCert {
    learner: &'static str,
    certificate: &'static str,
    date: &'static str,
    score: u8,
}

const RECENT_CERTS: &[RecentCert] = &[
    RecentCert {
        learner: "Dr. Elena Vasquez",
        certificate: "Signal Detection Specialist",
        date: "2026-02-15",
        score: 94,
    },
    RecentCert {
        learner: "Aisha Patel",
        certificate: "Case Processing Expert",
        date: "2026-02-15",
        score: 97,
    },
    RecentCert {
        learner: "Dr. Ahmed Hassan",
        certificate: "QPPV Ready",
        date: "2026-02-14",
        score: 92,
    },
    RecentCert {
        learner: "Dr. Thomas Richter",
        certificate: "Regulatory Affairs Professional",
        date: "2026-02-14",
        score: 88,
    },
    RecentCert {
        learner: "Dr. Mei-Lin Chen",
        certificate: "PV Fundamentals",
        date: "2026-02-13",
        score: 96,
    },
    RecentCert {
        learner: "Priya Sharma",
        certificate: "Signal Detection Specialist",
        date: "2026-02-13",
        score: 91,
    },
];

#[component]
pub fn AcademyCertificatesPage() -> impl IntoView {
    let total_issued: u32 = TEMPLATES.iter().map(|t| t.issued).sum();

    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Academy Certificates"</h1>
                    <p class="mt-1 text-slate-400">"Issue, manage, and verify learner certificates."</p>
                </div>
                <button class="rounded-lg bg-cyan-600 px-4 py-2 text-xs font-bold text-white hover:bg-cyan-500 transition-colors font-mono uppercase">"+ New Template"</button>
            </div>

            /* Stats */
            <div class="mt-6 grid gap-4 sm:grid-cols-3">
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5">
                    <p class="text-[9px] font-bold text-slate-500 uppercase tracking-widest font-mono">"Templates"</p>
                    <p class="text-2xl font-black text-white font-mono mt-2">{TEMPLATES.len().to_string()}</p>
                </div>
                <div class="rounded-xl border border-emerald-500/20 bg-emerald-500/5 p-5">
                    <p class="text-[9px] font-bold text-emerald-400 uppercase tracking-widest font-mono">"Total Issued"</p>
                    <p class="text-2xl font-black text-emerald-400 font-mono mt-2">{total_issued.to_string()}</p>
                </div>
                <div class="rounded-xl border border-cyan-500/20 bg-cyan-500/5 p-5">
                    <p class="text-[9px] font-bold text-cyan-400 uppercase tracking-widest font-mono">"This Month"</p>
                    <p class="text-2xl font-black text-cyan-400 font-mono mt-2">{RECENT_CERTS.len().to_string()}</p>
                </div>
            </div>

            /* Certificate Templates */
            <h2 class="mt-8 text-sm font-bold uppercase tracking-widest text-slate-500 font-mono mb-4">"Certificate Templates"</h2>
            <div class="grid gap-3 md:grid-cols-2 lg:grid-cols-3">
                {TEMPLATES.iter().map(|t| view! {
                    <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5 hover:border-slate-600 transition-colors cursor-pointer">
                        <div class="flex items-center justify-between">
                            <h3 class=format!("text-sm font-bold {}", t.color)>{t.name}</h3>
                            <span class="rounded-full bg-emerald-500/10 border border-emerald-500/20 px-2 py-0.5 text-[9px] text-emerald-400 font-mono uppercase font-bold">{t.status}</span>
                        </div>
                        <div class="mt-3 flex items-center justify-between text-xs font-mono">
                            <span class="text-slate-500">{t.level}</span>
                            <span class="text-slate-400">{format!("{} issued", t.issued)}</span>
                        </div>
                    </div>
                }).collect_view()}
            </div>

            /* Recently Issued */
            <h2 class="mt-8 text-sm font-bold uppercase tracking-widest text-slate-500 font-mono mb-4">"Recently Issued"</h2>
            <div class="rounded-xl border border-slate-800 overflow-hidden">
                <table class="w-full text-left text-sm">
                    <thead class="bg-slate-900/80 text-[10px] uppercase text-slate-500 font-mono tracking-widest">
                        <tr>
                            <th class="px-4 py-3">"Learner"</th>
                            <th class="px-4 py-3">"Certificate"</th>
                            <th class="px-4 py-3 text-right">"Score"</th>
                            <th class="px-4 py-3">"Date"</th>
                        </tr>
                    </thead>
                    <tbody class="text-slate-300">
                        {RECENT_CERTS.iter().map(|c| {
                            let score_cls = if c.score >= 90 { "text-emerald-400" } else { "text-amber-400" };
                            view! {
                                <tr class="border-t border-slate-800 hover:bg-slate-800/30 transition-colors">
                                    <td class="px-4 py-3 text-sm font-medium text-white">{c.learner}</td>
                                    <td class="px-4 py-3 text-xs text-cyan-400 font-mono">{c.certificate}</td>
                                    <td class="px-4 py-3 text-right">
                                        <span class=format!("text-xs font-bold font-mono {score_cls}")>{format!("{}%", c.score)}</span>
                                    </td>
                                    <td class="px-4 py-3 text-[10px] text-slate-600 font-mono">{c.date}</td>
                                </tr>
                            }
                        }).collect_view()}
                    </tbody>
                </table>
            </div>
        </div>
    }
}
