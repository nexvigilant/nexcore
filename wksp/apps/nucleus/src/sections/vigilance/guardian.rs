//! Guardian control — homeostasis loop, risk evaluation, activity log

use leptos::prelude::*;
use crate::api_client::{GuardianStatus, GuardianEvalRequest, GuardianEvalResult};

/// Server function to trigger guardian tick
#[server(GuardianTick, "/api")]
pub async fn trigger_guardian_tick() -> Result<serde_json::Value, ServerFnError> {
    use crate::api_client::server::ApiClient;

    let api_url = std::env::var("NEXCORE_API_URL").unwrap_or_else(|_| "http://localhost:3030".to_string());
    let api_key = std::env::var("NEXCORE_API_KEY").ok();
    let client = ApiClient::new(api_url, api_key);

    client.guardian_tick().await.map_err(ServerFnError::new)
}

/// Server function to fetch guardian status
#[server(GetGuardianStatus, "/api")]
pub async fn get_guardian_status() -> Result<GuardianStatus, ServerFnError> {
    use crate::api_client::server::ApiClient;

    let api_url = std::env::var("NEXCORE_API_URL").unwrap_or_else(|_| "http://localhost:3030".to_string());
    let api_key = std::env::var("NEXCORE_API_KEY").ok();
    let client = ApiClient::new(api_url, api_key);

    client.guardian_status().await.map_err(ServerFnError::new)
}

/// Server function to evaluate risk
#[server(EvaluateRisk, "/api")]
pub async fn evaluate_risk_action(drug: String, event: String, count: u64) -> Result<GuardianEvalResult, ServerFnError> {
    use crate::api_client::server::ApiClient;

    let api_url = std::env::var("NEXCORE_API_URL").unwrap_or_else(|_| "http://localhost:3030".to_string());
    let api_key = std::env::var("NEXCORE_API_KEY").ok();
    let client = ApiClient::new(api_url, api_key);

    let req = GuardianEvalRequest {
        drug_name: drug,
        event_name: event,
        case_count: count,
    };

    client.guardian_evaluate(&req).await.map_err(ServerFnError::new)
}

#[component]
pub fn GuardianPage() -> impl IntoView {
    let status = Resource::new(|| (), |_| get_guardian_status());
    let tick_action = ServerAction::<GuardianTick>::new();
    let eval_action = ServerAction::<EvaluateRisk>::new();
    
    // Evaluation form signals
    let drug_name = RwSignal::new(String::new());
    let event_name = RwSignal::new(String::new());
    let case_count = RwSignal::new(String::new());

    // Activity log
    let log_entries = RwSignal::new(Vec::<String>::new());

    // Refetch status after tick
    Effect::new(move |_| {
        if tick_action.value().get().is_some() {
            status.refetch();
            log_entries.update(|logs| logs.insert(0, format!("{}: Homeostasis tick completed", chrono::Utc::now().format("%H:%M:%S"))));
        }
    });

    view! {
        <div class="mx-auto max-w-4xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Guardian Control"</h1>
            <p class="mt-2 text-slate-400">"Homeostasis loop: SENSE \u{2192} COMPARE \u{2192} ACT"</p>

            <div class="mt-10 grid gap-6">
                <StatusSection status=status />
                <ControlsSection tick_action=tick_action status=status />
                <RiskEvaluationSection 
                    drug_name=drug_name 
                    event_name=event_name 
                    case_count=case_count 
                    eval_action=eval_action 
                />
                <LogSection entries=log_entries />
            </div>
        </div>
    }
}

#[component]
fn StatusSection(status: Resource<Result<GuardianStatus, ServerFnError>>) -> impl IntoView {
    view! {
        <section class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
            <h2 class="text-sm font-bold text-slate-500 uppercase tracking-widest mb-6">"Homeostasis Core"</h2>
            <Suspense fallback=|| view! { <div class="animate-pulse h-20 bg-slate-800/50 rounded-lg"></div> }>
                {move || status.get().map(|result| match result {
                    Ok(g) => view! { <GuardianStatusDisplay status=g /> }.into_any(),
                    Err(e) => view! { <p class="text-red-400 font-mono text-xs">{e.to_string()}</p> }.into_any()
                })}
            </Suspense>
        </section>
    }
}

#[component]
fn GuardianStatusDisplay(status: GuardianStatus) -> impl IntoView {
    let state_color = if status.state == "active" || status.state == "healthy" { "text-green-400" } else { "text-amber-400" };
    view! {
        <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
            <div class="space-y-1">
                <p class="text-[10px] text-slate-500 font-bold uppercase">"Lifecycle"</p>
                <p class=format!("text-lg font-mono font-bold {}", state_color)>{status.state.to_uppercase()}</p>
            </div>
            <div class="space-y-1">
                <p class="text-[10px] text-slate-500 font-bold uppercase">"Iteration"</p>
                <p class="text-lg font-mono text-white">"#" {status.iteration}</p>
            </div>
            <div class="space-y-1">
                <p class="text-[10px] text-slate-500 font-bold uppercase">"Sensors"</p>
                <p class="text-lg font-mono text-white">{status.sensors}</p>
            </div>
            <div class="space-y-1">
                <p class="text-[10px] text-slate-500 font-bold uppercase">"Actuators"</p>
                <p class="text-lg font-mono text-white">{status.actuators}</p>
            </div>
        </div>
    }
}

