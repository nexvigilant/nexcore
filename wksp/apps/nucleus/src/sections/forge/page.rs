//! Forge page — Primitive Depths roguelike game
//!
//! Collect the 16 Lex Primitiva symbols, battle antipatterns, and forge Rust code.
//! Grounded to σ(Sequence) + ς(State) + →(Causality)

use super::game::ForgeGame;
use leptos::prelude::*;

#[component]
pub fn ForgePage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-7xl px-4 py-8 md:py-12 min-h-[calc(100vh-4rem)] flex flex-col">
            <header class="mb-8">
                <div class="flex items-center gap-4 mb-2">
                    <div class="p-3 rounded-xl bg-amber-500/10 border border-amber-500/20">
                        <span class="text-3xl text-amber-400">"⚒"</span>
                    </div>
                    <div>
                        <h1 class="text-4xl md:text-5xl font-extrabold font-mono text-amber-500 uppercase tracking-tight">
                            "Primitive Depths"
                        </h1>
                        <p class="text-slate-400 font-medium">
                            "Collect the 16 Lex Primitiva \u{2014} battle antipatterns \u{2014} forge Rust code"
                        </p>
                    </div>
                </div>
                <div class="flex items-center gap-4 mt-3 text-xs font-mono text-slate-600">
                    <span>"v0.5.0-ALPHA"</span>
                    <span class="flex items-center gap-1">
                        <span class="h-2 w-2 rounded-full bg-emerald-500 animate-pulse"></span>
                        "ONLINE"
                    </span>
                    <span>"Use arrow buttons or tap adjacent cells to move"</span>
                </div>
            </header>

            <ForgeGame/>
        </div>
    }
}

#[component]
fn LexSymbol(symbol: &'static str, label: &'static str, active: bool) -> impl IntoView {
    let (bg, text, border) = if active {
        ("bg-amber-500/20", "text-amber-400", "border-amber-500/40")
    } else {
        ("bg-slate-950/50", "text-slate-700", "border-slate-800")
    };

    view! {
        <div class=format!("aspect-square rounded-lg border flex items-center justify-center text-lg font-serif transition-all hover:scale-105 cursor-help {bg} {text} {border}")
             title=label>
            {symbol}
        </div>
    }
}

#[component]
fn ArtifactItem(name: &'static str, type_: &'static str, color: &'static str) -> impl IntoView {
    view! {
        <div class="flex items-center gap-3">
            <div class="h-2 w-2 rounded-full bg-slate-700"></div>
            <div>
                <p class="text-xs font-bold text-slate-300">{name}</p>
                <p class=format!("text-[10px] font-mono uppercase tracking-tighter {color}")>{type_}</p>
            </div>
        </div>
    }
}
