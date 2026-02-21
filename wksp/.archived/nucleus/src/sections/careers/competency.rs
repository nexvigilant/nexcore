//! Competency self-assessment — rate yourself across 15 PV domains

use leptos::prelude::*;

#[component]
pub fn CompetencyPage() -> impl IntoView {
    let ratings: Vec<(RwSignal<u8>, &'static str, &'static str)> = vec![
        (RwSignal::new(0), "D01", "Safety Data Collection"),
        (RwSignal::new(0), "D02", "Data Management and Quality"),
        (RwSignal::new(0), "D03", "Signal Detection and Analysis"),
        (RwSignal::new(0), "D04", "Risk Management"),
        (RwSignal::new(0), "D05", "Regulatory Compliance"),
        (RwSignal::new(0), "D06", "Periodic Reporting"),
        (RwSignal::new(0), "D07", "Aggregate Data Analysis"),
        (RwSignal::new(0), "D08", "Signal Detection"),
        (RwSignal::new(0), "D09", "Risk Communication"),
        (RwSignal::new(0), "D10", "Benefit-Risk Assessment"),
        (RwSignal::new(0), "D11", "Audit and Inspection"),
        (RwSignal::new(0), "D12", "Process Improvement"),
        (RwSignal::new(0), "D13", "Stakeholder Management"),
        (RwSignal::new(0), "D14", "Technology and Innovation"),
        (RwSignal::new(0), "D15", "Leadership and Strategy"),
    ];

    let rated_count = {
        let ratings_clone: Vec<RwSignal<u8>> = ratings.iter().map(|(s, _, _)| *s).collect();
        Signal::derive(move || ratings_clone.iter().filter(|s| s.get() > 0).count())
    };

    let avg_rating = {
        let ratings_clone: Vec<RwSignal<u8>> = ratings.iter().map(|(s, _, _)| *s).collect();
        Signal::derive(move || {
            let vals: Vec<u8> = ratings_clone.iter().map(|s| s.get()).filter(|v| *v > 0).collect();
            if vals.is_empty() { 0.0 } else { vals.iter().map(|v| *v as f64).sum::<f64>() / vals.len() as f64 }
        })
    };

    let saved = RwSignal::new(false);
    let on_save = move |_| {
        saved.set(true);
    };

    view! {
        <div class="mx-auto max-w-4xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Competency Self-Assessment"</h1>
            <p class="mt-2 text-slate-400">"Rate your proficiency across all 15 PV domains (1-5)."</p>

            <div class="mt-6 flex gap-6 text-sm">
                <div class="rounded-lg border border-slate-800 bg-slate-900/50 px-4 py-3">
                    <p class="text-slate-500">"Domains Rated"</p>
                    <p class="text-xl font-bold text-white font-mono">{move || format!("{}/15", rated_count.get())}</p>
                </div>
                <div class="rounded-lg border border-slate-800 bg-slate-900/50 px-4 py-3">
                    <p class="text-slate-500">"Average"</p>
                    <p class="text-xl font-bold text-cyan-400 font-mono">{move || format!("{:.1}", avg_rating.get())}</p>
                </div>
            </div>

            <div class="mt-8 space-y-3">
                {ratings.into_iter().map(|(signal, code, name)| {
                    view! { <DomainRating code=code name=name rating=signal /> }
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
fn DomainRating(code: &'static str, name: &'static str, rating: RwSignal<u8>) -> impl IntoView {
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
