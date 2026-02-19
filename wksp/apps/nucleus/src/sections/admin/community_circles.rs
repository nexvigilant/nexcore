/*! Admin: Community circles administration — create, configure, manage discussion circles */

use leptos::prelude::*;

struct Circle {
    name: &'static str,
    members: u32,
    posts_week: u32,
    status: &'static str,
    category: &'static str,
    created: &'static str,
}

const CIRCLES: &[Circle] = &[
    Circle {
        name: "Signal Detection",
        members: 1240,
        posts_week: 87,
        status: "Active",
        category: "Technical",
        created: "2024-06-12",
    },
    Circle {
        name: "Regulatory EU",
        members: 850,
        posts_week: 42,
        status: "Active",
        category: "Regulatory",
        created: "2024-07-03",
    },
    Circle {
        name: "Career Transitions",
        members: 520,
        posts_week: 31,
        status: "Active",
        category: "Professional",
        created: "2024-08-19",
    },
    Circle {
        name: "ICSR Deep Dive",
        members: 310,
        posts_week: 24,
        status: "Active",
        category: "Technical",
        created: "2024-09-05",
    },
    Circle {
        name: "Benefit-Risk Assessment",
        members: 475,
        posts_week: 38,
        status: "Active",
        category: "Technical",
        created: "2024-10-14",
    },
    Circle {
        name: "FDA Compliance Updates",
        members: 690,
        posts_week: 19,
        status: "Active",
        category: "Regulatory",
        created: "2024-11-01",
    },
    Circle {
        name: "Legacy Systems",
        members: 120,
        posts_week: 0,
        status: "Archived",
        category: "Technical",
        created: "2024-06-20",
    },
    Circle {
        name: "Newcomer Onboarding",
        members: 95,
        posts_week: 3,
        status: "Draft",
        category: "General",
        created: "2025-01-08",
    },
];

