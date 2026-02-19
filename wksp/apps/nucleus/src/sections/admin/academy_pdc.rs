//! Admin: Academy PDC — Professional Development Continuum management

use leptos::prelude::*;

struct PdcStage {
    name: &'static str,
    desc: &'static str,
    cpas: u32,
    learners: u32,
    completion_rate: u8,
    color: &'static str,
}

const STAGES: &[PdcStage] = &[
    PdcStage {
        name: "Foundation",
        desc: "Core PV knowledge and fundamental skills",
        cpas: 12,
        learners: 234,
        completion_rate: 72,
        color: "text-emerald-400 border-emerald-500/20 bg-emerald-500/5",
    },
    PdcStage {
        name: "Intermediate",
        desc: "Applied PV practice and specialized skills",
        cpas: 16,
        learners: 156,
        completion_rate: 58,
        color: "text-cyan-400 border-cyan-500/20 bg-cyan-500/5",
    },
    PdcStage {
        name: "Advanced",
        desc: "Leadership, specialization, and strategic thinking",
        cpas: 12,
        learners: 67,
        completion_rate: 45,
        color: "text-amber-400 border-amber-500/20 bg-amber-500/5",
    },
    PdcStage {
        name: "Expert",
        desc: "Innovation, mentoring, and thought leadership",
        cpas: 8,
        learners: 23,
        completion_rate: 34,
        color: "text-violet-400 border-violet-500/20 bg-violet-500/5",
    },
];

struct Pathway {
    name: &'static str,
    stages: &'static str,
    ksbs: u32,
    enrolled: u32,
}

const PATHWAYS: &[Pathway] = &[
    Pathway {
        name: "Signal Detection Specialist",
        stages: "Foundation \u{2192} Advanced",
        ksbs: 28,
        enrolled: 89,
    },
    Pathway {
        name: "Case Processing Expert",
        stages: "Foundation \u{2192} Intermediate",
        ksbs: 22,
        enrolled: 134,
    },
    Pathway {
        name: "Regulatory Affairs Professional",
        stages: "Foundation \u{2192} Expert",
        ksbs: 36,
        enrolled: 45,
    },
    Pathway {
        name: "QPPV Readiness",
        stages: "Intermediate \u{2192} Expert",
        ksbs: 42,
        enrolled: 23,
    },
    Pathway {
        name: "PV System Auditor",
        stages: "Intermediate \u{2192} Advanced",
        ksbs: 18,
        enrolled: 56,
    },
    Pathway {
        name: "Clinical Safety Scientist",
        stages: "Foundation \u{2192} Advanced",
        ksbs: 32,
        enrolled: 34,
    },
    Pathway {
        name: "PV Technology Innovator",
        stages: "Intermediate \u{2192} Expert",
        ksbs: 24,
        enrolled: 67,
    },
];

