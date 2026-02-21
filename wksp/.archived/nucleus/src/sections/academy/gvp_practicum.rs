//! Scenario-based bridge from GVP learning into Guardian execution.

use super::gvp_data::{guardian_seed_for_module, GVP_MODULES};
use leptos::prelude::*;

#[component]
pub fn GvpPracticumPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-7xl px-4 py-12">
            <header class="mb-10">
                <p class="text-[11px] font-bold text-cyan-400 uppercase tracking-[0.2em] font-mono">"Academy Practicum"</p>
                <h1 class="mt-2 text-4xl font-bold text-white font-mono uppercase tracking-tight">"GVP to Guardian Execution Lab"</h1>
                <p class="mt-3 text-slate-400 max-w-4xl leading-relaxed">
                    "Run module-aligned safety scenarios directly in Guardian. Each launcher preloads the risk evaluation panel with a training signal."
                </p>
                <div class="mt-5 flex flex-wrap gap-3">
                    <a href="/academy/gvp-modules" class="rounded-full border border-cyan-500/30 bg-cyan-500/10 px-3 py-1 text-xs font-bold text-cyan-300 hover:text-cyan-200 transition-colors uppercase tracking-widest font-mono">
                        "Modules"
                    </a>
                    <a href="/academy/gvp-assessments" class="rounded-full border border-amber-500/30 bg-amber-500/10 px-3 py-1 text-xs font-bold text-amber-300 hover:text-amber-200 transition-colors uppercase tracking-widest font-mono">
                        "Assessments"
                    </a>
                    <a href="/academy/gvp-progress" class="rounded-full border border-emerald-500/30 bg-emerald-500/10 px-3 py-1 text-xs font-bold text-emerald-300 hover:text-emerald-200 transition-colors uppercase tracking-widest font-mono">
                        "Progress"
                    </a>
                </div>
            </header>

            <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
                {GVP_MODULES
                    .iter()
                    .map(|module| {
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

                        view! {
                            <article class="rounded-xl border border-slate-800 bg-slate-900/40 p-5 hover:border-slate-700 transition-colors">
                                <div class="flex items-start justify-between gap-3">
                                    <div>
                                        <p class="text-[10px] font-bold text-slate-500 uppercase tracking-widest font-mono">
                                            {"Module "}{module.code}
                                        </p>
                                        <h2 class="mt-1 text-base font-semibold text-white leading-snug">{module.title}</h2>
                                    </div>
                                    <span class=format!("rounded-full border px-2.5 py-1 text-[10px] font-bold uppercase tracking-widest font-mono {status_cls}")>
                                        {module.status}
                                    </span>
                                </div>

                                <div class="mt-4 space-y-1.5 text-[11px] font-mono text-slate-400">
                                    <p>{format!("Seed Drug: {}", drug)}</p>
                                    <p>{format!("Seed Event: {}", event)}</p>
                                    <p>{format!("Case Count: {}", count)}</p>
                                </div>

                                <div class="mt-5 flex items-center justify-between">
                                    <a
                                        href=launch_href
                                        class="text-xs font-bold text-emerald-400 hover:text-emerald-300 transition-colors uppercase tracking-widest font-mono"
                                    >
                                        "Launch in Guardian"
                                    </a>
                                    <a
                                        href=format!("/academy/gvp-modules/{}", module.code)
                                        class="text-[10px] text-cyan-400 hover:text-cyan-300 transition-colors font-mono uppercase tracking-widest"
                                    >
                                        "Open Module"
                                    </a>
                                </div>
                            </article>
                        }
                    })
                    .collect_view()}
            </div>
        </div>
    }
}
