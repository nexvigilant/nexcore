//! Admin: My work — admin's assigned tasks, reviews, and content items

use leptos::prelude::*;

struct WorkItem {
    title: &'static str,
    item_type: &'static str,
    priority: &'static str,
    due: &'static str,
    status: &'static str,
}

const MY_ITEMS: &[WorkItem] = &[
    WorkItem {
        title: "Review: GVP Module IX Deep Dive",
        item_type: "Review",
        priority: "High",
        due: "Today",
        status: "In Progress",
    },
    WorkItem {
        title: "Review: Naranjo Causality Workshop",
        item_type: "Review",
        priority: "Medium",
        due: "Tomorrow",
        status: "Pending",
    },
    WorkItem {
        title: "Draft: PSUR Writing Masterclass",
        item_type: "Authoring",
        priority: "High",
        due: "Feb 18",
        status: "In Progress",
    },
    WorkItem {
        title: "Update: Signal Detection curriculum mapping",
        item_type: "Task",
        priority: "Low",
        due: "Feb 20",
        status: "Pending",
    },
    WorkItem {
        title: "Approve: Benefit-Risk Framework certificate template",
        item_type: "Approval",
        priority: "Medium",
        due: "Feb 17",
        status: "Pending",
    },
    WorkItem {
        title: "QC: MedDRA Coding quiz question bank",
        item_type: "QC",
        priority: "High",
        due: "Today",
        status: "Overdue",
    },
];

#[component]
pub fn AcademyMyWorkPage() -> impl IntoView {
    let assigned = MY_ITEMS.iter().filter(|i| i.status != "Completed").count();
    let in_progress = MY_ITEMS
        .iter()
        .filter(|i| i.status == "In Progress")
        .count();
    let overdue = MY_ITEMS.iter().filter(|i| i.status == "Overdue").count();

    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <div>
                <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"My Work"</h1>
                <p class="mt-1 text-slate-400">"Your assigned tasks, pending reviews, and content items."</p>
            </div>

            /* Stats */
            <div class="mt-6 grid gap-4 sm:grid-cols-3">
                <div class="rounded-xl border border-cyan-500/20 bg-cyan-500/5 p-5">
                    <p class="text-[9px] font-bold text-cyan-400 uppercase tracking-widest font-mono">"Assigned"</p>
                    <p class="text-2xl font-black text-cyan-400 font-mono mt-2">{assigned.to_string()}</p>
                </div>
                <div class="rounded-xl border border-amber-500/20 bg-amber-500/5 p-5">
                    <p class="text-[9px] font-bold text-amber-400 uppercase tracking-widest font-mono">"In Progress"</p>
                    <p class="text-2xl font-black text-amber-400 font-mono mt-2">{in_progress.to_string()}</p>
                </div>
                <div class="rounded-xl border border-red-500/20 bg-red-500/5 p-5">
                    <p class="text-[9px] font-bold text-red-400 uppercase tracking-widest font-mono">"Overdue"</p>
                    <p class="text-2xl font-black text-red-400 font-mono mt-2">{overdue.to_string()}</p>
                </div>
            </div>

            /* Work items */
            <div class="mt-8 space-y-3">
                {MY_ITEMS.iter().map(|item| {
                    let priority_cls = match item.priority {
                        "High" => "text-red-400 bg-red-500/10 border-red-500/20",
                        "Medium" => "text-amber-400 bg-amber-500/10 border-amber-500/20",
                        _ => "text-slate-400 bg-slate-500/10 border-slate-500/20",
                    };
                    let status_cls = match item.status {
                        "In Progress" => "text-cyan-400",
                        "Overdue" => "text-red-400",
                        _ => "text-slate-500",
                    };
                    let type_cls = match item.item_type {
                        "Review" => "text-violet-400 bg-violet-500/10 border-violet-500/20",
                        "Authoring" => "text-cyan-400 bg-cyan-500/10 border-cyan-500/20",
                        "Approval" => "text-emerald-400 bg-emerald-500/10 border-emerald-500/20",
                        "QC" => "text-amber-400 bg-amber-500/10 border-amber-500/20",
                        _ => "text-slate-400 bg-slate-500/10 border-slate-500/20",
                    };
                    let left_border = if item.status == "Overdue" { "border-l-red-500" } else if item.status == "In Progress" { "border-l-cyan-500" } else { "border-l-slate-600" };

                    view! {
                        <div class=format!("rounded-xl border border-slate-800 bg-slate-900/50 p-5 border-l-2 {left_border}")>
                            <div class="flex items-center justify-between">
                                <div class="flex items-center gap-3 flex-wrap">
                                    <span class=format!("rounded-full border px-2 py-0.5 text-[9px] font-bold font-mono uppercase {type_cls}")>{item.item_type}</span>
                                    <span class=format!("rounded-full border px-2 py-0.5 text-[9px] font-bold font-mono uppercase {priority_cls}")>{item.priority}</span>
                                    <h3 class="text-sm font-medium text-white">{item.title}</h3>
                                </div>
                                <div class="flex items-center gap-4 shrink-0">
                                    <span class=format!("text-xs font-bold font-mono {status_cls}")>{item.status}</span>
                                    <span class="text-[10px] text-slate-600 font-mono">{item.due}</span>
                                </div>
                            </div>
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}
