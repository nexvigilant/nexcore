//! Community feed — latest posts

use leptos::prelude::*;
use crate::api_client::Post;

/// Server function to list community posts
#[server(ListPosts, "/api")]
pub async fn list_posts_action() -> Result<Vec<Post>, ServerFnError> {
    use crate::api_client::server::ApiClient;

    let api_url = std::env::var("NEXCORE_API_URL").unwrap_or_else(|_| "http://localhost:3030".to_string());
    let api_key = std::env::var("NEXCORE_API_KEY").ok();
    let client = ApiClient::new(api_url, api_key);

    client.community_list_posts().await
        .map_err(ServerFnError::new)
}

#[component]
pub fn FeedPage() -> impl IntoView {
    let posts = Resource::new(|| (), |_| list_posts_action());

    view! {
        <div class="mx-auto max-w-3xl px-4 py-8">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Community Feed"</h1>
                    <p class="mt-1 text-slate-400">"Latest from the NexVigilant network"</p>
                </div>
                <a href="/community/create-post" class="rounded-lg bg-cyan-600 px-4 py-2.5 text-sm font-bold text-white hover:bg-cyan-500 transition-all shadow-lg shadow-cyan-900/20">
                    "NEW POST"
                </a>
            </div>

            <div class="mt-8 flex gap-6 border-b border-slate-800 pb-4 overflow-x-auto no-scrollbar font-mono text-[11px] font-bold uppercase tracking-widest text-slate-500">
                <button class="text-cyan-400 border-b-2 border-cyan-400 pb-4 -mb-[18px]">"For You"</button>
                <button class="hover:text-white transition-colors">"Following"</button>
                <button class="hover:text-white transition-colors">"Intelligence"</button>
                <button class="hover:text-white transition-colors">"Circles"</button>
            </div>

            <div class="mt-10 space-y-6">
                <Suspense fallback=|| view! { <LoadingFeed /> }>
                    {move || posts.get().map(|result| match result {
                        Ok(list) => view! { <PostsListView list=list /> }.into_any(),
                        Err(e) => view! { <div class="p-4 rounded-lg bg-red-500/10 border border-red-500/20 text-red-400 text-sm font-mono">{e.to_string()}</div> }.into_any()
                    })}
                </Suspense>
            </div>
        </div>
    }
}

#[component]
fn LoadingFeed() -> impl IntoView {
    view! {
        <div class="space-y-4 animate-pulse">
            {(0..3).map(|_| view! { 
                <div class="rounded-xl border border-slate-800 bg-slate-900/30 p-6 h-40">
                    <div class="flex items-center gap-3">
                        <div class="h-10 w-10 rounded-full bg-slate-800"></div>
                        <div class="space-y-2">
                            <div class="h-3 w-32 bg-slate-800 rounded"></div>
                            <div class="h-2 w-24 bg-slate-800 rounded"></div>
                        </div>
                    </div>
                    <div class="mt-6 h-4 w-full bg-slate-800 rounded"></div>
                    <div class="mt-2 h-4 w-2/3 bg-slate-800 rounded"></div>
                </div>
            }).collect_view()}
        </div>
    }
}

#[component]
fn PostsListView(list: Vec<Post>) -> impl IntoView {
    if list.is_empty() {
        view! { <p class="text-slate-500 italic text-center py-12">"No activity yet. Be the first to post!"</p> }.into_any()
    } else {
        view! {
            <div class="space-y-6">
                {list.into_iter().rev().map(|post| view! { <PostItem post=post /> }).collect_view()}
            </div>
        }.into_any()
    }
}

#[component]
fn PostItem(post: Post) -> impl IntoView {
    let initial = post.author.chars().next().unwrap_or('?').to_string().to_uppercase();
    
    view! {
        <article class="rounded-xl border border-slate-800 bg-slate-900/50 p-6 hover:border-slate-700 transition-all group">
            <div class="flex items-center gap-3">
                <div class="h-10 w-10 rounded-full bg-slate-800 border border-slate-700 flex items-center justify-center text-sm font-bold text-cyan-400 font-mono">
                    {initial}
                </div>
                <div>
                    <p class="text-sm font-bold text-white group-hover:text-cyan-400 transition-colors">{post.author}</p>
                    <p class="text-[10px] text-slate-500 font-bold uppercase tracking-wider">{post.role}" · "{post.created_at.format("%Y-%m-%d %H:%M").to_string()}</p>
                </div>
            </div>
            <p class="mt-4 text-sm text-slate-300 leading-relaxed">{post.content}</p>
            <div class="mt-6 flex gap-8 font-mono text-[10px] font-bold text-slate-500">
                <button class="flex items-center gap-1.5 hover:text-cyan-400 transition-colors uppercase">
                    "Like" <span class="text-slate-700">"//"</span> {post.likes}
                </button>
                <button class="flex items-center gap-1.5 hover:text-cyan-400 transition-colors uppercase">
                    "Reply" <span class="text-slate-700">"//"</span> {post.replies}
                </button>
                <button class="hover:text-white transition-colors uppercase">"Share"</button>
            </div>
        </article>
    }
}