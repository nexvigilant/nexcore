//! Drug Safety Search — FAERS-powered adverse event lookup for healthcare professionals.
//! Search by drug name, view top adverse events with signal metrics.

use crate::api_client::{DrugEventsResult, SignalRequest, SignalResult};
use leptos::prelude::*;

#[server(SearchDrugEvents, "/api")]
pub async fn search_drug_events(
    drug_name: String,
    limit: u32,
) -> Result<DrugEventsResult, ServerFnError> {
    use crate::api_client::server::ApiClient;
    let api_url =
        crate::runtime_config::nexcore_api_url();
    let api_key = crate::runtime_config::nexcore_api_key();
    let client = ApiClient::new(api_url, api_key);
    client
        .drug_events(&drug_name, Some(limit))
        .await
        .map_err(ServerFnError::new)
}

#[server(QuickSignalCheck, "/api")]
pub async fn quick_signal_check(
    a: u64,
    b: u64,
    c: u64,
    d: u64,
) -> Result<SignalResult, ServerFnError> {
    use crate::api_client::server::ApiClient;
    let api_url =
        crate::runtime_config::nexcore_api_url();
    let api_key = crate::runtime_config::nexcore_api_key();
    let client = ApiClient::new(api_url, api_key);
    let req = SignalRequest { a, b, c, d };
    client
        .signal_complete(&req)
        .await
        .map_err(ServerFnError::new)
}

#[component]
pub fn DrugSafetyPage() -> impl IntoView {
    let drug_name = RwSignal::new(String::new());
    let limit = RwSignal::new(String::from("20"));
    let search_action = ServerAction::<SearchDrugEvents>::new();
    let result = search_action.value();

    view! {
        <div class="mx-auto max-w-5xl px-4 py-8">
            <header class="mb-8 text-center">
                <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">
                    "Drug Safety Intelligence"
                </h1>
                <p class="mt-2 text-slate-400">
                    "Search FDA Adverse Event Reporting System (FAERS) for drug safety signals"
                </p>
            </header>

            /* Search form */
            <section class="rounded-xl border border-slate-800 bg-slate-900/50 p-6 mb-8">
                <div class="flex flex-col md:flex-row gap-4 items-end">
                    <div class="flex-1">
                        <label class="text-[10px] text-slate-500 font-bold uppercase ml-1 block mb-1.5">"Drug Name"</label>
                        <input
                            type="text"
                            placeholder="e.g., Aspirin, Metformin, Atorvastatin..."
                            class="w-full rounded-lg border border-slate-700 bg-slate-950 px-4 py-3 text-sm text-white focus:border-amber-500 focus:outline-none font-mono placeholder:text-slate-600"
                            prop:value=move || drug_name.get()
                            on:input=move |ev| drug_name.set(event_target_value(&ev))
                            on:keydown=move |ev| {
                                if ev.key() == "Enter" && !drug_name.get().is_empty() {
                                    search_action.dispatch(SearchDrugEvents {
                                        drug_name: drug_name.get(),
                                        limit: limit.get().parse().unwrap_or(20),
                                    });
                                }
                            }
                        />
                    </div>
                    <div class="w-24">
                        <label class="text-[10px] text-slate-500 font-bold uppercase ml-1 block mb-1.5">"Limit"</label>
                        <input
                            type="number"
                            class="w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-3 text-sm text-white focus:border-amber-500 focus:outline-none font-mono"
                            prop:value=move || limit.get()
                            on:input=move |ev| limit.set(event_target_value(&ev))
                        />
                    </div>
                    <button
                        on:click=move |_| {
                            if !drug_name.get().is_empty() {
                                search_action.dispatch(SearchDrugEvents {
                                    drug_name: drug_name.get(),
                                    limit: limit.get().parse().unwrap_or(20),
                                });
                            }
                        }
                        disabled=move || search_action.pending().get() || drug_name.get().is_empty()
                        class="rounded-lg bg-amber-600 px-8 py-3 text-sm font-bold text-white hover:bg-amber-500 transition-all shadow-lg shadow-amber-900/20 disabled:opacity-50 uppercase tracking-widest whitespace-nowrap"
                    >
                        {move || if search_action.pending().get() { "SEARCHING..." } else { "SEARCH FAERS" }}
                    </button>
                </div>

                /* Common drug quick-select */
                <div class="mt-4 flex flex-wrap gap-2">
                    {["Aspirin", "Metformin", "Atorvastatin", "Lisinopril", "Omeprazole", "Warfarin", "Methotrexate", "Infliximab"]
                        .into_iter()
                        .map(|name| {
                            let n = name.to_string();
                            view! {
                                <button
                                    on:click=move |_| {
                                        drug_name.set(n.clone());
                                        search_action.dispatch(SearchDrugEvents {
                                            drug_name: n.clone(),
                                            limit: limit.get().parse().unwrap_or(20),
                                        });
                                    }
                                    class="px-3 py-1 rounded-full border border-slate-700 bg-slate-950 text-[10px] font-bold text-slate-500 font-mono uppercase hover:border-amber-500/50 hover:text-amber-400 transition-all"
                                >
                                    {name}
                                </button>
                            }
                        })
                        .collect_view()}
                </div>
            </section>

            /* Results */
            <Suspense fallback=move || view! {
                <div class="text-center py-20 text-slate-500 font-mono text-xs">"Enter a drug name to search FAERS..."</div>
            }>
                {move || result.get().map(|res| match res {
                    Ok(data) => view! { <DrugEventsResultView data=data /> }.into_any(),
                    Err(e) => view! {
                        <div class="p-4 rounded-xl bg-red-500/10 border border-red-500/20 text-red-400 font-mono text-xs text-center">
                            {e.to_string()}
                        </div>
                    }.into_any()
                })}
            </Suspense>
        </div>
    }
}

