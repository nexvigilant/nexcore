//! EMA GVP modules integration page for Academy.

use super::gvp_data::{guardian_seed_for_module, EMA_GVP_URL, GVP_MODULES};
use leptos::prelude::*;

#[component]
pub fn GvpModulesPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-12">
            <header class="mb-8">
                <p class="text-[11px] font-bold text-cyan-400 uppercase tracking-[0.2em] font-mono">"EMA Curriculum"</p>
                <h1 class="mt-2 text-4xl font-bold text-white font-mono uppercase tracking-tight">"GVP Modules I–XVI"</h1>
                <p class="mt-3 text-slate-400 max-w-3xl">
                    "Integrated reference track for all EMA Good Pharmacovigilance Practice modules in Academy. Modules XI–XIV remain void per EMA and map to external guidance pages."
                </p>
                <div class="mt-4 flex flex-wrap gap-3">
                    <a
                        href=EMA_GVP_URL
                        target="_blank"
                        rel="noopener noreferrer"
                        class="rounded-lg bg-cyan-600 px-4 py-2 text-xs font-bold text-white hover:bg-cyan-500 transition-colors uppercase tracking-widest font-mono"
                    >
                        "Open EMA Source"
                    </a>
                    <a
                        href="/academy/courses"
                        class="rounded-lg border border-slate-700 px-4 py-2 text-xs font-bold text-slate-300 hover:text-white transition-colors uppercase tracking-widest font-mono"
                    >
                        "Back to Courses"
                    </a>
                    <a
                        href="/academy/gvp-curriculum"
                        class="rounded-lg border border-cyan-500/30 bg-cyan-500/10 px-4 py-2 text-xs font-bold text-cyan-300 hover:text-cyan-200 transition-colors uppercase tracking-widest font-mono"
                    >
                        "Open Generated Curriculum"
                    </a>
                    <a
                        href="/academy/gvp-progress"
                        class="rounded-lg border border-emerald-500/30 bg-emerald-500/10 px-4 py-2 text-xs font-bold text-emerald-300 hover:text-emerald-200 transition-colors uppercase tracking-widest font-mono"
                    >
                        "Track Progress"
                    </a>
                    <a
                        href="/academy/gvp-assessments"
                        class="rounded-lg border border-amber-500/30 bg-amber-500/10 px-4 py-2 text-xs font-bold text-amber-300 hover:text-amber-200 transition-colors uppercase tracking-widest font-mono"
                    >
                        "Assessment Blueprints"
                    </a>
                    <a
                        href="/academy/gvp-practicum"
                        class="rounded-lg border border-emerald-500/30 bg-emerald-500/10 px-4 py-2 text-xs font-bold text-emerald-300 hover:text-emerald-200 transition-colors uppercase tracking-widest font-mono"
                    >
                        "Practicum Launcher"
                    </a>
                    <a
                        href="/vigilance/guardian"
                        class="rounded-lg border border-emerald-500/30 bg-emerald-500/10 px-4 py-2 text-xs font-bold text-emerald-300 hover:text-emerald-200 transition-colors uppercase tracking-widest font-mono"
                    >
                        "Apply in Guardian"
                    </a>
                </div>
            </header>

            <div class="grid gap-4 md:grid-cols-2">
                {GVP_MODULES.into_iter().map(|m| {
                    let (drug, event, count) = guardian_seed_for_module(m.code);
                    let guardian_href = format!(
                        "/vigilance/guardian?module={}&drug={}&event={}&count={}",
                        m.code, drug, event, count
                    );
                    let badge_cls = if m.status == "Final" {
                        "text-emerald-400 bg-emerald-500/10 border-emerald-500/20"
                    } else {
                        "text-amber-400 bg-amber-500/10 border-amber-500/20"
                    };
                    view! {
                        <article class="rounded-xl border border-slate-800 bg-slate-900/40 p-5 hover:border-slate-700 transition-colors">
                            <div class="flex items-start justify-between gap-3">
                                <div>
                                    <p class="text-[10px] font-bold text-slate-500 uppercase tracking-widest font-mono">
                                        {"Module "}{m.code}
                                    </p>
                                    <h2 class="mt-1 text-base font-semibold text-white leading-snug">{m.title}</h2>
                                </div>
                                <span class=format!("rounded-full border px-2.5 py-1 text-[10px] font-bold uppercase tracking-widest font-mono {badge_cls}")>
                                    {m.status}
                                </span>
                            </div>
                            <p class="mt-3 text-sm text-slate-400 leading-relaxed">{m.note}</p>
                            <div class="mt-4">
                                <a
                                    href=format!("/academy/gvp-modules/{}", m.code)
                                    class="inline-flex items-center gap-2 text-xs font-bold text-cyan-400 hover:text-cyan-300 transition-colors uppercase tracking-widest font-mono"
                                >
                                    "Open Module"
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
                        </article>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}
