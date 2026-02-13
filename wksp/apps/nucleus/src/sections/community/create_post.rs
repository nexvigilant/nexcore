//! Create post page

use leptos::prelude::*;
use crate::api_client::{Post, CreatePostRequest};
use crate::auth::use_auth;

/// Server function to create a post
#[server(CreatePostAction, "/api")]
pub async fn create_post_action(content: String, author: String, role: String) -> Result<Post, ServerFnError> {
    use crate::api_client::server::ApiClient;

    let api_url = std::env::var("NEXCORE_API_URL").unwrap_or_else(|_| "http://localhost:3030".to_string());
    let api_key = std::env::var("NEXCORE_API_KEY").ok();
    let client = ApiClient::new(api_url, api_key);

    let req = CreatePostRequest {
        author,
        role,
        content,
    };

    client.community_create_post(&req).await
        .map_err(ServerFnError::new)
}

#[component]
pub fn CreatePostPage() -> impl IntoView {
    let auth = use_auth();
    let content = RwSignal::new(String::new());
    let create_action = ServerAction::<CreatePostAction>::new();
    let result = create_action.value();

    let user_name = move || auth.user.get().map(|u| u.display_name.unwrap_or(u.email)).unwrap_or_else(|| "Anonymous".to_string());
    let user_role = move || auth.user.get().map(|u| format!("{:?}", u.role)).unwrap_or_else(|| "Guest".to_string());

    // Navigate back to feed on success
    Effect::new(move |_| {
        if result.get().is_some_and(|r| r.is_ok()) {
            let nav = leptos_router::hooks::use_navigate();
            nav("/community", Default::default());
        }
    });

    view! {
        <div class="mx-auto max-w-2xl px-4 py-8">
            <header class="mb-10">
                <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"New Publication"</h1>
                <p class="mt-2 text-slate-400">"Share insights, signals, or questions with the network"</p>
            </header>

            <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                <div class="flex items-center gap-3 mb-6">
                    <div class="h-8 w-8 rounded-full bg-slate-800 border border-slate-700 flex items-center justify-center text-xs font-bold text-cyan-400 font-mono">
                        {move || if let Some(u) = auth.user.get() {
                            u.email.chars().next().unwrap_or('?').to_string().to_uppercase()
                        } else {
                            "A".to_string()
                        }}
                    </div>
                    <div>
                        <p class="text-sm font-bold text-white">{user_name()}</p>
                        <p class="text-[10px] text-slate-500 font-bold uppercase tracking-widest">{user_role()}</p>
                    </div>
                </div>

                <textarea
                    rows="6"
                    placeholder="What's the signal?"
                    prop:value=move || content.get()
                    on:input=move |ev| content.set(event_target_value(&ev))
                    class="w-full rounded-lg border border-slate-700 bg-slate-950 p-4 text-sm text-white placeholder:text-slate-600 focus:border-cyan-500 focus:outline-none transition-all font-mono"
                ></textarea>

                <div class="mt-6 flex justify-between items-center">
                    <p class="text-[10px] text-slate-600 font-mono italic">"Supports markdown and primitive symbols"</p>
                    <div class="flex gap-3">
                        <a href="/community" class="rounded-lg px-4 py-2 text-sm font-bold text-slate-400 hover:text-white transition-colors">"CANCEL"</a>
                        <button
                            on:click=move |_| {
                                create_action.dispatch(CreatePostAction { 
                                    content: content.get(),
                                    author: user_name(),
                                    role: user_role(),
                                });
                            }
                            disabled=create_action.pending()
                            class="rounded-lg bg-cyan-600 px-6 py-2 text-sm font-bold text-white hover:bg-cyan-500 transition-all shadow-lg shadow-cyan-900/20 disabled:opacity-50"
                        >
                            {move || if create_action.pending().get() { "PUBLISHING..." } else { "PUBLISH POST" }}
                        </button>
                    </div>
                </div>

                {move || result.get().and_then(|res| res.err()).map(|e| {
                    view! { <p class="mt-4 text-red-400 text-xs font-mono">{e.to_string()}</p> }
                })}
            </div>
        </div>
    }
}