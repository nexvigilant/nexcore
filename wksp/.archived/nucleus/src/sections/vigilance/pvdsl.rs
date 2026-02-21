//! PVDSL editor page — compile and execute pharmacovigilance domain-specific language

use leptos::prelude::*;
use crate::api_client::PvdslResult;

/// Server function to execute PVDSL code
#[server(ExecutePvdsl, "/api")]
pub async fn execute_pvdsl_action(code: String) -> Result<PvdslResult, ServerFnError> {
    use crate::api_client::server::ApiClient;

    let api_url = std::env::var("NEXCORE_API_URL").unwrap_or_else(|_| "http://localhost:3030".to_string());
    let api_key = std::env::var("NEXCORE_API_KEY").ok();
    let client = ApiClient::new(api_url, api_key);

    client.pvdsl_execute(&code).await
        .map_err(ServerFnError::new)
}

#[component]
pub fn PvdslPage() -> impl IntoView {
    let code = RwSignal::new(String::from("// PVDSL Signal Logic\nλ a = 15\nλ b = 100\nλ c = 20\nλ d = 10000\n\nμ check_signal(a, b, c, d) → B {\n    prr(a, b, c, d) κ> 2.0\n}\n\ncheck_signal(a, b, c, d)"));
    let execute_action = ServerAction::<ExecutePvdsl>::new();
    let result = execute_action.value();

    view! {
        <div class="mx-auto max-w-5xl px-4 py-8">
            <header class="mb-10 flex justify-between items-center">
                <div>
                    <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"PVDSL Studio"</h1>
                    <p class="mt-2 text-slate-400">"Domain-specific language for programmable pharmacovigilance"</p>
                </div>
                <button 
                    on:click=move |_| {
                        execute_action.dispatch(ExecutePvdsl { code: code.get() });
                    }
                    disabled=execute_action.pending()
                    class="rounded-lg bg-cyan-600 px-8 py-2.5 text-sm font-bold text-white hover:bg-cyan-500 transition-all shadow-lg shadow-cyan-900/20 disabled:opacity-50 uppercase tracking-widest"
                >
                    {move || if execute_action.pending().get() { "EXECUTING..." } else { "RUN CODE" }}
                </button>
            </header>

            <div class="grid gap-6 lg:grid-cols-2">
                <section class="rounded-xl border border-slate-800 bg-slate-900/50 overflow-hidden flex flex-col h-[600px]">
                    <div class="bg-slate-800/50 px-4 py-2 border-b border-slate-700 flex justify-between items-center">
                        <span class="text-[10px] font-bold text-slate-400 uppercase tracking-widest font-mono">"Editor"</span>
                        <span class="text-[10px] font-bold text-cyan-500 font-mono">"PRIMA-ENGINE v1.0"</span>
                    </div>
                    <textarea
                        prop:value=move || code.get()
                        on:input=move |ev| code.set(event_target_value(&ev))
                        class="flex-1 w-full p-6 bg-transparent text-cyan-50 font-mono text-sm resize-none focus:outline-none leading-relaxed"
                        spellcheck="false"
                    ></textarea>
                </section>

                <section class="rounded-xl border border-slate-800 bg-slate-950/50 overflow-hidden flex flex-col h-[600px]">
                    <div class="bg-slate-900/50 px-4 py-2 border-b border-slate-800 flex justify-between items-center">
                        <span class="text-[10px] font-bold text-slate-500 uppercase tracking-widest font-mono">"Output Terminal"</span>
                        <div class="flex gap-2">
                            <div class="h-2 w-2 rounded-full bg-slate-700"></div>
                            <div class="h-2 w-2 rounded-full bg-slate-700"></div>
                            <div class="h-2 w-2 rounded-full bg-slate-700"></div>
                        </div>
                    </div>
                    <div class="flex-1 p-6 font-mono text-sm overflow-y-auto">
                        <Suspense fallback=|| view! { <p class="text-slate-700 animate-pulse">"WAITING FOR EXECUTION..."</p> }>
                            {move || result.get().map(|res| match res {
                                Ok(data) => view! { <PvdslOutputView data=data /> }.into_any(),
                                Err(e) => view! { <p class="text-red-500">{format!("! EXECUTION ERROR: {e}")}</p> }.into_any()
                            })}
                        </Suspense>
                    </div>
                </section>
            </div>
        </div>
    }
}

#[component]
fn PvdslOutputView(data: PvdslResult) -> impl IntoView {
    let status_color = if data.success { "text-green-400" } else { "text-red-400" };
    view! {
        <div class="space-y-4">
            <div class="flex gap-2 items-center">
                <span class=format!("text-[10px] font-bold px-1.5 py-0.5 rounded border border-current {}", status_color)>
                    {if data.success { "SUCCESS" } else { "FAILED" }}
                </span>
            </div>
            <pre class="text-slate-300 whitespace-pre-wrap leading-relaxed">{data.output}</pre>
        </div>
    }
}