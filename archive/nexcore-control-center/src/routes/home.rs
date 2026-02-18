//! Dashboard home page

use crate::api::metrics::{Alert, MetricsResponse, StatusResponse};
use leptos::prelude::*;

/// Server function to fetch metrics status
#[server(GetStatus)]
pub async fn get_status() -> Result<StatusResponse, ServerFnError> {
    use crate::api::MetricsClient;

    let api_key = std::env::var("CLAUDE_METRICS_API_KEY").ok();
    let client = MetricsClient::new(api_key);

    client
        .status()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

/// Server function to fetch metrics
#[server(GetMetrics)]
pub async fn get_metrics() -> Result<MetricsResponse, ServerFnError> {
    use crate::api::MetricsClient;

    let api_key = std::env::var("CLAUDE_METRICS_API_KEY").ok();
    let client = MetricsClient::new(api_key);

    client
        .get_metrics()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

/// Server function to fetch alerts
#[server(GetAlerts)]
pub async fn get_alerts() -> Result<Vec<Alert>, ServerFnError> {
    use crate::api::MetricsClient;

    let api_key = std::env::var("CLAUDE_METRICS_API_KEY").ok();
    let client = MetricsClient::new(api_key);

    client
        .get_alerts()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

/// Dashboard home page
#[component]
pub fn HomePage() -> impl IntoView {
    let status = Resource::new(|| (), |()| get_status());
    let metrics = Resource::new(|| (), |()| get_metrics());
    let alerts = Resource::new(|| (), |()| get_alerts());

    view! {
        <div class="min-h-screen bg-gray-900 text-white p-6">
            <header class="mb-8">
                <h1 class="text-3xl font-bold text-blue-400">"NexCore Control Center"</h1>
                <p class="text-gray-400">"Unified monitoring for all NexCore systems"</p>
            </header>

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-8">
                <Suspense fallback=move || view! { <StatCard title="Status" value="...".to_string() /> }>
                    {move || status.get().map(|result| match result {
                        Ok(s) => view! {
                            <StatCard title="Uptime" value=format!("{}s", s.uptime_seconds) />
                            <StatCard title="Metrics" value=s.metrics_count.to_string() />
                            <StatCard title="Dashboards" value=s.dashboards_count.to_string() />
                            <StatCard title="Alerts" value=s.alerts_count.to_string() />
                        }.into_any(),
                        Err(e) => view! { <div class="col-span-4 text-red-400">{e.to_string()}</div> }.into_any()
                    })}
                </Suspense>
            </div>

            // NexCore Systems Grid
            <div class="grid grid-cols-1 lg:grid-cols-3 gap-6 mb-8">
                <super::GuardianPanel />
                <super::FridayPanel />
                <super::BrainPanel />
            </div>

            // Metrics and Alerts Grid
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                <section class="bg-gray-800 rounded-lg p-6">
                    <h2 class="text-xl font-semibold mb-4 text-blue-300">"Cloud Metrics"</h2>
                    <Suspense fallback=move || view! { <p>"Loading metrics..."</p> }>
                        {move || metrics.get().map(|result| match result {
                            Ok(m) => view! {
                                <div class="space-y-2">
                                    {m.metrics.iter().map(|(k, v)| view! {
                                        <div class="flex justify-between py-1 border-b border-gray-700">
                                            <span class="text-gray-300">{k.clone()}</span>
                                            <span class="text-green-400 font-mono">{format!("{v:.2}")}</span>
                                        </div>
                                    }).collect_view()}
                                </div>
                            }.into_any(),
                            Err(e) => view! { <p class="text-red-400">{e.to_string()}</p> }.into_any()
                        })}
                    </Suspense>
                </section>

                <section class="bg-gray-800 rounded-lg p-6">
                    <h2 class="text-xl font-semibold mb-4 text-blue-300">"Alerts"</h2>
                    <Suspense fallback=move || view! { <p>"Loading alerts..."</p> }>
                        {move || alerts.get().map(|result| match result {
                            Ok(a) if a.is_empty() => view! {
                                <p class="text-gray-500">"No active alerts"</p>
                            }.into_any(),
                            Ok(a) => {
                                let alerts_list = a.clone();
                                view! {
                                    <div class="space-y-3">
                                        {alerts_list.into_iter().map(|alert| {
                                            let name = alert.name.clone();
                                            let severity = alert.severity.clone();
                                            let severity_class = match severity.as_str() {
                                                "critical" => "text-red-400",
                                                "warning" => "text-yellow-400",
                                                _ => "text-blue-400"
                                            };
                                            let expr = alert.expr.clone();
                                            view! {
                                                <div class="bg-gray-700 rounded p-3">
                                                    <div class="flex justify-between items-center">
                                                        <span class="font-medium">{name}</span>
                                                        <span class=severity_class>{severity}</span>
                                                    </div>
                                                    <p class="text-sm text-gray-400 mt-1">{expr}</p>
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
                                }.into_any()
                            },
                            Err(e) => view! { <p class="text-red-400">{e.to_string()}</p> }.into_any()
                        })}
                    </Suspense>
                </section>
            </div>

            <footer class="mt-8 text-center text-gray-500 text-sm">
                <p>"NexCore Control Center v0.1.0 | Powered by Ferrostack"</p>
            </footer>
        </div>
    }
}

/// Stat card component
#[component]
fn StatCard(title: &'static str, value: String) -> impl IntoView {
    view! {
        <div class="bg-gray-800 rounded-lg p-4">
            <p class="text-gray-400 text-sm">{title}</p>
            <p class="text-2xl font-bold text-white">{value}</p>
        </div>
    }
}
