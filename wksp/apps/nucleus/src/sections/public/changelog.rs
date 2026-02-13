//! Changelog — platform release notes and updates

use leptos::prelude::*;

#[component]
pub fn ChangelogPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-3xl px-4 py-16">
            <h1 class="text-4xl font-bold text-white">"Changelog"</h1>
            <p class="mt-2 text-slate-400">"What's new in Nucleus — release notes, improvements, and fixes."</p>

            <div class="mt-8 space-y-6">
                <ChangelogEntry
                    version="0.1.0"
                    date="February 2026"
                    items=vec![
                        "Initial Nucleus launch with Academy, Community, and Careers sections",
                        "Signal detection dashboard with PRR, ROR, IC, EBGM metrics",
                        "Firebase authentication with email and Google sign-in",
                        "15-domain KSB taxonomy browser",
                    ]
                />
            </div>
        </div>
    }
}

#[component]
fn ChangelogEntry(
    version: &'static str,
    date: &'static str,
    items: Vec<&'static str>,
) -> impl IntoView {
    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
            <div class="flex items-baseline gap-3">
                <span class="rounded-full bg-cyan-500/10 px-3 py-1 text-sm font-semibold text-cyan-400">
                    {format!("v{version}")}
                </span>
                <span class="text-sm text-slate-500">{date}</span>
            </div>
            <ul class="mt-4 space-y-2">
                {items.into_iter().map(|item| view! {
                    <li class="flex items-start gap-2 text-sm text-slate-300">
                        <span class="mt-1.5 h-1.5 w-1.5 shrink-0 rounded-full bg-cyan-400"></span>
                        {item}
                    </li>
                }).collect_view()}
            </ul>
        </div>
    }
}
