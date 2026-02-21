//! Build page — interactive course/content builder for learners to create portfolios

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

#[component]
pub fn BuildPage() -> impl IntoView {
    let params = use_params_map();
    let _id = move || params.read().get("id").unwrap_or_default();

    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <div>
                    <a href="/academy/portfolio" class="text-cyan-400 hover:text-cyan-300 text-sm">"Back to Portfolio"</a>
                    <h1 class="text-2xl font-bold text-white mt-2">"Portfolio Builder"</h1>
                    <p class="text-slate-400 mt-1">"Create and submit evidence for your competency portfolio"</p>
                </div>
                <button class="px-4 py-2 bg-cyan-600 hover:bg-cyan-500 text-white rounded-lg text-sm font-medium transition-colors">
                    "Save Draft"
                </button>
            </div>

            <div class="grid gap-6 lg:grid-cols-3">
                <div class="lg:col-span-2 space-y-4">
                    <div class="bg-slate-800/50 border border-slate-700/50 rounded-xl p-5">
                        <label class="text-sm font-semibold text-white block mb-2">"Title"</label>
                        <input
                            type="text"
                            class="w-full bg-slate-900/50 border border-slate-700/50 rounded-lg px-4 py-2 text-white placeholder-slate-500 focus:outline-none focus:border-cyan-500/50"
                            placeholder="Evidence title..."
                        />
                    </div>

                    <div class="bg-slate-800/50 border border-slate-700/50 rounded-xl p-5">
                        <label class="text-sm font-semibold text-white block mb-2">"Description"</label>
                        <textarea
                            class="w-full bg-slate-900/50 border border-slate-700/50 rounded-lg px-4 py-2 text-white placeholder-slate-500 focus:outline-none focus:border-cyan-500/50 min-h-[150px] resize-none"
                            placeholder="Describe the activity, context, and what you learned..."
                        />
                    </div>

                    <div class="bg-slate-800/50 border border-slate-700/50 rounded-xl p-5">
                        <label class="text-sm font-semibold text-white block mb-2">"Reflection"</label>
                        <textarea
                            class="w-full bg-slate-900/50 border border-slate-700/50 rounded-lg px-4 py-2 text-white placeholder-slate-500 focus:outline-none focus:border-cyan-500/50 min-h-[100px] resize-none"
                            placeholder="What would you do differently? How does this connect to your learning goals?"
                        />
                    </div>

                    <div class="bg-slate-800/50 border border-slate-700/50 rounded-xl p-5">
                        <label class="text-sm font-semibold text-white block mb-2">"Evidence Attachments"</label>
                        <div class="border-2 border-dashed border-slate-700/50 rounded-lg p-8 text-center">
                            <p class="text-slate-400">"Drag and drop files here or click to upload"</p>
                            <p class="text-xs text-slate-500 mt-2">"PDF, DOCX, PNG, JPG up to 10MB each"</p>
                        </div>
                    </div>
                </div>

                <div class="space-y-4">
                    <div class="bg-slate-800/50 border border-slate-700/50 rounded-xl p-5">
                        <h3 class="text-sm font-semibold text-white mb-3">"Linked Competencies"</h3>
                        <p class="text-xs text-slate-400 mb-3">"Select the EPAs/CPAs this evidence supports"</p>
                        <div class="space-y-2">
                            <div class="text-sm text-slate-500 text-center py-4">"No competencies linked yet"</div>
                        </div>
                        <button class="w-full mt-2 px-3 py-2 border border-slate-700/50 text-slate-400 hover:text-white rounded-lg text-xs transition-colors">
                            "Link Competency"
                        </button>
                    </div>

                    <div class="bg-slate-800/50 border border-slate-700/50 rounded-xl p-5">
                        <h3 class="text-sm font-semibold text-white mb-3">"Status"</h3>
                        <div class="space-y-2 text-sm">
                            <div class="flex justify-between">
                                <span class="text-slate-400">"State"</span>
                                <span class="text-amber-400 font-mono">"Draft"</span>
                            </div>
                            <div class="flex justify-between">
                                <span class="text-slate-400">"Created"</span>
                                <span class="text-slate-300 font-mono">"—"</span>
                            </div>
                            <div class="flex justify-between">
                                <span class="text-slate-400">"Assessor"</span>
                                <span class="text-slate-300 font-mono">"Not assigned"</span>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
