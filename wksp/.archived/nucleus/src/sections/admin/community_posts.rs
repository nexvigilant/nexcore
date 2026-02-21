/*! Admin: Community posts management — full moderation table with mock data */

use leptos::prelude::*;

struct Post {
    author: &'static str,
    circle: &'static str,
    preview: &'static str,
    likes: u32,
    replies: u32,
    time_ago: &'static str,
    status: &'static str,
}

const POSTS: &[Post] = &[
    Post {
        author: "Dr. Elena Vasquez",
        circle: "Signal Detection",
        preview: "New approach to using EBGM alongside traditional PRR for early signal identification in spontaneous reporting systems...",
        likes: 24,
        replies: 8,
        time_ago: "2h ago",
        status: "published",
    },
    Post {
        author: "James Okonkwo",
        circle: "Regulatory Updates",
        preview: "Summary of the latest EMA GVP Module IX revision changes and impact on signal management workflows across MAHs...",
        likes: 18,
        replies: 5,
        time_ago: "4h ago",
        status: "published",
    },
    Post {
        author: "Aisha Patel",
        circle: "Case Processing",
        preview: "Tips for improving MedDRA coding consistency across large distributed teams using standardized SOPs and QC checklists...",
        likes: 31,
        replies: 12,
        time_ago: "1d ago",
        status: "pinned",
    },
    Post {
        author: "anon_user_42",
        circle: "General Discussion",
        preview: "Has anyone else noticed issues with the FAERS data quality for Q4 2025? Missing demographics on ~12% of reports.",
        likes: 7,
        replies: 3,
        time_ago: "5h ago",
        status: "flagged",
    },
    Post {
        author: "Dr. Thomas Richter",
        circle: "Benefit-Risk",
        preview: "Framework comparison: EMA vs FDA approaches to benefit-risk assessment methodology — key divergence points and convergence trends...",
        likes: 42,
        replies: 15,
        time_ago: "1d ago",
        status: "published",
    },
    Post {
        author: "new_member_19",
        circle: "Career Advice",
        preview: "Looking for guidance on transitioning from clinical research to pharmacovigilance. What certifications matter most?",
        likes: 9,
        replies: 6,
        time_ago: "2d ago",
        status: "published",
    },
    Post {
        author: "trial_user_88",
        circle: "General Discussion",
        preview: "Check out this amazing PV tool [promotional link removed] — best aggregate analysis I have ever seen!!!",
        likes: 0,
        replies: 0,
        time_ago: "3d ago",
        status: "hidden",
    },
    Post {
        author: "Dr. Mei-Lin Chen",
        circle: "Signal Detection",
        preview: "Tutorial: Setting up automated signal detection using open-source tools and ROR/PRR with configurable thresholds...",
        likes: 56,
        replies: 22,
        time_ago: "3d ago",
        status: "pinned",
    },
    Post {
        author: "spam_bot_007",
        circle: "Regulatory Updates",
        preview: "URGENT: Buy cheap compliance certificates online! Fast delivery guaranteed [SPAM]",
        likes: 0,
        replies: 1,
        time_ago: "4d ago",
        status: "hidden",
    },
    Post {
        author: "Dr. Sarah Kimura",
        circle: "Benefit-Risk",
        preview: "Warning: Potential unreported hepatotoxicity cluster in post-market data for compound XR-4412. Three ICSRs flagged internally...",
        likes: 3,
        replies: 2,
        time_ago: "6h ago",
        status: "flagged",
    },
];

fn count_by_status(status: &str) -> usize {
    POSTS.iter().filter(|p| p.status == status).count()
}

