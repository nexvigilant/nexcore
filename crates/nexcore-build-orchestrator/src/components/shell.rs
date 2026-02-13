//! HTML shell, App component, and router setup.

use leptos::prelude::*;
use leptos_meta::{MetaTags, Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
};

/// HTML shell for SSR rendering.
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <link
                    href="https://cdn.jsdelivr.net/npm/tailwindcss@2/dist/tailwind.min.css"
                    rel="stylesheet"
                />
                <style>
                    ".status-pending { color: #f59e0b; }"
                    ".status-running { color: #3b82f6; animation: pulse 2s infinite; }"
                    ".status-completed { color: #10b981; }"
                    ".status-failed { color: #ef4444; }"
                    ".status-cancelled { color: #6b7280; }"
                    ".status-skipped { color: #9ca3af; }"
                    "@keyframes pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.5; } }"
                    ".log-output { font-family: 'Fira Code', monospace; font-size: 0.75rem; }"
                </style>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body class="bg-gray-900 text-gray-100 min-h-screen">
                <App/>
            </body>
        </html>
    }
}

/// Main App component with routing.
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    view! {
        <Title text="NexCore Build Orchestrator"/>
        <Router>
            <Nav/>
            <main class="container mx-auto px-4 py-6">
                <AppRoutes/>
            </main>
        </Router>
    }
}

/// Navigation bar.
#[component]
fn Nav() -> impl IntoView {
    view! {
        <nav class="bg-gray-800 border-b border-gray-700">
            <div class="container mx-auto px-4 py-3 flex items-center justify-between">
                <div class="flex items-center space-x-2">
                    <span class="text-xl font-bold text-green-400">"Build Orchestrator"</span>
                    <span class="text-xs text-gray-500">"v1.0"</span>
                </div>
                <div class="flex space-x-4">
                    <a href="/" class="text-gray-300 hover:text-white">"Dashboard"</a>
                    <a href="/history" class="text-gray-300 hover:text-white">"History"</a>
                </div>
            </div>
        </nav>
    }
}

/// Route definitions.
#[component]
fn AppRoutes() -> impl IntoView {
    view! {
        <Routes fallback=|| view! { <p class="text-red-400">"404 — Not Found"</p> }.into_view()>
            <Route path=StaticSegment("") view=crate::routes::home::HomePage />
            <Route path=StaticSegment("history") view=crate::routes::history::HistoryPage />
        </Routes>
    }
}
