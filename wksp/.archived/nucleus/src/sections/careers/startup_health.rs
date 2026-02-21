//! PV startup health check — comprehensive assessment of PV system readiness for startups and small MAHs

use leptos::prelude::*;

#[component]
pub fn StartupHealthPage() -> impl IntoView {
    let ratings: Vec<(RwSignal<u8>, &'static str, &'static str)> = vec![
        (RwSignal::new(0), "SH01", "Regulatory Compliance — marketing authorization PV obligations understood and met"),
        (RwSignal::new(0), "SH02", "SOPs and Procedures — documented standard operating procedures for all PV activities"),
        (RwSignal::new(0), "SH03", "Safety Database — validated system for ICSR collection, storage, and retrieval"),
        (RwSignal::new(0), "SH04", "QPPV Designated — qualified person for PV appointed with adequate authority"),
        (RwSignal::new(0), "SH05", "Signal Detection — defined process for routine signal detection and evaluation"),
        (RwSignal::new(0), "SH06", "ICSR Processing — end-to-end case intake, assessment, coding, and submission workflow"),
        (RwSignal::new(0), "SH07", "Periodic Reporting — PSUR/PBRER generation capability on regulatory timelines"),
        (RwSignal::new(0), "SH08", "Audit Trail — complete traceability of all PV decisions and data changes"),
        (RwSignal::new(0), "SH09", "Training Program — role-based PV training with documentation and refresh cycles"),
        (RwSignal::new(0), "SH10", "Vendor Management — oversight of outsourced PV activities with quality agreements"),
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
        else if avg < 2.0 { "Critical Gaps" }
        else if avg < 3.0 { "Major Gaps" }
        else if avg < 3.5 { "Developing" }
        else if avg < 4.0 { "Functional" }
        else if avg < 4.5 { "Mature" }
        else { "Inspection-Ready" }
    });

    let category_color = Signal::derive(move || {
        let avg = avg_rating.get();
        if avg < 1.0 { "text-slate-500" }
        else if avg < 2.0 { "text-red-500" }
        else if avg < 3.0 { "text-red-400" }
        else if avg < 3.5 { "text-amber-400" }
        else if avg < 4.0 { "text-yellow-400" }
        else if avg < 4.5 { "text-emerald-400" }
        else { "text-cyan-400" }
    });

    /* Count critical items (rated 1 or 2) for risk summary */
    let critical_count = {
        let r: Vec<RwSignal<u8>> = ratings.iter().map(|(s, _, _)| *s).collect();
        Signal::derive(move || r.iter().filter(|s| { let v = s.get(); v > 0 && v <= 2 }).count())
    };

    let saved = RwSignal::new(false);
    let on_save = move |_| { saved.set(true); };

    view! {
        <div class="mx-auto max-w-4xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"PV Startup Health Check"</h1>
            <p class="mt-2 text-slate-400">"Comprehensive PV system readiness assessment for startups and small MAHs (1-5)."</p>

            <div class="mt-6 flex flex-wrap gap-4 text-sm">
                <div class="rounded-lg border border-slate-800 bg-slate-900/50 px-4 py-3">
                    <p class="text-slate-500">"Areas Rated"</p>
                    <p class="text-xl font-bold text-white font-mono">{move || format!("{}/10", rated_count.get())}</p>
                </div>
                <div class="rounded-lg border border-slate-800 bg-slate-900/50 px-4 py-3">
                    <p class="text-slate-500">"Average"</p>
                    <p class="text-xl font-bold text-cyan-400 font-mono">{move || format!("{:.1}", avg_rating.get())}</p>
                </div>
                <div class="rounded-lg border border-slate-800 bg-slate-900/50 px-4 py-3">
                    <p class="text-slate-500">"Status"</p>
                    <p class=move || format!("text-xl font-bold font-mono {}", category_color.get())>{move || category.get()}</p>
                </div>
                <div class="rounded-lg border border-slate-800 bg-slate-900/50 px-4 py-3">
                    <p class="text-slate-500">"Critical Gaps"</p>
                    <p class=move || {
                        if critical_count.get() > 0 { "text-xl font-bold text-red-400 font-mono" }
                        else { "text-xl font-bold text-emerald-400 font-mono" }
                    }>{move || critical_count.get().to_string()}</p>
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
