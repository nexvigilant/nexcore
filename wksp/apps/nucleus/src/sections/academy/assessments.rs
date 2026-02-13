//! Academy assessments — knowledge checks and skill validation

use leptos::prelude::*;

#[component]
pub fn AssessmentsPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white">"Assessments"</h1>
            <p class="mt-2 text-slate-400">"Validate your knowledge across PV domains with structured assessments."</p>

            <div class="mt-8 grid gap-4 md:grid-cols-2">
                <AssessmentItem title="Domain Knowledge Check" questions=25 duration="20 min" status="Available"/>
                <AssessmentItem title="Signal Detection Practical" questions=10 duration="30 min" status="Available"/>
                <AssessmentItem title="Regulatory Framework Quiz" questions=30 duration="25 min" status="Locked"/>
                <AssessmentItem title="ICSR Processing Simulation" questions=5 duration="45 min" status="Locked"/>
                <AssessmentItem title="Benefit-Risk Assessment" questions=15 duration="35 min" status="Coming Soon"/>
                <AssessmentItem title="Comprehensive Final Exam" questions=50 duration="60 min" status="Coming Soon"/>
            </div>
        </div>
    }
}

#[component]
fn AssessmentItem(
    title: &'static str,
    questions: u32,
    duration: &'static str,
    status: &'static str,
) -> impl IntoView {
    let status_class = match status {
        "Available" => "text-emerald-400 bg-emerald-500/10",
        "Locked" => "text-amber-400 bg-amber-500/10",
        _ => "text-slate-400 bg-slate-800",
    };

    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5">
            <div class="flex items-center justify-between">
                <h3 class="font-semibold text-white">{title}</h3>
                <span class=format!("rounded-full px-2.5 py-0.5 text-xs font-medium {status_class}")>{status}</span>
            </div>
            <div class="mt-2 flex gap-4 text-xs text-slate-500">
                <span>{format!("{questions} questions")}</span>
                <span>{duration}</span>
            </div>
        </div>
    }
}
