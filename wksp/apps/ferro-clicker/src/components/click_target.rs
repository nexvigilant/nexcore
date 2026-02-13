//! ClickTarget component - Tier: T2-C

use leptos::prelude::*;

const BTN_CLASS: &str = "w-48 h-48 text-8xl bg-orange-600 rounded-full cursor-pointer";

#[component]
pub fn ClickTarget<F: Fn() + 'static>(on_click: F) -> impl IntoView {
    view! { <button on:click=move |_| on_click() id="clicker" class=BTN_CLASS>{"\u{1F980}"}</button> }
}
