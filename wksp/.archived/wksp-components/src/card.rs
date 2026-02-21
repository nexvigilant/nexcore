use leptos::prelude::*;

/// Reusable card container for dashboard widgets
/// Tier: T2-C (Boundary + State + Sequence)
#[component]
pub fn Card(
    #[prop(optional)] title: &'static str,
    #[prop(optional)] class: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <div class=format!("card {class}")>
            {if !title.is_empty() {
                Some(view! { <h3 class="card-title">{title}</h3> })
            } else {
                None
            }}
            <div class="card-body">
                {children()}
            </div>
        </div>
    }
}

/// Card variant with loading skeleton
#[component]
pub fn CardLoading() -> impl IntoView {
    view! {
        <div class="card card-loading">
            <div class="skeleton skeleton-title"></div>
            <div class="skeleton skeleton-text"></div>
            <div class="skeleton skeleton-text short"></div>
        </div>
    }
}
