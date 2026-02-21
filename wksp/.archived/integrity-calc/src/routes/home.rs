//! Home page — calculator with server function
//!
//! Tier: T3 | Primitives: σ Sequence, → Causality, ∂ Boundary

use leptos::prelude::*;
use crate::components::{TextInput, ConfigPanel, FeatureChart, VerdictBadge, PipelineDisplay};
use crate::pipeline::PipelineResult;

/// Server function: text + config → full pipeline result.
#[server(AnalyzeText)]
pub async fn analyze_text(
    text: String,
    bloom_level: u8,
    preset: String,
) -> Result<PipelineResult, ServerFnError> {
    let result = crate::pipeline::run_pipeline(&text, bloom_level, &preset);
    if !result.error.is_empty() {
        return Err(ServerFnError::new(result.error));
    }
    Ok(result)
}

#[component]
pub fn HomePage() -> impl IntoView {
    let text = RwSignal::new(String::new());
    let bloom_level = RwSignal::new(3u8);
    let preset = RwSignal::new("pv_education".to_string());

    let analyze_action = ServerAction::<AnalyzeText>::new();
    let result = analyze_action.value();

    let on_analyze = move |_| {
        analyze_action.dispatch(AnalyzeText {
            text: text.get(),
            bloom_level: bloom_level.get(),
            preset: preset.get(),
        });
    };

    let is_pending = analyze_action.pending();

    view! {
        <div class="min-h-screen bg-gray-900 text-gray-100">
            // Header
            <header class="border-b border-gray-700 bg-gray-900/80 backdrop-blur sticky top-0 z-10">
                <div class="max-w-5xl mx-auto px-4 py-4 flex items-center justify-between">
                    <div>
                        <h1 class="text-2xl font-bold text-white">"Integrity Calculator"</h1>
                        <p class="text-sm text-gray-400">"AI Text Detection Pipeline"</p>
                    </div>
                    <a href="/about" class="text-sm text-blue-400 hover:text-blue-300 transition-colors">
                        "How it works \u{2192}"
                    </a>
                </div>
            </header>

            <main class="max-w-5xl mx-auto px-4 py-6 space-y-6">
                // Input + Config row
                <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                    <div class="md:col-span-2 space-y-3">
                        <TextInput text=text />
                        <button
                            class="px-6 py-2 bg-blue-600 hover:bg-blue-500 disabled:bg-gray-600 \
                                   disabled:cursor-not-allowed text-white font-semibold rounded-lg \
                                   transition-colors"
                            on:click=on_analyze
                            disabled=move || is_pending.get() || text.get().split_whitespace().count() < 50
                        >
                            {move || if is_pending.get() { "Analyzing..." } else { "Analyze" }}
                        </button>
                    </div>
                    <div>
                        <ConfigPanel bloom_level=bloom_level preset=preset />
                    </div>
                </div>

                // Results section
                {move || {
                    result.get().map(|res| {
                        match res {
                            Ok(r) => {
                                let r2 = r.clone();
                                let r3 = r.clone();
                                view! {
                                    <div class="space-y-4">
                                        <VerdictBadge result=r />
                                        <FeatureChart result=r2 />
                                        <PipelineDisplay result=r3 />
                                    </div>
                                }.into_any()
                            }
                            Err(e) => {
                                view! {
                                    <div class="bg-red-900/50 border border-red-500 rounded-lg p-4 text-red-300">
                                        <strong>"Error: "</strong>
                                        {e.to_string()}
                                    </div>
                                }.into_any()
                            }
                        }
                    })
                }}
            </main>

            // Footer
            <footer class="border-t border-gray-700 mt-12 py-4 text-center text-xs text-gray-500">
                "Integrity Calculator \u{2022} NexVigilant \u{2022} Beer-Lambert \u{2192} Hill \u{2192} Arrhenius Pipeline"
            </footer>
        </div>
    }
}