#[component]
pub fn CommunityPostsPage() -> impl IntoView {
    let filter = RwSignal::new("all");

    let total = POSTS.len();
    let published = count_by_status("published");
    let flagged = count_by_status("flagged");
    let hidden = count_by_status("hidden");
    let pinned = count_by_status("pinned");

    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            /* Header */
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Post Management"</h1>
                    <p class="mt-1 text-slate-400 text-sm">"Review, moderate, and manage community posts across all circles."</p>
                </div>
                <a
                    href="/admin/community"
                    class="text-sm text-slate-400 hover:text-white transition-colors font-mono"
                >"\u{2190} Community Admin"</a>
            </div>

            /* Stats row */
            <div class="mt-6 grid gap-4 sm:grid-cols-4">
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5">
                    <p class="text-[9px] font-bold text-slate-500 uppercase tracking-widest font-mono">"Total Posts"</p>
                    <p class="text-2xl font-black text-white font-mono mt-2">{total.to_string()}</p>
                </div>
                <div class="rounded-xl border border-emerald-500/20 bg-emerald-500/5 p-5">
                    <p class="text-[9px] font-bold text-emerald-400 uppercase tracking-widest font-mono">"Published"</p>
                    <p class="text-2xl font-black text-emerald-400 font-mono mt-2">{published.to_string()}</p>
                </div>
                <div class="rounded-xl border border-amber-500/20 bg-amber-500/5 p-5">
                    <p class="text-[9px] font-bold text-amber-400 uppercase tracking-widest font-mono">"Flagged"</p>
                    <p class="text-2xl font-black text-amber-400 font-mono mt-2">{flagged.to_string()}</p>
                </div>
                <div class="rounded-xl border border-red-500/20 bg-red-500/5 p-5">
                    <p class="text-[9px] font-bold text-red-400 uppercase tracking-widest font-mono">"Hidden"</p>
                    <p class="text-2xl font-black text-red-400 font-mono mt-2">{hidden.to_string()}</p>
                </div>
            </div>

            /* Filter tabs */
            <div class="mt-8 flex gap-2 border-b border-slate-800 pb-3">
                {[
                    ("all", total),
                    ("published", published),
                    ("pinned", pinned),
                    ("flagged", flagged),
                    ("hidden", hidden),
                ].into_iter().map(|(tab, count)| {
                    let label = format!("{tab} ({count})");
                    view! {
                        <button
                            on:click=move |_| filter.set(tab)
                            class=move || if filter.get() == tab {
                                "rounded-lg px-4 py-1.5 text-[10px] font-bold text-cyan-400 bg-cyan-500/10 font-mono uppercase tracking-widest"
                            } else {
                                "rounded-lg px-4 py-1.5 text-[10px] font-medium text-slate-500 hover:text-white font-mono uppercase tracking-widest transition-colors"
                            }
                        >{label}</button>
                    }
                }).collect_view()}
            </div>

            /* Post table header */
            <div class="mt-6 hidden sm:grid grid-cols-12 gap-4 px-5 py-2">
                <span class="col-span-1 text-[9px] font-bold text-slate-600 uppercase tracking-widest font-mono">"Status"</span>
                <span class="col-span-2 text-[9px] font-bold text-slate-600 uppercase tracking-widest font-mono">"Author"</span>
                <span class="col-span-2 text-[9px] font-bold text-slate-600 uppercase tracking-widest font-mono">"Circle"</span>
                <span class="col-span-3 text-[9px] font-bold text-slate-600 uppercase tracking-widest font-mono">"Preview"</span>
                <span class="col-span-1 text-[9px] font-bold text-slate-600 uppercase tracking-widest font-mono text-right">"Engage"</span>
                <span class="col-span-1 text-[9px] font-bold text-slate-600 uppercase tracking-widest font-mono text-right">"Age"</span>
                <span class="col-span-2 text-[9px] font-bold text-slate-600 uppercase tracking-widest font-mono text-right">"Actions"</span>
            </div>

            /* Post rows */
            <div class="mt-1 space-y-2">
                {POSTS.iter().map(|post| {
                    let (status_text_cls, status_bg_cls) = match post.status {
                        "published" => ("text-emerald-400", "bg-emerald-500/10 border-emerald-500/20"),
                        "pinned"    => ("text-cyan-400",    "bg-cyan-500/10 border-cyan-500/20"),
                        "flagged"   => ("text-amber-400",   "bg-amber-500/10 border-amber-500/20"),
                        "hidden"    => ("text-red-400",     "bg-red-500/10 border-red-500/20"),
                        _           => ("text-slate-400",   "bg-slate-500/10 border-slate-500/20"),
                    };

                    let left_accent = match post.status {
                        "published" => "border-l-emerald-500/60",
                        "pinned"    => "border-l-cyan-500",
                        "flagged"   => "border-l-amber-500",
                        "hidden"    => "border-l-red-500",
                        _           => "border-l-slate-700",
                    };

                    let row_opacity = match post.status {
                        "hidden" => "opacity-50",
                        _        => "opacity-100",
                    };

                    let engagement = format!(
                        "\u{2764} {} \u{00B7} \u{1F4AC} {}",
                        post.likes, post.replies
                    );

                    let engagement_cls = if post.likes >= 30 {
                        "text-emerald-400"
                    } else if post.likes >= 10 {
                        "text-slate-300"
                    } else {
                        "text-slate-500"
                    };

                    /* Action buttons differ by current status */
                    let hide_label = match post.status {
                        "hidden" => "Unhide",
                        _        => "Hide",
                    };
                    let flag_label = match post.status {
                        "flagged" => "Unflag",
                        _         => "Flag",
                    };
                    let pin_label = match post.status {
                        "pinned" => "Unpin",
                        _        => "Pin",
                    };

                    view! {
                        <div class=format!(
                            "rounded-xl border border-slate-800 bg-slate-900/50 p-4 border-l-2 {left_accent} {row_opacity} \
                             hover:bg-slate-900/80 transition-colors group"
                        )>
                            /* Desktop: grid layout */
                            <div class="hidden sm:grid grid-cols-12 gap-4 items-center">
                                /* Status badge */
                                <div class="col-span-1">
                                    <span class=format!(
                                        "inline-block rounded-full border px-2 py-0.5 text-[9px] font-bold font-mono uppercase {status_bg_cls} {status_text_cls}"
                                    )>{post.status}</span>
                                </div>

                                /* Author */
                                <div class="col-span-2">
                                    <p class="text-sm text-slate-200 font-medium truncate">{post.author}</p>
                                </div>

                                /* Circle */
                                <div class="col-span-2">
                                    <span class="rounded-md bg-violet-500/10 border border-violet-500/20 px-2 py-0.5 text-[10px] font-mono text-violet-400 uppercase tracking-wide">
                                        {post.circle}
                                    </span>
                                </div>

                                /* Preview */
                                <div class="col-span-3">
                                    <p class="text-xs text-slate-400 truncate">{post.preview}</p>
                                </div>

                                /* Engagement */
                                <div class="col-span-1 text-right">
                                    <span class=format!("text-[10px] font-mono {engagement_cls}")>{engagement.clone()}</span>
                                </div>

                                /* Time */
                                <div class="col-span-1 text-right">
                                    <span class="text-[10px] text-slate-600 font-mono">{post.time_ago}</span>
                                </div>

                                /* Actions */
                                <div class="col-span-2 flex gap-1.5 justify-end">
                                    <button class="rounded border border-red-500/30 bg-red-500/5 px-2 py-1 text-[9px] font-bold text-red-400 hover:bg-red-500/20 transition-colors uppercase font-mono">
                                        {hide_label}
                                    </button>
                                    <button class="rounded border border-amber-500/30 bg-amber-500/5 px-2 py-1 text-[9px] font-bold text-amber-400 hover:bg-amber-500/20 transition-colors uppercase font-mono">
                                        {flag_label}
                                    </button>
                                    <button class="rounded border border-cyan-500/30 bg-cyan-500/5 px-2 py-1 text-[9px] font-bold text-cyan-400 hover:bg-cyan-500/20 transition-colors uppercase font-mono">
                                        {pin_label}
                                    </button>
                                </div>
                            </div>

                            /* Mobile: stacked layout */
                            <div class="sm:hidden space-y-2">
                                <div class="flex items-center gap-2 flex-wrap">
                                    <span class=format!(
                                        "rounded-full border px-2 py-0.5 text-[9px] font-bold font-mono uppercase {status_bg_cls} {status_text_cls}"
                                    )>{post.status}</span>
                                    <span class="rounded-md bg-violet-500/10 border border-violet-500/20 px-2 py-0.5 text-[10px] font-mono text-violet-400 uppercase tracking-wide">
                                        {post.circle}
                                    </span>
                                    <span class="text-[10px] text-slate-600 font-mono">{post.time_ago}</span>
                                </div>
                                <p class="text-xs text-slate-400 line-clamp-2">{post.preview}</p>
                                <div class="flex items-center justify-between">
                                    <p class="text-[10px] text-slate-500 font-mono">
                                        "By " <span class="text-slate-300">{post.author}</span>
                                        {format!(" \u{00B7} {}", engagement)}
                                    </p>
                                    <div class="flex gap-1">
                                        <button class="rounded border border-slate-700 px-2 py-0.5 text-[9px] font-bold text-slate-400 hover:text-white transition-colors uppercase font-mono">
                                            {hide_label}
                                        </button>
                                        <button class="rounded border border-slate-700 px-2 py-0.5 text-[9px] font-bold text-slate-400 hover:text-white transition-colors uppercase font-mono">
                                            {flag_label}
                                        </button>
                                        <button class="rounded border border-slate-700 px-2 py-0.5 text-[9px] font-bold text-slate-400 hover:text-white transition-colors uppercase font-mono">
                                            {pin_label}
                                        </button>
                                    </div>
                                </div>
                            </div>
                        </div>
                    }
                }).collect_view()}
            </div>

            /* Footer */
            <div class="mt-6 rounded-xl border border-slate-800 bg-slate-900/50 px-5 py-3 flex items-center justify-between">
                <p class="text-[10px] text-slate-500 font-mono uppercase tracking-widest">
                    {format!("Showing {} posts \u{00B7} {} published \u{00B7} {} pinned \u{00B7} {} flagged \u{00B7} {} hidden",
                        total, published, pinned, flagged, hidden)}
                </p>
                <div class="flex gap-2">
                    <button class="rounded border border-slate-700 px-3 py-1 text-[9px] font-bold text-slate-500 hover:text-white transition-colors uppercase font-mono">"Export CSV"</button>
                    <button class="rounded border border-slate-700 px-3 py-1 text-[9px] font-bold text-slate-500 hover:text-white transition-colors uppercase font-mono">"Bulk Actions"</button>
                </div>
            </div>
        </div>
    }
}
