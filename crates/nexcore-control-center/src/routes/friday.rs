//! FRIDAY orchestrator status panel

use leptos::prelude::*;
use crate::api::{FridayStatus, ProcessStatus, FridayComponents, FridaySource, FridayExecutor};

/// NexCore API base URL
const NEXCORE_API: &str = "http://localhost:3030";

/// Server function to fetch FRIDAY status from real API
#[server(GetFridayStatus)]
pub async fn get_friday_status() -> Result<FridayStatus, ServerFnError> {
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/api/v1/friday/status", NEXCORE_API))
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("API request failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(ServerFnError::new(format!(
            "API returned status {}",
            response.status()
        )));
    }

    let api: serde_json::Value = response
        .json()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to parse response: {}", e)))?;

    let sources: Vec<FridaySource> = api
        .get("sources")
        .and_then(|s| s.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|s| {
                    Some(FridaySource {
                        name: s.get("name")?.as_str()?.to_string(),
                        description: s.get("description")?.as_str()?.to_string(),
                        source_type: s.get("type")?.as_str()?.to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let executors: Vec<FridayExecutor> = api
        .get("executors")
        .and_then(|e| e.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|e| {
                    Some(FridayExecutor {
                        name: e.get("name")?.as_str()?.to_string(),
                        description: e.get("description")?.as_str()?.to_string(),
                        executor_type: e.get("type")?.as_str()?.to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(FridayStatus {
        status: api.get("status").and_then(|s| s.as_str()).unwrap_or("unknown").to_string(),
        process: ProcessStatus {
            name: api.get("process").and_then(|p| p.get("name")).and_then(|n| n.as_str()).unwrap_or("nexcore-friday").to_string(),
            running: api.get("process").and_then(|p| p.get("running")).and_then(|r| r.as_bool()).unwrap_or(false),
        },
        components: FridayComponents {
            event_bus: api.get("components").and_then(|c| c.get("event_bus")).cloned().unwrap_or(serde_json::json!({})),
            memory_layer: api.get("components").and_then(|c| c.get("memory_layer")).cloned().unwrap_or(serde_json::json!({})),
            decision_engine: api.get("components").and_then(|c| c.get("decision_engine")).cloned().unwrap_or(serde_json::json!({})),
        },
        sources,
        executors,
    })
}

/// FRIDAY status panel component
#[component]
pub fn FridayPanel() -> impl IntoView {
    let status = Resource::new(|| (), |_| get_friday_status());

    view! {
        <section class="bg-gray-800 rounded-lg p-6">
            <h2 class="text-xl font-semibold mb-4 text-blue-400">"FRIDAY Orchestrator"</h2>
            <Suspense fallback=move || view! { <p class="text-gray-400">"Loading..."</p> }>
                {move || status.get().map(|result| match result {
                    Ok(f) => {
                        let status_color = if f.process.running { "text-green-400" } else { "text-gray-500" };
                        view! {
                            <div class="space-y-4">
                                <div class="flex items-center gap-2">
                                    <span class="text-gray-400">"Process:"</span>
                                    <span class=status_color>{if f.process.running { "Running" } else { "Stopped" }}</span>
                                </div>
                                <div>
                                    <h3 class="text-sm font-medium text-gray-300 mb-2">"Sources"</h3>
                                    {f.sources.iter().map(|s| view! {
                                        <div class="text-sm py-1 text-gray-400">{s.name.clone()}</div>
                                    }).collect_view()}
                                </div>
                                <div>
                                    <h3 class="text-sm font-medium text-gray-300 mb-2">"Executors"</h3>
                                    {f.executors.iter().map(|e| view! {
                                        <div class="text-sm py-1 text-gray-400">{e.name.clone()}</div>
                                    }).collect_view()}
                                </div>
                            </div>
                        }.into_any()
                    },
                    Err(e) => view! { <p class="text-red-400">{e.to_string()}</p> }.into_any()
                })}
            </Suspense>
        </section>
    }
}
