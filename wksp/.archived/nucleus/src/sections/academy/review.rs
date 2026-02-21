//! Academy review — spaced repetition review queue

use leptos::prelude::*;

#[component]
pub fn ReviewPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-3xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white">"Review Queue"</h1>
            <p class="mt-2 text-slate-400">"Spaced repetition review of learned concepts to reinforce retention."</p>

            <div class="mt-8 rounded-xl border border-slate-800 bg-slate-900/50 p-8 text-center">
                <p class="text-lg font-medium text-white">"No items due for review"</p>
                <p class="mt-2 text-sm text-slate-500">"Complete course lessons to build your review queue. Items appear based on spaced repetition intervals."</p>
                <a href="/academy/courses" class="mt-4 inline-block text-sm text-cyan-400 hover:text-cyan-300">"Browse Courses"</a>
            </div>

            <div class="mt-8">
                <h2 class="text-lg font-semibold text-white">"Review Stats"</h2>
                <div class="mt-4 grid grid-cols-3 gap-4">
                    <StatBox label="Due Today" value="0"/>
                    <StatBox label="Reviewed This Week" value="0"/>
                    <StatBox label="Retention Rate" value="—"/>
                </div>
            </div>
        </div>
    }
}

#[component]
fn StatBox(label: &'static str, value: &'static str) -> impl IntoView {
    view! {
        <div class="rounded-lg border border-slate-800 bg-slate-900/50 p-4 text-center">
            <div class="text-2xl font-bold text-white">{value}</div>
            <div class="mt-1 text-xs text-slate-500">{label}</div>
        </div>
    }
}
