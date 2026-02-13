use crate::components::card::{Card, CardLoading};
use leptos::prelude::*;

/// Brain sessions & artifacts browser
/// Tier: T3 (domain — Brain working memory)
#[component]
pub fn BrainPage() -> impl IntoView {
    let sessions = RwSignal::new(Vec::<(String, String)>::new());
    let selected_session = RwSignal::new(Option::<String>::None);
    let artifacts = RwSignal::new(Vec::<(String, String, String)>::new());
    let loading = RwSignal::new(false);

    view! {
        <div class="page brain">
            <h1 class="page-title">"Brain Sessions"</h1>

            <Card title="Sessions">
                <button class="btn-secondary" style="margin-bottom: 1rem"
                    on:click=move |_| { loading.set(true); }
                >"Load Sessions"</button>
                <div class="session-list">
                    {move || {
                        let sess = sessions.get();
                        if sess.is_empty() {
                            view! { <p class="card-hint">"No sessions loaded. Tap Load Sessions."</p> }.into_any()
                        } else {
                            view! {
                                <ul class="item-list">
                                    {sess.into_iter().map(|(id, name)| {
                                        let id_clone = id.clone();
                                        view! {
                                            <li class="item-row" on:click=move |_| {
                                                selected_session.set(Some(id_clone.clone()));
                                            }>
                                                <span class="item-name">{name}</span>
                                                <span class="item-id">{id}</span>
                                            </li>
                                        }
                                    }).collect::<Vec<_>>()}
                                </ul>
                            }.into_any()
                        }
                    }}
                </div>
            </Card>

            <Show when=move || loading.get()>
                <CardLoading/>
            </Show>

            <Show when=move || selected_session.get().is_some()>
                <Card title="Artifacts">
                    {move || {
                        let arts = artifacts.get();
                        if arts.is_empty() {
                            view! { <p class="card-hint">"No artifacts in this session."</p> }.into_any()
                        } else {
                            view! {
                                <ul class="item-list">
                                    {arts.into_iter().map(|(id, atype, content)| {
                                        view! {
                                            <li class="item-row">
                                                <span class="item-badge">{atype}</span>
                                                <span class="item-name">{id}</span>
                                                <p class="item-preview">{
                                                    if content.len() > 100 {
                                                        format!("{}...", &content[..100])
                                                    } else {
                                                        content
                                                    }
                                                }</p>
                                            </li>
                                        }
                                    }).collect::<Vec<_>>()}
                                </ul>
                            }.into_any()
                        }
                    }}
                </Card>
            </Show>
        </div>
    }
}
