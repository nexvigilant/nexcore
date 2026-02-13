//! Onboarding interests — select PV domains and topics of interest

use leptos::prelude::*;

#[component]
pub fn InterestsPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-xl px-4 py-12">
            <div class="h-1 w-full rounded-full bg-slate-800">
                <div class="h-1 rounded-full bg-cyan-500 transition-all" style="width: 66%"/>
            </div>

            <h1 class="mt-8 text-3xl font-bold text-white">"Your Interests"</h1>
            <p class="mt-2 text-slate-400">"Select the PV domains you're most interested in."</p>

            <div class="mt-8 grid grid-cols-2 gap-3">
                <DomainToggle code="D01" name="Safety Data Collection"/>
                <DomainToggle code="D02" name="Data Management"/>
                <DomainToggle code="D03" name="Signal Detection"/>
                <DomainToggle code="D04" name="Risk Management"/>
                <DomainToggle code="D05" name="Regulatory Compliance"/>
                <DomainToggle code="D06" name="Periodic Reporting"/>
                <DomainToggle code="D07" name="Aggregate Analysis"/>
                <DomainToggle code="D08" name="Signal Analysis"/>
                <DomainToggle code="D09" name="Risk Communication"/>
                <DomainToggle code="D10" name="Benefit-Risk"/>
                <DomainToggle code="D11" name="Audit & Inspection"/>
                <DomainToggle code="D12" name="Process Improvement"/>
                <DomainToggle code="D13" name="Stakeholder Mgmt"/>
                <DomainToggle code="D14" name="Technology & Innovation"/>
                <DomainToggle code="D15" name="Leadership"/>
            </div>

            <div class="mt-8 flex justify-between">
                <a href="/onboarding/profile" class="text-sm text-slate-400 hover:text-white">"Back"</a>
                <a href="/academy" class="rounded-lg bg-cyan-500 px-6 py-2.5 font-medium text-white hover:bg-cyan-400 transition-colors">
                    "Get Started"
                </a>
            </div>
        </div>
    }
}

#[component]
fn DomainToggle(code: &'static str, name: &'static str) -> impl IntoView {
    let selected = RwSignal::new(false);
    view! {
        <button
            class=move || if selected.get() {
                "rounded-lg border border-cyan-500 bg-cyan-500/10 p-3 text-left transition-colors"
            } else {
                "rounded-lg border border-slate-800 bg-slate-900/50 p-3 text-left hover:border-slate-700 transition-colors"
            }
            on:click=move |_| selected.set(!selected.get())
        >
            <div class="text-xs font-mono text-slate-500">{code}</div>
            <div class="text-sm font-medium text-white">{name}</div>
        </button>
    }
}
