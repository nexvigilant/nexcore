//! adventure-hud - A Ferrostack Application
//!
//! Full-stack Leptos + Axum HUD for tracking Claude Code adventures.

pub mod routes;
pub mod components;
pub mod server;

use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

/// Shell function for SSR
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <link href="https://cdn.jsdelivr.net/npm/tailwindcss@2/dist/tailwind.min.css" rel="stylesheet"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body class="bg-gray-900">
                <App/>
            </body>
        </html>
    }
}

/// Main app - routes to Dashboard
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    view! {
        <Title text="Adventure HUD"/>
        <Router>
            <AppRoutes/>
        </Router>
    }
}

/// Route definitions extracted for clarity
#[component]
fn AppRoutes() -> impl IntoView {
    view! {
        <Routes fallback=|| "404".into_view()>
            <Route path=StaticSegment("") view=components::Dashboard />
            <Route path=StaticSegment("about") view=routes::AboutPage />
        </Routes>
    }
}