#[component]
fn ControlsSection(
    tick_action: ServerAction<GuardianTick>,
    status: Resource<Result<GuardianStatus, ServerFnError>>
) -> impl IntoView {
    view! {
        <section class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
            <h2 class="text-sm font-bold text-slate-500 uppercase tracking-widest mb-4">"Direct Effectors"</h2>
            <div class="flex flex-wrap gap-3">
                <button 
                    on:click=move |_| { tick_action.dispatch(GuardianTick {}); }
                    disabled=tick_action.pending()
                    class="rounded-lg bg-amber-600 px-6 py-2 text-sm font-bold text-white hover:bg-amber-500 transition-all disabled:opacity-50"
                >
                    {move || if tick_action.pending().get() { "TICKING..." } else { "EXECUTE TICK" }}
                </button>
                <button 
                    on:click=move |_| { status.refetch(); }
                    class="rounded-lg border border-slate-700 bg-slate-800/50 px-6 py-2 text-sm font-bold text-slate-300 hover:bg-slate-800 transition-all"
                >
                    "REFRESH"
                </button>
            </div>
        </section>
    }
}

#[component]
fn RiskEvaluationSection(
    drug_name: RwSignal<String>,
    event_name: RwSignal<String>,
    case_count: RwSignal<String>,
    eval_action: ServerAction<EvaluateRisk>
) -> impl IntoView {
    let on_submit = move |_| {
        eval_action.dispatch(EvaluateRisk {
            drug: drug_name.get(),
            event: event_name.get(),
            count: case_count.get().parse().unwrap_or(0),
        });
    };

    view! {
        <section class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
            <h2 class="text-sm font-bold text-slate-500 uppercase tracking-widest mb-6">"Synthetic Evaluation"</h2>
            <div class="grid gap-4 sm:grid-cols-3 items-end">
                <EvaluationInput label="Drug Name" signal=drug_name placeholder="e.g. Warfarin" />
                <EvaluationInput label="Adverse Event" signal=event_name placeholder="e.g. Bleeding" />
                <EvaluationInput label="Case Count" signal=case_count placeholder="e.g. 42" />
                <div class="sm:col-span-3">
                    <button 
                        on:click=move |_| { on_submit(()); }
                        disabled=eval_action.pending()
                        class="w-full rounded-lg bg-slate-800 border border-slate-700 py-3 text-sm font-bold text-cyan-400 hover:border-cyan-500/50 transition-all disabled:opacity-50"
                    >
                        "RUN RISK ASSESSMENT"
                    </button>
                </div>
            </div>
            
            <Suspense fallback=|| ()>
                {move || eval_action.value().get().map(|res| match res {
                    Ok(data) => view! { <RiskResultView data=data /> }.into_any(),
                    Err(e) => view! { <p class="mt-4 text-red-400 text-xs font-mono">{e.to_string()}</p> }.into_any()
                })}
            </Suspense>
        </section>
    }
}

#[component]
fn EvaluationInput(label: &'static str, signal: RwSignal<String>, placeholder: &'static str) -> impl IntoView {
    view! {
        <div class="space-y-1.5">
            <label class="text-[10px] text-slate-500 font-bold uppercase ml-1">{label}</label>
            <input type="text"
                prop:value=move || signal.get()
                on:input=move |ev| signal.set(event_target_value(&ev))
                class="w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-sm text-white focus:border-cyan-500 focus:outline-none transition-all font-mono"
                placeholder=placeholder
            />
        </div>
    }
}

#[component]
fn RiskResultView(data: GuardianEvalResult) -> impl IntoView {
    let level_color = match data.risk_level.as_str() {
        "High" | "Critical" => "text-red-400",
        "Medium" => "text-amber-400",
        _ => "text-green-400",
    };

    view! {
        <div class="mt-6 p-4 rounded-lg bg-slate-950/50 border border-slate-800">
            <div class="flex justify-between items-center mb-4">
                <span class="text-xs font-bold text-slate-500 uppercase font-mono">"Risk Level"</span>
                <span class=format!("text-sm font-bold font-mono {}", level_color)>{data.risk_level.to_uppercase()}</span>
            </div>
            <div class="space-y-3">
                <p class="text-xs text-slate-400 font-bold uppercase">"Recommended Actions"</p>
                <ul class="space-y-1.5">
                    {data.recommended_actions.into_iter().map(|action| view! {
                        <li class="text-sm text-slate-200 flex items-center gap-2">
                            <span class="text-cyan-500 text-xs">"▶"</span>
                            {action}
                        </li>
                    }).collect_view()}
                </ul>
            </div>
        </div>
    }
}

#[component]
fn LogSection(entries: RwSignal<Vec<String>>) -> impl IntoView {
    view! {
        <section class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
            <h2 class="text-sm font-bold text-slate-500 uppercase tracking-widest mb-4">"Temporal Stream"</h2>
            <div class="max-h-48 overflow-y-auto space-y-2 font-mono text-[11px]">
                {move || {
                    let logs = entries.get();
                    if logs.is_empty() {
                        view! { <p class="text-slate-600 italic">"NO STREAM DATA"</p> }.into_any()
                    } else {
                        view! {
                            <ul class="space-y-1">
                                {logs.into_iter().map(|e| view! {
                                    <li class="text-slate-400 border-l border-slate-800 pl-3 py-0.5">{e}</li>
                                }).collect_view()}
                            </ul>
                        }.into_any()
                    }
                }}
            </div>
        </section>
    }
}