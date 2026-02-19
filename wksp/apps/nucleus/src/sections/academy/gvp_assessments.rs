//! EMA GVP module assessment hub.

use super::gvp_data::{
    guardian_seed_for_module, has_assessment_pass, load_guardian_writeback,
    load_gvp_assessment_passes, module_evidence_count, save_gvp_assessment_passes,
    GvpAssessmentPass, GvpModule, GVP_MODULES,
};
use leptos::prelude::*;

#[derive(Clone)]
struct GvpAssessmentBlueprint {
    module_code: &'static str,
    title: String,
    question_count: u16,
    duration_minutes: u16,
    pass_mark_pct: u8,
    format: &'static str,
}

fn assessment_blueprint(module: &GvpModule) -> GvpAssessmentBlueprint {
    if module.status == "Void" {
        return GvpAssessmentBlueprint {
            module_code: module.code,
            title: format!("Module {} Bridging Assessment", module.code),
            question_count: 8,
            duration_minutes: 12,
            pass_mark_pct: 70,
            format: "Guidance Mapping",
        };
    }

    GvpAssessmentBlueprint {
        module_code: module.code,
        title: format!("Module {} Competency Check", module.code),
        question_count: 20,
        duration_minutes: 25,
        pass_mark_pct: 80,
        format: "Scenario + MCQ",
    }
}

