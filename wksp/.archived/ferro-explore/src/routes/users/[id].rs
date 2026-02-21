//! /users/[id] route (dynamic)

use leptos::prelude::*;
use leptos_router::hooks::use_params;
use leptos_router::params::Params;

#[derive(Params, PartialEq, Clone)]
struct RouteParams {
    pub id: String,
}

#[component]
pub fn IdPage() -> impl IntoView {
    let params = use_params::<RouteParams>();
    let id = move || params.get().map(|p| p.id.clone()).unwrap_or_default();

    view! {
        <div class="page">
            <h1>"IdPage"</h1>
            // TODO: Use extracted params
        </div>
    }
}
