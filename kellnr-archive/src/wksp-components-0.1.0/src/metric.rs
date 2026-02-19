use leptos::prelude::*;

/// Metric display widget with label, value, and optional unit
/// Tier: T2-P (Quantity + Mapping)
#[component]
pub fn Metric(
    label: &'static str,
    #[prop(into)] value: Signal<String>,
    #[prop(optional)] unit: &'static str,
    #[prop(optional)] class: &'static str,
) -> impl IntoView {
    view! {
        <div class=format!("metric {class}")>
            <span class="metric-label">{label}</span>
            <span class="metric-value">{move || value.get()}</span>
            {if !unit.is_empty() {
                Some(view! { <span class="metric-unit">{unit}</span> })
            } else {
                None
            }}
        </div>
    }
}

/// Metric that highlights based on threshold
#[component]
pub fn ThresholdMetric(
    label: &'static str,
    #[prop(into)] value: Signal<f64>,
    threshold: f64,
    #[prop(optional)] unit: &'static str,
    #[prop(optional)] invert: bool,
) -> impl IntoView {
    let class = move || {
        let v = value.get();
        let exceeded = if invert {
            v < threshold
        } else {
            v >= threshold
        };
        if exceeded {
            "metric signal-detected"
        } else {
            "metric signal-clear"
        }
    };

    view! {
        <div class=class>
            <span class="metric-label">{label}</span>
            <span class="metric-value">{move || format!("{:.3}", value.get())}</span>
            {if !unit.is_empty() {
                Some(view! { <span class="metric-unit">{unit}</span> })
            } else {
                None
            }}
        </div>
    }
}
