//! Discover results — filtered content from the discover flow
//!
//! Shows trending topics, popular posts, and recommended circles
//! based on query parameters from the Discover page search.

use leptos::prelude::*;

/* ── Static result data ─────────────────────────────── */

struct TrendingTopic {
    name: &'static str,
    post_count: u32,
    growth_pct: f32,
}

struct PopularPost {
    author: &'static str,
    role: &'static str,
    excerpt: &'static str,
    likes: u32,
    replies: u32,
    tag: &'static str,
}

struct RecommendedCircle {
    name: &'static str,
    description: &'static str,
    members: u32,
    category: &'static str,
}

const TRENDING: &[TrendingTopic] = &[
    TrendingTopic {
        name: "AI-Assisted Signal Detection",
        post_count: 47,
        growth_pct: 340.0,
    },
    TrendingTopic {
        name: "PBRER Writing Best Practices",
        post_count: 31,
        growth_pct: 180.0,
    },
    TrendingTopic {
        name: "MedDRA 27.0 Migration",
        post_count: 28,
        growth_pct: 220.0,
    },
    TrendingTopic {
        name: "Real-World Evidence in PV",
        post_count: 24,
        growth_pct: 150.0,
    },
    TrendingTopic {
        name: "EU GVP Module XVI Update",
        post_count: 19,
        growth_pct: 95.0,
    },
    TrendingTopic {
        name: "Patient-Reported Outcomes",
        post_count: 16,
        growth_pct: 88.0,
    },
];

const POSTS: &[PopularPost] = &[
    PopularPost {
        author: "Dr. Elena Vasquez",
        role: "Head of Signal Management",
        excerpt: "Our team reduced false-positive signal rates by 62% using a hybrid PRR-IC025 scoring model combined with automated MedDRA grouping. Here's the exact workflow we used...",
        likes: 89,
        replies: 23,
        tag: "Signal Detection",
    },
    PopularPost {
        author: "James Okonkwo",
        role: "Senior PV Scientist",
        excerpt: "After implementing structured benefit-risk assessment using the PrOACT-URL framework, our PRAC submissions now take 40% less time. The key was standardizing the value tree...",
        likes: 67,
        replies: 18,
        tag: "Benefit-Risk",
    },
    PopularPost {
        author: "Dr. Mei-Lin Chen",
        role: "QPPV Deputy",
        excerpt: "Navigating the MHRA post-Brexit divergence from EMA: a practical guide for dual-regulated MAHs. Includes our SOP template for parallel submissions...",
        likes: 54,
        replies: 31,
        tag: "Regulatory",
    },
    PopularPost {
        author: "Aisha Patel",
        role: "PV Data Engineer",
        excerpt: "We built a real-time ICSR deduplication pipeline using fuzzy matching on patient demographics and narrative similarity. Reduced duplicate rate from 8.3% to 0.4%...",
        likes: 45,
        replies: 12,
        tag: "Automation",
    },
    PopularPost {
        author: "Dr. Thomas Richter",
        role: "Medical Director",
        excerpt: "Case study: How we identified a novel hepatotoxicity signal for a rare disease drug with only 2,400 patients exposed. The power of combining FAERS mining with company data...",
        likes: 38,
        replies: 9,
        tag: "Case Study",
    },
];

const CIRCLES: &[RecommendedCircle] = &[
    RecommendedCircle {
        name: "Signal Hunters",
        description: "Collaborative disproportionality analysis, emerging safety signals, and detection methodology.",
        members: 342,
        category: "Analytics",
    },
    RecommendedCircle {
        name: "Regulatory Roundtable",
        description: "ICH guidelines, PSMF updates, and global regulatory submission strategies.",
        members: 218,
        category: "Regulatory",
    },
    RecommendedCircle {
        name: "AI & Automation in PV",
        description: "NLP for case processing, LLM safety assessment, and workflow automation.",
        members: 187,
        category: "Technology",
    },
    RecommendedCircle {
        name: "Aggregate Reporting Hub",
        description: "PSUR/PBRER authoring, cumulative reviews, and additive analyses best practices.",
        members: 156,
        category: "Reporting",
    },
    RecommendedCircle {
        name: "QPPV Network",
        description: "Challenges and strategies for Qualified Persons responsible for Pharmacovigilance.",
        members: 134,
        category: "Leadership",
    },
    RecommendedCircle {
        name: "Clinical Safety Liaison",
        description: "Bridge between clinical development safety and post-marketing pharmacovigilance.",
        members: 112,
        category: "Clinical",
    },
];

