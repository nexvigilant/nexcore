//! Guardian status panel

use leptos::prelude::*;
use crate::api::{GuardianStatus, Sensor, Actuator};

/// NexCore API base URL
const NEXCORE_API: &str = "http://localhost:3030";

/// Server function to fetch Guardian status from real API
#[server(GetGuardianStatus)]
pub async fn get_guardian_status() -> Result<GuardianStatus, ServerFnError> {
    // Call real NexCore API
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/api/v1/guardian/status", NEXCORE_API))
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

    // Parse API response
    let api_response: serde_json::Value = response
        .json()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to parse response: {}", e)))?;

    // Map API response to our types
    let sensors: Vec<Sensor> = api_response
        .get("sensors")
        .and_then(|s| s.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|s| {
                    Some(Sensor {
                        name: s.get("name")?.as_str()?.to_string(),
                        description: s.get("description")?.as_str()?.to_string(),
                        sensor_type: s.get("sensor_type")?.as_str()?.to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let actuators: Vec<Actuator> = api_response
        .get("actuators")
        .and_then(|a| a.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|a| {
                    Some(Actuator {
                        name: a.get("name")?.as_str()?.to_string(),
                        description: a.get("description")?.as_str()?.to_string(),
                        priority: a.get("priority")?.as_u64()? as u32,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(GuardianStatus {
        status: api_response
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("unknown")
            .to_string(),
        iteration_count: api_response
            .get("iteration_count")
            .and_then(|i| i.as_u64())
            .unwrap_or(0),
        sensor_count: sensors.len(),
        actuator_count: actuators.len(),
        sensors,
        actuators,
    })
}

/// Guardian status panel component
#[component]
pub fn GuardianPanel() -> impl IntoView {
    let status = Resource::new(|| (), |_| get_guardian_status());

    view! {
        <section class="bg-gray-800 rounded-lg p-6">
            <h2 class="text-xl font-semibold mb-4 text-green-400">"Guardian Homeostasis"</h2>
            <Suspense fallback=move || view! { <p class="text-gray-400">"Loading..."</p> }>
                {move || status.get().map(|result| match result {
                    Ok(g) => {
                        let status_color = if g.status == "healthy" { "text-green-400" } else { "text-yellow-400" };
                        view! {
                            <div class="space-y-4">
                                <div class="flex items-center gap-2">
                                    <span class="text-gray-400">"Status:"</span>
                                    <span class=status_color>{g.status.clone()}</span>
                                </div>
                                <div class="flex items-center gap-2">
                                    <span class="text-gray-400">"Iterations:"</span>
                                    <span class="text-white font-mono">{g.iteration_count}</span>
                                </div>

                                <div>
                                    <h3 class="text-sm font-medium text-gray-300 mb-2">"Sensors ("{g.sensor_count}")"</h3>
                                    <div class="space-y-1">
                                        {g.sensors.iter().map(|s| {
                                            let type_color = match s.sensor_type.as_str() {
                                                "PAMPs" => "text-red-400",
                                                "DAMPs" => "text-orange-400",
                                                _ => "text-blue-400"
                                            };
                                            view! {
                                                <div class="flex justify-between text-sm py-1 border-b border-gray-700">
                                                    <span class="text-gray-300">{s.name.clone()}</span>
                                                    <span class=type_color>{s.sensor_type.clone()}</span>
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
                                </div>

                                <div>
                                    <h3 class="text-sm font-medium text-gray-300 mb-2">"Actuators ("{g.actuator_count}")"</h3>
                                    <div class="space-y-1">
                                        {g.actuators.iter().map(|a| {
                                            view! {
                                                <div class="flex justify-between text-sm py-1 border-b border-gray-700">
                                                    <span class="text-gray-300">{a.name.clone()}</span>
                                                    <span class="text-purple-400">"P"{a.priority}</span>
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
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
