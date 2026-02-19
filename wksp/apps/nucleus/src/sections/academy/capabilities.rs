//! Capabilities page — competency areas, EPAs, and CPAs mapped to PV domains

use leptos::prelude::*;

#[component]
pub fn CapabilitiesPage() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <div>
                <h1 class="text-2xl font-bold text-white">"Capability Framework"</h1>
                <p class="text-slate-400 mt-1">"Entrustable Professional Activities and competency areas for pharmacovigilance"</p>
            </div>

            <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
                <Stat label="Competency Areas" value="7"/>
                <Stat label="EPAs" value="24"/>
                <Stat label="CPAs" value="48"/>
                <Stat label="Your Progress" value="0%"/>
            </div>

            <div class="space-y-4">
                <CapabilityArea
                    name="Signal Detection & Management"
                    epas=4
                    cpas=8
                    desc="Identify, validate, and manage safety signals from multiple data sources"
                    color="border-cyan-500/30"
                />
                <CapabilityArea
                    name="Individual Case Safety Reports"
                    epas=3
                    cpas=6
                    desc="Process, assess, and report ICSRs according to regulatory requirements"
                    color="border-emerald-500/30"
                />
                <CapabilityArea
                    name="Aggregate Safety Reporting"
                    epas=4
                    cpas=7
                    desc="PSUR, PBRER, DSUR, and PADER preparation and submission"
                    color="border-violet-500/30"
                />
                <CapabilityArea
                    name="Risk Management"
                    epas=3
                    cpas=6
                    desc="Risk management plans, REMS, risk minimization measures"
                    color="border-amber-500/30"
                />
                <CapabilityArea
                    name="Benefit-Risk Assessment"
                    epas=3
                    cpas=7
                    desc="Structured benefit-risk evaluation frameworks and decision-making"
                    color="border-red-500/30"
                />
                <CapabilityArea
                    name="Regulatory Intelligence"
                    epas=4
                    cpas=8
                    desc="Monitor and interpret regulatory guidance, inspections, and compliance"
                    color="border-blue-500/30"
                />
                <CapabilityArea
                    name="PV System & Quality"
                    epas=3
                    cpas=6
                    desc="PV system master file, quality management, audits and inspections"
                    color="border-pink-500/30"
                />
            </div>
        </div>
    }
}

#[component]
fn Stat(#[prop(into)] label: String, #[prop(into)] value: String) -> impl IntoView {
    view! {
        <div class="bg-slate-800/50 border border-slate-700/50 rounded-lg p-4 text-center">
            <div class="text-2xl font-bold text-white font-mono">{value}</div>
            <div class="text-sm text-slate-400">{label}</div>
        </div>
    }
}

#[component]
fn CapabilityArea(
    #[prop(into)] name: String,
    epas: u32,
    cpas: u32,
    #[prop(into)] desc: String,
    #[prop(into)] color: String,
) -> impl IntoView {
    view! {
        <div class={format!("bg-slate-800/50 border {} rounded-xl p-6 hover:bg-slate-800/70 transition-colors cursor-pointer", color)}>
            <div class="flex items-center justify-between">
                <h3 class="text-lg font-semibold text-white">{name}</h3>
                <div class="flex items-center gap-4 text-sm">
                    <span class="text-cyan-400 font-mono">{format!("{} EPAs", epas)}</span>
                    <span class="text-slate-500 font-mono">{format!("{} CPAs", cpas)}</span>
                </div>
            </div>
            <p class="text-slate-400 text-sm mt-2">{desc}</p>
            <div class="mt-4 w-full bg-slate-700/50 rounded-full h-2">
                <div class="bg-slate-600 h-2 rounded-full" style="width: 0%"></div>
            </div>
        </div>
    }
}
