//! Create circle page — form to create a new community circle

use leptos::prelude::*;

#[component]
pub fn CreateCirclePage() -> impl IntoView {
    view! {
        <div class="max-w-2xl mx-auto space-y-6">
            <div>
                <a href="/community/circles" class="text-cyan-400 hover:text-cyan-300 text-sm">"Back to Circles"</a>
                <h1 class="text-2xl font-bold text-white mt-2">"Create a Circle"</h1>
                <p class="text-slate-400 mt-1">"Start a focused discussion group around a pharmacovigilance topic"</p>
            </div>

            <div class="bg-slate-800/50 border border-slate-700/50 rounded-xl p-6 space-y-5">
                <div>
                    <label class="text-sm font-semibold text-white block mb-2">"Circle Name"</label>
                    <input
                        type="text"
                        class="w-full bg-slate-900/50 border border-slate-700/50 rounded-lg px-4 py-2 text-white placeholder-slate-500 focus:outline-none focus:border-cyan-500/50"
                        placeholder="e.g., Signal Detection Methods"
                    />
                </div>

                <div>
                    <label class="text-sm font-semibold text-white block mb-2">"Description"</label>
                    <textarea
                        class="w-full bg-slate-900/50 border border-slate-700/50 rounded-lg px-4 py-3 text-white placeholder-slate-500 focus:outline-none focus:border-cyan-500/50 min-h-[100px] resize-none"
                        placeholder="What is this circle about? What topics will be discussed?"
                    />
                </div>

                <div>
                    <label class="text-sm font-semibold text-white block mb-2">"Category"</label>
                    <select class="w-full bg-slate-900/50 border border-slate-700/50 rounded-lg px-4 py-2 text-white focus:outline-none focus:border-cyan-500/50">
                        <option>"Signal Detection"</option>
                        <option>"Case Processing"</option>
                        <option>"Regulatory Affairs"</option>
                        <option>"Risk Management"</option>
                        <option>"PV Technology"</option>
                        <option>"Career Development"</option>
                        <option>"General Discussion"</option>
                    </select>
                </div>

                <div>
                    <label class="text-sm font-semibold text-white block mb-2">"Visibility"</label>
                    <div class="space-y-2">
                        <label class="flex items-center gap-3 p-3 rounded-lg border border-slate-700/50 cursor-pointer hover:border-slate-600/50">
                            <input type="radio" name="visibility" value="public" checked class="accent-cyan-500"/>
                            <div>
                                <span class="text-white text-sm font-medium">"Public"</span>
                                <p class="text-xs text-slate-400">"Anyone can see and join"</p>
                            </div>
                        </label>
                        <label class="flex items-center gap-3 p-3 rounded-lg border border-slate-700/50 cursor-pointer hover:border-slate-600/50">
                            <input type="radio" name="visibility" value="private" class="accent-cyan-500"/>
                            <div>
                                <span class="text-white text-sm font-medium">"Private"</span>
                                <p class="text-xs text-slate-400">"Invite only, content hidden from non-members"</p>
                            </div>
                        </label>
                    </div>
                </div>

                <div>
                    <label class="text-sm font-semibold text-white block mb-2">"Guidelines"</label>
                    <textarea
                        class="w-full bg-slate-900/50 border border-slate-700/50 rounded-lg px-4 py-3 text-white placeholder-slate-500 focus:outline-none focus:border-cyan-500/50 min-h-[80px] resize-none"
                        placeholder="Optional: Set community guidelines for this circle"
                    />
                </div>

                <div class="flex justify-end gap-3 pt-2">
                    <a href="/community/circles" class="px-4 py-2 border border-slate-700 text-slate-400 hover:text-white rounded-lg text-sm font-medium transition-colors">
                        "Cancel"
                    </a>
                    <button class="px-6 py-2 bg-cyan-600 hover:bg-cyan-500 text-white rounded-lg text-sm font-medium transition-colors">
                        "Create Circle"
                    </button>
                </div>
            </div>
        </div>
    }
}
