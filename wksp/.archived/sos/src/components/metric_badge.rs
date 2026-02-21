/// Threshold pass/fail badge — kappa Comparison + partial Boundary
use leptos::prelude::*;

#[component]
pub fn MetricBadge(label: String, is_signal: bool) -> impl IntoView {
    let class = if is_signal {
        "metric-badge signal-detected"
    } else {
        "metric-badge no-signal"
    };

    let text = if is_signal {
        "SIGNAL"
    } else {
        "No Signal"
    };

    view! {
        <span class=class title=label>
            {text}
        </span>
    }
}
