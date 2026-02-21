/// Signal detection interface — ∂ Boundary + N Quantity + κ Comparison
/// Input 2x2 contingency table, get PRR/ROR/IC/EBGM/Chi-squared
use leptos::prelude::*;

use crate::api::signal::{self, SignalInput, SignalResponse};
use crate::components::metric_badge::MetricBadge;
use crate::components::signal_result::SignalResultCard;

#[component]
pub fn SignalsPage() -> impl IntoView {
    let (a, set_a) = signal(String::from("15"));
    let (b, set_b) = signal(String::from("100"));
    let (c, set_c) = signal(String::from("20"));
    let (d, set_d) = signal(String::from("10000"));

    let (result, set_result) = signal(None::<Result<SignalResponse, String>>);
    let (loading, set_loading) = signal(false);

    let run_detection = move |_| {
        let a_val: u64 = a.get().parse().unwrap_or(0);
        let b_val: u64 = b.get().parse().unwrap_or(0);
        let c_val: u64 = c.get().parse().unwrap_or(0);
        let d_val: u64 = d.get().parse().unwrap_or(0);

        set_loading.set(true);
        set_result.set(None);

        let input = SignalInput {
            a: a_val,
            b: b_val,
            c: c_val,
            d: d_val,
        };

        wasm_bindgen_futures::spawn_local(async move {
            let res = signal::detect_signal(&input).await;
            match res {
                Ok(resp) => set_result.set(Some(Ok(resp))),
                Err(e) => set_result.set(Some(Err(e.message))),
            }
            set_loading.set(false);
        });
    };

    view! {
        <div class="page">
            <header class="page-header">
                <h1 class="page-title">"Signal Detection"</h1>
                <p class="page-subtitle">"2x2 Contingency Analysis"</p>
            </header>

            <div class="input-grid">
                <div class="input-group">
                    <label class="input-label">"a (drug+event)"</label>
                    <input
                        class="input-field"
                        type="number"
                        prop:value=a
                        on:input=move |ev| set_a.set(event_target_value(&ev))
                    />
                </div>
                <div class="input-group">
                    <label class="input-label">"b (drug+no event)"</label>
                    <input
                        class="input-field"
                        type="number"
                        prop:value=b
                        on:input=move |ev| set_b.set(event_target_value(&ev))
                    />
                </div>
                <div class="input-group">
                    <label class="input-label">"c (no drug+event)"</label>
                    <input
                        class="input-field"
                        type="number"
                        prop:value=c
                        on:input=move |ev| set_c.set(event_target_value(&ev))
                    />
                </div>
                <div class="input-group">
                    <label class="input-label">"d (no drug+no event)"</label>
                    <input
                        class="input-field"
                        type="number"
                        prop:value=d
                        on:input=move |ev| set_d.set(event_target_value(&ev))
                    />
                </div>
            </div>

            <button
                class="btn-primary"
                on:click=run_detection
                disabled=loading
            >
                {move || if loading.get() { "Analyzing..." } else { "Detect Signal" }}
            </button>

            {move || {
                result.get().map(|res| match res {
                    Ok(resp) => view! {
                        <div class="results-section">
                            <div class="results-header">
                                <MetricBadge
                                    label="Overall Signal".to_string()
                                    is_signal=resp.overall_signal
                                />
                            </div>
                            <p class="results-summary">{resp.summary.clone()}</p>
                            <div class="results-grid">
                                {resp.metrics.iter().map(|m| {
                                    view! { <SignalResultCard metric=m.clone() /> }
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>
                    }.into_any(),
                    Err(msg) => view! {
                        <div class="error-card">
                            <div class="error-msg">{msg.clone()}</div>
                        </div>
                    }.into_any(),
                })
            }}
        </div>
    }
}

fn event_target_value(ev: &leptos::ev::Event) -> String {
    use wasm_bindgen::JsCast;
    let target = ev.target().expect("event target");
    target
        .unchecked_into::<web_sys::HtmlInputElement>()
        .value()
}
