//! Benefit-risk assessment — QBRI calculator

use leptos::prelude::*;

#[component]
pub fn BenefitRiskPage() -> impl IntoView {
    let benefit_name = RwSignal::new(String::new());
    let benefit_weight = RwSignal::new(String::from("1.0"));
    let benefit_score = RwSignal::new(String::from("0.5"));

    let risk_name = RwSignal::new(String::new());
    let risk_weight = RwSignal::new(String::from("1.0"));
    let risk_score = RwSignal::new(String::from("0.3"));

    let qbri_result = RwSignal::new(Option::<f64>::None);
    let interpretation = RwSignal::new(String::new());

    view! {
        <div class="mx-auto max-w-4xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white">"Benefit-Risk Assessment"</h1>
            <p class="mt-2 text-slate-400">"Quantitative Benefit-Risk Index (QBRI) computation"</p>

            // Benefit inputs
            <div class="mt-8 rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                <h2 class="text-lg font-semibold text-emerald-400">"Benefit"</h2>
                <div class="mt-4 space-y-4">
                    <div>
                        <label class="block text-sm font-medium text-slate-400">"Name"</label>
                        <input type="text"
                            class="mt-1 w-full rounded-lg border border-slate-700 bg-slate-800 px-3 py-2 text-white placeholder:text-slate-600 focus:border-emerald-500 focus:outline-none"
                            placeholder="e.g., Efficacy"
                            prop:value=move || benefit_name.get()
                            on:input=move |ev| benefit_name.set(event_target_value(&ev))
                        />
                    </div>
                    <div class="grid grid-cols-2 gap-4">
                        <div>
                            <label class="block text-sm font-medium text-slate-400">"Weight"</label>
                            <input type="number" step="0.1"
                                class="mt-1 w-full rounded-lg border border-slate-700 bg-slate-800 px-3 py-2 text-white focus:border-emerald-500 focus:outline-none"
                                prop:value=move || benefit_weight.get()
                                on:input=move |ev| benefit_weight.set(event_target_value(&ev))
                            />
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-slate-400">"Score (0-1)"</label>
                            <input type="number" step="0.1"
                                class="mt-1 w-full rounded-lg border border-slate-700 bg-slate-800 px-3 py-2 text-white focus:border-emerald-500 focus:outline-none"
                                prop:value=move || benefit_score.get()
                                on:input=move |ev| benefit_score.set(event_target_value(&ev))
                            />
                        </div>
                    </div>
                </div>
            </div>

            // Risk inputs
            <div class="mt-4 rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                <h2 class="text-lg font-semibold text-red-400">"Risk"</h2>
                <div class="mt-4 space-y-4">
                    <div>
                        <label class="block text-sm font-medium text-slate-400">"Name"</label>
                        <input type="text"
                            class="mt-1 w-full rounded-lg border border-slate-700 bg-slate-800 px-3 py-2 text-white placeholder:text-slate-600 focus:border-red-500 focus:outline-none"
                            placeholder="e.g., Adverse Events"
                            prop:value=move || risk_name.get()
                            on:input=move |ev| risk_name.set(event_target_value(&ev))
                        />
                    </div>
                    <div class="grid grid-cols-2 gap-4">
                        <div>
                            <label class="block text-sm font-medium text-slate-400">"Weight"</label>
                            <input type="number" step="0.1"
                                class="mt-1 w-full rounded-lg border border-slate-700 bg-slate-800 px-3 py-2 text-white focus:border-red-500 focus:outline-none"
                                prop:value=move || risk_weight.get()
                                on:input=move |ev| risk_weight.set(event_target_value(&ev))
                            />
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-slate-400">"Score (0-1)"</label>
                            <input type="number" step="0.1"
                                class="mt-1 w-full rounded-lg border border-slate-700 bg-slate-800 px-3 py-2 text-white focus:border-red-500 focus:outline-none"
                                prop:value=move || risk_score.get()
                                on:input=move |ev| risk_score.set(event_target_value(&ev))
                            />
                        </div>
                    </div>
                </div>
            </div>

            // Calculate button
            <button
                class="mt-4 w-full rounded-lg bg-amber-600 px-4 py-3 text-sm font-medium text-white hover:bg-amber-500 transition-colors"
                on:click=move |_| {
                    let bw: f64 = benefit_weight.get().parse().unwrap_or(1.0);
                    let bs: f64 = benefit_score.get().parse().unwrap_or(0.0);
                    let rw: f64 = risk_weight.get().parse().unwrap_or(1.0);
                    let rs: f64 = risk_score.get().parse().unwrap_or(0.0);

                    let benefit_total = bw * bs;
                    let risk_total = rw * rs;
                    let qbri = if risk_total > 0.0 {
                        benefit_total / risk_total
                    } else {
                        f64::INFINITY
                    };
                    qbri_result.set(Some(qbri));

                    let interp = if qbri > 2.0 {
                        "Favorable benefit-risk profile"
                    } else if qbri > 1.0 {
                        "Marginal benefit-risk profile"
                    } else {
                        "Unfavorable benefit-risk profile"
                    };
                    interpretation.set(interp.to_string());
                }
            >"Calculate QBRI"</button>

            // Result
            <Show when=move || qbri_result.get().is_some()>
                <div class=move || {
                    let q = qbri_result.get().unwrap_or(0.0);
                    if q > 2.0 {
                        "mt-6 rounded-xl border-2 border-emerald-500/50 bg-emerald-500/5 p-6 text-center"
                    } else if q > 1.0 {
                        "mt-6 rounded-xl border-2 border-amber-500/50 bg-amber-500/5 p-6 text-center"
                    } else {
                        "mt-6 rounded-xl border-2 border-red-500/50 bg-red-500/5 p-6 text-center"
                    }
                }>
                    <p class="text-xs font-medium uppercase tracking-wider text-slate-500">"QBRI"</p>
                    <p class="mt-2 text-4xl font-bold text-white">{move || {
                        qbri_result.get().map(|q| format!("{q:.3}")).unwrap_or_default()
                    }}</p>
                    <p class=move || {
                        let q = qbri_result.get().unwrap_or(0.0);
                        if q > 2.0 {
                            "mt-2 text-lg font-semibold text-emerald-400"
                        } else if q > 1.0 {
                            "mt-2 text-lg font-semibold text-amber-400"
                        } else {
                            "mt-2 text-lg font-semibold text-red-400"
                        }
                    }>
                        {move || interpretation.get()}
                    </p>
                    <p class="mt-2 text-xs text-slate-500">"Favorable (>2.0) | Marginal (1.0-2.0) | Unfavorable (<1.0)"</p>
                </div>
            </Show>
        </div>
    }
}
