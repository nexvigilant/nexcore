//! Home page route

use leptos::prelude::*;

/// Home page component
#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <div class="py-8">
            <h1 class="text-4xl font-bold mb-4">"Welcome to Ferrostack"</h1>
            <p class="text-gray-600 mb-4">
                "A Rust full-stack framework for Next.js developers."
            </p>
            <a href="/about" class="text-blue-600 hover:underline">
                "Learn more"
            </a>
        </div>
    }
}
