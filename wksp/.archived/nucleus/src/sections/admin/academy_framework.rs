//! Admin: Academy capability framework — manage competency areas, EPAs, and CPAs

use leptos::prelude::*;

struct CompetencyArea {
    name: &'static str,
    epas: u32,
    cpas: u32,
    mapped_ksbs: u32,
    color: &'static str,
    completion: u8,
}

const AREAS: &[CompetencyArea] = &[
    CompetencyArea {
        name: "Signal Detection & Management",
        epas: 4,
        cpas: 8,
        mapped_ksbs: 42,
        color: "text-red-400",
        completion: 85,
    },
    CompetencyArea {
        name: "Individual Case Safety Reports",
        epas: 3,
        cpas: 6,
        mapped_ksbs: 36,
        color: "text-cyan-400",
        completion: 92,
    },
    CompetencyArea {
        name: "Aggregate Safety Reporting",
        epas: 4,
        cpas: 7,
        mapped_ksbs: 34,
        color: "text-amber-400",
        completion: 78,
    },
    CompetencyArea {
        name: "Risk Management",
        epas: 3,
        cpas: 6,
        mapped_ksbs: 28,
        color: "text-violet-400",
        completion: 70,
    },
    CompetencyArea {
        name: "Benefit-Risk Assessment",
        epas: 3,
        cpas: 7,
        mapped_ksbs: 26,
        color: "text-emerald-400",
        completion: 65,
    },
    CompetencyArea {
        name: "Regulatory Intelligence",
        epas: 4,
        cpas: 8,
        mapped_ksbs: 38,
        color: "text-blue-400",
        completion: 88,
    },
    CompetencyArea {
        name: "PV System & Quality",
        epas: 3,
        cpas: 6,
        mapped_ksbs: 30,
        color: "text-orange-400",
        completion: 72,
    },
];

#[component]
pub fn AcademyFrameworkPage() -> impl IntoView {
    let total_epas: u32 = AREAS.iter().map(|a| a.epas).sum();
    let total_cpas: u32 = AREAS.iter().map(|a| a.cpas).sum();
    let total_ksbs: u32 = AREAS.iter().map(|a| a.mapped_ksbs).sum();

    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Capability Framework"</h1>
                    <p class="mt-1 text-slate-400">"Manage competency areas, entrustable professional activities, and clinical practice activities."</p>
                </div>
                <a href="/admin/academy/framework-browser" class="rounded-lg border border-slate-700 px-4 py-2 text-xs font-bold text-slate-400 hover:text-white hover:border-slate-600 transition-colors font-mono uppercase">"Browse"</a>
            </div>

            /* Stats */
            <div class="mt-6 grid gap-4 sm:grid-cols-4">
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5">
                    <p class="text-[9px] font-bold text-slate-500 uppercase tracking-widest font-mono">"Areas"</p>
                    <p class="text-2xl font-black text-white font-mono mt-2">{AREAS.len().to_string()}</p>
                </div>
                <div class="rounded-xl border border-cyan-500/20 bg-cyan-500/5 p-5">
                    <p class="text-[9px] font-bold text-cyan-400 uppercase tracking-widest font-mono">"EPAs"</p>
                    <p class="text-2xl font-black text-cyan-400 font-mono mt-2">{total_epas.to_string()}</p>
                </div>
                <div class="rounded-xl border border-amber-500/20 bg-amber-500/5 p-5">
                    <p class="text-[9px] font-bold text-amber-400 uppercase tracking-widest font-mono">"CPAs"</p>
                    <p class="text-2xl font-black text-amber-400 font-mono mt-2">{total_cpas.to_string()}</p>
                </div>
                <div class="rounded-xl border border-emerald-500/20 bg-emerald-500/5 p-5">
                    <p class="text-[9px] font-bold text-emerald-400 uppercase tracking-widest font-mono">"Mapped KSBs"</p>
                    <p class="text-2xl font-black text-emerald-400 font-mono mt-2">{total_ksbs.to_string()}</p>
                </div>
            </div>

            /* Competency Areas */
            <div class="mt-8 space-y-3">
                {AREAS.iter().map(|a| {
                    let bar_cls = if a.completion >= 80 { "bg-emerald-500" } else if a.completion >= 65 { "bg-amber-500" } else { "bg-red-500" };
                    view! {
                        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5 hover:border-amber-500/30 transition-colors cursor-pointer">
                            <div class="flex items-center justify-between">
                                <h3 class=format!("text-sm font-bold {}", a.color)>{a.name}</h3>
                                <div class="flex items-center gap-2">
                                    <div class="h-1.5 w-20 rounded-full bg-slate-800 overflow-hidden">
                                        <div class=format!("h-full rounded-full {bar_cls}") style=format!("width:{}%", a.completion)></div>
                                    </div>
                                    <span class="text-[10px] text-slate-500 font-mono">{format!("{}%", a.completion)}</span>
                                </div>
                            </div>
                            <div class="mt-3 flex items-center gap-6 text-xs font-mono">
                                <span class="text-cyan-400">{format!("{} EPAs", a.epas)}</span>
                                <span class="text-amber-400">{format!("{} CPAs", a.cpas)}</span>
                                <span class="text-slate-500">{format!("{} KSBs", a.mapped_ksbs)}</span>
                            </div>
                        </div>
                    }
                }).collect_view()}
            </div>

            <div class="mt-4 text-[10px] text-slate-600 font-mono">
                {format!("{} areas \u{00B7} {} EPAs \u{00B7} {} CPAs \u{00B7} {} KSBs", AREAS.len(), total_epas, total_cpas, total_ksbs)}
            </div>
        </div>
    }
}
