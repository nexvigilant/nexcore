//! Brain sessions panel

use leptos::prelude::*;
use crate::api::{BrainSessionsResponse, BrainSession};

/// NexCore API base URL
const NEXCORE_API: &str = "http://localhost:3030";

/// Server function to fetch Brain sessions from real API
#[server(GetBrainSessions)]
pub async fn get_brain_sessions() -> Result<BrainSessionsResponse, ServerFnError> {
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/api/v1/brain/session/list", NEXCORE_API))
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

    let sessions: Vec<BrainSession> = api
        .get("sessions")
        .and_then(|s| s.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|s| {
                    Some(BrainSession {
                        session_id: s.get("session_id")?.as_str()?.to_string(),
                        project: s.get("project").and_then(|p| p.as_str()).map(String::from),
                        description: s.get("description").and_then(|d| d.as_str()).map(String::from),
                        created_at: s.get("created_at")?.as_str()?.to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(BrainSessionsResponse {
        count: sessions.len(),
        sessions,
    })
}

/// Brain sessions panel component
#[component]
pub fn BrainPanel() -> impl IntoView {
    let sessions = Resource::new(|| (), |_| get_brain_sessions());

    view! {
        <section class="bg-gray-800 rounded-lg p-6">
            <h2 class="text-xl font-semibold mb-4 text-purple-400">"Brain Sessions"</h2>
            <Suspense fallback=move || view! { <p class="text-gray-400">"Loading..."</p> }>
                {move || sessions.get().map(|result| match result {
                    Ok(b) => view! {
                        <div class="space-y-3">
                            <p class="text-gray-400 text-sm">{b.count}" session(s)"</p>
                            {b.sessions.iter().map(|s| view! {
                                <div class="bg-gray-700 rounded p-3">
                                    <div class="text-sm font-mono text-gray-300 truncate">{s.session_id.clone()}</div>
                                    <div class="text-purple-300 mt-1">{s.project.clone().unwrap_or_default()}</div>
                                    <div class="text-gray-500 text-xs mt-1">{s.created_at.clone()}</div>
                                </div>
                            }).collect_view()}
                        </div>
                    }.into_any(),
                    Err(e) => view! { <p class="text-red-400">{e.to_string()}</p> }.into_any()
                })}
            </Suspense>
        </section>
    }
}
