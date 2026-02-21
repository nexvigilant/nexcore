//! Admin: Solutions management — consulting services, templates, programs

use leptos::prelude::*;

#[component]
pub fn SolutionsAdminPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8 space-y-8">
            <div>
                <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Solutions Admin"</h1>
                <p class="mt-1 text-slate-400">"Manage consulting services, workflow templates, and safety programs"</p>
            </div>

            <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                <Stat label="Services" value="6" sub="consulting offerings"/>
                <Stat label="Templates" value="9" sub="workflow templates"/>
                <Stat label="Programs" value="3" sub="safety programs"/>
            </div>

            <div>
                <h2 class="text-sm font-bold uppercase tracking-widest text-slate-500 mb-4">"Consulting Services"</h2>
                <div class="space-y-2">
                    <ItemRow name="PV System Setup" category="Setup" status="Active"/>
                    <ItemRow name="Signal Detection Audit" category="Audit" status="Active"/>
                    <ItemRow name="QPPV Advisory" category="Advisory" status="Active"/>
                    <ItemRow name="Regulatory Submission Support" category="Regulatory" status="Active"/>
                    <ItemRow name="GVP Compliance Review" category="Compliance" status="Active"/>
                    <ItemRow name="Safety Database Migration" category="Technical" status="Active"/>
                </div>
            </div>

            <div>
                <h2 class="text-sm font-bold uppercase tracking-widest text-slate-500 mb-4">"Workflow Templates"</h2>
                <div class="space-y-2">
                    <ItemRow name="ICSR Processing Workflow" category="Case Processing" status="Published"/>
                    <ItemRow name="Signal Evaluation Protocol" category="Signal Management" status="Published"/>
                    <ItemRow name="PSUR/PBRER Preparation" category="Aggregate Reports" status="Published"/>
                    <ItemRow name="DSUR Template" category="Clinical Trials" status="Published"/>
                    <ItemRow name="Risk Management Plan" category="Risk Management" status="Published"/>
                    <ItemRow name="Benefit-Risk Assessment" category="Evaluation" status="Published"/>
                    <ItemRow name="PADER Template" category="Post-Authorization" status="Published"/>
                    <ItemRow name="Safety Variation" category="Regulatory" status="Published"/>
                    <ItemRow name="Literature Monitoring" category="Surveillance" status="Published"/>
                </div>
            </div>

            <div>
                <h2 class="text-sm font-bold uppercase tracking-widest text-slate-500 mb-4">"Safety Programs"</h2>
                <div class="space-y-2">
                    <ItemRow name="PV System Certification" category="12-week" status="Active"/>
                    <ItemRow name="Signal Management Excellence" category="8-week" status="Active"/>
                    <ItemRow name="Regulatory Intelligence Program" category="Ongoing" status="Active"/>
                </div>
            </div>
        </div>
    }
}

#[component]
fn Stat(label: &'static str, value: &'static str, sub: &'static str) -> impl IntoView {
    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5">
            <p class="text-[10px] font-bold uppercase tracking-widest text-slate-500">{label}</p>
            <p class="mt-2 text-3xl font-bold font-mono text-cyan-400">{value}</p>
            <p class="mt-1 text-xs text-slate-500">{sub}</p>
        </div>
    }
}

#[component]
fn ItemRow(name: &'static str, category: &'static str, status: &'static str) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between rounded-lg border border-slate-800/50 bg-slate-900/30 p-4">
            <div class="flex items-center gap-3">
                <span class="text-sm text-white font-medium">{name}</span>
                <span class="rounded bg-slate-800 px-2 py-0.5 text-[10px] text-slate-500 font-mono uppercase">{category}</span>
            </div>
            <span class="rounded bg-emerald-500/10 px-2 py-0.5 text-[10px] text-emerald-400 font-mono uppercase">{status}</span>
        </div>
    }
}
