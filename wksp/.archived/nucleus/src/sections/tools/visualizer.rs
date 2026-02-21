//! Architecture Visualizer — Lex Primitiva composition mapping for software designs

use crate::api_client::BrainThinkRequest;
use leptos::prelude::*;

#[server(AnalyzeArchitecture, "/api")]
pub async fn analyze_arch_action(code: String) -> Result<String, ServerFnError> {
    use crate::api_client::server::ApiClient;
    let api_url =
        crate::runtime_config::nexcore_api_url();
    let api_key = crate::runtime_config::nexcore_api_key();
    let client = ApiClient::new(api_url, api_key);

    let req = BrainThinkRequest {
        prompt: format!(
            "Perform a deep primitive analysis on the following architecture/code. Decompose it into the 16 T1 Lex Primitiva symbols (σ, ς, μ, etc.) and explain the grounding: {code}"
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
pub fn ArchVisualizerPage() -> impl IntoView {
    let input = RwSignal::new(String::new());
    let analyze_action = ServerAction::<AnalyzeArchitecture>::new();
    let result = analyze_action.value();

    view! {
        <div class="mx-auto max-w-6xl px-4 py-12">
            <header class="mb-12">
                <div class="flex items-center gap-4 mb-4">
                    <a href="/tools" class="text-slate-500 hover:text-white transition-colors">{"\u{2190}"}</a>
                    <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Architecture Visualizer"</h1>
                </div>
                <p class="text-slate-400 max-w-2xl">
                    "Decompose system designs into their fundamental primitives. Ground your architecture to the 16 Lex Primitiva."
                </p>
            </header>

            <div class="grid gap-8 lg:grid-cols-2">
                /* ---- Input Section ---- */
                <section class="space-y-6">
                    <div class="rounded-2xl border border-slate-800 bg-slate-900/50 p-6">
                        <label class="block text-xs font-bold text-slate-500 uppercase tracking-widest mb-4 font-mono">
                            "System Description / Rust Code"
                        </label>
                        <textarea
                            prop:value=move || input.get()
                            on:input=move |ev| input.set(event_target_value(&ev))
                            class="w-full h-64 bg-slate-950 border border-slate-800 rounded-xl p-4 text-sm text-cyan-300 focus:border-cyan-500 focus:outline-none transition-all font-mono leading-relaxed"
                            placeholder="Paste architecture notes or a Rust module..."
                        ></textarea>

                        <button
                            on:click=move |_| {
                                analyze_action.dispatch(AnalyzeArchitecture { code: input.get() });
                            }
                            disabled=analyze_action.pending()
                            class="w-full mt-6 py-4 rounded-xl bg-violet-600/20 border border-violet-500/30 text-violet-400 font-black font-mono uppercase tracking-widest hover:bg-violet-600/30 transition-all disabled:opacity-50 shadow-lg shadow-violet-900/10"
                        >
                            {move || if analyze_action.pending().get() { "DECOMPOSING..." } else { "MAP TO PRIMITIVES" }}
                        </button>
                    </div>

                    /* Primitive Reference Grid */
                    <div class="grid grid-cols-8 gap-2">
                        <MiniSymbol sym="→" label="Causality" />
                        <MiniSymbol sym="N" label="Quantity" />
                        <MiniSymbol sym="∃" label="Existence" />
                        <MiniSymbol sym="κ" label="Comparison" />
                        <MiniSymbol sym="ς" label="State" />
                        <MiniSymbol sym="μ" label="Mapping" />
                        <MiniSymbol sym="σ" label="Sequence" />
                        <MiniSymbol sym="ρ" label="Recursion" />
                        <MiniSymbol sym="∅" label="Void" />
                        <MiniSymbol sym="∂" label="Boundary" />
                        <MiniSymbol sym="ν" label="Frequency" />
                        <MiniSymbol sym="λ" label="Location" />
                        <MiniSymbol sym="π" label="Persistence" />
                        <MiniSymbol sym="∝" label="Irreversibility" />
                        <MiniSymbol sym="Σ" label="Sum" />
                        <MiniSymbol sym="×" label="Product" />
                    </div>
                </section>

                /* ---- Visualization Output ---- */
                <section class="rounded-2xl border border-slate-800 bg-slate-950/50 overflow-hidden flex flex-col min-h-[600px]">
                    <div class="bg-slate-900/80 px-4 py-2 border-b border-slate-800 flex justify-between items-center">
                        <span class="text-[10px] font-bold text-slate-500 uppercase tracking-widest font-mono">"Grounding Report"</span>
                        <div class="flex items-center gap-2">
                            <span class="text-[9px] font-bold text-cyan-500 font-mono">"T1 ANALYSIS ACTIVE"</span>
                        </div>
                    </div>
                    <div class="flex-1 p-8 overflow-y-auto">
                        <Suspense fallback=|| view! { <p class="text-slate-700 animate-pulse font-mono uppercase tracking-[0.2em] text-xs">"Processing spectral composition..."</p> }>
                            {move || result.get().map(|res| match res {
                                Ok(report) => view! {
                                    <div class="prose prose-invert prose-sm max-w-none prose-headings:font-mono prose-headings:uppercase prose-p:text-slate-400 prose-strong:text-white animate-in fade-in slide-in-from-bottom-4 duration-700">
                                        {report}
                                    </div>
                                }.into_any(),
                                Err(e) => view! { <p class="text-red-500 font-mono text-xs">"! ERROR: " {e.to_string()}</p> }.into_any(),
                            })}
                        </Suspense>
                    </div>
                </section>
            </div>
        </div>
    }
}

#[component]
fn MiniSymbol(sym: &'static str, label: &'static str) -> impl IntoView {
    view! {
        <div class="aspect-square rounded bg-slate-900 border border-slate-800 flex items-center justify-center text-[10px] text-slate-500 hover:text-cyan-400 hover:border-cyan-500/30 transition-all cursor-help" title=label>
            {sym}
        </div>
    }
}
