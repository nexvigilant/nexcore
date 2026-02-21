//! Borrow Miner - A psychologically satisfying arcade mining game
//! that teaches Rust ownership concepts through gameplay mechanics.
//!
//! Built with Leptos (Ferrostack) - ported from Bevy.

pub mod game;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}

// Re-export for convenience
pub use game::GamePage;

use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

// ═══════════════════════════════════════════════════════════════════════════
// TERMINAL LUXE COLOR PALETTE
// ═══════════════════════════════════════════════════════════════════════════

pub const FOUNDATION_NAVY: &str = "#0a1628";
pub const AMBER_GOLD: &str = "#f4a623";
pub const PHOSPHOR_GREEN: &str = "#4ade80";
pub const CYAN_GLOW: &str = "#22d3ee";
pub const SLATE_DIM: &str = "#94a3b8";
pub const DEEP_SLATE: &str = "#1e293b";
pub const ERROR_RED: &str = "#ef4444";

// ═══════════════════════════════════════════════════════════════════════════
// GAME CONSTANTS
// ═══════════════════════════════════════════════════════════════════════════

pub const MAX_COMBO: u32 = 10;
pub const MAX_OWNED: usize = 5;
pub const COMBO_DECAY_INTERVAL: f64 = 2.0;
pub const MINING_COOLDOWN: f64 = 0.2;

/// Shell function for SSR - wraps the app in HTML structure
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
            </head>
            <body style=format!("margin: 0; padding: 0; background: {}; min-height: 100vh;", FOUNDATION_NAVY)>
                <App/>
            </body>
        </html>
    }
}

/// Main application component
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Title text="Borrow Miner - Learn Rust Ownership"/>
        <Stylesheet href="/pkg/borrow-miner.css"/>
        <Router>
            <Routes fallback=|| "404 - Not Found".into_view()>
                <Route path=StaticSegment("") view=game::GamePage />
            </Routes>
        </Router>
    }
}
