//! Signal decision framework — Bradford Hill criteria-based signal evaluation tool

use leptos::prelude::*;

#[component]
pub fn SignalDecisionPage() -> impl IntoView {
    let ratings: Vec<(RwSignal<u8>, &'static str, &'static str)> = vec![
        (RwSignal::new(0), "SD01", "Data Quality — completeness, accuracy, and reliability of source data"),
        (RwSignal::new(0), "SD02", "Statistical Strength — magnitude of disproportionality (PRR, ROR, IC, EBGM)"),
        (RwSignal::new(0), "SD03", "Biological Plausibility — known pharmacological mechanism supports the association"),
        (RwSignal::new(0), "SD04", "Temporal Relationship — timing between exposure and event is consistent"),
        (RwSignal::new(0), "SD05", "Dose-Response — evidence of dose-dependent effect relationship"),
        (RwSignal::new(0), "SD06", "Consistency — signal replicated across multiple data sources or populations"),
        (RwSignal::new(0), "SD07", "Specificity — association is specific to the drug-event pair"),
        (RwSignal::new(0), "SD08", "Public Health Impact — severity and frequency of the adverse event"),
    ];

    let rated_count = {
        let r: Vec<RwSignal<u8>> = ratings.iter().map(|(s, _, _)| *s).collect();
        Signal::derive(move || r.iter().filter(|s| s.get() > 0).count())
    };

    let avg_rating = {
        let r: Vec<RwSignal<u8>> = ratings.iter().map(|(s, _, _)| *s).collect();
        Signal::derive(move || {
            let vals: Vec<u8> = r.iter().map(|s| s.get()).filter(|v| *v > 0).collect();
            if vals.is_empty() { 0.0 } else { vals.iter().map(|v| *v as f64).sum::<f64>() / vals.len() as f64 }
        })
    };

    let decision = Signal::derive(move || {
        let avg = avg_rating.get();
        if avg < 1.0 { "Not Rated" }
        else if avg < 2.5 { "Close Signal" }
        else if avg < 3.5 { "Continue Monitoring" }
        else { "Validate Signal" }
    });

    let decision_color = Signal::derive(move || {
        let avg = avg_rating.get();
        if avg < 1.0 { "text-slate-500" }
        else if avg < 2.5 { "text-emerald-400" }
        else if avg < 3.5 { "text-amber-400" }
        else { "text-red-400" }
    });

    let decision_icon = Signal::derive(move || {
        let avg = avg_rating.get();
        if avg < 1.0 { "---" }
        else if avg < 2.5 { "CLOSE" }
        else if avg < 3.5 { "MONITOR" }
        else { "VALIDATE" }
    });

    let saved = RwSignal::new(false);
    let on_save = move |_| { saved.set(true); };

    view! {
        <div class="mx-auto max-w-4xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Signal Decision Framework"</h1>
            <p class="mt-2 text-slate-400">"Bradford Hill criteria-based signal evaluation. Rate each criterion (1-5) to determine action."</p>

            <div class="mt-6 flex gap-6 text-sm">
                <div class="rounded-lg border border-slate-800 bg-slate-900/50 px-4 py-3">
                    <p class="text-slate-500">"Criteria Rated"</p>
                    <p class="text-xl font-bold text-white font-mono">{move || format!("{}/8", rated_count.get())}</p>
                </div>
                <div class="rounded-lg border border-slate-800 bg-slate-900/50 px-4 py-3">
                    <p class="text-slate-500">"Strength"</p>
                    <p class="text-xl font-bold text-cyan-400 font-mono">{move || format!("{:.1}", avg_rating.get())}</p>
                </div>
                <div class="rounded-lg border border-slate-800 bg-slate-900/50 px-4 py-3">
                    <p class="text-slate-500">"Decision"</p>
                    <p class=move || format!("text-xl font-bold font-mono {}", decision_color.get())>{move || decision_icon.get()}</p>
                </div>
            </div>

            /* Decision explanation bar */
            <div class="mt-4 rounded-lg border border-slate-800 bg-slate-900/30 px-4 py-3">
                <p class="text-sm text-slate-400">
                    <span class="font-mono text-cyan-400">"Recommendation: "</span>
                    <span class=move || decision_color.get()>{move || decision.get()}</span>
                    {move || {
                        let avg = avg_rating.get();
                        if avg < 1.0 { " — Rate all criteria to receive a recommendation." }
                        else if avg < 2.5 { " — Evidence is weak. Document rationale and close." }
                        else if avg < 3.5 { " — Moderate evidence. Continue monitoring with defined review date." }
                        else { " — Strong evidence. Proceed to full signal validation and risk assessment." }
                    }}
                </p>
            </div>

            <div class="mt-8 space-y-3">
                {ratings.into_iter().map(|(signal, code, name)| {
                    view! { <RatingRow code=code name=name rating=signal /> }
                }).collect_view()}
            </div>

            <div class="mt-8 flex items-center gap-4">
                <button
                    on:click=on_save
                    class="rounded-lg bg-cyan-500 px-8 py-3 font-medium text-white hover:bg-cyan-400 transition-colors"
                >
                    "Save Assessment"
                </button>
                {move || saved.get().then(|| {
                    view! { <span class="text-sm text-emerald-400 font-mono">"Saved!"</span> }
                })}
            </div>
        </div>
    }
}

#[component]
fn RatingRow(code: &'static str, name: &'static str, rating: RwSignal<u8>) -> impl IntoView {
    view! {
        <div class="flex items-center gap-4 rounded-lg border border-slate-800 bg-slate-900/50 p-4">
            <span class="shrink-0 w-10 text-xs font-mono text-cyan-400">{code}</span>
            <span class="flex-1 text-sm text-white">{name}</span>
            <div class="flex gap-1">
                {(1u8..=5).map(|level| {
                    view! {
                        <button
                            on:click=move |_| rating.set(level)
                            class=move || {
                                if rating.get() >= level {
                                    "h-8 w-8 rounded border border-cyan-500 bg-cyan-500/20 text-xs text-cyan-400 font-bold transition-colors"
                                } else {
                                    "h-8 w-8 rounded border border-slate-700 text-xs text-slate-500 hover:border-cyan-500 hover:text-cyan-400 transition-colors"
                                }
                            }
                        >
                            {level.to_string()}
                        </button>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}
