//! Reusable section hub component — config-driven card grid pattern
//!
//! Used by Academy, Careers, Community hubs to render a consistent
//! grid of feature cards linking to sub-pages.

use leptos::prelude::*;

/// A single hub card configuration
pub struct HubCard {
    pub title: &'static str,
    pub description: &'static str,
    pub href: &'static str,
    pub icon: &'static str,
    pub color: &'static str,
}

/// Config-driven section hub with title, description, and card grid
#[component]
pub fn SectionHub(
    title: &'static str,
    description: &'static str,
    cards: Vec<HubCard>,
) -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white">{title}</h1>
            <p class="mt-2 text-slate-400">{description}</p>

            <div class="mt-8 grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                {cards.into_iter().map(|card| {
                    let border_class = format!(
                        "rounded-xl border border-slate-800 bg-slate-900/50 p-6 hover:border-{}-500/30 transition-colors",
                        card.color
                    );
                    let icon_class = format!("text-2xl text-{}-400", card.color);

                    view! {
                        <a href=card.href class=border_class>
                            <div class=icon_class>{card.icon}</div>
                            <h3 class="mt-3 font-semibold text-white">{card.title}</h3>
                            <p class="mt-1 text-sm text-slate-400">{card.description}</p>
                        </a>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}
