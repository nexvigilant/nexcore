//! API Explorer — interactive documentation and testing for NexCore endpoints

use leptos::prelude::*;
use serde_json::Value;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApiEndpoint {
    pub path: String,
    pub method: String,
    pub description: String,
}

#[server(GetEndpoints, "/api")]
pub async fn get_endpoints_action() -> Result<Vec<ApiEndpoint>, ServerFnError> {
    // In a real app, this might come from an OpenAPI spec or registry
    Ok(vec![
        ApiEndpoint {
            path: "/api/v1/health".into(),
            method: "GET".into(),
            description: "Core system health check".into(),
        },
        ApiEndpoint {
            path: "/api/v1/guardian/status".into(),
            method: "GET".into(),
            description: "Get current Guardian homeostasis state".into(),
        },
        ApiEndpoint {
            path: "/api/v1/brain/think".into(),
            method: "POST".into(),
            description: "Execute AI cognitive processing".into(),
        },
        ApiEndpoint {
            path: "/api/v1/academy/courses".into(),
            method: "GET".into(),
            description: "List all capability pathways".into(),
        },
        ApiEndpoint {
            path: "/api/v1/community/posts".into(),
            method: "GET".into(),
            description: "Fetch global community data feed".into(),
        },
    ])
}

#[server(ExecuteRequest, "/api")]
pub async fn execute_request_action(
    path: String,
    method: String,
    body: Option<String>,
) -> Result<Value, ServerFnError> {
    use crate::api_client::server::ApiClient;
    let api_url =
        crate::runtime_config::nexcore_api_url();
    let api_key = crate::runtime_config::nexcore_api_key();
    let client = ApiClient::new(api_url, api_key);

    // This is a simplified executor for the explorer
    // Real implementation would route based on path/method
    Ok(serde_json::json!({
        "status": "success",
        "telemetry": {
            "endpoint": path,
            "method": method,
            "timestamp": chrono::Utc::now().to_rfc3339()
        },
        "data": { "message": "Telemetric response simulated from NexCore" }
    }))
}

