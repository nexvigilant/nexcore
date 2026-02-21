//! Nucleus — NexVigilant Unified Portal
//!
//! The central hub (λ Location) unifying both pillars:
//! - Vigilance: signals, guardian, PVDSL, brain, causality
//! - Empowerment: academy, community, careers, intelligence
//!
//! 100% Rust frontend via Leptos 0.7 SSR + WASM hydration.

pub use wksp_api_client as api_client;
pub use wksp_components as ui;

pub mod app;
pub mod auth;
pub mod components;
pub mod firebase;
pub mod sections;
pub mod stripe;

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
                <meta name="mobile-web-app-capable" content="yes"/>
                <meta name="apple-mobile-web-app-status-bar-style" content="black-translucent"/>
                <meta name="description" content="NexVigilant unified portal — Vigilance + Empowerment for pharmacovigilance professionals"/>
                <link rel="manifest" href="/manifest.json"/>
                <link rel="icon" type="image/svg+xml" href="/favicon.svg"/>
                <link rel="apple-touch-icon" href="/icons/apple-touch-icon.png"/>
                <script src="https://cdn.tailwindcss.com"></script>
                <script>
                  "tailwind.config = {
                    theme: {
                      extend: {
                        colors: {
                          nexcore: {
                            black: '#020617',
                            dark: '#0f172a',
                            gold: '#fbbf24',
                            cyan: '#22d3ee',
                            amber: '#f59e0b',
                          }
                        },
                        boxShadow: {
                          'glow-cyan': '0 0 15px -3px rgba(34, 211, 238, 0.4)',
                          'glow-amber': '0 0 15px -3px rgba(245, 158, 11, 0.4)',
                        }
                      }
                    }
                  }"
                </script>
                <style>
                  "@layer components {
                    .glass-panel {
                      background: rgba(2, 6, 23, 0.5);
                      backdrop-filter: blur(12px);
                      border: 1px solid rgba(255, 255, 255, 0.1);
                    }
                    .glow-border-cyan {
                      border-color: rgba(34, 211, 238, 0.3);
                      box-shadow: 0 0 15px -3px rgba(34, 211, 238, 0.4);
                    }
                  }"
                </style>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <leptos_meta::MetaTags/>
            </head>
            <body class="bg-slate-950 text-slate-100 antialiased">
                <App/>
                <script>
                    "if ('serviceWorker' in navigator) { navigator.serviceWorker.register('/sw.js'); }"
                </script>
            </body>
        </html>
    }
}