#[component]
pub fn CommunityCirclesPage() -> impl IntoView {
    let total_circles = CIRCLES.len();
    let active_count = CIRCLES.iter().filter(|c| c.status == "Active").count();
    let total_members: u32 = CIRCLES.iter().map(|c| c.members).sum();
    let posts_this_week: u32 = CIRCLES.iter().map(|c| c.posts_week).sum();

    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            /* Header */
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">
                        "Circle Administration"
                    </h1>
                    <p class="mt-1 text-slate-400">
                        "Create, configure, and manage community circles."
                    </p>
                </div>
                <div class="flex gap-3 items-center">
                    <a
                        href="/admin/community"
                        class="text-sm text-slate-400 hover:text-white transition-colors font-mono"
                    >
                        "\u{2190} Back"
                    </a>
                    <button class="rounded-lg bg-cyan-600 px-4 py-2 text-sm font-bold text-white hover:bg-cyan-500 transition-colors font-mono uppercase tracking-widest">
                        "+ Create Circle"
                    </button>
                </div>
            </div>

            /* Stats row */
            <div class="mt-6 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5">
                    <p class="text-[9px] font-bold text-slate-500 uppercase tracking-widest font-mono">
                        "Total Circles"
                    </p>
                    <p class="text-2xl font-black text-white font-mono mt-2">
                        {total_circles.to_string()}
                    </p>
                </div>
                <div class="rounded-xl border border-emerald-500/20 bg-emerald-500/5 p-5">
                    <p class="text-[9px] font-bold text-emerald-400 uppercase tracking-widest font-mono">
                        "Active"
                    </p>
                    <p class="text-2xl font-black text-emerald-400 font-mono mt-2">
                        {active_count.to_string()}
                    </p>
                </div>
                <div class="rounded-xl border border-cyan-500/20 bg-cyan-500/5 p-5">
                    <p class="text-[9px] font-bold text-cyan-400 uppercase tracking-widest font-mono">
                        "Total Members"
                    </p>
                    <p class="text-2xl font-black text-cyan-400 font-mono mt-2">
                        {format!("{},{}",
                            total_members / 1000,
                            format!("{:03}", total_members % 1000)
                        )}
                    </p>
                </div>
                <div class="rounded-xl border border-violet-500/20 bg-violet-500/5 p-5">
                    <p class="text-[9px] font-bold text-violet-400 uppercase tracking-widest font-mono">
                        "Posts This Week"
                    </p>
                    <p class="text-2xl font-black text-violet-400 font-mono mt-2">
                        {posts_this_week.to_string()}
                    </p>
                </div>
            </div>

            /* Circle grid */
            <div class="mt-8 grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                {CIRCLES.iter().map(|circle| {
                    let (status_text, status_cls) = match circle.status {
                        "Active" => (
                            "Active",
                            "text-emerald-400 bg-emerald-500/10 border-emerald-500/20",
                        ),
                        "Archived" => (
                            "Archived",
                            "text-slate-500 bg-slate-800 border-slate-700",
                        ),
                        _ => (
                            "Draft",
                            "text-amber-400 bg-amber-500/10 border-amber-500/20",
                        ),
                    };

                    let category_cls = match circle.category {
                        "Technical" => "text-cyan-400",
                        "Regulatory" => "text-violet-400",
                        "Professional" => "text-amber-400",
                        _ => "text-slate-400",
                    };

                    let border_accent = match circle.status {
                        "Active" => "hover:border-emerald-500/30",
                        "Archived" => "hover:border-slate-600",
                        _ => "hover:border-amber-500/30",
                    };

                    let archive_label = match circle.status {
                        "Archived" => "Restore",
                        _ => "Archive",
                    };

                    view! {
                        <div class=format!(
                            "rounded-xl border border-slate-800 bg-slate-900/50 p-6 transition-colors {}",
                            border_accent
                        )>
                            /* Status + category row */
                            <div class="flex items-center justify-between mb-3">
                                <span class=format!(
                                    "rounded-full px-2 py-0.5 text-[9px] font-bold uppercase border {}",
                                    status_cls
                                )>
                                    {status_text}
                                </span>
                                <span class=format!(
                                    "text-[10px] font-mono uppercase tracking-widest {}",
                                    category_cls
                                )>
                                    {circle.category}
                                </span>
                            </div>

                            /* Name */
                            <h3 class="text-lg font-bold text-white mb-2">{circle.name}</h3>

                            /* Stats row within card */
                            <div class="flex items-center gap-4 mb-1">
                                <div class="flex items-center gap-1.5">
                                    <span class="text-[10px] text-slate-600 font-mono uppercase tracking-widest">
                                        "Members"
                                    </span>
                                    <span class="text-sm font-bold text-slate-300 font-mono">
                                        {circle.members.to_string()}
                                    </span>
                                </div>
                                <div class="flex items-center gap-1.5">
                                    <span class="text-[10px] text-slate-600 font-mono uppercase tracking-widest">
                                        "Posts/wk"
                                    </span>
                                    <span class="text-sm font-bold text-slate-300 font-mono">
                                        {circle.posts_week.to_string()}
                                    </span>
                                </div>
                            </div>

                            /* Created date */
                            <p class="text-[10px] text-slate-600 font-mono mt-1">
                                "Created " {circle.created}
                            </p>

                            /* Action buttons */
                            <div class="mt-4 flex gap-2">
                                <button class="flex-1 rounded border border-slate-700 py-1.5 text-[10px] font-bold text-slate-400 hover:text-white hover:border-slate-500 transition-colors uppercase tracking-widest font-mono">
                                    "Edit"
                                </button>
                                <button class="flex-1 rounded border border-slate-700 py-1.5 text-[10px] font-bold text-slate-400 hover:text-white hover:border-slate-500 transition-colors uppercase tracking-widest font-mono">
                                    "Members"
                                </button>
                                <button class="flex-1 rounded border border-slate-700 py-1.5 text-[10px] font-bold text-red-400/50 hover:text-red-400 hover:border-red-500/30 transition-colors uppercase tracking-widest font-mono">
                                    {archive_label}
                                </button>
                            </div>
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}
