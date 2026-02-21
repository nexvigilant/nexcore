//! Admin: PV Domains — manage pharmacovigilance domain taxonomy

use leptos::prelude::*;

struct PvDomain {
    name: &'static str,
    subdomains: u32,
    courses: u32,
    ksbs: u32,
    color: &'static str,
    lead: &'static str,
}

const DOMAINS: &[PvDomain] = &[
    PvDomain {
        name: "Signal Detection & Management",
        subdomains: 6,
        courses: 3,
        ksbs: 98,
        color: "text-red-400",
        lead: "Dr. Elena Vasquez",
    },
    PvDomain {
        name: "Individual Case Safety Reports",
        subdomains: 5,
        courses: 2,
        ksbs: 82,
        color: "text-cyan-400",
        lead: "Aisha Patel",
    },
    PvDomain {
        name: "Aggregate Safety Reporting",
        subdomains: 4,
        courses: 3,
        ksbs: 74,
        color: "text-amber-400",
        lead: "James Okonkwo",
    },
    PvDomain {
        name: "Risk Management & Minimization",
        subdomains: 4,
        courses: 2,
        ksbs: 66,
        color: "text-violet-400",
        lead: "Sarah Williams",
    },
    PvDomain {
        name: "Benefit-Risk Assessment",
        subdomains: 3,
        courses: 2,
        ksbs: 62,
        color: "text-emerald-400",
        lead: "Dr. Ahmed Hassan",
    },
    PvDomain {
        name: "Regulatory Intelligence & Compliance",
        subdomains: 5,
        courses: 3,
        ksbs: 82,
        color: "text-blue-400",
        lead: "Dr. Thomas Richter",
    },
    PvDomain {
        name: "PV System Quality & Governance",
        subdomains: 4,
        courses: 2,
        ksbs: 72,
        color: "text-orange-400",
        lead: "Dr. Mei-Lin Chen",
    },
    PvDomain {
        name: "Clinical Trial Safety",
        subdomains: 3,
        courses: 1,
        ksbs: 52,
        color: "text-rose-400",
        lead: "Dr. Henrik Larsson",
    },
    PvDomain {
        name: "Patient Safety & Communication",
        subdomains: 3,
        courses: 1,
        ksbs: 48,
        color: "text-teal-400",
        lead: "Maria Santos",
    },
    PvDomain {
        name: "PV Technology & Innovation",
        subdomains: 4,
        courses: 2,
        ksbs: 78,
        color: "text-indigo-400",
        lead: "Priya Sharma",
    },
];

#[component]
pub fn AcademyPvDomainsPage() -> impl IntoView {
    let total_ksbs: u32 = DOMAINS.iter().map(|d| d.ksbs).sum();
    let total_courses: u32 = DOMAINS.iter().map(|d| d.courses).sum();

    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"PV Domains"</h1>
                    <p class="mt-1 text-slate-400">"Manage the pharmacovigilance domain taxonomy used across the academy."</p>
                </div>
                <button class="rounded-lg bg-cyan-600 px-4 py-2 text-xs font-bold text-white hover:bg-cyan-500 transition-colors font-mono uppercase">"+ Add Domain"</button>
            </div>

            /* Stats */
            <div class="mt-6 grid gap-4 sm:grid-cols-4">
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5">
                    <p class="text-[9px] font-bold text-slate-500 uppercase tracking-widest font-mono">"Domains"</p>
                    <p class="text-2xl font-black text-white font-mono mt-2">{DOMAINS.len().to_string()}</p>
                </div>
                <div class="rounded-xl border border-cyan-500/20 bg-cyan-500/5 p-5">
                    <p class="text-[9px] font-bold text-cyan-400 uppercase tracking-widest font-mono">"Total KSBs"</p>
                    <p class="text-2xl font-black text-cyan-400 font-mono mt-2">{total_ksbs.to_string()}</p>
                </div>
                <div class="rounded-xl border border-emerald-500/20 bg-emerald-500/5 p-5">
                    <p class="text-[9px] font-bold text-emerald-400 uppercase tracking-widest font-mono">"Courses"</p>
                    <p class="text-2xl font-black text-emerald-400 font-mono mt-2">{total_courses.to_string()}</p>
                </div>
                <div class="rounded-xl border border-amber-500/20 bg-amber-500/5 p-5">
                    <p class="text-[9px] font-bold text-amber-400 uppercase tracking-widest font-mono">"Subdomains"</p>
                    <p class="text-2xl font-black text-amber-400 font-mono mt-2">{DOMAINS.iter().map(|d| d.subdomains).sum::<u32>().to_string()}</p>
                </div>
            </div>

            /* Domain list */
            <div class="mt-8 space-y-2">
                {DOMAINS.iter().map(|d| {
                    view! {
                        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5 hover:border-amber-500/30 transition-colors cursor-pointer">
                            <div class="flex items-center justify-between">
                                <div>
                                    <h3 class=format!("text-sm font-bold {}", d.color)>{d.name}</h3>
                                    <p class="text-[10px] text-slate-500 font-mono mt-1">{"Lead: "}{d.lead}</p>
                                </div>
                                <div class="flex items-center gap-6 text-xs font-mono">
                                    <div class="text-center">
                                        <p class="text-slate-500 text-[9px] uppercase">"Subs"</p>
                                        <p class="text-white font-bold">{d.subdomains.to_string()}</p>
                                    </div>
                                    <div class="text-center">
                                        <p class="text-slate-500 text-[9px] uppercase">"Courses"</p>
                                        <p class="text-cyan-400 font-bold">{d.courses.to_string()}</p>
                                    </div>
                                    <div class="text-center">
                                        <p class="text-slate-500 text-[9px] uppercase">"KSBs"</p>
                                        <p class="text-amber-400 font-bold">{d.ksbs.to_string()}</p>
                                    </div>
                                </div>
                            </div>
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}