/* ── Page component ─────────────────────────────────── */

#[component]
pub fn DiscoverResultsPage() -> impl IntoView {
    let active_section = RwSignal::new("trending");

    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            /* Header */
            <div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between mb-8">
                <div>
                    <div class="flex items-center gap-3 mb-2">
                        <a href="/community/discover" class="text-slate-500 hover:text-white transition-colors text-sm font-mono">
                            {"\u{2190} Discover"}
                        </a>
                    </div>
                    <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Discover Results"</h1>
                    <p class="mt-2 text-slate-400">"Explore trending topics, top posts, and recommended circles"</p>
                </div>
                <a
                    href="/community/discover/matches"
                    class="rounded-lg border border-cyan-500/30 bg-cyan-500/10 px-5 py-2.5 text-sm font-bold text-cyan-400 hover:bg-cyan-500/20 transition-colors font-mono uppercase tracking-widest"
                >
                    "View My Matches"
                </a>
            </div>

            /* Section tabs */
            <div class="flex gap-2 mb-8 border-b border-slate-800 pb-3">
                <SectionTab label="Trending" key="trending" active=active_section />
                <SectionTab label="Top Posts" key="posts" active=active_section />
                <SectionTab label="Circles" key="circles" active=active_section />
            </div>

            /* Content area */
            {move || match active_section.get() {
                "posts" => view! { <TopPostsSection /> }.into_any(),
                "circles" => view! { <CirclesSection /> }.into_any(),
                _ => view! { <TrendingSection /> }.into_any(),
            }}
        </div>
    }
}

#[component]
fn SectionTab(
    label: &'static str,
    key: &'static str,
    active: RwSignal<&'static str>,
) -> impl IntoView {
    view! {
        <button
            on:click=move |_| active.set(key)
            class=move || if active.get() == key {
                "rounded-t-lg px-5 py-2 text-xs font-bold text-cyan-400 border-b-2 border-cyan-400 font-mono uppercase tracking-widest"
            } else {
                "rounded-t-lg px-5 py-2 text-xs font-medium text-slate-500 hover:text-slate-300 font-mono uppercase tracking-widest transition-colors"
            }
        >
            {label}
        </button>
    }
}

/* ── Trending section ───────────────────────────────── */

#[component]
fn TrendingSection() -> impl IntoView {
    view! {
        <div class="space-y-4">
            <div class="flex items-center justify-between mb-2">
                <h2 class="text-[10px] font-bold text-slate-500 uppercase tracking-widest font-mono">"Trending This Week"</h2>
                <span class="text-[10px] text-slate-600 font-mono">{format!("{} topics", TRENDING.len())}</span>
            </div>
            {TRENDING.iter().enumerate().map(|(i, topic)| {
                let rank = i + 1;
                let growth_color = if topic.growth_pct > 200.0 {
                    "text-red-400"
                } else if topic.growth_pct > 100.0 {
                    "text-amber-400"
                } else {
                    "text-emerald-400"
                };
                view! {
                    <div class="flex items-center gap-4 rounded-xl border border-slate-800 bg-slate-900/50 p-5 hover:border-slate-700 transition-colors group">
                        <span class="text-2xl font-black text-slate-700 font-mono w-8 text-right">{format!("{:02}", rank)}</span>
                        <div class="flex-1 min-w-0">
                            <h3 class="text-sm font-bold text-white group-hover:text-cyan-400 transition-colors">{topic.name}</h3>
                            <p class="mt-1 text-[10px] text-slate-500 font-mono">
                                {format!("{} posts this week", topic.post_count)}
                            </p>
                        </div>
                        <div class="text-right">
                            <span class=format!("text-sm font-bold font-mono {growth_color}")>
                                {format!("+{:.0}%", topic.growth_pct)}
                            </span>
                            <p class="text-[9px] text-slate-600 font-mono">"growth"</p>
                        </div>
                    </div>
                }
            }).collect_view()}
        </div>
    }
}

/* ── Top posts section ──────────────────────────────── */

