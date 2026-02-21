//! Skills overview — professional competency mapping

use leptos::prelude::*;

#[component]
pub fn SkillsPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white">"Skills Overview"</h1>
            <p class="mt-2 text-slate-400">"Assess and track your professional capabilities"</p>

            <div class="mt-8 grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
                <SkillDomain domain="D01" name="PV Foundations" self_rating=0 target=3/>
                <SkillDomain domain="D02" name="AE Reporting" self_rating=0 target=3/>
                <SkillDomain domain="D03" name="Case Processing" self_rating=0 target=4/>
                <SkillDomain domain="D04" name="Literature" self_rating=0 target=2/>
                <SkillDomain domain="D05" name="Case Assessment" self_rating=0 target=4/>
                <SkillDomain domain="D06" name="Aggregate Reporting" self_rating=0 target=3/>
                <SkillDomain domain="D07" name="Risk Management" self_rating=0 target=3/>
                <SkillDomain domain="D08" name="Signal Detection" self_rating=0 target=5/>
                <SkillDomain domain="D09" name="Signal Evaluation" self_rating=0 target=4/>
                <SkillDomain domain="D10" name="Benefit-Risk" self_rating=0 target=4/>
            </div>

            <div class="mt-8 text-center">
                <button class="rounded-lg bg-amber-600 px-6 py-2.5 text-sm font-medium text-white hover:bg-amber-500 transition-colors">
                    "Take Competency Assessment"
                </button>
            </div>
        </div>
    }
}

#[component]
fn SkillDomain(
    domain: &'static str,
    name: &'static str,
    self_rating: u32,
    target: u32,
) -> impl IntoView {
    let pct = if target > 0 { (self_rating * 100) / target } else { 0 };
    let width = format!("width: {pct}%");

    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5">
            <div class="flex items-center gap-2">
                <span class="rounded bg-amber-500/10 px-2 py-0.5 text-xs font-medium text-amber-400">{domain}</span>
                <span class="text-sm font-medium text-white">{name}</span>
            </div>
            <div class="mt-3">
                <div class="flex justify-between text-xs text-slate-500">
                    <span>{format!("Level {self_rating}")}</span>
                    <span>{format!("Target: {target}")}</span>
                </div>
                <div class="mt-1 h-2 rounded-full bg-slate-800">
                    <div class="h-full rounded-full bg-amber-500 transition-all" style=width></div>
                </div>
            </div>
        </div>
    }
}
