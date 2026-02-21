//! Signal detection page — run and view PV signals

use leptos::prelude::*;
use crate::api_client::{SignalRequest, SignalResult};

/// Server function to run signal detection
#[server(RunSignalCheck, "/api")]
pub async fn run_signal_check(a: u64, b: u64, c: u64, d: u64) -> Result<SignalResult, ServerFnError> {
    use crate::api_client::server::ApiClient;

    let api_url = std::env::var("NEXCORE_API_URL").unwrap_or_else(|_| "http://localhost:3030".to_string());
    let api_key = std::env::var("NEXCORE_API_KEY").ok();
    let client = ApiClient::new(api_url, api_key);

    let req = SignalRequest { a, b, c, d };
    client.signal_complete(&req).await
        .map_err(ServerFnError::new)
}

#[component]
pub fn SignalsPage() -> impl IntoView {
    let a = RwSignal::new(String::from("15"));
    let b = RwSignal::new(String::from("100"));
    let c = RwSignal::new(String::from("20"));
    let d = RwSignal::new(String::from("10000"));
    
    let signal_action = ServerAction::<RunSignalCheck>::new();
    let result = signal_action.value();

    view! {
        <div class="mx-auto max-w-4xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Signal Detection"</h1>
            <p class="mt-2 text-slate-400">"Perform disproportionality analysis (PRR, ROR, IC, EBGM)"</p>

            <div class="mt-10 grid gap-6 md:grid-cols-2">
                <ContingencyTableSection a=a b=b c=c d=d action=signal_action />
                <ResultsSection result=result />
            </div>
        </div>
    }
}

#[component]
fn ContingencyTableSection(
    a: RwSignal<String>,
    b: RwSignal<String>,
    c: RwSignal<String>,
    d: RwSignal<String>,
    action: ServerAction<RunSignalCheck>
) -> impl IntoView {
    let on_submit = move |_| {
        action.dispatch(RunSignalCheck {
            a: a.get().parse().unwrap_or(0),
            b: b.get().parse().unwrap_or(0),
            c: c.get().parse().unwrap_or(0),
            d: d.get().parse().unwrap_or(0),
        });
    };

    view! {
        <section class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
            <h2 class="text-sm font-bold text-slate-500 uppercase tracking-widest mb-6">"Input Matrix"</h2>
            <div class="space-y-4">
                <InputRow label="a (Drug + Event)" signal=a />
                <InputRow label="b (Drug + No Event)" signal=b />
                <InputRow label="c (Other Drugs + Event)" signal=c />
                <InputRow label="d (Other Drugs + No Event)" signal=d />
                
                <button
                    class="w-full rounded-lg bg-amber-600 px-4 py-3 text-sm font-bold text-white hover:bg-amber-500 transition-all disabled:opacity-50 mt-4"
                    disabled=action.pending()
                    on:click=move |_| { on_submit(()); }
                >
                    {move || if action.pending().get() { "ANALYZING..." } else { "EXECUTE ANALYSIS" }}
                </button>
            </div>
        </section>
    }
}

#[component]
fn ResultsSection(result: RwSignal<Option<Result<SignalResult, ServerFnError>>>) -> impl IntoView {
    view! {
        <section class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
            <h2 class="text-sm font-bold text-slate-500 uppercase tracking-widest mb-6">"Analysis Result"</h2>
            <div class="mt-4">
                <Suspense fallback=|| view! { <p class="text-slate-500 italic font-mono text-xs">"WAITING FOR DATA..."</p> }>
                    {move || result.get().map(|res| match res {
                        Ok(data) => view! { <SignalResultsView data=data /> }.into_any(),
                        Err(e) => view! { <p class="text-red-400 text-xs font-mono">{e.to_string()}</p> }.into_any()
                    })}
                </Suspense>
            </div>
        </section>
    }
}

#[component]
fn InputRow(label: &'static str, signal: RwSignal<String>) -> impl IntoView {
    view! {
        <div class="space-y-1.5">
            <label class="text-[10px] text-slate-500 font-bold uppercase ml-1">{label}</label>
            <input type="number"
                class="w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-sm text-white focus:border-amber-500 focus:outline-none font-mono"
                prop:value=move || signal.get()
                on:input=move |ev| signal.set(event_target_value(&ev))
            />
        </div>
    }
}

#[component]
fn SignalResultsView(data: SignalResult) -> impl IntoView {
    view! {
        <div class="space-y-3">
            <ResultRow label="PRR" value=format!("{:.2}", data.prr) is_signal=data.prr_signal />
            <ResultRow label="ROR" value=format!("{:.2}", data.ror) is_signal=data.ror_signal />
            <ResultRow label="IC (0.25)" value=format!("{:.2}", data.ic025) is_signal=data.ic_signal />
            <ResultRow label="EBGM" value=format!("{:.2}", data.ebgm) is_signal=data.ebgm_signal />
            <ResultRow label="Chi-Square" value=format!("{:.2}", data.chi_square) is_signal=data.chi_signal />
            
            <div class="mt-8 pt-6 border-t border-slate-800">
                {if data.any_signal {
                    view! { <div class="rounded-lg bg-red-500/10 border border-red-500/20 text-red-400 p-4 text-center font-bold font-mono tracking-tighter">"SIGNAL DETECTED"</div> }
                } else {
                    view! { <div class="rounded-lg bg-green-500/10 border border-green-500/20 text-green-400 p-4 text-center font-bold font-mono tracking-tighter">"NO SIGNAL"</div> }
                }}
            </div>
        </div>
    }
}

#[component]
fn ResultRow(label: &'static str, value: String, is_signal: bool) -> impl IntoView {
    let color = if is_signal { "text-red-400" } else { "text-slate-300" };
    view! {
        <div class="flex justify-between items-center text-xs py-2 border-b border-slate-800/50">
            <span class="text-slate-500 font-bold uppercase tracking-tighter">{label}</span>
            <span class=format!("font-mono font-bold {}", color)>{value}</span>
        </div>
    }
}