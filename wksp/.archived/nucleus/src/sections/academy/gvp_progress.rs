//! Learner-facing progress tracker for EMA GVP module completion.

use super::gvp_data::{
    clear_guardian_writeback, clear_gvp_assessment_passes, guardian_seed_for_module,
    has_assessment_pass, load_guardian_writeback, load_gvp_assessment_passes,
    module_evidence_count, GVP_MODULES,
};
use leptos::prelude::*;

#[component]
pub fn GvpProgressPage() -> impl IntoView {
    let manual_completions = RwSignal::new(vec![false; GVP_MODULES.len()]);
    let evidence_rows = RwSignal::new(load_guardian_writeback());
    let assessment_rows = RwSignal::new(load_gvp_assessment_passes());

    let completed_count = Signal::derive(move || {
        GVP_MODULES
            .iter()
            .enumerate()
            .filter(|(idx, module)| {
                manual_completions.get().get(*idx).copied().unwrap_or(false)
                    || module_evidence_count(module.code, &evidence_rows.get()) > 0
            })
            .count() as u32
    });
    let evidence_completed_count = Signal::derive(move || {
        GVP_MODULES
            .iter()
            .filter(|module| module_evidence_count(module.code, &evidence_rows.get()) > 0)
            .count() as u32
    });
    let ready_count = Signal::derive(move || {
        GVP_MODULES
            .iter()
            .filter(|module| {
                module_evidence_count(module.code, &evidence_rows.get()) > 0
                    && has_assessment_pass(module.code, &assessment_rows.get())
            })
            .count() as u32
    });
    let manual_only_count = Signal::derive(move || {
        GVP_MODULES
            .iter()
            .enumerate()
            .filter(|(idx, module)| {
                manual_completions.get().get(*idx).copied().unwrap_or(false)
                    && module_evidence_count(module.code, &evidence_rows.get()) == 0
            })
            .count() as u32
    });
    let completion_pct =
        Signal::derive(move || ((completed_count.get() as f32 / 16.0) * 100.0).round() as u32);
    let evidence_completion_pct = Signal::derive(move || {
        ((evidence_completed_count.get() as f32 / 16.0) * 100.0).round() as u32
    });

    view! {
        <div class="mx-auto max-w-6xl px-4 py-12">
            <header class="mb-8">
                <p class="text-[11px] font-bold text-cyan-400 uppercase tracking-[0.2em] font-mono">"Academy Tracker"</p>
                <h1 class="mt-2 text-4xl font-bold text-white font-mono uppercase tracking-tight">"GVP Progress"</h1>
                <p class="mt-3 text-slate-400 max-w-3xl">
                    "Track your completion across EMA GVP modules and monitor readiness at a glance."
                </p>
                <div class="mt-4 flex flex-wrap gap-3">
                    <a href="/academy/gvp-assessments" class="rounded-lg border border-amber-500/30 bg-amber-500/10 px-4 py-2 text-xs font-bold text-amber-300 hover:text-amber-200 transition-colors uppercase tracking-widest font-mono">
                        "Open Assessments"
                    </a>
                    <a href="/academy/gvp-practicum" class="rounded-lg border border-emerald-500/30 bg-emerald-500/10 px-4 py-2 text-xs font-bold text-emerald-300 hover:text-emerald-200 transition-colors uppercase tracking-widest font-mono">
                        "Open Practicum"
                    </a>
                    <a href="/vigilance/guardian" class="rounded-lg border border-emerald-500/30 bg-emerald-500/10 px-4 py-2 text-xs font-bold text-emerald-300 hover:text-emerald-200 transition-colors uppercase tracking-widest font-mono">
                        "Apply in Guardian"
                    </a>
                    <a href="/academy/evidence-ledger" class="rounded-lg border border-cyan-500/30 bg-cyan-500/10 px-4 py-2 text-xs font-bold text-cyan-300 hover:text-cyan-200 transition-colors uppercase tracking-widest font-mono">
                        "Evidence Ledger"
                    </a>
                </div>
            </header>

            <section class="mb-8 rounded-2xl border border-slate-800 bg-slate-900/40 p-6">
                <div class="flex flex-wrap items-center justify-between gap-4">
                    <div>
                        <p class="text-[10px] font-bold text-slate-500 uppercase tracking-widest font-mono">"Completed Modules"</p>
                        <p class="mt-1 text-3xl font-bold text-white font-mono">{move || format!("{}/16", completed_count.get())}</p>
                        <p class="mt-1 text-[10px] text-emerald-300 font-mono uppercase tracking-widest">
                            {move || format!("Evidence-backed: {}/16", evidence_completed_count.get())}
                        </p>
                        <p class="mt-1 text-[10px] text-amber-300 font-mono uppercase tracking-widest">
                            {move || format!("Manual-only: {}", manual_only_count.get())}
                        </p>
                    </div>
                    <div class="min-w-[220px] flex-1 max-w-md">
                        <div class="flex justify-between items-center mb-2">
                            <span class="text-[10px] font-bold text-slate-500 uppercase tracking-widest font-mono">"Completion"</span>
                            <span class="text-[10px] font-bold text-cyan-300 font-mono">{move || format!("{}%", completion_pct.get())}</span>
                        </div>
                        <div class="h-2 w-full rounded-full bg-slate-800 overflow-hidden">
                            <div class="h-full bg-cyan-500 transition-all duration-300" style=move || format!("width: {}%", completion_pct.get())></div>
                        </div>
                        <div class="mt-3 flex justify-between items-center mb-2">
                            <span class="text-[10px] font-bold text-slate-500 uppercase tracking-widest font-mono">"Operational Readiness"</span>
                            <span class="text-[10px] font-bold text-emerald-300 font-mono">{move || format!("{}%", evidence_completion_pct.get())}</span>
                        </div>
                        <div class="h-2 w-full rounded-full bg-slate-800 overflow-hidden">
                            <div class="h-full bg-emerald-500 transition-all duration-300" style=move || format!("width: {}%", evidence_completion_pct.get())></div>
                        </div>
                    </div>
                    <button
                        class="rounded-lg border border-slate-700 px-3 py-2 text-xs font-bold text-slate-300 hover:text-white transition-colors uppercase tracking-widest font-mono"
                        on:click=move |_| manual_completions.set(vec![false; GVP_MODULES.len()])
                    >
                        "Reset Manual Flags"
                    </button>
                    <button
                        class="rounded-lg border border-rose-500/30 bg-rose-500/10 px-3 py-2 text-xs font-bold text-rose-300 hover:text-rose-200 transition-colors uppercase tracking-widest font-mono"
                        on:click=move |_| {
                            clear_guardian_writeback();
                            evidence_rows.set(Vec::new());
                        }
                    >
                        "Clear Guardian Evidence"
                    </button>
                    <button
                        class="rounded-lg border border-amber-500/30 bg-amber-500/10 px-3 py-2 text-xs font-bold text-amber-300 hover:text-amber-200 transition-colors uppercase tracking-widest font-mono"
                        on:click=move |_| {
                            clear_gvp_assessment_passes();
                            assessment_rows.set(Vec::new());
                        }
                    >
                        "Clear Assessment Passes"
                    </button>
                </div>
                <div class="mt-4 rounded-xl border border-emerald-500/20 bg-emerald-500/5 p-3">
                    <p class="text-[10px] font-bold text-emerald-300 uppercase tracking-widest font-mono">
                        {move || format!("Deployment Ready: {}/16 (assessment + evidence)", ready_count.get())}
                    </p>
                </div>
            </section>

            <div class="space-y-3">
                {GVP_MODULES.into_iter().enumerate().map(|(idx, module)| {
                    let (drug, event, count) = guardian_seed_for_module(module.code);
                    let guardian_href = format!(
                        "/vigilance/guardian?module={}&drug={}&event={}&count={}",
                        module.code, drug, event, count
                    );
                    let status_cls = if module.status == "Final" {
                        "text-emerald-400 bg-emerald-500/10 border-emerald-500/20"
                    } else {
                        "text-amber-400 bg-amber-500/10 border-amber-500/20"
                    };
                    view! {
                        <article class="rounded-xl border border-slate-800 bg-slate-900/30 p-4 flex flex-wrap items-center justify-between gap-4">
                            <div class="min-w-[260px]">
                                <p class="text-[10px] font-bold text-slate-500 uppercase tracking-widest font-mono">{"Module "}{module.code}</p>
                                <h2 class="text-sm font-semibold text-white mt-1">{module.title}</h2>
                                <p class="text-xs text-slate-400 mt-1">{module.pathway}</p>
                            </div>
                            <div class="flex items-center gap-3">
                                <span class=format!("rounded-full border px-2.5 py-1 text-[10px] font-bold uppercase tracking-widest font-mono {status_cls}")>
                                    {module.status}
                                </span>
                                {move || {
                                    let evidence_count = module_evidence_count(module.code, &evidence_rows.get());
                                    if evidence_count > 0 {
                                        view! {
                                            <span class="rounded-full border border-emerald-500/30 bg-emerald-500/10 px-2.5 py-1 text-[10px] font-bold uppercase tracking-widest font-mono text-emerald-300">
                                                {format!("Evidence {}", evidence_count)}
                                            </span>
                                        }.into_any()
                                    } else {
                                        view! { <span></span> }.into_any()
                                    }
                                }}
                                {move || {
                                    if has_assessment_pass(module.code, &assessment_rows.get()) {
                                        view! {
                                            <span class="rounded-full border border-cyan-500/30 bg-cyan-500/10 px-2.5 py-1 text-[10px] font-bold uppercase tracking-widest font-mono text-cyan-300">
                                                "Assessment Passed"
                                            </span>
                                        }.into_any()
                                    } else {
                                        view! { <span></span> }.into_any()
                                    }
                                }}
                                <a href=format!("/academy/gvp-modules/{}", module.code) class="text-xs font-bold text-cyan-400 hover:text-cyan-300 transition-colors uppercase tracking-widest font-mono">
                                    "Open"
                                </a>
                                <a href=guardian_href class="text-xs font-bold text-emerald-400 hover:text-emerald-300 transition-colors uppercase tracking-widest font-mono">
                                    "Launch"
                                </a>
                                <button
                                    class=move || {
                                        let manual_done = manual_completions.get().get(idx).copied().unwrap_or(false);
                                        let evidence_done = module_evidence_count(module.code, &evidence_rows.get()) > 0;
                                        if manual_done || evidence_done {
                                            "rounded-lg border border-emerald-500/30 bg-emerald-500/10 px-3 py-1.5 text-[10px] font-bold text-emerald-300 uppercase tracking-widest font-mono"
                                        } else {
                                            "rounded-lg border border-slate-700 bg-slate-900 px-3 py-1.5 text-[10px] font-bold text-slate-300 hover:text-white transition-colors uppercase tracking-widest font-mono"
                                        }
                                    }
                                    on:click=move |_| {
                                        manual_completions.update(|v| {
                                            if let Some(item) = v.get_mut(idx) {
                                                *item = !*item;
                                            }
                                        });
                                    }
                                >
                                    {move || {
                                        let manual_done = manual_completions.get().get(idx).copied().unwrap_or(false);
                                        let evidence_done = module_evidence_count(module.code, &evidence_rows.get()) > 0;
                                        let assessment_done = has_assessment_pass(module.code, &assessment_rows.get());
                                        if evidence_done && assessment_done {
                                            "Ready"
                                        } else if evidence_done {
                                            "Evidence Complete"
                                        } else if manual_done {
                                            "Manual Complete"
                                        } else {
                                            "Mark Complete"
                                        }
                                    }}
                                </button>
                                {move || {
                                    let manual_done = manual_completions.get().get(idx).copied().unwrap_or(false);
                                    let evidence_done = module_evidence_count(module.code, &evidence_rows.get()) > 0;
                                    let assessment_done = has_assessment_pass(module.code, &assessment_rows.get());
                                    if evidence_done && assessment_done {
                                        view! {
                                            <span class="text-[10px] font-bold text-emerald-300 uppercase tracking-widest font-mono">
                                                "Ready"
                                            </span>
                                        }.into_any()
                                    } else if manual_done && !evidence_done {
                                        view! {
                                            <span class="text-[10px] font-bold text-amber-300 uppercase tracking-widest font-mono">
                                                "Needs Guardian Evidence"
                                            </span>
                                        }.into_any()
                                    } else if evidence_done && !assessment_done {
                                        view! {
                                            <span class="text-[10px] font-bold text-amber-300 uppercase tracking-widest font-mono">
                                                "Needs Assessment Pass"
                                            </span>
                                        }.into_any()
                                    } else {
                                        view! { <span></span> }.into_any()
                                    }
                                }}
                            </div>
                        </article>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}