#[component]
pub fn ApiExplorerPage() -> impl IntoView {
    let endpoints = Resource::new(|| (), |_| get_endpoints_action());

    let selected_path = RwSignal::new(String::from("/api/v1/health"));
    let selected_method = RwSignal::new(String::from("GET"));
    let request_body = RwSignal::new(String::new());

    let execute_action = ServerAction::<ExecuteRequest>::new();
    let response = execute_action.value();

    view! {
        <div class="mx-auto max-w-6xl px-4 py-12">
            <header class="mb-12">
                <div class="flex items-center gap-4 mb-4">
                    <a href="/tools" class="text-slate-500 hover:text-white transition-colors">{"←"}</a>
                    <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"API Explorer"</h1>
                </div>
                <p class="text-slate-400 max-w-2xl">
                    "Interact directly with NexCore endpoints. Test payloads, inspect telemetry, and validate schema integrity."
                </p>
            </header>

            <div class="grid gap-8 lg:grid-cols-3">
                /* ---- Endpoint Registry ---- */
                <aside class="space-y-6">
                    <div class="rounded-2xl border border-slate-800 bg-slate-900/50 p-6 h-fit">
                        <h2 class="text-xs font-bold text-slate-500 uppercase tracking-widest mb-6 font-mono">"// ENDPOINT REGISTRY"</h2>

                        <Suspense fallback=|| view! { <div class="space-y-2 animate-pulse">{(0..5).map(|_| view! { <div class="h-10 bg-slate-800 rounded-lg"></div> }).collect_view()}</div> }>
                            {move || endpoints.get().map(|result| match result {
                                Ok(list) => view! {
                                    <div class="space-y-2">
                                        {list.into_iter().map(|ep| {
                                            let path = ep.path.clone();
                                            let method = ep.method.clone();
                                            let is_active = move || selected_path.get() == path;

                                            view! {
                                                <button
                                                    on:click=move |_| {
                                                        selected_path.set(ep.path.clone());
                                                        selected_method.set(ep.method.clone());
                                                    }
                                                    class=move || {
                                                        let base = "w-full text-left p-3 rounded-xl border transition-all group";
                                                        if is_active() {
                                                            format!("{base} bg-cyan-500/10 border-cyan-500/30 shadow-[0_0_15px_-5px_rgba(34,211,238,0.2)]")
                                                        } else {
                                                            format!("{base} bg-slate-950 border-slate-800 hover:border-slate-700")
                                                        }
                                                    }
                                                >
                                                    <div class="flex items-center gap-3">
                                                        <span class=move || {
                                                            let color = match method.as_str() {
                                                                "GET" => "text-emerald-400",
                                                                "POST" => "text-amber-400",
                                                                _ => "text-slate-400",
                                                            };
                                                            format!("text-[9px] font-black font-mono w-8 {color}")
                                                        }>{method.clone()}</span>
                                                        <span class="text-[11px] font-mono text-slate-300 truncate">{ep.path.clone()}</span>
                                                    </div>
                                                </button>
                                            }
                                        }).collect_view()}
                                    </div>
                                }.into_any(),
                                Err(_) => view! { <p class="text-red-500 text-xs font-mono">"Failed to load registry"</p> }.into_any(),
                            })}
                        </Suspense>
                    </div>
                </aside>

                /* ---- Request / Response Console ---- */
                <div class="lg:col-span-2 space-y-6">
                    <section class="rounded-2xl border border-slate-800 bg-slate-900/50 p-8">
                        <div class="flex items-center justify-between mb-8">
                            <div class="flex items-center gap-4 flex-1">
                                <span class="px-3 py-1 rounded bg-slate-950 border border-slate-800 text-xs font-black font-mono text-amber-400">
                                    {move || selected_method.get()}
                                </span>
                                <span class="text-sm font-mono text-white tracking-tight">{move || selected_path.get()}</span>
                            </div>
                            <button
                                on:click=move |_| {
                                    execute_action.dispatch(ExecuteRequest {
                                        path: selected_path.get(),
                                        method: selected_method.get(),
                                        body: Some(request_body.get()),
                                    });
                                }
                                disabled=execute_action.pending()
                                class="px-8 py-2.5 rounded-xl bg-cyan-600 text-white font-black font-mono uppercase tracking-widest text-xs hover:bg-cyan-500 transition-all disabled:opacity-50"
                            >
                                {move || if execute_action.pending().get() { "EXECUTING..." } else { "SEND REQUEST" }}
                            </button>
                        </div>

                        {move || if selected_method.get() == "POST" {
                            view! {
                                <div class="space-y-2">
                                    <label class="text-[10px] font-bold text-slate-500 uppercase tracking-widest font-mono ml-1">"Request Body (JSON)"</label>
                                    <textarea
                                        prop:value=move || request_body.get()
                                        on:input=move |ev| request_body.set(event_target_value(&ev))
                                        class="w-full h-32 bg-slate-950 border border-slate-800 rounded-xl p-4 text-xs text-white font-mono focus:border-cyan-500 focus:outline-none transition-all"
                                        placeholder="{ \"key\": \"value\" }"
                                    ></textarea>
                                </div>
                            }.into_any()
                        } else {
                            view! { <div/> }.into_any()
                        }}
                    </section>

                    <section class="rounded-2xl border border-slate-800 bg-slate-950 overflow-hidden flex flex-col h-[500px]">
                        <div class="bg-slate-900/80 px-6 py-3 border-b border-slate-800 flex justify-between items-center">
                            <span class="text-[10px] font-bold text-slate-500 uppercase tracking-widest font-mono">"Response Telemetry"</span>
                            <div class="flex gap-2">
                                <span class="h-2 w-2 rounded-full bg-emerald-500"></span>
                                <span class="text-[9px] font-bold text-emerald-500 font-mono">"200 OK"</span>
                            </div>
                        </div>
                        <div class="flex-1 p-8 overflow-y-auto font-mono text-[13px] leading-relaxed">
                            <Suspense fallback=|| view! { <p class="text-slate-800 animate-pulse uppercase tracking-[0.2em]">"Listening for signal..."</p> }>
                                {move || response.get().map(|res| match res {
                                    Ok(val) => view! {
                                        <pre class="text-cyan-200">{serde_json::to_string_pretty(&val).unwrap_or_default()}</pre>
                                    }.into_any(),
                                    Err(e) => view! { <p class="text-red-500">"ERROR: " {e.to_string()}</p> }.into_any(),
                                })}
                            </Suspense>
                        </div>
                    </section>
                </div>
            </div>
        </div>
    }
}
