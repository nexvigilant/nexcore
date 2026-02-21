//! Admin: Create new intelligence signal or report

use leptos::prelude::*;

#[component]
pub fn IntelligenceNewPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-3xl px-4 py-8">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"New Intelligence Entry"</h1>
                    <p class="mt-1 text-slate-400">"Create a manual signal report or intelligence article."</p>
                </div>
                <a href="/admin/intelligence" class="text-sm text-slate-400 hover:text-white transition-colors font-mono">"\u{2190} Intelligence"</a>
            </div>

            <div class="mt-8 rounded-xl border border-slate-800 bg-slate-900/50 p-8 space-y-6">
                <div>
                    <label class="block text-[10px] font-bold text-slate-500 uppercase tracking-widest font-mono mb-2">"Type"</label>
                    <div class="flex gap-3">
                        {["Signal Report", "Literature Review", "Regulatory Update", "Safety Communication"]
                            .into_iter().enumerate().map(|(i, label)| view! {
                                <button class=if i == 0 {
                                    "rounded-lg bg-cyan-600 px-4 py-2 text-xs font-bold text-white"
                                } else {
                                    "rounded-lg border border-slate-700 px-4 py-2 text-xs font-bold text-slate-400 hover:text-white transition-colors"
                                }>{label}</button>
                            }).collect::<Vec<_>>()}
                    </div>
                </div>

                <div>
                    <label class="block text-[10px] font-bold text-slate-500 uppercase tracking-widest font-mono mb-2">"Title"</label>
                    <input type_="text" class="w-full bg-slate-950 border border-slate-800 rounded-lg px-4 py-3 text-sm text-white focus:border-cyan-500 focus:outline-none" placeholder="Signal title or report name" />
                </div>

                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label class="block text-[10px] font-bold text-slate-500 uppercase tracking-widest font-mono mb-2">"Drug / Product"</label>
                        <input type_="text" class="w-full bg-slate-950 border border-slate-800 rounded-lg px-4 py-3 text-sm text-white focus:border-cyan-500 focus:outline-none" placeholder="e.g. Atorvastatin" />
                    </div>
                    <div>
                        <label class="block text-[10px] font-bold text-slate-500 uppercase tracking-widest font-mono mb-2">"Adverse Event"</label>
                        <input type_="text" class="w-full bg-slate-950 border border-slate-800 rounded-lg px-4 py-3 text-sm text-white focus:border-cyan-500 focus:outline-none" placeholder="MedDRA PT" />
                    </div>
                </div>

                <div>
                    <label class="block text-[10px] font-bold text-slate-500 uppercase tracking-widest font-mono mb-2">"Description"</label>
                    <textarea class="w-full bg-slate-950 border border-slate-800 rounded-lg px-4 py-3 text-sm text-white focus:border-cyan-500 focus:outline-none h-32 resize-none" placeholder="Describe the signal, evidence, and context..."></textarea>
                </div>

                <div>
                    <label class="block text-[10px] font-bold text-slate-500 uppercase tracking-widest font-mono mb-2">"Priority"</label>
                    <select class="bg-slate-950 border border-slate-800 rounded-lg px-4 py-3 text-sm text-white focus:border-cyan-500 focus:outline-none">
                        <option>"Low"</option>
                        <option>"Medium"</option>
                        <option selected>"High"</option>
                        <option>"Critical"</option>
                    </select>
                </div>

                <div class="flex justify-end gap-3 pt-4 border-t border-slate-800">
                    <a href="/admin/intelligence" class="rounded-lg border border-slate-700 px-6 py-2.5 text-xs font-bold text-slate-400 hover:text-white transition-colors uppercase tracking-widest">"Cancel"</a>
                    <button class="rounded-lg bg-cyan-600 px-6 py-2.5 text-xs font-bold text-white hover:bg-cyan-500 transition-colors uppercase tracking-widest">"Create Entry"</button>
                </div>
            </div>
        </div>
    }
}
