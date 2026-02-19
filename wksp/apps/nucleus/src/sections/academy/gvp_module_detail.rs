//! Deep-dive workspace for a single EMA GVP module.

use super::gvp_data::{
    guardian_seed_for_module, gvp_module_by_code, latest_module_evidence, load_guardian_writeback,
    module_evidence_count, EMA_GVP_URL,
};
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

#[derive(Clone)]
struct ModuleLesson {
    title: String,
    objective: String,
    duration_minutes: u16,
}

fn lessons_for(status: &str, code: &str) -> Vec<ModuleLesson> {
    if status == "Void" {
        return vec![ModuleLesson {
            title: format!("Module {code} Orientation"),
            objective:
                "Understand module void status and identify equivalent current EMA guidance."
                    .to_string(),
            duration_minutes: 20,
        }];
    }

    vec![
        ModuleLesson {
            title: format!("Module {code} Core Concepts"),
            objective: "Interpret scope, legal basis, and operational intent.".to_string(),
            duration_minutes: 35,
        },
        ModuleLesson {
            title: format!("Module {code} Implementation Controls"),
            objective: "Translate requirements into SOP checkpoints and QA evidence.".to_string(),
            duration_minutes: 45,
        },
        ModuleLesson {
            title: format!("Module {code} Case Application"),
            objective: "Apply requirements to a realistic post-authorisation scenario.".to_string(),
            duration_minutes: 50,
        },
        ModuleLesson {
            title: format!("Module {code} Competency Validation"),
            objective: "Pass objective checks against mapped KSB outcomes.".to_string(),
            duration_minutes: 30,
        },
    ]
}