#[component]
fn TopPostsSection() -> impl IntoView {
    view! {
        <div class="space-y-4">
            <div class="flex items-center justify-between mb-2">
                <h2 class="text-[10px] font-bold text-slate-500 uppercase tracking-widest font-mono">"Most Engaged Posts"</h2>
                <span class="text-[10px] text-slate-600 font-mono">{format!("{} posts", POSTS.len())}</span>
            </div>
            {POSTS.iter().map(|post| {
                let initial = post.author.chars().next().unwrap_or('?').to_uppercase().to_string();
                let tag_color = match post.tag {
                    "Signal Detection" => "text-red-400 bg-red-500/10 border-red-500/20",
                    "Benefit-Risk" => "text-amber-400 bg-amber-500/10 border-amber-500/20",
                    "Regulatory" => "text-violet-400 bg-violet-500/10 border-violet-500/20",
                    "Automation" => "text-cyan-400 bg-cyan-500/10 border-cyan-500/20",
                    "Case Study" => "text-emerald-400 bg-emerald-500/10 border-emerald-500/20",
                    _ => "text-slate-400 bg-slate-500/10 border-slate-500/20",
                };
                view! {
                    <article class="rounded-xl border border-slate-800 bg-slate-900/50 p-5 hover:border-slate-700 transition-colors">
                        <div class="flex items-center gap-3 mb-3">
                            <div class="h-9 w-9 rounded-full bg-cyan-500/10 flex items-center justify-center text-xs font-bold text-cyan-400 shrink-0">
                                {initial}
                            </div>
                            <div class="min-w-0 flex-1">
                                <p class="text-sm font-bold text-white truncate">{post.author}</p>
                                <p class="text-[10px] text-slate-500 font-mono uppercase tracking-widest">{post.role}</p>
                            </div>
                            <span class=format!("rounded-full border px-2.5 py-0.5 text-[9px] font-bold font-mono uppercase {tag_color}")>
                                {post.tag}
                            </span>
                        </div>
                        <p class="text-sm text-slate-300 leading-relaxed">{post.excerpt}</p>
                        <div class="mt-4 flex items-center gap-6 text-[10px] font-bold text-slate-500 font-mono uppercase tracking-widest">
                            <span class="flex items-center gap-1.5">
                                <svg class="h-3.5 w-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z" />
                                </svg>
                                {post.likes.to_string()}
                            </span>
                            <span class="flex items-center gap-1.5">
                                <svg class="h-3.5 w-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
                                </svg>
                                {post.replies.to_string()}
                            </span>
                        </div>
                    </article>
                }
            }).collect_view()}
        </div>
    }
}

/* ── Circles section ────────────────────────────────── */

#[component]
fn CirclesSection() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between mb-2">
                <h2 class="text-[10px] font-bold text-slate-500 uppercase tracking-widest font-mono">"Recommended Circles"</h2>
                <a href="/community/circles" class="text-[10px] text-cyan-400 hover:text-cyan-300 font-mono uppercase tracking-widest transition-colors">
                    "Browse All"
                </a>
            </div>
            <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                {CIRCLES.iter().map(|circle| {
                    let initial = circle.name.chars().next().unwrap_or('?').to_string();
                    let cat_color = match circle.category {
                        "Analytics" => "text-red-400 bg-red-500/10",
                        "Regulatory" => "text-violet-400 bg-violet-500/10",
                        "Technology" => "text-cyan-400 bg-cyan-500/10",
                        "Reporting" => "text-amber-400 bg-amber-500/10",
                        "Leadership" => "text-emerald-400 bg-emerald-500/10",
                        "Clinical" => "text-rose-400 bg-rose-500/10",
                        _ => "text-slate-400 bg-slate-500/10",
                    };
                    view! {
                        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5 hover:border-slate-700 transition-colors group">
                            <div class="flex items-center gap-3 mb-3">
                                <div class="h-10 w-10 rounded-full bg-violet-500/10 flex items-center justify-center text-sm font-bold text-violet-400 shrink-0">
                                    {initial}
                                </div>
                                <div class="min-w-0 flex-1">
                                    <h3 class="text-sm font-bold text-white group-hover:text-cyan-400 transition-colors truncate">
                                        {circle.name}
                                    </h3>
                                    <span class=format!("text-[9px] font-bold font-mono uppercase rounded-full px-2 py-0.5 {cat_color}")>
                                        {circle.category}
                                    </span>
                                </div>
                            </div>
                            <p class="text-xs text-slate-400 leading-relaxed mb-3">{circle.description}</p>
                            <div class="flex items-center justify-between">
                                <span class="text-[10px] text-slate-600 font-mono">{format!("{} members", circle.members)}</span>
                                <button class="rounded-md border border-slate-700 px-3 py-1 text-[10px] font-bold text-slate-400 hover:border-cyan-500 hover:text-cyan-400 transition-colors uppercase tracking-wider font-mono">
                                    "Join"
                                </button>
                            </div>
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}
