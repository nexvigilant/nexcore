//! Brain sessions & artifacts browser

use leptos::prelude::*;

#[component]
pub fn BrainPage() -> impl IntoView {
    let sessions = RwSignal::new(Vec::<(String, String)>::new());
    let selected_session = RwSignal::new(Option::<String>::None);
    let artifacts = RwSignal::new(Vec::<(String, String, String)>::new());
    let loading = RwSignal::new(false);

    view! {
        <div class="mx-auto max-w-4xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white">"Brain Sessions"</h1>
            <p class="mt-2 text-slate-400">"Working memory: sessions, artifacts, and implicit learning"</p>

            // Sessions
            <div class="mt-8 rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                <div class="flex items-center justify-between">
                    <h2 class="text-lg font-semibold text-white">"Sessions"</h2>
                    <button
                        class="rounded-lg bg-amber-600 px-4 py-2 text-sm font-medium text-white hover:bg-amber-500 transition-colors"
                        on:click=move |_| loading.set(true)
                    >"Load Sessions"</button>
                </div>
                <div class="mt-4">
                    {move || {
                        let sess = sessions.get();
                        if sess.is_empty() {
                            view! {
                                <p class="text-sm text-slate-500">"No sessions loaded. Tap Load Sessions to fetch from nexcore API."</p>
                            }.into_any()
                        } else {
                            view! {
                                <ul class="space-y-2">
                                    {sess.into_iter().map(|(id, name)| {
                                        let id_clone = id.clone();
                                        let is_selected = {
                                            let id_for_check = id.clone();
                                            move || selected_session.get().as_deref() == Some(&id_for_check)
                                        };
                                        view! {
                                            <li
                                                class=move || {
                                                    if is_selected() {
                                                        "cursor-pointer rounded-lg border border-amber-500/50 bg-amber-500/10 px-4 py-3"
                                                    } else {
                                                        "cursor-pointer rounded-lg border border-slate-700 bg-slate-800/50 px-4 py-3 hover:border-slate-600"
                                                    }
                                                }
                                                on:click=move |_| selected_session.set(Some(id_clone.clone()))
                                            >
                                                <p class="font-medium text-white">{name}</p>
                                                <p class="mt-0.5 text-xs text-slate-500">{id}</p>
                                            </li>
                                        }
                                    }).collect::<Vec<_>>()}
                                </ul>
                            }.into_any()
                        }
                    }}
                </div>
            </div>

            // Loading indicator
            <Show when=move || loading.get()>
                <div class="mt-4 flex items-center gap-2 text-sm text-slate-400">
                    <div class="h-4 w-4 animate-spin rounded-full border-2 border-amber-500 border-t-transparent"></div>
                    "Loading sessions..."
                </div>
            </Show>

            // Artifacts
            <Show when=move || selected_session.get().is_some()>
                <div class="mt-6 rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                    <h2 class="text-lg font-semibold text-white">"Artifacts"</h2>
                    <div class="mt-4">
                        {move || {
                            let arts = artifacts.get();
                            if arts.is_empty() {
                                view! { <p class="text-sm text-slate-500">"No artifacts in this session."</p> }.into_any()
                            } else {
                                view! {
                                    <ul class="space-y-3">
                                        {arts.into_iter().map(|(id, atype, content)| {
                                            let preview = if content.len() > 200 {
                                                format!("{}...", &content[..200])
                                            } else {
                                                content
                                            };
                                            view! {
                                                <li class="rounded-lg border border-slate-700 bg-slate-800/50 p-4">
                                                    <div class="flex items-center gap-2">
                                                        <span class="rounded bg-cyan-500/10 px-2 py-0.5 text-xs font-medium text-cyan-400">{atype}</span>
                                                        <span class="text-sm font-medium text-white">{id}</span>
                                                    </div>
                                                    <p class="mt-2 text-sm text-slate-400">{preview}</p>
                                                </li>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </ul>
                                }.into_any()
                            }
                        }}
                    </div>
                </div>
            </Show>
        </div>
    }
}
