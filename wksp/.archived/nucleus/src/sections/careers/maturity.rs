//! Career maturity model — assess career development stage

use leptos::prelude::*;

#[component]
pub fn MaturityPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-4xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white">"Career Maturity Model"</h1>
            <p class="mt-2 text-slate-400">"Understand where you are in your PV career journey and what comes next."</p>

            <div class="mt-8 space-y-4">
                <MaturityLevel level=1 name="Foundation" desc="Building core PV knowledge, understanding regulatory requirements, learning ICSR processing." active=true/>
                <MaturityLevel level=2 name="Practitioner" desc="Independent signal analysis, case processing proficiency, cross-functional collaboration." active=false/>
                <MaturityLevel level=3 name="Specialist" desc="Deep domain expertise, mentoring others, contributing to process improvement." active=false/>
                <MaturityLevel level=4 name="Leader" desc="Strategic oversight, team management, regulatory authority engagement." active=false/>
                <MaturityLevel level=5 name="Expert" desc="Industry thought leadership, innovation, shaping PV practice standards." active=false/>
            </div>
        </div>
    }
}

#[component]
fn MaturityLevel(level: u32, name: &'static str, desc: &'static str, active: bool) -> impl IntoView {
    let border = if active { "border-cyan-500/50" } else { "border-slate-800" };
    let indicator = if active { "bg-cyan-500" } else { "bg-slate-700" };

    view! {
        <div class=format!("flex gap-4 rounded-xl border {border} bg-slate-900/50 p-5")>
            <div class="flex flex-col items-center">
                <div class=format!("flex h-8 w-8 items-center justify-center rounded-full {indicator} text-sm font-bold text-white")>
                    {level}
                </div>
            </div>
            <div>
                <h3 class="font-semibold text-white">{name}</h3>
                <p class="mt-1 text-sm text-slate-400">{desc}</p>
            </div>
        </div>
    }
}
