//! CPA track catalog from embedded PV KSB workbook.

use super::pv_ksb_framework::all_cpas;
use leptos::prelude::*;

#[component]
pub fn CpaTracksPage() -> impl IntoView {
    let cpas = StoredValue::new(all_cpas());
    let query = RwSignal::new(String::new());

    let filtered = Signal::derive(move || {
        let q = query.get().to_ascii_lowercase();
        cpas.get_value()
            .into_iter()
            .filter(|c| {
                q.is_empty()
                    || c.cpa_id.to_ascii_lowercase().contains(&q)
                    || c.cpa_name.to_ascii_lowercase().contains(&q)
                    || c.focus_area.to_ascii_lowercase().contains(&q)
                    || c.executive_summary.to_ascii_lowercase().contains(&q)
            })
            .collect::<Vec<_>>()
    });

    view! {
        <div class="mx-auto max-w-7xl px-4 py-12">
            <header class="mb-8">
                <p class="text-[11px] font-bold text-amber-400 uppercase tracking-[0.2em] font-mono">"Capability Progression"</p>
                <h1 class="mt-2 text-4xl font-bold text-white font-mono uppercase tracking-tight">"CPA Catalog"</h1>
                <p class="mt-3 text-slate-400 max-w-4xl">
                    "Capability Progressive Activities integrated from the embedded PV KSB workbook."
                </p>
                <div class="mt-4 flex flex-wrap gap-3">
                    <a href="/academy/epa-tracks" class="rounded-lg border border-cyan-500/30 bg-cyan-500/10 px-4 py-2 text-xs font-bold text-cyan-300 hover:text-cyan-200 transition-colors uppercase tracking-widest font-mono">"EPA Tracks"</a>
                    <a href="/academy/pv-framework" class="rounded-lg border border-slate-700 px-4 py-2 text-xs font-bold text-slate-300 hover:text-white transition-colors uppercase tracking-widest font-mono">"Framework"</a>
                    <a href="/academy/guardian-bridge" class="rounded-lg border border-emerald-500/30 bg-emerald-500/10 px-4 py-2 text-xs font-bold text-emerald-300 hover:text-emerald-200 transition-colors uppercase tracking-widest font-mono">"Bridge Map"</a>
                    <a href="/vigilance/guardian" class="rounded-lg border border-emerald-500/30 bg-emerald-500/10 px-4 py-2 text-xs font-bold text-emerald-300 hover:text-emerald-200 transition-colors uppercase tracking-widest font-mono">"Apply in Guardian"</a>
                </div>
                <div class="mt-4 rounded-xl border border-emerald-500/20 bg-emerald-500/5 p-4">
                    <p class="text-[10px] font-bold text-emerald-300 uppercase tracking-widest font-mono">"Connection Path"</p>
                    <p class="mt-1 text-xs text-slate-300">"Each CPA defines how capability grows; Guardian is where that capability is validated in operational vigilance decisions."</p>
                </div>
            </header>

            <input
                type="text"
                placeholder="Search CPA ID, focus area, or summary..."
                prop:value=move || query.get()
                on:input=move |ev| query.set(event_target_value(&ev))
                class="mb-6 w-full rounded-lg border border-slate-700 bg-slate-950 px-4 py-2.5 text-sm text-white placeholder-slate-500 focus:border-amber-500 focus:outline-none"
            />

            <div class="grid gap-4 md:grid-cols-2">
                {move || filtered.get().into_iter().map(|cpa| view! {
                    <article class="rounded-xl border border-slate-800 bg-slate-900/40 p-5 hover:border-slate-700 transition-colors">
                        <div class="flex items-center justify-between gap-3">
                            <span class="rounded-full border border-amber-500/20 bg-amber-500/10 px-2.5 py-1 text-[10px] font-bold text-amber-300 font-mono uppercase tracking-widest">
                                {cpa.cpa_id.clone()}
                            </span>
                            <span class="text-[10px] text-slate-500 font-mono uppercase tracking-widest">
                                {cpa.career_stage.clone()}
                            </span>
                        </div>
                        <h2 class="mt-3 text-lg font-semibold text-white">{cpa.cpa_name.clone()}</h2>
                        <p class="mt-1 text-xs text-slate-400">{cpa.focus_area.clone()}</p>
                        <p class="mt-3 text-sm text-slate-300 line-clamp-4">{cpa.executive_summary.clone()}</p>
                        <div class="mt-4 flex items-center justify-between">
                            <a href=format!("/academy/cpa/{}", cpa.cpa_id) class="text-xs font-bold text-amber-400 hover:text-amber-300 transition-colors uppercase tracking-widest font-mono">
                                "Open CPA"
                            </a>
                            <a href=format!("/vigilance/guardian?module={}&event={}&count=2", cpa.cpa_id, cpa.focus_area.to_ascii_lowercase().replace(' ', "-")) class="text-xs font-bold text-emerald-400 hover:text-emerald-300 transition-colors uppercase tracking-widest font-mono">
                                "Launch Guardian"
                            </a>
                        </div>
                    </article>
                }).collect_view()}
            </div>
        </div>
    }
}
