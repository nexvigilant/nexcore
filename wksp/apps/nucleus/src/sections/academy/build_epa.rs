//! Build EPA page — structured EPA evidence submission with workplace-based assessment

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

#[component]
pub fn BuildEpaPage() -> impl IntoView {
    let params = use_params_map();
    let epa_id = move || params.read().get("epaId").unwrap_or_default();

    view! {
        <div class="space-y-6">
            <div>
                <a href="/academy/capabilities" class="text-cyan-400 hover:text-cyan-300 text-sm">"Back to Capabilities"</a>
                <h1 class="text-2xl font-bold text-white mt-2">"EPA Evidence Builder"</h1>
                <p class="text-slate-400 mt-1">{move || format!("Submitting evidence for EPA: {}", epa_id())}</p>
            </div>

            <div class="grid gap-6 lg:grid-cols-2">
                <div class="bg-slate-800/50 border border-slate-700/50 rounded-xl p-5">
                    <h2 class="text-lg font-semibold text-white mb-4">"Activity Details"</h2>
                    <div class="space-y-4">
                        <Field label="Activity Type" placeholder="Direct observation, case discussion, mini-CEX..."/>
                        <Field label="Setting" placeholder="Clinical, regulatory, industry..."/>
                        <Field label="Date Performed" placeholder="YYYY-MM-DD"/>
                        <Field label="Duration" placeholder="e.g., 2 hours"/>
                        <div>
                            <label class="text-sm text-slate-400 block mb-1">"Complexity Level"</label>
                            <select class="w-full bg-slate-900/50 border border-slate-700/50 rounded-lg px-4 py-2 text-white focus:outline-none focus:border-cyan-500/50">
                                <option>"Low — routine, well-defined"</option>
                                <option>"Medium — some ambiguity"</option>
                                <option>"High — complex, novel situation"</option>
                            </select>
                        </div>
                    </div>
                </div>

                <div class="bg-slate-800/50 border border-slate-700/50 rounded-xl p-5">
                    <h2 class="text-lg font-semibold text-white mb-4">"Entrustment Scale"</h2>
                    <div class="space-y-3">
                        <EntrustLevel level=1 desc="Observe only — not yet trusted to perform" active=false/>
                        <EntrustLevel level=2 desc="Perform with direct supervision" active=false/>
                        <EntrustLevel level=3 desc="Perform with indirect supervision" active=false/>
                        <EntrustLevel level=4 desc="Perform unsupervised" active=false/>
                        <EntrustLevel level=5 desc="Supervise and teach others" active=false/>
                    </div>
                </div>
            </div>

            <div class="bg-slate-800/50 border border-slate-700/50 rounded-xl p-5">
                <h2 class="text-lg font-semibold text-white mb-4">"Narrative Evidence"</h2>
                <textarea
                    class="w-full bg-slate-900/50 border border-slate-700/50 rounded-lg px-4 py-3 text-white placeholder-slate-500 focus:outline-none focus:border-cyan-500/50 min-h-[200px] resize-none"
                    placeholder="Describe what you did, the decisions you made, challenges encountered, and outcomes achieved..."
                />
            </div>

            <div class="flex justify-end gap-3">
                <button class="px-4 py-2 border border-slate-700 text-slate-400 hover:text-white rounded-lg text-sm font-medium transition-colors">
                    "Save Draft"
                </button>
                <button class="px-4 py-2 bg-cyan-600 hover:bg-cyan-500 text-white rounded-lg text-sm font-medium transition-colors">
                    "Submit for Verification"
                </button>
            </div>
        </div>
    }
}

#[component]
fn Field(#[prop(into)] label: String, #[prop(into)] placeholder: String) -> impl IntoView {
    view! {
        <div>
            <label class="text-sm text-slate-400 block mb-1">{label}</label>
            <input
                type="text"
                class="w-full bg-slate-900/50 border border-slate-700/50 rounded-lg px-4 py-2 text-white placeholder-slate-500 focus:outline-none focus:border-cyan-500/50"
                placeholder=placeholder
            />
        </div>
    }
}

#[component]
fn EntrustLevel(level: u32, #[prop(into)] desc: String, active: bool) -> impl IntoView {
    let cls = if active {
        "border-cyan-500/50 bg-cyan-500/10"
    } else {
        "border-slate-700/50 bg-slate-900/30 hover:border-slate-600/50"
    };
    view! {
        <div class={format!("border rounded-lg p-3 cursor-pointer transition-colors {}", cls)}>
            <div class="flex items-center gap-3">
                <div class="w-8 h-8 rounded-full bg-slate-800 flex items-center justify-center text-sm font-bold text-white">
                    {level.to_string()}
                </div>
                <span class="text-sm text-slate-300">{desc}</span>
            </div>
        </div>
    }
}