#[component]
fn DrugEventsResultView(data: DrugEventsResult) -> impl IntoView {
    let signal_count = data.events.iter().filter(|e| e.is_signal).count();
    let total_events = data.events.len();

    view! {
        <div>
            /* Summary header */
            {
                let sig_color = if signal_count > 0 { "text-red-400" } else { "text-green-400" };
                view! {
                    <div class="grid grid-cols-3 gap-4 mb-6">
                        <StatCard label="Drug" value=data.drug.clone() color="text-white" />
                        <StatCard label="Total Reports" value=data.total_reports.to_string() color="text-cyan-400" />
                        <StatCard label="Signals Detected" value=format!("{} / {}", signal_count, total_events) color=sig_color />
                    </div>
                }
            }

            /* Events table */
            <section class="rounded-xl border border-slate-800 bg-slate-900/50 overflow-hidden">
                <div class="px-6 py-4 border-b border-slate-800">
                    <h2 class="text-sm font-bold text-slate-500 uppercase tracking-widest">"Adverse Events"</h2>
                </div>
                <div class="divide-y divide-slate-800">
                    /* Table header */
                    <div class="grid grid-cols-5 gap-4 px-6 py-3 text-[10px] font-bold text-slate-600 uppercase tracking-widest font-mono">
                        <div>"Event"</div>
                        <div class="text-right">"Reports"</div>
                        <div class="text-right">"PRR"</div>
                        <div class="text-right">"ROR"</div>
                        <div class="text-center">"Signal"</div>
                    </div>
                    /* Rows */
                    {data.events.into_iter().map(|event| {
                        let is_signal = event.is_signal;
                        let row_class = if is_signal {
                            "grid grid-cols-5 gap-4 px-6 py-3 hover:bg-red-500/5 transition-colors"
                        } else {
                            "grid grid-cols-5 gap-4 px-6 py-3 hover:bg-slate-800/50 transition-colors"
                        };
                        view! {
                            <div class=row_class>
                                <div class="text-sm text-slate-300 font-mono truncate">{event.event}</div>
                                <div class="text-sm text-slate-400 font-mono text-right">{event.count.to_string()}</div>
                                <div class={format!("text-sm font-mono text-right {}", if is_signal { "text-red-400" } else { "text-slate-400" })}>
                                    {event.prr.map(|v| format!("{:.2}", v)).unwrap_or_else(|| "-".to_string())}
                                </div>
                                <div class={format!("text-sm font-mono text-right {}", if is_signal { "text-red-400" } else { "text-slate-400" })}>
                                    {event.ror.map(|v| format!("{:.2}", v)).unwrap_or_else(|| "-".to_string())}
                                </div>
                                <div class="text-center">
                                    {if is_signal {
                                        view! {
                                            <span class="inline-flex px-2 py-0.5 rounded-full bg-red-500/10 border border-red-500/20 text-red-400 text-[10px] font-bold font-mono uppercase">"Signal"</span>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <span class="inline-flex px-2 py-0.5 rounded-full bg-slate-500/10 border border-slate-500/20 text-slate-500 text-[10px] font-bold font-mono uppercase">"None"</span>
                                        }.into_any()
                                    }}
                                </div>
                            </div>
                        }
                    }).collect_view()}
                </div>
            </section>
        </div>
    }
}

#[component]
fn StatCard(label: &'static str, value: String, color: &'static str) -> impl IntoView {
    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-4 text-center">
            <p class="text-[10px] text-slate-500 font-bold uppercase tracking-widest">{label}</p>
            <p class=format!("text-xl font-bold font-mono mt-1 {}", color)>{value}</p>
        </div>
    }
}
