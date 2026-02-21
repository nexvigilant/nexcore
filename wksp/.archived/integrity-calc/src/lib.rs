//! integrity-calc — AI Text Detection Pipeline Calculator
//!
//! Leptos 0.7 + Axum web application for interactive integrity analysis.
//! Computation is inlined (~400 lines of pure math), zero external coupling.

pub mod pipeline;
pub mod routes;
pub mod components;

use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

/// WASM entry point for hydration.
#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}

/// Shell function for SSR — wraps the app in HTML structure.
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
                // Tailwind CDN for styling
                <script src="https://cdn.tailwindcss.com"></script>
            </head>
            <body class="bg-gray-900">
                <App/>
            </body>
        </html>
    }
}

/// Main application component.
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Title text="Integrity Calculator — AI Text Detection"/>
        <Router>
            <Routes fallback=|| {
                view! {
                    <div class="min-h-screen bg-gray-900 text-gray-100 flex items-center justify-center">
                        <div class="text-center">
                            <h1 class="text-4xl font-bold mb-4">"404"</h1>
                            <p class="text-gray-400">"Page not found"</p>
                            <a href="/" class="text-blue-400 hover:text-blue-300 mt-4 inline-block">
                                "\u{2190} Back to Calculator"
                            </a>
                        </div>
                    </div>
                }.into_view()
            }>
                <Route path=StaticSegment("") view=routes::HomePage />
                <Route path=StaticSegment("about") view=routes::AboutPage />
            </Routes>
        </Router>
    }
}
