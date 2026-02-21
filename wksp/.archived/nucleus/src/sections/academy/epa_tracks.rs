//! EPA track catalog from embedded PV KSB workbook.

use super::pv_ksb_framework::all_epas;
use leptos::prelude::*;

#[component]
pub fn EpaTracksPage() -> impl IntoView {
    let epas = StoredValue::new(all_epas());
    let query = RwSignal::new(String::new());

    let filtered = Signal::derive(move || {
        let q = query.get().to_ascii_lowercase();
        epas.get_value()
            .into_iter()
            .filter(|e| {
                q.is_empty()
                    || e.epa_id.to_ascii_lowercase().contains(&q)
                    || e.epa_name.to_ascii_lowercase().contains(&q)
                    || e.focus_area.to_ascii_lowercase().contains(&q)
                    || e.description.to_ascii_lowercase().contains(&q)
            })
            .collect::<Vec<_>>()
    });

    view! {
        <div class="mx-auto max-w-7xl px-4 py-12">
            <header class="mb-8">
                <p class="text-[11px] font-bold text-cyan-400 uppercase tracking-[0.2em] font-mono">"Entrustable Tracks"</p>
                <h1 class="mt-2 text-4xl font-bold text-white font-mono uppercase tracking-tight">"EPA Catalog"</h1>
                <p class="mt-3 text-slate-400 max-w-4xl">
                    "Entrustable Professional Activities integrated from the 2025-12-08 PV KSB master workbook."
                </p>
                <div class="mt-4 flex flex-wrap gap-3">
                    <a href="/academy/pv-framework" class="rounded-lg border border-cyan-500/30 bg-cyan-500/10 px-4 py-2 text-xs font-bold text-cyan-300 hover:text-cyan-200 transition-colors uppercase tracking-widest font-mono">"Framework"</a>
                    <a href="/academy/cpa-tracks" class="rounded-lg border border-amber-500/30 bg-amber-500/10 px-4 py-2 text-xs font-bold text-amber-300 hover:text-amber-200 transition-colors uppercase tracking-widest font-mono">"CPA Tracks"</a>
                    <a href="/academy/guardian-bridge" class="rounded-lg border border-emerald-500/30 bg-emerald-500/10 px-4 py-2 text-xs font-bold text-emerald-300 hover:text-emerald-200 transition-colors uppercase tracking-widest font-mono">"Bridge Map"</a>
                    <a href="/vigilance/guardian" class="rounded-lg border border-emerald-500/30 bg-emerald-500/10 px-4 py-2 text-xs font-bold text-emerald-300 hover:text-emerald-200 transition-colors uppercase tracking-widest font-mono">"Apply in Guardian"</a>
                </div>
                <div class="mt-4 rounded-xl border border-emerald-500/20 bg-emerald-500/5 p-4">
                    <p class="text-[10px] font-bold text-emerald-300 uppercase tracking-widest font-mono">"Connection Path"</p>
                    <p class="mt-1 text-xs text-slate-300">"Each EPA defines what to perform; Guardian is where practitioners execute those workflows against live safety signals."</p>
                </div>
            </header>

            <input
                type="text"
                placeholder="Search EPA ID, focus area, or description..."
                prop:value=move || query.get()
                on:input=move |ev| query.set(event_target_value(&ev))
                class="mb-6 w-full rounded-lg border border-slate-700 bg-slate-950 px-4 py-2.5 text-sm text-white placeholder-slate-500 focus:border-cyan-500 focus:outline-none"
            />

            <div class="grid gap-4 md:grid-cols-2">
                {move || filtered.get().into_iter().map(|epa| view! {
                    <article class="rounded-xl border border-slate-800 bg-slate-900/40 p-5 hover:border-slate-700 transition-colors">
                        <div class="flex items-center justify-between gap-3">
                            <span class="rounded-full border border-cyan-500/20 bg-cyan-500/10 px-2.5 py-1 text-[10px] font-bold text-cyan-300 font-mono uppercase tracking-widest">
                                {epa.epa_id.clone()}
                            </span>
                            <span class="text-[10px] text-slate-500 font-mono uppercase tracking-widest">
                                {epa.tier.clone()}
                            </span>
                        </div>
                        <h2 class="mt-3 text-lg font-semibold text-white">{epa.epa_name.clone()}</h2>
                        <p class="mt-1 text-xs text-slate-400">{epa.focus_area.clone()}</p>
                        <p class="mt-3 text-sm text-slate-300 line-clamp-4">{epa.description.clone()}</p>
                        <div class="mt-4 flex items-center justify-between">
                            <a href=format!("/academy/epa/{}", epa.epa_id) class="text-xs font-bold text-cyan-400 hover:text-cyan-300 transition-colors uppercase tracking-widest font-mono">
                                "Open EPA"
                            </a>
                            <a href=format!("/vigilance/guardian?module={}&event={}&count=2", epa.epa_id, epa.focus_area.to_ascii_lowercase().replace(' ', "-")) class="text-xs font-bold text-emerald-400 hover:text-emerald-300 transition-colors uppercase tracking-widest font-mono">
                                "Launch Guardian"
                            </a>
                        </div>
                    </article>
                }).collect_view()}
            </div>
        </div>
    }
}
