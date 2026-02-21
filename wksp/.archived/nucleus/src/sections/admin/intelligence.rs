//! Admin: Intelligence management — reports, data sources, research

use leptos::prelude::*;

#[component]
pub fn IntelligenceAdminPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold text-white">"Intelligence Admin"</h1>
                    <p class="mt-1 text-slate-400">"Pharma intelligence, data sources, and research"</p>
                </div>
                <a href="/admin" class="text-sm text-slate-400 hover:text-white transition-colors">"\u{2190} Dashboard"</a>
            </div>

            // Data sources
            <div class="mt-8 rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                <h2 class="text-lg font-semibold text-white">"Data Sources"</h2>
                <div class="mt-4 space-y-3">
                    {[("FDA FAERS", "Adverse event reports", "Active"),
                      ("WHO VigiBase", "Global pharmacovigilance", "Pending"),
                      ("EudraVigilance", "EU adverse reaction reports", "Pending"),
                      ("ClinicalTrials.gov", "Clinical trial registry", "Pending")].into_iter().map(|(name, desc, status)| {
                        let status_class = if status == "Active" {
                            "text-emerald-400 bg-emerald-500/10"
                        } else {
                            "text-slate-400 bg-slate-500/10"
                        };
                        view! {
                            <div class="flex items-center justify-between rounded-lg border border-slate-800 bg-slate-900/30 px-4 py-3">
                                <div>
                                    <p class="font-medium text-white">{name}</p>
                                    <p class="text-xs text-slate-500">{desc}</p>
                                </div>
                                <span class=format!("rounded-full px-2.5 py-0.5 text-xs font-medium {status_class}")>{status}</span>
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>

            // Reports
            <div class="mt-6 rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                <div class="flex items-center justify-between">
                    <h2 class="text-lg font-semibold text-white">"Intelligence Reports"</h2>
                    <button class="rounded-lg bg-amber-600 px-4 py-2 text-sm font-medium text-white hover:bg-amber-500">"+ New Report"</button>
                </div>
                <div class="mt-4 rounded-xl border border-slate-800 bg-slate-900/30 p-8 text-center">
                    <p class="text-slate-400">"No reports generated"</p>
                    <p class="mt-2 text-sm text-slate-500">"Create signal detection reports, trend analyses, and safety summaries."</p>
                </div>
            </div>
        </div>
    }
}
