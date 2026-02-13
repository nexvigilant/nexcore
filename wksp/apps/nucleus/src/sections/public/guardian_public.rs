//! Guardian public page — overview of the Guardian homeostasis system

use leptos::prelude::*;

#[component]
pub fn GuardianPublicPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-4xl px-4 py-16">
            <h1 class="text-4xl font-bold text-white">"Guardian"</h1>
            <p class="mt-3 text-lg text-slate-400">
                "Autonomous safety monitoring inspired by the human immune system."
            </p>

            <div class="mt-10 grid gap-6 md:grid-cols-2">
                <PhaseCard
                    phase="SENSE"
                    desc="Continuous monitoring of safety signals across multiple data sources. PAMPs detect known threat patterns; DAMPs detect novel anomalies."
                    color="emerald"
                />
                <PhaseCard
                    phase="COMPARE"
                    desc="Disproportionality analysis using PRR, ROR, IC, EBGM, and Chi-squared metrics against configurable thresholds."
                    color="amber"
                />
                <PhaseCard
                    phase="ACT"
                    desc="Automated escalation, alerting, and reporting when signals breach safety boundaries. 24 governance Acts enforce compliance."
                    color="rose"
                />
                <PhaseCard
                    phase="LEARN"
                    desc="Feedback loops refine detection sensitivity over time. The system improves with every signal it processes."
                    color="violet"
                />
            </div>

            <div class="mt-12 text-center">
                <a href="/signup" class="rounded-lg bg-cyan-500 px-8 py-3 font-medium text-white hover:bg-cyan-400 transition-colors">
                    "Get Access"
                </a>
            </div>
        </div>
    }
}

#[component]
fn PhaseCard(phase: &'static str, desc: &'static str, color: &'static str) -> impl IntoView {
    let card_class = format!("rounded-xl border border-slate-800 bg-slate-900/50 p-6 hover:border-{color}-500/30 transition-colors");
    let title_class = format!("text-lg font-bold text-{color}-400");

    view! {
        <div class=card_class>
            <h3 class=title_class>{phase}</h3>
            <p class="mt-2 text-sm text-slate-400">{desc}</p>
        </div>
    }
}
