//! Pathway detail page — individual learning pathway with EPA progression

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

#[component]
pub fn PathwayDetailPage() -> impl IntoView {
    let params = use_params_map();
    let epa_id = move || params.read().get("epaId").unwrap_or_default();

    view! {
        <div class="space-y-6">
            <div>
                <a href="/academy/pathways" class="text-cyan-400 hover:text-cyan-300 text-sm">"Back to Pathways"</a>
                <h1 class="text-2xl font-bold text-white mt-2">"Signal Detection & Management"</h1>
                <p class="text-slate-400 mt-1">{move || format!("Pathway EPA: {}", epa_id())}</p>
            </div>

            <div class="grid gap-4 sm:grid-cols-4">
                <Stat label="EPAs" value="4"/>
                <Stat label="Milestones" value="12"/>
                <Stat label="Est. Duration" value="16 weeks"/>
                <Stat label="Completed" value="0%"/>
            </div>

            <div>
                <h2 class="text-lg font-semibold text-white mb-4">"Pathway Milestones"</h2>
                <div class="space-y-3">
                    <Milestone
                        number=1
                        title="Understand signal detection theory"
                        desc="Learn the fundamentals of disproportionality analysis, PRR, ROR, IC, and EBGM"
                        status="Not Started"
                    />
                    <Milestone
                        number=2
                        title="Apply signal detection to FAERS data"
                        desc="Run signal detection algorithms on real FDA adverse event data and interpret results"
                        status="Not Started"
                    />
                    <Milestone
                        number=3
                        title="Signal validation and prioritization"
                        desc="Evaluate detected signals for clinical significance and prioritize for action"
                        status="Not Started"
                    />
                    <Milestone
                        number=4
                        title="Signal management workflow"
                        desc="Document and track signals through the complete lifecycle from detection to closure"
                        status="Not Started"
                    />
                    <Milestone
                        number=5
                        title="PSUR signal section authoring"
                        desc="Write the signal evaluation section of a Periodic Safety Update Report"
                        status="Not Started"
                    />
                    <Milestone
                        number=6
                        title="Benefit-risk impact assessment"
                        desc="Assess how detected signals affect the product's benefit-risk profile"
                        status="Not Started"
                    />
                </div>
            </div>

            <div>
                <h2 class="text-lg font-semibold text-white mb-4">"Recommended Resources"</h2>
                <div class="grid gap-4 lg:grid-cols-2">
                    <Resource title="ICH E2E: Pharmacovigilance Planning" rtype="Guideline"/>
                    <Resource title="CIOMS VIII: Signal Detection" rtype="Working Group Report"/>
                    <Resource title="EMA GVP Module IX: Signal Management" rtype="Guidance"/>
                    <Resource title="Signal Detection in PV (Bate, 2009)" rtype="Literature"/>
                </div>
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
fn Milestone(
    number: u32,
    #[prop(into)] title: String,
    #[prop(into)] desc: String,
    #[prop(into)] status: String,
) -> impl IntoView {
    view! {
        <div class="bg-slate-800/50 border border-slate-700/50 rounded-xl p-5 flex items-start gap-4">
            <div class="w-10 h-10 rounded-full bg-slate-700/50 flex items-center justify-center text-slate-400 font-bold shrink-0">
                {number.to_string()}
            </div>
            <div class="flex-1">
                <div class="flex items-center justify-between">
                    <h3 class="text-white font-semibold">{title}</h3>
                    <span class="text-xs text-slate-500 font-mono">{status}</span>
                </div>
                <p class="text-slate-400 text-sm mt-1">{desc}</p>
            </div>
        </div>
    }
}

#[component]
fn Resource(#[prop(into)] title: String, #[prop(into)] rtype: String) -> impl IntoView {
    view! {
        <div class="bg-slate-800/30 border border-slate-700/30 rounded-lg p-4 hover:border-slate-600/50 transition-colors cursor-pointer">
            <h4 class="text-sm text-white font-medium">{title}</h4>
            <span class="text-xs text-slate-500">{rtype}</span>
        </div>
    }
}
