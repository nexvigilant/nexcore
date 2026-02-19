//! History page route.

use leptos::prelude::*;

/// History page — shows past pipeline runs.
#[component]
pub fn HistoryPage() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <h1 class="text-2xl font-bold">"Build History"</h1>
            <crate::components::history_table::HistoryTable runs=vec![] />
        </div>
    }
}
