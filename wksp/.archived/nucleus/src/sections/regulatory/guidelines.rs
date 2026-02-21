//! Regulatory guidelines reference

use leptos::prelude::*;

#[component]
pub fn GuidelinesPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-4xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white">"Guidelines Library"</h1>
            <p class="mt-2 text-slate-400">"Quick reference to key PV guidelines and regulations."</p>

            <div class="mt-8 space-y-3">
                <GuidelineRow code="ICH E2A" title="Clinical Safety Data Management" scope="ICSR definitions and expedited reporting"/>
                <GuidelineRow code="ICH E2B(R3)" title="Electronic Transmission of ICSRs" scope="E2B format and data elements"/>
                <GuidelineRow code="ICH E2C(R2)" title="Periodic Benefit-Risk Evaluation" scope="PBRER format and content"/>
                <GuidelineRow code="ICH E2D" title="Post-Approval Safety Data" scope="Expedited and periodic reporting requirements"/>
                <GuidelineRow code="ICH E2E" title="Pharmacovigilance Planning" scope="Risk-based PV planning"/>
                <GuidelineRow code="ICH E2F" title="DSUR" scope="Development Safety Update Report"/>
                <GuidelineRow code="GVP Module VI" title="Management of Adverse Reactions" scope="Collection, reporting, follow-up"/>
                <GuidelineRow code="GVP Module IX" title="Signal Management" scope="Signal detection, evaluation, and action"/>
            </div>
        </div>
    }
}

#[component]
fn GuidelineRow(code: &'static str, title: &'static str, scope: &'static str) -> impl IntoView {
    view! {
        <div class="flex items-start gap-4 rounded-lg border border-slate-800 bg-slate-900/50 p-4 hover:border-slate-700 transition-colors">
            <span class="shrink-0 rounded bg-slate-800 px-2 py-1 text-xs font-mono text-cyan-400">{code}</span>
            <div>
                <h3 class="font-medium text-white">{title}</h3>
                <p class="mt-0.5 text-xs text-slate-500">{scope}</p>
            </div>
        </div>
    }
}
