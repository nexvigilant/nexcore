//! education-machine — Primitive-first learning engine
//!
//! Full-stack Leptos + Axum application for teaching any subject
//! via primitive decomposition, Bayesian mastery assessment,
//! and spaced repetition review scheduling.

pub mod routes;
pub mod components;
pub mod server;

use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    ParamSegment, StaticSegment,
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

/// Main app — routes to pages
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    view! {
        <Title text="Education Machine"/>
        <Router>
            <AppRoutes/>
        </Router>
    }
}

/// Route definitions
#[component]
fn AppRoutes() -> impl IntoView {
    view! {
        <Routes fallback=|| "404".into_view()>
            <Route path=StaticSegment("") view=routes::HomePage />
            <Route path=(StaticSegment("subject"), ParamSegment("id")) view=routes::SubjectPage />
            <Route path=(StaticSegment("lesson"), ParamSegment("id")) view=routes::LessonPage />
            <Route path=(StaticSegment("assess"), ParamSegment("subject_id")) view=routes::AssessmentPage />
        </Routes>
    }
}
