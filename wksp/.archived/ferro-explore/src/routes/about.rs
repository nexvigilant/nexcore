//! About page route

use leptos::prelude::*;

/// About page component
#[component]
pub fn AboutPage() -> impl IntoView {
    view! {
        <div class="py-8">
            <h1 class="text-4xl font-bold mb-4">"About"</h1>
            <p class="text-gray-600 mb-4">
                "Built with Leptos + Axum, styled like Next.js."
            </p>
            <a href="/" class="text-blue-600 hover:underline">
                "Back home"
            </a>
        </div>
    }
}
