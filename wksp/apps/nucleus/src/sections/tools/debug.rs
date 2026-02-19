//! Debug Assistant — AI-powered debugging and anomaly analysis

use crate::api_client::BrainThinkRequest;
use leptos::prelude::*;

#[server(AnalyzeDebug, "/api")]
pub async fn analyze_debug_action(logs: String) -> Result<String, ServerFnError> {
    use crate::api_client::server::ApiClient;
    let api_url =
        crate::runtime_config::nexcore_api_url();
    let api_key = crate::runtime_config::nexcore_api_key();
    let client = ApiClient::new(api_url, api_key);

    let req = BrainThinkRequest {
        prompt: format!(
            "Analyze the following logs/stack trace for logical errors, performance bottlenecks, or system anomalies. Suggest a fix: {logs}"
        ),
        session_id: None,
    };

    client
        .brain_think(&req)
        .await
        .map(|res| res.content)
        .map_err(ServerFnError::new)
}

#[component]
pub fn DebugPage() -> impl IntoView {
    let logs = RwSignal::new(String::new());
    let analyze_action = ServerAction::<AnalyzeDebug>::new();
    let result = analyze_action.value();

    view! {
        <div class="mx-auto max-w-5xl px-4 py-12">
            <header class="mb-12">
                <div class="flex items-center gap-4 mb-4">
                    <a href="/tools" class="text-slate-500 hover:text-white transition-colors">{"\u{2190}"}</a>
                    <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Debug Assistant"</h1>
                </div>
                <p class="text-slate-400">"Analyze stack traces and logs to identify root causes and anomalies."</p>
            </header>

            <div class="grid gap-8 lg:grid-cols-2">
                /* ---- Input Section ---- */
                <section class="space-y-6">
                    <div class="rounded-2xl border border-slate-800 bg-slate-900/50 p-6">
                        <label class="block text-xs font-bold text-slate-500 uppercase tracking-widest mb-4 font-mono">
                            "System Logs / Stack Trace"
                        </label>
                        <textarea
                            prop:value=move || logs.get()
                            on:input=move |ev| logs.set(event_target_value(&ev))
                            class="w-full h-64 bg-slate-950 border border-slate-800 rounded-xl p-4 text-[12px] text-red-300 focus:border-red-500 focus:outline-none transition-all font-mono leading-relaxed"
                            placeholder="Paste your error logs or stack trace here..."
                        ></textarea>

                        <button
                            on:click=move |_| {
                                analyze_action.dispatch(AnalyzeDebug { logs: logs.get() });
                            }
                            disabled=analyze_action.pending()
                            class="w-full mt-6 py-4 rounded-xl bg-red-600/20 border border-red-500/30 text-red-400 font-black font-mono uppercase tracking-widest hover:bg-red-600/30 transition-all disabled:opacity-50 shadow-lg shadow-red-900/10"
                        >
                            {move || if analyze_action.pending().get() { "ANALYZING..." } else { "RUN DIAGNOSTICS" }}
                        </button>
                    </div>

                    <div class="rounded-2xl border border-slate-800 bg-slate-900/30 p-6">
                        <h4 class="text-[10px] font-bold text-slate-500 uppercase tracking-[0.2em] mb-4 font-mono">"Capabilities"</h4>
                        <ul class="space-y-3">
                            <DebugCapability label="Logical Anomaly Detection" />
                            <DebugCapability label="Memory Leak Analysis" />
                            <DebugCapability label="Concurrency Deadlock ID" />
                            <DebugCapability label="Async State Trace" />
                        </ul>
                    </div>
                </section>

                /* ---- Output Section ---- */
                <section class="rounded-2xl border border-slate-800 bg-slate-950/50 overflow-hidden flex flex-col h-[700px]">
                    <div class="bg-slate-900/80 px-4 py-2 border-b border-slate-800 flex justify-between items-center">
                        <span class="text-[10px] font-bold text-slate-500 uppercase tracking-widest font-mono">"Diagnostic Output"</span>
                        <span class="h-2 w-2 rounded-full bg-red-500 animate-pulse"></span>
                    </div>
                    <div class="flex-1 p-8 overflow-y-auto font-sans text-sm leading-relaxed">
                        <Suspense fallback=|| view! { <p class="text-slate-700 animate-pulse font-mono uppercase tracking-widest text-xs">"Waiting for telemetry..."</p> }>
                            {move || result.get().map(|res| match res {
                                Ok(analysis) => view! {
                                    <div class="text-slate-300 animate-in fade-in duration-700">
                                        <h3 class="text-cyan-400 font-mono text-xs uppercase mb-6 font-bold tracking-widest">"Analysis Complete"</h3>
                                        <div class="prose prose-invert prose-sm">
                                            {analysis}
                                        </div>
                                    </div>
                                }.into_any(),
                                Err(e) => view! {
                                    <p class="text-red-500 font-mono text-xs">"! ERROR: " {e.to_string()}</p>
                                }.into_any()
                            })}
                        </Suspense>
                    </div>
                </section>
            </div>
        </div>
    }
}

#[component]
fn DebugCapability(label: &'static str) -> impl IntoView {
    view! {
        <li class="flex items-center gap-3 text-xs text-slate-400">
            <span class="text-red-500">"»"</span>
            {label}
        </li>
    }
}
