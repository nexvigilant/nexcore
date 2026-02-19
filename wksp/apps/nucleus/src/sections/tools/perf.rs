//! Performance Analyzer — optimization analysis for engineering workflows

use crate::api_client::BrainThinkRequest;
use leptos::prelude::*;

#[server(AnalyzePerformance, "/api")]
pub async fn analyze_perf_action(profile: String) -> Result<String, ServerFnError> {
    use crate::api_client::server::ApiClient;
    let api_url =
        crate::runtime_config::nexcore_api_url();
    let api_key = crate::runtime_config::nexcore_api_key();
    let client = ApiClient::new(api_url, api_key);

    let req = BrainThinkRequest {
        prompt: format!(
            "Analyze the following performance profile/code for bottlenecks and optimization opportunities. Focus on async overhead, memory allocation, and instruction efficiency: {profile}"
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
pub fn PerfPage() -> impl IntoView {
    let profile = RwSignal::new(String::new());
    let analyze_action = ServerAction::<AnalyzePerformance>::new();
    let result = analyze_action.value();

    view! {
        <div class="mx-auto max-w-5xl px-4 py-12">
            <header class="mb-12">
                <div class="flex items-center gap-4 mb-4">
                    <a href="/tools" class="text-slate-500 hover:text-white transition-colors">{"\u{2190}"}</a>
                    <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Performance Analyzer"</h1>
                </div>
                <p class="text-slate-400">"Optimize async workflows and high-throughput data pipelines."</p>
            </header>

            <div class="grid gap-8 lg:grid-cols-2">
                /* ---- Input Section ---- */
                <section class="space-y-6">
                    <div class="rounded-2xl border border-slate-800 bg-slate-900/50 p-6">
                        <label class="block text-xs font-bold text-slate-500 uppercase tracking-widest mb-4 font-mono">
                            "Code / Performance Profile"
                        </label>
                        <textarea
                            prop:value=move || profile.get()
                            on:input=move |ev| profile.set(event_target_value(&ev))
                            class="w-full h-64 bg-slate-950 border border-slate-800 rounded-xl p-4 text-sm text-emerald-300 focus:border-emerald-500 focus:outline-none transition-all font-mono leading-relaxed"
                            placeholder="Paste your performance metrics, flamegraph data, or hot path code here..."
                        ></textarea>

                        <button
                            on:click=move |_| {
                                analyze_action.dispatch(AnalyzePerformance { profile: profile.get() });
                            }
                            disabled=analyze_action.pending()
                            class="w-full mt-6 py-4 rounded-xl bg-emerald-600/20 border border-emerald-500/30 text-emerald-400 font-black font-mono uppercase tracking-widest hover:bg-emerald-600/30 transition-all disabled:opacity-50 shadow-lg shadow-emerald-900/10"
                        >
                            {move || if analyze_action.pending().get() { "COMPUTING..." } else { "OPTIMIZE ARCHITECTURE" }}
                        </button>
                    </div>

                    <div class="grid grid-cols-2 gap-4">
                        <MetricBadge label="Latencies" value="ν Frequency" />
                        <MetricBadge label="Throughput" value="Σ Sum" />
                        <MetricBadge label="Allocation" value="N Quantity" />
                        <MetricBadge label="State" value="ς State" />
                    </div>
                </section>

                /* ---- Output Section ---- */
                <section class="rounded-2xl border border-slate-800 bg-slate-950/50 overflow-hidden flex flex-col h-[700px]">
                    <div class="bg-slate-900/80 px-4 py-2 border-b border-slate-800 flex justify-between items-center">
                        <span class="text-[10px] font-bold text-slate-500 uppercase tracking-widest font-mono">"Optimization Report"</span>
                        <span class="h-2 w-2 rounded-full bg-emerald-500 animate-pulse"></span>
                    </div>
                    <div class="flex-1 p-8 overflow-y-auto font-sans text-sm leading-relaxed">
                        <Suspense fallback=|| view! { <p class="text-slate-700 animate-pulse font-mono uppercase tracking-widest text-xs">"Processing profile..."</p> }>
                            {move || result.get().map(|res| match res {
                                Ok(report) => view! {
                                    <div class="text-slate-300 animate-in fade-in duration-700">
                                        <h3 class="text-emerald-400 font-mono text-xs uppercase mb-6 font-bold tracking-widest">"System Strategy"</h3>
                                        <div class="prose prose-invert prose-sm max-w-none">
                                            {report}
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
fn MetricBadge(label: &'static str, value: &'static str) -> impl IntoView {
    view! {
        <div class="p-4 rounded-xl border border-slate-800 bg-slate-900/30">
            <p class="text-[9px] font-bold text-slate-600 uppercase tracking-widest mb-1 font-mono">{label}</p>
            <p class="text-xs font-bold text-slate-400 font-mono">{value}</p>
        </div>
    }
}
