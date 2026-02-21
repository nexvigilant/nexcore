//! Performance conditions analysis — Mager & Pipe performance analysis framework for PV teams

use leptos::prelude::*;

#[component]
pub fn PerformanceConditionsPage() -> impl IntoView {
    let ratings: Vec<(RwSignal<u8>, &'static str, &'static str)> = vec![
        (RwSignal::new(0), "PC01", "Knowledge — does the person know what is expected and how to do it?"),
        (RwSignal::new(0), "PC02", "Skills — does the person have practiced ability to perform the task?"),
        (RwSignal::new(0), "PC03", "Motivation — is there sufficient incentive and desire to perform?"),
        (RwSignal::new(0), "PC04", "Environment — does the work environment support the desired performance?"),
        (RwSignal::new(0), "PC05", "Tools and Resources — are proper tools, systems, and references available?"),
        (RwSignal::new(0), "PC06", "Feedback — does the person receive timely, specific performance feedback?"),
        (RwSignal::new(0), "PC07", "Consequences — are there meaningful consequences for performance (or lack)?"),
        (RwSignal::new(0), "PC08", "Capacity — does the person have the physical/cognitive capacity for the role?"),
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

    let category = Signal::derive(move || {
        let avg = avg_rating.get();
        if avg < 1.0 { "Not Rated" }
        else if avg < 2.0 { "Performance Gap" }
        else if avg < 3.0 { "Conditions Need Work" }
        else if avg < 4.0 { "Adequate Conditions" }
        else if avg < 4.5 { "Strong Conditions" }
        else { "Optimal Conditions" }
    });

    let category_color = Signal::derive(move || {
        let avg = avg_rating.get();
        if avg < 1.0 { "text-slate-500" }
        else if avg < 2.0 { "text-red-400" }
        else if avg < 3.0 { "text-amber-400" }
        else if avg < 4.0 { "text-yellow-400" }
        else if avg < 4.5 { "text-emerald-400" }
        else { "text-cyan-400" }
    });

    let saved = RwSignal::new(false);
    let on_save = move |_| { saved.set(true); };

    view! {
        <div class="mx-auto max-w-4xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Performance Conditions Analysis"</h1>
            <p class="mt-2 text-slate-400">"Mager & Pipe framework: diagnose why PV performance gaps exist (1-5)."</p>

            <div class="mt-6 flex gap-6 text-sm">
                <div class="rounded-lg border border-slate-800 bg-slate-900/50 px-4 py-3">
                    <p class="text-slate-500">"Conditions Rated"</p>
                    <p class="text-xl font-bold text-white font-mono">{move || format!("{}/8", rated_count.get())}</p>
                </div>
                <div class="rounded-lg border border-slate-800 bg-slate-900/50 px-4 py-3">
                    <p class="text-slate-500">"Average"</p>
                    <p class="text-xl font-bold text-cyan-400 font-mono">{move || format!("{:.1}", avg_rating.get())}</p>
                </div>
                <div class="rounded-lg border border-slate-800 bg-slate-900/50 px-4 py-3">
                    <p class="text-slate-500">"Diagnosis"</p>
                    <p class=move || format!("text-xl font-bold font-mono {}", category_color.get())>{move || category.get()}</p>
                </div>
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
