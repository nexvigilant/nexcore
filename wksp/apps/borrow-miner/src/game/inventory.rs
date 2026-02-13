//! Inventory footer - Owned ores and Drop button

use leptos::prelude::*;
use leptos::ev::MouseEvent;
use std::collections::VecDeque;
use crate::{
    DEEP_SLATE, FOUNDATION_NAVY, MAX_OWNED, PHOSPHOR_GREEN, SLATE_DIM,
};
use super::{GameState, OreType};

#[component]
pub fn InventoryFooter() -> impl IntoView {
    let state = expect_context::<GameState>();

    let handle_drop = {
        let state = state.clone();
        move |e: MouseEvent| {
            e.stop_propagation();
            state.drop_ore();
        }
    };

    view! {
        <footer style=footer_style() on:click=|e| e.stop_propagation()>
            <OwnershipLabel owned=state.owned_ores />
            <div style="display: flex; gap: 1rem; align-items: center; justify-content: space-between;">
                <InventorySlots owned=state.owned_ores />
                <DropButton owned=state.owned_ores on_drop=handle_drop />
                <DroppedStats count=state.dropped_count />
            </div>
        </footer>
    }
}

fn footer_style() -> String {
    format!(
        "background: {}; padding: 1.5rem 2rem; border-top: 2px solid {};",
        DEEP_SLATE, PHOSPHOR_GREEN
    )
}

#[component]
fn OwnershipLabel(owned: RwSignal<VecDeque<OreType>>) -> impl IntoView {
    let style = format!(
        "color: {}; font-size: 0.75rem; text-transform: uppercase; \
         letter-spacing: 0.1em; margin-bottom: 0.75rem; \
         display: flex; align-items: center; gap: 0.5rem;",
        PHOSPHOR_GREEN
    );

    view! {
        <div style=style>
            <span>"OWNED<'a>"</span>
            <span style=format!("color: {};", SLATE_DIM)>
                {move || format!("({}/{})", owned.get().len(), MAX_OWNED)}
            </span>
        </div>
    }
}

#[component]
fn InventorySlots(owned: RwSignal<VecDeque<OreType>>) -> impl IntoView {
    view! {
        <div style="display: flex; gap: 0.5rem;">
            {move || render_slots(owned.get())}
        </div>
    }
}

fn render_slots(ores: VecDeque<OreType>) -> impl IntoView {
    (0..MAX_OWNED).map(|i| {
        let ore = ores.get(i).copied();
        slot_div(ore)
    }).collect_view()
}

fn slot_div(ore: Option<OreType>) -> impl IntoView {
    let (bg, content) = match ore {
        Some(o) => (o.color(), o.symbol().to_string()),
        None => (DEEP_SLATE, "·".to_string()),
    };
    let style = slot_style(ore.is_some(), bg);
    view! { <div style=style>{content}</div> }
}

fn slot_style(has_ore: bool, border_color: &str) -> String {
    let bg = if has_ore { DEEP_SLATE } else { FOUNDATION_NAVY };
    format!(
        "width: 60px; height: 60px; background: {}; border: 2px solid {}; \
         border-radius: 8px; display: flex; align-items: center; \
         justify-content: center; font-size: 1.5rem;",
        bg, border_color
    )
}

#[component]
fn DropButton<F>(owned: RwSignal<VecDeque<OreType>>, on_drop: F) -> impl IntoView
where
    F: Fn(MouseEvent) + 'static,
{
    let style = move || drop_button_style(owned.get().is_empty());

    view! {
        <button
            style=style
            on:click=on_drop
            disabled=move || owned.get().is_empty()
        >
            "DROP(&mut self)"
        </button>
    }
}

fn drop_button_style(is_empty: bool) -> String {
    let (bg, color, border, cursor) = if is_empty {
        (DEEP_SLATE, SLATE_DIM, SLATE_DIM, "not-allowed")
    } else {
        (FOUNDATION_NAVY, PHOSPHOR_GREEN, PHOSPHOR_GREEN, "pointer")
    };
    format!(
        "padding: 1rem 2rem; background: {}; color: {}; \
         border: 2px solid {}; border-radius: 8px; \
         font-family: inherit; font-size: 1rem; \
         cursor: {}; transition: all 0.2s;",
        bg, color, border, cursor
    )
}

#[component]
fn DroppedStats(count: RwSignal<usize>) -> impl IntoView {
    view! {
        <div style=format!("color: {}; text-align: right;", SLATE_DIM)>
            <div style="font-size: 0.75rem;">"Dropped"</div>
            <div style=format!("color: {}; font-size: 1.25rem;", PHOSPHOR_GREEN)>
                {move || count.get()}
            </div>
        </div>
    }
}