#[component]
pub fn GvpAssessmentsPage() -> impl IntoView {
    let include_void = RwSignal::new(true);
    let passes = RwSignal::new(load_gvp_assessment_passes());

    let total_assessments = Signal::derive(move || {
        if include_void.get() {
            GVP_MODULES.len()
        } else {
            GVP_MODULES.iter().filter(|m| m.status != "Void").count()
        }
    });

    view! {
        <div class="mx-auto max-w-7xl px-4 py-12">
            <header class="mb-10">
                <p class="text-[11px] font-bold text-cyan-400 uppercase tracking-[0.2em] font-mono">"Academy Assessment Grid"</p>
                <h1 class="mt-2 text-4xl font-bold text-white font-mono uppercase tracking-tight">"EMA GVP Module Assessments"</h1>
                <p class="mt-3 text-slate-400 max-w-4xl leading-relaxed">
                    "Assessment blueprints for all EMA GVP modules. Final modules use scenario-based competency checks; void modules use guidance-mapping bridge checks."
                </p>

                <div class="mt-5 flex flex-wrap items-center gap-3">
                    <a href="/academy/gvp-modules" class="rounded-full border border-cyan-500/30 bg-cyan-500/10 px-3 py-1 text-xs font-bold text-cyan-300 hover:text-cyan-200 transition-colors uppercase tracking-widest font-mono">
                        "Module Catalog"
                    </a>
                    <a href="/academy/gvp-curriculum" class="rounded-full border border-slate-700 bg-slate-900/40 px-3 py-1 text-xs font-bold text-slate-300 hover:text-white transition-colors uppercase tracking-widest font-mono">
                        "Curriculum"
                    </a>
                    <a href="/academy/gvp-progress" class="rounded-full border border-emerald-500/30 bg-emerald-500/10 px-3 py-1 text-xs font-bold text-emerald-300 hover:text-emerald-200 transition-colors uppercase tracking-widest font-mono">
                        "Progress"
                    </a>
                    <a href="/academy/gvp-practicum" class="rounded-full border border-emerald-500/30 bg-emerald-500/10 px-3 py-1 text-xs font-bold text-emerald-300 hover:text-emerald-200 transition-colors uppercase tracking-widest font-mono">
                        "Practicum"
                    </a>
                    <a href="/vigilance/guardian" class="rounded-full border border-emerald-500/30 bg-emerald-500/10 px-3 py-1 text-xs font-bold text-emerald-300 hover:text-emerald-200 transition-colors uppercase tracking-widest font-mono">
                        "Apply in Guardian"
                    </a>
                    <label class="ml-2 inline-flex items-center gap-2 text-xs font-bold text-slate-300 font-mono uppercase tracking-widest">
                        <input
                            type="checkbox"
                            prop:checked=include_void
                            on:change=move |ev| include_void.set(event_target_checked(&ev))
                            class="h-4 w-4 rounded border-slate-700 bg-slate-900 text-cyan-500"
                        />
                        "Include Void Modules"
                    </label>
                    <span class="rounded-full border border-slate-700 bg-slate-900/50 px-3 py-1 text-xs font-bold text-slate-300 font-mono">
                        {move || format!("Assessments: {}", total_assessments.get())}
                    </span>
                </div>
            </header>

            <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
                {move || GVP_MODULES
                    .iter()
                    .filter(|m| include_void.get() || m.status != "Void")
                    .map(|module| {
                        let bp = assessment_blueprint(module);
                        let (drug, event, count) = guardian_seed_for_module(module.code);
                        let launch_href = format!(
                            "/vigilance/guardian?module={}&drug={}&event={}&count={}",
                            module.code, drug, event, count
                        );
                        let status_cls = if module.status == "Final" {
                            "text-emerald-400 bg-emerald-500/10 border-emerald-500/20"
                        } else {
                            "text-amber-400 bg-amber-500/10 border-amber-500/20"
                        };
                        let passed = has_assessment_pass(module.code, &passes.get());
                        let evidenced = module_evidence_count(module.code, &load_guardian_writeback()) > 0;

                        view! {
                            <article class="rounded-xl border border-slate-800 bg-slate-900/40 p-5 hover:border-slate-700 transition-colors">
                                <div class="flex items-start justify-between gap-3">
                                    <div>
                                        <p class="text-[10px] font-bold text-slate-500 uppercase tracking-widest font-mono">
                                            {"Module "}{bp.module_code}
                                        </p>
                                        <h2 class="mt-1 text-base font-semibold text-white leading-snug">{bp.title}</h2>
                                    </div>
                                    <span class=format!("rounded-full border px-2.5 py-1 text-[10px] font-bold uppercase tracking-widest font-mono {status_cls}")>
                                        {module.status}
                                    </span>
                                </div>

                                <div class="mt-4 space-y-1.5 text-[11px] font-mono text-slate-400">
                                    <p>{format!("Format: {}", bp.format)}</p>
                                    <p>{format!("Questions: {}", bp.question_count)}</p>
                                    <p>{format!("Duration: {} min", bp.duration_minutes)}</p>
                                    <p>{format!("Pass Mark: {}%", bp.pass_mark_pct)}</p>
                                </div>
                                <div class="mt-3 flex flex-wrap gap-2">
                                    {if passed {
                                        view! { <span class="rounded-full border border-emerald-500/30 bg-emerald-500/10 px-2 py-0.5 text-[10px] font-bold text-emerald-300 uppercase tracking-widest font-mono">"Assessment Passed"</span> }.into_any()
                                    } else {
                                        view! { <span class="rounded-full border border-amber-500/30 bg-amber-500/10 px-2 py-0.5 text-[10px] font-bold text-amber-300 uppercase tracking-widest font-mono">"Assessment Pending"</span> }.into_any()
                                    }}
                                    {if evidenced {
                                        view! { <span class="rounded-full border border-emerald-500/30 bg-emerald-500/10 px-2 py-0.5 text-[10px] font-bold text-emerald-300 uppercase tracking-widest font-mono">"Execution Verified"</span> }.into_any()
                                    } else {
                                        view! { <span class="rounded-full border border-slate-700 bg-slate-900/50 px-2 py-0.5 text-[10px] font-bold text-slate-400 uppercase tracking-widest font-mono">"Execution Pending"</span> }.into_any()
                                    }}
                                </div>

                                <div class="mt-5 flex items-center justify-between">
                                    <a
                                        href=format!("/academy/gvp-modules/{}", module.code)
                                        class="text-xs font-bold text-cyan-400 hover:text-cyan-300 transition-colors uppercase tracking-widest font-mono"
                                    >
                                        "Open Workspace"
                                    </a>
                                    <a
                                        href=launch_href
                                        class="text-[10px] text-emerald-400 hover:text-emerald-300 transition-colors font-mono uppercase tracking-widest"
                                    >
                                        "Launch in Guardian"
                                    </a>
                                </div>
                                <div class="mt-3 flex flex-wrap gap-2">
                                    <button
                                        on:click=move |_| {
                                            let mut next = passes.get();
                                            let now_epoch = std::time::SystemTime::now()
                                                .duration_since(std::time::UNIX_EPOCH)
                                                .map(|d| d.as_secs())
                                                .unwrap_or_default();
                                            let rec = GvpAssessmentPass {
                                                module_code: module.code.to_string(),
                                                score_pct: bp.pass_mark_pct,
                                                passed: true,
                                                recorded_at: format!("unix:{now_epoch}"),
                                            };
                                            if let Some(idx) = next.iter().position(|r| r.module_code.eq_ignore_ascii_case(module.code)) {
                                                next[idx] = rec;
                                            } else {
                                                next.push(rec);
                                            }
                                            save_gvp_assessment_passes(&next);
                                            passes.set(next);
                                        }
                                        class="rounded-lg border border-emerald-500/30 bg-emerald-500/10 px-2.5 py-1.5 text-[10px] font-bold text-emerald-300 hover:text-emerald-200 uppercase tracking-widest font-mono"
                                    >
                                        "Record Pass"
                                    </button>
                                    <button
                                        on:click=move |_| {
                                            let mut next = passes.get();
                                            next.retain(|r| !r.module_code.eq_ignore_ascii_case(module.code));
                                            save_gvp_assessment_passes(&next);
                                            passes.set(next);
                                        }
                                        class="rounded-lg border border-slate-700 bg-slate-900/70 px-2.5 py-1.5 text-[10px] font-bold text-slate-300 hover:text-white uppercase tracking-widest font-mono"
                                    >
                                        "Clear Pass"
                                    </button>
                                </div>
                            </article>
                        }
                    })
                    .collect_view()}
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::assessment_blueprint;
    use crate::sections::academy::gvp_data::GVP_MODULES;

    #[test]
    fn has_one_assessment_per_module() {
        assert_eq!(GVP_MODULES.len(), 16);
        let generated = GVP_MODULES
            .iter()
            .map(assessment_blueprint)
            .collect::<Vec<_>>();
        assert_eq!(generated.len(), 16);
    }
}