#[component]
pub fn AcademyPdcPage() -> impl IntoView {
    let total_learners: u32 = STAGES.iter().map(|s| s.learners).sum();
    let total_cpas: u32 = STAGES.iter().map(|s| s.cpas).sum();

    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <div>
                <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Professional Development Continuum"</h1>
                <p class="mt-1 text-slate-400">"Manage CPAs, learning pathways, and professional development requirements."</p>
            </div>

            /* Stats */
            <div class="mt-6 grid gap-4 sm:grid-cols-4">
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5">
                    <p class="text-[9px] font-bold text-slate-500 uppercase tracking-widest font-mono">"CPAs"</p>
                    <p class="text-2xl font-black text-white font-mono mt-2">{total_cpas.to_string()}</p>
                </div>
                <div class="rounded-xl border border-cyan-500/20 bg-cyan-500/5 p-5">
                    <p class="text-[9px] font-bold text-cyan-400 uppercase tracking-widest font-mono">"Pathways"</p>
                    <p class="text-2xl font-black text-cyan-400 font-mono mt-2">{PATHWAYS.len().to_string()}</p>
                </div>
                <div class="rounded-xl border border-emerald-500/20 bg-emerald-500/5 p-5">
                    <p class="text-[9px] font-bold text-emerald-400 uppercase tracking-widest font-mono">"Learners"</p>
                    <p class="text-2xl font-black text-emerald-400 font-mono mt-2">{total_learners.to_string()}</p>
                </div>
                <div class="rounded-xl border border-amber-500/20 bg-amber-500/5 p-5">
                    <p class="text-[9px] font-bold text-amber-400 uppercase tracking-widest font-mono">"Stages"</p>
                    <p class="text-2xl font-black text-amber-400 font-mono mt-2">{STAGES.len().to_string()}</p>
                </div>
            </div>

            /* Development Stages */
            <h2 class="mt-8 text-sm font-bold uppercase tracking-widest text-slate-500 font-mono mb-4">"Development Stages"</h2>
            <div class="grid gap-4 lg:grid-cols-4">
                {STAGES.iter().map(|s| {
                    let progress_cls = if s.completion_rate >= 70 { "bg-emerald-500" } else if s.completion_rate >= 50 { "bg-amber-500" } else { "bg-red-500" };
                    view! {
                        <div class=format!("rounded-xl border p-5 {}", s.color)>
                            <h3 class="text-lg font-bold text-white">{s.name}</h3>
                            <p class="text-[10px] text-slate-400 mt-1">{s.desc}</p>
                            <div class="mt-4 space-y-2">
                                <div class="flex justify-between text-xs font-mono">
                                    <span class="text-slate-500">"CPAs"</span>
                                    <span class="text-white font-bold">{s.cpas.to_string()}</span>
                                </div>
                                <div class="flex justify-between text-xs font-mono">
                                    <span class="text-slate-500">"Learners"</span>
                                    <span class="text-white font-bold">{s.learners.to_string()}</span>
                                </div>
                                <div class="flex items-center gap-2">
                                    <div class="flex-1 h-1.5 rounded-full bg-slate-800 overflow-hidden">
                                        <div class=format!("h-full rounded-full {progress_cls}") style=format!("width:{}%", s.completion_rate)></div>
                                    </div>
                                    <span class="text-[10px] text-slate-500 font-mono">{format!("{}%", s.completion_rate)}</span>
                                </div>
                            </div>
                        </div>
                    }
                }).collect_view()}
            </div>

            /* Pathways */
            <h2 class="mt-8 text-sm font-bold uppercase tracking-widest text-slate-500 font-mono mb-4">"Learning Pathways"</h2>
            <div class="rounded-xl border border-slate-800 overflow-hidden">
                <table class="w-full text-left text-sm">
                    <thead class="bg-slate-900/80 text-[10px] uppercase text-slate-500 font-mono tracking-widest">
                        <tr>
                            <th class="px-4 py-3">"Pathway"</th>
                            <th class="px-4 py-3">"Stages"</th>
                            <th class="px-4 py-3 text-right">"KSBs"</th>
                            <th class="px-4 py-3 text-right">"Enrolled"</th>
                        </tr>
                    </thead>
                    <tbody class="text-slate-300">
                        {PATHWAYS.iter().map(|p| view! {
                            <tr class="border-t border-slate-800 hover:bg-slate-800/30 transition-colors">
                                <td class="px-4 py-3 text-sm font-medium text-white">{p.name}</td>
                                <td class="px-4 py-3 text-xs text-slate-500 font-mono">{p.stages}</td>
                                <td class="px-4 py-3 text-xs text-cyan-400 font-mono text-right">{p.ksbs.to_string()}</td>
                                <td class="px-4 py-3 text-xs text-slate-400 font-mono text-right">{p.enrolled.to_string()}</td>
                            </tr>
                        }).collect_view()}
                    </tbody>
                </table>
            </div>
        </div>
    }
}
