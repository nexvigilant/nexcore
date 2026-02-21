//! Generated Academy curriculum map for EMA GVP modules.

use super::gvp_data::{guardian_seed_for_module, GvpModule, GVP_MODULES};
use leptos::prelude::*;

#[derive(Clone)]
struct GeneratedLesson {
    title: String,
    objective: String,
    duration_minutes: u16,
}

fn generate_lessons(m: &GvpModule) -> Vec<GeneratedLesson> {
    if m.status == "Void" {
        return vec![GeneratedLesson {
            title: format!("Module {} Orientation", m.code),
            objective:
                "Understand why this module is marked void and identify linked EMA guidance."
                    .to_string(),
            duration_minutes: 20,
        }];
    }

    vec![
        GeneratedLesson {
            title: format!("Module {} Foundations", m.code),
            objective: format!(
                "Define the scope and intent of GVP Module {} for operational PV.",
                m.code
            ),
            duration_minutes: 35,
        },
        GeneratedLesson {
            title: format!("Module {} Regulatory Requirements", m.code),
            objective: "Translate EMA expectations into SOP-level implementation controls."
                .to_string(),
            duration_minutes: 45,
        },
        GeneratedLesson {
            title: format!("Module {} Applied Case Workshop", m.code),
            objective: "Apply requirements to a realistic post-authorisation safety scenario."
                .to_string(),
            duration_minutes: 50,
        },
        GeneratedLesson {
            title: format!("Module {} Competency Check", m.code),
            objective: "Demonstrate evidence-backed execution against mapped KSB domains."
                .to_string(),
            duration_minutes: 30,
        },
    ]
}

#[component]
pub fn GvpCurriculumPage() -> impl IntoView {
    let total_lessons = GVP_MODULES
        .iter()
        .map(|m| generate_lessons(m).len() as u32)
        .sum::<u32>();

    view! {
        <div class="mx-auto max-w-7xl px-4 py-12">
            <header class="mb-10">
                <p class="text-[11px] font-bold text-cyan-400 uppercase tracking-[0.2em] font-mono">"Academy Curriculum Engine"</p>
                <h1 class="mt-2 text-4xl font-bold text-white font-mono uppercase tracking-tight">"EMA GVP Module Integration Matrix"</h1>
                <p class="mt-3 text-slate-400 max-w-4xl leading-relaxed">
                    "Auto-generated learning blueprint mapping every EMA GVP module to Academy pathways and KSB domains, with generated lesson plans ready for content authoring."
                </p>
                <div class="mt-4 flex flex-wrap gap-3 text-xs font-mono">
                    <span class="rounded-full border border-slate-700 bg-slate-900/50 px-3 py-1 text-slate-300">"Modules: 16"</span>
                    <span class="rounded-full border border-slate-700 bg-slate-900/50 px-3 py-1 text-slate-300">{format!("Generated Lessons: {total_lessons}")}</span>
                    <a href="/academy/gvp-modules" class="rounded-full border border-cyan-500/30 bg-cyan-500/10 px-3 py-1 text-cyan-300 hover:text-cyan-200 transition-colors">"Open Module Catalog"</a>
                    <a href="/academy/gvp-progress" class="rounded-full border border-emerald-500/30 bg-emerald-500/10 px-3 py-1 text-emerald-300 hover:text-emerald-200 transition-colors">"Track Progress"</a>
                    <a href="/academy/gvp-assessments" class="rounded-full border border-amber-500/30 bg-amber-500/10 px-3 py-1 text-amber-300 hover:text-amber-200 transition-colors">"Open Assessments"</a>
                    <a href="/academy/gvp-practicum" class="rounded-full border border-emerald-500/30 bg-emerald-500/10 px-3 py-1 text-emerald-300 hover:text-emerald-200 transition-colors">"Open Practicum"</a>
                    <a href="/vigilance/guardian" class="rounded-full border border-emerald-500/30 bg-emerald-500/10 px-3 py-1 text-emerald-300 hover:text-emerald-200 transition-colors">"Apply in Guardian"</a>
                </div>
            </header>

            <div class="space-y-5">
                {GVP_MODULES.into_iter().map(|module| {
                    let lessons = generate_lessons(&module);
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
                        <section class="rounded-2xl border border-slate-800 bg-slate-900/40 p-5">
                            <div class="flex flex-wrap items-start justify-between gap-4">
                                <div>
                                    <p class="text-[10px] font-bold text-slate-500 uppercase tracking-widest font-mono">{"Module "}{module.code}</p>
                                    <h2 class="mt-1 text-lg font-semibold text-white">{module.title}</h2>
                                    <p class="mt-1 text-sm text-slate-400">{module.pathway}</p>
                                </div>
                                <span class=format!("rounded-full border px-2.5 py-1 text-[10px] font-bold uppercase tracking-widest font-mono {status_cls}")>
                                    {module.status}
                                </span>
                            </div>

                            <div class="mt-4 flex flex-wrap gap-2">
                                {module.ksb_domains.iter().map(|domain| view! {
                                    <a href="/academy/skills" class="rounded-md border border-cyan-500/20 bg-cyan-500/5 px-2.5 py-1 text-[10px] font-bold font-mono text-cyan-300 hover:text-cyan-200 transition-colors">
                                        {format!("KSB {}", domain)}
                                    </a>
                                }).collect_view()}
                            </div>

                            <div class="mt-4">
                                <a
                                    href=format!("/academy/gvp-modules/{}", module.code)
                                    class="inline-flex items-center gap-2 text-xs font-bold text-cyan-400 hover:text-cyan-300 transition-colors uppercase tracking-widest font-mono"
                                >
                                    "Open Module Workspace"
                                    <span>"→"</span>
                                </a>
                                <a
                                    href=guardian_href
                                    class="ml-4 inline-flex items-center gap-2 text-xs font-bold text-emerald-400 hover:text-emerald-300 transition-colors uppercase tracking-widest font-mono"
                                >
                                    "Launch in Guardian"
                                    <span>"→"</span>
                                </a>
                            </div>

                            <div class="mt-4 grid gap-3 md:grid-cols-2">
                                {lessons.into_iter().map(|lesson| view! {
                                    <article class="rounded-xl border border-slate-800 bg-slate-950/40 p-4">
                                        <h3 class="text-sm font-semibold text-white">{lesson.title}</h3>
                                        <p class="mt-1 text-xs text-slate-400 leading-relaxed">{lesson.objective}</p>
                                        <p class="mt-2 text-[10px] font-mono text-slate-500 uppercase tracking-widest">{format!("{} min", lesson.duration_minutes)}</p>
                                    </article>
                                }).collect_view()}
                            </div>
                        </section>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::generate_lessons;
    use crate::sections::academy::gvp_data::GVP_MODULES;

    #[test]
    fn generated_lessons_cover_all_modules() {
        let total = GVP_MODULES
            .iter()
            .map(|m| generate_lessons(m).len())
            .sum::<usize>();
        assert!(total >= 16);
    }

    #[test]
    fn void_modules_have_orientation_only() {
        let void_count = GVP_MODULES
            .iter()
            .filter(|m| m.status == "Void")
            .filter(|m| generate_lessons(m).len() == 1)
            .count();
        assert_eq!(void_count, 4);
    }
}
