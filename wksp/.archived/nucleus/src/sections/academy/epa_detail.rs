//! EPA detail view with mapped domains.

use super::gvp_data::{latest_module_evidence, load_guardian_writeback, module_evidence_count};
use super::pv_ksb_framework::{epa_by_id, epa_domain_links};
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

#[component]
pub fn EpaDetailPage() -> impl IntoView {
    let params = use_params_map();
    let epa_id = move || params.get().get("id").unwrap_or_default();
    let evidence_rows = RwSignal::new(load_guardian_writeback());

    view! {
        <div class="mx-auto max-w-5xl px-4 py-12">
            <a href="/academy/epa-tracks" class="text-xs font-mono text-cyan-400 hover:text-cyan-300 uppercase tracking-widest">"< Back to EPA Catalog"</a>
            {move || {
                let id = epa_id();
                match epa_by_id(&id) {
                    Some(epa) => {
                        let domains = epa_domain_links(&epa.epa_id);
                        let evidence_count = module_evidence_count(&epa.epa_id, &evidence_rows.get());
                        let latest = latest_module_evidence(&epa.epa_id, &evidence_rows.get());
                        view! {
                            <>
                                <header class="mt-6 mb-8">
                                    <p class="text-[10px] font-bold text-cyan-400 uppercase tracking-widest font-mono">{epa.epa_id.clone()}</p>
                                    <h1 class="mt-2 text-3xl font-bold text-white">{epa.epa_name.clone()}</h1>
                                    <p class="mt-2 text-sm text-slate-400">{epa.focus_area.clone()}</p>
                                    <p class="mt-4 text-slate-300">{epa.description.clone()}</p>
                                    <div class="mt-4 rounded-xl border border-emerald-500/20 bg-emerald-500/5 p-4">
                                        <p class="text-[10px] font-bold text-emerald-300 uppercase tracking-widest font-mono">"How It Connects"</p>
                                        <p class="mt-1 text-xs text-slate-300">"Use this EPA as execution intent, then run scenario triage in Guardian to apply the activity in live context."</p>
                                        <p class="mt-2 text-[10px] text-emerald-300 font-mono uppercase tracking-widest">
                                            {format!("Guardian Evidence: {}", evidence_count)}
                                        </p>
                                        {latest.map(|entry| view! {
                                            <p class="mt-1 text-[11px] text-slate-300">
                                                {format!(
                                                    "Latest: {} / {} / {} cases — {} ({:.2})",
                                                    entry.drug_name,
                                                    entry.event_name,
                                                    entry.case_count,
                                                    entry.risk_level,
                                                    entry.risk_score
                                                )}
                                            </p>
                                        })}
                                    </div>
                                    <div class="mt-4 flex flex-wrap gap-3">
                                        <a href=format!("/vigilance/guardian?module={}&event={}&count=3", epa.epa_id, epa.focus_area.to_ascii_lowercase().replace(' ', "-")) class="rounded-lg border border-emerald-500/30 bg-emerald-500/10 px-4 py-2 text-xs font-bold text-emerald-300 hover:text-emerald-200 uppercase tracking-widest font-mono">
                                            "Apply in Guardian"
                                        </a>
                                        <a href="/academy/cpa-tracks" class="rounded-lg border border-amber-500/30 bg-amber-500/10 px-4 py-2 text-xs font-bold text-amber-300 hover:text-amber-200 uppercase tracking-widest font-mono">
                                            "Open CPA Tracks"
                                        </a>
                                    </div>
                                </header>

                                <section class="rounded-2xl border border-slate-800 bg-slate-900/40 p-5">
                                    <h2 class="text-sm font-bold text-white uppercase tracking-widest font-mono">"Mapped Domains"</h2>
                                    <div class="mt-4 grid gap-3 md:grid-cols-2">
                                        {domains.into_iter().map(|d| view! {
                                            <article class="rounded-lg border border-slate-800 bg-slate-950/40 p-3">
                                                <div class="flex items-center justify-between">
                                                    <span class="text-[10px] font-bold text-cyan-300 font-mono uppercase tracking-widest">{d.domain_id.clone()}</span>
                                                    <span class="text-[9px] text-slate-500 font-mono uppercase">{d.role.clone()}</span>
                                                </div>
                                                <p class="mt-1 text-sm text-white">{d.domain_name.clone()}</p>
                                                <p class="mt-1 text-[10px] text-slate-500 font-mono">{d.level.clone()}</p>
                                                <a href=format!("/vigilance/guardian?module={}&event={}&count=2", d.domain_id, d.domain_name.to_ascii_lowercase().replace(' ', "-")) class="mt-2 inline-flex text-[10px] font-bold text-emerald-300 hover:text-emerald-200 uppercase tracking-widest font-mono">
                                                    "Launch Domain in Guardian"
                                                </a>
                                            </article>
                                        }).collect_view()}
                                    </div>
                                </section>
                            </>
                        }.into_any()
                    }
                    None => view! {
                        <div class="mt-8 rounded-xl border border-red-500/20 bg-red-500/10 p-6">
                            <p class="text-sm text-red-300 font-mono">"Unknown EPA ID."</p>
                        </div>
                    }.into_any(),
                }
            }}
        </div>
    }
}
