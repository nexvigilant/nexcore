//! Home route — redirects to dashboard

use leptos::prelude::*;

use crate::components::DashboardView;

/// Home page — renders the main dashboard.
#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <DashboardView />
    }
}
