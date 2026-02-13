use crate::components::card::Card;
use crate::components::input::{NumberInput, TextInput};
use crate::components::metric::Metric;
use crate::components::modal::BottomSheet;
use crate::components::status_badge::{Status, StatusBadge};
use leptos::prelude::*;

/// Guardian real-time control page
/// Tier: T3 (domain — Guardian homeostasis)
#[component]
pub fn GuardianPage() -> impl IntoView {
    let guardian_status = RwSignal::new(Status::Unknown);
    let iteration = RwSignal::new(String::from("\u{2014}"));
    let amplification = RwSignal::new(String::from("\u{2014}"));
    let sensors = RwSignal::new(String::from("\u{2014}"));
    let actuators = RwSignal::new(String::from("\u{2014}"));
    let log_entries = RwSignal::new(Vec::<String>::new());

    // Evaluation form
    let drug_name = RwSignal::new(String::new());
    let event_name = RwSignal::new(String::new());
    let case_count = RwSignal::new(String::new());
    let eval_result = RwSignal::new(String::new());
    let show_eval = RwSignal::new(false);

    view! {
        <div class="page guardian">
            <h1 class="page-title">"Guardian Control"</h1>

            <Card title="Homeostasis Loop">
                <StatusBadge status=Signal::derive(move || guardian_status.get()) label="Loop"/>
                <div class="metrics-row">
                    <Metric label="Iteration" value=Signal::derive(move || iteration.get())/>
                    <Metric label="Amplification" value=Signal::derive(move || amplification.get())/>
                </div>
                <div class="metrics-row">
                    <Metric label="Sensors" value=Signal::derive(move || sensors.get())/>
                    <Metric label="Actuators" value=Signal::derive(move || actuators.get())/>
                </div>
            </Card>

            <Card title="Controls">
                <div class="control-buttons">
                    <button class="btn-primary">"Tick"</button>
                    <button class="btn-secondary">"Reset"</button>
                    <button class="btn-secondary">"Refresh Status"</button>
                </div>
            </Card>

            <Card title="Risk Evaluation">
                <div class="eval-form">
                    <TextInput
                        label="Drug Name"
                        value=drug_name
                        placeholder="e.g., Aspirin"
                    />
                    <TextInput
                        label="Adverse Event"
                        value=event_name
                        placeholder="e.g., Headache"
                    />
                    <NumberInput
                        label="Case Count"
                        value=case_count
                        placeholder="e.g., 15"
                    />
                    <button class="btn-primary"
                        on:click=move |_| {
                            show_eval.set(true);
                        }
                    >"Evaluate Risk"</button>
                </div>
            </Card>

            <Show when=move || show_eval.get()>
                <BottomSheet
                    on_close=Callback::new(move |_| show_eval.set(false))
                    title="Risk Evaluation"
                >
                    <div class="eval-result">
                        <pre>{move || eval_result.get()}</pre>
                    </div>
                </BottomSheet>
            </Show>

            <Card title="Activity Log">
                <div class="log-entries">
                    {move || {
                        let entries = log_entries.get();
                        if entries.is_empty() {
                            view! { <p class="card-hint">"No activity yet. Tap Tick to start."</p> }.into_any()
                        } else {
                            view! {
                                <ul class="log-list">
                                    {entries.into_iter().map(|e| view! { <li>{e}</li> }).collect::<Vec<_>>()}
                                </ul>
                            }.into_any()
                        }
                    }}
                </div>
            </Card>
        </div>
    }
}
