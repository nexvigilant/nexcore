//! nexcore-control-center - nexcore Unified Monitoring Dashboard
//!
//! Built with Ferrostack (Leptos + Axum). 100% Rust.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod api;
pub mod components;
pub mod routes;

use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

/// Shell function for SSR - wraps the app in HTML structure
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en" class="dark">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <link rel="stylesheet" href="/pkg/style.css"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body class="bg-gray-900 text-white">
                <App/>
            </body>
        </html>
    }
}

/// Main application component
#[component]
pub fn App() -> impl IntoView {
    // Provides context for meta tags
    provide_meta_context();

    view! {
        <Title text="Welcome to Ferrostack"/>
        <Router>
            <main class="container mx-auto px-4">
                <Routes fallback=|| "404 - Not Found".into_view()>
                    <Route path=StaticSegment("") view=routes::HomePage />
                    <Route path=StaticSegment("about") view=routes::AboutPage />
                </Routes>
            </main>
        </Router>
    }
}
