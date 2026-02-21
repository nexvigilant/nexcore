//! Academy evidence ledger across GVP/EPA/CPA launch contexts.

use super::gvp_data::{
    has_assessment_pass, load_guardian_writeback, load_gvp_assessment_passes,
    GuardianWritebackEvidence, GvpAssessmentPass,
};
use leptos::prelude::*;

fn layer_for(module_code: &str) -> &'static str {
    let code = module_code.to_ascii_uppercase();
    if code.starts_with("EPA") {
        "EPA"
    } else if code.starts_with("CPA") {
        "CPA"
    } else {
        "GVP"
    }
}

fn csv_header() -> &'static str {
    "layer,module_code,drug_name,event_name,case_count,risk_level,risk_score,assessment_passed,recorded_at"
}

fn csv_row(row: &GuardianWritebackEvidence, assessment_passed: bool) -> String {
    format!(
        "{},{},{},{},{},{},{:.4},{},{}",
        layer_for(&row.module_code),
        row.module_code,
        row.drug_name,
        row.event_name,
        row.case_count,
        row.risk_level,
        row.risk_score,
        assessment_passed,
        row.recorded_at
    )
}

#[component]
pub fn EvidenceLedgerPage() -> impl IntoView {
    let evidence_rows = RwSignal::new(load_guardian_writeback());
    let assessment_rows: RwSignal<Vec<GvpAssessmentPass>> =
        RwSignal::new(load_gvp_assessment_passes());
    let query = RwSignal::new(String::new());
    let layer = RwSignal::new(String::from("all"));
    let only_ready = RwSignal::new(false);

    let filtered = Signal::derive(move || {
        let q = query.get().to_ascii_lowercase();
        let active_layer = layer.get().to_ascii_lowercase();
        let ready_only = only_ready.get();
        evidence_rows
            .get()
            .into_iter()
            .filter(|row| {
                let row_layer = layer_for(&row.module_code).to_ascii_lowercase();
                if active_layer != "all" && row_layer != active_layer {
                    return false;
                }
                let passed = has_assessment_pass(&row.module_code, &assessment_rows.get());
                if ready_only && !passed {
                    return false;
                }
                q.is_empty()
                    || row.module_code.to_ascii_lowercase().contains(&q)
                    || row.drug_name.to_ascii_lowercase().contains(&q)
                    || row.event_name.to_ascii_lowercase().contains(&q)
                    || row.risk_level.to_ascii_lowercase().contains(&q)
                    || row.recorded_at.to_ascii_lowercase().contains(&q)
            })
            .collect::<Vec<_>>()
    });

    let csv_payload = Signal::derive(move || {
        let mut lines = vec![csv_header().to_string()];
        for row in filtered.get() {
            let passed = has_assessment_pass(&row.module_code, &assessment_rows.get());
            lines.push(csv_row(&row, passed));
        }
        lines.join("\n")
    });

    view! {
        <div class="mx-auto max-w-7xl px-4 py-12">
            <header class="mb-8">
                <p class="text-[11px] font-bold text-emerald-400 uppercase tracking-[0.2em] font-mono">"Execution Evidence"</p>
                <h1 class="mt-2 text-4xl font-bold text-white font-mono uppercase tracking-tight">"Academy Evidence Ledger"</h1>
                <p class="mt-3 text-slate-400 max-w-4xl">
                    "Unified audit surface for Guardian writeback evidence across GVP, EPA, and CPA contexts."
                </p>
                <div class="mt-4 flex flex-wrap gap-3">
                    <a href="/academy/gvp-progress" class="rounded-lg border border-cyan-500/30 bg-cyan-500/10 px-4 py-2 text-xs font-bold text-cyan-300 hover:text-cyan-200 uppercase tracking-widest font-mono">"GVP Progress"</a>
                    <a href="/academy/guardian-bridge" class="rounded-lg border border-emerald-500/30 bg-emerald-500/10 px-4 py-2 text-xs font-bold text-emerald-300 hover:text-emerald-200 uppercase tracking-widest font-mono">"Bridge Map"</a>
                    <a href="/vigilance/guardian" class="rounded-lg border border-emerald-500/30 bg-emerald-500/10 px-4 py-2 text-xs font-bold text-emerald-300 hover:text-emerald-200 uppercase tracking-widest font-mono">"Open Guardian"</a>
                </div>
            </header>

            <section class="mb-6 rounded-2xl border border-slate-800 bg-slate-900/40 p-5">
                <div class="grid gap-3 md:grid-cols-4">
                    <input
                        type="text"
                        placeholder="Search module, drug, event..."
                        prop:value=move || query.get()
                        on:input=move |ev| query.set(event_target_value(&ev))
                        class="md:col-span-2 rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-sm text-white placeholder:text-slate-500 focus:border-emerald-500 focus:outline-none"
                    />
                    <select
                        prop:value=move || layer.get()
                        on:change=move |ev| layer.set(event_target_value(&ev))
                        class="rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-sm text-white focus:border-emerald-500 focus:outline-none"
                    >
                        <option value="all">"All Layers"</option>
                        <option value="gvp">"GVP"</option>
                        <option value="epa">"EPA"</option>
                        <option value="cpa">"CPA"</option>
                    </select>
                    <label class="inline-flex items-center gap-2 rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-xs text-slate-300 uppercase tracking-widest font-mono">
                        <input
                            type="checkbox"
                            prop:checked=only_ready
                            on:change=move |ev| only_ready.set(event_target_checked(&ev))
                            class="h-4 w-4 rounded border-slate-700 bg-slate-900 text-emerald-500"
                        />
                        "Ready Only"
                    </label>
                </div>
            </section>

            <section class="rounded-2xl border border-slate-800 bg-slate-900/40 p-5">
                <div class="flex flex-wrap items-center justify-between gap-3">
                    <h2 class="text-sm font-bold text-white uppercase tracking-widest font-mono">"Evidence Rows"</h2>
                    <span class="text-xs text-slate-400 font-mono">{move || format!("{} row(s)", filtered.get().len())}</span>
                </div>
                <div class="mt-4 space-y-2 max-h-[420px] overflow-y-auto pr-1">
                    {move || {
                        let rows = filtered.get();
                        if rows.is_empty() {
                            view! {
                                <p class="text-sm text-slate-500">"No evidence rows match current filters."</p>
                            }.into_any()
                        } else {
                            rows.into_iter().map(|row| {
                                let layer_label = layer_for(&row.module_code).to_string();
                                let assessment_passed = has_assessment_pass(&row.module_code, &assessment_rows.get());
                                view! {
                                    <article class="rounded-lg border border-slate-800 bg-slate-950/40 px-3 py-2">
                                        <div class="flex flex-wrap items-center justify-between gap-2">
                                            <div class="min-w-0">
                                                <p class="text-[10px] text-slate-500 font-mono uppercase tracking-widest">
                                                    {format!("{} · {}", layer_label, row.module_code)}
                                                </p>
                                                <p class="text-sm text-white truncate">
                                                    {format!("{} / {} / {} cases", row.drug_name, row.event_name, row.case_count)}
                                                </p>
                                                <p class="text-[10px] text-slate-500 font-mono">{row.recorded_at}</p>
                                            </div>
                                            <div class="flex flex-wrap items-center gap-2">
                                                <span class="rounded-full border border-emerald-500/30 bg-emerald-500/10 px-2 py-0.5 text-[10px] font-bold text-emerald-300 uppercase tracking-widest font-mono">
                                                    {format!("{} {:.2}", row.risk_level, row.risk_score)}
                                                </span>
                                                {if assessment_passed {
                                                    view! {
                                                        <span class="rounded-full border border-cyan-500/30 bg-cyan-500/10 px-2 py-0.5 text-[10px] font-bold text-cyan-300 uppercase tracking-widest font-mono">
                                                            "Assessment Pass"
                                                        </span>
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <span class="rounded-full border border-amber-500/30 bg-amber-500/10 px-2 py-0.5 text-[10px] font-bold text-amber-300 uppercase tracking-widest font-mono">
                                                            "Assessment Pending"
                                                        </span>
                                                    }.into_any()
                                                }}
                                            </div>
                                        </div>
                                    </article>
                                }
                            }).collect_view().into_any()
                        }
                    }}
                </div>
            </section>

            <section class="mt-6 rounded-2xl border border-slate-800 bg-slate-900/40 p-5">
                <h2 class="text-sm font-bold text-white uppercase tracking-widest font-mono">"CSV Export (Filtered)"</h2>
                <p class="mt-1 text-xs text-slate-400">"Copy this payload into your compliance archive or BI workflow."</p>
                <textarea
                    readonly=true
                    prop:value=move || csv_payload.get()
                    class="mt-3 h-48 w-full rounded-lg border border-slate-700 bg-slate-950 p-3 text-[11px] text-slate-300 font-mono focus:outline-none"
                />
            </section>
        </div>
    }
}
