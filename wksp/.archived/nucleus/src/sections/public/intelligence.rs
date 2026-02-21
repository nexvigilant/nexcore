//! Intelligence hub — articles, series, insights

use leptos::prelude::*;

/// Articles listing page
#[component]
pub fn ArticlesPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-16">
            <h1 class="text-4xl font-bold text-white">"Intelligence"</h1>
            <p class="mt-4 text-lg text-slate-400">"Insights and analysis from the NexVigilant intelligence team"</p>

            <div class="mt-8 flex gap-4">
                <button class="rounded-lg bg-cyan-600 px-4 py-2 text-sm font-medium text-white">"All"</button>
                <button class="rounded-lg border border-slate-700 px-4 py-2 text-sm text-slate-400 hover:text-white transition-colors">"Safety Signals"</button>
                <button class="rounded-lg border border-slate-700 px-4 py-2 text-sm text-slate-400 hover:text-white transition-colors">"Industry"</button>
                <button class="rounded-lg border border-slate-700 px-4 py-2 text-sm text-slate-400 hover:text-white transition-colors">"Education"</button>
            </div>

            <div class="mt-8 grid gap-6 md:grid-cols-2 lg:grid-cols-3">
                <ArticleCard
                    title="Understanding PRR Signal Detection"
                    excerpt="A deep dive into Proportional Reporting Ratios and their role in pharmacovigilance."
                    category="Education"
                    date="Feb 2026"
                />
                <ArticleCard
                    title="Q4 2025 Safety Signal Summary"
                    excerpt="Key safety signals identified across major therapeutic areas."
                    category="Safety Signals"
                    date="Jan 2026"
                />
                <ArticleCard
                    title="The Future of PV Technology"
                    excerpt="How AI and automation are transforming drug safety monitoring."
                    category="Industry"
                    date="Jan 2026"
                />
            </div>
        </div>
    }
}

#[component]
fn ArticleCard(
    title: &'static str,
    excerpt: &'static str,
    category: &'static str,
    date: &'static str,
) -> impl IntoView {
    view! {
        <article class="rounded-xl border border-slate-800 bg-slate-900/50 p-6 hover:border-slate-700 transition-colors">
            <div class="flex items-center gap-2 text-xs">
                <span class="rounded-full bg-cyan-500/10 px-2.5 py-0.5 text-cyan-400">{category}</span>
                <span class="text-slate-500">{date}</span>
            </div>
            <h3 class="mt-3 text-lg font-semibold text-white">{title}</h3>
            <p class="mt-2 text-sm text-slate-400 line-clamp-2">{excerpt}</p>
            <a href="/intelligence/article" class="mt-4 inline-block text-sm text-cyan-400 hover:text-cyan-300 transition-colors">"Read more"</a>
        </article>
    }
}

/// Single article detail page
#[component]
pub fn ArticleDetailPage() -> impl IntoView {
    view! {
        <article class="mx-auto max-w-3xl px-4 py-16">
            <a href="/intelligence" class="text-sm text-cyan-400 hover:text-cyan-300 transition-colors">"< Back to Intelligence"</a>
            <h1 class="mt-6 text-4xl font-bold text-white">"Article Title"</h1>
            <div class="mt-4 flex items-center gap-4 text-sm text-slate-400">
                <span>"NexVigilant Team"</span>
                <span>"February 8, 2026"</span>
                <span class="rounded-full bg-cyan-500/10 px-2.5 py-0.5 text-cyan-400">"Education"</span>
            </div>
            <div class="prose prose-invert mt-8 max-w-none">
                <p class="text-slate-300">"Article content will be loaded from Firestore and rendered here."</p>
            </div>
        </article>
    }
}

/// Series listing page
#[component]
pub fn SeriesPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-16">
            <h1 class="text-4xl font-bold text-white">"Series"</h1>
            <p class="mt-4 text-lg text-slate-400">"Multi-part deep dives into critical topics"</p>

            <div class="mt-8 grid gap-6 md:grid-cols-2">
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-8">
                    <span class="text-xs font-medium uppercase tracking-wider text-cyan-400">"5 Parts"</span>
                    <h3 class="mt-2 text-xl font-bold text-white">"Foundations of Pharmacovigilance"</h3>
                    <p class="mt-2 text-slate-400">"From adverse event reporting to signal detection — a comprehensive introduction."</p>
                    <a href="/intelligence/series/foundations" class="mt-4 inline-block text-sm text-cyan-400 hover:text-cyan-300 transition-colors">"View series"</a>
                </div>
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-8">
                    <span class="text-xs font-medium uppercase tracking-wider text-amber-400">"3 Parts"</span>
                    <h3 class="mt-2 text-xl font-bold text-white">"Career Transitions in Drug Safety"</h3>
                    <p class="mt-2 text-slate-400">"Navigate your path from clinical practice to pharmacovigilance."</p>
                    <a href="/intelligence/series/career-transitions" class="mt-4 inline-block text-sm text-cyan-400 hover:text-cyan-300 transition-colors">"View series"</a>
                </div>
            </div>
        </div>
    }
}
