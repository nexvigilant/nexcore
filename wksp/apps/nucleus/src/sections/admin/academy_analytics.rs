//! Admin: Academy analytics dashboard

use leptos::prelude::*;

#[component]
pub fn AcademyAnalyticsPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white">"Academy Analytics"</h1>
            <p class="mt-1 text-slate-400">"Enrollment trends, completion rates, and learning insights."</p>

            <div class="mt-6 grid gap-4 md:grid-cols-4">
                <MetricCard label="Total Enrollments" value="0"/>
                <MetricCard label="Active Learners" value="0"/>
                <MetricCard label="Completion Rate" value="—"/>
                <MetricCard label="Avg. Score" value="—"/>
            </div>

            <div class="mt-8 rounded-xl border border-slate-800 bg-slate-900/50 p-8 text-center">
                <p class="text-slate-500">"Analytics charts will appear once learners begin enrolling in courses."</p>
            </div>
        </div>
    }
}

#[component]
fn MetricCard(label: &'static str, value: &'static str) -> impl IntoView {
    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-4 text-center">
            <div class="text-2xl font-bold text-white">{value}</div>
            <div class="mt-1 text-xs text-slate-500">{label}</div>
        </div>
    }
}
