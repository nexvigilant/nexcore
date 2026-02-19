//! Forge Game Engine — Roguelike exploration and code forging
//!
//! Grounded to σ Sequence (progression) and ς State (game state).

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEntity {
    pub id: String,
    pub x: i32,
    pub y: i32,
    pub kind: EntityKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EntityKind {
    Player,
    Primitive(String),
    Antipattern(String),
    Exit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeState {
    pub player_pos: (i32, i32),
    pub entities: Vec<GameEntity>,
    pub inventory: Vec<String>,
    pub health: i32,
    pub floor: u32,
    pub logs: Vec<String>,
}

impl Default for ForgeState {
    fn default() -> Self {
        Self {
            player_pos: (2, 2),
            entities: vec![
                GameEntity {
                    id: "p1".into(),
                    x: 1,
                    y: 1,
                    kind: EntityKind::Primitive("σ".into()),
                },
                GameEntity {
                    id: "p2".into(),
                    x: 3,
                    y: 4,
                    kind: EntityKind::Primitive("μ".into()),
                },
                GameEntity {
                    id: "a1".into(),
                    x: 5,
                    y: 2,
                    kind: EntityKind::Antipattern("Ghost Pointer".into()),
                },
                GameEntity {
                    id: "exit".into(),
                    x: 7,
                    y: 7,
                    kind: EntityKind::Exit,
                },
            ],
            inventory: vec![],
            health: 100,
            floor: 1,
            logs: vec!["Welcome to the Primitive Depths.".into()],
        }
    }
}

#[component]
pub fn ForgeGame() -> impl IntoView {
    let state = RwSignal::new(ForgeState::default());

    let move_player = move |dx: i32, dy: i32| {
        state.update(|s| {
            let next_x = (s.player_pos.0 + dx).clamp(0, 7);
            let next_y = (s.player_pos.1 + dy).clamp(0, 7);
            s.player_pos = (next_x, next_y);

            /* Collision detection */
            let mut collected = None;
            s.entities.retain(|e| {
                if e.x == next_x && e.y == next_y {
                    match &e.kind {
                        EntityKind::Primitive(sym) => {
                            collected = Some(sym.clone());
                            false
                        }
                        EntityKind::Antipattern(name) => {
                            s.health -= 15;
                            s.logs.insert(0, format!("COLLISION: Damaged by {name}."));
                            true
                        }
                        EntityKind::Exit => {
                            s.floor += 1;
                            s.logs
                                .insert(0, format!("DESCENDING: Entering Floor {}.", s.floor));
                            // In a real game we'd regenerate the floor here
                            true
                        }
                        _ => true,
                    }
                } else {
                    true
                }
            });

            if let Some(sym) = collected {
                s.inventory.push(sym.clone());
                s.logs
                    .insert(0, format!("FOUND: Collected Lex Symbol {sym}."));
            }
        });
    };

    view! {
        <div class="grid lg:grid-cols-4 gap-8">
            /* ---- Game Board ---- */
            <div class="lg:col-span-3">
                <div class="aspect-square max-w-2xl mx-auto rounded-3xl border border-slate-800 bg-slate-950 p-4 relative overflow-hidden shadow-2xl">
                    <div class="grid grid-cols-8 grid-rows-8 h-full w-full gap-1">
                        {move || (0..64).map(|i| {
                            let x = i % 8;
                            let y = i / 8;
                            let is_player = move || state.get().player_pos == (x, y);

                            let entity = move || state.get().entities.iter().find(|e| e.x == x && e.y == y).cloned();

                            view! {
                                <div class="bg-slate-900/30 rounded-lg border border-slate-800/50 flex items-center justify-center relative">
                                    {move || if is_player() {
                                        view! { <div class="text-2xl animate-pulse">"🕹\u{fe0f}"</div> }.into_any()
                                    } else if let Some(e) = entity() {
                                        match e.kind {
                                            EntityKind::Primitive(sym) => view! { <div class="text-amber-500 font-serif text-xl"> {sym} </div> }.into_any(),
                                            EntityKind::Antipattern(_) => view! { <div class="text-red-500 text-xl"> "👻" </div> }.into_any(),
                                            EntityKind::Exit => view! { <div class="text-cyan-500 text-xl"> "⚙\u{fe0f}" </div> }.into_any(),
                                            _ => view! { <div/> }.into_any(),
                                        }
                                    } else {
                                        view! { <div/> }.into_any()
                                    }}
                                </div>
                            }
                        }).collect_view()}
                    </div>

                    /* Mobile Controls Overlay */
                    <div class="absolute bottom-8 right-8 flex flex-col gap-2 scale-125 origin-bottom-right">
                        <button on:click=move |_| move_player(0, -1) class="h-10 w-10 bg-slate-800 rounded-lg flex items-center justify-center border border-slate-700 hover:bg-slate-700">"↑"</button>
                        <div class="flex gap-2">
                            <button on:click=move |_| move_player(-1, 0) class="h-10 w-10 bg-slate-800 rounded-lg flex items-center justify-center border border-slate-700 hover:bg-slate-700">"←"</button>
                            <button on:click=move |_| move_player(0, 1) class="h-10 w-10 bg-slate-800 rounded-lg flex items-center justify-center border border-slate-700 hover:bg-slate-700">"↓"</button>
                            <button on:click=move |_| move_player(1, 0) class="h-10 w-10 bg-slate-800 rounded-lg flex items-center justify-center border border-slate-700 hover:bg-slate-700">"→"</button>
                        </div>
                    </div>
                </div>
            </div>

            /* ---- Sidebar: Telemetry & Inventory ---- */
            <aside class="space-y-6">
                <div class="rounded-2xl border border-slate-800 bg-slate-900/50 p-6">
                    <h3 class="text-xs font-bold text-slate-500 uppercase tracking-widest mb-6">"// TELEMETRY"</h3>
                    <div class="space-y-4">
                        <TelemetryRow label="Health" value=move || format!("{}%", state.get().health) color="text-red-400" />
                        <TelemetryRow label="Floor" value=move || state.get().floor.to_string() color="text-cyan-400" />
                        <TelemetryRow label="Symbols" value=move || state.get().inventory.len().to_string() color="text-amber-400" />
                    </div>
                </div>

                <div class="rounded-2xl border border-slate-800 bg-slate-900/50 p-6">
                    <h3 class="text-xs font-bold text-slate-500 uppercase tracking-widest mb-6">"// COLLECTION"</h3>
                    <div class="flex flex-wrap gap-2">
                        {move || state.get().inventory.into_iter().map(|sym| view! {
                            <div class="h-8 w-8 rounded-lg bg-amber-500/10 border border-amber-500/30 flex items-center justify-center text-amber-400 font-serif">
                                {sym}
                            </div>
                        }).collect_view()}
                    </div>
                </div>

                <div class="rounded-2xl border border-slate-800 bg-slate-900/30 p-6 h-48 flex flex-col">
                    <h3 class="text-xs font-bold text-slate-500 uppercase tracking-widest mb-4">"// LOGS"</h3>
                    <div class="flex-1 overflow-y-auto font-mono text-[10px] space-y-2 text-slate-500">
                        {move || state.get().logs.into_iter().map(|log| view! {
                            <p class="border-l border-slate-800 pl-2">{log}</p>
                        }).collect_view()}
                    </div>
                </div>
            </aside>
        </div>
    }
}

#[component]
fn TelemetryRow<V, F>(label: &'static str, value: F, color: &'static str) -> impl IntoView
where
    V: IntoView + 'static,
    F: Fn() -> V + 'static,
{
    view! {
        <div class="flex justify-between items-center text-xs font-mono">
            <span class="text-slate-500 uppercase">{label}</span>
            <span class=format!("font-bold {color}")>{value()}</span>
        </div>
    }
}
