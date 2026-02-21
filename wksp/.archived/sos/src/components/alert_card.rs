/// Guardian alert display card — ν Frequency + ∂ Boundary (severity levels)
use leptos::prelude::*;

use crate::api::guardian::GuardianAlert;

#[component]
pub fn AlertCard(alert: GuardianAlert) -> impl IntoView {
    let severity_class = match alert.severity.to_lowercase().as_str() {
        "critical" => "severity-critical",
        "high" => "severity-high",
        "medium" => "severity-medium",
        _ => "severity-low",
    };

    view! {
        <div class=format!("alert-card {severity_class}")>
            <div class="alert-header">
                <span class="alert-severity">{alert.severity.clone()}</span>
                <span class="alert-time">{alert.timestamp.clone()}</span>
            </div>
            <div class="alert-body">
                <p class="alert-message">{alert.message.clone()}</p>
            </div>
            <div class="alert-footer">
                <span class="alert-source">{alert.source.clone()}</span>
                <span class="alert-id">{alert.id.clone()}</span>
            </div>
        </div>
    }
}
