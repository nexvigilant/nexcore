use crate::components::card::Card;
use crate::components::input::{NumberInput, TextInput};
use leptos::prelude::*;

/// QBRI calculator — Quantitative Benefit-Risk Index
/// Tier: T3 (domain — benefit-risk)
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
        <div class="page benefit-risk">
            <h1 class="page-title">"Benefit-Risk (QBRI)"</h1>

            <Card title="Benefit">
                <TextInput
                    label="Name"
                    value=benefit_name
                    placeholder="e.g., Efficacy"
                />
                <div class="row-2">
                    <NumberInput
                        label="Weight"
                        value=benefit_weight
                        step="0.1"
                        decimal=true
                    />
                    <NumberInput
                        label="Score"
                        value=benefit_score
                        step="0.1"
                        decimal=true
                    />
                </div>
            </Card>

            <Card title="Risk">
                <TextInput
                    label="Name"
                    value=risk_name
                    placeholder="e.g., Adverse Events"
                />
                <div class="row-2">
                    <NumberInput
                        label="Weight"
                        value=risk_weight
                        step="0.1"
                        decimal=true
                    />
                    <NumberInput
                        label="Score"
                        value=risk_score
                        step="0.1"
                        decimal=true
                    />
                </div>
            </Card>

            <button class="btn-primary btn-full"
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

            <Show when=move || qbri_result.get().is_some()>
                <Card title="QBRI Result">
                    <div class="qbri-result">
                        <span class="result-score">{move || {
                            qbri_result.get().map(|q| format!("{q:.3}")).unwrap_or_default()
                        }}</span>
                        <span class="result-interpretation">{move || interpretation.get()}</span>
                    </div>
                </Card>
            </Show>
        </div>
    }
}
