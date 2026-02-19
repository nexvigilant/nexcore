//! Admin: Academy content pipeline — content hierarchy and authoring workflow

use leptos::prelude::*;

struct ContentItem {
    title: &'static str,
    author: &'static str,
    stage: &'static str,
    domain: &'static str,
    updated: &'static str,
    words: u32,
}

const ITEMS: &[ContentItem] = &[
    ContentItem {
        title: "Advanced Signal Detection Methods",
        author: "Dr. Elena Vasquez",
        stage: "Draft",
        domain: "Signal Detection",
        updated: "2h ago",
        words: 4200,
    },
    ContentItem {
        title: "PBRER Authoring Guide",
        author: "James Okonkwo",
        stage: "Draft",
        domain: "Aggregate Reporting",
        updated: "1d ago",
        words: 6800,
    },
    ContentItem {
        title: "MedDRA Coding Best Practices",
        author: "Aisha Patel",
        stage: "Draft",
        domain: "Case Processing",
        updated: "3d ago",
        words: 3100,
    },
    ContentItem {
        title: "GVP Module IX Deep Dive",
        author: "Dr. Thomas Richter",
        stage: "Review",
        domain: "Signal Detection",
        updated: "4h ago",
        words: 8500,
    },
    ContentItem {
        title: "Naranjo Causality Workshop",
        author: "Dr. Mei-Lin Chen",
        stage: "Review",
        domain: "Case Processing",
        updated: "1d ago",
        words: 5200,
    },
    ContentItem {
        title: "Risk Management Plans 101",
        author: "Sarah Williams",
        stage: "Revision",
        domain: "Risk Management",
        updated: "6h ago",
        words: 7100,
    },
    ContentItem {
        title: "Benefit-Risk Framework Intro",
        author: "Dr. Ahmed Hassan",
        stage: "Approved",
        domain: "Benefit-Risk",
        updated: "2d ago",
        words: 4800,
    },
    ContentItem {
        title: "Introduction to Pharmacovigilance",
        author: "Dr. Sarah Mitchell",
        stage: "Published",
        domain: "Foundations",
        updated: "1w ago",
        words: 12400,
    },
];

#[component]
pub fn AcademyContentPipelinePage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Content Pipeline"</h1>
                    <p class="mt-1 text-slate-400">"Manage the content authoring pipeline from draft to publication."</p>
                </div>
                <button class="rounded-lg bg-cyan-600 px-4 py-2 text-xs font-bold text-white hover:bg-cyan-500 transition-colors font-mono uppercase">"+ New Content"</button>
            </div>

            /* Pipeline stage cards */
            <div class="mt-6 grid gap-4 sm:grid-cols-5">
                {[("Draft", "3", "text-slate-400 border-slate-600 bg-slate-500/5"),
                  ("Review", "2", "text-amber-400 border-amber-500/20 bg-amber-500/5"),
                  ("Revision", "1", "text-red-400 border-red-500/20 bg-red-500/5"),
                  ("Approved", "1", "text-emerald-400 border-emerald-500/20 bg-emerald-500/5"),
                  ("Published", "1", "text-cyan-400 border-cyan-500/20 bg-cyan-500/5")]
                    .into_iter().map(|(stage, count, cls)| view! {
                        <div class=format!("rounded-xl border p-5 {cls}")>
                            <p class="text-[9px] font-bold uppercase tracking-widest font-mono">{stage}</p>
                            <p class="text-2xl font-black font-mono mt-2">{count}</p>
                        </div>
                    }).collect_view()}
            </div>

            /* Content table */
            <div class="mt-8 rounded-xl border border-slate-800 overflow-hidden">
                <table class="w-full text-left text-sm">
                    <thead class="bg-slate-900/80 text-[10px] uppercase text-slate-500 font-mono tracking-widest">
                        <tr>
                            <th class="px-4 py-3">"Title"</th>
                            <th class="px-4 py-3">"Author"</th>
                            <th class="px-4 py-3">"Domain"</th>
                            <th class="px-4 py-3 text-right">"Words"</th>
                            <th class="px-4 py-3">"Stage"</th>
                            <th class="px-4 py-3">"Updated"</th>
                        </tr>
                    </thead>
                    <tbody class="text-slate-300">
                        {ITEMS.iter().map(|item| {
                            let stage_cls = match item.stage {
                                "Draft" => "text-slate-400 bg-slate-500/10 border-slate-500/20",
                                "Review" => "text-amber-400 bg-amber-500/10 border-amber-500/20",
                                "Revision" => "text-red-400 bg-red-500/10 border-red-500/20",
                                "Approved" => "text-emerald-400 bg-emerald-500/10 border-emerald-500/20",
                                "Published" => "text-cyan-400 bg-cyan-500/10 border-cyan-500/20",
                                _ => "text-slate-400 bg-slate-500/10 border-slate-500/20",
                            };
                            view! {
                                <tr class="border-t border-slate-800 hover:bg-slate-800/30 transition-colors">
                                    <td class="px-4 py-3 text-sm font-medium text-white">{item.title}</td>
                                    <td class="px-4 py-3 text-xs text-slate-400">{item.author}</td>
                                    <td class="px-4 py-3 text-xs text-slate-500 font-mono">{item.domain}</td>
                                    <td class="px-4 py-3 text-xs text-slate-500 font-mono text-right">{item.words.to_string()}</td>
                                    <td class="px-4 py-3">
                                        <span class=format!("rounded-full border px-2 py-0.5 text-[9px] font-bold font-mono uppercase {stage_cls}")>
                                            {item.stage}
                                        </span>
                                    </td>
                                    <td class="px-4 py-3 text-[10px] text-slate-600 font-mono">{item.updated}</td>
                                </tr>
                            }
                        }).collect_view()}
                    </tbody>
                </table>
            </div>

            <div class="mt-4 text-[10px] text-slate-600 font-mono">
                {format!("{} items in pipeline", ITEMS.len())}
            </div>
        </div>
    }
}
