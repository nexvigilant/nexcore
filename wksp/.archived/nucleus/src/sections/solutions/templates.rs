//! Solution templates — reusable PV workflow templates

use leptos::prelude::*;

#[component]
pub fn TemplatesPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white">"Solution Templates"</h1>
            <p class="mt-2 text-slate-400">"Ready-to-use templates for common pharmacovigilance workflows."</p>

            <div class="mt-8 grid gap-4 md:grid-cols-2">
                <TemplateCard
                    title="Signal Detection Workflow"
                    category="Detection"
                    desc="Complete PRR/ROR/IC analysis pipeline with configurable thresholds."
                />
                <TemplateCard
                    title="ICSR Processing SOP"
                    category="Compliance"
                    desc="Standard operating procedure for individual case safety report processing."
                />
                <TemplateCard
                    title="PBRER Template"
                    category="Reporting"
                    desc="Periodic benefit-risk evaluation report structure per ICH E2C(R2)."
                />
                <TemplateCard
                    title="Risk Management Plan"
                    category="Risk"
                    desc="EU RMP template with safety specification and pharmacovigilance plan."
                />
            </div>
        </div>
    }
}

#[component]
fn TemplateCard(title: &'static str, category: &'static str, desc: &'static str) -> impl IntoView {
    let cat_color = match category {
        "Detection" => "text-cyan-400 bg-cyan-500/10",
        "Compliance" => "text-amber-400 bg-amber-500/10",
        "Reporting" => "text-violet-400 bg-violet-500/10",
        "Risk" => "text-rose-400 bg-rose-500/10",
        _ => "text-slate-400 bg-slate-800",
    };

    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5 hover:border-slate-700 transition-colors cursor-pointer">
            <span class=format!("rounded-full px-2.5 py-0.5 text-xs font-medium {cat_color}")>{category}</span>
            <h3 class="mt-2 font-semibold text-white">{title}</h3>
            <p class="mt-1 text-sm text-slate-400">{desc}</p>
        </div>
    }
}
