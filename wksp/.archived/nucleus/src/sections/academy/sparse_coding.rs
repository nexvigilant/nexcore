//! Interactive sparse coding lab — neural network visualization for PV signal patterns

use leptos::prelude::*;

#[component]
pub fn SparseCodingPage() -> impl IntoView {
    let (iterations, set_iterations) = signal(100u32);
    let (sparsity, set_sparsity) = signal(0.1f64);

    view! {
        <div class="space-y-6">
            <div>
                <a href="/academy/learn" class="text-cyan-400 hover:text-cyan-300 text-sm">"Back to Learning Hub"</a>
                <h1 class="text-2xl font-bold text-white mt-2">"Interactive: Sparse Coding for PV Signals"</h1>
                <p class="text-slate-400 mt-1">"Explore how sparse representation learning identifies patterns in adverse event data"</p>
            </div>

            <div class="grid gap-6 lg:grid-cols-3">
                <div class="lg:col-span-2">
                    <div class="bg-slate-800/50 border border-slate-700/50 rounded-xl p-6">
                        <h2 class="text-lg font-semibold text-white mb-4">"Signal Pattern Space"</h2>
                        /* Visualization placeholder — would use canvas/SVG in production */
                        <div class="bg-slate-900/80 rounded-lg h-80 flex items-center justify-center border border-slate-700/30">
                            <div class="text-center">
                                <div class="text-4xl font-bold font-mono text-cyan-400/50">"[ . . . x . . x . . . ]"</div>
                                <p class="text-slate-500 text-sm mt-4">"Sparse representation of adverse event co-occurrence patterns"</p>
                                <p class="text-slate-600 text-xs mt-1">"Each dot represents a dictionary atom, x marks active features"</p>
                            </div>
                        </div>
                    </div>
                </div>

                <div class="space-y-4">
                    <div class="bg-slate-800/50 border border-slate-700/50 rounded-xl p-5">
                        <h3 class="text-sm font-semibold text-white mb-4">"Parameters"</h3>
                        <div class="space-y-4">
                            <div>
                                <label class="text-xs text-slate-400 block mb-1">{move || format!("Iterations: {}", iterations.get())}</label>
                                <input
                                    type="range"
                                    min="10" max="1000" step="10"
                                    class="w-full accent-cyan-500"
                                    prop:value=move || iterations.get().to_string()
                                    on:input=move |ev| {
                                        if let Ok(v) = event_target_value(&ev).parse() {
                                            set_iterations.set(v);
                                        }
                                    }
                                />
                            </div>
                            <div>
                                <label class="text-xs text-slate-400 block mb-1">{move || format!("Sparsity: {:.2}", sparsity.get())}</label>
                                <input
                                    type="range"
                                    min="0.01" max="0.5" step="0.01"
                                    class="w-full accent-cyan-500"
                                    prop:value=move || format!("{:.2}", sparsity.get())
                                    on:input=move |ev| {
                                        if let Ok(v) = event_target_value(&ev).parse() {
                                            set_sparsity.set(v);
                                        }
                                    }
                                />
                            </div>
                            <button class="w-full px-4 py-2 bg-cyan-600 hover:bg-cyan-500 text-white rounded-lg text-sm font-medium transition-colors">
                                "Run Analysis"
                            </button>
                        </div>
                    </div>

                    <div class="bg-slate-800/50 border border-slate-700/50 rounded-xl p-5">
                        <h3 class="text-sm font-semibold text-white mb-3">"Concept"</h3>
                        <p class="text-sm text-slate-400 leading-relaxed">
                            "Sparse coding finds a small set of active features (dictionary atoms) that explain complex safety patterns. Like PV signal detection, it separates true signals from noise by finding the minimal representation."
                        </p>
                    </div>

                    <div class="bg-slate-800/50 border border-slate-700/50 rounded-xl p-5">
                        <h3 class="text-sm font-semibold text-white mb-3">"PV Application"</h3>
                        <ul class="text-sm text-slate-400 space-y-2">
                            <li>"Drug-event co-occurrence patterns"</li>
                            <li>"Temporal clustering of adverse events"</li>
                            <li>"Multi-drug interaction detection"</li>
                            <li>"Rare event amplification"</li>
                        </ul>
                    </div>
                </div>
            </div>
        </div>
    }
}
