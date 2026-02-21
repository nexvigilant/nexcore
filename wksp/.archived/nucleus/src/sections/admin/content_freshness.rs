/*! Admin: Content freshness monitoring and review scheduling */

use leptos::prelude::*;

struct ContentItem {
    title: &'static str,
    content_type: &'static str,
    last_updated: &'static str,
    age_days: u32,
    author: &'static str,
    freshness: &'static str,
}

const CONTENT: &[ContentItem] = &[
    ContentItem {
        title: "Introduction to Pharmacovigilance",
        content_type: "Course Module",
        last_updated: "2026-01-28",
        age_days: 18,
        author: "Dr. Sarah Mitchell",
        freshness: "Fresh",
    },
    ContentItem {
        title: "GVP Module IX Signal Management",
        content_type: "Course Module",
        last_updated: "2026-01-14",
        age_days: 32,
        author: "Dr. Mei-Lin Chen",
        freshness: "Fresh",
    },
    ContentItem {
        title: "Benefit-Risk Framework Guide",
        content_type: "Article",
        last_updated: "2026-01-23",
        age_days: 23,
        author: "Dr. Ahmed Hassan",
        freshness: "Fresh",
    },
    ContentItem {
        title: "Naranjo Causality Assessment Guide",
        content_type: "Reference",
        last_updated: "2025-12-10",
        age_days: 67,
        author: "Dr. Ahmed Hassan",
        freshness: "Fresh",
    },
    ContentItem {
        title: "PBRER Authoring Handbook",
        content_type: "Course Module",
        last_updated: "2025-11-18",
        age_days: 89,
        author: "James Okonkwo",
        freshness: "Fresh",
    },
    ContentItem {
        title: "EU Regulatory Framework Overview",
        content_type: "Article",
        last_updated: "2025-09-26",
        age_days: 142,
        author: "Dr. Thomas Richter",
        freshness: "Review Due",
    },
    ContentItem {
        title: "Signal Detection Methodology",
        content_type: "Assessment",
        last_updated: "2025-09-12",
        age_days: 156,
        author: "Dr. Elena Vasquez",
        freshness: "Review Due",
    },
    ContentItem {
        title: "ICSR Processing Guidelines",
        content_type: "Reference",
        last_updated: "2025-08-22",
        age_days: 178,
        author: "Aisha Patel",
        freshness: "Review Due",
    },
    ContentItem {
        title: "Risk Minimization Measures",
        content_type: "Article",
        last_updated: "2025-08-01",
        age_days: 198,
        author: "Sarah Williams",
        freshness: "Stale",
    },
    ContentItem {
        title: "MedDRA Coding Best Practices",
        content_type: "Assessment",
        last_updated: "2025-07-20",
        age_days: 210,
        author: "James Okonkwo",
        freshness: "Stale",
    },
];

fn freshness_badge_cls(freshness: &str) -> &'static str {
    match freshness {
        "Fresh" => "text-emerald-400 bg-emerald-500/10 border-emerald-500/20",
        "Review Due" => "text-amber-400 bg-amber-500/10 border-amber-500/20",
        _ => "text-red-400 bg-red-500/10 border-red-500/20",
    }
}

fn age_color_cls(age: u32) -> &'static str {
    if age >= 180 {
        "text-red-400"
    } else if age >= 90 {
        "text-amber-400"
    } else {
        "text-emerald-400"
    }
}

fn type_badge_cls(content_type: &str) -> &'static str {
    match content_type {
        "Course Module" => "text-cyan-400 bg-cyan-500/10 border-cyan-500/20",
        "Article" => "text-violet-400 bg-violet-500/10 border-violet-500/20",
        "Assessment" => "text-orange-400 bg-orange-500/10 border-orange-500/20",
        _ => "text-slate-400 bg-slate-500/10 border-slate-500/20",
    }
}

