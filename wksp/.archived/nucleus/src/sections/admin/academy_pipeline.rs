//! Admin: Academy content pipeline — course authoring workflow stages

use leptos::prelude::*;

struct PipelineItem {
    title: &'static str,
    author: &'static str,
    stage: &'static str,
    priority: &'static str,
    days_in_stage: u32,
}

const PIPELINE: &[PipelineItem] = &[
    PipelineItem {
        title: "Advanced Signal Detection Methods",
        author: "Dr. Elena Vasquez",
        stage: "Draft",
        priority: "High",
        days_in_stage: 3,
    },
    PipelineItem {
        title: "PBRER Authoring Guide",
        author: "James Okonkwo",
        stage: "Draft",
        priority: "Medium",
        days_in_stage: 7,
    },
    PipelineItem {
        title: "GVP Module IX Deep Dive",
        author: "Dr. Thomas Richter",
        stage: "Review",
        priority: "High",
        days_in_stage: 2,
    },
    PipelineItem {
        title: "Naranjo Causality Workshop",
        author: "Dr. Mei-Lin Chen",
        stage: "Review",
        priority: "Medium",
        days_in_stage: 1,
    },
    PipelineItem {
        title: "Risk Management Plans 101",
        author: "Sarah Williams",
        stage: "Revision",
        priority: "Low",
        days_in_stage: 4,
    },
    PipelineItem {
        title: "Benefit-Risk Framework Intro",
        author: "Dr. Ahmed Hassan",
        stage: "Approved",
        priority: "Medium",
        days_in_stage: 1,
    },
    PipelineItem {
        title: "Introduction to Pharmacovigilance",
        author: "Dr. Sarah Mitchell",
        stage: "Published",
        priority: "Low",
        days_in_stage: 0,
    },
    PipelineItem {
        title: "MedDRA Coding Best Practices",
        author: "Aisha Patel",
        stage: "Draft",
        priority: "High",
        days_in_stage: 5,
    },
];

#[component]
pub fn AcademyPipelinePage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Content Pipeline"</h1>
                    <p class="mt-1 text-slate-400">"Track course content through authoring, review, and publication stages."</p>
                </div>
                <a href="/admin/academy/courses/new" class="rounded-lg bg-cyan-600 px-4 py-2 text-xs font-bold text-white hover:bg-cyan-500 transition-colors font-mono uppercase">"Create Course"</a>
            </div>

            /* Stage cards */
            <div class="mt-6 grid gap-4 sm:grid-cols-5">
                {[("Draft", 3u32, "border-slate-600 bg-slate-500/5 text-slate-400"),
                  ("Review", 2, "border-amber-500/30 bg-amber-500/5 text-amber-400"),
                  ("Revision", 1, "border-red-500/30 bg-red-500/5 text-red-400"),
                  ("Approved", 1, "border-emerald-500/30 bg-emerald-500/5 text-emerald-400"),
                  ("Published", 1, "border-cyan-500/30 bg-cyan-500/5 text-cyan-400")]
                    .into_iter().map(|(stage, count, cls)| view! {
                        <div class=format!("rounded-xl border p-4 text-center {cls}")>
                            <p class="text-2xl font-bold font-mono">{count.to_string()}</p>
                            <p class="text-[10px] font-bold uppercase tracking-widest font-mono mt-1">{stage}</p>
                        </div>
                    }).collect_view()}
            </div>

            /* Pipeline items */
            <div class="mt-8 space-y-2">
                <h2 class="text-sm font-bold uppercase tracking-widest text-slate-500 font-mono mb-4">"Pipeline Items"</h2>
                {PIPELINE.iter().map(|item| {
                    let stage_cls = match item.stage {
                        "Draft" => "text-slate-400 bg-slate-500/10 border-slate-500/20",
                        "Review" => "text-amber-400 bg-amber-500/10 border-amber-500/20",
                        "Revision" => "text-red-400 bg-red-500/10 border-red-500/20",
                        "Approved" => "text-emerald-400 bg-emerald-500/10 border-emerald-500/20",
                        "Published" => "text-cyan-400 bg-cyan-500/10 border-cyan-500/20",
                        _ => "text-slate-400 bg-slate-500/10 border-slate-500/20",
                    };
                    let priority_cls = match item.priority {
                        "High" => "text-red-400",
                        "Medium" => "text-amber-400",
                        _ => "text-slate-500",
                    };
                    view! {
                        <div class="flex items-center justify-between rounded-xl border border-slate-800 bg-slate-900/50 p-4 hover:border-slate-600 transition-colors">
                            <div class="flex items-center gap-4">
                                <span class=format!("rounded-full border px-2 py-0.5 text-[9px] font-bold font-mono uppercase {stage_cls}")>{item.stage}</span>
                                <div>
                                    <p class="text-sm font-medium text-white">{item.title}</p>
                                    <p class="text-[10px] text-slate-500 font-mono mt-0.5">{item.author}</p>
                                </div>
                            </div>
                            <div class="flex items-center gap-4 shrink-0">
                                <span class=format!("text-[10px] font-bold font-mono {priority_cls}")>{item.priority}</span>
                                <span class="text-[10px] text-slate-600 font-mono">{format!("{}d in stage", item.days_in_stage)}</span>
                            </div>
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}
