use crate::components::card::Card;
use crate::components::metric::ThresholdMetric;
use leptos::prelude::*;

/// Signal detection workspace — 2x2 contingency table input + 5 metrics
/// Tier: T3 (domain — PV signal detection)
#[component]
pub fn SignalsPage() -> impl IntoView {
    // 2x2 table inputs
    let cell_a = RwSignal::new(String::new());
    let cell_b = RwSignal::new(String::new());
    let cell_c = RwSignal::new(String::new());
    let cell_d = RwSignal::new(String::new());

    // Results
    let prr = RwSignal::new(0.0_f64);
    let ror = RwSignal::new(0.0_f64);
    let ic = RwSignal::new(0.0_f64);
    let ebgm = RwSignal::new(0.0_f64);
    let chi2 = RwSignal::new(0.0_f64);
    let any_signal = RwSignal::new(false);
    let show_results = RwSignal::new(false);
    let loading = RwSignal::new(false);
    let error_msg = RwSignal::new(String::new());

    // Threshold selection
    let _threshold = RwSignal::new(String::from("default"));

    let results_class = move || {
        if any_signal.get() {
            "results-grid signal-alert"
        } else {
            "results-grid signal-clear"
        }
    };

    view! {
        <div class="page signals">
            <h1 class="page-title">"Signal Detection"</h1>

            <Card title="2x2 Contingency Table">
                <div class="contingency-table">
                    <div class="ct-header"></div>
                    <div class="ct-header">"Drug +"</div>
                    <div class="ct-header">"Drug \u{2212}"</div>

                    <div class="ct-label">"Event +"</div>
                    <input type="number" inputmode="numeric" class="ct-input"
                        placeholder="a"
                        prop:value=move || cell_a.get()
                        on:input=move |ev| cell_a.set(event_target_value(&ev))
                    />
                    <input type="number" inputmode="numeric" class="ct-input"
                        placeholder="b"
                        prop:value=move || cell_b.get()
                        on:input=move |ev| cell_b.set(event_target_value(&ev))
                    />

                    <div class="ct-label">"Event \u{2212}"</div>
                    <input type="number" inputmode="numeric" class="ct-input"
                        placeholder="c"
                        prop:value=move || cell_c.get()
                        on:input=move |ev| cell_c.set(event_target_value(&ev))
                    />
                    <input type="number" inputmode="numeric" class="ct-input"
                        placeholder="d"
                        prop:value=move || cell_d.get()
                        on:input=move |ev| cell_d.set(event_target_value(&ev))
                    />
                </div>

                <div class="threshold-select">
                    <label class="input-label">"Threshold Profile"</label>
                    <select class="input-field"
                        on:change=move |ev| {
                            _threshold.set(event_target_value(&ev));
                        }
                    >
                        <option value="default" selected>"Default"</option>
                        <option value="strict">"Strict"</option>
                        <option value="sensitive">"Sensitive"</option>
                    </select>
                </div>

                <button class="btn-primary"
                    disabled=move || loading.get()
                    on:click=move |_| {
                        loading.set(true);
                        // Will be wired to server function
                    }
                >
                    {move || if loading.get() { "Analyzing..." } else { "Detect Signals" }}
                </button>
            </Card>

            <Show when=move || !error_msg.get().is_empty()>
                <div class="error-banner">{move || error_msg.get()}</div>
            </Show>

            <Show when=move || show_results.get()>
                <div class=results_class>
                    <Card title="Results">
                        <div class="signal-summary">
                            {move || if any_signal.get() {
                                "SIGNAL DETECTED"
                            } else {
                                "No signal detected"
                            }}
                        </div>
                    </Card>

                    <ThresholdMetric label="PRR" value=Signal::derive(move || prr.get()) threshold=2.0/>
                    <ThresholdMetric label="ROR" value=Signal::derive(move || ror.get()) threshold=2.0/>
                    <ThresholdMetric label="IC" value=Signal::derive(move || ic.get()) threshold=0.0/>
                    <ThresholdMetric label="EBGM" value=Signal::derive(move || ebgm.get()) threshold=2.0/>
                    <ThresholdMetric label="Chi\u{00B2}" value=Signal::derive(move || chi2.get()) threshold=3.841/>
                </div>
            </Show>
        </div>
    }
}
