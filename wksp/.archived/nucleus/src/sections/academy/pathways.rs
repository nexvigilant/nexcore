//! Academy pathways page — browse competency-based learning paths

use leptos::prelude::*;
use crate::api_client::LearningPathway;

/// Server function to list academy pathways
#[server(ListPathways, "/api")]
pub async fn list_pathways_action() -> Result<Vec<LearningPathway>, ServerFnError> {
    use crate::api_client::server::ApiClient;

    let api_url = std::env::var("NEXCORE_API_URL").unwrap_or_else(|_| "http://localhost:3030".to_string());
    let api_key = std::env::var("NEXCORE_API_KEY").ok();
    let client = ApiClient::new(api_url, api_key);

    client.academy_pathways().await
        .map_err(ServerFnError::new)
}

#[component]
pub fn PathwaysPage() -> impl IntoView {
    let pathways = Resource::new(|| (), |_| list_pathways_action());

    view! {
        <div class="mx-auto max-w-6xl px-4 py-12">
            <header class="mb-12">
                <h1 class="text-4xl font-bold text-white font-mono uppercase tracking-tight">"PATHWAYS"</h1>
                <p class="mt-2 text-slate-400">"Linear sequences of professional growth grounded to Lex Primitiva."</p>
            </header>

            <div class="grid gap-8">
                <Suspense fallback=|| view! { <div class="p-12 animate-pulse text-slate-500">"SYNCHRONIZING..."</div> }>
                    {move || pathways.get().map(|result| match result {
                        Ok(list) => view! { <PathwaysListView list=list /> }.into_any(),
                        Err(e) => view! { <div class="p-8 text-red-400 font-mono text-sm">{e.to_string()}</div> }.into_any()
                    })}
                </Suspense>
            </div>
        </div>
    }
}

#[component]
fn PathwaysListView(list: Vec<LearningPathway>) -> impl IntoView {
    if list.is_empty() {
        view! { <p class="text-slate-500 italic py-20 text-center font-mono">"NO PATHWAYS ACTIVE IN THIS SECTOR"</p> }.into_any()
    } else {
        view! {
            <div class="space-y-8">
                {list.into_iter().map(|pathway| view! { <PathwayRow pathway=pathway /> }).collect_view()}
            </div>
        }.into_any()
    }
}

#[component]
fn PathwayRow(pathway: LearningPathway) -> impl IntoView {
    view! {
        <div class="glass-panel p-8 rounded-2xl border border-slate-800 hover:border-cyan-500/30 transition-all group">
            <div class="flex flex-col md:flex-row justify-between gap-6">
                <div class="flex-1">
                    <h3 class="text-2xl font-bold text-white font-mono uppercase tracking-tight mb-2 group-hover:text-cyan-400 transition-colors">
                        {pathway.title}
                    </h3>
                    <p class="text-slate-400 max-w-2xl">{pathway.description}</p>
                </div>
                <div class="flex items-center gap-4">
                    <div class="text-right">
                        <p class="text-[10px] font-bold text-slate-600 font-mono uppercase tracking-widest">"COMPLETION"</p>
                        <p class="text-xl font-black text-slate-500 font-mono">"0%"</p>
                    </div>
                    <button class="rounded-lg bg-cyan-600 px-6 py-2.5 text-xs font-bold text-white hover:bg-cyan-500 transition-all font-mono uppercase tracking-widest shadow-lg shadow-cyan-900/20">
                        "ENGAGE"
                    </button>
                </div>
            </div>

            <div class="mt-8 flex items-center gap-2 overflow-x-auto pb-4">
                {pathway.nodes.into_iter().map(|node| view! { 
                    <div class="flex items-center shrink-0">
                        <div class="w-48 p-4 rounded-xl border border-slate-800 bg-slate-900/30">
                            <p class="text-[9px] font-bold text-cyan-500 font-mono mb-1 uppercase">{node.level}</p>
                            <p class="text-xs text-slate-300 font-bold leading-tight">{node.title}</p>
                        </div>
                        <span class="px-2 text-slate-700 font-mono">"---"</span>
                    </div>
                }).collect_view()}
                <div class="w-12 h-12 rounded-full border-2 border-dashed border-slate-800 flex items-center justify-center text-slate-700 font-mono text-xs">
                    "END"
                </div>
            </div>
        </div>
    }
}