#[component]
pub fn ContentFreshnessPage() -> impl IntoView {
    let fresh_count = CONTENT.iter().filter(|c| c.freshness == "Fresh").count();
    let review_count = CONTENT
        .iter()
        .filter(|c| c.freshness == "Review Due")
        .count();
    let stale_count = CONTENT.iter().filter(|c| c.freshness == "Stale").count();
    let total_age: u32 = CONTENT.iter().map(|c| c.age_days).sum();
    let avg_age = if CONTENT.is_empty() {
        0
    } else {
        total_age / CONTENT.len() as u32
    };

    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            /* Header */
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">
                        "Content Freshness"
                    </h1>
                    <p class="mt-1 text-slate-400">
                        "Monitor content age, schedule reviews, and flag stale material across all learning resources."
                    </p>
                </div>
                <a
                    href="/admin/content"
                    class="text-sm text-slate-400 hover:text-white transition-colors font-mono"
                >
                    "\u{2190} Content Admin"
                </a>
            </div>

            /* Stats row */
            <div class="mt-6 grid gap-4 sm:grid-cols-4">
                <div class="rounded-xl border border-emerald-500/20 bg-emerald-500/5 p-5">
                    <p class="text-[9px] font-bold text-emerald-400 uppercase tracking-widest font-mono">
                        "Fresh"
                    </p>
                    <p class="text-2xl font-black text-emerald-400 font-mono mt-2">
                        {fresh_count.to_string()}
                    </p>
                    <p class="text-[10px] text-emerald-500/60 font-mono mt-1">"< 90 days old"</p>
                </div>
                <div class="rounded-xl border border-amber-500/20 bg-amber-500/5 p-5">
                    <p class="text-[9px] font-bold text-amber-400 uppercase tracking-widest font-mono">
                        "Review Due"
                    </p>
                    <p class="text-2xl font-black text-amber-400 font-mono mt-2">
                        {review_count.to_string()}
                    </p>
                    <p class="text-[10px] text-amber-500/60 font-mono mt-1">"90\u{2013}179 days old"</p>
                </div>
                <div class="rounded-xl border border-red-500/20 bg-red-500/5 p-5">
                    <p class="text-[9px] font-bold text-red-400 uppercase tracking-widest font-mono">
                        "Stale"
                    </p>
                    <p class="text-2xl font-black text-red-400 font-mono mt-2">
                        {stale_count.to_string()}
                    </p>
                    <p class="text-[10px] text-red-500/60 font-mono mt-1">"180+ days old"</p>
                </div>
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5">
                    <p class="text-[9px] font-bold text-slate-500 uppercase tracking-widest font-mono">
                        "Avg Age"
                    </p>
                    <p class="text-2xl font-black text-white font-mono mt-2">
                        {format!("{}d", avg_age)}
                    </p>
                    <p class="text-[10px] text-slate-600 font-mono mt-1">
                        {format!("across {} items", CONTENT.len())}
                    </p>
                </div>
            </div>

            /* Data table */
            <div class="mt-8 rounded-xl border border-slate-800 overflow-hidden">
                <table class="w-full text-left text-sm">
                    <thead class="bg-slate-900/80 text-[10px] uppercase text-slate-500 font-mono tracking-widest">
                        <tr>
                            <th class="px-4 py-3">"Title"</th>
                            <th class="px-4 py-3">"Type"</th>
                            <th class="px-4 py-3">"Last Updated"</th>
                            <th class="px-4 py-3 text-right">"Age"</th>
                            <th class="px-4 py-3">"Author"</th>
                            <th class="px-4 py-3">"Freshness"</th>
                            <th class="px-4 py-3 text-right">"Action"</th>
                        </tr>
                    </thead>
                    <tbody class="text-slate-300">
                        {CONTENT
                            .iter()
                            .map(|item| {
                                let badge_cls = freshness_badge_cls(item.freshness);
                                let age_cls = age_color_cls(item.age_days);
                                let type_cls = type_badge_cls(item.content_type);
                                let review_btn_cls = if item.freshness == "Fresh" {
                                    "rounded-lg border border-slate-700 bg-slate-800/50 px-3 py-1 text-[10px] font-bold text-slate-500 font-mono uppercase tracking-widest cursor-default"
                                } else {
                                    "rounded-lg border border-cyan-500/30 bg-cyan-500/10 px-3 py-1 text-[10px] font-bold text-cyan-400 font-mono uppercase tracking-widest hover:bg-cyan-500/20 transition-colors cursor-pointer"
                                };
                                view! {
                                    <tr class="border-t border-slate-800 hover:bg-slate-800/30 transition-colors">
                                        /* Title */
                                        <td class="px-4 py-3">
                                            <p class="text-sm font-medium text-white leading-tight">
                                                {item.title}
                                            </p>
                                        </td>
                                        /* Type badge */
                                        <td class="px-4 py-3">
                                            <span class=format!(
                                                "rounded-full border px-2 py-0.5 text-[9px] font-bold font-mono uppercase whitespace-nowrap {type_cls}",
                                            )>{item.content_type}</span>
                                        </td>
                                        /* Last updated */
                                        <td class="px-4 py-3 text-[10px] text-slate-500 font-mono whitespace-nowrap">
                                            {item.last_updated}
                                        </td>
                                        /* Age (color-coded) */
                                        <td class="px-4 py-3 text-right">
                                            <span class=format!(
                                                "text-xs font-bold font-mono tabular-nums {age_cls}",
                                            )>{format!("{}d", item.age_days)}</span>
                                        </td>
                                        /* Author */
                                        <td class="px-4 py-3">
                                            <span class="text-[10px] text-slate-500 font-mono whitespace-nowrap">
                                                {item.author}
                                            </span>
                                        </td>
                                        /* Freshness badge */
                                        <td class="px-4 py-3">
                                            <span class=format!(
                                                "rounded-full border px-2 py-0.5 text-[9px] font-bold font-mono uppercase whitespace-nowrap {badge_cls}",
                                            )>{item.freshness}</span>
                                        </td>
                                        /* Review action */
                                        <td class="px-4 py-3 text-right">
                                            <button class=review_btn_cls>"Review"</button>
                                        </td>
                                    </tr>
                                }
                            })
                            .collect_view()}
                    </tbody>
                </table>
            </div>

            /* Footer summary */
            <div class="mt-4 rounded-xl border border-slate-800 bg-slate-900/50 px-5 py-3">
                <div class="flex items-center justify-between text-[10px] text-slate-600 font-mono">
                    <div class="flex items-center gap-6">
                        <span>{format!("{} total items", CONTENT.len())}</span>
                        <span class="text-emerald-500/60">
                            {format!(
                                "\u{25CF} {} fresh ({}%)",
                                fresh_count,
                                if CONTENT.is_empty() { 0 } else { fresh_count * 100 / CONTENT.len() },
                            )}
                        </span>
                        <span class="text-amber-500/60">
                            {format!(
                                "\u{25CF} {} review due ({}%)",
                                review_count,
                                if CONTENT.is_empty() { 0 } else { review_count * 100 / CONTENT.len() },
                            )}
                        </span>
                        <span class="text-red-500/60">
                            {format!(
                                "\u{25CF} {} stale ({}%)",
                                stale_count,
                                if CONTENT.is_empty() { 0 } else { stale_count * 100 / CONTENT.len() },
                            )}
                        </span>
                    </div>
                    <span>
                        {format!(
                            "avg age: {}d \u{00B7} oldest: {}d \u{00B7} newest: {}d",
                            avg_age,
                            CONTENT.iter().map(|c| c.age_days).max().unwrap_or(0),
                            CONTENT.iter().map(|c| c.age_days).min().unwrap_or(0),
                        )}
                    </span>
                </div>
            </div>
        </div>
    }
}
