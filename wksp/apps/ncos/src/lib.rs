//! NCOS — NexCore Operating System Mobile PWA
//!
//! Full-stack Leptos 0.7 application providing mobile access
//! to all NexCore capabilities via the nexcore-api REST backend.
//!
//! ## Primitive Foundation
//! - σ (Sequence): Request → API → Response render pipeline
//! - μ (Mapping): API endpoint → UI module mapping
//! - ρ (Recursion): Recursive component tree (Leptos views)
//! - ∂ (Boundary): PWA offline/online boundary, auth boundary
//! - ς (State): Reactive signals (`RwSignal<T>`)
//! - ∃ (Existence): Service worker cache existence checks

// Use shared workspace crates
pub use wksp_api_client as api_client;
pub use wksp_components as components;

pub mod app;
pub mod auth;
pub mod pages;

use leptos::prelude::*;

pub use app::App;

/// Shell function for SSR — wraps the app in HTML structure
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover"/>
                <meta name="theme-color" content="#1a1a2e"/>
                <meta name="apple-mobile-web-app-capable" content="yes"/>
                <meta name="apple-mobile-web-app-status-bar-style" content="black-translucent"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <leptos_meta::MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}
