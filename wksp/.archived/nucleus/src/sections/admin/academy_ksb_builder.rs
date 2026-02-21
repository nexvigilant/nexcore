//! Admin: KSB Builder — create and edit Knowledge, Skills, and Behaviours

use leptos::prelude::*;

#[component]
pub fn AcademyKsbBuilderPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8 space-y-8">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"KSB Builder"</h1>
                    <p class="mt-1 text-slate-400">"Create and edit Knowledge, Skills, and Behaviours mapped to competency framework"</p>
                </div>
                <a href="/admin/academy/ksb-builder/review" class="rounded-lg border border-slate-700 px-3 py-2 text-xs font-bold text-slate-400 hover:text-white hover:border-slate-600 transition-colors">"REVIEW QUEUE"</a>
            </div>
            <div class="bg-slate-800/50 border border-slate-700/50 rounded-xl p-6 space-y-4">
                <div>
                    <label class="text-sm font-semibold text-white block mb-2">"KSB Type"</label>
                    <div class="flex gap-3">
                        <button class="px-4 py-2 bg-cyan-600/20 text-cyan-400 border border-cyan-600/30 rounded-lg text-sm font-medium">"Knowledge"</button>
                        <button class="px-4 py-2 text-slate-400 border border-slate-700/50 rounded-lg text-sm font-medium hover:text-white transition-colors">"Skill"</button>
                        <button class="px-4 py-2 text-slate-400 border border-slate-700/50 rounded-lg text-sm font-medium hover:text-white transition-colors">"Behaviour"</button>
                    </div>
                </div>
                <div>
                    <label class="text-sm font-semibold text-white block mb-2">"Title"</label>
                    <input type="text" class="w-full bg-slate-900/50 border border-slate-700/50 rounded-lg px-4 py-2 text-white placeholder-slate-500 focus:outline-none focus:border-cyan-500/50" placeholder="e.g., Understand disproportionality analysis methods"/>
                </div>
                <div>
                    <label class="text-sm font-semibold text-white block mb-2">"Description"</label>
                    <textarea class="w-full bg-slate-900/50 border border-slate-700/50 rounded-lg px-4 py-3 text-white placeholder-slate-500 focus:outline-none focus:border-cyan-500/50 min-h-[100px] resize-none" placeholder="Detailed description of this KSB..."/>
                </div>
                <div>
                    <label class="text-sm font-semibold text-white block mb-2">"Linked CPA"</label>
                    <select class="w-full bg-slate-900/50 border border-slate-700/50 rounded-lg px-4 py-2 text-white focus:outline-none focus:border-cyan-500/50">
                        <option>"Select a CPA..."</option>
                        <option>"CPA 1.1.1: Run disproportionality analysis"</option>
                        <option>"CPA 1.1.2: Interpret signal metrics"</option>
                        <option>"CPA 1.2.1: Assess clinical significance"</option>
                    </select>
                </div>
                <div>
                    <label class="text-sm font-semibold text-white block mb-2">"Bloom's Level"</label>
                    <select class="w-full bg-slate-900/50 border border-slate-700/50 rounded-lg px-4 py-2 text-white focus:outline-none focus:border-cyan-500/50">
                        <option>"Remember"</option>
                        <option>"Understand"</option>
                        <option>"Apply"</option>
                        <option>"Analyze"</option>
                        <option>"Evaluate"</option>
                        <option>"Create"</option>
                    </select>
                </div>
                <div class="flex justify-end gap-3 pt-2">
                    <button class="px-4 py-2 border border-slate-700 text-slate-400 hover:text-white rounded-lg text-sm font-medium transition-colors">"Save Draft"</button>
                    <button class="px-6 py-2 bg-cyan-600 hover:bg-cyan-500 text-white rounded-lg text-sm font-medium transition-colors">"Create KSB"</button>
                </div>
            </div>
        </div>
    }
}
