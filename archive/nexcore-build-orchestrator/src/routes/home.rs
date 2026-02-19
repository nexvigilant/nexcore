//! Dashboard home route.

use leptos::prelude::*;

/// Home page — renders the dashboard.
#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <crate::components::dashboard::Dashboard />
    }
}
