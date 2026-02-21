//! Circles — topic-focused professional groups

use leptos::prelude::*;
use crate::api_client::{Circle, JoinRequest};
use crate::auth::use_auth;

/// Server function to list community circles
#[server(ListCircles, "/api")]
pub async fn list_circles_action() -> Result<Vec<Circle>, ServerFnError> {
    use crate::api_client::server::ApiClient;

    let api_url = std::env::var("NEXCORE_API_URL").unwrap_or_else(|_| "http://localhost:3030".to_string());
    let api_key = std::env::var("NEXCORE_API_KEY").ok();
    let client = ApiClient::new(api_url, api_key);

    client.community_list_circles().await
        .map_err(ServerFnError::new)
}

/// Server function to join a circle
#[server(JoinCircle, "/api")]
pub async fn join_circle_action(circle_id: String, user_id: String) -> Result<(), ServerFnError> {
    use crate::api_client::server::ApiClient;

    let api_url = std::env::var("NEXCORE_API_URL").unwrap_or_else(|_| "http://localhost:3030".to_string());
    let api_key = std::env::var("NEXCORE_API_KEY").ok();
    let client = ApiClient::new(api_url, api_key);

    let req = JoinRequest { user_id };
    client.community_join_circle(&circle_id, &req).await
        .map(|_| ())
        .map_err(ServerFnError::new)
}

#[component]
pub fn CirclesPage() -> impl IntoView {
    let circles = Resource::new(|| (), |_| list_circles_action());

    view! {
        <div class="mx-auto max-w-6xl px-4 py-12">
            <div class="flex items-center justify-between mb-12">
                <div>
                    <h1 class="text-4xl font-bold text-white font-mono uppercase tracking-tight">"CIRCLES"</h1>
                    <p class="mt-2 text-slate-400">"Topic-based professional groups for deep collaboration"</p>
                </div>
                <button class="rounded-lg border border-slate-700 px-6 py-2.5 text-sm font-bold text-slate-300 hover:bg-slate-800 hover:text-white transition-all">
                    "CREATE CIRCLE"
                </button>
            </div>

            <div class="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
                <Suspense fallback=|| view! { <LoadingGrid /> }>
                    {move || circles.get().map(|result| match result {
                        Ok(list) => view! { <CirclesListView list=list /> }.into_any(),
                        Err(e) => view! { <div class="col-span-full p-6 rounded-xl bg-red-500/10 border border-red-500/20 text-red-400 font-mono text-sm">{e.to_string()}</div> }.into_any()
                    })}
                </Suspense>
            </div>
        </div>
    }
}

#[component]
fn LoadingGrid() -> impl IntoView {
    view! {
        <div class="col-span-full grid gap-6 md:grid-cols-2 lg:grid-cols-3 w-full">
            {(0..6).map(|_| view! { 
                <div class="rounded-2xl border border-slate-800 bg-slate-900/30 p-8 h-64 animate-pulse">
                    <div class="h-4 w-48 bg-slate-800 rounded mb-4"></div>
                    <div class="h-4 w-full bg-slate-800 rounded mb-2"></div>
                    <div class="h-4 w-2/3 bg-slate-800 rounded"></div>
                </div>
            }).collect_view()}
        </div>
    }
}

#[component]
fn CirclesListView(list: Vec<Circle>) -> impl IntoView {
    if list.is_empty() {
        view! { <p class="col-span-full text-slate-500 italic text-center py-20 font-mono">"NO CIRCLES DETECTED"</p> }.into_any()
    } else {
        view! {
            {list.into_iter().map(|circle| view! { <CircleCard circle=circle /> }).collect_view()}
        }.into_any()
    }
}

#[component]
fn CircleCard(circle: Circle) -> impl IntoView {
    let auth = use_auth();
    let join_action = ServerAction::<JoinCircle>::new();
    let result = join_action.value();
    let circle_id = circle.id.clone();

    view! {
        <div class="rounded-2xl border border-slate-800 bg-slate-900/50 p-8 flex flex-col hover:border-slate-700 transition-all group relative overflow-hidden">
            <div class="absolute top-0 right-0 p-4 opacity-10 group-hover:opacity-20 transition-opacity">
                <CircleIcon />
            </div>

            <h3 class="text-xl font-bold text-white font-mono tracking-tight group-hover:text-cyan-400 transition-colors mb-3">
                {circle.name.clone()}
            </h3>
            
            <p class="text-sm text-slate-400 leading-relaxed flex-grow mb-6">
                {circle.description.clone()}
            </p>

            <div class="flex items-center gap-4 text-[10px] font-bold text-slate-500 font-mono uppercase tracking-widest mb-8">
                <span class="flex items-center gap-1.5"><span class="h-1 w-1 rounded-full bg-slate-700"></span>{circle.member_count}" members"</span>
                <span class="flex items-center gap-1.5"><span class="h-1 w-1 rounded-full bg-slate-700"></span>{circle.post_count}" posts"</span>
            </div>

            {move || match result.get() {
                Some(Ok(_)) => view! { 
                    <span class="w-full rounded-lg bg-green-500/10 border border-green-500/20 py-2.5 text-center text-xs font-bold text-green-400 uppercase tracking-widest">
                        "JOINED"
                    </span>
                }.into_any(),
                _ => {
                    let cid = circle_id.clone();
                    view! {
                        <button 
                            on:click=move |_| {
                                let uid = auth.user.get().map(|u| u.uid).unwrap_or_else(|| "anonymous".to_string());
                                join_action.dispatch(JoinCircle { 
                                    circle_id: cid.clone(),
                                    user_id: uid,
                                });
                            }
                            disabled=join_action.pending()
                            class="w-full rounded-lg border border-violet-500/50 py-2.5 text-xs font-bold text-violet-400 hover:bg-violet-500 hover:text-white transition-all uppercase tracking-widest disabled:opacity-50"
                        >
                            {move || if join_action.pending().get() { "JOINING..." } else { "JOIN CIRCLE" }}
                        </button>
                    }.into_any()
                }
            }}
        </div>
    }
}

#[component]
fn CircleIcon() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="10"></circle>
            <path d="M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20"></path>
            <path d="M2 12h20"></path>
        </svg>
    }
}
