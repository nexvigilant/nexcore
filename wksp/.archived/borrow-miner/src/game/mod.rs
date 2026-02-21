//! Game module - Core game logic and UI for Borrow Miner

mod types;
mod state;
mod header;
mod mining_area;
mod inventory;
mod signals;
mod achievements;
mod challenges;
#[cfg(feature = "ssr")]
mod faers;

pub use types::*;
pub use state::*;
pub use header::*;
pub use mining_area::*;
pub use inventory::*;
pub use signals::*;
pub use achievements::*;
pub use challenges::*;
#[cfg(feature = "ssr")]
pub use faers::*;

use leptos::prelude::*;
use crate::FOUNDATION_NAVY;

/// Main game page - composes all sub-components
#[component]
pub fn GamePage() -> impl IntoView {
    // Initialize game state context
    let state = GameState::new();
    provide_context(state.clone());

    let shake = state.shake;

    let shake_style = move || {
        if shake.get() {
            "transform: translate(2px, -2px);"
        } else {
            ""
        }
    };

    let handle_mine = {
        let state = state.clone();
        move |_| state.mine()
    };

    view! {
        <div
            style=move || format!(
                "min-height: 100vh; background: {}; color: white; \
                 font-family: 'JetBrains Mono', 'Fira Code', monospace; \
                 user-select: none; {}",
                FOUNDATION_NAVY,
                shake_style()
            )
            on:click=handle_mine
        >
            <GameHeader />
            <MiningArea />
            <InventoryFooter />
        </div>
    }
}
