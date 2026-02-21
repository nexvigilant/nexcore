//! Header component - Score, Combo meter, Depth display
//!
//! Uses T2-P wrapper types for type safety.

use leptos::prelude::*;
use crate::{AMBER_GOLD, CYAN_GLOW, DEEP_SLATE, MAX_COMBO, SLATE_DIM};
use super::{Combo, Depth, GameState, Score};

#[component]
pub fn GameHeader() -> impl IntoView {
    let state = expect_context::<GameState>();

    view! {
        <header style=format!(
            "background: {}; padding: 1rem 2rem; \
             display: flex; justify-content: space-between; align-items: center; \
             border-bottom: 2px solid {};",
            DEEP_SLATE, AMBER_GOLD
        )>
            <ScoreDisplay score=state.score />
            <ComboMeter combo=state.combo />
            <DepthDisplay depth=state.depth />
        </header>
    }
}

#[component]
fn ScoreDisplay(score: RwSignal<Score>) -> impl IntoView {
    view! {
        <div style="text-align: left;">
            <div style=format!(
                "color: {}; font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.1em;",
                SLATE_DIM
            )>"SCORE"</div>
            <div style=format!("color: {}; font-size: 2rem; font-weight: bold;", AMBER_GOLD)>
                {move || format!("{:0>8}", score.get().0)}
            </div>
        </div>
    }
}

#[component]
fn ComboMeter(combo: RwSignal<Combo>) -> impl IntoView {
    view! {
        <div style="text-align: center;">
            <div style=format!(
                "color: {}; font-size: 0.75rem; text-transform: uppercase; \
                 letter-spacing: 0.1em; margin-bottom: 0.5rem;",
                SLATE_DIM
            )>"COMBO"</div>
            <div style="display: flex; gap: 4px; justify-content: center;">
                {move || render_combo_bars(combo.get())}
            </div>
            <div style=format!("color: {}; font-size: 1rem; margin-top: 0.25rem;", AMBER_GOLD)>
                {move || format!("×{:.1}", 1.0 + (combo.get().0 as f64 * 0.1))}
            </div>
        </div>
    }
}

fn render_combo_bars(current: Combo) -> impl IntoView {
    (0..MAX_COMBO).map(|i| {
        let bg = if i < current.0 { AMBER_GOLD } else { DEEP_SLATE };
        view! {
            <div style=format!(
                "width: 12px; height: 24px; background: {}; border-radius: 2px; transition: background 0.1s;",
                bg
            )></div>
        }
    }).collect_view()
}

#[component]
fn DepthDisplay(depth: RwSignal<Depth>) -> impl IntoView {
    view! {
        <div style="text-align: right;">
            <div style=format!(
                "color: {}; font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.1em;",
                SLATE_DIM
            )>"DEPTH"</div>
            <div style=format!("color: {}; font-size: 2rem; font-weight: bold;", CYAN_GLOW)>
                {move || format!("{:.2}m", depth.get().0)}
            </div>
        </div>
    }
}
