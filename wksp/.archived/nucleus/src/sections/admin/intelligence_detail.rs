//! Admin: Intelligence signal detail view

use crate::sections::intelligence_data::article_by_slug;
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

#[component]
pub fn IntelligenceDetailPage() -> impl IntoView {
    let params = use_params_map();
    let slug = move || params.get().get("slug").unwrap_or_default();

    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Signal Detail"</h1>
                    <p class="mt-1 text-slate-400">"Deep-dive view of detected signal with evidence trail."</p>
                </div>
                <a href="/admin/intelligence" class="text-sm text-slate-400 hover:text-white transition-colors font-mono">"\u{2190} Intelligence"</a>
            </div>

            <div class="mt-8 grid gap-6 lg:grid-cols-3">
                <div class="lg:col-span-2 space-y-6">
                    <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                        <h3 class="text-sm font-bold text-white font-mono uppercase tracking-widest mb-4">"Signal Summary"</h3>
                        {move || {
                            let article = article_by_slug(&slug());
                            match article {
                                Some(a) => view! {
                                    <div class="space-y-4">
                                        <div class="grid grid-cols-2 gap-4">
                                            <div>
                                                <p class="text-[9px] font-bold text-slate-500 uppercase tracking-widest font-mono">"Title"</p>
                                                <p class="text-sm text-white mt-1">{a.title}</p>
                                            </div>
                                            <div>
                                                <p class="text-[9px] font-bold text-slate-500 uppercase tracking-widest font-mono">"Category"</p>
                                                <p class="text-sm text-white mt-1">{a.category}</p>
                                            </div>
                                            <div>
                                                <p class="text-[9px] font-bold text-slate-500 uppercase tracking-widest font-mono">"Published"</p>
                                                <p class="text-sm text-white mt-1">{a.date}</p>
                                            </div>
                                            <div>
                                                <p class="text-[9px] font-bold text-slate-500 uppercase tracking-widest font-mono">"Read Time"</p>
                                                <p class="text-sm text-white mt-1">{a.read_time}</p>
                                            </div>
                                        </div>
                                        <p class="text-sm text-slate-300 leading-relaxed">{a.excerpt}</p>
                                    </div>
                                }.into_any(),
                                None => view! {
                                    <p class="text-sm text-red-400">"No intelligence entry found for this slug."</p>
                                }.into_any(),
                            }
                        }}
                    </div>

                    <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                        <h3 class="text-sm font-bold text-white font-mono uppercase tracking-widest mb-4">"Narrative Excerpt"</h3>
                        <div class="grid grid-cols-3 gap-4">
                            {move || {
                                let article = article_by_slug(&slug());
                                match article {
                                    Some(a) => a.body.iter().take(3).map(|p| view! {
                                        <div class="bg-slate-950/50 rounded-lg border border-slate-800 p-3">
                                            <p class="text-sm text-slate-300 leading-relaxed">{*p}</p>
                                        </div>
                                    }).collect_view().into_any(),
                                    None => view! {
                                        <div class="bg-slate-950/50 rounded-lg border border-slate-800 p-3">
                                            <p class="text-sm text-slate-500">"No narrative available."</p>
                                        </div>
                                    }.into_any(),
                                }
                            }}
                        </div>
                    </div>
                </div>

                <div class="space-y-6">
                    <div class="rounded-xl border border-red-500/20 bg-red-500/5 p-6">
                        <h3 class="text-sm font-bold text-red-400 font-mono uppercase tracking-widest mb-2">"Risk Assessment"</h3>
                        <p class="text-3xl font-black text-red-400 font-mono">"HIGH"</p>
                        <p class="text-xs text-slate-400 mt-2">"Multiple algorithms confirm disproportionality. Review recommended."</p>
                    </div>

                    <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                        <h3 class="text-sm font-bold text-white font-mono uppercase tracking-widest mb-4">"Actions"</h3>
                        <div class="space-y-2">
                            <button class="w-full rounded-lg bg-amber-600 px-4 py-2 text-xs font-bold text-white hover:bg-amber-500 transition-colors uppercase tracking-widest">"Escalate to QPPV"</button>
                            <button class="w-full rounded-lg border border-slate-700 px-4 py-2 text-xs font-bold text-slate-400 hover:text-white transition-colors uppercase tracking-widest">"Generate Report"</button>
                            <button class="w-full rounded-lg border border-slate-700 px-4 py-2 text-xs font-bold text-slate-400 hover:text-white transition-colors uppercase tracking-widest">"Mark Reviewed"</button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
