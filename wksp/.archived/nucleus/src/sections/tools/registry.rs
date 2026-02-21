//! Registry HUD — monitor Kellogg crate registry and lifecycle events

use leptos::prelude::*;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegistryEvent {
    pub id: String,
    pub crate_name: String,
    pub version: String,
    pub action: String, // "published", "yanked", "indexed"
    pub timestamp: String,
}

#[server(GetRegistryEvents, "/api")]
pub async fn get_registry_events_action() -> Result<Vec<RegistryEvent>, ServerFnError> {
    // Simulated events from Kellogg
    Ok(vec![
        RegistryEvent {
            id: "e1".into(),
            crate_name: "wksp-api-client".into(),
            version: "0.2.1".into(),
            action: "published".into(),
            timestamp: "2026-02-15 10:30".into(),
        },
        RegistryEvent {
            id: "e2".into(),
            crate_name: "nexcore-brain".into(),
            version: "1.4.0".into(),
            action: "indexed".into(),
            timestamp: "2026-02-15 09:15".into(),
        },
        RegistryEvent {
            id: "e3".into(),
            crate_name: "stem-bio".into(),
            version: "0.1.5".into(),
            action: "published".into(),
            timestamp: "2026-02-14 18:45".into(),
        },
    ])
}

#[component]
pub fn RegistryHudPage() -> impl IntoView {
    let events = Resource::new(|| (), |_| get_registry_events_action());

    view! {
        <div class="mx-auto max-w-6xl px-4 py-12">
            <header class="mb-12">
                <div class="flex items-center gap-4 mb-4">
                    <a href="/tools" class="text-slate-500 hover:text-white transition-colors">{"\u{2190}"}</a>
                    <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Registry HUD"</h1>
                </div>
                <p class="text-slate-400">"Monitor the Kellogg crate registry and artifact lifecycle events."</p>
            </header>

            <div class="grid gap-8 lg:grid-cols-3">
                /* ---- Registry Health ---- */
                <div class="lg:col-span-1 space-y-6">
                    <div class="rounded-2xl border border-slate-800 bg-slate-900/50 p-6">
                        <h3 class="text-xs font-bold text-slate-500 uppercase tracking-widest mb-6 font-mono">"// REGISTRY STATUS"</h3>
                        <div class="space-y-4">
                            <StatusItem label="Kellogg-1" status="Online" color="bg-emerald-500" />
                            <StatusItem label="Indexer" status="Active" color="bg-emerald-500" />
                            <StatusItem label="Storage" status="92% Free" color="bg-cyan-500" />
                        </div>
                    </div>

                    <div class="rounded-2xl border border-slate-800 bg-slate-900/30 p-6">
                        <h3 class="text-xs font-bold text-slate-500 uppercase tracking-widest mb-4 font-mono">"Distribution"</h3>
                        <div class="h-40 flex items-end gap-2">
                            {(0..10).map(|i| {
                                let h = (i * 10 + 20) % 100;
                                view! { <div class="flex-1 bg-slate-800 rounded-t-sm" style=format!("height: {h}%")></div> }
                            }).collect_view()}
                        </div>
                        <p class="mt-4 text-[10px] text-slate-600 font-mono text-center uppercase tracking-widest">"Crate Publishing Velocity"</p>
                    </div>
                </div>

                /* ---- Event Stream ---- */
                <div class="lg:col-span-2">
                    <div class="rounded-2xl border border-slate-800 bg-slate-950/50 overflow-hidden flex flex-col h-[600px]">
                        <div class="bg-slate-900/80 px-6 py-3 border-b border-slate-800 flex justify-between items-center">
                            <span class="text-[10px] font-bold text-slate-500 uppercase tracking-widest font-mono">"Artifact Lifecycle Stream"</span>
                            <span class="h-2 w-2 rounded-full bg-emerald-500 animate-pulse"></span>
                        </div>
                        <div class="flex-1 overflow-y-auto p-4 space-y-2">
                            <Suspense fallback=|| view! { <div class="animate-pulse space-y-2">{(0..5).map(|_| view! { <div class="h-16 bg-slate-900 rounded-xl"></div> }).collect_view()}</div> }>
                                {move || events.get().map(|result| match result {
                                    Ok(list) => view! {
                                        <div class="space-y-2">
                                            {list.into_iter().map(|evt| view! { <EventRow event=evt /> }).collect_view()}
                                        </div>
                                    }.into_any(),
                                    Err(_) => view! { <p class="p-8 text-center text-red-500 font-mono text-xs">"Stream Disconnected"</p> }.into_any(),
                                })}
                            </Suspense>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn StatusItem(label: &'static str, status: &'static str, color: &'static str) -> impl IntoView {
    view! {
        <div class="flex justify-between items-center">
            <span class="text-xs text-slate-400 font-mono">{label}</span>
            <div class="flex items-center gap-2">
                <span class=format!("h-1.5 w-1.5 rounded-full {color}")></span>
                <span class="text-xs text-white font-bold">{status}</span>
            </div>
        </div>
    }
}

#[component]
fn EventRow(event: RegistryEvent) -> impl IntoView {
    let action_cls = match event.action.as_str() {
        "published" => "text-emerald-400 border-emerald-500/30 bg-emerald-500/5",
        "indexed" => "text-cyan-400 border-cyan-500/30 bg-cyan-500/5",
        "yanked" => "text-red-400 border-red-500/30 bg-red-500/5",
        _ => "text-slate-500 border-slate-800 bg-slate-900",
    };

    view! {
        <div class="p-4 rounded-xl border border-slate-800 bg-slate-900/30 hover:border-slate-700 transition-all flex items-center justify-between group">
            <div class="flex items-center gap-4">
                <div class="h-8 w-8 rounded-lg bg-slate-950 border border-slate-800 flex items-center justify-center text-[10px] font-black text-slate-500 font-mono group-hover:text-cyan-400 transition-colors">
                    "ς"
                </div>
                <div>
                    <div class="flex items-center gap-2">
                        <span class="text-sm font-bold text-white tracking-tight">{event.crate_name}</span>
                        <span class="text-[10px] text-slate-500 font-mono">"v"{event.version}</span>
                    </div>
                    <p class="text-[9px] text-slate-600 font-mono mt-0.5">{event.timestamp}</p>
                </div>
            </div>
            <span class=format!("px-2 py-0.5 rounded border text-[9px] font-bold font-mono uppercase tracking-widest {}", action_cls)>
                {event.action}
            </span>
        </div>
    }
}
