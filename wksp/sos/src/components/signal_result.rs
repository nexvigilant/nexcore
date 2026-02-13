/// Signal metric result display — N Quantity + kappa Comparison
use leptos::prelude::*;

use crate::api::signal::MetricResult;
use crate::components::metric_badge::MetricBadge;

#[component]
pub fn SignalResultCard(metric: MetricResult) -> impl IntoView {
    view! {
        <div class="signal-result-card">
            <div class="result-header">
                <span class="result-name">{metric.name.clone()}</span>
                <MetricBadge label=metric.name.clone() is_signal=metric.signal />
            </div>
            <div class="result-values">
                <div class="result-row">
                    <span class="result-label">"Value"</span>
                    <span class="result-value">{format!("{:.4}", metric.value)}</span>
                </div>
                <div class="result-row">
                    <span class="result-label">"Threshold"</span>
                    <span class="result-value">{format!("{:.4}", metric.threshold)}</span>
                </div>
            </div>
            <p class="result-interpretation">{metric.interpretation.clone()}</p>
        </div>
    }
}
