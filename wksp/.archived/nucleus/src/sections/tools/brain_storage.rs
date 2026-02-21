//! Brain Artifact Manager — browse and manage generated engineering artifacts

use crate::api_client::{BrainArtifact, BrainSession};
use leptos::prelude::*;

#[server(GetArtifacts, "/api")]
pub async fn get_artifacts_action(session_id: String) -> Result<Vec<BrainArtifact>, ServerFnError> {
    use crate::api_client::server::ApiClient;
    let api_url =
        crate::runtime_config::nexcore_api_url();
    let api_key = crate::runtime_config::nexcore_api_key();
    let client = ApiClient::new(api_url, api_key);

    let session = client
        .brain_session_load(&session_id)
        .await
        .map_err(ServerFnError::new)?;
    // Return session artifacts as a single-element vec for now
    Ok(vec![BrainArtifact {
        id: session.id,
        artifact_type: "session".to_string(),
        content: session.name,
        version: 0,
    }])
}

#[server(GetSessions, "/api")]
pub async fn get_sessions_action() -> Result<Vec<BrainSession>, ServerFnError> {
    use crate::api_client::server::ApiClient;
    let api_url =
        crate::runtime_config::nexcore_api_url();
    let api_key = crate::runtime_config::nexcore_api_key();
    let client = ApiClient::new(api_url, api_key);

    client.brain_sessions().await.map_err(ServerFnError::new)
}

#[component]
pub fn ArtifactManagerPage() -> impl IntoView {
    let sessions = Resource::new(|| (), |_| get_sessions_action());
    let selected_session_id = RwSignal::new(Option::<String>::None);

    let artifacts = Resource::new(
        move || selected_session_id.get(),
        |id| async move {
            match id {
                Some(sid) => get_artifacts_action(sid).await,
                None => Ok(vec![]),
            }
        },
    );

    view! {
        <div class="mx-auto max-w-6xl px-4 py-12">
            <header class="mb-12 flex justify-between items-center">
                <div>
                    <div class="flex items-center gap-4 mb-4">
                        <a href="/tools" class="text-slate-500 hover:text-white transition-colors">{"\u{2190}"}</a>
                        <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Brain Storage"</h1>
                    </div>
                    <p class="text-slate-400">"Working memory and persistent artifacts from AI-assisted engineering sessions."</p>
                </div>
            </header>

            <div class="grid gap-8 lg:grid-cols-4">
                /* ---- Sessions List ---- */
                <aside class="lg:col-span-1 space-y-6">
                    <div class="rounded-2xl border border-slate-800 bg-slate-900/50 p-6 h-fit">
                        <h2 class="text-xs font-bold text-slate-500 uppercase tracking-widest mb-6 font-mono">"// SESSIONS"</h2>

                        <Suspense fallback=|| view! { <div class="space-y-2 animate-pulse">{(0..5).map(|_| view! { <div class="h-12 bg-slate-800 rounded-xl"></div> }).collect_view()}</div> }>
                            {move || sessions.get().map(|result| match result {
                                Ok(list) => view! {
                                    <div class="space-y-2">
                                        {list.into_iter().map(|s| {
                                            let id = s.id.clone();
                                            let is_active = move || selected_session_id.get().as_deref() == Some(&id);
                                            let sid = s.id.clone();
                                            view! {
                                                <button
                                                    on:click=move |_| selected_session_id.set(Some(sid.clone()))
                                                    class=move || {
                                                        let base = "w-full text-left p-4 rounded-xl border transition-all group relative overflow-hidden";
                                                        if is_active() {
                                                            format!("{base} bg-amber-500/10 border-amber-500/30 text-amber-400 shadow-[0_0_15px_-5px_rgba(245,158,11,0.2)]")
                                                        } else {
                                                            format!("{base} bg-slate-950 border-slate-800 hover:border-slate-700 text-slate-400")
                                                        }
                                                    }
                                                >
                                                    <p class="text-xs font-bold truncate">{s.name}</p>
                                                    <p class="text-[9px] font-mono opacity-50 mt-1">{s.created_at}</p>
                                                </button>
                                            }
                                        }).collect_view()}
                                    </div>
                                }.into_any(),
                                Err(_) => view! { <p class="text-red-500 text-xs font-mono">"Failed to load sessions"</p> }.into_any(),
                            })}
                        </Suspense>
                    </div>
                </aside>

                /* ---- Artifacts Grid ---- */
                <div class="lg:col-span-3">
                    <Suspense fallback=|| view! { <div class="grid gap-4 md:grid-cols-2">{(0..4).map(|_| view! { <div class="h-48 bg-slate-900 rounded-2xl animate-pulse"></div> }).collect_view()}</div> }>
                        {move || artifacts.get().map(|result| match result {
                            Ok(list) => {
                                if list.is_empty() {
                                    view! {
                                        <div class="h-96 rounded-3xl border border-dashed border-slate-800 flex flex-col items-center justify-center text-center p-12">
                                            <span class="text-4xl mb-4">"🧠"</span>
                                            <p class="text-slate-500 font-mono text-sm uppercase tracking-widest">"Select a session to view artifacts"</p>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="grid gap-4 md:grid-cols-2">
                                            {list.into_iter().map(|art| view! { <ArtifactCard artifact=art /> }).collect_view()}
                                        </div>
                                    }.into_any()
                                }
                            }
                            Err(e) => view! { <div class="p-8 bg-red-500/10 border border-red-500/20 text-red-400 font-mono">{e.to_string()}</div> }.into_any(),
                        })}
                    </Suspense>
                </div>
            </div>
        </div>
    }
}

#[component]
fn ArtifactCard(artifact: BrainArtifact) -> impl IntoView {
    view! {
        <div class="glass-panel p-6 rounded-2xl border border-slate-800 hover:border-cyan-500/30 transition-all group flex flex-col">
            <div class="flex justify-between items-start mb-4">
                <span class="px-2 py-0.5 rounded bg-slate-800 text-[9px] font-black font-mono text-cyan-400 uppercase border border-cyan-500/20">
                    {artifact.artifact_type}
                </span>
                <span class="text-[9px] font-mono text-slate-600">"v" {artifact.version}</span>
            </div>

            <h3 class="text-sm font-bold text-white mb-4 font-mono truncate">{artifact.id}</h3>

            <div class="bg-slate-950 rounded-xl p-4 flex-1 overflow-hidden relative">
                <pre class="text-[10px] text-slate-500 font-mono line-clamp-6">{artifact.content}</pre>
                <div class="absolute inset-x-0 bottom-0 h-12 bg-gradient-to-t from-slate-950 to-transparent"></div>
            </div>

            <button class="mt-4 w-full py-2 rounded-lg bg-slate-800 text-[10px] font-bold text-slate-400 uppercase tracking-widest hover:text-white transition-all">
                "View full artifact"
            </button>
        </div>
    }
}
