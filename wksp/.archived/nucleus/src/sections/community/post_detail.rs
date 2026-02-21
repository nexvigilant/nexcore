//! Post detail page — view full post and replies

use crate::api_client::Post;
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

/// Server function to fetch a specific post
#[server(GetPost, "/api")]
pub async fn get_post(post_id: String) -> Result<Post, ServerFnError> {
    use crate::api_client::server::ApiClient;

    let api_url =
        crate::runtime_config::nexcore_api_url();
    let api_key = crate::runtime_config::nexcore_api_key();
    let client = ApiClient::new(api_url, api_key);

    // For now, list and find (will add direct API endpoint later)
    let posts = client
        .community_list_posts()
        .await
        .map_err(ServerFnError::new)?;
    posts
        .into_iter()
        .find(|p| p.id == post_id)
        .ok_or_else(|| ServerFnError::new("Post not found"))
}

#[component]
pub fn PostDetailPage() -> impl IntoView {
    let params = use_params_map();
    let post_id = move || params.with(|p| p.get("postId").unwrap_or_default());

    let post = Resource::new(move || post_id(), move |id| get_post(id));

    view! {
        <div class="mx-auto max-w-3xl px-4 py-8">
            <Suspense fallback=|| view! { <div class="animate-pulse h-48 bg-slate-900/50 rounded-xl border border-slate-800"></div> }>
                {move || post.get().map(|result| match result {
                    Ok(p) => view! { <PostView post=p /> }.into_any(),
                    Err(e) => view! {
                        <div class="rounded-xl border border-red-500/20 bg-red-500/5 p-8 text-center">
                            <p class="text-red-400 font-mono">"Failed to load post: "{e.to_string()}</p>
                            <a href="/community" class="mt-4 inline-block text-sm text-slate-500 hover:text-white underline">"Back to feed"</a>
                        </div>
                    }.into_any()
                })}
            </Suspense>
        </div>
    }
}

#[component]
fn PostView(post: Post) -> impl IntoView {
    view! {
        <div class="space-y-6">
            /* Main Post */
            <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                <div class="flex items-center gap-3 mb-4">
                    <div class="h-10 w-10 rounded-full bg-cyan-500/10 flex items-center justify-center text-sm font-bold text-cyan-400 border border-cyan-500/20">
                        {post.author.chars().next().unwrap_or('?').to_uppercase().to_string()}
                    </div>
                    <div>
                        <p class="font-bold text-white leading-none">{post.author}</p>
                        <p class="text-xs text-slate-500 mt-1">{post.role}</p>
                    </div>
                </div>

                <div class="text-slate-200 whitespace-pre-wrap leading-relaxed text-lg">
                    {post.content}
                </div>

                <div class="mt-6 pt-4 border-t border-slate-800 flex items-center gap-6">
                    <button class="flex items-center gap-2 text-sm font-bold text-slate-500 hover:text-cyan-400 transition-colors">
                        <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z" />
                        </svg>
                        {post.likes}
                    </button>
                    <button class="flex items-center gap-2 text-sm font-bold text-slate-500 hover:text-cyan-400 transition-colors">
                        <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
                        </svg>
                        {post.replies}
                    </button>
                </div>
            </div>

            /* Replies placeholder */
            <div class="space-y-4">
                <h3 class="text-sm font-bold text-slate-500 uppercase tracking-widest px-2">"Replies"</h3>
                <div class="text-center py-12 border border-dashed border-slate-800 rounded-xl text-slate-600">
                    "No replies yet. Be the first to respond!"
                </div>
            </div>
        </div>
    }
}
