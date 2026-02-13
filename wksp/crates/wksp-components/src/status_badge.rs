use leptos::prelude::*;

/// Health/status indicator badge
/// Tier: T2-P (State + Existence)
#[component]
pub fn StatusBadge(
    #[prop(into)] status: Signal<Status>,
    #[prop(optional)] label: &'static str,
) -> impl IntoView {
    let class = move || {
        let s = status.get();
        let color = match s {
            Status::Healthy => "status-healthy",
            Status::Warning => "status-warning",
            Status::Error => "status-error",
            Status::Unknown => "status-unknown",
            Status::Offline => "status-offline",
        };
        format!("status-badge {color}")
    };

    let text = move || {
        let s = status.get();
        match s {
            Status::Healthy => "Healthy",
            Status::Warning => "Warning",
            Status::Error => "Error",
            Status::Unknown => "Unknown",
            Status::Offline => "Offline",
        }
    };

    view! {
        <span class=class>
            <span class="status-dot"></span>
            {if !label.is_empty() {
                Some(view! { <span class="status-label">{label}</span> })
            } else {
                None
            }}
            <span class="status-text">{text}</span>
        </span>
    }
}

/// System status enum
/// Tier: T1 (Sum type — Σ)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Status {
    Healthy,
    Warning,
    Error,
    #[default]
    Unknown,
    Offline,
}
