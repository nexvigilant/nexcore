/// Brain sessions & artifacts browser — pi Persistence + lambda Location
/// Browse sessions, view artifacts
use leptos::prelude::*;

use crate::api::brain;

#[component]
pub fn BrainPage() -> impl IntoView {
    let sessions = LocalResource::new(|| brain::list_sessions());
    let (selected_session, set_selected_session) = signal(None::<String>);
    let (artifacts, set_artifacts) = signal(Vec::<brain::Artifact>::new());
    let (artifact_loading, set_artifact_loading) = signal(false);
    let (selected_artifact, set_selected_artifact) = signal(None::<brain::Artifact>);

    let load_artifacts = move |session_id: String| {
        set_selected_session.set(Some(session_id.clone()));
        set_artifact_loading.set(true);
        set_selected_artifact.set(None);

        wasm_bindgen_futures::spawn_local(async move {
            match brain::get_session_artifacts(&session_id).await {
                Ok(arts) => set_artifacts.set(arts),
                Err(e) => log::error!("Failed to load artifacts: {}", e.message),
            }
            set_artifact_loading.set(false);
        });
    };

    view! {
        <div class="page">
            <header class="page-header">
                <h1 class="page-title">"Brain"</h1>
                <p class="page-subtitle">"Sessions & Artifacts"</p>
            </header>

            // Session list
            <Suspense fallback=move || view! { <div class="loading">"Loading sessions..."</div> }>
                {move || {
                    sessions.read().as_ref().map(|result| {
                        match result {
                            Ok(list) => view! {
                                <div class="session-list">
                                    {list.iter().map(|s| {
                                        let sid = s.id.clone();
                                        let sid2 = s.id.clone();
                                        let name = s.name.clone();
                                        let count = s.artifact_count;
                                        let is_selected = move || {
                                            selected_session.get().as_deref() == Some(&*sid2)
                                        };
                                        view! {
                                            <button
                                                class=move || if is_selected() { "session-card selected" } else { "session-card" }
                                                on:click=move |_| load_artifacts(sid.clone())
                                            >
                                                <div class="session-name">{name.clone()}</div>
                                                <div class="session-meta">
                                                    <span>{count}" artifacts"</span>
                                                </div>
                                            </button>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any(),
                            Err(e) => view! {
                                <div class="error-card">
                                    <div class="error-msg">{e.message.clone()}</div>
                                </div>
                            }.into_any(),
                        }
                    })
                }}
            </Suspense>

            // Artifact list (when session selected)
            {move || {
                if selected_session.get().is_some() {
                    if artifact_loading.get() {
                        Some(view! { <div class="loading">"Loading artifacts..."</div> }.into_any())
                    } else {
                        let arts = artifacts.get();
                        if arts.is_empty() {
                            Some(view! { <div class="empty-state"><p>"No artifacts in this session"</p></div> }.into_any())
                        } else {
                            Some(view! {
                                <div class="artifact-list">
                                    {arts.iter().map(|a| {
                                        let art = a.clone();
                                        let name = a.name.clone();
                                        let atype = a.artifact_type.clone();
                                        view! {
                                            <button
                                                class="artifact-card"
                                                on:click=move |_| set_selected_artifact.set(Some(art.clone()))
                                            >
                                                <div class="artifact-name">{name.clone()}</div>
                                                <div class="artifact-type">{atype.clone()}</div>
                                            </button>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any())
                        }
                    }
                } else {
                    None
                }
            }}

            // Artifact detail
            {move || {
                selected_artifact.get().map(|art| {
                    view! {
                        <div class="artifact-detail">
                            <div class="artifact-detail-header">
                                <h3>{art.name.clone()}</h3>
                                <span class="badge">{art.artifact_type.clone()}</span>
                                <span class="badge">"v"{art.version}</span>
                            </div>
                            <pre class="artifact-content">{art.content.clone()}</pre>
                        </div>
                    }
                })
            }}
        </div>
    }
}
