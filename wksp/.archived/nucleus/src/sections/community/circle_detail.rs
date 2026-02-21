//! Circle detail page — view posts and members within a circle

use crate::api_client::{CircleDetail, CommunityMember, Post};
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

/* ------------------------------------------------------------------ */
/*  Server functions                                                   */
/* ------------------------------------------------------------------ */

#[server(GetCircleDetail, "/api")]
pub async fn get_circle_detail(id: String) -> Result<CircleDetail, ServerFnError> {
    use crate::api_client::server::ApiClient;
    let api_url =
        crate::runtime_config::nexcore_api_url();
    let api_key = crate::runtime_config::nexcore_api_key();
    let client = ApiClient::new(api_url, api_key);

    client
        .community_get_circle(&id)
        .await
        .map_err(ServerFnError::new)
}

/* ------------------------------------------------------------------ */
/*  Page component                                                     */
/* ------------------------------------------------------------------ */

#[component]
pub fn CircleDetailPage() -> impl IntoView {
    let params = use_params_map();
    let circle_id = move || params.get().get("id").unwrap_or_default();

    let detail = Resource::new(move || circle_id(), |id| get_circle_detail(id));
    let active_tab = RwSignal::new("posts");

    view! {
        <div class="mx-auto max-w-5xl px-4 py-12">
            <Suspense fallback=|| view! { <div class="animate-pulse h-96 bg-slate-900/30 rounded-3xl"></div> }>
                {move || detail.get().map(|result| match result {
                    Ok(data) => view! { <CircleView data=data active_tab=active_tab /> }.into_any(),
                    Err(e) => view! { <div class="p-8 text-red-400 font-mono">{e.to_string()}</div> }.into_any()
                })}
            </Suspense>
        </div>
    }
}

#[component]
fn CircleView(data: CircleDetail, active_tab: RwSignal<&'static str>) -> impl IntoView {
    let circle = data.circle;

    view! {
        <header class="mb-12">
            <div class="flex items-center gap-4 mb-6">
                <a href="/community/circles" class="text-slate-500 hover:text-white transition-colors">{"\u{2190}"}</a>
                <span class="text-xs font-bold text-violet-400 font-mono uppercase tracking-widest">"Community Circle"</span>
            </div>

            <div class="flex flex-col md:flex-row justify-between items-start md:items-end gap-6">
                <div>
                    <h1 class="text-4xl font-black text-white font-mono uppercase tracking-tighter leading-none mb-4">
                        {circle.name}
                    </h1>
                    <p class="text-slate-400 max-w-2xl leading-relaxed">
                        {circle.description}
                    </p>
                </div>

                <button class="px-8 py-3 rounded-xl bg-violet-600 text-white font-black font-mono uppercase tracking-widest hover:bg-violet-500 transition-all shadow-lg shadow-violet-900/20">
                    "JOINED"
                </button>
            </div>

            <div class="mt-12 flex gap-8 border-b border-slate-800 font-mono text-[11px] font-bold uppercase tracking-widest text-slate-500">
                <button
                    on:click=move |_| active_tab.set("posts")
                    class=move || if active_tab.get() == "posts" { "text-cyan-400 border-b-2 border-cyan-400 pb-4 -mb-[2px]" } else { "hover:text-white transition-colors pb-4" }
                >
                    "Discussion (" {circle.post_count} ")"
                </button>
                <button
                    on:click=move |_| active_tab.set("members")
                    class=move || if active_tab.get() == "members" { "text-cyan-400 border-b-2 border-cyan-400 pb-4 -mb-[2px]" } else { "hover:text-white transition-colors pb-4" }
                >
                    "Collaborators (" {circle.member_count} ")"
                </button>
            </div>
        </header>

        <main>
            {move || match active_tab.get() {
                "posts" => view! { <CirclePosts posts=data.posts.clone() /> }.into_any(),
                "members" => view! { <CircleMembers members=data.members.clone() /> }.into_any(),
                _ => view! { <div/> }.into_any()
            }}
        </main>
    }
}

#[component]
fn CirclePosts(posts: Vec<Post>) -> impl IntoView {
    if posts.is_empty() {
        view! { <p class="text-slate-600 italic py-12 text-center font-mono">"NO RECENT DISCOURSE IN THIS CIRCLE"</p> }.into_any()
    } else {
        view! {
            <div class="space-y-6">
                {posts.into_iter().map(|post| view! { <PostItem post=post /> }).collect_view()}
            </div>
        }
        .into_any()
    }
}

#[component]
fn PostItem(post: Post) -> impl IntoView {
    let initial = post
        .author
        .chars()
        .next()
        .unwrap_or('?')
        .to_string()
        .to_uppercase();
    view! {
        <article class="glass-panel p-6 rounded-2xl border border-slate-800 hover:border-slate-700 transition-all group">
            <div class="flex items-center gap-3 mb-4">
                <div class="h-10 w-10 rounded-full bg-slate-800 border border-slate-700 flex items-center justify-center text-sm font-bold text-cyan-400 font-mono">
                    {initial}
                </div>
                <div>
                    <p class="text-sm font-bold text-white group-hover:text-cyan-400 transition-colors">{post.author}</p>
                    <p class="text-[10px] text-slate-500 font-bold uppercase tracking-wider">{post.created_at.format("%Y-%m-%d").to_string()}</p>
                </div>
            </div>
            <p class="text-sm text-slate-300 leading-relaxed mb-6">{post.content}</p>
            <div class="flex gap-6 text-[10px] font-bold text-slate-500 font-mono uppercase">
                <button class="hover:text-cyan-400 transition-colors">"Support // " {post.likes}</button>
                <button class="hover:text-cyan-400 transition-colors">"Interact // " {post.replies}</button>
            </div>
        </article>
    }
}

#[component]
fn CircleMembers(members: Vec<CommunityMember>) -> impl IntoView {
    view! {
        <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
            {members.into_iter().map(|m| view! { <MemberCard member=m /> }).collect_view()}
        </div>
    }
}

#[component]
fn MemberCard(member: CommunityMember) -> impl IntoView {
    let initial = member
        .name
        .chars()
        .next()
        .unwrap_or('?')
        .to_string()
        .to_uppercase();
    view! {
        <div class="glass-panel p-6 rounded-2xl border border-slate-800 flex items-center gap-4 hover:border-violet-500/30 transition-all">
            <div class="h-12 w-12 rounded-full bg-slate-800 border border-slate-700 flex items-center justify-center text-lg font-bold text-violet-400 font-mono">
                {initial}
            </div>
            <div>
                <p class="text-sm font-bold text-white">{member.name}</p>
                <p class="text-[10px] text-slate-500 font-bold uppercase tracking-widest">{member.role}</p>
            </div>
        </div>
    }
}