#[component]
pub fn GvpModuleDetailPage() -> impl IntoView {
    let params = use_params_map();
    let code = move || params.get().get("code").unwrap_or_default();
    let evidence_rows = RwSignal::new(load_guardian_writeback());

    view! {
        <div class="mx-auto max-w-6xl px-4 py-12">
            <a href="/academy/gvp-modules" class="text-xs font-mono text-cyan-400 hover:text-cyan-300 transition-colors uppercase tracking-widest">
                "< Back to GVP Modules"
            </a>
            <a href="/academy/gvp-progress" class="ml-4 text-xs font-mono text-emerald-400 hover:text-emerald-300 transition-colors uppercase tracking-widest">
                "Track Progress"
            </a>
            <a href="/academy/gvp-assessments" class="ml-4 text-xs font-mono text-amber-400 hover:text-amber-300 transition-colors uppercase tracking-widest">
                "Assessments"
            </a>
            <a href="/academy/gvp-practicum" class="ml-4 text-xs font-mono text-emerald-400 hover:text-emerald-300 transition-colors uppercase tracking-widest">
                "Practicum"
            </a>
            <a href="/vigilance/guardian" class="ml-4 text-xs font-mono text-emerald-400 hover:text-emerald-300 transition-colors uppercase tracking-widest">
                "Apply in Guardian"
            </a>
            {move || {
                let module = gvp_module_by_code(&code());
                match module {
                    Some(m) => {
                        let lessons = lessons_for(m.status, m.code);
                        let (drug, event, count) = guardian_seed_for_module(m.code);
                        let evidence_count = module_evidence_count(m.code, &evidence_rows.get());
                        let latest = latest_module_evidence(m.code, &evidence_rows.get());
                        let guardian_href = format!(
                            "/vigilance/guardian?module={}&drug={}&event={}&count={}",
                            m.code, drug, event, count
                        );
                        let status_cls = if m.status == "Final" {
                            "text-emerald-400 bg-emerald-500/10 border-emerald-500/20"
                        } else {
                            "text-amber-400 bg-amber-500/10 border-amber-500/20"
                        };
                        view! {
                            <>
                                <header class="mt-6 mb-8">
                                    <div class="flex flex-wrap items-center justify-between gap-4">
                                        <div>
                                            <p class="text-[10px] font-bold text-slate-500 uppercase tracking-widest font-mono">{"Module "}{m.code}</p>
                                            <h1 class="mt-1 text-3xl font-bold text-white font-mono uppercase tracking-tight">{m.title}</h1>
                                            <p class="mt-2 text-slate-400">{m.pathway}</p>
                                        </div>
                                        <span class=format!("rounded-full border px-3 py-1 text-[10px] font-bold uppercase tracking-widest font-mono {status_cls}")>
                                            {m.status}
                                        </span>
                                    </div>
                                    <p class="mt-4 text-sm text-slate-300 leading-relaxed">{m.note}</p>
                                    <div class="mt-4 flex flex-wrap gap-2">
                                        {m.ksb_domains.iter().map(|d| view! {
                                            <a href="/academy/skills" class="rounded-md border border-cyan-500/20 bg-cyan-500/5 px-2.5 py-1 text-[10px] font-bold font-mono text-cyan-300 hover:text-cyan-200 transition-colors">
                                                {format!("KSB {}", d)}
                                            </a>
                                        }).collect_view()}
                                        {if evidence_count > 0 {
                                            view! {
                                                <span class="rounded-md border border-emerald-500/30 bg-emerald-500/10 px-2.5 py-1 text-[10px] font-bold font-mono text-emerald-300 uppercase tracking-widest">
                                                    {format!("Guardian Evidence {}", evidence_count)}
                                                </span>
                                            }.into_any()
                                        } else {
                                            view! { <span></span> }.into_any()
                                        }}
                                        <a
                                            href=guardian_href
                                            class="rounded-md border border-emerald-500/30 bg-emerald-500/10 px-2.5 py-1 text-[10px] font-bold font-mono text-emerald-300 hover:text-emerald-200 transition-colors uppercase tracking-widest"
                                        >
                                            "Launch Scenario"
                                        </a>
                                    </div>
                                    {latest.map(|entry| view! {
                                        <div class="mt-4 rounded-xl border border-emerald-500/20 bg-emerald-500/5 p-4">
                                            <p class="text-[10px] font-bold text-emerald-300 uppercase tracking-widest font-mono">"Latest Guardian Writeback"</p>
                                            <p class="mt-1 text-xs text-slate-300">
                                                {format!(
                                                    "{} / {} / {} cases — {} ({:.2})",
                                                    entry.drug_name,
                                                    entry.event_name,
                                                    entry.case_count,
                                                    entry.risk_level,
                                                    entry.risk_score
                                                )}
                                            </p>
                                            <p class="mt-1 text-[10px] text-slate-500 font-mono">{entry.recorded_at}</p>
                                            <a href="/academy/gvp-progress" class="mt-2 inline-flex text-[10px] font-bold text-emerald-300 hover:text-emerald-200 uppercase tracking-widest font-mono">
                                                "View Progress Ledger"
                                            </a>
                                        </div>
                                    })}
                                </header>

                                <section class="rounded-2xl border border-slate-800 bg-slate-900/40 p-6">
                                    <div class="flex items-center justify-between gap-3">
                                        <h2 class="text-sm font-bold text-white uppercase tracking-widest font-mono">"Generated Lesson Stack"</h2>
                                        <a
                                            href=EMA_GVP_URL
                                            target="_blank"
                                            rel="noopener noreferrer"
                                            class="text-xs font-bold text-cyan-400 hover:text-cyan-300 transition-colors uppercase tracking-widest font-mono"
                                        >
                                            "Open EMA Source ↗"
                                        </a>
                                    </div>
                                    <div class="mt-4 grid gap-3 md:grid-cols-2">
                                        {lessons.into_iter().map(|l| view! {
                                            <article class="rounded-xl border border-slate-800 bg-slate-950/40 p-4">
                                                <h3 class="text-sm font-semibold text-white">{l.title}</h3>
                                                <p class="mt-1 text-xs text-slate-400 leading-relaxed">{l.objective}</p>
                                                <p class="mt-2 text-[10px] font-mono text-slate-500 uppercase tracking-widest">{format!("{} min", l.duration_minutes)}</p>
                                            </article>
                                        }).collect_view()}
                                    </div>
                                </section>
                            </>
                        }.into_any()
                    }
                    None => view! {
                        <div class="mt-8 rounded-xl border border-red-500/20 bg-red-500/10 p-6">
                            <p class="text-sm text-red-300 font-mono">"Unknown GVP module code."</p>
                        </div>
                    }.into_any(),
                }
            }}
        </div>
    }
}
